use crate::invalid;
use std::io;

fn extract(bytes: &[u8], start: usize, width: u8) -> u32 {
    let byte = start / 8;
    let shift = start % 8;
    let count = (shift + usize::from(width)).div_ceil(8);
    let mut value = 0u64;
    for i in 0..count {
        value |= u64::from(bytes.get(byte + i).copied().unwrap_or(0)) << (8 * i);
    }
    ((value >> shift) & ((1u64 << width) - 1)) as u32
}

/// Bounded reverse little-endian fields; the highest terminal one-bit is excluded.
pub struct ReverseBits<'a> {
    bytes: &'a [u8],
    remaining: usize,
}
impl<'a> ReverseBits<'a> {
    /// Require the nonzero terminal byte before exposing any bits.
    pub fn new(bytes: &'a [u8]) -> io::Result<Self> {
        let last = *bytes.last().ok_or_else(invalid)?;
        if last == 0 {
            return Err(invalid());
        }
        let remaining = (bytes.len() - 1)
            .checked_mul(8)
            .and_then(|n| n.checked_add((7 - last.leading_zeros()) as usize))
            .ok_or_else(invalid)?;
        Ok(Self { bytes, remaining })
    }
    /// Number of useful bits still available, excluding the terminal flag.
    pub fn remaining(&self) -> usize {
        self.remaining
    }
    /// Consume up to 32 bits strictly within the physical bitstream.
    pub fn take(&mut self, width: u8) -> io::Result<u32> {
        if width > 32 || usize::from(width) > self.remaining {
            return Err(invalid());
        }
        self.remaining -= usize::from(width);
        Ok(extract(self.bytes, self.remaining, width))
    }
    /// Inspect a Huffman prefix, padding only unavailable low-order bits with zero.
    /// A caller must still consume the actual code length strictly with `take`.
    pub fn padded_prefix(&self, width: u8) -> io::Result<u32> {
        if width > 32 {
            return Err(invalid());
        }
        let actual = self.remaining.min(usize::from(width)) as u8;
        if actual == 0 {
            return Ok(0);
        }
        Ok(extract(self.bytes, self.remaining - usize::from(actual), actual) << (width - actual))
    }
}

pub(super) struct ForwardBits<'a> {
    bytes: &'a [u8],
    position: usize,
}
impl<'a> ForwardBits<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }
    pub fn peek(&self, width: u8) -> u32 {
        extract(self.bytes, self.position, width)
    }
    pub fn take(&mut self, width: u8) -> io::Result<u32> {
        if width > 32
            || self
                .position
                .checked_add(usize::from(width))
                .is_none_or(|end| end > self.bytes.len().saturating_mul(8))
        {
            return Err(invalid());
        }
        let value = self.peek(width);
        self.position += usize::from(width);
        Ok(value)
    }
    pub fn consumed(&self) -> usize {
        self.position.div_ceil(8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reverse_fields_follow_specification_order_and_exact_boundaries() {
        let mut bits = ReverseBits::new(&[0x10, 0x0d]).unwrap();
        assert_eq!(bits.remaining(), 11);
        assert_eq!(bits.take(1).unwrap(), 1);
        assert_eq!(bits.take(2).unwrap(), 1);
        assert_eq!(bits.take(4).unwrap(), 1);
        assert_eq!(bits.take(4).unwrap(), 0);
        assert_eq!(bits.remaining(), 0);
        assert_eq!(bits.padded_prefix(32).unwrap(), 0);
        assert!(bits.take(1).is_err());
        assert_eq!(bits.take(0).unwrap(), 0);
        assert!(ReverseBits::new(&[]).is_err());
        assert!(ReverseBits::new(&[0]).is_err());
        let mut bits = ReverseBits::new(&[0x1b]).unwrap();
        assert_eq!(bits.padded_prefix(8).unwrap(), 0xb0);
        assert!(bits.take(5).is_err());
        assert_eq!(bits.take(4).unwrap(), 11);
        let mut bits = ReverseBits::new(&[0xef, 0xcd, 0xab, 0x89, 1]).unwrap();
        assert_eq!(bits.take(32).unwrap(), 0x89ab_cdef);
    }
}
