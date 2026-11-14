use super::*;
use std::io::Cursor;
const PHRASE: &[u8] = b"abracadabra abracadabra\n";
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/streams")
            .join(name),
    )
    .unwrap()
}
fn decode(bytes: &[u8], chunk: usize) -> io::Result<Vec<u8>> {
    let mut reader = ZstdReader::new(Cursor::new(bytes), bytes.len() as u64, 128 << 20)?;
    let mut output = Vec::new();
    let mut buffer = vec![0; chunk];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            return Ok(output);
        }
        output.extend_from_slice(&buffer[..n]);
    }
}
#[test]
fn independent_reference_frames_declare_exact_checks_sizes_and_terminal_states() {
    for (name, expected) in [
        ("phrase.zst", PHRASE.repeat(6)),
        ("phrase-no-checksum.zst", PHRASE.repeat(6)),
        ("phrase-unknown-size.zst", PHRASE.repeat(6)),
        ("empty.zst", vec![]),
        ("multiple.zst", PHRASE.repeat(12)),
        ("leading-skip.zst", PHRASE.repeat(6)),
        ("rle.zst", vec![0; 200000]),
        ("multiblock.zst", PHRASE.repeat(60000)),
    ] {
        let bytes = fixture(name);
        for chunk in [1, 73, 65536] {
            assert_eq!(
                decode(&bytes, chunk).unwrap(),
                expected,
                "{name}, chunk {chunk}"
            );
        }
    }
    let mut state = 0x726f6f74u32;
    let expected: Vec<_> = (0..700000)
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
        .collect();
    for name in [
        "entropy.zst",
        "entropy-level-1.zst",
        "entropy-level-9.zst",
        "entropy-level-19.zst",
    ] {
        assert_eq!(decode(&fixture(name), 65536).unwrap(), expected, "{name}");
    }
    assert_eq!(
        decode(&fixture("window-wrap.zst"), 73).unwrap(),
        expected[..50000]
    );
    let mut state = 0x726f6f74u32;
    let random: Vec<u8> = (0..200000)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as u8
        })
        .collect();
    assert_eq!(decode(&fixture("random.zst"), 65536).unwrap(), random);
}
#[test]
fn truncation_mutation_and_trailing_garbage_cannot_succeed_or_resume() {
    let bytes = fixture("phrase.zst");
    for end in 0..bytes.len() {
        assert!(decode(&bytes[..end], 73).is_err(), "prefix {end}");
    }
    for at in 0..bytes.len() {
        for bit in [1, 2, 4, 8, 16, 32, 64, 128] {
            if at == 4 && bit == 16 {
                continue;
            } // Explicitly unused header bit is transparent.
            let mut bad = bytes.clone();
            bad[at] ^= bit;
            let mut reader =
                ZstdReader::new(Cursor::new(&bad), bad.len() as u64, 128 << 20).unwrap();
            let mut output = Vec::new();
            assert!(
                reader.read_to_end(&mut output).is_err(),
                "byte {at}, bit {bit}"
            );
            assert!(reader.read(&mut [0]).is_err());
        }
    }
    let mut unused = bytes.clone();
    unused[4] |= 16;
    assert_eq!(decode(&unused, 7).unwrap(), PHRASE.repeat(6));
    let mut bad = bytes.clone();
    bad.extend([0, 0, 0, 0]);
    assert!(decode(&bad, 73).is_err());
    assert!(decode(&fixture("only-skip.zst"), 73).is_err());
}
#[test]
fn dictionaries_window_admission_and_physical_input_ranges_are_explicit() {
    let bytes = fixture("phrase.zst");
    let mut zero_dictionary = bytes.clone();
    zero_dictionary[4] |= 1;
    zero_dictionary.insert(5, 0);
    assert_eq!(decode(&zero_dictionary, 7).unwrap(), PHRASE.repeat(6));
    zero_dictionary[5] = 1;
    assert_eq!(
        decode(&zero_dictionary, 7).unwrap_err().kind(),
        io::ErrorKind::Unsupported
    );
    let mut oversized = MAGIC.to_le_bytes().to_vec();
    oversized.extend([0, 0x88]);
    assert_eq!(
        decode(&oversized, 7).unwrap_err().kind(),
        io::ErrorKind::OutOfMemory
    );
    assert!(ZstdReader::new(Cursor::new(&bytes), bytes.len() as u64, OVERHEAD - 1).is_err());
    let mut source = bytes.clone();
    source.extend([42; 16]);
    let mut reader = ZstdReader::new(Cursor::new(source), bytes.len() as u64, 128 << 20).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, PHRASE.repeat(6));
    let mut reader =
        ZstdReader::new(Cursor::new(&bytes), bytes.len() as u64 + 1, 128 << 20).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    // A single-segment content size also controls window and block-size admission.
    let mut too_small = bytes;
    too_small[5] = 1;
    assert!(decode(&too_small, 7).is_err());
}

#[test]
fn item_and_block_limits_admit_the_exact_boundary_and_refuse_one_more() {
    for count in [MAX_ITEMS, MAX_ITEMS + 1] {
        let mut bytes = MAGIC.to_le_bytes().to_vec();
        bytes.extend([0, 0]);
        bytes.resize(6 + 3 * count, 0);
        *bytes.get_mut(6 + 3 * (count - 1)).unwrap() = 1;
        let result = decode(&bytes, 8192);
        if count == MAX_ITEMS {
            assert!(result.unwrap().is_empty());
        } else {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::OutOfMemory);
        }
    }
    for count in [MAX_ITEMS - 1, MAX_ITEMS] {
        let mut bytes = Vec::new();
        for _ in 0..count {
            bytes.extend(0x184d2a50u32.to_le_bytes());
            bytes.extend([0; 4]);
        }
        bytes.extend(fixture("empty.zst"));
        let result = decode(&bytes, 8192);
        if count == MAX_ITEMS - 1 {
            assert!(result.unwrap().is_empty());
        } else {
            assert_eq!(result.unwrap_err().kind(), io::ErrorKind::OutOfMemory);
        }
    }
}
