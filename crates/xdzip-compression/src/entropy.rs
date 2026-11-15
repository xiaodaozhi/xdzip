//! Canonical prefix assignment shared by independently framed compression methods.
use crate::{invalid, stream_io::filled};
use std::io;
pub(crate) struct Canonical {
    counts: [u16; 17],
    first: [u32; 17],
    offset: [u16; 17],
    symbols: Vec<u16>,
    singleton: Option<u16>,
}
impl Canonical {
    pub fn single(symbol: u16) -> Self {
        Self {
            counts: [0; 17],
            first: [0; 17],
            offset: [0; 17],
            symbols: vec![],
            singleton: Some(symbol),
        }
    }
    pub fn is_single(&self) -> bool {
        self.singleton.is_some()
    }
    pub fn from_lengths(lengths: &[u8], maximum: u8, allow_single: bool) -> io::Result<Self> {
        if lengths.is_empty() || lengths.len() > 4096 || !(1..=16).contains(&maximum) {
            return Err(invalid());
        }
        let mut counts = [0u16; 17];
        for &n in lengths {
            if n > maximum {
                return Err(invalid());
            }
            if n != 0 {
                counts[n as usize] += 1;
            }
        }
        let used: u16 = counts.iter().sum();
        if used == 1 && allow_single {
            return Ok(Self::single(
                lengths.iter().position(|&n| n != 0).ok_or_else(invalid)? as u16,
            ));
        }
        let mut left = 1i32;
        let mut first = [0u32; 17];
        let mut offset = [0u16; 17];
        for n in 1..=usize::from(maximum) {
            left = left * 2 - i32::from(counts[n]);
            if left < 0 {
                return Err(invalid());
            }
            first[n] = (first[n - 1] + u32::from(counts[n - 1])) * 2;
            offset[n] = offset[n - 1] + counts[n - 1];
        }
        if left != 0 || used < 2 {
            return Err(invalid());
        }
        let mut symbols = filled(usize::from(used), 0u16)?;
        let mut next = offset;
        for (symbol, &n) in lengths.iter().enumerate() {
            if n != 0 {
                let at = &mut next[n as usize];
                symbols[usize::from(*at)] = symbol as u16;
                *at += 1;
            }
        }
        Ok(Self {
            counts,
            first,
            offset,
            symbols,
            singleton: None,
        })
    }
    pub fn symbol(&self, mut bit: impl FnMut() -> io::Result<u8>) -> io::Result<u16> {
        if let Some(value) = self.singleton {
            return Ok(value);
        }
        let mut code = 0u32;
        for n in 1..17 {
            let next = bit()?;
            if next > 1 {
                return Err(invalid());
            }
            code = (code << 1) | u32::from(next);
            if code >= self.first[n] && code - self.first[n] < u32::from(self.counts[n]) {
                return self
                    .symbols
                    .get(usize::from(self.offset[n]) + (code - self.first[n]) as usize)
                    .copied()
                    .ok_or_else(invalid);
            }
        }
        Err(invalid())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn full_sixteen_bit_codes_and_exact_completeness_are_not_truncated() {
        let mut lengths: Vec<u8> = (1..=15).collect();
        lengths.extend([16, 16]);
        let tree = Canonical::from_lengths(&lengths, 16, false).unwrap();
        let mut count = 0;
        assert_eq!(
            tree.symbol(|| {
                count += 1;
                Ok(1)
            })
            .unwrap(),
            16
        );
        assert_eq!(count, 16);
        assert!(Canonical::from_lengths(&lengths, 15, false).is_err());
        for lengths in [&[1, 1, 1][..], &[2, 2], &[0, 0], &[1]] {
            assert!(Canonical::from_lengths(lengths, 16, false).is_err());
        }
        let tree = Canonical::from_lengths(&[0, 16], 16, true).unwrap();
        assert_eq!(
            tree.symbol(|| panic!("singleton reads no source")).unwrap(),
            1
        );
        let tree = Canonical::from_lengths(&[1, 1], 16, false).unwrap();
        assert!(tree.symbol(|| Ok(2)).is_err());
    }
}
