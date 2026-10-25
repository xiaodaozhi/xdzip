//! RAR29's four-byte carryless binding for the shared original PPMd H model.
//! Byte alignment, escape commands and final-file checks belong to the caller.
use std::io::{self, Read};
use xdzip_codec_api::{Cancellation, payload::DecodeError};
use xdzip_compression::{PpmdCapacity, PpmdModel, PpmdRange, PpmdWork, ppmd_memory_required};

fn invalid() -> io::Error {
    io::Error::other(DecodeError::Integrity)
}
fn check(control: &Cancellation) -> io::Result<()> {
    control
        .check()
        .map_err(|error| io::Error::other(crate::decode_error(error)))
}
fn byte(input: &mut dyn Read, control: &Cancellation) -> io::Result<u8> {
    let mut buffer = [0];
    loop {
        check(control)?;
        match input.read(&mut buffer) {
            Ok(1) => return Ok(buffer[0]),
            Ok(_) => return Err(invalid()),
            Err(error)
                if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none() => {}
            // Wrap source errors before crossing shared-model I/O: an arbitrary
            // source InvalidData/OOM must retain its source classification.
            Err(error) => return Err(io::Error::other(crate::algorithm::input_error(error))),
        }
    }
}
fn decode_error(error: io::Error) -> DecodeError {
    if let Some(error) = error
        .get_ref()
        .and_then(|e| e.downcast_ref::<DecodeError>())
    {
        *error
    } else if error.kind() == io::ErrorKind::OutOfMemory {
        DecodeError::Limit
    } else if error.kind() == io::ErrorKind::InvalidData {
        DecodeError::Integrity
    } else {
        DecodeError::SourceIo
    }
}

struct Range {
    low: u32,
    code: u32,
    width: u32,
}
impl Range {
    fn new(input: &mut dyn Read, control: &Cancellation) -> io::Result<Self> {
        let mut code = 0;
        for _ in 0..4 {
            code = (code << 8) | u32::from(byte(input, control)?);
        }
        Ok(Self {
            low: 0,
            code,
            width: u32::MAX,
        })
    }
    fn normalize(&mut self, input: &mut dyn Read, control: &Cancellation) -> io::Result<()> {
        const TOP: u32 = 1 << 24;
        const BOTTOM: u32 = 1 << 15;
        // Each legal interval becomes normalized within at most four shifts.
        // A zero corrective width is damaged state, never an endless read loop.
        let mut shifts = 0;
        loop {
            if (self.low ^ self.low.wrapping_add(self.width)) < TOP {
                // The stable-prefix branch takes precedence over BOTTOM.
            } else if self.width < BOTTOM {
                self.width = self.low.wrapping_neg() & (BOTTOM - 1);
            } else {
                return Ok(());
            }
            if self.width == 0 || shifts == 4 {
                return Err(invalid());
            }
            shifts += 1;
            self.code = (self.code << 8) | u32::from(byte(input, control)?);
            self.low <<= 8;
            self.width <<= 8;
        }
    }
    fn select(&mut self, unit: u32, start: u32, frequency: u32) -> io::Result<()> {
        if unit == 0 || frequency == 0 {
            return Err(invalid());
        }
        self.low = self
            .low
            .wrapping_add(start.checked_mul(unit).ok_or_else(invalid)?);
        self.width = frequency.checked_mul(unit).ok_or_else(invalid)?;
        Ok(())
    }
}

struct Work<'a> {
    control: &'a Cancellation,
    counter: &'a mut u64,
    start: u64,
    maximum: u64,
}
impl PpmdWork for Work<'_> {
    fn checkpoint(&mut self) -> io::Result<()> {
        check(self.control)
    }
    fn reserve_initialization(&mut self, bytes: u64) -> io::Result<()> {
        self.checkpoint()?;
        if bytes > self.maximum.saturating_sub(*self.counter - self.start) {
            return Err(io::Error::other(DecodeError::Limit));
        }
        Ok(())
    }
    fn initialized(&mut self, bytes: u64) -> io::Result<()> {
        *self.counter = self
            .counter
            .checked_add(bytes)
            .ok_or_else(|| io::Error::other(DecodeError::Limit))?;
        Ok(())
    }
}
struct Binding<'a> {
    range: &'a mut Range,
    input: &'a mut dyn Read,
    work: Work<'a>,
}
impl PpmdWork for Binding<'_> {
    fn checkpoint(&mut self) -> io::Result<()> {
        self.work.checkpoint()
    }
    fn reserve_initialization(&mut self, bytes: u64) -> io::Result<()> {
        self.work.reserve_initialization(bytes)
    }
    fn initialized(&mut self, bytes: u64) -> io::Result<()> {
        self.work.initialized(bytes)
    }
}
impl PpmdRange for Binding<'_> {
    fn count(&mut self, total: u32) -> io::Result<(u32, u32)> {
        self.range.normalize(self.input, self.work.control)?;
        let unit = self
            .range
            .width
            .checked_div(total)
            .filter(|u| *u != 0)
            .ok_or_else(invalid)?;
        let count = self.range.code.wrapping_sub(self.range.low) / unit;
        if count >= total {
            return Err(invalid());
        }
        Ok((count, unit))
    }
    fn select(&mut self, unit: u32, start: u32, frequency: u32) -> io::Result<()> {
        self.range.select(unit, start, frequency)
    }
    fn binary(&mut self, probability: u16) -> io::Result<bool> {
        if probability == 0 || probability >= 16384 {
            return Err(invalid());
        }
        let (count, unit) = self.count(16384)?;
        let probability = u32::from(probability);
        let hit = count < probability;
        let (start, width) = if hit {
            (0, probability)
        } else {
            (probability, 16384 - probability)
        };
        // RAR discards division remainder on both branches, unlike PPMd7z.
        self.range.select(unit, start, width)?;
        Ok(hit)
    }
}

