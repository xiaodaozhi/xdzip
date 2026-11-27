use super::*;
#[test]
fn entry_comment_and_crc_reach_metadata() {
    let original = fixture("m1-phrase.arj");
    let control = Cancellation::default();
    let index = inspect(&mut Cursor::new(&original), &control, ScanLimits::default()).unwrap();
    let at = index.records[0].header_offset as usize;
    let size = u16::from_le_bytes(original[at + 2..at + 4].try_into().unwrap()) as usize;
    let mut body = original[at + 4..at + 4 + size].to_vec();
    assert_eq!(body.pop(), Some(0));
    body.extend_from_slice(b"owned note\0");
    let mut bytes = original[..at].to_vec();
    bytes.extend_from_slice(&[0x60, 0xea]);
    bytes.extend_from_slice(&(body.len() as u16).to_le_bytes());
    bytes.extend_from_slice(&body);
    bytes.extend_from_slice(&xdzip_compression::crc32(&body).to_le_bytes());
    bytes.extend_from_slice(&original[at + 8 + size..]);
    let rows = ArjCodec
        .scan(&mut Cursor::new(bytes), &control, ScanLimits::default())
        .unwrap();
    assert_eq!(rows[0].details.comment, b"owned note");
    assert_eq!(rows[0].details.crc, Some((index.records[0].crc32, 32)));
}
use crate::{test_cases as cases, test_known as known};
use std::io::{Cursor, Read, Seek};
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/arj")
            .join(name),
    )
    .unwrap()
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
fn every_native_method_projects_and_decodes_with_exact_frozen_rows() {
    let cancel = Cancellation::default();
    for case in cases::CASES {
        let bytes = fixture(case.name);
        let mut source = Cursor::new(&bytes);
        assert_eq!(ArjCodec::candidate(&mut source, &cancel), Ok(true));
        assert_eq!(source.position(), 0);
        let rows = ArjCodec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(
            (row.format, row.method, row.size, row.packed_size),
            (
                Format::Arj,
                u16::from(case.method),
                case.decoded as u64,
                case.packed as u64
            )
        );
        assert_eq!(row.raw_name, b"HELLO.BIN");
        assert_eq!(row.display_name, "HELLO.BIN");
        assert!(row.payload_supported());
        let mut reader = ArjCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink::default();
        reader.decode(row, &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, known::plain(case.kind));
        assert_eq!(xdzip_compression::crc32(&sink.0), case.crc);
        assert!(case.method == 0 || case.method == case.requested);
        assert_eq!(bytes.len() - case.offset - 4, case.packed);
        let mut changed = row.clone();
        changed.packed_size += 1;
        assert_eq!(
            reader.decode(&changed, &mut sink, &cancel),
            Err(DecodeError::Integrity)
        );
        assert_eq!(
            reader.decode(row, &mut sink, &cancel),
            Err(DecodeError::Integrity)
        );
        drop(reader);
        let mut changed = rows.clone();
        changed[0].method = 8;
        assert!(matches!(
            ArjCodec.open_payload(&mut source, &changed, &cancel),
            Err(DecodeError::Integrity)
        ));
    }
}
#[test]
fn unknown_encrypted_and_special_records_never_gain_payload_authority() {
    let cancel = Cancellation::default();
    let mut bytes = fixture("m1-phrase.arj");
    let index = inspect(&mut Cursor::new(&bytes), &cancel, ScanLimits::default()).unwrap();
    let offset = index.records[0].header_offset as usize;
    // ARJ method 8 must remain 8, without ZIP Deflate admission.
    bytes[offset + 9] = 8;
    let size = u16::from_le_bytes(bytes[offset + 2..offset + 4].try_into().unwrap()) as usize;
    let crc = xdzip_compression::crc32(&bytes[offset + 4..offset + 4 + size]);
    bytes[offset + 4 + size..offset + 8 + size].copy_from_slice(&crc.to_le_bytes());
    let mut source = Cursor::new(&bytes);
    let rows = ArjCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows[0].method, 8);
    assert!(!rows[0].payload_supported());
    let mut reader = ArjCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&rows[0], &mut sink, &cancel),
        Err(DecodeError::Unsupported)
    );
    assert!(sink.0.is_empty());
    let mut index = index;
    for kind in [2, 4, 5, 6, 255] {
        index.records[0].kind = kind;
        assert!(!project(&index, &cancel, ScanLimits::default()).unwrap()[0].payload_supported());
    }
    index.records[0].kind = 0;
    index.records[0].flags |= 1;
    let rows = project(&index, &cancel, ScanLimits::default()).unwrap();
    assert!(rows[0].encrypted && !rows[0].password_supported && !rows[0].payload_supported());
}
#[test]
fn portable_and_unix_backslashes_remain_suspicious_dos_separators_are_explicit() {
    let cancel = Cancellation::default();
    let mut index = inspect(
        &mut Cursor::new(fixture("m1-phrase.arj")),
        &cancel,
        ScanLimits::default(),
    )
    .unwrap();
    for (host, flags, effective, warning) in [
        (11, 0, "folder/file.bin", false),
        (11, 16, "folder\\file.bin", true),
        (2, 0, "folder\\file.bin", true),
        (2, 16, "folder\\file.bin", true),
    ] {
        let record = &mut index.records[0];
        record.host = host;
        record.flags = flags;
        record.attributes = 0;
        record.raw_name = b"folder\\file.bin".to_vec();
        let rows = project(&index, &cancel, ScanLimits::default()).unwrap();
        let row = &rows[0];
        assert_eq!(row.name_bytes(), effective.as_bytes());
        assert_eq!(row.warning, warning);
    }
    index.records[0].raw_name = b"../outside".to_vec();
    assert!(project(&index, &cancel, ScanLimits::default()).unwrap()[0].warning);
    let record = &mut index.records[0];
    record.raw_name = b"folder".to_vec();
    record.kind = 3;
    record.method = 0;
    record.size = 0;
    record.packed_size = 0;
    record.crc32 = 0;
    let rows = project(&index, &cancel, ScanLimits::default()).unwrap();
    assert!(rows[0].directory && rows[0].payload_supported());
    assert_eq!(rows[0].decoded_name.as_deref(), Some("folder/"));
    assert_eq!(rows[0].unix_mode, 0);
    assert!(matches!(
        project(
            &index,
            &cancel,
            ScanLimits {
                name_bytes: 6,
                ..ScanLimits::default()
            }
        ),
        Err(ScanError::Limit)
    ));
    assert!(matches!(
        project(
            &index,
            &cancel,
            ScanLimits {
                display_name_bytes: 1,
                ..ScanLimits::default()
            }
        ),
        Err(ScanError::Limit)
    ));
}
#[test]
fn short_candidates_rewind_and_host_typed_invalid_is_not_a_format_miss() {
    let cancel = Cancellation::default();
    let bytes = fixture("m1-phrase.arj");
    for length in 0..12 {
        let mut source = Cursor::new(&bytes[..length]);
        assert_eq!(ArjCodec::candidate(&mut source, &cancel), Ok(false));
        assert_eq!(source.position(), 0);
    }
    struct Fault(Cursor<Vec<u8>>);
    impl Read for Fault {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other(ScanError::Invalid))
        }
    }
    impl Seek for Fault {
        fn seek(&mut self, at: SeekFrom) -> std::io::Result<u64> {
            self.0.seek(at)
        }
    }
    impl ReadSource for Fault {}
    let mut fault = Fault(Cursor::new(bytes));
    assert_eq!(ArjCodec::candidate(&mut fault, &cancel), Err(ScanError::Io));
    assert_eq!(fault.0.position(), 0);
}

