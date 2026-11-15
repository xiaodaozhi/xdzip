use crate::{
    adaptive_huffman::Adaptive,
    bitstream::Bits,
    entropy::Canonical,
    invalid,
    stream_io::{filled, resource},
};
use std::io::{self, Read};
/// Conservative history, adaptive tree, ranks, canonical positions and I/O admission.
pub fn lh_dynamic_memory_required() -> u64 {
    192 * 1024
}
enum Positions {
    Fixed(Canonical),
    Growing { model: Adaptive, next: u64 },
}
/// Known LH2 position-counter variants sharing the same container method marker.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Lh2Variant {
    /// Original LHa for Unix leaves an initial singleton's weight unchanged.
    LhaUnix,
    /// jLHA updates that singleton through its generic adaptive tree.
    Jlha,
}
/// Exact-size LH1/LH2 dynamic Huffman/LZ stream, without container framing or CRC.
pub struct LhDynamicReader<R, P = fn() -> io::Result<()>> {
    bits: Bits<R, P, true>,
    model: Adaptive,
    positions: Positions,
    method: u8,
    produced: u64,
    dictionary: Vec<u8>,
    position: usize,
    remaining: u64,
    copy: usize,
    distance: usize,
    ended: bool,
    failed: bool,
}
impl<R: Read> LhDynamicReader<R> {
    /// Admit the exact packed range, decoded size, memory and work before allocation.
    pub fn new(
        source: R,
        method: u8,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
    ) -> io::Result<Self> {
        Self::with_poll(source, method, packed, decoded, memory, work, || Ok(()))
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> LhDynamicReader<R, P> {
    /// Select an explicit LH2 variant; callers must validate the complete result and CRC.
    pub fn with_lh2_variant(
        source: R,
        variant: Lh2Variant,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
        poll: P,
    ) -> io::Result<Self> {
        let mut result = Self::with_poll(source, 2, packed, decoded, memory, work, poll)?;
        if variant == Lh2Variant::LhaUnix
            && let Positions::Growing { model, .. } = &mut result.positions
        {
            model.use_unix_position_counter()?;
        }
        Ok(result)
    }
    /// Cooperative control includes adaptive swaps, rebuilding and overlapping history.
    pub fn with_poll(
        source: R,
        method: u8,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
        poll: P,
    ) -> io::Result<Self> {
        if !matches!(method, 1 | 2) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "unsupported_dynamic_lha_method",
            ));
        }
        if memory < lh_dynamic_memory_required()
            || packed > 24 * 1024 * 1024 * 1024
            || decoded > 24 * 1024 * 1024 * 1024
        {
            return Err(resource());
        }
        if decoded == 0 && packed != 0 {
            return Err(invalid());
        }
        let mut bits = Bits::with_poll(source, packed, work.min(96 * 1024 * 1024 * 1024), poll);
        let dictionary_size = if method == 1 { 4096 } else { 8192 };
        bits.charge(dictionary_size + 64)?;
        let model = Adaptive::new(if method == 1 { 314 } else { 286 }, &mut bits)?;
        let lengths: Vec<u8> = (0..64)
            .map(|at| match at {
                0 => 3,
                1..=3 => 4,
                4..=11 => 5,
                12..=23 => 6,
                24..=47 => 7,
                _ => 8,
            })
            .collect();
        let positions = if method == 1 {
            Positions::Fixed(Canonical::from_lengths(&lengths, 16, false)?)
        } else {
            Positions::Growing {
                model: Adaptive::growing(128, &mut bits)?,
                next: 64,
            }
        };
        Ok(Self {
            bits,
            model,
            positions,
            method,
            produced: 0,
            dictionary: filled(dictionary_size as usize, b' ')?,
            position: 0,
            remaining: decoded,
            copy: 0,
            distance: 0,
            ended: false,
            failed: false,
        })
    }
    /// Remaining actual-work allowance, retained across successful container members.
    pub fn work_remaining(&self) -> u64 {
        self.bits.work
    }
    fn decode(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let maximum = output.len().min(8192);
        let mut at = 0;
        while at < maximum && !self.ended {
            if self.remaining == 0 {
                if self.copy != 0 {
                    return Err(invalid());
                }
                self.bits.finish_ignored_padding()?;
                self.ended = true;
                break;
            }
            let byte = if self.copy != 0 {
                let from = (self.position + self.dictionary.len() - self.distance)
                    & (self.dictionary.len() - 1);
                self.copy -= 1;
                self.dictionary[from]
            } else {
                let mut symbol = self.model.decode(&mut self.bits)?;
                if self.method == 2 && symbol == 285 {
                    symbol += self.bits.get(8)? as u16;
                    if symbol > 509 {
                        return Err(invalid());
                    }
                }
                if symbol < 256 {
                    symbol as u8
                } else {
                    let count = usize::from(symbol) - 253;
                    if count as u64 > self.remaining {
                        return Err(invalid());
                    }
                    let upper = match &mut self.positions {
                        Positions::Fixed(tree) => tree.symbol(|| Ok(self.bits.get(1)? as u8))?,
                        Positions::Growing { model, next } => {
                            while self.produced > *next && *next < 8192 {
                                model.grow((*next / 64) as usize, &mut self.bits)?;
                                *next += 64;
                            }
                            model.decode(&mut self.bits)?
                        }
                    };
                    self.distance = (usize::from(upper) << 6) + self.bits.get(6)? as usize + 1;
                    self.copy = count;
                    continue;
                }
            };
            self.bits.charge(1)?;
            self.dictionary[self.position] = byte;
            self.position = (self.position + 1) & (self.dictionary.len() - 1);
            self.produced += 1;
            self.remaining -= 1;
            output[at] = byte;
            at += 1;
        }
        Ok(at)
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for LhDynamicReader<R, P> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        match self.decode(output) {
            Ok(n) => Ok(n),
            Err(error) => {
                self.failed = true;
                Err(error)
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn exact_work_memory_spans_truncation_and_mutations_keep_dynamic_state_terminal() {
        for method in [1, 2] {
            let name = format!("lh{method}-tail1.lzh");
            let bytes = std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/lha")
                    .join(name),
            )
            .unwrap();
            let raw = &bytes[43..bytes.len() - 1];
            let memory = lh_dynamic_memory_required();
            let mut reader = LhDynamicReader::new(
                Cursor::new(raw),
                method,
                raw.len() as u64,
                1,
                memory,
                1_000_000,
            )
            .unwrap();
            let mut output = vec![];
            reader.read_to_end(&mut output).unwrap();
            assert_eq!(output, b"A");
            let cost = 1_000_000 - reader.work_remaining();
            let mut reader =
                LhDynamicReader::new(Cursor::new(raw), method, raw.len() as u64, 1, memory, cost)
                    .unwrap();
            reader.read_to_end(&mut vec![]).unwrap();
            assert_eq!(reader.work_remaining(), 0);
            assert!(
                LhDynamicReader::new(
                    Cursor::new(raw),
                    method,
                    raw.len() as u64,
                    1,
                    memory - 1,
                    cost
                )
                .is_err()
            );
            let mut reader = LhDynamicReader::new(
                Cursor::new(raw),
                method,
                raw.len() as u64,
                1,
                memory,
                cost - 1,
            )
            .unwrap();
            assert_eq!(
                reader.read_to_end(&mut vec![]).unwrap_err().kind(),
                io::ErrorKind::OutOfMemory
            );
            assert!(reader.read(&mut [0]).is_err());
            for length in 0..raw.len() {
                let mut reader = LhDynamicReader::new(
                    Cursor::new(&raw[..length]),
                    method,
                    raw.len() as u64,
                    1,
                    memory,
                    1_000_000,
                )
                .unwrap();
                assert!(reader.read_to_end(&mut vec![]).is_err());
                assert!(reader.read(&mut [0]).is_err());
            }
            let mut padded = raw.to_vec();
            padded.push(0);
            let mut reader = LhDynamicReader::new(
                Cursor::new(&padded),
                method,
                padded.len() as u64,
                1,
                memory,
                1_000_000,
            )
            .unwrap();
            assert!(reader.read_to_end(&mut vec![]).is_err());
            let name = format!("lh{method}-h0.lzh");
            let bytes = std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/lha")
                    .join(name),
            )
            .unwrap();
            let raw = &bytes[33..bytes.len() - 1];
            for bit in 0..raw.len() * 8 {
                let mut bytes = raw.to_vec();
                bytes[bit / 8] ^= 1 << (bit % 8);
                let mut reader = LhDynamicReader::new(
                    Cursor::new(bytes),
                    method,
                    raw.len() as u64,
                    144,
                    memory,
                    1_000_000,
                )
                .unwrap();
                let mut output = vec![];
                if reader.read_to_end(&mut output).is_err() {
                    assert!(reader.read(&mut [0]).is_err());
                } else {
                    assert_eq!(output.len(), 144);
                    assert_eq!(reader.read(&mut [0]).unwrap(), 0);
                }
                assert!(output.len() <= 144);
            }
        }
    }
    #[test]
    fn adaptive_work_control_and_typed_source_interruption_are_terminal() {
        struct Source;
        impl Read for Source {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::new(io::ErrorKind::Interrupted, "typed-source"))
            }
        }
        for method in [1, 2] {
            let mut reader = LhDynamicReader::new(
                Source,
                method,
                10,
                1,
                lh_dynamic_memory_required(),
                1_000_000,
            )
            .unwrap();
            let e = reader.read(&mut [0]).unwrap_err();
            assert_ne!(e.kind(), io::ErrorKind::Interrupted);
            assert_eq!(e.to_string(), "typed-source");
            assert!(reader.read(&mut [0]).is_err());
            let name = format!("lh{method}-random.lzh");
            let bytes = std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/lha")
                    .join(name),
            )
            .unwrap();
            let raw = &bytes[43..bytes.len() - 1];
            let mut polls = 0;
            let mut reader = LhDynamicReader::with_poll(
                Cursor::new(raw),
                method,
                raw.len() as u64,
                65000,
                lh_dynamic_memory_required(),
                96 * 1024 * 1024 * 1024,
                || {
                    polls += 1;
                    if polls == 5 {
                        Err(io::Error::other("adaptive-cancel"))
                    } else {
                        Ok(())
                    }
                },
            )
            .unwrap();
            let mut output = vec![];
            assert_eq!(
                reader.read_to_end(&mut output).unwrap_err().to_string(),
                "adaptive-cancel"
            );
            assert!(output.len() < 65000);
            assert!(reader.read(&mut [0]).is_err());
        }
    }
    #[test]
    fn original_unix_position_counter_matches_the_author_reference_output() {
        let bytes = include_bytes!("../../../tests/fixtures/lha/lh2-unix-spaces.lzh");
        let mut reader = LhDynamicReader::with_lh2_variant(
            std::io::Cursor::new(&bytes[43..89]),
            Lh2Variant::LhaUnix,
            46,
            4096,
            lh_dynamic_memory_required(),
            96 * 1024 * 1024 * 1024,
            || Ok(()),
        )
        .unwrap();
        let mut output = vec![];
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, vec![32; 4096]);
        assert_eq!(crate::crc16(&output), 34503);
    }
    #[test]
    fn independently_encoded_dynamic_lha_covers_initial_tree_adaptation_rescaling_and_history() {
        let mut cases = 0;
        for &(name, method, header, packed, decoded, crc) in crate::lzh::fixture_cases::CASES {
            if !matches!(method, "lh1" | "lh2") || name == "lh2-unix-spaces.lzh" {
                continue;
            }
            let bytes = std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/lha")
                    .join(name),
            )
            .unwrap();
            let mut reader = LhDynamicReader::new(
                Cursor::new(&bytes[header..header + packed]),
                if method == "lh1" { 1 } else { 2 },
                packed as u64,
                decoded as u64,
                lh_dynamic_memory_required(),
                96 * 1024 * 1024 * 1024,
            )
            .unwrap();
            let mut output = vec![];
            reader
                .read_to_end(&mut output)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(output, crate::lzh::tests::expected(name), "{name}");
            assert_eq!(crate::crc16(&output), crc);
            cases += 1;
        }
        assert_eq!(cases, 16);
    }
}
