//! Independent LH5 writer with bounded matching and complete canonical wire trees.
use crate::{
    invalid,
    match_finder::{MatchFinder, search_depth},
    stream_io::{preserve_io_error, resource, retryable_io},
};
use std::io::{self, Write};
const CHUNK: usize = 32768;
const HISTORY: usize = 8192;
const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_WORK: u64 = 96 * 1024 * 1024 * 1024;
/// Admission for hash chains, retained history/input, bounded commands and output buffering.
pub fn lzh_writer_memory_required(level: u8) -> io::Result<u64> {
    search_depth(level)?;
    Ok(1024 * 1024)
}
struct Work<P> {
    remaining: u64,
    until_poll: u64,
    poll: P,
}
impl<P: FnMut() -> io::Result<()>> Work<P> {
    fn charge(&mut self, mut amount: u64) -> io::Result<()> {
        self.remaining = self.remaining.checked_sub(amount).ok_or_else(resource)?;
        while amount >= self.until_poll {
            amount -= self.until_poll;
            (self.poll)().map_err(preserve_io_error)?;
            self.until_poll = 8192;
        }
        self.until_poll -= amount;
        Ok(())
    }
}
struct Output<W> {
    sink: W,
    buffer: Vec<u8>,
    value: u8,
    count: u8,
    written: u64,
}
impl<W: Write> Output<W> {
    fn flush_bytes<P: FnMut() -> io::Result<()>>(&mut self, work: &mut Work<P>) -> io::Result<()> {
        let mut offset = 0;
        while offset < self.buffer.len() {
            work.charge(1)?;
            let left = self.buffer.len() - offset;
            if self
                .written
                .checked_add(left as u64)
                .is_none_or(|n| n > MAX_BYTES)
            {
                return Err(resource());
            }
            match self.sink.write(&self.buffer[offset..]) {
                Err(e) if retryable_io(&e) => continue,
                Err(e) => return Err(preserve_io_error(e)),
                Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
                Ok(n) if n <= left => {
                    self.written += n as u64;
                    offset += n;
                }
                Ok(_) => return Err(invalid()),
            }
        }
        self.buffer.clear();
        Ok(())
    }
    fn put<P: FnMut() -> io::Result<()>>(
        &mut self,
        value: u16,
        count: u8,
        work: &mut Work<P>,
    ) -> io::Result<()> {
        work.charge(u64::from(count))?;
        for bit in (0..count).rev() {
            self.value = (self.value << 1) | ((value >> bit) & 1) as u8;
            self.count += 1;
            if self.count == 8 {
                self.buffer.push(self.value);
                self.value = 0;
                self.count = 0;
                if self.buffer.len() == 8192 {
                    self.flush_bytes(work)?;
                }
            }
        }
        Ok(())
    }
}
#[derive(Clone, Copy)]
struct Token {
    symbol: u16,
    distance: u16,
}
/// Raw LH5 streaming encoder. The container supplies lengths, CRC and publication policy.
/// Levels 1/6/9 search 1/16/64 prior candidates; matches span 4–256 bytes.
pub struct LzhWriter<W, P = fn() -> io::Result<()>> {
    output: Output<W>,
    input: Vec<u8>,
    tokens: Vec<Token>,
    history: usize,
    depth: usize,
    total: u64,
    work: Work<P>,
    failed: bool,
}
impl<W: Write> LzhWriter<W> {
    /// Create a bounded raw LH5 writer without an external cancellation callback.
    pub fn new(sink: W, level: u8, memory_limit: u64, work_limit: u64) -> io::Result<Self> {
        Self::with_poll(sink, level, memory_limit, work_limit, || Ok(()))
    }
}
impl<W: Write, P: FnMut() -> io::Result<()>> LzhWriter<W, P> {
    /// Admit memory/work before allocation; all algorithm work shares the supplied ledger.
    pub fn with_poll(
        sink: W,
        level: u8,
        memory_limit: u64,
        work_limit: u64,
        mut poll: P,
    ) -> io::Result<Self> {
        if memory_limit < lzh_writer_memory_required(level)? {
            return Err(resource());
        }
        poll().map_err(preserve_io_error)?;
        let mut input = Vec::new();
        input
            .try_reserve_exact(HISTORY + CHUNK)
            .map_err(|_| resource())?;
        let mut tokens = Vec::new();
        tokens.try_reserve_exact(CHUNK).map_err(|_| resource())?;
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(8192).map_err(|_| resource())?;
        Ok(Self {
            output: Output {
                sink,
                buffer,
                value: 0,
                count: 0,
                written: 0,
            },
            input,
            tokens,
            history: 0,
            depth: search_depth(level)?,
            total: 0,
            work: Work {
                remaining: work_limit.min(MAX_WORK),
                until_poll: 8192,
                poll,
            },
            failed: false,
        })
    }
    /// Remaining algorithm work before finalization.
    pub fn work_remaining(&self) -> u64 {
        self.work.remaining
    }
    fn block(&mut self) -> io::Result<()> {
        if self.input.len() == self.history {
            return Ok(());
        }
        let mut charge = |amount| self.work.charge(amount);
        let mut finder = MatchFinder::with_work(self.depth, &mut charge)?;
        finder.seed_with_work(&self.input, 0, self.history, &mut charge)?;
        self.tokens.clear();
        let mut at = self.history;
        while at < self.input.len() {
            charge(1)?;
            let (distance, length) = if at + 4 <= self.input.len() {
                finder.find_bounded(
                    &self.input,
                    at,
                    self.input.len().min(at + 256),
                    HISTORY,
                    &mut charge,
                )?
            } else {
                (0, 0)
            };
            if length >= 4 {
                self.tokens.push(Token {
                    symbol: (length + 253) as u16,
                    distance: distance as u16,
                });
                finder.seed_with_work(&self.input, at + 1, at + length, &mut charge)?;
                at += length;
            } else {
                self.tokens.push(Token {
                    symbol: u16::from(self.input[at]),
                    distance: 0,
                });
                at += 1;
            }
        }
        // Complete C tree: two 8-bit and 508 9-bit codes. Its length alphabet
        // consists only of 8/9 (wire symbols 10/11), each with one prefix bit.
        let output = &mut self.output;
        let work = &mut self.work;
        output.put(self.tokens.len() as u16, 16, work)?;
        output.put(12, 5, work)?;
        for i in 0..12 {
            output.put(u16::from(i >= 10), 3, work)?;
            if i == 2 {
                output.put(0, 2, work)?;
            }
        }
        output.put(510, 9, work)?;
        for symbol in 0..510 {
            output.put(u16::from(symbol >= 2), 1, work)?;
        }
        // Complete position tree: two 3-bit and twelve 4-bit codes.
        output.put(14, 4, work)?;
        for position in 0..14 {
            output.put(if position < 2 { 3 } else { 4 }, 3, work)?;
        }
        for token in &self.tokens {
            output.put(
                if token.symbol < 2 {
                    token.symbol
                } else {
                    token.symbol + 2
                },
                if token.symbol < 2 { 8 } else { 9 },
                work,
            )?;
            if token.distance != 0 {
                let offset = token.distance - 1;
                let position = (u16::BITS - offset.leading_zeros()) as u16;
                output.put(
                    if position < 2 { position } else { position + 2 },
                    if position < 2 { 3 } else { 4 },
                    work,
                )?;
                if position > 1 {
                    output.put(offset - (1 << (position - 1)), (position - 1) as u8, work)?;
                }
            }
        }
        let keep = self.input.len().min(HISTORY);
        work.charge(keep as u64)?;
        let start = self.input.len() - keep;
        self.input.copy_within(start.., 0);
        self.input.truncate(keep);
        self.history = keep;
        Ok(())
    }
    /// Finish the exact last byte with zero fill and return the owned host sink.
    /// Empty input produces zero packed bytes; no standalone empty block is emitted.
    pub fn finish(self) -> io::Result<W> {
        self.finish_with_work().map(|(sink, _)| sink)
    }
    /// Finalize and retain the remaining cumulative work for a subsequent member.
    pub fn finish_with_work(mut self) -> io::Result<(W, u64)> {
        if self.failed {
            return Err(invalid());
        }
        (self.work.poll)().map_err(preserve_io_error)?;
        self.block()?;
        if self.output.count != 0 {
            self.output.put(0, 8 - self.output.count, &mut self.work)?;
        }
        self.output.flush_bytes(&mut self.work)?;
        Ok((self.output.sink, self.work.remaining))
    }
}
impl<W: Write, P: FnMut() -> io::Result<()>> Write for LzhWriter<W, P> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failed {
            return Err(invalid());
        }
        let result = (|| {
            (self.work.poll)().map_err(preserve_io_error)?;
            if bytes.is_empty() {
                return Ok(0);
            }
            if self.input.len() == self.history + CHUNK {
                self.block()?;
            }
            let take = bytes
                .len()
                .min(8192)
                .min(CHUNK + self.history - self.input.len());
            let total = self
                .total
                .checked_add(take as u64)
                .filter(|n| *n <= MAX_BYTES)
                .ok_or_else(resource)?;
            self.work.charge(take as u64)?;
            self.input.extend_from_slice(&bytes[..take]);
            self.total = total;
            Ok(take)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        let result = (|| {
            (self.work.poll)().map_err(preserve_io_error)?;
            self.block()?;
            self.output.flush_bytes(&mut self.work)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
#[cfg(test)]
mod tests;
