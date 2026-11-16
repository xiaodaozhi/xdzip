use super::{bits::Bits, prefix::Prefix};
use std::io::Read;
use std::io::{self, Cursor};
#[derive(Default)]
struct Wire {
    bytes: Vec<u8>,
    used: u8,
}
impl Wire {
    fn put(&mut self, value: u32, count: u8) {
        for n in 0..count {
            if self.used == 0 {
                self.bytes.push(0);
            }
            let at = self.bytes.len() - 1;
            self.bytes[at] |= (((value >> n) & 1) as u8) << self.used;
            self.used = (self.used + 1) % 8;
        }
    }
    fn code(&mut self, value: u32, count: u8) {
        self.put(value.reverse_bits() >> (32 - count), count);
    }
    fn bits(&self) -> Bits<Cursor<&[u8]>> {
        Bits::new(Cursor::new(&self.bytes), self.bytes.len() as u64, 1_000_000)
    }
}
#[test]
fn exact_bit_fields_zero_fill_and_typed_terminal_input_errors() {
    let mut wire = Wire::default();
    wire.put(0xabc, 12);
    wire.put(0xfedcba, 24);
    wire.put(0, 4);
    let mut bits = wire.bits();
    assert_eq!(bits.get(12).unwrap(), 0xabc);
    assert_eq!(bits.get(24).unwrap(), 0xfedcba);
    bits.finish().unwrap();
    let mut bits = Bits::new(Cursor::new([0x80]), 1, 100);
    bits.get(1).unwrap();
    assert!(bits.finish().is_err());
    let mut bits = Bits::new(Cursor::new([0, 0]), 2, 100);
    bits.get(1).unwrap();
    assert!(bits.finish().is_err());
    let mut bits = Bits::new(Cursor::new([1]), 1, 0);
    assert_eq!(bits.get(1).unwrap_err().kind(), io::ErrorKind::OutOfMemory);
    struct Fail;
    impl io::Read for Fail {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::other("owned-brotli-input"))
        }
    }
    let mut bits = Bits::new(Fail, 1, 100);
    assert_eq!(bits.get(1).unwrap_err().to_string(), "owned-brotli-input");
}
#[test]
fn every_simple_tree_shape_keeps_canonical_order_and_zero_bit_singletons() {
    for (symbols, lens, select) in [
        (vec![200], vec![0], None),
        (vec![200, 3], vec![1, 1], None),
        (vec![200, 3, 91], vec![1, 2, 2], None),
        (vec![200, 3, 91, 9], vec![2, 2, 2, 2], Some(0)),
        (vec![200, 3, 91, 9], vec![1, 2, 3, 3], Some(1)),
    ] {
        let mut wire = Wire::default();
        wire.put(1, 2);
        wire.put(symbols.len() as u32 - 1, 2);
        for &s in &symbols {
            wire.put(s, 8);
        }
        if let Some(s) = select {
            wire.put(s, 1);
        }
        let mut assignments = vec![];
        let mut code = 0;
        for len in 1..=3 {
            let mut active: Vec<_> = symbols
                .iter()
                .zip(&lens)
                .filter(|&(_, &l)| l == len)
                .map(|(&s, _)| s)
                .collect();
            active.sort();
            for s in active {
                assignments.push((s, code, len));
                code += 1;
            }
            code <<= 1;
        }
        for &s in &symbols {
            if symbols.len() > 1 {
                let &(_, c, n) = assignments.iter().find(|&&(v, _, _)| v == s).unwrap();
                wire.code(c, n);
            }
        }
        let mut bits = wire.bits();
        let tree = Prefix::read(&mut bits, 256).unwrap();
        for s in symbols {
            assert_eq!(tree.symbol(&mut bits).unwrap(), s as u16);
        }
        bits.finish().unwrap();
    }
    for values in [[258, 0], [3, 3]] {
        let mut wire = Wire::default();
        wire.put(1, 2);
        wire.put(1, 2);
        for s in values {
            wire.put(s, 9);
        }
        assert!(Prefix::read(&mut wire.bits(), 258).is_err());
    }
}
#[test]
fn complex_singleton_repeat_alphabet_expands_all_256_literal_codes() {
    let mut wire = Wire::default();
    wire.put(0, 2);
    for symbol in [1, 2, 3, 4, 0, 5, 17, 6, 16, 7, 8, 9, 10, 11, 12, 13, 14, 15] {
        if symbol == 16 {
            wire.put(0b0111, 4);
        } else {
            wire.put(0, 2);
        }
    }
    for extra in [2, 2, 2, 1] {
        wire.put(extra, 2);
    }
    for value in [0, 255, 65, 128] {
        wire.code(value, 8);
    }
    let mut bits = wire.bits();
    let tree = Prefix::read(&mut bits, 256).unwrap();
    for value in [0, 255, 65, 128] {
        assert_eq!(tree.symbol(&mut bits).unwrap(), value);
    }
    bits.finish().unwrap();
}
#[test]
fn complex_zero_repeat_chains_and_exact_alphabet_admission() {
    let mut wire = Wire::default();
    wire.put(0, 2);
    for symbol in [1, 2, 3, 4, 0, 5, 17, 6, 16, 7, 8, 9, 10, 11, 12, 13, 14, 15] {
        if symbol < 14 {
            wire.put(1, 2);
        } else {
            wire.put(15, 4);
        }
    }
    // Complete code-length tree: symbols 0..13 use 4 bits, 14..17 use 5 bits.
    let symbol = |wire: &mut Wire, s: u32| {
        if s < 14 {
            wire.code(s, 4);
        } else {
            wire.code(28 + s - 14, 5);
        }
    };
    symbol(&mut wire, 1);
    symbol(&mut wire, 17);
    wire.put(0, 3);
    symbol(&mut wire, 17);
    wire.put(0, 3);
    symbol(&mut wire, 2);
    symbol(&mut wire, 3);
    symbol(&mut wire, 3);
    // Nonzero lengths occur at positions 0,12,13,14: canonical 0,10,110,111.
    for (code, width) in [(0, 1), (2, 2), (6, 3), (7, 3)] {
        wire.code(code, width);
    }
    let mut bits = wire.bits();
    let tree = Prefix::read(&mut bits, 15).unwrap();
    for value in [0, 12, 13, 14] {
        assert_eq!(tree.symbol(&mut bits).unwrap(), value);
    }
    bits.finish().unwrap();
    assert!(Prefix::read(&mut wire.bits(), 14).is_err());
    let mut bits = wire.bits();
    bits.work = 14;
    assert_eq!(
        Prefix::read(&mut bits, 15).err().unwrap().kind(),
        io::ErrorKind::OutOfMemory
    );
}
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/brotli")
            .join(name),
    )
    .unwrap()
}
#[test]
fn independently_verified_real_streams_cover_levels_windows_contexts_and_all_dictionary_transforms()
{
    for &(name, size, crc) in super::fixture_cases::CASES {
        let bytes = fixture(name);
        for quantum in [1, 8192, 65536] {
            let mut reader = super::BrotliReader::new(
                Cursor::new(&bytes),
                bytes.len() as u64,
                32 * 1024 * 1024,
                96 * 1024 * 1024 * 1024,
                24 * 1024 * 1024 * 1024,
            )
            .unwrap();
            let mut decoded = Vec::new();
            let mut buffer = vec![0; quantum];
            loop {
                let n = reader
                    .read(&mut buffer)
                    .unwrap_or_else(|e| panic!("{name}/{quantum}: {e}"));
                if n == 0 {
                    break;
                }
                decoded.extend_from_slice(&buffer[..n]);
            }
            assert_eq!(decoded.len(), size, "{name}/{quantum}");
            assert_eq!(crate::crc32(&decoded), crc, "{name}/{quantum}");
        }
    }
}
#[test]
fn exact_raw_input_work_memory_and_output_limits_are_terminal() {
    let bytes = fixture("raw.br");
    let memory = super::brotli_memory_required(16).unwrap();
    let run = |memory, work, decoded| {
        super::BrotliReader::new(
            Cursor::new(&bytes),
            bytes.len() as u64,
            memory,
            work,
            decoded,
        )
    };
    let mut reader = run(memory, 32, 1).unwrap();
    let mut output = vec![];
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, b"Z");
    assert!(matches!(run(memory-1,32,1),Err(e) if e.kind()==io::ErrorKind::OutOfMemory));
    for (work, decoded) in [(31, 1), (32, 0)] {
        let mut reader = run(memory, work, decoded).unwrap();
        assert_eq!(
            reader.read_to_end(&mut vec![]).unwrap_err().kind(),
            io::ErrorKind::OutOfMemory
        );
        assert_eq!(
            reader.read(&mut [0]).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    }
    let bytes = fixture("q11-text.br");
    for end in 0..bytes.len() {
        let mut result = super::BrotliReader::new(
            Cursor::new(&bytes[..end]),
            end as u64,
            32 * 1024 * 1024,
            1_000_000,
            1_000_000,
        );
        assert!(
            result.as_mut().is_err() || result.as_mut().unwrap().read_to_end(&mut vec![]).is_err(),
            "cut {end}"
        );
    }
    let mut bytes = bytes;
    bytes.push(0);
    let mut reader = super::BrotliReader::new(
        Cursor::new(&bytes),
        bytes.len() as u64,
        32 * 1024 * 1024,
        1_000_000,
        1_000_000,
    )
    .unwrap();
    assert!(reader.read_to_end(&mut vec![]).is_err());
}
#[test]
fn cooperative_poll_failure_is_typed_bounded_and_sticky_without_output_authority() {
    use std::{cell::Cell, rc::Rc};
    let calls = Rc::new(Cell::new(0));
    let count = Rc::clone(&calls);
    let bytes = fixture("varied.br");
    let mut reader = super::BrotliReader::with_poll(
        Cursor::new(&bytes),
        bytes.len() as u64,
        32 * 1024 * 1024,
        1_000_000,
        1_000_000,
        move || {
            count.set(count.get() + 1);
            Err(io::Error::other("owned-brotli-cancel"))
        },
    );
    if let Ok(reader) = &mut reader {
        let error = reader.read_to_end(&mut vec![]).unwrap_err();
        assert_eq!(error.to_string(), "owned-brotli-cancel");
        assert_eq!(
            reader.read(&mut [0]).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
    } else {
        assert_eq!(reader.err().unwrap().to_string(), "owned-brotli-cancel");
    }
    assert_eq!(calls.get(), 1);
}
#[test]
fn malformed_fill_metadata_and_shared_window_headers_are_rejected() {
    for offset in [2, 4] {
        let mut bytes = fixture("raw.br");
        bytes[offset] |= 128;
        let mut reader = super::BrotliReader::new(
            Cursor::new(&bytes),
            bytes.len() as u64,
            32 * 1024 * 1024,
            1_000_000,
            1_000_000,
        )
        .unwrap();
        assert!(reader.read_to_end(&mut vec![]).is_err());
    }
    let mut wire = Wire::default();
    wire.put(0, 1);
    wire.put(0, 1);
    wire.put(3, 2);
    wire.put(0, 1);
    wire.put(2, 2);
    wire.put(0, 16);
    let mut reader = super::BrotliReader::new(
        Cursor::new(&wire.bytes),
        wire.bytes.len() as u64,
        32 * 1024 * 1024,
        1_000_000,
        1_000_000,
    )
    .unwrap();
    assert!(reader.read_to_end(&mut vec![]).is_err());
    assert!(
        matches!(super::BrotliReader::new(Cursor::new([0x11]),1,32*1024*1024,1_000_000,1_000_000),Err(e) if e.kind()==io::ErrorKind::Unsupported)
    );
    assert!(
        matches!(super::BrotliReader::new(Cursor::new([]),24*1024*1024*1024+1,32*1024*1024,1_000_000,1_000_000),Err(e) if e.kind()==io::ErrorKind::OutOfMemory)
    );
}
#[test]
fn short_input_and_only_bare_interruptions_preserve_the_same_full_stream() {
    struct Short {
        source: Cursor<Vec<u8>>,
        interrupted: bool,
    }
    impl Read for Short {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            let n = bytes.len().min(1);
            self.source.read(&mut bytes[..n])
        }
    }
    let bytes = fixture("q11-text.br");
    let n = bytes.len() as u64;
    let mut reader = super::BrotliReader::new(
        Short {
            source: Cursor::new(bytes),
            interrupted: false,
        },
        n,
        32 * 1024 * 1024,
        1_000_000,
        1_000_000,
    )
    .unwrap();
    let mut output = vec![];
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, b"abracadabra abracadabra\n".repeat(6));
    #[derive(Debug)]
    struct Stop;
    impl std::fmt::Display for Stop {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("owned-terminal")
        }
    }
    impl std::error::Error for Stop {}
    struct Terminal;
    impl Read for Terminal {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Interrupted, Stop))
        }
    }
    let error = super::BrotliReader::new(Terminal, n, 32 * 1024 * 1024, 1_000_000, 1_000_000)
        .err()
        .unwrap();
    assert_ne!(error.kind(), io::ErrorKind::Interrupted);
    assert!(error.get_ref().unwrap().downcast_ref::<Stop>().is_some());
}
#[test]
fn deterministic_bit_mutations_never_panic_or_resume_a_failed_reader() {
    for name in ["q11-text.br", "dictionary-all.br", "block-switches.br"] {
        let original = fixture(name);
        for sample in 0usize..192 {
            let mut bytes = original.clone();
            let at = (sample * 137 + 17) % bytes.len();
            bytes[at] ^= 1 << (sample % 8);
            let mut reader = match super::BrotliReader::new(
                Cursor::new(&bytes),
                bytes.len() as u64,
                32 * 1024 * 1024,
                1_000_000,
                1_000_000,
            ) {
                Ok(r) => r,
                Err(_) => continue,
            };
            let mut buffer = [0; 8192];
            let mut failed = false;
            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => break,
                    Ok(_) => {}
                    Err(_) => {
                        failed = true;
                        break;
                    }
                }
            }
            if failed {
                assert_eq!(
                    reader.read(&mut buffer).unwrap_err().kind(),
                    io::ErrorKind::InvalidData
                );
            }
        }
    }
}
