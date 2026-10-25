use crate::{DISTANCE_BITS, DISTANCES, LENGTH_BITS, LENGTHS, invalid};
use std::io::{self, BufReader, Read, Write};
use std::{cmp::Reverse, collections::BinaryHeap};

// Construct an optimal frequency tree with deterministic ties. Rare alphabets
// exceeding the wire depth limit use the fixed tree, never truncate lengths.
fn code_lengths(frequencies: &[u32]) -> Option<Vec<u8>> {
    let mut queue = BinaryHeap::new();
    let mut parents = vec![usize::MAX; frequencies.len()];
    for (symbol, frequency) in frequencies.iter().enumerate() {
        if *frequency != 0 {
            queue.push(Reverse((u64::from(*frequency), symbol)));
        }
    }
    let mut result = vec![0u8; frequencies.len()];
    if queue.len() == 1 {
        result[queue.pop()?.0.1] = 1;
        return Some(result);
    }
    while queue.len() > 1 {
        let Reverse((a, left)) = queue.pop()?;
        let Reverse((b, right)) = queue.pop()?;
        let parent = parents.len();
        parents.push(usize::MAX);
        parents[left] = parent;
        parents[right] = parent;
        queue.push(Reverse((a + b, parent)));
    }
    for (symbol, frequency) in frequencies.iter().enumerate() {
        if *frequency == 0 {
            continue;
        }
        let mut node = symbol;
        while parents[node] != usize::MAX {
            result[symbol] += 1;
            if result[symbol] > 15 {
                return None;
            }
            node = parents[node];
        }
    }
    Some(result)
}

fn wire_codes(lengths: &[u8]) -> Vec<(u32, u8)> {
    let mut count = [0u32; 16];
    for length in lengths {
        if *length != 0 {
            count[*length as usize] += 1;
        }
    }
    let mut next = [0u32; 16];
    for length in 1..16 {
        next[length] = (next[length - 1] + count[length - 1]) << 1;
    }
    lengths
        .iter()
        .map(|length| {
            if *length == 0 {
                return (0, 0);
            }
            let code = next[*length as usize];
            next[*length as usize] += 1;
            (code.reverse_bits() >> (32 - length), *length)
        })
        .collect()
}

pub(crate) struct Bits<R> {
    pub(crate) source: BufReader<R>,
    pub(crate) consumed: u64,
    value: u32,
    count: u8,
}
impl<R: Read> Bits<R> {
    pub(crate) fn new(source: R) -> Self {
        Self {
            source: BufReader::new(source),
            consumed: 0,
            value: 0,
            count: 0,
        }
    }
    pub(crate) fn get(&mut self, n: u8) -> io::Result<u32> {
        while self.count < n {
            let mut byte = [0];
            self.source.read_exact(&mut byte)?;
            self.consumed = self.consumed.checked_add(1).ok_or_else(invalid)?;
            self.value |= u32::from(byte[0]) << self.count;
            self.count += 8;
        }
        let value = self.value & ((1u32 << n) - 1);
        self.value >>= n;
        self.count -= n;
        Ok(value)
    }
    pub(crate) fn align(&mut self) {
        self.value = 0;
        self.count = 0;
    }
}

struct Huffman {
    count: [u16; 16],
    first: [u16; 16],
    offset: [usize; 16],
    symbols: Vec<u16>,
}
impl Huffman {
    fn new(lengths: &[u8], empty: bool, single: bool) -> io::Result<Self> {
        let mut count = [0u16; 16];
        for length in lengths {
            if *length > 15 {
                return Err(invalid());
            }
            if *length != 0 {
                count[*length as usize] += 1;
            }
        }
        let used: usize = count.iter().map(|n| usize::from(*n)).sum();
        if used == 0 && !empty {
            return Err(invalid());
        }
        let mut available = 1i32;
        for n in count.iter().skip(1) {
            available = available * 2 - i32::from(*n);
            if available < 0 {
                return Err(invalid());
            }
        }
        if available != 0 && used != 0 && !(single && count[1] == 1 && used == 1) {
            return Err(invalid());
        }
        let mut first = [0; 16];
        let mut offset = [0; 16];
        for length in 1..16 {
            first[length] = (first[length - 1] + count[length - 1]) << 1;
            offset[length] = offset[length - 1] + usize::from(count[length - 1]);
        }
        let mut symbols = vec![0; used];
        let mut next = offset;
        for (symbol, length) in lengths.iter().enumerate() {
            if *length != 0 {
                symbols[next[*length as usize]] = symbol as u16;
                next[*length as usize] += 1;
            }
        }
        Ok(Self {
            count,
            first,
            offset,
            symbols,
        })
    }
    fn symbol<R: Read>(&self, bits: &mut Bits<R>) -> io::Result<u16> {
        let mut code = 0u32;
        for length in 1..16 {
            code = (code << 1) | bits.get(1)?;
            let first = u32::from(self.first[length]);
            if code >= first && code - first < u32::from(self.count[length]) {
                return Ok(self.symbols[self.offset[length] + (code - first) as usize]);
            }
        }
        Err(invalid())
    }
}