/// Task-private PPM state. An error seals it until an explicit `clear`.
#[derive(Default)]
pub(crate) struct Model {
    model: Option<PpmdModel>,
    range: Option<Range>,
    failed: bool,
    initialized: u64,
}
impl Model {
    pub fn clear(&mut self) {
        self.model = None;
        self.range = None;
        self.failed = false;
    }
    pub fn allocated_bytes(&self) -> u64 {
        self.model.as_ref().map_or(0, PpmdModel::allocated_bytes)
            + std::mem::size_of::<Self>() as u64
    }
    pub fn init(
        &mut self,
        input: &mut dyn Read,
        control: &Cancellation,
        maximum_memory: u64,
        maximum_initialization_work: u64,
    ) -> Result<Option<u8>, DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let result = self
            .initialize(input, control, maximum_memory, maximum_initialization_work)
            .map_err(decode_error);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn initialize(
        &mut self,
        input: &mut dyn Read,
        control: &Cancellation,
        maximum_memory: u64,
        maximum_initialization_work: u64,
    ) -> io::Result<Option<u8>> {
        let flags = byte(input, control)?;
        if flags & 0x80 == 0 {
            return Err(invalid());
        }
        if flags & 0x20 != 0 {
            let memory = (u32::from(byte(input, control)?) + 1) << 20;
            let code = (flags & 31) + 1;
            let order = if code > 16 {
                16 + 3 * (code - 16)
            } else {
                code
            };
            if order < 2 {
                return Err(invalid());
            }
            let required = ppmd_memory_required(memory)?;
            if required > maximum_memory {
                return Err(io::Error::other(DecodeError::Limit));
            }
            // Release first: a reset has no observable prior statistics, and
            // must not transiently hold old and new large Rust arenas at once.
            self.model = None;
            self.range = None;
            check(control)?;
            let start = self.initialized;
            let mut work = Work {
                control,
                counter: &mut self.initialized,
                start,
                maximum: maximum_initialization_work,
            };
            let model = PpmdModel::with_work(order, memory, PpmdCapacity::Rar, &mut work)?;
            if model.allocated_bytes() + std::mem::size_of::<Self>() as u64 > maximum_memory {
                return Err(io::Error::other(DecodeError::Limit));
            }
            self.model = Some(model);
        } else if self.model.is_none() {
            return Err(invalid());
        } else if self.allocated_bytes() > maximum_memory {
            return Err(io::Error::other(DecodeError::Limit));
        }
        let escape = if flags & 0x40 != 0 {
            Some(byte(input, control)?)
        } else {
            None
        };
        self.range = Some(Range::new(input, control)?);
        check(control)?;
        Ok(escape)
    }
    pub fn decode_byte(
        &mut self,
        input: &mut dyn Read,
        control: &Cancellation,
        maximum_initialization_work: u64,
    ) -> Result<u8, DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let result = self
            .symbol(input, control, maximum_initialization_work)
            .map_err(decode_error);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn symbol(
        &mut self,
        input: &mut dyn Read,
        control: &Cancellation,
        maximum_initialization_work: u64,
    ) -> io::Result<u8> {
        let model = self.model.as_mut().ok_or_else(invalid)?;
        let range = self.range.as_mut().ok_or_else(invalid)?;
        let start = self.initialized;
        let work = Work {
            control,
            counter: &mut self.initialized,
            start,
            maximum: maximum_initialization_work,
        };
        let symbol = model
            .decode(&mut Binding { range, input, work })?
            .ok_or_else(invalid)?;
        // Complete the shared model update before returning any control byte.
        range.normalize(input, control)?;
        check(control)?;
        Ok(symbol)
    }
    pub fn initialization_work(&self) -> u64 {
        self.initialized
    }
}

#[cfg(test)]
#[path = "ppm29_tests.rs"]
mod tests;
