// crates/xdzip-codec-native/src/tests.rs

use super::*;
use std::io::{self, Write};

fn fixture() -> BufferedInput {
    BufferedInput {
        entries: vec![
            EncodeEntry {
                name: "folder/first.txt".into(),
                directory: false,
                size: 180_000,
                modified_unix: 1_700_000_000,
            },
            EncodeEntry {
                name: "folder/second.txt".into(),
                directory: false,
                size: 90_000,
                modified_unix: 1_700_000_000,
            },
            EncodeEntry {
                name: "空目录".into(),
                directory: true,
                size: 0,
                modified_unix: 1_700_000_000,
            },
        ],
        bytes: vec![b"abcdef".repeat(30_000), b"xyz".repeat(30_000), vec![]],
    }
}
fn settings(password: Option<&str>, solid: bool, headers: bool) -> EncodeSettings<'_> {
    EncodeSettings {
        method: "lzma2",
        level: 5,
        password,
        solid,
        encrypt_headers: headers,
    }
}
fn create(password: Option<&str>, solid: bool, headers: bool) -> Vec<u8> {
    let mut bytes = Vec::new();
    NativeCodec {
        format: Format::SevenZip,
    }
    .encode(
        &fixture(),
        &mut bytes,
        settings(password, solid, headers),
        &Cancellation::default(),
        &mut |_| {},
    )
    .unwrap();
    bytes
}
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    skipped: u64,
}
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn charge_skipped(&mut self, count: u64) -> Result<(), DecodeError> {
        self.skipped += count;
        Ok(())
    }
}
#[test]
fn official_7z_roundtrip_and_solid_stream_continuation() {
    let codec = NativeCodec {
        format: Format::SevenZip,
    };
    for solid in [false, true] {
        let mut source = Cursor::new(create(None, solid, false));
        let control = Cancellation::default();
        let rows = codec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 3);
        let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
        let expected = fixture();
        for row in &rows {
            let mut sink = Sink::default();
            reader.decode(row, &mut sink, &control).unwrap();
            let index = expected
                .entries
                .iter()
                .position(|entry| {
                    entry.name == String::from_utf8_lossy(row.name_bytes()).trim_end_matches('/')
                })
                .unwrap();
            assert_eq!(sink.bytes, expected.bytes[index]);
            assert_eq!(
                sink.skipped, 0,
                "sequential solid decoding must not replay preceding records"
            );
        }
    }
}
#[test]
fn encrypted_headers_require_password_and_retry_without_partial_index() {
    let codec = NativeCodec {
        format: Format::SevenZip,
    };
    let mut source = Cursor::new(create(Some("native fixture password"), true, true));
    let control = Cancellation::default();
    assert_eq!(
        codec.scan(&mut source, &control, ScanLimits::default()),
        Err(ScanError::PasswordRequired)
    );
    assert_eq!(
        codec.scan_with_password(
            &mut source,
            &control,
            ScanLimits::default(),
            Some("incorrect")
        ),
        Err(ScanError::WrongPassword)
    );
    let rows = codec
        .scan_with_password(
            &mut source,
            &control,
            ScanLimits::default(),
            Some("native fixture password"),
        )
        .unwrap();
    let mut reader = codec
        .open_payload_with_password(
            &mut source,
            &rows,
            &control,
            Some("native fixture password"),
        )
        .unwrap();
    for row in rows.iter().filter(|row| !row.directory) {
        let mut sink = Sink::default();
        reader.decode(row, &mut sink, &control).unwrap();
        assert_eq!(sink.bytes.len() as u64, row.size);
    }
}
#[test]
fn solid_selected_entry_charges_skipped_real_bytes() {
    let codec = NativeCodec {
        format: Format::SevenZip,
    };
    let mut source = Cursor::new(create(None, true, false));
    let control = Cancellation::default();
    let rows = codec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let last = rows.iter().rfind(|row| !row.directory).unwrap();
    let prior: u64 = rows
        .iter()
        .take_while(|row| row.id != last.id)
        .map(|row| row.size)
        .sum();
    let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    reader.decode(last, &mut sink, &control).unwrap();
    assert_eq!(sink.skipped, prior);
    assert_eq!(sink.bytes.len() as u64, last.size);
}
#[test]
fn solid_rewrite_preserves_content_and_applies_rename_addition() {
    let codec = NativeCodec {
        format: Format::SevenZip,
    };
    let control = Cancellation::default();
    let mut source = Cursor::new(create(None, true, false));
    let rows = codec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let keep = vec![true; rows.len()];
    let mut renames = vec![None; rows.len()];
    let renamed = rows
        .iter()
        .position(|row| row.name_bytes() == b"folder/first.txt")
        .unwrap();
    renames[renamed] = Some("renamed.txt".into());
    let addition = BufferedInput {
        entries: vec![EncodeEntry {
            name: "new.txt".into(),
            directory: false,
            size: 3,
            modified_unix: 0,
        }],
        bytes: vec![b"new".to_vec()],
    };
    let mut output = Cursor::new(Vec::new());
    codec
        .rewrite(
            &mut source,
            RewriteInput {
                keep: &keep,
                renames: &renames,
                additions: &addition,
                settings: settings(None, true, false),
            },
            &mut output,
            &control,
            &mut |_| {},
        )
        .unwrap();
    let rows = codec
        .scan(&mut output, &control, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().any(|row| row.name_bytes() == b"renamed.txt"));
    let mut reader = codec.open_payload(&mut output, &rows, &control).unwrap();
    for row in rows.iter().filter(|row| !row.directory) {
        let mut sink = Sink::default();
        reader.decode(row, &mut sink, &control).unwrap();
        let expected = match row.name_bytes() {
            b"renamed.txt" => b"abcdef".repeat(30_000),
            b"folder/second.txt" => b"xyz".repeat(30_000),
            b"new.txt" => b"new".to_vec(),
            _ => panic!("unexpected name"),
        };
        assert_eq!(sink.bytes, expected);
    }
}
#[test]
fn output_failure_cancellation_and_read_budget_fail_closed() {
    struct Broken;
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("controlled failure"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    impl EncodeOutput for Broken {
        fn write_at(&mut self, _: u64, _: &[u8]) -> Result<(), EncodeError> {
            Err(EncodeError::Output)
        }
    }
    let codec = NativeCodec {
        format: Format::SevenZip,
    };
    assert_eq!(
        codec.encode(
            &fixture(),
            &mut Broken,
            settings(None, false, false),
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Output)
    );
    let control = Cancellation::default();
    control.cancel();
    assert_eq!(
        codec.scan(&mut Cursor::new(vec![]), &control, ScanLimits::default()),
        Err(ScanError::Cancelled)
    );
    let mut source = Cursor::new(create(None, false, false));
    assert_eq!(
        codec.scan(
            &mut source,
            &Cancellation::default(),
            ScanLimits {
                read_bytes: 1,
                ..ScanLimits::default()
            }
        ),
        Err(ScanError::Limit)
    );
}
#[test]
fn malformed_7z_and_wrong_container_never_produce_index() {
    let codec = NativeCodec {
        format: Format::SevenZip,
    };
    for bytes in [
        b"PK\x03\x04not a 7z".to_vec(),
        b"7z\xbc\xaf\x27\x1c\x00".to_vec(),
    ] {
        assert_eq!(
            codec.scan(
                &mut Cursor::new(bytes),
                &Cancellation::default(),
                ScanLimits::default()
            ),
            Err(ScanError::Invalid)
        );
    }
}
#[test]
fn read_only_native_formats_reject_creation_and_modification_without_starting_worker() {
    for format in [Format::Rar, Format::Cab, Format::Iso, Format::Udf] {
        let codec = NativeCodec { format };
        assert_eq!(
            codec.encode(
                &fixture(),
                &mut Vec::new(),
                settings(None, false, false),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Unsupported)
        );
        assert_eq!(
            codec.rewrite(
                &mut Cursor::new(vec![]),
                RewriteInput {
                    keep: &[],
                    renames: &[],
                    additions: &fixture(),
                    settings: settings(None, false, false)
                },
                &mut Cursor::new(vec![]),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Unsupported)
        );
    }
}

#[test]
fn cabinet_start_binds_bounded_raw_names_to_authorized_volume_order() {
    let codec = NativeCodec {
        format: Format::Cab,
    };
    let names = [b"start.cab".to_vec(), b"last.bin".to_vec()];
    let request = codec
        .request(
            0,
            30,
            None,
            0,
            Default::default(),
            (Some(&[10, 20]), Some(&names)),
        )
        .unwrap();
    assert_eq!(&request[..2], &[0, 8]);
    let mut fields = Fields(&request[2..]);
    fields.u64().unwrap();
    fields.u32().unwrap();
    fields.u32().unwrap();
    fields.u64().unwrap();
    assert_eq!(fields.byte(), Ok(0));
    assert_eq!(fields.string(4096), Ok(String::new()));
    fields.u64().unwrap();
    assert_eq!(fields.u32(), Ok(2));
    assert_eq!(fields.u64(), Ok(10));
    assert_eq!(fields.u64(), Ok(20));
    assert_eq!(fields.u32(), Ok(2));
    assert_eq!(fields.bytes(4096), Ok(names[0].clone()));
    assert_eq!(fields.bytes(4096), Ok(names[1].clone()));
    fields.end().unwrap();
    assert_eq!(
        codec.request(0, 30, None, 0, Default::default(), (Some(&[10, 20]), None)),
        Err(Fault::Invalid)
    );
    assert_eq!(
        codec.request(
            0,
            30,
            None,
            0,
            Default::default(),
            (Some(&[30]), Some(&names))
        ),
        Err(Fault::Invalid)
    );
    let names = vec![vec![b'x'; 4096]; 1024];
    let lengths = vec![1; 1024];
    assert_eq!(
        codec.request(
            0,
            1024,
            None,
            0,
            Default::default(),
            (Some(&lengths), Some(&names))
        ),
        Err(Fault::Limit)
    );
}

#[test]
fn single_cabinet_file_and_cursor_authorize_only_their_own_stream() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/cab-iso");
    let control = Cancellation::default();
    let codec = NativeCodec {
        format: Format::Cab,
    };
    let bytes = std::fs::read(root.join("named.cab")).unwrap();
    let mut cursor = Cursor::new(bytes);
    let rows = codec
        .scan(&mut cursor, &control, Default::default())
        .unwrap();
    assert_eq!(rows.len(), 3);
    let hello = rows
        .iter()
        .find(|row| row.name_bytes() == b"hello.txt")
        .unwrap();
    let mut sink = Sink::default();
    codec
        .open_payload(&mut cursor, &rows, &control)
        .unwrap()
        .decode(hello, &mut sink, &control)
        .unwrap();
    assert_eq!(sink.bytes, b"xdzip read-only archive fixture\n");
    let mut file = std::fs::File::open(root.join("named.cab")).unwrap();
    assert_eq!(
        codec.scan(&mut file, &control, Default::default()).unwrap(),
        rows
    );
    let mut first_part = std::fs::File::open(root.join("cabd_multi_basic_pt1.cab")).unwrap();
    assert_eq!(
        codec.scan(&mut first_part, &control, Default::default()),
        Err(ScanError::MissingVolume)
    );
}

