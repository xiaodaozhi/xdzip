//! Sized LZ4 blocks with incremental overlap copying and exact terminal grammar.
use crate::{
    invalid,
    stream_io::{Input, READ_QUANTUM, filled, resource},
};
use std::io::{self, Read};
const WINDOW: usize = 65536;
const MEMORY: u64 = 256 * 1024;
enum State {
    Token,
    Literals {
        remaining: u64,
        original: u64,
        nibble: u8,
    },
    Match {
        remaining: u64,
        offset: usize,
    },
}
/// Streaming raw LZ4 block decoder. The caller owns framing and checksums.
pub struct Lz4BlockReader<R> {
    input: Input<R>,
    history: Vec<u8>,
    cursor: usize,
    valid: usize,
    state: State,
    decoded: u64,
    output_limit: u64,
    last_match: Option<u64>,
    ended: bool,
    failed: bool,
}
impl<R: Read> Lz4BlockReader<R> {
    /// Admit fixed decoder/history-export storage before allocating. History is
    /// chronological and belongs only to this container's preceding linked block.
    pub fn new(
        source: R,
        packed: u64,
        output_limit: u64,
        history: &[u8],
        memory_limit: u64,
    ) -> io::Result<Self> {
        if history.len() > WINDOW {
            return Err(invalid());
        }
        if memory_limit < MEMORY {
            return Err(resource());
        }
        let mut dictionary = filled(WINDOW, 0)?;
        dictionary[..history.len()].copy_from_slice(history);
        Ok(Self {
            input: Input::new(source, packed),
            history: dictionary,
            cursor: history.len() % WINDOW,
            valid: history.len(),
            state: State::Token,
            decoded: 0,
            output_limit,
            last_match: None,
            ended: false,
            failed: false,
        })
    }
    /// Complete decoded bytes delivered, excluding seed history.
    pub fn decoded_bytes(&self) -> u64 {
        self.decoded
    }
    /// Transfer chronological history only after exact successful block EOF.
    pub fn into_history(self) -> io::Result<Vec<u8>> {
        if !self.ended || self.failed {
            return Err(invalid());
        }
        let mut history = Vec::new();
        history
            .try_reserve_exact(self.valid)
            .map_err(|_| resource())?;
        let start = (self.cursor + WINDOW - self.valid) % WINDOW;
        let first = self.valid.min(WINDOW - start);
        history.extend_from_slice(&self.history[start..start + first]);
        history.extend_from_slice(&self.history[..self.valid - first]);
        Ok(history)
    }
    fn byte(&mut self) -> io::Result<u8> {
        self.input.byte(self.input.packed)
    }
    fn length(&mut self, nibble: u8) -> io::Result<u64> {
        let mut length = u64::from(nibble);
        if nibble == 15 {
            loop {
                let byte = self.byte()?;
                length = length
                    .checked_add(u64::from(byte))
                    .filter(|n| *n <= self.output_limit - self.decoded)
                    .ok_or_else(invalid)?;
                if byte != 255 {
                    break;
                }
            }
        }
        Ok(length)
    }
    fn push(&mut self, byte: u8) -> io::Result<()> {
        self.decoded = self
            .decoded
            .checked_add(1)
            .filter(|n| *n <= self.output_limit)
            .ok_or_else(invalid)?;
        self.history[self.cursor] = byte;
        self.cursor = (self.cursor + 1) % WINDOW;
        self.valid = (self.valid + 1).min(WINDOW);
        Ok(())
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let mut count = 0;
        while count < output.len().min(READ_QUANTUM) {
            match self.state {
                State::Token => {
                    let token = self.byte()?;
                    let length = self.length(token >> 4)?;
                    if length > self.output_limit - self.decoded
                        || length > self.input.packed - self.input.consumed
                    {
                        return Err(invalid());
                    }
                    self.state = State::Literals {
                        remaining: length,
                        original: length,
                        nibble: token & 15,
                    };
                }
                State::Literals {
                    remaining: 0,
                    original,
                    nibble,
                } => {
                    if self.input.consumed == self.input.packed {
                        if self
                            .last_match
                            .is_some_and(|at| original < 5 || self.decoded - at < 12)
                        {
                            return Err(invalid());
                        }
                        self.ended = true;
                        break;
                    }
                    let low = self.byte()?;
                    let high = self.byte()?;
                    let offset = usize::from(u16::from_le_bytes([low, high]));
                    let length = self.length(nibble)?.checked_add(4).ok_or_else(invalid)?;
                    if offset == 0
                        || offset > self.valid
                        || length > self.output_limit - self.decoded
                    {
                        return Err(invalid());
                    }
                    self.last_match = Some(self.decoded);
                    self.state = State::Match {
                        remaining: length,
                        offset,
                    };
                }
                State::Literals {
                    remaining,
                    original,
                    nibble,
                } => {
                    let byte = self.byte()?;
                    self.push(byte)?;
                    output[count] = byte;
                    count += 1;
                    self.state = State::Literals {
                        remaining: remaining - 1,
                        original,
                        nibble,
                    };
                }
                State::Match { remaining: 0, .. } => self.state = State::Token,
                State::Match { remaining, offset } => {
                    let byte = self.history[(self.cursor + WINDOW - offset) % WINDOW];
                    self.push(byte)?;
                    output[count] = byte;
                    count += 1;
                    self.state = State::Match {
                        remaining: remaining - 1,
                        offset,
                    };
                }
            }
        }
        Ok(count)
    }
}
impl<R: Read> Read for Lz4BlockReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(0);
        }
        self.read_inner(bytes).inspect_err(|_| self.failed = true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn decode(
        encoded: &[u8],
        history: &[u8],
        limit: u64,
        chunk: usize,
    ) -> io::Result<(Vec<u8>, Vec<u8>)> {
        let mut reader = Lz4BlockReader::new(
            Cursor::new(encoded),
            encoded.len() as u64,
            limit,
            history,
            MEMORY,
        )?;
        let mut output = Vec::new();
        let mut buffer = vec![0; chunk];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            output.extend_from_slice(&buffer[..n]);
        }
        Ok((output, reader.into_history()?))
    }
    #[test]
    fn normative_literals_overlap_dictionary_and_unused_terminal_bits() {
        for chunk in [1, 3, 8192] {
            assert!(decode(&[0], &[], 0, chunk).unwrap().0.is_empty());
            assert!(decode(&[15], &[], 0, chunk).unwrap().0.is_empty());
            assert_eq!(decode(&[0x1f, b'a'], &[], 1, chunk).unwrap().0, b"a");
            assert_eq!(decode(b"\x5fhello", &[], 5, chunk).unwrap().0, b"hello");
            assert_eq!(
                decode(b"\x13a\x01\0\x50bbbbb", &[], 13, chunk).unwrap().0,
                b"aaaaaaaabbbbb"
            );
            let dictionary: Vec<u8> = (0..65536).map(|i| ((i * 73) ^ (i >> 8)) as u8).collect();
            let expected = [&dictionary[1..8], b"bbbbb"].concat();
            let (actual, history) =
                decode(b"\x03\xff\xff\x50bbbbb", &dictionary, 12, chunk).unwrap();
            assert_eq!(actual, expected);
            assert_eq!(&history[history.len() - 12..], expected);
        }
    }
    #[test]
    fn malformed_lengths_offsets_and_terminal_conditions_poison() {
        for input in [
            b"".as_slice(),
            b"\xf0",
            b"\xf0\xff",
            b"\x13a\0\0\x50bbbbb",
            b"\x03\x01\0\x50bbbbb",
            b"\x13a\x02\0\x50bbbbb",
            b"\x12a\x01\0\x50bbbbb",
            b"\x13a\x01\0\x40bbbb",
            b"\x13a\x01\0",
            b"\x50four",
            b"\x0f\x01\0\xff\xff\xff",
        ] {
            let mut reader =
                Lz4BlockReader::new(Cursor::new(input), input.len() as u64, 32, &[], MEMORY)
                    .unwrap();
            assert!(reader.read_to_end(&mut Vec::new()).is_err(), "{input:?}");
            assert!(reader.read(&mut [0]).is_err());
            assert!(reader.into_history().is_err());
        }
        assert!(Lz4BlockReader::new(Cursor::new([0]), 1, 0, &[], MEMORY - 1).is_err());
        let unfinished = Lz4BlockReader::new(Cursor::new([0]), 1, 0, &[], MEMORY).unwrap();
        assert!(unfinished.into_history().is_err());
    }
    #[test]
    fn large_literal_lengths_roll_history_and_stay_inside_the_physical_span() {
        let plain: Vec<u8> = (0..131072).map(|i| ((i * 73) ^ (i >> 8)) as u8).collect();
        let mut encoded = vec![0xf0];
        let mut length = plain.len() - 15;
        while length >= 255 {
            encoded.push(255);
            length -= 255;
        }
        encoded.push(length as u8);
        encoded.extend_from_slice(&plain);
        let packed = encoded.len() as u64;
        encoded.extend_from_slice(b"outside block");
        let mut source = Cursor::new(&encoded);
        let mut reader =
            Lz4BlockReader::new(&mut source, packed, plain.len() as u64, &[], MEMORY).unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, plain);
        assert_eq!(reader.decoded_bytes(), plain.len() as u64);
        assert_eq!(
            reader.into_history().unwrap(),
            plain[plain.len() - WINDOW..]
        );
        assert_eq!(source.position(), packed);
    }
}