fn fixed() -> (Huffman, Huffman) {
    let mut lengths = [8; 288];
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    (
        Huffman::new(&lengths, false, false).expect("complete fixed alphabet"),
        Huffman::new(&[5; 32], false, false).expect("complete distance alphabet"),
    )
}

fn dynamic<R: Read>(bits: &mut Bits<R>) -> io::Result<(Huffman, Huffman)> {
    let literal_count = bits.get(5)? as usize + 257;
    let distance_count = bits.get(5)? as usize + 1;
    let code_count = bits.get(4)? as usize + 4;
    if literal_count > 286 {
        return Err(invalid());
    }
    let order = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut codes = [0; 19];
    for at in &order[..code_count] {
        codes[*at] = bits.get(3)? as u8;
    }
    let decoder = Huffman::new(&codes, false, false)?;
    let total = literal_count + distance_count;
    let mut lengths = Vec::with_capacity(total);
    while lengths.len() < total {
        let symbol = decoder.symbol(bits)?;
        let (length, repeat) = match symbol {
            0..=15 => (symbol as u8, 1),
            16 => (
                *lengths.last().ok_or_else(invalid)?,
                bits.get(2)? as usize + 3,
            ),
            17 => (0, bits.get(3)? as usize + 3),
            18 => (0, bits.get(7)? as usize + 11),
            _ => return Err(invalid()),
        };
        if lengths.len() + repeat > total {
            return Err(invalid());
        }
        lengths.resize(lengths.len() + repeat, length);
    }
    if lengths[256] == 0 {
        return Err(invalid());
    }
    Ok((
        Huffman::new(&lengths[..literal_count], false, true)?,
        Huffman::new(&lengths[literal_count..], true, true)?,
    ))
}

