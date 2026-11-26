use super::*;
use crate::OpticalCodec;
use std::io::Cursor;
use std::io::Write;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
use xdzip_codec_api::{Format, MetadataCodec, ScanLimits};

struct Input {
    entries: Vec<EncodeEntry>,
    contents: Vec<Vec<u8>>,
}
impl Input {
    fn new(items: &[(&str, bool, &[u8])]) -> Self {
        Self {
            entries: items
                .iter()
                .map(|(name, directory, bytes)| EncodeEntry {
                    name: (*name).into(),
                    directory: *directory,
                    size: bytes.len() as u64,
                    modified_unix: 1_700_000_000,
                })
                .collect(),
            contents: items.iter().map(|(_, _, bytes)| bytes.to_vec()).collect(),
        }
    }
}
impl EncodeInput for Input {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(Cursor::new(&self.contents[index])))
    }
}
fn settings() -> EncodeSettings<'static> {
    EncodeSettings {
        method: "iso9660",
        level: 0,
        solid: false,
        encrypt_headers: false,
        password: None,
    }
}
fn image(input: &Input) -> Vec<u8> {
    let mut out = vec![];
    encode(
        input,
        &mut out,
        settings(),
        &Cancellation::default(),
        &mut |_| {},
    )
    .unwrap();
    out
}
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend(bytes);
        Ok(())
    }
}
#[test]
fn joliet_round_trip_preserves_unsorted_names_empty_files_and_sector_boundaries() {
    let payload = vec![0xa5; SECTOR * 3 + 1];
    let input = Input::new(&[
        ("中文/较长文件名.txt", false, &payload),
        ("zero", false, b""),
        ("中文", true, b""),
        ("empty", true, b""),
    ]);
    let bytes = image(&input);
    assert_eq!(bytes.len() % SECTOR, 0);
    assert_eq!(&bytes[17 * SECTOR + 88..17 * SECTOR + 91], b"%/E");
    let cancel = Cancellation::default();
    let codec = OpticalCodec {
        format: Format::Iso,
    };
    let mut source = Cursor::new(bytes.clone());
    let rows = codec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut decoder = codec.open_payload(&mut source, &rows, &cancel).unwrap();
    assert_eq!(rows.len(), input.entries.len());
    for row in &rows {
        let name = String::from_utf8(row.name_bytes().to_vec()).unwrap();
        let name = name.trim_end_matches('/');
        let index = input
            .entries
            .iter()
            .position(|entry| entry.name == name)
            .unwrap();
        assert_eq!(row.directory, input.entries[index].directory);
        if !row.directory {
            let mut sink = Sink(vec![]);
            decoder.decode(row, &mut sink, &cancel).unwrap();
            assert_eq!(sink.0, input.contents[index]);
        }
    }
    // Disabling the SVD independently exercises the primary ISO namespace.
    let mut primary = bytes;
    primary[17 * SECTOR..18 * SECTOR].fill(0);
    primary[17 * SECTOR..17 * SECTOR + 7].copy_from_slice(b"\xffCD001\x01");
    let primary_rows = codec
        .scan(&mut Cursor::new(primary), &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(primary_rows.len(), rows.len());
    assert!(primary_rows.iter().all(|row| row.name_bytes().is_ascii()));
}
#[test]
fn empty_image_and_multi_sector_directories_have_valid_endian_path_tables() {
    for count in [0, 120] {
        let mut input = Input::new(&[]);
        for i in 0..count {
            input.entries.push(EncodeEntry {
                name: format!("file-{i:03}-中文.bin"),
                directory: false,
                size: 0,
                modified_unix: 0,
            });
            input.contents.push(vec![]);
        }
        let bytes = image(&input);
        for ns in 0..2 {
            let vd = &bytes[(16 + ns) * SECTOR..(17 + ns) * SECTOR];
            let length = u32::from_le_bytes(vd[132..136].try_into().unwrap());
            assert_eq!(length, u32::from_be_bytes(vd[136..140].try_into().unwrap()));
            let little = u32::from_le_bytes(vd[140..144].try_into().unwrap()) as usize * SECTOR;
            let big = u32::from_be_bytes(vd[148..152].try_into().unwrap()) as usize * SECTOR;
            assert_eq!(bytes[little], 1);
            assert_eq!(bytes[big], 1);
            assert_eq!(
                u32::from_le_bytes(bytes[little + 2..little + 6].try_into().unwrap()),
                u32::from_be_bytes(bytes[big + 2..big + 6].try_into().unwrap())
            );
            assert_eq!(&bytes[little + 6..little + 8], &[1, 0]);
            assert_eq!(&bytes[big + 6..big + 8], &[0, 1]);
        }
        let rows = OpticalCodec {
            format: Format::Iso,
        }
        .scan(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap();
        assert_eq!(rows.len(), count);
    }
}
#[test]
fn rejects_unrepresentable_names_collisions_and_missing_parents_before_output() {
    for input in [
        Input::new(&[("emoji-😀.txt", false, b"")]),
        Input::new(&[("semi;colon", false, b"")]),
        Input::new(&[("question?.txt", false, b"")]),
        Input::new(&[("asterisk*.txt", true, b"")]),
        Input::new(&[("../unsafe", false, b"")]),
        Input::new(&[("missing/file", false, b"")]),
        Input::new(&[("same", false, b""), ("same", true, b"")]),
        Input::new(&[("file", false, b""), ("file/sub", false, b"")]),
        Input::new(&[(&"x".repeat(65), false, b"")]),
    ] {
        let mut out = vec![];
        assert!(
            encode(
                &input,
                &mut out,
                settings(),
                &Cancellation::default(),
                &mut |_| {}
            )
            .is_err()
        );
        assert!(out.is_empty());
    }
    image(&Input::new(&[(&"中".repeat(64), false, b"ok")]));
}
#[test]
fn size_depth_options_and_pre_cancel_are_rejected_without_writing() {
    let first = "a".repeat(32);
    let second = format!("{first}/{}", "b".repeat(32));
    let accepted = format!("{second}/{}", "c".repeat(53)); // 240 bytes including version and parent count.
    image(&Input::new(&[
        (&first, true, b""),
        (&second, true, b""),
        (&accepted, false, b"ok"),
    ]));
    let rejected = format!("{accepted}x");
    let input = Input::new(&[
        (&first, true, b""),
        (&second, true, b""),
        (&rejected, false, b"bad"),
    ]);
    let mut output = vec![];
    assert_eq!(
        encode(
            &input,
            &mut output,
            settings(),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Name)
    );
    assert!(output.is_empty());
    let mut input = Input::new(&[("huge", false, b"")]);
    input.entries[0].size = 1u64 << 32;
    assert!(Plan::new(&input.entries, &Cancellation::default()).is_err());
    let input = Input::new(&[("a/b/c/d/e/f/g/h/file", false, b"")]);
    assert!(Plan::new(&input.entries, &Cancellation::default()).is_err());
    let input = Input::new(&[("file", false, b"hello")]);
    let cancel = Cancellation::default();
    cancel.cancel();
    let mut out = vec![];
    assert_eq!(
        encode(&input, &mut out, settings(), &cancel, &mut |_| {}),
        Err(EncodeError::Cancelled)
    );
    assert!(out.is_empty());
    let mut bad = settings();
    bad.password = Some("secret");
    assert_eq!(
        encode(&input, &mut out, bad, &Cancellation::default(), &mut |_| {}),
        Err(EncodeError::Unsupported)
    );
    assert!(out.is_empty());
}
#[test]
fn source_growth_truncation_output_failure_and_midstream_cancellation_propagate() {
    for declared in [2, 4] {
        let mut input = Input::new(&[("file", false, b"abc")]);
        input.entries[0].size = declared;
        assert_eq!(
            encode(
                &input,
                &mut vec![],
                settings(),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Source)
        );
    }
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("owned fault"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl EncodeOutput for Broken {}
    let input = Input::new(&[("file", false, &vec![42; 300_000])]);
    assert_eq!(
        encode(
            &input,
            &mut Broken,
            settings(),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Output)
    );
    let cancel = Cancellation::default();
    assert_eq!(
        encode(&input, &mut vec![], settings(), &cancel, &mut |_| cancel
            .cancel()),
        Err(EncodeError::Cancelled)
    );
}
