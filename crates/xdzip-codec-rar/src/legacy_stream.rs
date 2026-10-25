//! Shared bounded bit access, history and delivery for the older LZ families.
use crate::algorithm::WorkOutput;
use crate::lz_bits::{BitRead, Input};
use crate::{FILE_LIMIT, decode_error};
use std::io::Read;
use xdzip_codec_api::{Cancellation, payload::DecodeError};

pub(crate) struct Bits<'a> {
    input: Input<'a>,
    reservoir: u64,
    available: u8,
    offset: u8,
}
impl<'a> Bits<'a> {
    pub fn new(source: &'a mut dyn Read) -> Self {
        Self {
            input: Input::new(source),
            reservoir: 0,
            available: 0,
            offset: 0,
        }
    }
    pub fn peek(&mut self, count: u8) -> Result<Option<u64>, DecodeError> {
        if count > 56 {
            return Err(DecodeError::Integrity);
        }
        while self.available < count {
            let Some(byte) = self.input.next()? else {
                return Ok(None);
            };
            self.reservoir = (self.reservoir << 8) | u64::from(byte);
            self.available += 8;
        }
        Ok(Some(if count == 0 {
            0
        } else {
            self.reservoir >> (self.available - count)
        }))
    }
    pub fn get(&mut self, count: u8) -> Result<u64, DecodeError> {
        let value = self.peek(count)?.ok_or(DecodeError::Integrity)?;
        self.available -= count;
        self.reservoir &= (1u64 << self.available) - 1;
        self.offset = (self.offset + count) % 8;
        Ok(value)
    }
    /// The old optional-tail rule starts at the current physical byte, including
    /// its already consumed high bits. Lookahead never pads missing input.
    pub fn byte_tail_at_least(&mut self, bytes: usize) -> Result<bool, DecodeError> {
        let bits = bytes
            .checked_mul(8)
            .and_then(|n| n.checked_sub(usize::from(self.offset)))
            .and_then(|n| u8::try_from(n).ok())
            .filter(|n| *n <= 56)
            .ok_or(DecodeError::Integrity)?;
        Ok(self.peek(bits)?.is_some())
    }
    /// Known-size older streams have no invented EOF marker. The container has
    /// already bounded this input and independently checks content integrity.
    pub fn drain(&mut self, control: &Cancellation) -> Result<(), DecodeError> {
        self.reservoir = 0;
        self.available = 0;
        loop {
            control.check().map_err(decode_error)?;
            for _ in 0..8192 {
                if self.input.next()?.is_none() {
                    return Ok(());
                }
            }
        }
    }
}
impl BitRead for Bits<'_> {
    fn get(&mut self, count: u8) -> Result<u64, DecodeError> {
        Bits::get(self, count)
    }
}

#[derive(Default)]
pub(crate) struct History {
    bytes: Vec<u8>,
    active: usize,
    position: usize,
    available: usize,
}
impl History {
    pub fn clear(&mut self) {
        self.active = 0;
        self.position = 0;
        self.available = 0;
    }
    pub fn len(&self) -> usize {
        self.bytes.capacity()
    }
    pub fn prepare(
        &mut self,
        size: usize,
        solid: bool,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        if !size.is_power_of_two() || !(65536..=1048576).contains(&size) {
            return Err(DecodeError::Unsupported);
        }
        if solid {
            return if self.active == 0 || size > self.active {
                Err(DecodeError::Unsupported)
            } else {
                Ok(())
            };
        }
        if size > self.bytes.len() {
            let extra = size - self.bytes.len();
            sink.reserve_aux(extra as u64)?;
            self.bytes
                .try_reserve_exact(extra)
                .map_err(|_| DecodeError::Limit)?;
            while self.bytes.len() < size {
                control.check().map_err(decode_error)?;
                let count = (size - self.bytes.len()).min(8192);
                self.bytes.resize(self.bytes.len() + count, 0);
                sink.produced_aux(count as u64)?;
            }
        }
        // Invalidating old history avoids repeatedly clearing a large buffer.
        // No previous byte is readable until current history makes it valid.
        self.active = size;
        self.position = 0;
        self.available = 0;
        Ok(())
    }
    fn put(&mut self, byte: u8) -> Result<(), DecodeError> {
        if self.active == 0 {
            return Err(DecodeError::Integrity);
        }
        self.bytes[self.position] = byte;
        self.position = (self.position + 1) % self.active;
        self.available = (self.available + 1).min(self.active);
        Ok(())
    }
    fn valid_distance(&self, distance: u64) -> bool {
        distance != 0 && distance <= self.available as u64 && distance <= self.active as u64
    }
    fn previous(&self, distance: u64) -> u8 {
        self.bytes[(self.position + self.active - distance as usize) % self.active]
    }
}

pub(crate) struct Delivery {
    bytes: [u8; 8192],
    pending: usize,
    position: u64,
    expected: u64,
}
impl Delivery {
    pub fn new(expected: u64) -> Self {
        Self {
            bytes: [0; 8192],
            pending: 0,
            position: 0,
            expected,
        }
    }
    pub fn position(&self) -> u64 {
        self.position
    }
    pub fn remaining(&self) -> u64 {
        self.expected - self.position
    }
    fn flush(
        &mut self,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        let count = std::mem::take(&mut self.pending);
        if count != 0 {
            sink.produced_work(count as u64)?;
            sink.write_precharged_chunk(&self.bytes[..count])?;
        }
        Ok(())
    }
    fn reserve(
        &mut self,
        count: usize,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        if count > 8192
            || count as u64 > self.remaining()
            || self
                .position
                .checked_add(count as u64)
                .is_none_or(|n| n > FILE_LIMIT)
        {
            return Err(DecodeError::Integrity);
        }
        if self.pending + count > self.bytes.len() {
            self.flush(sink, control)?;
        }
        sink.reserve_work(count as u64)
    }
    fn put(&mut self, byte: u8, history: &mut History) -> Result<(), DecodeError> {
        history.put(byte)?;
        self.bytes[self.pending] = byte;
        self.pending += 1;
        self.position += 1;
        Ok(())
    }
    pub fn byte(
        &mut self,
        byte: u8,
        history: &mut History,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if history.active == 0 {
            return Err(DecodeError::Integrity);
        }
        self.reserve(1, sink, control)?;
        self.put(byte, history)
    }
    pub fn copy(
        &mut self,
        distance: u64,
        length: usize,
        history: &mut History,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if length == 0 || !history.valid_distance(distance) {
            return Err(DecodeError::Integrity);
        }
        self.reserve(length, sink, control)?;
        for _ in 0..length {
            self.put(history.previous(distance), history)?;
        }
        Ok(())
    }
    pub fn finish(
        &mut self,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if self.position != self.expected {
            return Err(DecodeError::Integrity);
        }
        self.flush(sink, control)
    }
    /// Error paths retain actual generated work without publishing pending bytes.
    pub fn settle_error(&mut self, sink: &mut dyn WorkOutput) -> Result<(), DecodeError> {
        let count = std::mem::take(&mut self.pending);
        if count == 0 {
            Ok(())
        } else {
            sink.produced_work(count as u64)
        }
    }
}

#[cfg(test)]
#[path = "legacy_stream_tests.rs"]
mod tests;
