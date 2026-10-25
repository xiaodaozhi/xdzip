use super::*;
use std::io::{self, Cursor};
use xdzip_codec_api::payload::DecodedSink;

#[derive(Default)]
struct Meter {
    data: Vec<u8>,
    reserved: u64,
    credit: u64,
    work: u64,
    aux: u64,
    work_calls: usize,
}
impl DecodedSink for Meter {
    fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
        panic!("legacy output must consume private prepaid credit")
    }
    fn write_precharged_chunk(&mut self, data: &[u8]) -> Result<(), DecodeError> {
        self.credit = self
            .credit
            .checked_sub(data.len() as u64)
            .ok_or(DecodeError::Integrity)?;
        assert!(data.len() <= 8192);
        self.data.extend(data);
        Ok(())
    }
}
impl WorkOutput for Meter {
    fn reserve_work(&mut self, n: u64) -> Result<(), DecodeError> {
        self.reserved += n;
        assert!(self.reserved <= 8192);
        Ok(())
    }
    fn produced_work(&mut self, n: u64) -> Result<(), DecodeError> {
        self.reserved = self.reserved.checked_sub(n).ok_or(DecodeError::Integrity)?;
        self.work += n;
        self.credit += n;
        self.work_calls += 1;
        Ok(())
    }
    fn produced_aux(&mut self, n: u64) -> Result<(), DecodeError> {
        self.aux += n;
        Ok(())
    }
}

#[test]
fn real_lookahead_preserves_bits_and_never_invents_missing_input() {
    let mut source = Cursor::new([0xa6, 0x35, 0x7e, 0x81, 0x42]);
    let mut bits = Bits::new(&mut source);
    assert_eq!(bits.peek(8).unwrap(), Some(0xa6));
    assert_eq!(bits.get(3).unwrap(), 5);
    assert!(bits.byte_tail_at_least(5).unwrap());
    assert_eq!(bits.get(5).unwrap(), 6);
    assert!(!bits.byte_tail_at_least(5).unwrap());
    assert_eq!(bits.get(16).unwrap(), 0x357e);
    assert_eq!(bits.peek(17).unwrap(), None);
    assert_eq!(bits.get(16).unwrap(), 0x8142);
    assert_eq!(bits.get(1), Err(DecodeError::Integrity));
}

#[test]
fn typed_interrupted_and_cancelled_tail_keep_their_classification() {
    struct Fault;
    impl Read for Fault {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                xdzip_codec_api::ScanError::Limit,
            ))
        }
    }
    assert_eq!(Bits::new(&mut Fault).peek(8), Err(DecodeError::Limit));
    let control = Cancellation::default();
    control.cancel();
    assert_eq!(
        Bits::new(&mut Cursor::new([1, 2, 3])).drain(&control),
        Err(DecodeError::Cancelled)
    );
}

#[test]
fn actual_work_is_batched_and_history_reset_cannot_reveal_previous_file() {
    let control = Cancellation::default();
    let mut history = History::default();
    let mut sink = Meter::default();
    history.prepare(65536, false, &mut sink, &control).unwrap();
    let mut output = Delivery::new(10000);
    for i in 0..10000 {
        output
            .byte((i % 251) as u8, &mut history, &mut sink, &control)
            .unwrap();
    }
    output.finish(&mut sink, &control).unwrap();
    assert_eq!(
        sink.data,
        (0..10000).map(|i| (i % 251) as u8).collect::<Vec<_>>()
    );
    assert_eq!(
        (sink.work, sink.work_calls, sink.reserved, sink.credit),
        (10000, 2, 0, 0)
    );
    assert_eq!(sink.aux, 65536);
    history.prepare(65536, false, &mut sink, &control).unwrap();
    assert_eq!(sink.aux, 65536);
    assert_eq!(
        Delivery::new(1).copy(1, 1, &mut history, &mut sink, &control),
        Err(DecodeError::Integrity)
    );
}

#[test]
fn overlapping_copy_is_exact_and_failed_pending_bytes_only_charge_actual_work() {
    let control = Cancellation::default();
    let mut history = History::default();
    let mut sink = Meter::default();
    history.prepare(65536, false, &mut sink, &control).unwrap();
    let mut output = Delivery::new(21);
    output
        .byte(b'A', &mut history, &mut sink, &control)
        .unwrap();
    output
        .copy(1, 20, &mut history, &mut sink, &control)
        .unwrap();
    output.finish(&mut sink, &control).unwrap();
    assert_eq!(sink.data, vec![b'A'; 21]);
    let mut failed = Delivery::new(4);
    let mut sink = Meter::default();
    for byte in b"abc" {
        failed
            .byte(*byte, &mut history, &mut sink, &control)
            .unwrap();
    }
    failed.settle_error(&mut sink).unwrap();
    assert_eq!((sink.work, sink.reserved, sink.credit), (3, 0, 3));
    assert!(sink.data.is_empty());
    assert_eq!(
        failed.copy(1, 0, &mut history, &mut sink, &control),
        Err(DecodeError::Integrity)
    );
}
