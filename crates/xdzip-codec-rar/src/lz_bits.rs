use crate::algorithm::input_error;
use std::io::{self, Read};
use xdzip_codec_api::payload::DecodeError;

pub(crate) struct Input<'a> {
    source: &'a mut dyn Read,
    bytes: [u8; 8192],
    at: usize,
    end: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn oversubscription_empty_tree_and_missing_code_are_errors() {
        assert!(Huffman::new(&[1, 1, 1]).is_err());
        for lengths in [&[0, 0][..], &[2][..]] {
            let tree = Huffman::new(lengths).unwrap();
            let mut source = Cursor::new([0x40]);
            let mut input = Input::new(&mut source);
            let mut bits = Bits::new(&mut input, 1, 2).unwrap();
            assert!(tree.symbol(&mut bits).is_err());
        }
    }
    #[test]
    fn bounded_bits_do_not_borrow_low_padding_or_next_bytes() {
        let mut source = Cursor::new([0xff, 0xff]);
        let mut input = Input::new(&mut source);
        let mut bits = Bits::new(&mut input, 1, 3).unwrap();
        assert_eq!(bits.get(3).unwrap(), 7);
        assert_eq!(bits.get(1), Err(DecodeError::Integrity));
    }
    #[test]
    fn table_runs_cannot_cross_alphabet_ends() {
        for bytes in [vec![0xff, 0xff], {
            let mut bytes = vec![0; 10];
            bytes[9] = 1;
            bytes.extend([0x7f; 4]);
            bytes
        }] {
            let mut cursor = Cursor::new(bytes.clone());
            let mut input = Input::new(&mut cursor);
            let mut bits = Bits::new(&mut input, bytes.len() as u64, 8).unwrap();
            assert!(Tables::read(&mut bits, 0).is_err());
        }
    }

    #[test]
    fn typed_interrupted_input_is_not_retried_or_reclassified() {
        use xdzip_codec_api::ScanError;
        struct Fault {
            after: bool,
            error: ScanError,
            kind: io::ErrorKind,
        }
        impl Read for Fault {
            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                if self.after {
                    self.after = false;
                    b[0] = 0x41;
                    Ok(1)
                } else {
                    Err(io::Error::new(self.kind, self.error))
                }
            }
        }
        for error in [
            ScanError::Cancelled,
            ScanError::Limit,
            ScanError::SourceChanged,
            ScanError::Io,
        ] {
            for kind in [io::ErrorKind::Other, io::ErrorKind::Interrupted] {
                for after in [false, true] {
                    let mut source = Fault { after, error, kind };
                    let mut input = Input::new(&mut source);
                    if after {
                        assert_eq!(input.byte().unwrap(), 0x41);
                    }
                    assert_eq!(input.byte(), Err(crate::decode_error(error)));
                }
            }
        }
    }
}
impl<'a> Input<'a> {
    pub fn new(source: &'a mut dyn Read) -> Self {
        Self {
            source,
            bytes: [0; 8192],
            at: 0,
            end: 0,
        }
    }
    pub(crate) fn next(&mut self) -> Result<Option<u8>, DecodeError> {
        if self.at == self.end {
            self.end = loop {
                match self.source.read(&mut self.bytes) {
                    Err(error)
                        if error.kind() == io::ErrorKind::Interrupted
                            && error.get_ref().is_none() =>
                    {
                        continue;
                    }
                    result => break result.map_err(input_error)?,
                }
            };
            self.at = 0;
            if self.end == 0 {
                return Ok(None);
            }
        }
        let result = self.bytes[self.at];
        self.at += 1;
        Ok(Some(result))
    }
    pub fn byte(&mut self) -> Result<u8, DecodeError> {
        self.next()?.ok_or(DecodeError::Integrity)
    }
    pub fn end_padding(&mut self, maximum: u8) -> Result<(), DecodeError> {
        // Read-ahead belongs to this buffer. An outer CBC reader alone cannot
        // account for alignment bytes already fetched past the last LZ block.
        for _ in 0..=maximum {
            if self.next()?.is_none() {
                return Ok(());
            }
        }
        Err(DecodeError::Integrity)
    }
    pub fn discard_remaining(
        &mut self,
        control: &xdzip_codec_api::Cancellation,
    ) -> Result<u64, DecodeError> {
        let mut count = (self.end - self.at) as u64;
        self.at = self.end;
        loop {
            control.check().map_err(crate::decode_error)?;
            let n = match self.source.read(&mut self.bytes) {
                Err(error)
                    if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none() =>
                {
                    continue;
                }
                result => result.map_err(input_error)?,
            };
            if n == 0 {
                return Ok(count);
            }
            count = count
                .checked_add(n as u64)
                .filter(|n| *n <= crate::INPUT_LIMIT)
                .ok_or(DecodeError::Limit)?;
        }
    }
}
pub(crate) struct Bits<'a, 'b> {
    input: &'a mut Input<'b>,
    remaining: u64,
    byte: u8,
    available: u8,
}
pub(crate) trait BitRead {
    fn get(&mut self, count: u8) -> Result<u64, DecodeError>;
}
impl BitRead for Bits<'_, '_> {
    fn get(&mut self, count: u8) -> Result<u64, DecodeError> {
        Bits::get(self, count)
    }
}
impl<'a, 'b> Bits<'a, 'b> {
    pub fn new(input: &'a mut Input<'b>, bytes: u64, last: u8) -> Result<Self, DecodeError> {
        let remaining = if bytes == 0 {
            0
        } else {
            (bytes - 1)
                .checked_mul(8)
                .and_then(|n| n.checked_add(u64::from(last)))
                .ok_or(DecodeError::Integrity)?
        };
        Ok(Self {
            input,
            remaining,
            byte: 0,
            available: 0,
        })
    }
    pub fn remaining(&self) -> u64 {
        self.remaining
    }
    pub fn get(&mut self, count: u8) -> Result<u64, DecodeError> {
        if count > 64 || u64::from(count) > self.remaining {
            return Err(DecodeError::Integrity);
        }
        let mut out = 0u64;
        for _ in 0..count {
            if self.available == 0 {
                self.byte = self.input.byte()?;
                self.available = 8;
            }
            out = (out << 1) | u64::from(self.byte >> 7);
            self.byte <<= 1;
            self.available -= 1;
            self.remaining -= 1;
        }
        Ok(out)
    }
}

