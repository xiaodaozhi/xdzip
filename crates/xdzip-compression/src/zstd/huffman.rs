use super::{FseTable, ReverseBits};
use crate::invalid;
use std::io;

#[derive(Clone, Copy, Default)]
struct Code {
    symbol: u8,
    bits: u8,
}
/// A complete bounded Zstandard literal prefix table, reusable only within its frame.
#[derive(Clone)]
pub struct HuffmanTable {
    log: u8,
    codes: Vec<Code>,
}
fn weights(bytes: &[u8]) -> io::Result<Vec<u8>> {
    let (table, used) = FseTable::read_header(bytes, 11, 6)?;
    let mut bits = ReverseBits::new(bytes.get(used..).ok_or_else(invalid)?)?;
    let mut states = [table.initial(&mut bits)?, table.initial(&mut bits)?];
    let mut output = Vec::with_capacity(255);
    let mut active = 0;
    loop {
        let row = table.row(states[active])?;
        output.push(row.symbol);
        if usize::from(row.bits) > bits.remaining() {
            // A final FSE transition implicitly pads missing bits with zero.
            // Its just-decoded symbol and the other state's symbol end the sequence.
            let final_state = usize::from(row.baseline) + bits.padded_prefix(row.bits)? as usize;
            table.row(final_state)?;
            output.push(table.row(states[active ^ 1])?.symbol);
            if output.len() > 255 {
                return Err(invalid());
            }
            return Ok(output);
        }
        if output.len() >= 255 {
            return Err(invalid());
        }
        states[active] = table.advance(states[active], &mut bits)?;
        active ^= 1;
    }
}
impl HuffmanTable {
    fn from_weights(weights: &mut Vec<u8>) -> io::Result<Self> {
        if weights.is_empty() || weights.len() > 255 || weights.iter().any(|w| *w > 11) {
            return Err(invalid());
        }
        let sum = weights
            .iter()
            .filter(|w| **w != 0)
            .map(|w| 1usize << (*w - 1))
            .sum::<usize>();
        if sum == 0 {
            return Err(invalid());
        }
        let log = (usize::BITS - sum.leading_zeros()) as u8;
        if log > 11 {
            return Err(invalid());
        }
        let rest = (1usize << log) - sum;
        if !rest.is_power_of_two() {
            return Err(invalid());
        }
        weights.push(rest.trailing_zeros() as u8 + 1);
        if !weights.contains(&1) {
            return Err(invalid());
        }
        let mut codes = Vec::new();
        codes
            .try_reserve_exact(1 << log)
            .map_err(|_| io::ErrorKind::OutOfMemory)?;
        codes.resize(1 << log, Code::default());
        let mut position = 0;
        for weight in 1..=log {
            for (symbol, actual) in weights.iter().enumerate() {
                if *actual != weight {
                    continue;
                }
                let width = 1usize << (weight - 1);
                let range = codes
                    .get_mut(position..position + width)
                    .ok_or_else(invalid)?;
                range.fill(Code {
                    symbol: symbol as u8,
                    bits: log + 1 - weight,
                });
                position += width;
            }
        }
        if position != codes.len() || codes.iter().any(|code| code.bits == 0) {
            return Err(invalid());
        }
        Ok(Self { log, codes })
    }
    /// Parse direct or FSE-compressed weights and return the exact description length.
    pub fn read_description(bytes: &[u8]) -> io::Result<(Self, usize)> {
        let first = *bytes.first().ok_or_else(invalid)?;
        let (mut decoded, used) = if first < 128 {
            let end = usize::from(first) + 1;
            (weights(bytes.get(1..end).ok_or_else(invalid)?)?, end)
        } else {
            let count = usize::from(first) - 127;
            let end = 1 + count.div_ceil(2);
            let packed = bytes.get(1..end).ok_or_else(invalid)?;
            let decoded = (0..count)
                .map(|i| {
                    if i % 2 == 0 {
                        packed[i / 2] >> 4
                    } else {
                        packed[i / 2] & 15
                    }
                })
                .collect();
            (decoded, end)
        };
        Ok((Self::from_weights(&mut decoded)?, used))
    }
    /// Decode exactly the admitted symbol count, rejecting leftover or missing bits.
    pub fn decode(&self, bytes: &[u8], size: usize) -> io::Result<Vec<u8>> {
        if size > 128 * 1024 {
            return Err(invalid());
        }
        let mut bits = ReverseBits::new(bytes)?;
        let mut output = Vec::new();
        output
            .try_reserve_exact(size)
            .map_err(|_| io::ErrorKind::OutOfMemory)?;
        for _ in 0..size {
            let index = bits.padded_prefix(self.log)? as usize;
            let code = self.codes.get(index).ok_or_else(invalid)?;
            bits.take(code.bits)?;
            output.push(code.symbol);
        }
        if bits.remaining() != 0 {
            return Err(invalid());
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn direct_weights_assign_natural_order_and_consume_exact_prefix_bits() {
        // A..F use weights 4,3,2,0,1 and implied 1.
        let mut weights = vec![4, 3, 2, 0, 1];
        let table = HuffmanTable::from_weights(&mut weights).unwrap();
        assert_eq!(table.decode(&[0x01, 0x0d], 4).unwrap(), [0, 1, 4, 5]);
        assert!(table.decode(&[0x01, 0x0d], 3).is_err());
        assert!(table.decode(&[0x01, 0x0d], 5).is_err());
        let (direct, used) = HuffmanTable::read_description(&[132, 0x43, 0x20, 0x10]).unwrap();
        assert_eq!(used, 4);
        assert_eq!(direct.decode(&[0x01, 0x0d], 4).unwrap(), [0, 1, 4, 5]);
        for input in [
            vec![],
            vec![0],
            vec![128],
            vec![128, 0],
            vec![128, 0xf0],
            vec![129, 0x22],
        ] {
            assert!(HuffmanTable::read_description(&input).is_err());
        }
    }
}
