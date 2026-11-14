use super::{FseSequenceKind, FseTable, HuffmanTable, ReverseBits};
use crate::invalid;
use std::io;
const BLOCK: usize = 128 * 1024;
const OVERHEAD: u64 = 4 * 1024 * 1024;
struct Input<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> Input<'a> {
    fn take(&mut self, size: usize) -> io::Result<&'a [u8]> {
        let end = self.position.checked_add(size).ok_or_else(invalid)?;
        let bytes = self.bytes.get(self.position..end).ok_or_else(invalid)?;
        self.position = end;
        Ok(bytes)
    }
    fn byte(&mut self) -> io::Result<u8> {
        Ok(self.take(1)?[0])
    }
    fn remaining(&self) -> &'a [u8] {
        &self.bytes[self.position..]
    }
    fn little(&mut self, size: usize) -> io::Result<u64> {
        Ok(self
            .take(size)?
            .iter()
            .enumerate()
            .fold(0, |value, (i, b)| value | (u64::from(*b) << (8 * i))))
    }
}

/// One frame's bounded Zstandard block/history/entropy state, with no source authority.
pub struct ZstdBlockDecoder {
    history: Vec<u8>,
    cursor: usize,
    filled: usize,
    output: Vec<u8>,
    literals: Vec<u8>,
    huffman: Option<HuffmanTable>,
    tables: [Option<FseTable>; 3],
    repeats: [usize; 3],
    failed: bool,
}
impl ZstdBlockDecoder {
    /// Admit a complete window and all bounded block/table scratch before allocation.
    pub fn new(window: u64, memory_limit: u64) -> io::Result<Self> {
        if window > 64 * 1024 * 1024
            || window
                .checked_add(OVERHEAD)
                .is_none_or(|n| n > memory_limit)
        {
            return Err(io::ErrorKind::OutOfMemory.into());
        }
        let window = usize::try_from(window).map_err(|_| io::ErrorKind::OutOfMemory)?;
        fn allocated(size: usize) -> io::Result<Vec<u8>> {
            let mut bytes = Vec::new();
            bytes
                .try_reserve_exact(size)
                .map_err(|_| io::ErrorKind::OutOfMemory)?;
            Ok(bytes)
        }
        let mut history = allocated(window)?;
        history.resize(window, 0);
        Ok(Self {
            history,
            cursor: 0,
            filled: 0,
            output: allocated(BLOCK)?,
            literals: allocated(BLOCK)?,
            huffman: None,
            tables: [None, None, None],
            repeats: [1, 4, 8],
            failed: false,
        })
    }
    fn bound(&self) -> usize {
        self.history.len().min(BLOCK)
    }
    fn start(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        self.output.clear();
        Ok(())
    }
    fn append(&mut self, byte: u8) -> io::Result<()> {
        if self.output.len() == self.bound() {
            return Err(invalid());
        }
        self.history[self.cursor] = byte;
        self.cursor += 1;
        if self.cursor == self.history.len() {
            self.cursor = 0;
        }
        self.filled = (self.filled + 1).min(self.history.len());
        self.output.push(byte);
        Ok(())
    }
    fn literals(&mut self, source: &mut Input<'_>) -> io::Result<()> {
        let first = *source.remaining().first().ok_or_else(invalid)?;
        let kind = first & 3;
        let mode = (first >> 2) & 3;
        self.literals.clear();
        if kind <= 1 {
            let size = if mode & 1 == 0 {
                usize::from(source.byte()? >> 3)
            } else {
                (source.little(if mode == 1 { 2 } else { 3 })? >> 4) as usize
            };
            if size > self.bound() {
                return Err(invalid());
            }
            if kind == 0 {
                self.literals.extend_from_slice(source.take(size)?);
            } else {
                self.literals.resize(size, source.byte()?);
            }
            return Ok(());
        }
        let field_bits = match mode {
            0 | 1 => 10,
            2 => 14,
            _ => 18,
        };
        let header = source.little(match mode {
            0 | 1 => 3,
            2 => 4,
            _ => 5,
        })?;
        let size = ((header >> 4) & ((1u64 << field_bits) - 1)) as usize;
        let packed = (header >> (4 + field_bits)) as usize;
        if size > self.bound() || packed == 0 {
            return Err(invalid());
        }
        let payload = source.take(packed)?;
        let payload = if kind == 2 {
            let (table, used) = HuffmanTable::read_description(payload)?;
            self.huffman = Some(table);
            &payload[used..]
        } else {
            payload
        };
        let table = self.huffman.as_ref().ok_or_else(invalid)?;
        if mode == 0 {
            self.literals = table.decode(payload, size)?;
        } else {
            if size < 6 || payload.len() < 10 {
                return Err(invalid());
            }
            let mut streams = Input {
                bytes: payload,
                position: 0,
            };
            let lengths = [
                streams.little(2)? as usize,
                streams.little(2)? as usize,
                streams.little(2)? as usize,
            ];
            let portion = size.div_ceil(4);
            for length in lengths {
                let bytes = table.decode(streams.take(length)?, portion)?;
                self.literals.extend(bytes);
            }
            let final_size = size.checked_sub(3 * portion).ok_or_else(invalid)?;
            self.literals
                .extend(table.decode(streams.remaining(), final_size)?);
        }
        Ok(())
    }
    fn copy_literals(&mut self, at: &mut usize, size: usize) -> io::Result<()> {
        let end = at.checked_add(size).ok_or_else(invalid)?;
        if end > self.literals.len()
            || self
                .output
                .len()
                .checked_add(size)
                .is_none_or(|n| n > self.bound())
        {
            return Err(invalid());
        }
        while *at < end {
            let byte = self.literals[*at];
            self.append(byte)?;
            *at += 1;
        }
        Ok(())
    }
    fn table(&mut self, source: &mut Input<'_>, index: usize, mode: u8) -> io::Result<()> {
        let kind = [
            FseSequenceKind::LiteralLength,
            FseSequenceKind::Offset,
            FseSequenceKind::MatchLength,
        ][index];
        let max = [35, 31, 52][index];
        let next = match mode {
            0 => FseTable::predefined(kind)?,
            1 => {
                let symbol = source.byte()?;
                if symbol > max {
                    return Err(invalid());
                }
                FseTable::rle(symbol)
            }
            2 => {
                let (table, used) =
                    FseTable::read_header(source.remaining(), max, if index == 1 { 8 } else { 9 })?;
                source.take(used)?;
                table
            }
            3 => {
                self.tables[index].as_ref().ok_or_else(invalid)?;
                return Ok(());
            }
            _ => return Err(invalid()),
        };
        self.tables[index] = Some(next);
        Ok(())
    }
    fn offset(&mut self, value: u64, literals: usize) -> io::Result<usize> {
        if value > 3 {
            let offset = usize::try_from(value - 3).map_err(|_| invalid())?;
            self.repeats = [offset, self.repeats[0], self.repeats[1]];
            return Ok(offset);
        }
        if value == 0 {
            return Err(invalid());
        }
        let index = value as usize - 1 + usize::from(literals == 0);
        if index == 3 {
            let offset = self.repeats[0]
                .checked_sub(1)
                .filter(|n| *n != 0)
                .ok_or_else(invalid)?;
            self.repeats = [offset, self.repeats[0], self.repeats[1]];
        } else {
            self.repeats[..=index].rotate_right(1);
        }
        Ok(self.repeats[0])
    }
    fn copy_match(&mut self, offset: usize, length: usize) -> io::Result<()> {
        if offset == 0
            || offset > self.filled
            || self
                .output
                .len()
                .checked_add(length)
                .is_none_or(|n| n > self.bound())
        {
            return Err(invalid());
        }
        for _ in 0..length {
            let from = (self.cursor + self.history.len() - offset) % self.history.len();
            let byte = self.history[from];
            self.append(byte)?;
        }
        Ok(())
    }
    fn compressed(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.len() > self.bound() {
            return Err(invalid());
        }
        let mut source = Input { bytes, position: 0 };
        self.literals(&mut source)?;
        let first = source.byte()?;
        let count = match first {
            0..=127 => usize::from(first),
            128..=254 => ((usize::from(first) - 128) << 8) + usize::from(source.byte()?),
            _ => source.little(2)? as usize + 0x7f00,
        };
        let mut literal_at = 0;
        if count == 0 {
            if !source.remaining().is_empty() {
                return Err(invalid());
            }
        } else {
            if count > self.bound() / 3 {
                return Err(invalid());
            }
            let modes = source.byte()?;
            if modes & 3 != 0 {
                return Err(invalid());
            }
            for (index, shift) in [6, 4, 2].into_iter().enumerate() {
                self.table(&mut source, index, (modes >> shift) & 3)?;
            }
            let mut bits = ReverseBits::new(source.remaining())?;
            let mut states = [
                self.tables[0]
                    .as_ref()
                    .expect("LL table")
                    .initial(&mut bits)?,
                self.tables[1]
                    .as_ref()
                    .expect("OF table")
                    .initial(&mut bits)?,
                self.tables[2]
                    .as_ref()
                    .expect("ML table")
                    .initial(&mut bits)?,
            ];
            for sequence in 0..count {
                let ll = self.tables[0]
                    .as_ref()
                    .expect("LL table")
                    .row(states[0])?
                    .symbol;
                let of = self.tables[1]
                    .as_ref()
                    .expect("OF table")
                    .row(states[1])?
                    .symbol;
                let ml = self.tables[2]
                    .as_ref()
                    .expect("ML table")
                    .row(states[2])?
                    .symbol;
                let offset_value = (1u64 << of) + u64::from(bits.take(of)?);
                let (base, width) = match_length(ml)?;
                let length = base + bits.take(width)? as usize;
                let (base, width) = literal_length(ll)?;
                let literals = base + bits.take(width)? as usize;
                self.copy_literals(&mut literal_at, literals)?;
                let offset = self.offset(offset_value, literals)?;
                self.copy_match(offset, length)?;
                if sequence + 1 != count {
                    for index in [0, 2, 1] {
                        states[index] = self.tables[index]
                            .as_ref()
                            .expect("admitted table")
                            .advance(states[index], &mut bits)?;
                    }
                }
            }
            if bits.remaining() != 0 {
                return Err(invalid());
            }
        }
        let remaining = self.literals.len() - literal_at;
        self.copy_literals(&mut literal_at, remaining)
    }
    /// Decode a raw block while retaining history and prior entropy/offset state.
    pub fn raw(&mut self, bytes: &[u8]) -> io::Result<&[u8]> {
        self.start()?;
        let result = (|| {
            if bytes.len() > self.bound() {
                return Err(invalid());
            }
            for byte in bytes {
                self.append(*byte)?;
            }
            Ok(())
        })();
        self.failed = result.is_err();
        result.map(|_| self.output.as_slice())
    }
    /// Decode a block-level repeated byte with the same work/history admission.
    pub fn rle(&mut self, byte: u8, size: usize) -> io::Result<&[u8]> {
        self.start()?;
        let result = (|| {
            if size > self.bound() {
                return Err(invalid());
            }
            for _ in 0..size {
                self.append(byte)?;
            }
            Ok(())
        })();
        self.failed = result.is_err();
        result.map(|_| self.output.as_slice())
    }
    /// Decode one exact compressed block, preserving only frame-defined reusable state.
    pub fn decode_compressed(&mut self, bytes: &[u8]) -> io::Result<&[u8]> {
        self.start()?;
        let result = self.compressed(bytes);
        self.failed = result.is_err();
        result.map(|_| self.output.as_slice())
    }
}
fn literal_length(code: u8) -> io::Result<(usize, u8)> {
    const BASE: [usize; 20] = [
        16, 18, 20, 22, 24, 28, 32, 40, 48, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384,
        32768, 65536,
    ];
    const BITS: [u8; 20] = [
        1, 1, 1, 1, 2, 2, 3, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ];
    match code {
        0..=15 => Ok((usize::from(code), 0)),
        16..=35 => Ok((BASE[usize::from(code - 16)], BITS[usize::from(code - 16)])),
        _ => Err(invalid()),
    }
}
fn match_length(code: u8) -> io::Result<(usize, u8)> {
    const BASE: [usize; 21] = [
        35, 37, 39, 41, 43, 47, 51, 59, 67, 83, 99, 131, 259, 515, 1027, 2051, 4099, 8195, 16387,
        32771, 65539,
    ];
    const BITS: [u8; 21] = [
        1, 1, 1, 1, 2, 2, 3, 3, 4, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ];
    match code {
        0..=31 => Ok((usize::from(code) + 3, 0)),
        32..=52 => Ok((BASE[usize::from(code - 32)], BITS[usize::from(code - 32)])),
        _ => Err(invalid()),
    }
}
