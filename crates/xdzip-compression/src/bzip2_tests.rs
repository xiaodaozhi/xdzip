use super::*;
#[path = "bzip2_vectors.rs"]
mod vectors;

// Python stdlib bz2 compressed this project-owned phrase at level one.
const PHRASE: &[u8] = &[
    66, 90, 104, 49, 49, 65, 89, 38, 83, 89, 254, 136, 227, 182, 0, 0, 23, 209, 128, 0, 16, 64, 0,
    60, 0, 16, 0, 32, 0, 32, 165, 64, 208, 134, 3, 101, 133, 133, 213, 213, 194, 194, 240, 88, 47,
    139, 185, 34, 156, 40, 72, 127, 68, 113, 219, 0,
];
const EMPTY: &[u8] = &[66, 90, 104, 57, 23, 114, 69, 56, 80, 144, 0, 0, 0, 0];
fn phrase() -> Vec<u8> {
    b"abracadabra abracadabra\n".repeat(6)
}

struct Single<'a>(&'a [u8]);
impl Read for Single<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() || self.0.is_empty() {
            return Ok(0);
        }
        output[0] = self.0[0];
        self.0 = &self.0[1..];
        Ok(1)
    }
}
#[test]
fn bzip2_python_phrase_empty_single_byte_io_and_finish() {
    let mut reader = Bzip2Reader::new(Single(PHRASE), PHRASE.len() as u64, 144, 8 << 20).unwrap();
    assert_eq!(reader.consumed_bytes(), 4);
    let mut output = Vec::new();
    assert_eq!(reader.read(&mut [0; 0]).unwrap(), 0);
    let mut byte = [0];
    while reader.read(&mut byte).unwrap() != 0 {
        output.push(byte[0]);
    }
    assert_eq!(output, phrase());
    assert_eq!(reader.consumed_bytes(), PHRASE.len() as u64);
    assert_eq!(reader.decoded_bytes(), 144);
    reader.finish().unwrap();
    reader.finish().unwrap();
    let mut reader = Bzip2Reader::new(PHRASE, PHRASE.len() as u64, 144, 8 << 20).unwrap();
    let mut output = [0; 144];
    reader.read_exact(&mut output).unwrap();
    reader.finish().unwrap();
    assert_eq!(output.as_slice(), phrase());
    let mut reader = Bzip2Reader::new(EMPTY, EMPTY.len() as u64, 0, 8 << 20).unwrap();
    reader.finish().unwrap();
}

#[test]
fn bzip2_actual_header_memory_admission_and_invalid_headers() {
    assert_eq!(bzip2_memory_required(1).unwrap(), 565536);
    assert_eq!(bzip2_memory_required(9).unwrap(), 4565536);
    for level in [0, 10, 255] {
        assert!(bzip2_memory_required(level).is_err());
    }
    for level in 1..=9 {
        let mut empty = EMPTY.to_vec();
        empty[3] = b'0' + level;
        let budget = bzip2_memory_required(level).unwrap();
        let error = Bzip2Reader::new(empty.as_slice(), empty.len() as u64, 0, budget - 1)
            .err()
            .unwrap();
        assert_eq!(error.kind(), io::ErrorKind::OutOfMemory);
        let mut reader = Bzip2Reader::new(empty.as_slice(), empty.len() as u64, 0, budget).unwrap();
        assert_eq!(reader.consumed_bytes(), 4);
        reader.finish().unwrap();
    }
    for header in [b"BZh0", b"BZh:", b"BZx1", b"bZh1"] {
        assert!(Bzip2Reader::new(header.as_slice(), 4, 0, u64::MAX).is_err());
    }
}

