use super::bits::Bits;
use crate::{invalid, stream_io::filled};
use std::io::{self, Read};
/// Canonical MSB-first symbol codes within the LSB-first Brotli field stream.
/// Count/offset storage avoids a 2^15 table per declared context.
pub(super) struct Prefix {
    counts: [u16; 16],
    first: [u16; 16],
    offset: [u16; 16],
    symbols: Vec<u16>,
    singleton: Option<u16>,
}
impl Prefix {
    fn single(symbol: u16) -> Self {
        Self {
            counts: [0; 16],
            first: [0; 16],
            offset: [0; 16],
            symbols: vec![],
            singleton: Some(symbol),
        }
    }
    fn from_lengths(lengths: &[u8], allow_single: bool) -> io::Result<Self> {
        if lengths.is_empty() || lengths.len() > 704 {
            return Err(invalid());
        }
        let mut counts = [0u16; 16];
        for &n in lengths {
            if n > 15 {
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
        let mut first = [0u16; 16];
        let mut offset = [0u16; 16];
        for n in 1..16 {
            left = left * 2 - i32::from(counts[n]);
            if left < 0 {
                return Err(invalid());
            }
            first[n] = ((u32::from(first[n - 1]) + u32::from(counts[n - 1])) * 2)
                .try_into()
                .map_err(|_| invalid())?;
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
    pub fn symbol<R: Read, P: FnMut() -> io::Result<()>>(
        &self,
        bits: &mut Bits<R, P>,
    ) -> io::Result<u16> {
        if let Some(value) = self.singleton {
            bits.charge(1)?;
            return Ok(value);
        }
        let mut value = 0u16;
        for n in 1..16 {
            value = (value << 1) | bits.get(1)? as u16;
            if value >= self.first[n] && value - self.first[n] < self.counts[n] {
                return self
                    .symbols
                    .get(usize::from(self.offset[n] + value - self.first[n]))
                    .copied()
                    .ok_or_else(invalid);
            }
        }
        Err(invalid())
    }
    pub fn read<R: Read, P: FnMut() -> io::Result<()>>(
        bits: &mut Bits<R, P>,
        alphabet: usize,
    ) -> io::Result<Self> {
        if !(2..=704).contains(&alphabet) {
            return Err(invalid());
        }
        bits.charge(alphabet as u64)?;
        let skip = bits.get(2)? as usize;
        if skip == 1 {
            let count = bits.get(2)? as usize + 1;
            let width = (usize::BITS - (alphabet - 1).leading_zeros()) as u8;
            let mut symbols = [0usize; 4];
            for at in 0..count {
                let value = bits.get(width)? as usize;
                if value >= alphabet || symbols[..at].contains(&value) {
                    return Err(invalid());
                }
                symbols[at] = value;
            }
            if count == 1 {
                return Ok(Self::single(symbols[0] as u16));
            }
            let lens = match count {
                2 => [1, 1, 0, 0],
                3 => [1, 2, 2, 0],
                4 => {
                    if bits.get(1)? == 0 {
                        [2, 2, 2, 2]
                    } else {
                        [1, 2, 3, 3]
                    }
                }
                _ => return Err(invalid()),
            };
            let mut lengths = filled(alphabet, 0u8)?;
            for at in 0..count {
                lengths[symbols[at]] = lens[at];
            }
            return Self::from_lengths(&lengths, false);
        }
        // RFC 7932 section 3.5 gives this code-length alphabet order and wire code.
        const ORDER: [usize; 18] = [1, 2, 3, 4, 0, 5, 17, 6, 16, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let mut code_lengths = [0u8; 18];
        let mut space = 32i32;
        for &symbol in &ORDER[skip..] {
            let value = match bits.get(2)? {
                0 => 0,
                1 => 4,
                2 => 3,
                3 => {
                    if bits.get(1)? == 0 {
                        2
                    } else if bits.get(1)? == 0 {
                        1
                    } else {
                        5
                    }
                }
                _ => unreachable!(),
            };
            code_lengths[symbol] = value;
            if value != 0 {
                space -= 32 >> value;
                if space < 0 {
                    return Err(invalid());
                }
            }
            if space == 0 {
                break;
            }
        }
        let code = Self::from_lengths(&code_lengths, true)?;
        let mut lengths = filled(alphabet, 0u8)?;
        let mut at = 0usize;
        let mut space = 32768i32;
        let mut previous = 8u8;
        let mut last_repeat = None;
        let mut repeats = 0usize;
        while space != 0 {
            if at == alphabet {
                return Err(invalid());
            }
            let value = code.symbol(bits)?;
            if value < 16 {
                let value = value as u8;
                lengths[at] = value;
                at += 1;
                last_repeat = None;
                repeats = 0;
                if value != 0 {
                    previous = value;
                    space -= 32768 >> value;
                }
            } else {
                let width = if value == 16 { 2 } else { 3 };
                let extra = bits.get(width)? as usize + 3;
                let old = if last_repeat == Some(value) {
                    repeats
                } else {
                    0
                };
                let new = if old == 0 {
                    extra
                } else {
                    (old - 2)
                        .checked_mul(1 << width)
                        .and_then(|n| n.checked_add(extra))
                        .ok_or_else(invalid)?
                };
                let count = new.checked_sub(old).ok_or_else(invalid)?;
                let end = at
                    .checked_add(count)
                    .filter(|&n| n <= alphabet)
                    .ok_or_else(invalid)?;
                bits.charge(count as u64)?;
                let n = if value == 16 { previous } else { 0 };
                lengths[at..end].fill(n);
                at = end;
                if n != 0 {
                    space -= (32768 >> n) * count as i32;
                }
                last_repeat = Some(value);
                repeats = new;
            }
            if space < 0 {
                return Err(invalid());
            }
        }
        Self::from_lengths(&lengths, false)
    }
}
