use super::*;
use crate::tests::{fixture, refresh, with_extension};
use std::io::Cursor;

fn pma_fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/pma")
            .join(name),
    )
    .unwrap()
}
#[test]
fn pma_methods_are_scoped_and_frozen_without_lha_variant_trials() {
    let cancel = Cancellation::default();
    for (name, method, expected) in [
        ("pm0-allbytes.pma", 0, (0..=255).collect::<Vec<u8>>()),
        ("pm2-zeros.pma", 2, vec![0; 65536]),
        (
            "pm2-deep28.pma",
            2,
            (32..=39).rev().cycle().take(128).collect(),
        ),
    ] {
        let bytes = pma_fixture(name);
        let rows = PmaCodec
            .scan(&mut Cursor::new(&bytes), &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows[0].format, Format::Pma);
        assert_eq!(rows[0].method, method);
        assert!(rows[0].payload_supported());
        let mut source = Cursor::new(&bytes);
        let mut reader = PmaCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink::default();
        reader.decode(&rows[0], &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, expected);
        let mut changed = rows.clone();
        changed[0].format = Format::Lha;
        assert!(matches!(
            PmaCodec.open_payload(&mut Cursor::new(&bytes), &changed, &cancel),
            Err(DecodeError::Integrity)
        ));
    }
    let mut bytes = pma_fixture("pm0-allbytes.pma");
    bytes[5] = b'1';
    refresh(&mut bytes, 33);
    let rows = PmaCodec
        .scan(&mut Cursor::new(&bytes), &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows[0].method, 1);
    assert!(rows[0].warning && !rows[0].payload_supported());
    let mut source = Cursor::new(&bytes);
    let mut reader = PmaCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&rows[0], &mut sink, &cancel),
        Err(DecodeError::Unsupported)
    );
    assert!(sink.0.is_empty());
}
#[test]
fn pma_proposals_require_exact_complete_framing_and_do_not_expand_lha_padding() {
    let cancel = Cancellation::default();
    for name in ["empty.pma", "pm2-boundary-1024.pma"] {
        let bytes = pma_fixture(name);
        let mut source = Cursor::new(&bytes);
        assert_eq!(PmaCodec::candidate(&mut source, &cancel), Ok(true));
        assert_eq!(source.position(), 0);
        assert_eq!(LhaCodec::candidate(&mut source, &cancel), Ok(false));
        PmaCodec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert!(
            LhaCodec
                .scan(&mut source, &cancel, ScanLimits::default())
                .is_err()
        );
    }
    let mut bad = pma_fixture("empty.pma");
    *bad.last_mut().unwrap() = 0;
    assert_eq!(
        PmaCodec::candidate(&mut Cursor::new(&bad), &cancel),
        Ok(true)
    );
    assert!(
        PmaCodec
            .scan(&mut Cursor::new(&bad), &cancel, ScanLimits::default())
            .is_err()
    );
    assert_eq!(
        PmaCodec::candidate(&mut Cursor::new([0]), &cancel),
        Ok(false)
    );
}
#[test]
fn typed_invalid_source_failure_cannot_be_reinterpreted_as_a_weak_format_guess() {
    struct Fault;
    impl std::io::Read for Fault {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                ScanError::Invalid,
            ))
        }
    }
    impl std::io::Seek for Fault {
        fn seek(&mut self, at: std::io::SeekFrom) -> std::io::Result<u64> {
            Ok(if matches!(at, std::io::SeekFrom::End(_)) {
                128
            } else {
                0
            })
        }
    }
    impl ReadSource for Fault {}
    let cancel = Cancellation::default();
    assert_eq!(PmaCodec::candidate(&mut Fault, &cancel), Err(ScanError::Io));
    assert_eq!(LhaCodec::candidate(&mut Fault, &cancel), Err(ScanError::Io));
    assert_eq!(
        inspect_pma(&mut Fault, &cancel, ScanLimits::default()),
        Err(ScanError::Io)
    );
    assert_eq!(
        inspect(&mut Fault, &cancel, ScanLimits::default()),
        Err(ScanError::Io)
    );
}

#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}
fn rows(bytes: &[u8]) -> Vec<Entry> {
    LhaCodec
        .scan(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap()
}
#[test]
fn proposals_reset_the_source_and_never_replace_complete_validation() {
    let cancel = Cancellation::default();
    for (bytes, expected) in [
        (vec![0], true),
        (vec![], false),
        (b"xx-lh5-".to_vec(), true),
        (b"ordinary".to_vec(), false),
    ] {
        let mut source = Cursor::new(bytes);
        source.set_position(2);
        assert_eq!(LhaCodec::candidate(&mut source, &cancel), Ok(expected));
        assert_eq!(source.position(), 0);
    }
    assert!(
        LhaCodec
            .scan(&mut Cursor::new(b"xx-lh5-"), &cancel, ScanLimits::default())
            .is_err()
    );
}
#[test]
fn projected_names_methods_and_special_objects_control_payload_admission() {
    let named = rows(&fixture("named-shiftjis.lzh"));
    assert_eq!(
        named[0].decoded_name.as_deref(),
        Some("nested/ソフトウェアの設定/ソフトウェアの設定ファイルと日本語.txt")
    );
    assert!(named[0].raw_name.contains(&255));
    assert!(named[0].payload_supported());
    let link = rows(&with_extension(0x50, &0xa1ffu16.to_le_bytes()));
    assert!(link[0].link && link[0].warning && !link[0].payload_supported());
    assert_eq!(link[0].unix_mode, 0);
    let mut unknown = fixture("lh0-h0.lzh");
    unknown[2..7].copy_from_slice(b"-lhx-");
    refresh(&mut unknown, 33);
    let unknown = rows(&unknown);
    assert!(unknown[0].warning && !unknown[0].payload_supported());
    let mut ancient = fixture("lh2-h0.lzh");
    ancient.drain(31..33);
    ancient[0] -= 2;
    refresh(&mut ancient, 31);
    assert!(!rows(&ancient)[0].payload_supported());
}
#[test]
fn frozen_rows_are_compared_at_open_and_decode_and_errors_poison_the_instance() {
    let cancel = Cancellation::default();
    let bytes = fixture("lh5-h2.lzh");
    let snapshot = rows(&bytes);
    let mut changed = snapshot.clone();
    changed[0].display_name.push('x');
    assert!(matches!(
        LhaCodec.open_payload(&mut Cursor::new(&bytes), &changed, &cancel),
        Err(DecodeError::Integrity)
    ));
    let mut source = Cursor::new(&bytes);
    let mut reader = LhaCodec
        .open_payload(&mut source, &snapshot, &cancel)
        .unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(&changed[0], &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        reader.decode(&snapshot[0], &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
    assert!(sink.0.is_empty());
    let mut source = Cursor::new(&bytes);
    LhaCodec
        .open_payload(&mut source, &snapshot, &cancel)
        .unwrap()
        .decode(&snapshot[0], &mut sink, &cancel)
        .unwrap();
    assert_eq!(sink.0, b"abracadabra abracadabra\n".repeat(6));
}
