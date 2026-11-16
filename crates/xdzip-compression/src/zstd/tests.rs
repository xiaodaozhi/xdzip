use super::*;
use std::io::{Cursor, Read};
fn byte(source: &mut Cursor<&[u8]>) -> u8 {
    let mut bytes = [0];
    source.read_exact(&mut bytes).unwrap();
    bytes[0]
}
fn field(source: &mut Cursor<&[u8]>, count: usize) -> u64 {
    let mut value = 0;
    for shift in 0..count {
        value |= u64::from(byte(source)) << (8 * shift);
    }
    value
}
// Test-only envelope unpacking isolates block state from the eventual stream adapter.
fn decode_blocks(bytes: &[u8]) -> Vec<u8> {
    let mut source = Cursor::new(bytes);
    assert_eq!(field(&mut source, 4), 0xfd2fb528);
    let descriptor = byte(&mut source);
    let single = descriptor & 32 != 0;
    let mut window = if single {
        0
    } else {
        let descriptor = byte(&mut source);
        let base = 1u64 << (10 + (descriptor >> 3));
        base + (base >> 3) * u64::from(descriptor & 7)
    };
    let dictionary_size = [0, 1, 2, 4][usize::from(descriptor & 3)];
    assert_eq!(field(&mut source, dictionary_size), 0);
    let mode = descriptor >> 6;
    let size_bytes = match mode {
        0 => usize::from(single),
        1 => 2,
        2 => 4,
        _ => 8,
    };
    let expected_size =
        (size_bytes != 0).then(|| field(&mut source, size_bytes) + if mode == 1 { 256 } else { 0 });
    if single {
        window = expected_size.unwrap();
    }
    let mut decoder = ZstdBlockDecoder::new(window, 128 << 20).unwrap();
    let mut output = Vec::new();
    loop {
        let header = field(&mut source, 3);
        let size = (header >> 3) as usize;
        let kind = (header >> 1) & 3;
        let actual = if kind == 1 { 1 } else { size };
        let mut payload = vec![0; actual];
        source.read_exact(&mut payload).unwrap();
        let decoded = match kind {
            0 => decoder.raw(&payload),
            1 => decoder.rle(payload[0], size),
            2 => decoder.decode_compressed(&payload),
            _ => panic!("reserved reference block"),
        };
        output.extend_from_slice(
            decoded.unwrap_or_else(|e| panic!("reference block at {}: {e}", source.position())),
        );
        if header & 1 != 0 {
            break;
        }
    }
    if descriptor & 4 != 0 {
        assert_eq!(
            field(&mut source, 4),
            u64::from(crate::xxhash64(&output) as u32)
        );
    }
    assert_eq!(source.position() as usize, bytes.len());
    if let Some(size) = expected_size {
        assert_eq!(output.len() as u64, size);
    }
    output
}
fn entropy(length: usize) -> Vec<u8> {
    let mut state = 0x726f6f74u32;
    (0..length)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            let value = state as u8;
            if value < 128 {
                97
            } else if value < 192 {
                98
            } else {
                32 + (value & 63)
            }
        })
        .collect()
}
#[test]
fn independent_standard_blocks_retain_entropy_history_and_repeat_offsets() {
    let phrase = b"abracadabra abracadabra\n".repeat(6);
    for (packed, plain) in [
        (
            include_bytes!("../../../../tests/fixtures/streams/empty.zst").as_slice(),
            vec![],
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/phrase.zst").as_slice(),
            phrase.clone(),
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/phrase-no-checksum.zst").as_slice(),
            phrase.clone(),
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/phrase-unknown-size.zst").as_slice(),
            phrase.clone(),
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/multiblock.zst").as_slice(),
            phrase.repeat(10000),
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/rle.zst").as_slice(),
            vec![0; 200000],
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/entropy.zst").as_slice(),
            entropy(700000),
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/entropy-level-1.zst").as_slice(),
            entropy(700000),
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/entropy-level-9.zst").as_slice(),
            entropy(700000),
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/entropy-level-19.zst").as_slice(),
            entropy(700000),
        ),
        (
            include_bytes!("../../../../tests/fixtures/streams/window-wrap.zst").as_slice(),
            entropy(50000),
        ),
    ] {
        assert_eq!(decode_blocks(packed), plain);
    }
}

#[test]
fn malformed_block_state_is_bounded_poisoned_and_cannot_be_reused() {
    assert!(ZstdBlockDecoder::new(u64::MAX, u64::MAX).is_err());
    assert!(ZstdBlockDecoder::new(1024, (4 << 20) + 1023).is_err());
    for bytes in [
        vec![],
        vec![0, 1, 0xfc, 1],
        vec![0, 255, 255, 255],
        vec![0xff, 0, 0, 0, 0],
        vec![0, 0, 1],
    ] {
        let mut decoder = ZstdBlockDecoder::new(128 << 10, 8 << 20).unwrap();
        assert!(decoder.decode_compressed(&bytes).is_err());
        assert!(decoder.raw(&[]).is_err());
        assert!(decoder.rle(0, 0).is_err());
    }
    let mut decoder = ZstdBlockDecoder::new(1024, 8 << 20).unwrap();
    assert_eq!(decoder.raw(&[1, 2, 3]).unwrap(), [1, 2, 3]);
    assert!(decoder.rle(0, 1025).is_err());
    assert!(decoder.raw(&[]).is_err());
    let mut decoder = ZstdBlockDecoder::new(0, 8 << 20).unwrap();
    assert!(decoder.raw(&[0]).is_err());
}
