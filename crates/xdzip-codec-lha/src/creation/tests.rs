use super::*;
use crate::{LhaCodec, inspect};
use std::io::Cursor;
use xdzip_codec_api::{
    MetadataCodec, ScanLimits,
    payload::{DecodeError, DecodedSink, PayloadCodec},
};
struct Input {
    entries: Vec<EncodeEntry>,
    data: Vec<Vec<u8>>,
}
impl Input {
    fn new(items: &[(&str, bool, Vec<u8>)]) -> Self {
        Self {
            entries: items
                .iter()
                .map(|(name, directory, data)| EncodeEntry {
                    name: (*name).into(),
                    directory: *directory,
                    size: data.len() as u64,
                    modified_unix: 1700000000,
                })
                .collect(),
            data: items.iter().map(|(_, _, d)| d.clone()).collect(),
        }
    }
}
impl EncodeInput for Input {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        assert!(!self.entries[index].directory);
        Ok(Box::new(Cursor::new(&self.data[index])))
    }
}
fn settings(level: u8) -> EncodeSettings<'static> {
    EncodeSettings {
        method: "lh5",
        level,
        solid: false,
        encrypt_headers: false,
        password: None,
    }
}
fn create(input: &Input, level: u8) -> Vec<u8> {
    let mut out = vec![];
    encode(
        input,
        &mut out,
        settings(level),
        &Cancellation::default(),
        &mut |_| {},
    )
    .unwrap();
    out
}
#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}
#[test]
fn multi_member_header2_names_directories_empty_files_crc_and_cursor_restore_roundtrip() {
    let input = Input::new(&[
        ("nested", true, vec![]),
        ("nested/资料", true, vec![]),
        (
            "nested/资料/说明.txt",
            false,
            b"abracadabra abracadabra\n".repeat(4096),
        ),
        ("empty.bin", false, vec![]),
        ("last.txt", false, b"last".to_vec()),
    ]);
    let cancel = Cancellation::default();
    for level in [1, 6, 9] {
        let mut cursor = Cursor::new(vec![]);
        let mut seen = vec![];
        encode(&input, &mut cursor, settings(level), &cancel, &mut |p| {
            seen.push(p.input_bytes)
        })
        .unwrap();
        let bytes = cursor.into_inner();
        assert_eq!(bytes.last(), Some(&0));
        assert_eq!(
            seen.last(),
            Some(&input.data.iter().map(|b| b.len() as u64).sum())
        );
        let index = inspect(&mut Cursor::new(&bytes), &cancel, ScanLimits::default()).unwrap();
        assert_eq!(index.records.len(), 5);
        for record in &index.records {
            let at = record.header_offset as usize;
            assert_eq!(record.level, 2);
            assert_eq!(
                u32::from_le_bytes(bytes[at + 15..at + 19].try_into().unwrap()),
                1700000000
            );
        }
        let rows = LhaCodec
            .scan(&mut Cursor::new(&bytes), &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(
            rows[2].decoded_name.as_deref(),
            Some("nested/资料/说明.txt")
        );
        assert!(rows.iter().all(|r| !r.warning));
        let mut source = Cursor::new(&bytes);
        let mut reader = LhaCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        for (i, row) in rows.iter().enumerate().filter(|(_, r)| !r.directory) {
            let mut sink = Sink::default();
            reader.decode(row, &mut sink, &cancel).unwrap();
            assert_eq!(sink.0, input.data[i]);
            assert_eq!(row.method, 5);
        }
        assert!(rows[2].packed_size < rows[2].size / 10);
    }
    assert_eq!(create(&Input::new(&[]), 6), [0]);
}
#[test]
fn exact_256_boundary_padding_and_header_crc_are_preserved_after_payload_patch() {
    let name = format!("{}.bin", "x".repeat(218));
    let input = Input::new(&[(&name, false, b"hello".to_vec())]);
    let bytes = create(&input, 6);
    let index = inspect(
        &mut Cursor::new(&bytes),
        &Cancellation::default(),
        ScanLimits::default(),
    )
    .unwrap();
    assert_eq!(index.records[0].payload_offset, 257);
    assert_eq!(bytes[256], 0);
    let mut bad = bytes.clone();
    bad[27] ^= 1;
    assert!(
        inspect(
            &mut Cursor::new(bad),
            &Cancellation::default(),
            ScanLimits::default()
        )
        .is_err()
    );
}
#[test]
fn raw_directory_terminators_count_toward_the_total_name_limit_before_output() {
    let name = std::iter::repeat_n("x".repeat(255), 16)
        .collect::<Vec<_>>()
        .join("/");
    assert_eq!(name.len(), 4095);
    let entries = (0..4097)
        .map(|_| EncodeEntry {
            name: name.clone(),
            directory: true,
            size: 0,
            modified_unix: 1700000000,
        })
        .collect();
    let input = Input {
        entries,
        data: vec![],
    };
    let mut out = vec![];
    assert_eq!(
        encode(
            &input,
            &mut out,
            settings(6),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Limit)
    );
    assert!(out.is_empty());
}
#[test]
fn unsupported_settings_invalid_metadata_and_source_size_changes_cannot_finish() {
    let cancel = Cancellation::default();
    let mut input = Input::new(&[("hello", false, b"hello".to_vec())]);
    for settings in [
        EncodeSettings {
            method: "store",
            ..settings(6)
        },
        settings(0),
        EncodeSettings {
            password: Some("owned"),
            ..settings(6)
        },
        EncodeSettings {
            solid: true,
            ..settings(6)
        },
        EncodeSettings {
            encrypt_headers: true,
            ..settings(6)
        },
    ] {
        let mut out = vec![];
        assert_eq!(
            encode(&input, &mut out, settings, &cancel, &mut |_| {}),
            Err(EncodeError::Unsupported)
        );
        assert!(out.is_empty());
    }
    for (name, size, time) in [
        ("../escape", 5, 1700000000),
        ("hello", u64::from(u32::MAX) + 1, 1700000000),
        ("hello", 5, -1),
    ] {
        input.entries[0].name = name.into();
        input.entries[0].size = size;
        input.entries[0].modified_unix = time;
        let mut out = vec![];
        assert!(encode(&input, &mut out, settings(6), &cancel, &mut |_| {}).is_err());
        assert!(out.is_empty());
    }
    input.entries[0].name = "hello".into();
    input.entries[0].modified_unix = 1700000000;
    for size in [4, 6] {
        input.entries[0].size = size;
        assert_eq!(
            encode(&input, &mut vec![], settings(6), &cancel, &mut |_| {}),
            Err(EncodeError::Source)
        );
    }
}
#[test]
fn cancellation_and_output_patch_failure_remain_terminal_and_no_valid_end_is_added() {
    let input = Input::new(&[("hello", false, vec![b'A'; 100000])]);
    let cancel = Cancellation::default();
    let mut out = vec![];
    assert_eq!(
        encode(&input, &mut out, settings(9), &cancel, &mut |_| cancel
            .cancel()),
        Err(EncodeError::Cancelled)
    );
    assert!(
        inspect(
            &mut Cursor::new(&out),
            &Cancellation::default(),
            ScanLimits::default()
        )
        .is_err()
    );
    struct PatchFail(Vec<u8>);
    impl Write for PatchFail {
        fn write(&mut self, b: &[u8]) -> io::Result<usize> {
            self.0.write(b)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl EncodeOutput for PatchFail {
        fn write_at(&mut self, _: u64, _: &[u8]) -> Result<(), EncodeError> {
            Err(EncodeError::Output)
        }
    }
    let mut out = PatchFail(vec![]);
    assert_eq!(
        encode(
            &input,
            &mut out,
            settings(6),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Output)
    );
    assert!(
        inspect(
            &mut Cursor::new(&out.0),
            &Cancellation::default(),
            ScanLimits::default()
        )
        .is_err()
    );
}
#[test]
fn typed_source_failures_keep_their_category_without_retry_or_probe_fallback() {
    struct FailInput(Input, ScanError);
    impl EncodeInput for FailInput {
        fn entries(&self) -> &[EncodeEntry] {
            self.0.entries()
        }
        fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            struct Fail(ScanError);
            impl Read for Fail {
                fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                    Err(io::Error::new(io::ErrorKind::Interrupted, self.0))
                }
            }
            Ok(Box::new(Fail(self.1)))
        }
    }
    for (error, expected) in [
        (ScanError::Cancelled, EncodeError::Cancelled),
        (ScanError::Limit, EncodeError::Limit),
        (ScanError::SourceChanged, EncodeError::Source),
    ] {
        let input = FailInput(Input::new(&[("hello", false, b"hello".to_vec())]), error);
        assert_eq!(
            encode(
                &input,
                &mut vec![],
                settings(6),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(expected)
        );
    }
}
#[test]
#[ignore = "exports owned containers for the explicit independent creation-reference check"]
fn export_owned_lzh_container_reference_outputs() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build")
        .canonicalize()
        .unwrap();
    let out = std::path::PathBuf::from(
        std::env::var_os("XDZIP_LZH_CONTAINER_REFERENCE_OUTPUT")
            .expect("explicit owned output required"),
    );
    assert!(out.is_absolute());
    assert!(
        out.parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(&root)
    );
    std::fs::create_dir(&out).unwrap();
    for level in [1, 6, 9] {
        for (profile, items) in [
            (
                "ordinary",
                vec![
                    ("nested", true, vec![]),
                    ("nested/empty-dir", true, vec![]),
                    (
                        "nested/hello.bin",
                        false,
                        b"abracadabra abracadabra\n".repeat(6 * 4096),
                    ),
                    ("empty.bin", false, vec![]),
                    ("allbytes.bin", false, (0..=255).collect()),
                ],
            ),
            (
                "unicode",
                vec![
                    ("资料", true, vec![]),
                    (
                        "资料/说明.txt",
                        false,
                        b"owned Unicode name payload\n".to_vec(),
                    ),
                ],
            ),
        ] {
            let bytes = create(&Input::new(&items), level);
            std::fs::write(out.join(format!("{profile}-{level}.lzh")), bytes).unwrap();
        }
    }
}