#[test]
fn bzip2_truncation_crc_wrong_size_and_poison() {
    for cut in 0..PHRASE.len() {
        match Bzip2Reader::new(&PHRASE[..cut], cut as u64, 144, 8 << 20) {
            Err(_) => assert!(cut < 4),
            Ok(mut reader) => {
                assert!(reader.read_to_end(&mut Vec::new()).is_err(), "cut={cut}");
                assert!(reader.read(&mut [0]).is_err());
                assert!(reader.finish().is_err());
            }
        }
    }
    for size in [0, 143, 145] {
        let mut reader = Bzip2Reader::new(PHRASE, PHRASE.len() as u64, size, 8 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
    }
    for index in [10, PHRASE.len() - 2] {
        let mut bad = PHRASE.to_vec();
        bad[index] ^= 1;
        let mut reader = Bzip2Reader::new(bad.as_slice(), bad.len() as u64, 144, 8 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err(), "byte={index}");
        assert!(reader.finish().is_err());
    }
    let mut reader = Bzip2Reader::new(PHRASE, PHRASE.len() as u64, 144, 8 << 20).unwrap();
    assert!(reader.finish().is_err());
    assert!(reader.read(&mut [0]).is_err());
}

#[test]
fn bzip2_single_stream_exact_span_and_padding_policy() {
    for suffix in [&[0][..], EMPTY, PHRASE] {
        let mut bytes = PHRASE.to_vec();
        bytes.extend_from_slice(suffix);
        let mut reader =
            Bzip2Reader::new(bytes.as_slice(), bytes.len() as u64, 144, 8 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
        let mut reader =
            Bzip2Reader::new(bytes.as_slice(), PHRASE.len() as u64, 144, 8 << 20).unwrap();
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, phrase());
    }
    let mut padding = PHRASE.to_vec();
    *padding.last_mut().unwrap() |= 1;
    let mut reader =
        Bzip2Reader::new(padding.as_slice(), padding.len() as u64, 144, 8 << 20).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, phrase());
}

#[test]
fn bzip2_crc_polynomial_and_incomplete_huffman_trees() {
    let mut crc = u32::MAX;
    for &byte in b"123456789" {
        crc = crc_byte(crc, byte);
    }
    assert_eq!(!crc, 0xfc89_1918);
    assert!(Huffman::build(&[1, 1, 1]).is_err());
    assert!(Huffman::build(&[0, 1]).is_err());
    assert!(Huffman::build(&[21, 1]).is_err());
    let tree = Huffman::build(&[2, 2, 3]).unwrap();
    assert_eq!((tree.first[2], tree.first[3]), (0, 4));
    let mut bits = Bits {
        input: Input::new(&[0xff][..], 1),
        pending: 0,
        available: 0,
    };
    assert!(tree.decode(&mut bits).is_err(), "Huffman hole is invalid");
    assert!(
        Huffman::build(&[20; 258]).is_ok(),
        "unused incomplete dummy table is admitted"
    );
}

