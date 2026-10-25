use crate::lz_bits::{BitRead, Huffman, Input};
use std::io::{self, Read};
use xdzip_codec_api::payload::DecodeError;

/// RAR29 has no length-delimited RAR5 blocks. Only its explicit mode/end
/// controls may align this stream; PPM receives the complete pending byte.
pub(crate) struct Stream<'a> {
    input: Input<'a>,
    byte: u8,
    available: u8,
    pending: Option<u8>,
}

impl<'a> Stream<'a> {
    pub fn new(input: &'a mut dyn Read) -> Self {
        Self {
            input: Input::new(input),
            byte: 0,
            available: 0,
            pending: None,
        }
    }
    pub fn align(&mut self) {
        self.available = 0;
    }
    fn next(&mut self) -> Result<Option<u8>, DecodeError> {
        if let Some(byte) = self.pending.take() {
            Ok(Some(byte))
        } else {
            self.input.next()
        }
    }
    pub fn peek_byte(&mut self) -> Result<u8, DecodeError> {
        if self.available != 0 {
            return Err(DecodeError::Integrity);
        }
        if self.pending.is_none() {
            self.pending = self.input.next()?;
        }
        self.pending.ok_or(DecodeError::Integrity)
    }
    pub fn finish(&mut self, padding: u8) -> Result<(), DecodeError> {
        self.align();
        let padding = if self.pending.take().is_some() {
            padding.checked_sub(1).ok_or(DecodeError::Integrity)?
        } else {
            padding
        };
        self.input.end_padding(padding)
    }
    pub fn discard_remaining(
        &mut self,
        control: &xdzip_codec_api::Cancellation,
    ) -> Result<(), DecodeError> {
        self.align();
        self.pending = None;
        self.input.discard_remaining(control)?;
        Ok(())
    }
}
impl BitRead for Stream<'_> {
    fn get(&mut self, count: u8) -> Result<u64, DecodeError> {
        if count > 64 {
            return Err(DecodeError::Integrity);
        }
        let mut value = 0u64;
        for _ in 0..count {
            if self.available == 0 {
                self.byte = self.next()?.ok_or(DecodeError::Integrity)?;
                self.available = 8;
            }
            value = (value << 1) | u64::from(self.byte >> 7);
            self.byte <<= 1;
            self.available -= 1;
        }
        Ok(value)
    }
}
impl Read for Stream<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.available != 0 {
            return Err(io::Error::other(DecodeError::Integrity));
        }
        if output.is_empty() {
            return Ok(0);
        }
        match self.next().map_err(io::Error::other)? {
            Some(byte) => {
                output[0] = byte;
                Ok(1)
            }
            None => Ok(0),
        }
    }
}

pub(crate) struct Tables {
    pub main: Huffman,
    pub distance: Huffman,
    pub low: Huffman,
    pub repeat: Huffman,
}
impl Tables {
    /// The mode bit has been inspected without consumption. PPM callers take
    /// that whole byte instead; this function exclusively admits LZ mode.
    pub fn read(stream: &mut Stream<'_>, previous: &mut [u8; 404]) -> Result<Self, DecodeError> {
        stream.align();
        if stream.get(1)? != 0 {
            return Err(DecodeError::Unsupported);
        }
        if stream.get(1)? == 0 {
            previous.fill(0);
        }
        let mut meta = [0u8; 20];
        let mut position = 0usize;
        while position < meta.len() {
            let length = stream.get(4)? as u8;
            if length == 15 {
                let zeros = stream.get(4)? as usize;
                if zeros != 0 {
                    position = position
                        .checked_add(zeros + 2)
                        .filter(|n| *n <= meta.len())
                        .ok_or(DecodeError::Integrity)?;
                    continue;
                }
            }
            meta[position] = length;
            position += 1;
        }
        let meta = Huffman::new(&meta)?;
        let mut lengths = [0u8; 404];
        position = 0;
        while position < lengths.len() {
            let symbol = meta.symbol(stream)?;
            if symbol <= 15 {
                lengths[position] = (symbol as u8 + previous[position]) & 15;
                position += 1;
                continue;
            }
            let (value, count) = match symbol {
                16 | 17 => {
                    let value = *lengths
                        .get(position.checked_sub(1).ok_or(DecodeError::Integrity)?)
                        .ok_or(DecodeError::Integrity)?;
                    (
                        value,
                        if symbol == 16 {
                            3 + stream.get(3)? as usize
                        } else {
                            11 + stream.get(7)? as usize
                        },
                    )
                }
                18 => (0, 3 + stream.get(3)? as usize),
                19 => (0, 11 + stream.get(7)? as usize),
                _ => return Err(DecodeError::Integrity),
            };
            let end = position
                .checked_add(count)
                .filter(|n| *n <= lengths.len())
                .ok_or(DecodeError::Integrity)?;
            lengths[position..end].fill(value);
            position = end;
        }
        let result = Self {
            main: Huffman::new(&lengths[..299])?,
            distance: Huffman::new(&lengths[299..359])?,
            low: Huffman::new(&lengths[359..376])?,
            repeat: Huffman::new(&lengths[376..])?,
        };
        *previous = lengths;
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn ppm_peek_keeps_complete_byte_and_unaligned_read_is_rejected() {
        let mut source = Cursor::new([0xa1, 0x59]);
        let mut stream = Stream::new(&mut source);
        assert_eq!(stream.peek_byte().unwrap(), 0xa1);
        assert_eq!(stream.peek_byte().unwrap(), 0xa1);
        let mut byte = [0];
        stream.read_exact(&mut byte).unwrap();
        assert_eq!(byte, [0xa1]);
        assert_eq!(stream.get(3).unwrap(), 2);
        assert!(stream.read(&mut byte).is_err());
        stream.align();
        stream.finish(0).unwrap();
    }
    #[test]
    fn first_repeat_cannot_read_a_previous_table_value() {
        let mut bytes = [0; 11];
        bytes[8] = 4;
        let mut source = Cursor::new(bytes);
        let mut stream = Stream::new(&mut source);
        assert!(matches!(
            Tables::read(&mut stream, &mut [7; 404]),
            Err(DecodeError::Integrity)
        ));
    }
}
