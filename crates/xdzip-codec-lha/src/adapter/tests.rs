use super::*;
use crate::tests::{fixture, refresh, with_extension};
use std::io::Cursor;

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