#[test]
fn cabinet_volume_names_have_their_own_limit_independent_of_entry_names() {
    struct Named {
        cursor: Cursor<Vec<u8>>,
        length: [u64; 1],
        names: [Vec<u8>; 1],
    }
    impl Read for Named {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            self.cursor.read(bytes)
        }
    }
    impl std::io::Seek for Named {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.cursor.seek(position)
        }
    }
    impl ReadSource for Named {
        fn volume_lengths(&self) -> Option<&[u64]> {
            Some(&self.length)
        }
        fn volume_names(&self) -> Option<&[Vec<u8>]> {
            Some(&self.names)
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/cab-iso/named.cab");
    let bytes = std::fs::read(path).unwrap();
    let mut source = Named {
        length: [bytes.len() as u64],
        cursor: Cursor::new(bytes),
        names: [b"x".repeat(100)],
    };
    let rows = NativeCodec {
        format: Format::Cab,
    }
    .scan(
        &mut source,
        &Default::default(),
        ScanLimits {
            name_bytes: 32,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|row| row.raw_name.len() <= 32));
}

fn rar_fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/native-archives")
            .join(name),
    )
    .unwrap()
}
#[test]
fn official_unrar_reads_legacy_rar5_header_encryption_and_solid() {
    let codec = NativeCodec {
        format: Format::Rar,
    };
    for name in [
        "test_read_format_rar4_encrypted_filenames.rar",
        "test_read_format_rar5_encrypted_filenames.rar",
        "test_read_format_rar5_solid_encrypted.rar",
        "test_read_format_rar5_solid_encrypted_filenames.rar",
    ] {
        let mut source = Cursor::new(rar_fixture(name));
        let control = Cancellation::default();
        if name.contains("filenames") {
            assert_eq!(
                codec.scan(&mut source, &control, ScanLimits::default()),
                Err(ScanError::PasswordRequired)
            );
            let wrong = codec.scan_with_password(
                &mut source,
                &control,
                ScanLimits::default(),
                Some("wrong"),
            );
            assert!(matches!(
                wrong,
                Err(ScanError::WrongPassword | ScanError::Invalid)
            ));
        }
        let rows = codec
            .scan_with_password(
                &mut source,
                &control,
                ScanLimits::default(),
                Some("password"),
            )
            .unwrap();
        assert_eq!(rows.len(), 4);
        let mut reader = codec
            .open_payload_with_password(&mut source, &rows, &control, Some("password"))
            .unwrap();
        for row in rows.iter().filter(|row| !row.directory && !row.link) {
            let mut sink = Sink::default();
            reader
                .decode_with_password(row, &mut sink, &control, Some("password"))
                .unwrap();
            assert_eq!(
                sink.bytes,
                format!("This is from {}", String::from_utf8_lossy(row.name_bytes())).into_bytes()
            );
        }
    }
}
#[test]
fn unrar_nonsolid_mixed_passwords_do_not_replay_another_files_secret() {
    let codec = NativeCodec {
        format: Format::Rar,
    };
    for name in [
        "test_read_format_rar4_encrypted.rar",
        "test_read_format_rar5_encrypted.rar",
    ] {
        let mut source = Cursor::new(rar_fixture(name));
        let control = Cancellation::default();
        let rows = codec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
        for row in rows.iter().filter(|row| !row.directory && !row.link) {
            let password = match row.name_bytes() {
                b"b.txt" => Some("password"),
                b"d.txt" => Some("password2"),
                _ => None,
            };
            let mut sink = Sink::default();
            reader
                .decode_with_password(row, &mut sink, &control, password)
                .unwrap();
            assert_eq!(
                sink.bytes,
                format!("This is from {}", String::from_utf8_lossy(row.name_bytes())).into_bytes()
            );
        }
    }
}
struct Volumes {
    source: Cursor<Vec<u8>>,
    lengths: Vec<u64>,
}
impl Read for Volumes {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.source.read(bytes)
    }
}
impl std::io::Seek for Volumes {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.source.seek(position)
    }
}
impl ReadSource for Volumes {
    fn volume_lengths(&self) -> Option<&[u64]> {
        Some(&self.lengths)
    }
}
fn rar_volumes(count: usize) -> Volumes {
    let parts: Vec<_> = (1..=count)
        .map(|index| {
            rar_fixture(&format!(
                "test_read_format_rar5_multiarchive_solid.part{index:02}.rar"
            ))
        })
        .collect();
    Volumes {
        lengths: parts.iter().map(|part| part.len() as u64).collect(),
        source: Cursor::new(parts.concat()),
    }
}
#[test]
fn unrar_preserves_solid_state_across_authorized_volumes_and_rejects_missing_tail() {
    let codec = NativeCodec {
        format: Format::Rar,
    };
    let control = Cancellation::default();
    let mut source = rar_volumes(4);
    let rows = codec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
    for row in rows.iter().filter(|row| !row.directory && !row.link) {
        let mut sink = Sink::default();
        reader.decode(row, &mut sink, &control).unwrap();
        assert_eq!(sink.bytes.len() as u64, row.size);
        assert_eq!(sink.skipped, 0);
    }
    drop(reader);
    assert_eq!(
        codec.scan(&mut rar_volumes(3), &control, ScanLimits::default()),
        Err(ScanError::MissingVolume)
    );
}
#[test]
fn mtime_survives_solid_rewrite() {
    let codec = NativeCodec {
        format: Format::SevenZip,
    };
    let control = Cancellation::default();
    let mut source = Cursor::new(create(None, true, false));
    let rows = codec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert!(rows.iter().all(|row| row.modified_dos != 0));
    let keep = vec![true; rows.len()];
    let additions = BufferedInput {
        entries: vec![],
        bytes: vec![],
    };
    let mut output = Cursor::new(vec![]);
    codec
        .rewrite(
            &mut source,
            RewriteInput {
                keep: &keep,
                renames: &[],
                additions: &additions,
                settings: settings(None, true, false),
            },
            &mut output,
            &control,
            &mut |_| {},
        )
        .unwrap();
    let rewritten = codec
        .scan(&mut output, &control, ScanLimits::default())
        .unwrap();
    for row in &rows {
        assert_eq!(
            rewritten
                .iter()
                .find(|candidate| candidate.name_bytes() == row.name_bytes())
                .unwrap()
                .modified_dos,
            row.modified_dos
        );
    }
}
#[test]
fn export_created_7z_for_independent_interoperability_when_requested() {
    let Some(destination) = std::env::var_os("XDZIP_NATIVE_INTEROP_OUTPUT") else {
        return;
    };
    let destination = std::path::PathBuf::from(destination);
    let build = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build")
        .canonicalize()
        .unwrap();
    std::fs::create_dir_all(&destination).unwrap();
    assert!(destination.canonicalize().unwrap().starts_with(&build));
    let input = fixture();
    let expected = destination.join("expected");
    std::fs::create_dir_all(&expected).unwrap();
    for (entry, bytes) in input.entries.iter().zip(&input.bytes) {
        let path = expected.join(&entry.name);
        if entry.directory {
            std::fs::create_dir_all(path).unwrap();
        } else {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, bytes).unwrap();
        }
    }
    for (name, password, solid, headers) in [
        ("ordinary.7z", None, false, false),
        ("solid.7z", None, true, false),
        (
            "encrypted.7z",
            Some("native fixture password"),
            false,
            false,
        ),
        (
            "encrypted-headers.7z",
            Some("native fixture password"),
            true,
            true,
        ),
    ] {
        std::fs::write(destination.join(name), create(password, solid, headers)).unwrap();
    }
}
