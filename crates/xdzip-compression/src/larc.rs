use crate::{
    bitstream::Bits,
    invalid,
    stream_io::{filled, resource},
};
use std::io::{self, Read};

/// Historical LArc methods; `-lz4-` is a container Store marker, not modern LZ4.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LarcMethod {
    /// MSB-first flags and absolute 11-bit dictionary references.
    Lzs,
    /// Eight LSB-first flags followed by byte-aligned commands.
    Lz5,
}
/// Dictionary plus conservative input, control and stack admission.
pub fn larc_memory_required(method: LarcMethod) -> u64 {
    32 * 1024
        + match method {
            LarcMethod::Lzs => 2048,
            LarcMethod::Lz5 => 4096,
        }
}
/// Exact-span, exact-size LArc LZS/LZ5 decoder without container or CRC authority.
pub struct LarcReader<R, P = fn() -> io::Result<()>> {
    bits: Bits<R, P, true>,
    method: LarcMethod,
    dictionary: Vec<u8>,
    initialized: Vec<bool>,
    position: usize,
    copy_position: usize,
    copy: usize,
    flags: u8,
    flag_count: u8,
    remaining: u64,
    ended: bool,
    failed: bool,
}
impl<R: Read> LarcReader<R> {
    /// Admit the caller-owned source, sizes, memory and actual work before decoding.
    pub fn new(
        source: R,
        method: LarcMethod,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
    ) -> io::Result<Self> {
        Self::with_poll(source, method, packed, decoded, memory, work, || Ok(()))
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> LarcReader<R, P> {
    /// Unconsumed actual-work allowance; repeated container members must share it.
    pub fn work_remaining(&self) -> u64 {
        self.bits.work
    }
    /// Poll bounded actual work, including command parsing and overlapping copies.
    pub fn with_poll(
        source: R,
        method: LarcMethod,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
        poll: P,
    ) -> io::Result<Self> {
        const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
        if packed > MAX_BYTES || decoded > MAX_BYTES || memory < larc_memory_required(method) {
            return Err(resource());
        }
        if decoded == 0 && packed != 0 {
            return Err(invalid());
        }
        let size = if method == LarcMethod::Lzs {
            2048
        } else {
            4096
        };
        let mut bits = Bits::with_poll(source, packed, work.min(96 * 1024 * 1024 * 1024), poll);
        bits.charge(size as u64)?;
        let mut dictionary = filled(size, b' ')?;
        let mut initialized = filled(size, true)?;
        if method == LarcMethod::Lz5 {
            for (at, byte) in dictionary.iter_mut().enumerate() {
                *byte = match at {
                    0..=3327 => (at / 13) as u8,
                    3328..=3583 => (at - 3328) as u8,
                    3584..=3839 => (3839 - at) as u8,
                    3840..=3967 => 0,
                    _ => b' ',
                };
            }
            // Original encoders never use this unwritten tail. Historical
            // decoders disagree on its initial bytes, so it grants no history.
            initialized[size - 18..].fill(false);
        }
        Ok(Self {
            bits,
            method,
            dictionary,
            initialized,
            position: size - if method == LarcMethod::Lzs { 17 } else { 18 },
            copy_position: 0,
            copy: 0,
            flags: 0,
            flag_count: 0,
            remaining: decoded,
            ended: false,
            failed: false,
        })
    }
    fn command(&mut self) -> io::Result<Option<u8>> {
        let literal = if self.method == LarcMethod::Lzs {
            self.bits.get(1)? != 0
        } else {
            if self.flag_count == 0 {
                self.flags = self.bits.get(8)? as u8;
                self.flag_count = 8;
            }
            self.flag_count -= 1;
            let result = self.flags & 1 != 0;
            self.flags >>= 1;
            result
        };
        if literal {
            return Ok(Some(self.bits.get(8)? as u8));
        }
        if self.method == LarcMethod::Lzs {
            self.copy_position = self.bits.get(11)? as usize;
            self.copy = self.bits.get(4)? as usize + 2;
        } else {
            let low = self.bits.get(8)? as usize;
            let high = self.bits.get(8)? as usize;
            self.copy_position = low | ((high & 0xf0) << 4);
            self.copy = (high & 15) + 3;
        }
        if self.copy as u64 > self.remaining {
            return Err(invalid());
        }
        Ok(None)
    }
    fn decode(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let maximum = output.len().min(8192);
        let mut at = 0;
        while at < maximum && !self.ended {
            if self.remaining == 0 {
                if self.copy != 0 {
                    return Err(invalid());
                }
                self.bits.finish_ignored_padding()?;
                self.ended = true;
                break;
            }
            let byte = if self.copy != 0 {
                if !self.initialized[self.copy_position] {
                    return Err(invalid());
                }
                let byte = self.dictionary[self.copy_position];
                self.copy_position = (self.copy_position + 1) & (self.dictionary.len() - 1);
                self.copy -= 1;
                byte
            } else if let Some(byte) = self.command()? {
                byte
            } else {
                continue;
            };
            self.bits.charge(1)?;
            self.dictionary[self.position] = byte;
            self.initialized[self.position] = true;
            self.position = (self.position + 1) & (self.dictionary.len() - 1);
            self.remaining -= 1;
            output[at] = byte;
            at += 1;
        }
        Ok(at)
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for LarcReader<R, P> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        match self.decode(output) {
            Ok(n) => Ok(n),
            Err(error) => {
                self.failed = true;
                Err(error)
            }
        }
    }
}
#[cfg(test)]
mod tests;