#[derive(Clone)]
pub(crate) struct Huffman {
    counts: [u16; 16],
    first: [u32; 16],
    offsets: [u16; 16],
    symbols: Vec<u16>,
}
impl Huffman {
    pub fn new(lengths: &[u8]) -> Result<Self, DecodeError> {
        let mut counts = [0u16; 16];
        for length in lengths {
            if *length > 15 {
                return Err(DecodeError::Integrity);
            }
            if *length != 0 {
                counts[usize::from(*length)] += 1;
            }
        }
        let mut first = [0u32; 16];
        let mut offsets = [0u16; 16];
        let mut offset = 0u16;
        for length in 1..16 {
            if length > 1 {
                first[length] = 2 * (first[length - 1] + u32::from(counts[length - 1]));
            }
            if first[length] + u32::from(counts[length]) > 1u32 << length {
                return Err(DecodeError::Integrity);
            }
            offsets[length] = offset;
            offset += counts[length];
        }
        let mut symbols = vec![0; usize::from(offset)];
        let mut next = offsets;
        for (symbol, length) in lengths.iter().enumerate() {
            if *length != 0 {
                let length = usize::from(*length);
                symbols[usize::from(next[length])] =
                    u16::try_from(symbol).map_err(|_| DecodeError::Integrity)?;
                next[length] += 1;
            }
        }
        Ok(Self {
            counts,
            first,
            offsets,
            symbols,
        })
    }
    pub fn symbol<B: BitRead + ?Sized>(&self, bits: &mut B) -> Result<u16, DecodeError> {
        let mut code = 0u32;
        for length in 1..16 {
            code = (code << 1) | bits.get(1)? as u32;
            if code >= self.first[length]
                && code - self.first[length] < u32::from(self.counts[length])
            {
                return self
                    .symbols
                    .get(usize::from(self.offsets[length]) + (code - self.first[length]) as usize)
                    .copied()
                    .ok_or(DecodeError::Integrity);
            }
        }
        Err(DecodeError::Integrity)
    }
}

#[derive(Clone)]
pub(crate) struct Tables {
    pub main: Huffman,
    pub distance: Huffman,
    pub low: Huffman,
    pub repeat: Huffman,
}
impl Tables {
    pub fn read(bits: &mut Bits<'_, '_>, algorithm: u8) -> Result<Self, DecodeError> {
        let mut meta = [0u8; 20];
        let mut at = 0;
        while at < meta.len() {
            let n = bits.get(4)? as u8;
            if n == 15 {
                let run = bits.get(4)? as usize;
                if run != 0 {
                    let end = at + run + 2;
                    if end > meta.len() {
                        return Err(DecodeError::Integrity);
                    }
                    at = end;
                    continue;
                }
            }
            meta[at] = n;
            at += 1;
        }
        let huffman = Huffman::new(&meta)?;
        let distance = if algorithm == 0 { 64 } else { 80 };
        let total = 306 + distance + 16 + 44;
        let mut lengths = Vec::<u8>::with_capacity(total);
        while lengths.len() < total {
            let symbol = huffman.symbol(bits)?;
            if symbol <= 15 {
                lengths.push(symbol as u8);
                continue;
            }
            let (value, n) = match symbol {
                16 => (
                    *lengths.last().ok_or(DecodeError::Integrity)?,
                    3 + bits.get(3)? as usize,
                ),
                17 => (
                    *lengths.last().ok_or(DecodeError::Integrity)?,
                    11 + bits.get(7)? as usize,
                ),
                18 => (0, 3 + bits.get(3)? as usize),
                19 => (0, 11 + bits.get(7)? as usize),
                _ => return Err(DecodeError::Integrity),
            };
            let end = lengths
                .len()
                .checked_add(n)
                .filter(|n| *n <= total)
                .ok_or(DecodeError::Integrity)?;
            lengths.resize(end, value);
        }
        Ok(Self {
            main: Huffman::new(&lengths[..306])?,
            distance: Huffman::new(&lengths[306..306 + distance])?,
            low: Huffman::new(&lengths[306 + distance..322 + distance])?,
            repeat: Huffman::new(&lengths[322 + distance..])?,
        })
    }
}
