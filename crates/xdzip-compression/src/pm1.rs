//! PMarc PM1 decoding with bounded explicit prefixes, ranks and output-position profiles.
use crate::{
    bitstream::Bits,
    invalid, pmarc,
    stream_io::{filled, preserve_io_error, resource},
};
use std::io::{self, Read};
const WINDOW: usize = 16384;
const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_WORK: u64 = 96 * 1024 * 1024 * 1024;
// A leading one marks prefix width. Slots 6/7 are the second d/e codes of selector 17.
// These are wire codewords, independently expressed without reference tree traversal.
const LITERAL_CODES: [[u8; 8]; 32] = [
    [16, 17, 9, 5, 6, 7, 0, 0],
    [8, 9, 10, 6, 7, 11, 0, 0],
    [8, 9, 5, 6, 14, 15, 0, 0],
    [4, 10, 11, 6, 14, 15, 0, 0],
    [4, 10, 6, 11, 14, 15, 0, 0],
    [4, 10, 6, 7, 22, 23, 0, 0],
    [4, 5, 12, 13, 14, 15, 0, 0],
    [4, 5, 12, 7, 26, 27, 0, 0],
    [4, 5, 6, 14, 30, 31, 0, 0],
    [2, 24, 13, 14, 15, 25, 0, 0],
    [2, 24, 13, 7, 50, 51, 0, 0],
    [2, 24, 25, 13, 14, 15, 0, 0],
    [2, 12, 26, 14, 15, 27, 0, 0],
    [2, 12, 13, 14, 30, 31, 0, 0],
    [2, 12, 7, 26, 54, 55, 0, 0],
    [2, 6, 28, 29, 30, 31, 0, 0],
    [2, 6, 14, 30, 62, 63, 0, 0],
    [0, 0, 5, 8, 9, 0, 6, 7],
    [4, 10, 6, 7, 11, 0, 0, 0],
    [4, 5, 6, 14, 15, 0, 0, 0],
    [2, 24, 13, 7, 25, 0, 0, 0],
    [2, 12, 13, 14, 15, 0, 0, 0],
    [2, 12, 7, 26, 27, 0, 0, 0],
    [2, 6, 14, 30, 31, 0, 0, 0],
    [8, 9, 5, 3, 0, 0, 0, 0],
    [4, 10, 3, 11, 0, 0, 0, 0],
    [4, 5, 6, 7, 0, 0, 0, 0],
    [2, 12, 7, 13, 0, 0, 0, 0],
    [2, 6, 14, 15, 0, 0, 0, 0],
    [2, 6, 7, 0, 0, 0, 0, 0],
    [2, 3, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 0],
];
/// History, buffered input, byte ranks, fixed prefixes and bounded scratch.
pub fn pm1_memory_required() -> u64 {
    WINDOW as u64 + 32 * 1024
}
/// One exact PM1 span. The container must validate decoded size and its declared CRC.
pub struct Pm1Reader<R, P = fn() -> io::Result<()>> {
    bits: Bits<R, P, true>,
    history: Vec<u8>,
    recent: [u8; 256],
    selector: Option<usize>,
    literals: usize,
    copying: usize,
    distance: usize,
    implicit_copy: bool,
    position: usize,
    produced: u64,
    expected: u64,
    finished: bool,
    failure: Option<io::ErrorKind>,
}
impl<R: Read> Pm1Reader<R> {
    /// Admit private hard ceilings as well as smaller caller limits.
    pub fn new(source: R, packed: u64, decoded: u64, memory: u64, work: u64) -> io::Result<Self> {
        Self::with_poll(source, packed, decoded, memory, work, || Ok(()))
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Pm1Reader<R, P> {
    /// Poll during source-independent history copies and byte-rank movements.
    pub fn with_poll(
        source: R,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
        mut poll: P,
    ) -> io::Result<Self> {
        if packed > MAX_BYTES || decoded > MAX_BYTES || memory < pm1_memory_required() {
            return Err(resource());
        }
        poll().map_err(preserve_io_error)?;
        let mut bits = Bits::with_poll(source, packed, work.min(MAX_WORK), poll);
        bits.charge((WINDOW + 256) as u64)?;
        let history = filled(WINDOW, 0)?;
        Ok(Self {
            bits,
            history,
            recent: pmarc::initial_order(),
            selector: None,
            literals: 0,
            copying: 0,
            distance: 0,
            implicit_copy: false,
            position: 0,
            produced: 0,
            expected: decoded,
            finished: false,
            failure: None,
        })
    }
    /// Preserve actual work across physical members in the caller's cumulative ledger.
    pub fn work_remaining(&self) -> u64 {
        self.bits.work
    }
    fn literal(&mut self) -> io::Result<u8> {
        let codes = &LITERAL_CODES[self.selector.ok_or_else(invalid)?];
        let mut prefix = 1u8;
        for depth in 0..=5 {
            for (slot, &code) in codes.iter().enumerate() {
                self.bits.charge(1)?;
                if code == prefix {
                    let symbol = if slot < 6 { slot } else { slot - 3 };
                    let (base, width) =
                        [(0, 4), (16, 4), (32, 5), (64, 6), (128, 6), (192, 6)][symbol];
                    return self
                        .recent
                        .get(base + self.bits.get(width)? as usize)
                        .copied()
                        .ok_or_else(invalid);
                }
            }
            if depth < 5 {
                prefix = (prefix << 1) | self.bits.get(1)? as u8;
            }
        }
        Err(invalid())
    }
    fn literal_count(&mut self) -> io::Result<usize> {
        let first = self.bits.get(2)? as usize;
        if first < 3 {
            return Ok(first + 1);
        }
        let second = self.bits.get(3)? as usize;
        if second < 7 {
            return Ok(second + 4);
        }
        match self.bits.get(4)? as usize {
            n @ 0..=13 => Ok(n + 11),
            14 => Ok(25 + self.bits.get(6)? as usize),
            _ => Ok(89 + self.bits.get(7)? as usize),
        }
    }
    fn copy_count(&mut self) -> io::Result<usize> {
        let first = self.bits.get(2)? as usize;
        if first < 3 {
            return Ok(first + 3);
        }
        match self.bits.get(3)? as usize {
            n @ 0..=4 => Ok(n + 6),
            5 => Ok(11 + self.bits.get(2)? as usize),
            6 => Ok(15 + self.bits.get(3)? as usize),
            _ => match self.bits.get(6)? as usize {
                n @ 0..=61 => Ok(n + 23),
                62 => Ok(85 + self.bits.get(5)? as usize),
                _ => Ok(117 + self.bits.get(7)? as usize),
            },
        }
    }
    fn copy_kind(&mut self) -> io::Result<u8> {
        let codes: &[(u8, u8)] = match self.produced {
            0..64 => &[(2, 0), (3, 2)],
            64..576 => &[(4, 0), (5, 1), (6, 3), (7, 2)],
            576..2624 => &[(8, 0), (9, 1), (5, 4), (6, 3), (7, 2)],
            _ => &[(8, 0), (9, 1), (5, 4), (6, 3), (14, 5), (15, 2)],
        };
        let mut prefix = 1u8;
        for _ in 0..3 {
            prefix = (prefix << 1) | self.bits.get(1)? as u8;
            for &(code, kind) in codes {
                self.bits.charge(1)?;
                if code == prefix {
                    return Ok(kind);
                }
            }
        }
        Err(invalid())
    }
    fn copy(&mut self) -> io::Result<()> {
        let kind = self.copy_kind()?;
        let length = if kind < 2 { 2 } else { self.copy_count()? };
        let (base, width) = match kind {
            0 | 2 => (0, 6),
            1 => (64, 8),
            3 => (64, 8 + u8::from(self.produced >= 320)),
            4 => (
                576,
                8 + [832, 1088, 1600]
                    .iter()
                    .filter(|&&n| self.produced >= n)
                    .count() as u8,
            ),
            5 => (
                2624,
                8 + [2880, 3136, 3648, 4672, 6720]
                    .iter()
                    .filter(|&&n| self.produced >= n)
                    .count() as u8,
            ),
            _ => return Err(invalid()),
        };
        let offset = base + self.bits.get(width)? as usize;
        if offset >= WINDOW
            || offset as u64 >= self.produced
            || length as u64 > self.expected - self.produced
        {
            return Err(invalid());
        }
        self.copying = length;
        self.distance = offset + 1;
        Ok(())
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.finished {
            return Ok(0);
        }
        if self.selector.is_none() && self.expected != 0 {
            self.selector = Some(self.bits.get(5)? as usize);
        }
        let mut count = 0;
        while count < output.len().min(8192) && self.produced < self.expected {
            let byte = if self.copying != 0 {
                self.bits.charge(1)?;
                self.copying -= 1;
                self.history[(self.position + WINDOW - self.distance) % WINDOW]
            } else if self.literals != 0 {
                self.literals -= 1;
                self.literal()?
            } else {
                if self.implicit_copy {
                    self.implicit_copy = false;
                    self.copy()?;
                } else if self.bits.get(1)? == 0 {
                    self.copy()?;
                } else {
                    self.literals = self.literal_count()?;
                    if self.literals as u64 > self.expected - self.produced {
                        return Err(invalid());
                    }
                    self.implicit_copy = self.literals < 216;
                }
                continue;
            };
            self.bits.charge(1)?;
            pmarc::observe(&mut self.recent, byte, |n| self.bits.charge(n))?;
            self.history[self.position] = byte;
            self.position = (self.position + 1) % WINDOW;
            output[count] = byte;
            count += 1;
            self.produced += 1;
        }
        if self.produced == self.expected {
            if self.copying != 0 || self.literals != 0 {
                return Err(invalid());
            }
            self.bits.finish_ignored_padding()?;
            self.finished = true;
        }
        Ok(count)
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for Pm1Reader<R, P> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if let Some(kind) = self.failure {
            return Err(io::Error::new(kind, "terminal_pm1_error"));
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
