//! Bounded BZip2 block writer. Uses cyclic rank doubling and two canonical
//! tables with identical lengths. Wire fields mirror the independent reader.
use crate::bzip2::{BLOCK_MAGIC, END_MAGIC, crc_byte};
use std::io::{self, BufWriter, Write};

const BLOCK: usize = 64 * 1024;

/// A streaming BZip2 encoder with bounded input blocks.
pub struct Bzip2Writer<W: Write> {
    out: BufWriter<W>,
    pending: u64,
    bits: u8,
    input: Vec<u8>,
    combined_crc: u32,
}
impl<W: Write> Bzip2Writer<W> {
    /// Start a BZh1 stream with bounded 64 KiB input blocks.
    pub fn new(mut out: W) -> io::Result<Self> {
        out.write_all(b"BZh1")?;
        Ok(Self {
            out: BufWriter::new(out),
            pending: 0,
            bits: 0,
            input: Vec::with_capacity(BLOCK),
            combined_crc: 0,
        })
    }

    fn put(&mut self, value: u64, width: u8) -> io::Result<()> {
        self.pending = (self.pending << width) | value;
        self.bits += width;
        while self.bits >= 8 {
            self.bits -= 8;
            self.out.write_all(&[(self.pending >> self.bits) as u8])?;
        }
        self.pending &= (1u64 << self.bits) - 1;
        Ok(())
    }

    fn block(&mut self) -> io::Result<()> {
        if self.input.is_empty() {
            return Ok(());
        }
        let checksum = !self.input.iter().copied().fold(!0, crc_byte);
        self.combined_crc = self.combined_crc.rotate_left(1) ^ checksum;
        let encoded = encode_runs(&self.input);
        let (origin, last) = cyclic_transform(&encoded);
        let mut used = [0u16; 16];
        for &b in &last {
            used[usize::from(b >> 4)] |= 0x8000 >> (b & 15);
        }
        let mut alphabet = (0u8..=255)
            .filter(|&b| used[usize::from(b >> 4)] & (0x8000 >> (b & 15)) != 0)
            .collect::<Vec<_>>();
        let end = alphabet.len() + 1;
        let symbols = move_to_front(&last, &mut alphabet, end);
        self.put(BLOCK_MAGIC, 48)?;
        self.put(u64::from(checksum), 32)?;
        self.put(0, 1)?; // Ordinary non-randomized blocks.
        self.put(origin as u64, 24)?;
        for &group in &used {
            self.put(u64::from(group != 0), 1)?;
        }
        for &group in used.iter().filter(|&&group| group != 0) {
            self.put(u64::from(group), 16)?;
        }
        self.put(2, 3)?;
        let groups = symbols.len().div_ceil(50);
        self.put(groups as u64, 15)?;
        for _ in 0..groups {
            self.put(0, 1)?; // Select the first table for each group.
        }
        let alphabet_size = end + 1;
        let width = (alphabet_size - 1).ilog2() as u8 + 1;
        let short = (1 << width) - alphabet_size;
        for _ in 0..2 {
            self.put(u64::from(width - u8::from(short != 0)), 5)?;
            for symbol in 0..alphabet_size {
                if short != 0 && symbol == short {
                    self.put(0b10, 2)?; // Increase the length by one.
                }
                self.put(0, 1)?; // Accept the current length.
            }
        }
        for symbol in symbols {
            if symbol < short {
                self.put(symbol as u64, width - 1)?;
            } else {
                self.put((symbol + short) as u64, width)?;
            }
        }
        self.input.clear();
        Ok(())
    }

    /// Finalize CRCs and the stream terminator explicitly.
    pub fn finish(mut self) -> io::Result<W> {
        self.block()?;
        self.put(END_MAGIC, 48)?;
        self.put(u64::from(self.combined_crc), 32)?;
        if self.bits != 0 {
            self.put(0, 8 - self.bits)?;
        }
        self.out.into_inner().map_err(|error| error.into_error())
    }
}

impl<W: Write> Write for Bzip2Writer<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let take = bytes.len().min(BLOCK - self.input.len());
        self.input.extend_from_slice(&bytes[..take]);
        if self.input.len() == BLOCK {
            self.block()?;
        }
        Ok(take)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.out.flush()
    }
}

fn encode_runs(input: &[u8]) -> Vec<u8> {
    let mut result = Vec::with_capacity(input.len() + input.len() / 4);
    let mut at = 0;
    while at < input.len() {
        let byte = input[at];
        let count = input[at..]
            .iter()
            .take(255)
            .take_while(|&&b| b == byte)
            .count();
        result.extend(std::iter::repeat_n(byte, count.min(4)));
        if count >= 4 {
            result.push((count - 4) as u8);
        }
        at += count;
    }
    result
}

fn cyclic_transform(input: &[u8]) -> (usize, Vec<u8>) {
    let len = input.len();
    let mut order: Vec<_> = (0..len).collect();
    let mut ranks: Vec<_> = input.iter().map(|&b| usize::from(b)).collect();
    let mut next = vec![0; len];
    let mut stride = 1;
    loop {
        let key = |at: usize| (ranks[at], ranks[(at + stride) % len]);
        order.sort_unstable_by_key(|&at| key(at));
        let mut rank = 0;
        next[order[0]] = rank;
        for pair in order.windows(2) {
            rank += usize::from(key(pair[0]) != key(pair[1]));
            next[pair[1]] = rank;
        }
        if rank + 1 == len || stride * 2 >= len {
            break;
        }
        std::mem::swap(&mut ranks, &mut next);
        stride *= 2;
    }
    let original = order
        .iter()
        .position(|&at| at == 0)
        .expect("rotation permutation");
    let last = order
        .into_iter()
        .map(|at| input[(at + len - 1) % len])
        .collect();
    (original, last)
}

fn move_to_front(last: &[u8], alphabet: &mut [u8], end: usize) -> Vec<usize> {
    fn run(output: &mut Vec<usize>, mut count: usize) {
        while count != 0 {
            count -= 1;
            output.push(count & 1);
            count >>= 1;
        }
    }
    let mut output = Vec::with_capacity(last.len() + 1);
    let mut count = 0;
    for &byte in last {
        let at = alphabet
            .iter()
            .position(|&b| b == byte)
            .expect("block alphabet");
        if at == 0 {
            count += 1;
        } else {
            run(&mut output, count);
            count = 0;
            output.push(at + 1);
            alphabet[..=at].rotate_right(1);
        }
    }
    run(&mut output, count);
    output.push(end);
    output
}