// A deliberately small mathematical test author. It sorts cyclic rotations
// directly, uses fixed-width canonical codes, and never calls a decoder. This
// is test-data construction, not a product compression path.
struct Author {
    bytes: Vec<u8>,
    bits: usize,
}
impl Author {
    fn new() -> Self {
        Self {
            bytes: b"BZh1".to_vec(),
            bits: 32,
        }
    }
    fn put(&mut self, value: u64, width: u8) {
        for shift in (0..width).rev() {
            if self.bits.is_multiple_of(8) {
                self.bytes.push(0);
            }
            self.bytes[self.bits / 8] |= (((value >> shift) & 1) as u8) << (7 - self.bits % 8);
            self.bits += 1;
        }
    }
    fn block(
        &mut self,
        plain: &[u8],
        randomized: bool,
        extra_selectors: usize,
        alternating: bool,
        omit_last_count: bool,
    ) -> usize {
        let mut rle = Vec::new();
        let mut position = 0;
        while position < plain.len() {
            let byte = plain[position];
            let mut count = 1;
            while position + count < plain.len() && plain[position + count] == byte && count < 259 {
                count += 1;
            }
            if count < 4 {
                rle.extend(std::iter::repeat_n(byte, count));
            } else {
                rle.extend_from_slice(&[byte; 4]);
                rle.push((count - 4) as u8);
            }
            position += count;
        }
        if omit_last_count {
            assert_eq!(rle.pop(), Some(0));
        }
        if randomized {
            let mut timer = 0;
            let mut index = 0;
            for byte in &mut rle {
                if timer == 0 {
                    timer = RANDOM_PERIODS[index];
                    index = (index + 1) % 512;
                }
                timer -= 1;
                if timer == 1 {
                    *byte ^= 1;
                }
            }
        }
        let n = rle.len();
        assert!(n != 0);
        let mut rotations: Vec<usize> = (0..n).collect();
        rotations.sort_by(|&a, &b| {
            rle[a..]
                .iter()
                .chain(rle[..a].iter())
                .cmp(rle[b..].iter().chain(rle[..b].iter()))
        });
        let origin = rotations.iter().position(|&value| value == 0).unwrap();
        let last: Vec<u8> = rotations
            .iter()
            .map(|&start| rle[(start + n - 1) % n])
            .collect();
        let mut used = [false; 256];
        for &byte in &last {
            used[byte as usize] = true;
        }
        let mut order: Vec<u8> = (0..=255).filter(|&value| used[value as usize]).collect();
        let alphabet = order.len();
        let mut tokens = Vec::new();
        let mut run = 0usize;
        fn flush(tokens: &mut Vec<usize>, mut run: usize) {
            while run != 0 {
                let symbol = if run & 1 != 0 { 0 } else { 1 };
                tokens.push(symbol);
                run = (run - symbol - 1) / 2;
            }
        }
        for byte in last {
            let index = order.iter().position(|&value| value == byte).unwrap();
            if index == 0 {
                run += 1;
                continue;
            }
            flush(&mut tokens, run);
            run = 0;
            tokens.push(index + 1);
            order[..=index].rotate_right(1);
        }
        flush(&mut tokens, run);
        tokens.push(alphabet + 1);
        let groups = tokens.len().div_ceil(50);
        let selectors = groups + extra_selectors;
        assert!(selectors <= 32767);
        let mut crc = u32::MAX;
        for &byte in plain {
            crc = crc_byte(crc, byte);
        }
        self.put(BLOCK_MAGIC, 48);
        self.put(u64::from(!crc), 32);
        self.put(u64::from(randomized), 1);
        self.put(origin as u64, 24);
        for group in 0..16 {
            self.put(
                u64::from(used[group * 16..group * 16 + 16].iter().any(|&value| value)),
                1,
            );
        }
        for group in 0..16 {
            if used[group * 16..group * 16 + 16].iter().any(|&value| value) {
                for &value in &used[group * 16..group * 16 + 16] {
                    self.put(u64::from(value), 1);
                }
            }
        }
        self.put(2, 3);
        self.put(selectors as u64, 15);
        let mut table_order = [0, 1];
        for selector in 0..selectors {
            let table = if alternating && selector < groups {
                selector % 2
            } else {
                0
            };
            let index = table_order
                .iter()
                .position(|&value| value == table)
                .unwrap();
            for _ in 0..index {
                self.put(1, 1);
            }
            self.put(0, 1);
            table_order[..=index].rotate_right(1);
        }
        let width = (usize::BITS - (alphabet + 1).leading_zeros()) as u8;
        for table in 0..2 {
            self.put(
                u64::from(if table == 1 && !alternating {
                    20
                } else {
                    width
                }),
                5,
            );
            for _ in 0..alphabet + 2 {
                self.put(0, 1);
            }
        }
        for token in tokens {
            self.put(token as u64, width);
        }
        self.bits
    }
    fn finish(mut self, plains: &[&[u8]]) -> Vec<u8> {
        let mut combined = 0u32;
        for plain in plains {
            let mut crc = u32::MAX;
            for &byte in *plain {
                crc = crc_byte(crc, byte);
            }
            combined = combined.rotate_left(1) ^ !crc;
        }
        self.put(END_MAGIC, 48);
        self.put(u64::from(combined), 32);
        self.bytes
    }
}

fn random_plain() -> Vec<u8> {
    let mut plain: Vec<u8> = (0..613).map(|index| ((index * 37) % 251) as u8).collect();
    plain.extend_from_slice(&[254; 259]); // RLE count at byte 618 must be derandomized too.
    plain.extend((0..700).map(|index| ((index * 61 + 7) % 251) as u8));
    plain.extend_from_slice(&[253; 4]);
    plain
}

#[test]
fn bzip2_mathematical_randomized_rle_and_combined_blocks() {
    let first = random_plain();
    let second = b"non-byte-aligned second block\n".repeat(3);
    let mut author = Author::new();
    let boundary = author.block(&first, true, 18010, true, false);
    assert!(!boundary.is_multiple_of(8));
    author.block(&second, false, 2, false, false);
    let packed = author.finish(&[&first, &second]);
    let expected = [first, second].concat();
    let mut reader = Bzip2Reader::new(
        packed.as_slice(),
        packed.len() as u64,
        expected.len() as u64,
        8 << 20,
    )
    .unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, expected);
    reader.finish().unwrap();
    // Four repeated bytes followed by a zero count are ordinary author output.
    let mut author = Author::new();
    author.block(b"AAAA", false, 0, false, false);
    let packed = author.finish(&[b"AAAA"]);
    let mut reader = Bzip2Reader::new(packed.as_slice(), packed.len() as u64, 4, 8 << 20).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, b"AAAA");
}

