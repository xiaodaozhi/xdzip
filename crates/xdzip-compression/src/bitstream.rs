use crate::{
    invalid,
    stream_io::{Input, resource},
};
use std::io::{self, Read};
pub(crate) struct Bits<R, P = fn() -> io::Result<()>, const MSB: bool = false> {
    pub input: Input<R>,
    value: u32,
    count: u8,
    pub work: u64,
    poll: P,
    until_poll: u64,
}
#[cfg(test)]
impl<R: Read> Bits<R> {
    pub fn new(source: R, packed: u64, work: u64) -> Self {
        Self::with_poll(source, packed, work, || Ok(()))
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>, const MSB: bool> Bits<R, P, MSB> {
    pub fn with_poll(source: R, packed: u64, work: u64, poll: P) -> Self {
        Self {
            input: Input::new(source, packed),
            value: 0,
            count: 0,
            work,
            poll,
            until_poll: 8192,
        }
    }
    pub fn charge(&mut self, amount: u64) -> io::Result<()> {
        self.work = self.work.checked_sub(amount).ok_or_else(resource)?;
        if amount >= self.until_poll {
            (self.poll)().map_err(crate::stream_io::preserve_io_error)?;
            self.until_poll = 8192;
        } else {
            self.until_poll -= amount;
        }
        Ok(())
    }
    pub fn get(&mut self, n: u8) -> io::Result<u32> {
        if n > 24 {
            return Err(invalid());
        }
        self.charge(u64::from(n).max(1))?;
        while self.count < n {
            let byte = self.input.byte(self.input.packed)?;
            self.value |= u32::from(if MSB { byte.reverse_bits() } else { byte }) << self.count;
            self.count += 8;
        }
        let value = self.value & ((1u32 << n) - 1);
        self.value >>= n;
        self.count -= n;
        Ok(if MSB && n != 0 {
            value.reverse_bits() >> (32 - n)
        } else {
            value
        })
    }
    pub fn finish_ignored_padding(&mut self) -> io::Result<()> {
        if self.input.consumed != self.input.packed {
            return Err(invalid());
        }
        Ok(())
    }
    pub fn align_zero(&mut self) -> io::Result<()> {
        if self.value != 0 {
            return Err(invalid());
        }
        self.count = 0;
        Ok(())
    }
    pub fn finish(&mut self) -> io::Result<()> {
        self.align_zero()?;
        if self.input.consumed != self.input.packed {
            return Err(invalid());
        }
        Ok(())
    }
}