#[test]
fn multibyte_trail_backslashes_are_decoded_before_dos_path_separators() {
    let cancel = Cancellation::default();
    let mut index = inspect(
        &mut Cursor::new(fixture("m1-phrase.arj")),
        &cancel,
        ScanLimits::default(),
    )
    .unwrap();
    // Python's independent shift_jis codec encodes the complete known name.
    let hex = "666f6c6465725c835c8374836783458346834182cc90dd92e85c835c8374836783458346834182cc90dd92e8837483408343838b82c693fa967b8cea2e747874";
    let raw = (0..hex.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&hex[at..at + 2], 16).unwrap())
        .collect();
    index.records[0].raw_name = raw;
    index.records[0].flags = 0;
    let mut ansi = index.records[0].clone();
    ansi.flags = 2;
    ansi.raw_name = "另一组/说明.txt".as_bytes().to_vec();
    index.records.push(ansi);
    let rows = project(&index, &cancel, ScanLimits::default()).unwrap();
    assert_eq!(
        rows[0].name_bytes(),
        "folder/ソフトウェアの設定/ソフトウェアの設定ファイルと日本語.txt".as_bytes()
    );
    assert_eq!(rows[1].name_bytes(), "另一组/说明.txt".as_bytes());
    assert!(rows.iter().all(Entry::payload_supported));
}