#[test]
fn bzip2_fixed_independently_read_math_inputs_and_7z_padding() {
    let randomized = random_plain();
    let second = b"non-byte-aligned second block\n".repeat(3);
    for (fixture, expected, padding) in [
        (&vectors::RANDOMIZED_COUNT, randomized.clone(), 4),
        (&vectors::EXTRA_SELECTORS, second.clone(), 2),
        (&vectors::FOUR_WITHOUT_COUNT, b"AAAA".to_vec(), 6),
        (&vectors::FOUR_WITH_ZERO_COUNT, b"AAAA".to_vec(), 0),
        (&vectors::TWO_BLOCKS, [randomized, second].concat(), 4),
        (
            &vectors::TWO_BLOCKS_FIRST_FOUR_WITHOUT_COUNT,
            [
                &b"AAAA"[..],
                &b"normal block after four-count boundary\n"[..],
            ]
            .concat(),
            5,
        ),
    ] {
        let original = vectors::bytes(fixture);
        for nonzero in [false, true] {
            if nonzero && padding == 0 {
                continue;
            }
            let mut packed = original.clone();
            if nonzero {
                *packed.last_mut().unwrap() |= (1 << padding) - 1;
            }
            let mut reader = Bzip2Reader::new(
                packed.as_slice(),
                packed.len() as u64,
                expected.len() as u64,
                8 << 20,
            )
            .unwrap();
            let mut output = Vec::new();
            reader.read_to_end(&mut output).unwrap();
            reader.finish().unwrap();
            assert_eq!(output, expected, "{} nonzero={nonzero}", fixture.name);
            assert_eq!(reader.consumed_bytes(), packed.len() as u64);
        }
    }
    // The old-randomization flag is meaningful; clearing it without changing
    // the randomized data must not pass the original block CRC.
    let mut packed = vectors::bytes(&vectors::RANDOMIZED_COUNT);
    packed[14] &= 0x7f;
    let mut reader = Bzip2Reader::new(
        packed.as_slice(),
        packed.len() as u64,
        random_plain().len() as u64,
        8 << 20,
    )
    .unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
}

#[test]
fn bzip2_missing_rle_count_compatibility_and_bounded_bit_mutations() {
    let mut author = Author::new();
    author.block(b"AAAA", false, 0, false, true);
    let packed = author.finish(&[b"AAAA"]);
    let mut reader = Bzip2Reader::new(packed.as_slice(), packed.len() as u64, 4, 8 << 20).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, b"AAAA");
    reader.finish().unwrap();
    for size in [3, 5] {
        let mut reader =
            Bzip2Reader::new(packed.as_slice(), packed.len() as u64, size, 8 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
    }
    let mut bad_crc = packed;
    bad_crc[10] ^= 1;
    let mut reader =
        Bzip2Reader::new(bad_crc.as_slice(), bad_crc.len() as u64, 4, 8 << 20).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    for bit in 0..PHRASE.len() * 8 {
        let mut packed = PHRASE.to_vec();
        packed[bit / 8] ^= 1 << (7 - bit % 8);
        if let Ok(mut reader) =
            Bzip2Reader::new(packed.as_slice(), packed.len() as u64, 144, 8 << 20)
        {
            let mut output = Vec::new();
            match reader.read_to_end(&mut output) {
                Ok(_) => {
                    assert_eq!(output, phrase(), "accepted mutation bit {bit}");
                    reader.finish().unwrap();
                }
                Err(_) => {
                    assert!(reader.read(&mut [0]).is_err());
                    assert!(reader.finish().is_err());
                }
            }
        }
    }
}

