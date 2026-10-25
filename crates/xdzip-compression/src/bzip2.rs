//! Complete single-stream BZip2 decoder with independently implemented entropy
//! decoding, stable inverse BWT, historical derandomization and streaming RLE.
use crate::{
    invalid,
    stream_io::{Input, READ_QUANTUM, filled, resource},
};
use std::io::{self, Read};

#[path = "bzip2_random.rs"]
mod random;
use random::RANDOM_PERIODS;

const SELECTORS: usize = 18002;
const FIXED_MEMORY: u64 = 65536;
const BLOCK_MAGIC: u64 = 0x3141_5926_5359;
const END_MAGIC: u64 = 0x1772_4538_5090;

/// Conservative working-memory budget for BZh1..BZh9, covering the last column,
/// stable permutation, selectors, entropy tables and bounded buffered input.
/// The block bound applies before RLE expansion; output is never preallocated.
pub fn bzip2_memory_required(level: u8) -> io::Result<u64> {
    if !(1..=9).contains(&level) {
        return Err(invalid());
    }
    Ok(u64::from(level) * 100000 * 5 + FIXED_MEMORY)
}

struct Bits<R> {
    input: Input<R>,
    pending: u64,
    available: u8,
}
impl<R: Read> Bits<R> {
    fn take(&mut self, count: u8) -> io::Result<u32> {
        while self.available < count {
            self.pending = (self.pending << 8) | u64::from(self.input.byte(self.input.packed)?);
            self.available += 8;
        }
        self.available -= count;
        let result = ((self.pending >> self.available) & ((1u64 << count) - 1)) as u32;
        self.pending &= (1u64 << self.available) - 1;
        Ok(result)
    }
    fn magic(&mut self) -> io::Result<u64> {
        Ok((u64::from(self.take(24)?) << 24) | u64::from(self.take(24)?))
    }
    fn end(&self) -> bool {
        // The 7z coder accepts arbitrary unused low bits in the final byte.
        // They are not an additional byte or an additional stream.
        self.input.consumed == self.input.packed
    }
}

struct Huffman {
    counts: [u16; 21],
    first: [u32; 21],
    offsets: [u16; 21],
    symbols: [u16; 258],
    maximum: u8,
}
impl Huffman {
    fn build(lengths: &[u8]) -> io::Result<Self> {
        let mut result = Self {
            counts: [0; 21],
            first: [0; 21],
            offsets: [0; 21],
            symbols: [0; 258],
            maximum: 0,
        };
        if lengths.len() > result.symbols.len() {
            return Err(invalid());
        }
        for &length in lengths {
            if !(1..=20).contains(&length) {
                return Err(invalid());
            }
            result.counts[length as usize] += 1;
            result.maximum = result.maximum.max(length);
        }
        let mut offset = 0;
        for length in 1..=20 {
            result.first[length] =
                (result.first[length - 1] + u32::from(result.counts[length - 1])) * 2;
            if result.first[length] + u32::from(result.counts[length]) > 1 << length {
                return Err(invalid());
            }
            result.offsets[length] = offset;
            offset += result.counts[length];
        }
        let mut used = [0u16; 21];
        for (symbol, &length) in lengths.iter().enumerate() {
            let length = length as usize;
            let index = result.offsets[length] + used[length];
            result.symbols[index as usize] = symbol as u16;
            used[length] += 1;
        }
        Ok(result)
    }
    fn decode<R: Read>(&self, bits: &mut Bits<R>) -> io::Result<u16> {
        let mut code = 0u32;
        for length in 1..=usize::from(self.maximum) {
            code = (code << 1) | bits.take(1)?;
            if let Some(delta) = code.checked_sub(self.first[length])
                && delta < u32::from(self.counts[length])
            {
                return Ok(self.symbols[usize::from(self.offsets[length]) + delta as usize]);
            }
        }
        Err(invalid())
    }
}

