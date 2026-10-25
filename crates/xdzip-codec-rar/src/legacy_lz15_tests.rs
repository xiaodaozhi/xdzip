use super::*;
use std::io::Cursor;
use xdzip_codec_api::payload::DecodedSink;

#[derive(Default)]
struct Meter {
    data: Vec<u8>,
    reserved: u64,
    credit: u64,
    work: u64,
    aux: u64,
}
impl DecodedSink for Meter {
    fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
        panic!("legacy output must use previously settled work")
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.credit = self.credit.checked_sub(bytes.len() as u64).unwrap();
        self.data.extend(bytes);
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
        self.reserved = self.reserved.checked_sub(n).unwrap();
        self.work += n;
        self.credit += n;
        Ok(())
    }
    fn produced_aux(&mut self, n: u64) -> Result<(), DecodeError> {
        self.aux += n;
        Ok(())
    }
}

// These are isolated mathematical command vectors, not a RAR encoder.
fn vector(bits: &str) -> Cursor<Vec<u8>> {
    assert!(bits.bytes().all(|b| matches!(b, b'0' | b'1')));
    Cursor::new(
        bits.as_bytes()
            .chunks(8)
            .map(|part| {
                part.iter()
                    .enumerate()
                    .fold(0, |value, (i, b)| value | ((b - b'0') << (7 - i)))
            })
            .collect(),
    )
}
fn descriptor() -> Descriptor {
    Descriptor {
        method: 15,
        compression: 3,
        size: Some(0),
        ..Descriptor::default()
    }
}
fn prepared() -> Lz15 {
    let mut decoder = Lz15::default();
    decoder
        .begin(
            &descriptor(),
            &mut Meter::default(),
            &Cancellation::default(),
        )
        .unwrap();
    decoder
}

#[test]
fn eight_zero_length_prefix_consumes_its_full_following_octet() {
    let mut decoder = prepared();
    // Eight zero bits, value5, Hf0 rank0, seven-bit distance1, marker21.
    let mut source = vector("00000000000001010000000000110101");
    let mut input = Bits::new(&mut source);
    assert!(matches!(decoder.long(&mut input), Ok(Action::Copy(1, 16))));
    assert_eq!(input.get(5).unwrap(), 21);
    assert_eq!(decoder.last, Some((1, 16)));
    assert_eq!(decoder.recent[0], Some(1));
    assert_eq!(
        prepared().long(&mut Bits::new(&mut Cursor::new([0]))).err(),
        Some(DecodeError::Integrity)
    );
}

#[test]
fn special_escape_and_short_toggle_never_publish_or_change_match_history() {
    let mut decoder = prepared();
    decoder.special = true;
    decoder.last = Some((7, 9));
    decoder.recent = [Some(33), Some(2), Some(3), Some(4)];
    let recent = decoder.recent;
    // Hf1 zero, leave-special bit, then an untouched marker.
    let mut source = vector("00000110101010");
    let mut input = Bits::new(&mut source);
    assert!(matches!(decoder.huffman(&mut input), Ok(Action::None)));
    assert!(!decoder.special);
    assert_eq!(input.get(8).unwrap(), 0xaa);
    // Short slot10 followed by L1 symbol255 toggles the short codebook.
    let mut source = vector("1000111111111111");
    assert!(matches!(
        decoder.short(&mut Bits::new(&mut source)),
        Ok(Action::None)
    ));
    assert!(decoder.short_variant);
    assert_eq!(decoder.last, Some((7, 9)));
    assert_eq!(decoder.recent, recent);
    decoder.special = true;
    // Hf1 zero, match control0, length selector1, Hf2 rank1, low1.
    let mut source = vector("00000010000100001");
    assert!(matches!(
        decoder.huffman(&mut Bits::new(&mut source)),
        Ok(Action::Copy(33, 4))
    ));
    assert_eq!(decoder.last, Some((7, 9)));
    assert_eq!(decoder.recent, recent);
}

#[test]
fn repeat_of_last_and_missing_history_have_distinct_behaviour() {
    let mut decoder = prepared();
    decoder.last = Some((9, 12));
    decoder.recent = [Some(1), Some(2), Some(3), Some(4)];
    decoder.repeats = 2;
    let mut source = vector("11010101");
    let mut input = Bits::new(&mut source);
    assert!(matches!(decoder.short(&mut input), Ok(Action::Copy(9, 12))));
    assert_eq!(input.get(7).unwrap(), 0x55);
    assert_eq!(decoder.recent, [Some(1), Some(2), Some(3), Some(4)]);
    decoder.last = None;
    assert_eq!(decoder.repeat().err(), Some(DecodeError::Integrity));
}

#[test]
fn solid_boundary_preserves_adaptation_but_resets_per_file_control() {
    let mut decoder = prepared();
    decoder.average2 = 100;
    decoder.last = Some((3, 7));
    decoder.short_variant = true;
    decoder.special = true;
    decoder.repeats = 2;
    decoder.flag_left = 5;
    let mut d = descriptor();
    d.solid = true;
    let mut sink = Meter::default();
    decoder
        .begin(&d, &mut sink, &Cancellation::default())
        .unwrap();
    assert_eq!(decoder.average2, 100);
    assert_eq!(decoder.last, Some((3, 7)));
    assert!(decoder.short_variant);
    assert!(!decoder.special);
    assert_eq!((decoder.repeats, decoder.flag_left), (0, 0));
    assert_eq!(sink.aux, 0);
    d.solid = false;
    decoder
        .begin(&d, &mut sink, &Cancellation::default())
        .unwrap();
    assert_eq!(decoder.average2, 0);
    assert_eq!(decoder.last, None);
    assert!(!decoder.short_variant);
    assert_eq!(sink.aux, 0);
}

#[test]
fn cancelled_decode_is_poisoned_until_reset_and_version10_stays_unsupported() {
    let mut d = descriptor();
    d.method = 10;
    assert_eq!(
        Lz15::default().memory_required(&d),
        Err(DecodeError::Unsupported)
    );
    d.method = 15;
    let control = Cancellation::default();
    control.cancel();
    let mut decoder = Lz15::default();
    let mut sink = Meter::default();
    assert_eq!(
        decoder.decode(&d, &mut Cursor::new([]), &mut sink, &control, 0),
        Err(DecodeError::Cancelled)
    );
    assert_eq!(
        decoder.decode(
            &d,
            &mut Cursor::new([]),
            &mut sink,
            &Cancellation::default(),
            0
        ),
        Err(DecodeError::Integrity)
    );
    assert_eq!((sink.work, sink.credit, sink.data.len()), (0, 0, 0));
    decoder.reset();
    decoder
        .decode(
            &d,
            &mut Cursor::new([]),
            &mut sink,
            &Cancellation::default(),
            0,
        )
        .unwrap();
}