#[test]
fn bzip2_selectors_length_bounds_and_token_group_boundary() {
    fn entropy_reader(author: Author) -> Bzip2Reader<std::io::Cursor<Vec<u8>>> {
        let size = author.bytes.len() as u64;
        Bzip2Reader::new(std::io::Cursor::new(author.bytes), size, 1, 8 << 20).unwrap()
    }
    for tables in [0, 1, 7] {
        let mut author = Author::new();
        author.put(tables, 3);
        author.put(1, 15);
        assert!(entropy_reader(author).entropy(3).is_err());
    }
    let mut author = Author::new();
    author.put(2, 3);
    author.put(0, 15);
    assert!(entropy_reader(author).entropy(3).is_err());
    for selector_count in [1, 18003] {
        let mut author = Author::new();
        author.put(2, 3);
        author.put(selector_count, 15);
        for _ in 1..selector_count {
            author.put(0, 1);
        }
        author.put(3, 2); // unary reaches two, including an unused selector after the storage bound.
        assert!(entropy_reader(author).entropy(3).is_err());
    }
    for (initial, adjustment) in [(0, None), (21, None), (1, Some(1)), (20, Some(0))] {
        let mut author = Author::new();
        author.put(2, 3);
        author.put(1, 15);
        author.put(0, 1);
        author.put(initial, 5);
        if let Some(bit) = adjustment {
            author.put(1, 1);
            author.put(bit, 1);
        }
        assert!(entropy_reader(author).entropy(3).is_err());
    }
    let mut author = Author::new();
    author.put(2, 3);
    author.put(32767, 15);
    for _ in 0..32767 {
        author.put(0, 1);
    }
    for _ in 0..2 {
        author.put(2, 5);
        for _ in 0..3 {
            author.put(0, 1);
        }
    }
    let mut reader = entropy_reader(author);
    assert_eq!(reader.entropy(3).unwrap().selector_count, SELECTORS);
    // Every Huffman symbol, including EOB, consumes a group position. The 51st
    // symbol needs another selector even if it terminates the block.
    let mut encoded = [0u8; 13];
    encoded[12] = 0b0000_1000;
    let mut bits = Bits {
        input: Input::new(encoded.as_slice(), 13),
        pending: 0,
        available: 0,
    };
    let mut entropy = Entropy {
        tables: vec![
            Huffman::build(&[2, 2, 2]).unwrap(),
            Huffman::build(&[2, 2, 2]).unwrap(),
        ],
        selector: 0,
        left: 0,
        active: 0,
        selector_count: 1,
    };
    for _ in 0..50 {
        assert_eq!(entropy.next(&mut bits, &[0, 1]).unwrap(), 0);
    }
    let consumed = bits.input.consumed;
    assert!(entropy.next(&mut bits, &[0, 1]).is_err());
    assert_eq!(bits.input.consumed, consumed);
    entropy.selector_count = 2;
    assert_eq!(entropy.next(&mut bits, &[0, 1]).unwrap(), 2);
    assert_eq!(entropy.active, 1);
}

fn run_wire(tokens: &[u16], origin: usize, plain: &[u8]) -> Vec<u8> {
    let mut author = Author::new();
    let mut crc = u32::MAX;
    for &byte in plain {
        crc = crc_byte(crc, byte);
    }
    author.put(BLOCK_MAGIC, 48);
    author.put(u64::from(!crc), 32);
    author.put(0, 1);
    author.put(origin as u64, 24);
    author.put(1 << 11, 16);
    author.put(1 << 14, 16); // Alphabet is byte 65.
    author.put(2, 3);
    author.put(1, 15);
    author.put(0, 1);
    for width in [2, 20] {
        author.put(width, 5);
        for _ in 0..3 {
            author.put(0, 1);
        }
    }
    for &token in tokens {
        author.put(u64::from(token), 2);
    }
    author.put(2, 2);
    author.finish(&[plain])
}