struct Entropy {
    tables: Vec<Huffman>,
    selector: usize,
    left: usize,
    active: usize,
    selector_count: usize,
}
impl Entropy {
    fn next<R: Read>(&mut self, bits: &mut Bits<R>, selectors: &[u8]) -> io::Result<u16> {
        if self.left == 0 {
            if self.selector >= self.selector_count {
                return Err(invalid());
            }
            self.active = usize::from(selectors[self.selector]);
            self.selector += 1;
            self.left = 50;
        }
        self.left -= 1;
        self.tables[self.active].decode(bits)
    }
}

const fn crc_table() -> [u32; 256] {
    let mut result = [0; 256];
    let mut byte = 0;
    while byte < 256 {
        let mut value = (byte as u32) << 24;
        let mut bit = 0;
        while bit < 8 {
            value = (value << 1)
                ^ if value & 0x8000_0000 != 0 {
                    0x04c1_1db7
                } else {
                    0
                };
            bit += 1;
        }
        result[byte] = value;
        byte += 1;
    }
    result
}
const CRC: [u32; 256] = crc_table();
fn crc_byte(crc: u32, byte: u8) -> u32 {
    (crc << 8) ^ CRC[((crc >> 24) as u8 ^ byte) as usize]
}

struct Block {
    remaining: usize,
    cursor: usize,
    expected_crc: u32,
    crc: u32,
    randomized: bool,
    random_count: u16,
    random_index: usize,
    repeated: u8,
    streak: u8,
    extra: u8,
    length: usize,
}
impl Block {
    fn intermediate(&mut self, last: &[u8], permutation: &[u32]) -> io::Result<u8> {
        if self.remaining == 0 || self.cursor >= self.length {
            return Err(invalid());
        }
        let mut byte = last[self.cursor];
        self.cursor = permutation[self.cursor] as usize;
        self.remaining -= 1;
        if self.randomized {
            if self.random_count == 0 {
                self.random_count = RANDOM_PERIODS[self.random_index];
                self.random_index = (self.random_index + 1) % RANDOM_PERIODS.len();
            }
            self.random_count -= 1;
            if self.random_count == 1 {
                byte ^= 1;
            }
        }
        Ok(byte)
    }
    fn next(&mut self, last: &[u8], permutation: &[u32]) -> io::Result<Option<u8>> {
        let byte = loop {
            if self.extra != 0 {
                self.extra -= 1;
                break self.repeated;
            }
            if self.streak == 4 {
                // Historical writers can end a block immediately after the
                // fourth repeated byte. CRC and declared size still apply.
                if self.remaining == 0 {
                    return Ok(None);
                }
                self.extra = self.intermediate(last, permutation)?;
                self.streak = 0;
                continue;
            }
            if self.remaining == 0 {
                return Ok(None);
            }
            let byte = self.intermediate(last, permutation)?;
            if self.streak != 0 && byte == self.repeated {
                self.streak += 1;
            } else {
                self.repeated = byte;
                self.streak = 1;
            }
            break byte;
        };
        self.crc = crc_byte(self.crc, byte);
        Ok(Some(byte))
    }
}

