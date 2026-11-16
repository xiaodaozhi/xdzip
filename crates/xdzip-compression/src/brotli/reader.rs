use super::{bits::Bits, dictionary, meta::Meta};
use crate::{
    invalid,
    stream_io::{filled, preserve_io_error, resource},
};
use std::io::{self, Read};
const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_WORK: u64 = 96 * 1024 * 1024 * 1024;
/// Window plus a conservative bound for all prefix symbols, contexts, input and fixed state.
/// Every declared tree/map is itself bounded before allocation; static wire data grants no I/O.
pub fn brotli_memory_required(window_bits: u8) -> Option<u64> {
    (10..=24)
        .contains(&window_bits)
        .then(|| (1u64 << window_bits) + 2 * 1024 * 1024)
}
enum Stage {
    Command,
    Literals,
    Resolve,
    Copy,
    Dictionary,
}
/// Strict single RFC 7932 stream with bounded history, prefix storage and cumulative work.
/// No checksum or decoded-size field exists; the caller must drain to EOF and retain staging.
pub struct BrotliReader<R, P = fn() -> io::Result<()>> {
    bits: Bits<R, P>,
    history: Vec<u8>,
    position: usize,
    total: u64,
    decoded: u64,
    meta: Option<Meta>,
    remaining: usize,
    last: bool,
    uncompressed: bool,
    stage: Stage,
    insert: usize,
    copy: usize,
    implicit: bool,
    distance: usize,
    cache: [u32; 4],
    word: [u8; 37],
    word_at: usize,
    word_size: usize,
    ended: bool,
    failed: bool,
}
impl<R: Read> BrotliReader<R> {
    /// Admit standard 10–24-bit windows before allocating history. No shared dictionary is accepted.
    pub fn new(
        source: R,
        packed: u64,
        memory_limit: u64,
        work_limit: u64,
        decoded_limit: u64,
    ) -> io::Result<Self> {
        Self::with_poll(
            source,
            packed,
            memory_limit,
            work_limit,
            decoded_limit,
            || Ok(()),
        )
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> BrotliReader<R, P> {
    /// Poll format-independent cooperative control every bounded work quantum, including zero-output loops.
    /// The callback receives no source or file authority. Its error poisons the reader.
    pub fn with_poll(
        source: R,
        packed: u64,
        memory_limit: u64,
        work_limit: u64,
        decoded_limit: u64,
        poll: P,
    ) -> io::Result<Self> {
        if packed > MAX_BYTES {
            return Err(resource());
        }
        let mut bits = Bits::with_poll(source, packed, work_limit.min(MAX_WORK), poll);
        let window = if bits.get(1)? == 0 {
            16
        } else {
            let n = bits.get(3)? as u8;
            if n != 0 {
                17 + n
            } else {
                match bits.get(3)? as u8 {
                    0 => 17,
                    1 => {
                        return Err(io::Error::new(
                            io::ErrorKind::Unsupported,
                            "unsupported_shared_brotli",
                        ));
                    }
                    n => 8 + n,
                }
            }
        };
        let memory = brotli_memory_required(window).ok_or_else(invalid)?;
        if memory > memory_limit {
            return Err(resource());
        }
        let history = filled(1usize << window, 0u8)?;
        Ok(Self {
            bits,
            history,
            position: 0,
            total: 0,
            decoded: decoded_limit.min(MAX_BYTES),
            meta: None,
            remaining: 0,
            last: false,
            uncompressed: false,
            stage: Stage::Command,
            insert: 0,
            copy: 0,
            implicit: false,
            distance: 0,
            cache: [4, 11, 15, 16],
            word: [0; 37],
            word_at: 0,
            word_size: 0,
            ended: false,
            failed: false,
        })
    }
    fn end_block(&mut self) -> io::Result<()> {
        if let Some(meta) = self.meta.take() {
            meta.complete()?;
        }
        if self.last {
            self.bits.finish()?;
            self.ended = true;
        }
        Ok(())
    }
    fn header(&mut self) -> io::Result<()> {
        self.last = self.bits.get(1)? != 0;
        if self.last && self.bits.get(1)? != 0 {
            self.bits.finish()?;
            self.ended = true;
            return Ok(());
        }
        let n = self.bits.get(2)? as u8;
        if n == 3 {
            if self.bits.get(1)? != 0 {
                return Err(invalid());
            }
            let width = self.bits.get(2)? as u8 * 8;
            let value = self.bits.get(width)?;
            if width > 8 && value >> (width - 8) == 0 {
                return Err(invalid());
            }
            let size = if width == 0 { 0 } else { value + 1 };
            self.bits.align_zero()?;
            for _ in 0..size {
                self.bits.get(8)?;
            }
            self.end_block()?;
            return Ok(());
        }
        let width = (n + 4) * 4;
        let size = self.bits.get(width)?;
        if width > 16 && size >> (width - 4) == 0 {
            return Err(invalid());
        }
        self.remaining = size as usize + 1;
        if self.remaining as u64 > self.decoded {
            return Err(resource());
        }
        self.uncompressed = !self.last && self.bits.get(1)? != 0;
        if self.uncompressed {
            self.bits.align_zero()?;
            self.meta = None;
        } else {
            self.meta = Some(Meta::read(&mut self.bits)?);
        }
        self.stage = Stage::Command;
        Ok(())
    }
    fn emit(&mut self, byte: u8, output: &mut [u8], at: &mut usize) -> io::Result<()> {
        self.bits.charge(1)?;
        self.decoded = self.decoded.checked_sub(1).ok_or_else(resource)?;
        self.remaining = self.remaining.checked_sub(1).ok_or_else(invalid)?;
        self.total = self.total.checked_add(1).ok_or_else(resource)?;
        self.history[self.position] = byte;
        self.position = (self.position + 1) & (self.history.len() - 1);
        output[*at] = byte;
        *at += 1;
        Ok(())
    }
    fn decode(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let maximum = output.len().min(8192);
        let mut at = 0;
        while at < maximum && !self.ended {
            if self.remaining == 0 {
                if self.meta.is_some() || self.uncompressed {
                    self.end_block()?;
                    self.uncompressed = false;
                    if self.ended {
                        break;
                    }
                }
                self.header()?;
                continue;
            }
            if self.uncompressed {
                let byte = self.bits.get(8)? as u8;
                self.emit(byte, output, &mut at)?;
                continue;
            }
            match self.stage {
                Stage::Command => {
                    let (insert, copy, implicit) = self
                        .meta
                        .as_mut()
                        .ok_or_else(invalid)?
                        .command(&mut self.bits)?;
                    if insert > self.remaining {
                        return Err(invalid());
                    }
                    self.insert = insert;
                    self.copy = copy;
                    self.implicit = implicit;
                    self.stage = Stage::Literals;
                }
                Stage::Literals => {
                    if self.insert == 0 {
                        self.stage = Stage::Resolve;
                        continue;
                    }
                    let last = self.history
                        [(self.position + self.history.len() - 1) & (self.history.len() - 1)];
                    let before = self.history
                        [(self.position + self.history.len() - 2) & (self.history.len() - 1)];
                    let byte = self.meta.as_mut().ok_or_else(invalid)?.literal(
                        &mut self.bits,
                        last,
                        before,
                    )?;
                    self.emit(byte, output, &mut at)?;
                    self.insert -= 1;
                }
                Stage::Resolve => {
                    let (symbol, distance) = self.meta.as_mut().ok_or_else(invalid)?.distance(
                        &mut self.bits,
                        self.copy,
                        self.implicit,
                        &self.cache,
                    )?;
                    let maximum = (self.history.len() as u64 - 16).min(self.total);
                    if u64::from(distance) <= maximum {
                        if self.copy > self.remaining {
                            return Err(invalid());
                        }
                        self.distance = distance as usize;
                        if symbol != 0 {
                            self.cache.rotate_right(1);
                            self.cache[0] = distance;
                        }
                        self.stage = Stage::Copy;
                    } else {
                        let (word, size) = dictionary::word(
                            self.copy,
                            (u64::from(distance) - maximum - 1) as usize,
                        )?;
                        if size > self.remaining {
                            return Err(invalid());
                        }
                        self.word = word;
                        self.word_at = 0;
                        self.word_size = size;
                        self.stage = Stage::Dictionary;
                    }
                }
                Stage::Copy => {
                    if self.copy == 0 {
                        self.stage = Stage::Command;
                        continue;
                    }
                    let from = (self.position + self.history.len() - self.distance)
                        & (self.history.len() - 1);
                    let byte = self.history[from];
                    self.emit(byte, output, &mut at)?;
                    self.copy -= 1;
                }
                Stage::Dictionary => {
                    if self.word_at == self.word_size {
                        self.stage = Stage::Command;
                        continue;
                    }
                    let byte = self.word[self.word_at];
                    self.emit(byte, output, &mut at)?;
                    self.word_at += 1;
                }
            }
        }
        Ok(at)
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for BrotliReader<R, P> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        match self.decode(output) {
            Ok(count) => Ok(count),
            Err(error) => {
                self.failed = true;
                Err(preserve_io_error(error))
            }
        }
    }
}