#[test]
fn bzip2_run_arithmetic_bwt_period_origin_and_huffman_holes() {
    for (tokens, count, origin) in [
        (&[0][..], 1, 0),
        (&[1][..], 2, 1),
        (&[0, 0][..], 3, 2),
        (&[1, 0][..], 4, 3),
        (&[0, 1][..], 69, 4),
    ] {
        let plain = vec![b'A'; count];
        let packed = run_wire(tokens, origin, &plain);
        let mut reader = Bzip2Reader::new(
            packed.as_slice(),
            packed.len() as u64,
            count as u64,
            8 << 20,
        )
        .unwrap();
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, plain);
        reader.finish().unwrap();
    }
    for packed in [
        run_wire(&[0], 1, b"A"),
        run_wire(&[0], 100000, b"A"),
        run_wire(&[0; 17], 0, b"A"),
        run_wire(&[3], 0, b"A"),
    ] {
        let mut reader =
            Bzip2Reader::new(packed.as_slice(), packed.len() as u64, 1, 8 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
        assert!(reader.finish().is_err());
    }
    let mut packed = run_wire(&[0], 0, b"A");
    for bit in 137..153 {
        packed[bit / 8] &= !(1 << (7 - bit % 8));
    }
    let mut reader = Bzip2Reader::new(packed.as_slice(), packed.len() as u64, 1, 8 << 20).unwrap();
    assert!(
        reader.read_to_end(&mut Vec::new()).is_err(),
        "data block cannot have an empty alphabet"
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Signal {
    SourceChanged,
    Cancelled,
    Limit,
}
impl std::fmt::Display for Signal {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(output, "{self:?}")
    }
}
impl std::error::Error for Signal {}
struct FaultSource<'a> {
    bytes: &'a [u8],
    before: usize,
    signal: Option<Signal>,
    raised: bool,
    calls: std::rc::Rc<std::cell::Cell<usize>>,
}
impl Read for FaultSource<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.calls.set(self.calls.get() + 1);
        if self.before == 0 && !self.raised {
            self.raised = true;
            return Err(match self.signal {
                Some(signal) => io::Error::new(io::ErrorKind::Interrupted, signal),
                None => io::Error::from(io::ErrorKind::Interrupted),
            });
        }
        let count = output.len().min(self.bytes.len()).min(if self.raised {
            usize::MAX
        } else {
            self.before
        });
        output[..count].copy_from_slice(&self.bytes[..count]);
        self.bytes = &self.bytes[count..];
        self.before = self.before.saturating_sub(count);
        Ok(count)
    }
}
fn typed(error: io::Error, signal: Signal) {
    assert_eq!(error.kind(), io::ErrorKind::Other);
    assert_eq!(
        error.get_ref().unwrap().downcast_ref::<Signal>(),
        Some(&signal)
    );
}

#[test]
fn bzip2_typed_init_and_partial_source_errors_survive_read_helpers() {
    let first = b"first typed input block\n".repeat(3);
    let second = b"second input block\n".repeat(2);
    let mut author = Author::new();
    let boundary = author.block(&first, false, 0, false, false);
    author.block(&second, false, 0, false, false);
    let packed = author.finish(&[&first, &second]);
    let size = (first.len() + second.len()) as u64;
    for signal in [Signal::SourceChanged, Signal::Cancelled, Signal::Limit] {
        for before in [0, 2] {
            let source = FaultSource {
                bytes: &packed,
                before,
                signal: Some(signal),
                raised: false,
                calls: std::rc::Rc::new(std::cell::Cell::new(0)),
            };
            match Bzip2Reader::new(source, packed.len() as u64, size, 8 << 20) {
                Ok(_) => panic!("typed constructor failure was retried"),
                Err(error) => typed(error, signal),
            }
        }
        for partial in [false, true] {
            for method in 0..3 {
                let calls = std::rc::Rc::new(std::cell::Cell::new(0));
                let source = FaultSource {
                    bytes: &packed,
                    before: if partial { boundary.div_ceil(8) } else { 4 },
                    signal: Some(signal),
                    raised: false,
                    calls: calls.clone(),
                };
                let mut reader =
                    Bzip2Reader::new(source, packed.len() as u64, size, 8 << 20).unwrap();
                if partial {
                    let mut byte = [0];
                    reader.read_exact(&mut byte).unwrap();
                    assert_eq!(byte[0], first[0]);
                }
                let error = match method {
                    0 => loop {
                        match reader.read(&mut [0; 16]) {
                            Err(error) => break error,
                            Ok(count) => assert!(count != 0),
                        }
                    },
                    1 => reader.read_to_end(&mut Vec::new()).unwrap_err(),
                    _ => reader.read_exact(&mut vec![0; size as usize]).unwrap_err(),
                };
                typed(error, signal);
                if partial {
                    assert!(reader.decoded_bytes() > 0);
                }
                let before = calls.get();
                assert!(reader.read(&mut [0]).is_err());
                assert!(reader.finish().is_err());
                assert_eq!(calls.get(), before);
            }
        }
    }
    for before in [0, 2, 4, boundary.div_ceil(8)] {
        let source = FaultSource {
            bytes: &packed,
            before,
            signal: None,
            raised: false,
            calls: std::rc::Rc::new(std::cell::Cell::new(0)),
        };
        let mut reader = Bzip2Reader::new(source, packed.len() as u64, size, 8 << 20).unwrap();
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, [first.clone(), second.clone()].concat());
    }
}

