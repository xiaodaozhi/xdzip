use super::*;
use crate::{test_cases as cases, test_known as known};
use std::io::{Cursor, Read};
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/arc")
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
fn all_owned_rows_and_methods_are_scoped_frozen_and_fully_verified() {
    let cancel = Cancellation::default();
    for case in cases::CASES {
        let bytes = fixture(case.name);
        let mut source = Cursor::new(&bytes);
        assert_eq!(ArcCodec::candidate(&mut source, &cancel), Ok(true));
        assert_eq!(source.position(), 0);
        let rows = ArcCodec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 1);
        let row = &rows[0];
        assert_eq!(
            (row.format, row.method, row.size, row.packed_size),
            (
                Format::Arc,
                u16::from(case.method),
                case.decoded as u64,
                case.packed as u64
            )
        );
        assert_eq!(row.name_bytes(), b"HELLO.BIN");
        assert!(row.payload_supported());
        let mut reader = ArcCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink::default();
        reader.decode(row, &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, known::plain(case.kind));
        assert_eq!(xdzip_compression::crc16(&sink.0), case.crc);
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
        changed[0].format = Format::Zip;
        assert!(matches!(
            ArcCodec.open_payload(&mut source, &changed, &cancel),
            Err(DecodeError::Integrity)
        ));
    }
}
#[test]
fn unknown_methods_are_visible_without_cross_format_alias_or_payload_authority() {
    let cancel = Cancellation::default();
    let mut bytes = fixture("m2-phrase.arc");
    bytes[1] = 14;
    let mut source = Cursor::new(&bytes);
    let rows = ArcCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows[0].method, 14);
    assert!(!rows[0].payload_supported());
    let mut reader = ArcCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&rows[0], &mut sink, &cancel),
        Err(DecodeError::Unsupported)
    );
    assert!(sink.0.is_empty());
}
#[test]
fn names_are_decoded_before_dos_separators_and_suspicious_paths_have_no_authority() {
    let cancel = Cancellation::default();
    for (raw, effective, warning) in [
        (b"nest\\A.BIN".as_slice(), "nest/A.BIN", false),
        (b"../A.BIN".as_slice(), "../A.BIN", true),
        (b"A:/file".as_slice(), "A:/file", true),
        (b"folder/".as_slice(), "folder/", true),
        (
            b"\x83\x5c\x83\x5c\x83\x5c\x83\x5c.txt".as_slice(),
            "ソソソソ.txt",
            false,
        ),
    ] {
        let mut bytes = fixture("m2-phrase.arc");
        bytes[2..15].fill(0);
        bytes[2..2 + raw.len()].copy_from_slice(raw);
        let rows = ArcCodec
            .scan(&mut Cursor::new(&bytes), &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows[0].raw_name, raw);
        assert_eq!(rows[0].name_bytes(), effective.as_bytes());
        assert_eq!(rows[0].warning, warning);
        assert_eq!(rows[0].payload_supported(), !warning);
    }
    let bytes = fixture("m2-phrase.arc");
    assert!(matches!(
        ArcCodec.scan(
            &mut Cursor::new(&bytes),
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
fn proposals_rewind_short_sources_and_exact_empty_marker_requires_no_trailing_data() {
    let cancel = Cancellation::default();
    let bytes = fixture("m2-phrase.arc");
    for length in 0..29 {
        let mut source = Cursor::new(&bytes[..length]);
        assert_eq!(ArcCodec::candidate(&mut source, &cancel), Ok(false));
        assert_eq!(source.position(), 0);
    }
    for (bytes, expected) in [
        (vec![0x1a, 0], true),
        (vec![0x1a, 0, 0], false),
        (vec![0, 0], false),
    ] {
        let mut source = Cursor::new(bytes);
        assert_eq!(ArcCodec::candidate(&mut source, &cancel), Ok(expected));
        assert_eq!(source.position(), 0);
    }
    struct Fault(Cursor<Vec<u8>>);
    impl Read for Fault {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other(ScanError::Invalid))
        }
    }
    impl std::io::Seek for Fault {
        fn seek(&mut self, at: SeekFrom) -> std::io::Result<u64> {
            self.0.seek(at)
        }
    }
    impl ReadSource for Fault {}
    assert_eq!(
        ArcCodec::candidate(&mut Fault(Cursor::new(bytes)), &cancel),
        Err(ScanError::Io)
    );
}
