//! Independently expressed PMarc PM2 wire decoding over shared bounded prefix primitives.
use crate::{
    bitstream::Bits,
    entropy::Prefix,
    invalid,
    stream_io::{filled, preserve_io_error, resource},
};
use std::io::{self, Read};
const DICTIONARY: usize = 8192;
const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_WORK: u64 = 96 * 1024 * 1024 * 1024;
/// History, buffered input, recency ordering, prefix tables and bounded scratch.
pub fn pm2_memory_required() -> u64 {
    DICTIONARY as u64 + 32 * 1024
}
/// Raw PM2 reader; the container supplies exact packed/decoded spans and validates CRC.
pub struct Pm2Reader<R, P = fn() -> io::Result<()>> {
    bits: Bits<R, P, true>,
    history: Vec<u8>,
    recent: [u8; 256],
    codes: Option<Prefix<29>>,
    positions: Option<Prefix<9>>,
    need_positions: bool,
    position: usize,
    copying: usize,
    distance: usize,
    produced: u64,
    expected: u64,
    next_boundary: u64,
    stage: u8,
    started: bool,
    finished: bool,
    failure: Option<io::ErrorKind>,
}
impl<R: Read> Pm2Reader<R> {
    /// Open one admitted PM2 span with hard private resource ceilings.
    pub fn new(source: R, packed: u64, decoded: u64, memory: u64, work: u64) -> io::Result<Self> {
        Self::with_poll(source, packed, decoded, memory, work, || Ok(()))
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Pm2Reader<R, P> {
    /// Poll during source-independent copies, recency updates and tree construction as well.
    pub fn with_poll(
        source: R,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
        mut poll: P,
    ) -> io::Result<Self> {
        if packed > MAX_BYTES || decoded > MAX_BYTES || memory < pm2_memory_required() {
            return Err(resource());
        }
        poll().map_err(preserve_io_error)?;
        let mut bits = Bits::with_poll(source, packed, work.min(MAX_WORK), poll);
        bits.charge((DICTIONARY + 256) as u64)?;
        let history = filled(DICTIONARY, b' ')?;
        let mut recent = [0; 256];
        let mut at = 0;
        for range in [
            0x20..=0x7f,
            0x00..=0x1f,
            0xa0..=0xdf,
            0x80..=0x9f,
            0xe0..=0xff,
        ] {
            for byte in range {
                recent[at] = byte;
                at += 1;
            }
        }
        Ok(Self {
            bits,
            history,
            recent,
            codes: None,
            positions: None,
            need_positions: false,
            position: 0,
            copying: 0,
            distance: 0,
            produced: 0,
            expected: decoded,
            next_boundary: 1024,
            stage: 0,
            started: false,
            finished: false,
            failure: None,
        })
    }
    /// Retain actual algorithm work across members in a container-owned cumulative ledger.
    pub fn work_remaining(&self) -> u64 {
        self.bits.work
    }
    fn code_tree(&mut self) -> io::Result<()> {
        self.bits.charge(29)?;
        let count = self.bits.get(5)? as usize;
        let minimum = self.bits.get(3)? as u8;
        if !(1..=29).contains(&count) {
            return Err(invalid());
        }
        self.need_positions = count >= 10 && !(count == 29 && minimum == 0);
        self.codes = Some(if minimum == 0 {
            Prefix::single((count - 1) as u16)
        } else {
            let width = self.bits.get(3)? as u8;
            let mut lengths = [0; 29];
            for length in &mut lengths[..count] {
                let value = self.bits.get(width)? as u8;
                *length = if value == 0 {
                    0
                } else {
                    minimum.checked_add(value - 1).ok_or_else(invalid)?
                };
            }
            self.bits.charge(count as u64)?;
            Prefix::from_lengths(&lengths[..count], 28, false)?
        });
        Ok(())
    }
    fn position_tree(&mut self, count: usize) -> io::Result<()> {
        if !self.need_positions {
            return Ok(());
        }
        self.bits.charge(8)?;
        let mut lengths = [0; 8];
        let mut single = None;
        let mut used = 0;
        for (symbol, length) in lengths[..count].iter_mut().enumerate() {
            *length = self.bits.get(3)? as u8;
            if *length != 0 {
                used += 1;
                single = Some(symbol as u16);
            }
        }
        self.bits.charge(count as u64)?;
        self.positions = match used {
            0 => None,
            1 => Some(Prefix::single(single.ok_or_else(invalid)?)),
            _ => Some(Prefix::from_lengths(&lengths[..count], 7, false)?),
        };
        Ok(())
    }
    fn scheduled_update(&mut self) -> io::Result<()> {
        match self.stage {
            0 => {
                self.position_tree(6)?;
                self.next_boundary += 1024;
                self.stage = 1;
            }
            1 => {
                self.position_tree(7)?;
                self.next_boundary += 2048;
                self.stage = 2;
            }
            2 => {
                if self.bits.get(1)? != 0 {
                    self.code_tree()?;
                }
                self.position_tree(8)?;
                self.next_boundary += 4096;
                self.stage = 3;
            }
            _ => {
                if self.bits.get(1)? != 0 {
                    self.code_tree()?;
                    self.position_tree(8)?;
                }
                self.next_boundary += 4096;
            }
        }
        Ok(())
    }
    fn command(&mut self) -> io::Result<Option<u8>> {
        let code = self
            .codes
            .as_ref()
            .ok_or_else(invalid)?
            .symbol(|| Ok(self.bits.get(1)? as u8))? as usize;
        self.bits.charge(1)?;
        if code < 8 {
            let (base, width) = [
                (0, 3),
                (8, 3),
                (16, 4),
                (32, 5),
                (64, 5),
                (96, 5),
                (128, 6),
                (192, 6),
            ][code];
            let rank = base + self.bits.get(width)? as usize;
            return self.recent.get(rank).copied().map(Some).ok_or_else(invalid);
        }
        let class = code - 8;
        let length = if class < 15 {
            class + 2
        } else {
            let (base, width) = *[(17, 3), (25, 3), (33, 5), (65, 6), (129, 7), (256, 0)]
                .get(class - 15)
                .ok_or_else(invalid)?;
            base + self.bits.get(width)? as usize
        };
        let offset = if class >= 20 {
            0
        } else if class == 0 {
            self.bits.get(6)? as usize
        } else {
            let position = self
                .positions
                .as_ref()
                .ok_or_else(invalid)?
                .symbol(|| Ok(self.bits.get(1)? as u8))? as u8;
            self.bits.charge(1)?;
            if position == 0 {
                self.bits.get(6)? as usize
            } else if position <= 7 {
                let width = position + 5;
                (1usize << width) + self.bits.get(width)? as usize
            } else {
                return Err(invalid());
            }
        };
        if length > 256 || length as u64 > self.expected - self.produced || offset >= DICTIONARY {
            return Err(invalid());
        }
        self.copying = length;
        self.distance = offset + 1;
        Ok(None)
    }
    fn observe(&mut self, byte: u8) -> io::Result<()> {
        let mut rank = 0;
        while rank < self.recent.len() {
            self.bits.charge(1)?;
            if self.recent[rank] == byte {
                break;
            }
            rank += 1;
        }
        if rank == self.recent.len() {
            return Err(invalid());
        }
        for index in (1..=rank).rev() {
            self.bits.charge(1)?;
            self.recent[index] = self.recent[index - 1];
        }
        self.recent[0] = byte;
        Ok(())
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.finished {
            return Ok(0);
        }
        if !self.started && self.expected != 0 {
            self.bits.get(1)?;
            self.code_tree()?;
            self.position_tree(5)?;
            self.started = true;
        }
        let mut count = 0;
        while count < output.len().min(8192) && self.produced < self.expected {
            let byte = if self.copying != 0 {
                self.bits.charge(1)?;
                self.copying -= 1;
                self.history[(self.position + DICTIONARY - self.distance) % DICTIONARY]
            } else if let Some(byte) = self.command()? {
                byte
            } else {
                continue;
            };
            self.bits.charge(1)?;
            self.observe(byte)?;
            self.history[self.position] = byte;
            self.position = (self.position + 1) % DICTIONARY;
            output[count] = byte;
            count += 1;
            self.produced += 1;
            if self.produced == self.next_boundary {
                self.scheduled_update()?;
            }
        }
        if self.produced == self.expected {
            if self.copying != 0 {
                return Err(invalid());
            }
            self.bits.finish_ignored_padding()?;
            self.finished = true;
        }
        Ok(count)
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for Pm2Reader<R, P> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if let Some(kind) = self.failure {
            return Err(io::Error::new(kind, "terminal_pm2_error"));
        }
        if output.is_empty() {
            return Ok(0);
        }
        match self.read_inner(output) {
            Ok(n) => Ok(n),
            Err(error) => {
                let error = preserve_io_error(error);
                self.failure = Some(error.kind());
                Err(error)
            }
        }
    }
}
#[cfg(test)]
mod tests;