#[test]
fn bzip2_combined_crc_checks_block_order_independently() {
    let first = b"first CRC block\n";
    let second = b"different CRC block\n";
    let mut author = Author::new();
    author.block(second, false, 0, false, false);
    author.block(first, false, 0, false, false);
    // Both block CRCs remain correct, but the final checksum belongs to the
    // opposite block order and must reject after all declared output bytes.
    let packed = author.finish(&[first, second]);
    let size = (first.len() + second.len()) as u64;
    let mut reader =
        Bzip2Reader::new(packed.as_slice(), packed.len() as u64, size, 8 << 20).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    assert_eq!(reader.decoded_bytes(), size);
    assert_eq!(reader.consumed_bytes(), packed.len() as u64);
}

#[test]
#[ignore = "exports self-authored BZip2 mathematical streams for independent official readback"]
fn bzip2_export_mathematical_streams() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_BZIP2_VECTORS").expect("vector directory"),
    )
    .join("math");
    std::fs::create_dir_all(&directory).unwrap();
    let first = random_plain();
    let second = b"non-byte-aligned second block\n".repeat(3);
    for (name, randomized, extra, alternating, omit, plain) in [
        ("randomized-count", true, 0, true, false, first.as_slice()),
        (
            "extra-selectors",
            false,
            18010,
            false,
            false,
            second.as_slice(),
        ),
        ("four-without-count", false, 0, false, true, &b"AAAA"[..]),
        ("four-with-zero-count", false, 0, false, false, &b"AAAA"[..]),
    ] {
        let mut author = Author::new();
        author.block(plain, randomized, extra, alternating, omit);
        let packed = author.finish(&[plain]);
        std::fs::write(directory.join(format!("{name}.bz2")), packed).unwrap();
        std::fs::write(directory.join(format!("{name}.plain")), plain).unwrap();
    }
    let mut author = Author::new();
    author.block(&first, true, 18010, true, false);
    author.block(&second, false, 2, false, false);
    std::fs::write(
        directory.join("two-blocks.bz2"),
        author.finish(&[&first, &second]),
    )
    .unwrap();
    std::fs::write(directory.join("two-blocks.plain"), [first, second].concat()).unwrap();
    let second = b"normal block after four-count boundary\n";
    let mut author = Author::new();
    author.block(b"AAAA", false, 0, false, true);
    author.block(second, false, 0, false, false);
    std::fs::write(
        directory.join("two-blocks-first-four-without-count.bz2"),
        author.finish(&[b"AAAA", second]),
    )
    .unwrap();
    std::fs::write(
        directory.join("two-blocks-first-four-without-count.plain"),
        [&b"AAAA"[..], second].concat(),
    )
    .unwrap();
}

#[test]
#[ignore = "requires independently prepared Python bz2 streams"]
fn bzip2_independent_python_matrix() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_BZIP2_VECTORS").expect("vector directory"),
    );
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    let mut cases = 0;
    for path in entries {
        if path.extension().is_none_or(|extension| extension != "bz2") {
            continue;
        }
        let packed = std::fs::read(&path).unwrap();
        let expected = std::fs::read(path.with_extension("plain")).unwrap();
        let mut reader = Bzip2Reader::new(
            Single(&packed),
            packed.len() as u64,
            expected.len() as u64,
            8 << 20,
        )
        .unwrap();
        let mut output = Vec::new();
        let mut chunk = [0; 32768];
        loop {
            let count = reader.read(&mut chunk).unwrap();
            if count == 0 {
                break;
            }
            assert!(count <= 8192);
            output.extend_from_slice(&chunk[..count]);
        }
        assert_eq!(output, expected, "{}", path.display());
        reader.finish().unwrap();
        assert_eq!(reader.consumed_bytes(), packed.len() as u64);
        cases += 1;
    }
    assert!(cases >= 10);
}