/// Exact-span complete BZip2 stream reader. Valid EOF requires every block CRC,
/// combined CRC, exact output size and exact packed input. Unused low bits of
/// the final byte are ignored for 7z compatibility; additional bytes and
/// concatenated streams are rejected.
/// A block ending after four repeated bytes may omit its count byte.
/// Each read emits at most 8 KiB. Starting a block first processes at most
/// 900000 intermediate bytes; inverse BWT requires the complete bounded block.
pub struct Bzip2Reader<R> {
    bits: Bits<R>,
    last: Vec<u8>,
    permutation: Vec<u32>,
    selectors: Vec<u8>,
    block: Option<Block>,
    expected: u64,
    decoded: u64,
    combined: u32,
    ended: bool,
    failed: bool,
}
impl<R: Read> Bzip2Reader<R> {
    /// Read the four-byte BZhN header within the authorized packed range, then
    /// admit the actual level's full working memory before allocating it.
    /// Typed initialization I/O errors preserve their concrete error object.
    pub fn new(source: R, packed: u64, expected: u64, memory_limit: u64) -> io::Result<Self> {
        let mut input = Input::new(source, packed);
        if input.byte(packed)? != b'B' || input.byte(packed)? != b'Z' || input.byte(packed)? != b'h'
        {
            return Err(invalid());
        }
        let level = input.byte(packed)?.checked_sub(b'0').ok_or_else(invalid)?;
        if bzip2_memory_required(level)? > memory_limit {
            return Err(resource());
        }
        let maximum = usize::from(level) * 100000;
        Ok(Self {
            bits: Bits {
                input,
                pending: 0,
                available: 0,
            },
            last: filled(maximum, 0)?,
            permutation: filled(maximum, 0)?,
            selectors: filled(SELECTORS, 0)?,
            block: None,
            expected,
            decoded: 0,
            combined: 0,
            ended: false,
            failed: false,
        })
    }
    /// Logical bytes consumed, including the four-byte header and current bit byte.
    pub fn consumed_bytes(&self) -> u64 {
        self.bits.input.consumed
    }
    /// Uncompressed bytes decoded into caller buffers.
    pub fn decoded_bytes(&self) -> u64 {
        self.decoded
    }
    fn entropy(&mut self, alphabet: usize) -> io::Result<Entropy> {
        let tables = self.bits.take(3)? as usize;
        let selectors = self.bits.take(15)? as usize;
        if !(2..=6).contains(&tables) || selectors == 0 {
            return Err(invalid());
        }
        let mut order = [0u8, 1, 2, 3, 4, 5];
        for index in 0..selectors {
            let mut unary = 0;
            while self.bits.take(1)? != 0 {
                unary += 1;
                if unary >= tables {
                    return Err(invalid());
                }
            }
            let value = order[unary];
            order[..=unary].rotate_right(1);
            if index < SELECTORS {
                self.selectors[index] = value;
            }
        }
        let mut huffman = Vec::new();
        huffman.try_reserve_exact(tables).map_err(|_| resource())?;
        let mut lengths = [0u8; 258];
        for _ in 0..tables {
            let mut length = self.bits.take(5)? as u8;
            if !(1..=20).contains(&length) {
                return Err(invalid());
            }
            for value in &mut lengths[..alphabet] {
                while self.bits.take(1)? != 0 {
                    let adjustment = self.bits.take(1)?;
                    length = if adjustment == 0 {
                        length.checked_add(1)
                    } else {
                        length.checked_sub(1)
                    }
                    .ok_or_else(invalid)?;
                    if !(1..=20).contains(&length) {
                        return Err(invalid());
                    }
                }
                *value = length;
            }
            huffman.push(Huffman::build(&lengths[..alphabet])?);
        }
        Ok(Entropy {
            tables: huffman,
            selector: 0,
            left: 0,
            active: 0,
            selector_count: selectors.min(SELECTORS),
        })
    }
    fn load_block(&mut self) -> io::Result<()> {
        let magic = self.bits.magic()?;
        if magic == END_MAGIC {
            if self.bits.take(32)? != self.combined
                || self.decoded != self.expected
                || !self.bits.end()
            {
                return Err(invalid());
            }
            self.ended = true;
            return Ok(());
        }
        if magic != BLOCK_MAGIC || self.decoded == self.expected {
            return Err(invalid());
        }
        let expected_crc = self.bits.take(32)?;
        let randomized = self.bits.take(1)? != 0;
        let origin = self.bits.take(24)? as usize;
        if origin >= self.last.len() {
            return Err(invalid());
        }
        let groups = self.bits.take(16)?;
        let mut alphabet = [0u8; 256];
        let mut count = 0usize;
        for group in 0..16 {
            if groups & (1 << (15 - group)) == 0 {
                continue;
            }
            let members = self.bits.take(16)?;
            for member in 0..16 {
                if members & (1 << (15 - member)) != 0 {
                    alphabet[count] = (group * 16 + member) as u8;
                    count += 1;
                }
            }
        }
        if count == 0 {
            return Err(invalid());
        }
        let mut entropy = self.entropy(count + 2)?;
        let mut size = 0usize;
        let mut symbol = entropy.next(&mut self.bits, &self.selectors)?;
        loop {
            let mut run = 0usize;
            let mut weight = 1usize;
            while symbol <= 1 {
                let term = weight
                    .checked_mul(usize::from(symbol) + 1)
                    .ok_or_else(invalid)?;
                run = run.checked_add(term).ok_or_else(invalid)?;
                if run > self.last.len() - size {
                    return Err(invalid());
                }
                weight = weight.checked_mul(2).ok_or_else(invalid)?;
                symbol = entropy.next(&mut self.bits, &self.selectors)?;
            }
            self.last[size..size + run].fill(alphabet[0]);
            size += run;
            if symbol as usize == count + 1 {
                break;
            }
            let index = usize::from(symbol).checked_sub(1).ok_or_else(invalid)?;
            if index >= count || size == self.last.len() {
                return Err(invalid());
            }
            let byte = alphabet[index];
            alphabet[..=index].rotate_right(1);
            self.last[size] = byte;
            size += 1;
            symbol = entropy.next(&mut self.bits, &self.selectors)?;
        }
        if size == 0 || origin >= size {
            return Err(invalid());
        }
        let mut frequencies = [0usize; 256];
        for &byte in &self.last[..size] {
            frequencies[byte as usize] += 1;
        }
        let mut accumulated = 0;
        for frequency in &mut frequencies {
            let count = *frequency;
            *frequency = accumulated;
            accumulated += count;
        }
        for (index, &byte) in self.last[..size].iter().enumerate() {
            let target = &mut frequencies[byte as usize];
            self.permutation[*target] = index as u32;
            *target += 1;
        }
        self.block = Some(Block {
            remaining: size,
            cursor: self.permutation[origin] as usize,
            expected_crc,
            crc: u32::MAX,
            randomized,
            random_count: 0,
            random_index: 0,
            repeated: 0,
            streak: 0,
            extra: 0,
            length: size,
        });
        Ok(())
    }
    fn next(&mut self) -> io::Result<Option<u8>> {
        loop {
            if let Some(block) = &mut self.block {
                if let Some(byte) = block.next(&self.last, &self.permutation)? {
                    return Ok(Some(byte));
                }
                let crc = !block.crc;
                if crc != block.expected_crc {
                    return Err(invalid());
                }
                self.combined = self.combined.rotate_left(1) ^ crc;
                self.block = None;
            }
            self.load_block()?;
            if self.ended {
                return Ok(None);
            }
        }
    }
    /// Validate remaining count/CRC/end fields after all declared output was
    /// read, without discarding unread output. Any failure poisons the reader.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(());
        }
        let result = if self.decoded != self.expected {
            Err(invalid())
        } else {
            self.next().and_then(|byte| {
                if byte.is_none() {
                    Ok(())
                } else {
                    Err(invalid())
                }
            })
        };
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let limit = output.len().min(READ_QUANTUM);
        let mut count = 0;
        while count < limit {
            if self.decoded == self.expected {
                self.finish()?;
                break;
            }
            match self.next()? {
                Some(byte) => {
                    output[count] = byte;
                    self.decoded = self.decoded.checked_add(1).ok_or_else(invalid)?;
                    count += 1;
                }
                None => break,
            }
        }
        Ok(count)
    }
}
impl<R: Read> Read for Bzip2Reader<R> {
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

#[cfg(test)]
#[path = "bzip2_tests.rs"]
mod tests;