enum Block {
    Header,
    Stored(usize),
    Coded(Box<(Huffman, Huffman)>),
    End,
}
pub(crate) struct Inflate {
    block: Block,
    last: bool,
    history: Box<[u8; 32768]>,
    position: usize,
    available: usize,
    distance: usize,
    copying: usize,
}
impl Inflate {
    pub(crate) fn new() -> Self {
        Self {
            block: Block::Header,
            last: false,
            history: Box::new([0; 32768]),
            position: 0,
            available: 0,
            distance: 0,
            copying: 0,
        }
    }
    fn emit(&mut self, byte: u8, output: &mut [u8], at: &mut usize) {
        output[*at] = byte;
        *at += 1;
        self.history[self.position] = byte;
        self.position = (self.position + 1) & 32767;
        self.available = (self.available + 1).min(32768);
    }
    pub(crate) fn remember(&mut self, bytes: &[u8]) {
        if bytes.len() >= 32768 {
            self.history.copy_from_slice(&bytes[bytes.len() - 32768..]);
            self.position = 0;
            self.available = 32768;
        } else {
            let first = bytes.len().min(32768 - self.position);
            self.history[self.position..self.position + first].copy_from_slice(&bytes[..first]);
            self.history[..bytes.len() - first].copy_from_slice(&bytes[first..]);
            self.position = (self.position + bytes.len()) & 32767;
            self.available = (self.available + bytes.len()).min(32768);
        }
    }
    pub(crate) fn read<R: Read>(
        &mut self,
        bits: &mut Bits<R>,
        output: &mut [u8],
    ) -> io::Result<usize> {
        let mut at = 0;
        while at < output.len() {
            if self.copying != 0 {
                let byte = self.history[(self.position + 32768 - self.distance) & 32767];
                self.emit(byte, output, &mut at);
                self.copying -= 1;
                continue;
            }
            match &mut self.block {
                Block::Header => {
                    self.last = bits.get(1)? != 0;
                    self.block = match bits.get(2)? {
                        0 => {
                            bits.align();
                            let length = bits.get(16)?;
                            if bits.get(16)? != length ^ 65535 {
                                return Err(invalid());
                            }
                            Block::Stored(length as usize)
                        }
                        1 => {
                            let (lit, dist) = fixed();
                            Block::Coded(Box::new((lit, dist)))
                        }
                        2 => {
                            let (lit, dist) = dynamic(bits)?;
                            Block::Coded(Box::new((lit, dist)))
                        }
                        _ => return Err(invalid()),
                    };
                }
                Block::Stored(left) => {
                    if *left == 0 {
                        self.block = if self.last { Block::End } else { Block::Header };
                    } else {
                        let take = (*left).min(output.len() - at);
                        let n = loop {
                            match bits.source.read(&mut output[at..at + take]) {
                                Err(error) if error.kind() == io::ErrorKind::Interrupted => {
                                    continue;
                                }
                                result => break result?,
                            }
                        };
                        if n == 0 {
                            return Err(io::ErrorKind::UnexpectedEof.into());
                        }
                        *left -= n;
                        bits.consumed = bits.consumed.checked_add(n as u64).ok_or_else(invalid)?;
                        self.remember(&output[at..at + n]);
                        at += n;
                    }
                }
                Block::Coded(codes) => match codes.0.symbol(bits)? {
                    byte @ 0..=255 => self.emit(byte as u8, output, &mut at),
                    256 => {
                        self.block = if self.last { Block::End } else { Block::Header };
                    }
                    symbol @ 257..=285 => {
                        let index = usize::from(symbol - 257);
                        let length = LENGTHS[index] + bits.get(LENGTH_BITS[index])? as usize;
                        let distance = usize::from(codes.1.symbol(bits)?);
                        if distance >= 30 {
                            return Err(invalid());
                        }
                        let distance =
                            DISTANCES[distance] + bits.get(DISTANCE_BITS[distance])? as usize;
                        if distance > self.available {
                            return Err(invalid());
                        }
                        self.distance = distance;
                        self.copying = length;
                    }
                    _ => return Err(invalid()),
                },
                Block::End => {
                    bits.align();
                    break;
                }
            }
        }
        Ok(at)
    }
}

