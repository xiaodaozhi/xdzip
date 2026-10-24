// crates/xdzip-codec-zip/src/names_tests.rs

use super::*;
use legacy_fixture as fixture;
use std::io::Cursor;
use xdzip_codec_api::{names::NameEncoding, payload::*};
fn scan(bytes: &[u8]) -> Vec<Entry> {
    ZipCodec
        .scan(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap()
}
#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, data: &[u8]) -> Result<(), DecodeError> {
        self.0.extend(data);
        Ok(())
    }
}
#[test]
fn independent_legacy_samples_resolve_full_paths_and_preserve_payload_identity() {
    for (encoding, sample) in fixture::samples() {
        let bytes = fixture::zip_sample(&sample, encoding, false);
        let rows = scan(&bytes);
        let control = Cancellation::default();
        let mut source = Cursor::new(&bytes);
        let mut reader = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
        for (row, (raw, expected)) in rows.iter().zip(sample) {
            assert_eq!(row.raw_name, raw);
            assert_eq!(
                row.display_name, expected,
                "{encoding}: {:?}",
                row.name_encoding
            );
            assert_eq!(row.name_bytes(), expected.as_bytes(), "{encoding}");
            assert!(!row.warning, "{encoding}");
            let mut sink = Sink::default();
            reader.decode(row, &mut sink, &control).unwrap();
            assert_eq!(
                sink.0,
                if row.directory {
                    &[][..]
                } else {
                    fixture::CONTENT
                }
            );
        }
    }
}
#[test]
fn unicode_precedence_crc_versions_conflicts_and_declared_utf8_damage() {
    let raw = b"fallback.txt";
    let unicode = "真实名称.txt".as_bytes();
    let extra = fixture::unicode_extra(raw, unicode);
    let rows = scan(&fixture::zip(&[(raw, 0, &extra)]));
    assert_eq!(rows[0].name_encoding, NameEncoding::UnicodePath);
    assert_eq!(rows[0].display_name, "真实名称.txt");
    assert_eq!(rows[0].raw_name, raw);
    let rows = scan(&fixture::zip(&[(raw, 2048, &extra)]));
    assert_eq!(rows[0].name_encoding, NameEncoding::Utf8);
    assert_eq!(rows[0].display_name, "fallback.txt");
    for mode in [0, 1] {
        let mut stale = extra.clone();
        if mode == 0 {
            stale[5] ^= 1;
        } else {
            stale[4] = 2;
        }
        assert_eq!(
            scan(&fixture::zip(&[(raw, 0, &stale)]))[0].display_name,
            "fallback.txt"
        );
    }
    let mut duplicate = extra.clone();
    duplicate.extend(&extra);
    let damaged = fixture::unicode_extra(raw, &[255]);
    for bad in [duplicate, damaged] {
        let rows = scan(&fixture::zip(&[(raw, 0, &bad)]));
        assert!(rows[0].warning);
        assert_eq!(rows[0].name_encoding, NameEncoding::Unresolved);
    }
    let rows = scan(&fixture::zip(&[(b"\xd6\xd0\xce\xc4.txt", 2048, &[])]));
    assert!(rows[0].warning);
    assert!(rows[0].display_name.starts_with("\\xD6"));
}
#[test]
fn unicode_names_cannot_bypass_path_review_and_payload_rechecks_local_extra() {
    let raw = b"fallback.txt";
    for name in [
        "../outside.txt",
        "/absolute.txt",
        "C:/absolute.txt",
        "bad\u{202e}.txt",
    ] {
        let bytes = fixture::zip(&[(raw, 0, &fixture::unicode_extra(raw, name.as_bytes()))]);
        assert!(scan(&bytes)[0].warning);
    }
    let extra = fixture::unicode_extra(raw, b"target-a.txt");
    let mut bytes = fixture::zip(&[(raw, 0, &extra)]);
    let rows = scan(&bytes);
    let at = 30 + raw.len() + extra.len() - 5;
    bytes[at] = b'b';
    let control = Cancellation::default();
    let mut source = Cursor::new(&bytes);
    let mut reader = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Integrity)
    );
    assert!(sink.0.is_empty());
    let mut forged = rows.clone();
    forged[0].decoded_name = Some("other.txt".into());
    let mut source = Cursor::new(fixture::zip(&[(raw, 0, &extra)]));
    assert!(matches!(
        ZipCodec.open_payload(&mut source, &forged, &control),
        Err(DecodeError::Integrity)
    ));
}
#[test]
fn decoded_name_budgets_and_cancellation_apply_before_index_publication() {
    let extra = fixture::unicode_extra(b"x", "名称".repeat(100).as_bytes());
    let bytes = fixture::zip(&[(b"x", 0, &extra)]);
    for limits in [
        ScanLimits {
            name_bytes: 100,
            ..ScanLimits::default()
        },
        ScanLimits {
            display_name_bytes: 100,
            ..ScanLimits::default()
        },
    ] {
        assert_eq!(
            ZipCodec.scan(&mut Cursor::new(&bytes), &Cancellation::default(), limits),
            Err(ScanError::Limit)
        );
    }
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        ZipCodec.scan(&mut Cursor::new(bytes), &cancel, ScanLimits::default()),
        Err(ScanError::Cancelled)
    );
}

#[test]
fn payload_reader_pins_the_effective_name_after_opening() {
    let raw = b"fallback.txt";
    let extra = fixture::unicode_extra(raw, "真实名称.txt".as_bytes());
    let bytes = fixture::zip(&[(raw, 0, &extra)]);
    let rows = scan(&bytes);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let mut reader = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut forged = rows[0].clone();
    forged.decoded_name = Some("other.txt".into());
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&forged, &mut sink, &control),
        Err(DecodeError::Integrity)
    );
    assert!(sink.0.is_empty());
    reader.decode(&rows[0], &mut sink, &control).unwrap();
    assert_eq!(sink.0, fixture::CONTENT);
}
