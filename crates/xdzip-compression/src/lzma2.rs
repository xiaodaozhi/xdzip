use crate::{
    invalid,
    lzma::{self, Dictionary, Input, Models, Properties, READ_QUANTUM, Range, Token},
};
use std::io::{self, Read};

pub(crate) fn dictionary_size(property: u8) -> io::Result<usize> {
    match property {
        0..=39 => Ok(((2u64 + u64::from(property & 1)) << (property / 2 + 11)) as usize),
        40 => Ok(u32::MAX as usize),
        _ => Err(invalid()),
    }
}

/// Conservative reader allocation budget, including the largest permitted
/// LZMA2 literal model arena. This is an implementation admission policy.
pub fn lzma2_memory_required(property: u8) -> io::Result<u64> {
    Ok(lzma::memory_required(dictionary_size(property)?, 16))
}

enum Chunk {
    Header,
    Raw(u64),
    Compressed { remaining: u64, range: Range },
}

/// Streaming raw LZMA2 reader with exact packed/output lengths. Chunk range
/// state is independent; model, dictionary and repetition resets follow the
/// control byte. A valid EOF requires the overall 00 marker and no trailing data.
pub struct Lzma2Reader<R> {
    input: Input<R>,
    dictionary: Dictionary,
    models: Models,
    chunk: Chunk,
    expected: u64,
    decoded: u64,
    pending: (usize, usize),
    dictionary_ready: bool,
    properties_ready: bool,
    ended: bool,
    failed: bool,
}
impl<R: Read> Lzma2Reader<R> {
    /// Allocate only after the complete dictionary/model budget is admitted.
    pub fn new(
        source: R,
        packed: u64,
        expected: u64,
        property: u8,
        memory_limit: u64,
    ) -> io::Result<Self> {
        let size = dictionary_size(property)?;
        if lzma::memory_required(size, 16) > memory_limit {
            return Err(lzma::resource());
        }
        // Allocate all 16 contexts once; later property resets cannot increase
        // the arena's capacity beyond the amount admitted here.
        let models = Models::new(Properties {
            lc: 4,
            lp: 0,
            pb: 2,
        })?;
        Ok(Self {
            input: Input::new(source, packed),
            dictionary: Dictionary::new(size)?,
            models,
            chunk: Chunk::Header,
            expected,
            decoded: 0,
            pending: (0, 0),
            dictionary_ready: false,
            properties_ready: false,
            ended: false,
            failed: false,
        })
    }
    /// Logical compressed bytes consumed, excluding buffered read-ahead.
    pub fn consumed_bytes(&self) -> u64 {
        self.input.consumed
    }
    /// Output bytes delivered to the consumer.
    pub fn decoded_bytes(&self) -> u64 {
        self.decoded
    }
    fn header(&mut self) -> io::Result<()> {
        let control = self.input.byte(self.input.packed)?;
        if control == 0 {
            if self.decoded != self.expected || self.input.consumed != self.input.packed {
                return Err(invalid());
            }
            self.ended = true;
            return Ok(());
        }
        if control <= 2 {
            if control == 1 {
                self.dictionary.reset();
                self.dictionary_ready = true;
                self.properties_ready = false;
            } else if !self.dictionary_ready {
                return Err(invalid());
            }
            let size = u64::from(self.input.word()?) + 1;
            if size > self.expected - self.decoded {
                return Err(invalid());
            }
            self.chunk = Chunk::Raw(size);
            return Ok(());
        }
        if control < 0x80 {
            return Err(invalid());
        }
        let unpacked = (u64::from(control & 31) << 16) + u64::from(self.input.word()?) + 1;
        let packed = u64::from(self.input.word()?) + 1;
        if unpacked > self.expected - self.decoded {
            return Err(invalid());
        }
        if control >= 0xe0 {
            self.dictionary.reset();
            self.dictionary_ready = true;
        } else if !self.dictionary_ready {
            return Err(invalid());
        }
        if control >= 0xc0 {
            let properties = Properties::parse(self.input.byte(self.input.packed)?)?;
            if properties.lc + properties.lp > 4 {
                return Err(invalid());
            }
            self.models.reset(properties)?;
            self.properties_ready = true;
        } else {
            if !self.properties_ready {
                return Err(invalid());
            }
            if control >= 0xa0 {
                self.models.reset(self.models.properties)?;
            }
        }
        let boundary = self
            .input
            .consumed
            .checked_add(packed)
            .ok_or_else(invalid)?;
        if boundary > self.input.packed {
            return Err(invalid());
        }
        let range = Range::new(&mut self.input, boundary)?;
        self.chunk = Chunk::Compressed {
            remaining: unpacked,
            range,
        };
        Ok(())
    }
    fn complete_chunk(&mut self) -> io::Result<()> {
        match &self.chunk {
            Chunk::Raw(0) => self.chunk = Chunk::Header,
            Chunk::Compressed {
                remaining: 0,
                range,
            } => {
                if self.pending.1 != 0 || !range.complete(&self.input) {
                    return Err(invalid());
                }
                self.chunk = Chunk::Header;
            }
            _ => return Err(invalid()),
        }
        Ok(())
    }
    /// Validate terminal headers after the full output has been read. This
    /// never silently decodes or discards remaining output, and errors poison.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(());
        }
        let result = (|| {
            if self.decoded != self.expected {
                return Err(invalid());
            }
            if !matches!(self.chunk, Chunk::Header) {
                self.complete_chunk()?;
            }
            self.header()?;
            if !self.ended {
                return Err(invalid());
            }
            Ok(())
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let mut count = 0;
        while count < output.len().min(READ_QUANTUM) {
            if matches!(
                self.chunk,
                Chunk::Raw(0) | Chunk::Compressed { remaining: 0, .. }
            ) {
                self.complete_chunk()?;
            }
            if matches!(self.chunk, Chunk::Header) {
                self.header()?;
                if self.ended {
                    break;
                }
            }
            let byte = match &mut self.chunk {
                Chunk::Header => return Err(invalid()),
                Chunk::Raw(remaining) => {
                    *remaining -= 1;
                    self.input.byte(self.input.packed)?
                }
                Chunk::Compressed { remaining, range } => {
                    let byte = if self.pending.1 != 0 {
                        self.pending.1 -= 1;
                        self.dictionary.get(self.pending.0)?
                    } else {
                        match lzma::token(
                            &mut self.models,
                            &self.dictionary,
                            range,
                            &mut self.input,
                        )? {
                            Token::Literal(byte) => byte,
                            Token::Match { distance, length } => {
                                if length as u64 > *remaining {
                                    return Err(invalid());
                                }
                                self.pending = (distance, length - 1);
                                self.dictionary.get(distance)?
                            }
                            Token::End => return Err(invalid()),
                        }
                    };
                    *remaining -= 1;
                    byte
                }
            };
            self.dictionary.push(byte)?;
            output[count] = byte;
            self.decoded += 1;
            count += 1;
        }
        Ok(count)
    }
}
impl<R: Read> Read for Lzma2Reader<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(0);
        }
        let result = self.read_inner(output);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
