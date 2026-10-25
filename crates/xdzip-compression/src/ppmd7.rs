//! PPMd var.H with the 7z range coder. Other PPMd/range variants are distinct.
use crate::{
    invalid,
    stream_io::{Input, READ_QUANTUM, resource},
};
use std::io::{self, Read};

#[path = "ppmd7_arena.rs"]
mod arena;
#[path = "ppmd7_model.rs"]
mod model;

const FIXED_MEMORY: u64 = 65536;
fn properties(properties: &[u8]) -> io::Result<(u8, u32)> {
    if properties.len() < 5 {
        return Err(invalid());
    }
    let order = properties[0];
    let memory = u32::from_le_bytes(properties[1..5].try_into().map_err(|_| invalid())?);
    if !(2..=64).contains(&order) || !(2048..=u32::MAX - 36).contains(&memory) {
        return Err(invalid());
    }
    Ok((order, memory))
}
/// Actual safe-storage admission: history, typed arena, deterministic ledger's
/// bounded traversal scratch and fixed models/input. The declared model memory
/// controls format restarts and is not the actual Rust allocation bound.
pub fn ppmd7_memory_required(properties_bytes: &[u8]) -> io::Result<u64> {
    let (_, memory) = properties(properties_bytes)?;
    let slot =
        std::mem::size_of::<arena::Cell>() as u64 + std::mem::size_of::<arena::Address>() as u64;
    u64::from(memory / 12)
        .checked_mul(slot)
        .and_then(|value| value.checked_add(u64::from(memory)))
        .and_then(|value| value.checked_add(FIXED_MEMORY))
        .ok_or_else(resource)
}

struct Range {
    width: u32,
    code: u32,
}
impl Range {
    fn new<R: Read>(input: &mut Input<R>) -> io::Result<Self> {
        if input.byte(input.packed)? != 0 {
            return Err(invalid());
        }
        let mut code = 0;
        for _ in 0..4 {
            code = (code << 8) | u32::from(input.byte(input.packed)?);
        }
        if code == u32::MAX {
            return Err(invalid());
        }
        Ok(Self {
            width: u32::MAX,
            code,
        })
    }
    fn normalize<R: Read>(&mut self, input: &mut Input<R>) -> io::Result<()> {
        while self.width < 1 << 24 {
            self.width <<= 8;
            self.code = (self.code << 8) | u32::from(input.byte(input.packed)?);
        }
        if self.code >= self.width {
            return Err(invalid());
        }
        Ok(())
    }
    fn count(&self, total: u32) -> io::Result<(u32, u32)> {
        if total == 0 {
            return Err(invalid());
        }
        let unit = self.width / total;
        if unit == 0 {
            return Err(invalid());
        }
        let count = self.code / unit;
        if count >= total {
            return Err(invalid());
        }
        Ok((count, unit))
    }
    fn select<R: Read>(
        &mut self,
        input: &mut Input<R>,
        unit: u32,
        start: u32,
        frequency: u32,
    ) -> io::Result<()> {
        if frequency == 0 {
            return Err(invalid());
        }
        self.code = self
            .code
            .checked_sub(start.checked_mul(unit).ok_or_else(invalid)?)
            .ok_or_else(invalid)?;
        self.width = frequency.checked_mul(unit).ok_or_else(invalid)?;
        self.normalize(input)
    }
    fn binary<R: Read>(&mut self, input: &mut Input<R>, probability: &mut u16) -> io::Result<bool> {
        let split = (self.width / 16384) * u32::from(*probability);
        let mean = (*probability + 32) / 128;
        let hit = self.code < split;
        if hit {
            self.width = split;
            *probability = *probability - mean + 128;
        } else {
            self.code -= split;
            self.width -= split;
            *probability -= mean;
        }
        if self.width == 0 {
            return Err(invalid());
        }
        self.normalize(input)?;
        Ok(hit)
    }
}

/// Exact-span streaming PPMd7z reader. Known-size streams require code zero at
/// exactly N and never decode an extra END to repair nonzero code. Unknown size
/// requires actual END. Both require exact packed consumption and poison on error.
/// Read emits <=8 KiB; deterministic ledger glue/restart work depends on admitted M.
pub struct Ppmd7Reader<R> {
    input: Input<R>,
    range: Option<Range>,
    model: model::Model,
    expected: Option<u64>,
    decoded: u64,
    ended: bool,
    failed: bool,
}
impl<R: Read> Ppmd7Reader<R> {
    /// Admit all safe storage before allocation. Accept five-byte properties
    /// with ignored extension bytes, matching the 7z C++ reader profile.
    pub fn new(
        source: R,
        packed: u64,
        expected: Option<u64>,
        properties_bytes: &[u8],
        memory_limit: u64,
    ) -> io::Result<Self> {
        let (order, memory) = properties(properties_bytes)?;
        if ppmd7_memory_required(properties_bytes)? > memory_limit {
            return Err(resource());
        }
        Ok(Self {
            input: Input::new(source, packed),
            range: None,
            model: model::Model::new(order, memory)?,
            expected,
            decoded: 0,
            ended: false,
            failed: false,
        })
    }
    /// Logical packed bytes, excluding buffered read-ahead.
    pub fn consumed_bytes(&self) -> u64 {
        self.input.consumed
    }
    /// Uncompressed bytes decoded into caller buffers.
    pub fn decoded_bytes(&self) -> u64 {
        self.decoded
    }
    fn initialize(&mut self) -> io::Result<()> {
        if self.range.is_none() {
            self.range = Some(Range::new(&mut self.input)?);
        }
        Ok(())
    }
    fn terminal(&mut self) -> io::Result<()> {
        self.initialize()?;
        if self.expected != Some(self.decoded)
            || self.range.as_ref().ok_or_else(invalid)?.code != 0
            || self.input.consumed != self.input.packed
        {
            return Err(invalid());
        }
        self.ended = true;
        Ok(())
    }
    /// Validate the known-size terminal state without discarding unread data.
    /// Unknown-size users drain to actual END first. Errors seal the reader.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(());
        }
        let result = self.terminal();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.initialize()?;
        let mut count = 0;
        while count < output.len().min(READ_QUANTUM) {
            if self.expected == Some(self.decoded) {
                self.terminal()?;
                break;
            }
            match self
                .model
                .decode(self.range.as_mut().ok_or_else(invalid)?, &mut self.input)?
            {
                Some(byte) => {
                    output[count] = byte;
                    count += 1;
                    self.decoded = self.decoded.checked_add(1).ok_or_else(invalid)?;
                }
                None => {
                    if self.expected.is_some()
                        || self.range.as_ref().ok_or_else(invalid)?.code != 0
                        || self.input.consumed != self.input.packed
                    {
                        return Err(invalid());
                    }
                    self.ended = true;
                    break;
                }
            }
        }
        Ok(count)
    }
}
impl<R: Read> Read for Ppmd7Reader<R> {
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
#[path = "ppmd7_tests.rs"]
mod tests;