pub(crate) struct BitOutput<W> {
    pub(crate) sink: W,
    bytes: Vec<u8>,
    value: u32,
    count: u8,
}
impl<W: Write> BitOutput<W> {
    pub(crate) fn new(sink: W) -> Self {
        Self {
            sink,
            bytes: Vec::with_capacity(65536),
            value: 0,
            count: 0,
        }
    }
    fn put(&mut self, value: u32, count: u8) -> io::Result<()> {
        self.value |= value << self.count;
        self.count += count;
        while self.count >= 8 {
            self.bytes.push(self.value as u8);
            if self.bytes.len() == 65536 {
                self.flush_bytes()?;
            }
            self.value >>= 8;
            self.count -= 8;
        }
        Ok(())
    }
    pub(crate) fn align(&mut self) -> io::Result<()> {
        if self.count != 0 {
            self.put(0, 8 - self.count)?;
        }
        self.flush_bytes()
    }
    pub(crate) fn flush_bytes(&mut self) -> io::Result<()> {
        self.sink.write_all(&self.bytes)?;
        self.bytes.clear();
        Ok(())
    }
    fn code(&mut self, codes: &[(u32, u8)], symbol: usize) -> io::Result<()> {
        let (code, width) = codes[symbol];
        self.put(code, width)
    }
    fn tree(&mut self, literals: &[u8], distances: &[u8]) -> io::Result<()> {
        self.put(29, 5)?; // 286 literal/length codes
        self.put(29, 5)?; // 30 distance codes
        self.put(15, 4)?; // all 19 code-length codes
        // Complete code-length alphabet: thirteen 4-bit and six 5-bit codes.
        // This avoids unbounded optimization of this small secondary alphabet.
        let mut secondary = [4; 19];
        secondary[13..].fill(5);
        let codes = wire_codes(&secondary);
        for symbol in [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
        ] {
            self.put(u32::from(secondary[symbol]), 3)?;
        }
        let lengths: Vec<u8> = literals.iter().chain(distances).copied().collect();
        let mut at = 0;
        while at < lengths.len() {
            let value = lengths[at];
            let run = lengths[at..].iter().take_while(|v| **v == value).count();
            if value == 0 && run >= 3 {
                let take = run.min(138);
                if take >= 11 {
                    self.code(&codes, 18)?;
                    self.put((take - 11) as u32, 7)?;
                } else {
                    self.code(&codes, 17)?;
                    self.put((take - 3) as u32, 3)?;
                }
                at += take;
            } else {
                self.code(&codes, usize::from(value))?;
                at += 1;
                let mut left = run - 1;
                while left >= 3 {
                    let take = left.min(6);
                    self.code(&codes, 16)?;
                    self.put((take - 3) as u32, 2)?;
                    at += take;
                    left -= take;
                }
                for _ in 0..left {
                    self.code(&codes, usize::from(value))?;
                    at += 1;
                }
            }
        }
        Ok(())
    }
    pub(crate) fn block(&mut self, bytes: &[u8], last: bool, level: u8) -> io::Result<()> {
        if level == 0 {
            self.put(u32::from(last), 1)?;
            self.put(0, 2)?;
            self.align()?;
            let length = u16::try_from(bytes.len()).map_err(|_| invalid())?;
            self.sink.write_all(&length.to_le_bytes())?;
            self.sink.write_all(&(!length).to_le_bytes())?;
            return self.sink.write_all(bytes);
        }
        // Independent bounded hash chains. Matches never exceed the 32 KiB
        // wire window or 258-byte length. Encoder history restarts per block.
        let mut heads = vec![usize::MAX; 65536];
        let mut previous = vec![usize::MAX; bytes.len()];
        let depth = match level {
            1 => 4,
            6 => 32,
            _ => 128,
        };
        let hash = |at: usize| {
            ((usize::from(bytes[at]) * 251 + usize::from(bytes[at + 1])) * 251
                + usize::from(bytes[at + 2]))
                & 65535
        };
        let mut at = 0;
        let mut tokens = Vec::new();
        let mut literals = [0u32; 286];
        let mut distances = [0u32; 30];
        literals[256] = 1;
        while at < bytes.len() {
            let mut best = 0;
            let mut distance = 0;
            if at + 2 < bytes.len() {
                let mut candidate = heads[hash(at)];
                for _ in 0..depth {
                    if candidate == usize::MAX || at - candidate > 32768 {
                        break;
                    }
                    let mut length = 0;
                    while length < 258
                        && at + length < bytes.len()
                        && bytes[candidate + length] == bytes[at + length]
                    {
                        length += 1;
                    }
                    if length > best {
                        best = length;
                        distance = at - candidate;
                    }
                    if best == 258 {
                        break;
                    }
                    candidate = previous[candidate];
                }
            }
            let take = if best >= 3 { best } else { 1 };
            if best >= 3 {
                let code = (0..29)
                    .rev()
                    .find(|i| LENGTHS[*i] <= best)
                    .expect("length at least three");
                literals[code + 257] += 1;
                let code = (0..30)
                    .rev()
                    .find(|i| DISTANCES[*i] <= distance)
                    .expect("positive distance");
                distances[code] += 1;
                tokens.push((best as u16, distance as u16));
            } else {
                literals[usize::from(bytes[at])] += 1;
                tokens.push((u16::from(bytes[at]), 0));
            }
            for (offset, link) in previous.iter_mut().enumerate().skip(at).take(take) {
                if offset + 2 < bytes.len() {
                    let key = hash(offset);
                    *link = heads[key];
                    heads[key] = offset;
                }
            }
            at += take;
        }
        if distances.iter().all(|n| *n == 0) {
            distances[0] = 1;
        }
        let lengths = code_lengths(&literals).zip(code_lengths(&distances));
        if let Some((lit, dist)) = &lengths {
            let bits: usize = literals
                .iter()
                .zip(lit)
                .map(|(n, width)| *n as usize * usize::from(*width))
                .sum::<usize>()
                + distances
                    .iter()
                    .zip(dist)
                    .map(|(n, width)| *n as usize * usize::from(*width))
                    .sum::<usize>()
                + literals[257..]
                    .iter()
                    .zip(LENGTH_BITS)
                    .map(|(n, extra)| *n as usize * usize::from(extra))
                    .sum::<usize>()
                + distances
                    .iter()
                    .zip(DISTANCE_BITS)
                    .map(|(n, extra)| *n as usize * usize::from(extra))
                    .sum::<usize>();
            if !bytes.is_empty() && bits + 2048 >= bytes.len() * 8 {
                let chunks = bytes.chunks(65535);
                let count = chunks.len();
                for (index, chunk) in chunks.enumerate() {
                    self.block(chunk, last && index + 1 == count, 0)?;
                }
                return Ok(());
            }
        }
        self.put(u32::from(last), 1)?;
        let (literal_codes, distance_codes) = if let Some((literals, distances)) = lengths {
            self.put(2, 2)?;
            self.tree(&literals, &distances)?;
            (wire_codes(&literals), wire_codes(&distances))
        } else {
            self.put(1, 2)?;
            let mut lengths = [8; 288];
            lengths[144..256].fill(9);
            lengths[256..280].fill(7);
            (wire_codes(&lengths), wire_codes(&[5; 32]))
        };
        for (value, distance) in tokens {
            let value = usize::from(value);
            let distance = usize::from(distance);
            if distance == 0 {
                self.code(&literal_codes, value)?;
            } else {
                let code = (0..29)
                    .rev()
                    .find(|i| LENGTHS[*i] <= value)
                    .expect("match length");
                self.code(&literal_codes, code + 257)?;
                self.put((value - LENGTHS[code]) as u32, LENGTH_BITS[code])?;
                let code = (0..30)
                    .rev()
                    .find(|i| DISTANCES[*i] <= distance)
                    .expect("match distance");
                self.code(&distance_codes, code)?;
                self.put((distance - DISTANCES[code]) as u32, DISTANCE_BITS[code])?;
            }
        }
        self.code(&literal_codes, 256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn huffman_admission_rejects_oversubscription_missing_codes_and_bad_lengths() {
        assert!(Huffman::new(&[1, 1, 1], false, true).is_err());
        assert!(Huffman::new(&[2, 2], false, true).is_err());
        assert!(Huffman::new(&[16], false, true).is_err());
        assert!(Huffman::new(&[0, 0], false, true).is_err());
        assert!(Huffman::new(&[0], true, true).is_ok());
        assert!(Huffman::new(&[1], false, true).is_ok());
        assert!(Huffman::new(&[1], false, false).is_err());
    }
    #[test]
    fn stored_history_is_retained_for_following_coded_distance_32768() {
        let prefix: Vec<u8> = (0..=255).cycle().take(65535).collect();
        let mut raw = BitOutput::new(Vec::new());
        raw.block(&prefix, false, 0).unwrap();
        raw.put(3, 3).unwrap(); // final fixed Huffman block
        let mut lengths = [8; 288];
        lengths[144..256].fill(9);
        lengths[256..280].fill(7);
        let literals = wire_codes(&lengths);
        raw.code(&literals, 285).unwrap();
        raw.code(&wire_codes(&[5; 32]), 29).unwrap();
        raw.put(8191, 13).unwrap();
        raw.code(&literals, 256).unwrap();
        raw.align().unwrap();
        let mut expected = prefix.clone();
        expected.extend_from_slice(&prefix[prefix.len() - 32768..prefix.len() - 32768 + 258]);
        let mut gzip = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 255];
        gzip.extend_from_slice(&raw.sink);
        gzip.extend_from_slice(&(!crate::crc(!0, &expected)).to_le_bytes());
        gzip.extend_from_slice(&(expected.len() as u32).to_le_bytes());
        let mut oracle = Vec::new();
        flate2::read::MultiGzDecoder::new(gzip.as_slice())
            .read_to_end(&mut oracle)
            .unwrap();
        assert_eq!(oracle, expected);
        for chunk in [113, 65536] {
            let mut reader = crate::GzipReader::new(gzip.as_slice());
            let mut buffer = vec![0; chunk];
            let mut actual = Vec::new();
            loop {
                let n = reader.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                actual.extend_from_slice(&buffer[..n]);
            }
            assert_eq!(actual, expected);
        }
    }
}
