use super::bits::{ForwardBits, ReverseBits};
use crate::invalid;
use std::io;

/// One fully resolved finite-state entropy transition.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FseRow {
    /// Decoded symbol.
    pub symbol: u8,
    /// Number of additional bits needed to update the state.
    pub bits: u8,
    /// Baseline to add to that field.
    pub baseline: u16,
}
/// Bounded Zstandard FSE table. Repeat ownership belongs to its enclosing frame.
#[derive(Clone)]
pub struct FseTable {
    log: u8,
    rows: Vec<FseRow>,
}
/// Independent entropy alphabets used by Zstandard sequence tuples.
#[derive(Clone, Copy)]
pub enum FseSequenceKind {
    /// Literal length codes 0..35.
    LiteralLength,
    /// Match length codes 0..52.
    MatchLength,
    /// Offset codes, including repeat-offset values.
    Offset,
}
impl FseTable {
    /// Build one of RFC 8878's normative predefined sequence distributions.
    pub fn predefined(kind: FseSequenceKind) -> io::Result<Self> {
        match kind {
            FseSequenceKind::LiteralLength => Self::from_distribution(
                6,
                &[
                    4, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3,
                    2, 1, 1, 1, 1, 1, -1, -1, -1, -1,
                ],
            ),
            FseSequenceKind::MatchLength => Self::from_distribution(
                6,
                &[
                    1, 4, 3, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, -1, -1, -1, -1, -1,
                    -1, -1,
                ],
            ),
            FseSequenceKind::Offset => Self::from_distribution(
                5,
                &[
                    1, 1, 1, 1, 1, 1, 2, 2, 2, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, -1, -1,
                    -1, -1, -1,
                ],
            ),
        }
    }
    /// Construct the format-defined table from normalized probabilities.
    pub fn from_distribution(log: u8, probabilities: &[i16]) -> io::Result<Self> {
        if !(5..=9).contains(&log)
            || !(2..=256).contains(&probabilities.len())
            || probabilities.iter().any(|n| *n < -1)
            || probabilities.iter().filter(|n| **n != 0).count() < 2
        {
            return Err(invalid());
        }
        let size = 1usize << log;
        if probabilities
            .iter()
            .map(|n| usize::from(n.unsigned_abs()))
            .sum::<usize>()
            != size
        {
            return Err(invalid());
        }
        let mut rows = Vec::new();
        rows.try_reserve_exact(size)
            .map_err(|_| io::ErrorKind::OutOfMemory)?;
        rows.resize(size, FseRow::default());
        let low_count = probabilities.iter().filter(|n| **n == -1).count();
        let high = size - low_count;
        let mut low_at = size;
        for (symbol, count) in probabilities.iter().enumerate() {
            if *count == -1 {
                low_at -= 1;
                rows[low_at].symbol = symbol as u8;
            }
        }
        let step = (size >> 1) + (size >> 3) + 3;
        let mut position = 0;
        for (symbol, count) in probabilities.iter().enumerate() {
            for _ in 0..(*count).max(0) {
                rows[position].symbol = symbol as u8;
                position = (position + step) & (size - 1);
                while position >= high {
                    position = (position + step) & (size - 1);
                }
            }
        }
        if position != 0 {
            return Err(invalid());
        }
        let mut counters = [0u16; 256];
        for (counter, count) in counters.iter_mut().zip(probabilities) {
            *counter = count.unsigned_abs();
        }
        for row in &mut rows {
            let next = &mut counters[usize::from(row.symbol)];
            if *next == 0 {
                return Err(invalid());
            }
            row.bits = log - (15 - next.leading_zeros()) as u8;
            row.baseline = ((*next as usize) << row.bits)
                .checked_sub(size)
                .ok_or_else(invalid)? as u16;
            *next += 1;
        }
        Ok(Self { log, rows })
    }
    /// A format-defined RLE table has one state and consumes no state bits.
    pub fn rle(symbol: u8) -> Self {
        Self {
            log: 0,
            rows: vec![FseRow {
                symbol,
                bits: 0,
                baseline: 0,
            }],
        }
    }
    /// Parse a forward normalized-count description with explicit alphabet/log limits.
    pub fn read_header(bytes: &[u8], max_symbol: u8, max_log: u8) -> io::Result<(Self, usize)> {
        let mut bits = ForwardBits::new(bytes);
        let log = bits.take(4)? as u8 + 5;
        if log > max_log || max_log > 9 {
            return Err(invalid());
        }
        let mut remaining = 1usize << log;
        let mut probabilities = Vec::with_capacity(usize::from(max_symbol) + 1);
        while remaining != 0 {
            if probabilities.len() > usize::from(max_symbol) {
                return Err(invalid());
            }
            let width = (usize::BITS - (remaining + 1).leading_zeros()) as u8;
            let shorter = (1usize << width) - remaining - 2;
            let low = bits.peek(width - 1) as usize;
            let value = if low < shorter {
                bits.take(width - 1)? as usize
            } else {
                let full = bits.take(width)? as usize;
                if full >= 1 << (width - 1) {
                    full - shorter
                } else {
                    full
                }
            };
            if value > remaining + 1 {
                return Err(invalid());
            }
            let count = value as i16 - 1;
            remaining = remaining
                .checked_sub(usize::from(count.unsigned_abs()))
                .ok_or_else(invalid)?;
            probabilities.push(count);
            if count == 0 {
                loop {
                    let zeros = bits.take(2)? as usize;
                    if probabilities.len() + zeros > usize::from(max_symbol) + 1 {
                        return Err(invalid());
                    }
                    probabilities.resize(probabilities.len() + zeros, 0);
                    if zeros != 3 {
                        break;
                    }
                }
            }
        }
        Ok((
            Self::from_distribution(log, &probabilities)?,
            bits.consumed(),
        ))
    }
    /// Initialize a state strictly within this table's entropy bitstream.
    pub fn initial(&self, bits: &mut ReverseBits<'_>) -> io::Result<usize> {
        Ok(bits.take(self.log)? as usize)
    }
    /// Retrieve an admitted state, retaining the distinction between symbol and transition.
    pub fn row(&self, state: usize) -> io::Result<FseRow> {
        self.rows.get(state).copied().ok_or_else(invalid)
    }
    /// Consume the row's exact field and move to another admitted state.
    pub fn advance(&self, state: usize, bits: &mut ReverseBits<'_>) -> io::Result<usize> {
        let row = self.row(state)?;
        let next = usize::from(row.baseline) + bits.take(row.bits)? as usize;
        self.row(next)?;
        Ok(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_predefined_row_matches_the_published_wire_tables() {
        let expected = include_bytes!("../../../../tests/fixtures/streams/zstd-fse-tables.bin");
        let mut records = expected.as_chunks::<4>().0.iter();
        for kind in [
            FseSequenceKind::LiteralLength,
            FseSequenceKind::MatchLength,
            FseSequenceKind::Offset,
        ] {
            let table = FseTable::predefined(kind).unwrap();
            for (state, row) in table.rows.into_iter().enumerate() {
                let bytes = records.next().unwrap();
                assert_eq!(
                    row,
                    FseRow {
                        symbol: bytes[0],
                        bits: bytes[1],
                        baseline: u16::from_le_bytes(bytes[2..].try_into().unwrap())
                    },
                    "state {state}, table log {}",
                    table.log
                );
            }
        }
        assert!(records.next().is_none());
    }
    #[test]
    fn normalized_tables_require_exact_probability_space_and_alphabet() {
        for (log, counts) in [
            (4, vec![8, 8]),
            (10, vec![512, 512]),
            (5, vec![32]),
            (5, vec![15, 16]),
            (5, vec![-2, 30]),
            (5, vec![33, -1]),
        ] {
            assert!(FseTable::from_distribution(log, &counts).is_err());
        }
        let table = FseTable::from_distribution(5, &[-1; 32]).unwrap();
        assert_eq!(
            table.rows[0],
            FseRow {
                symbol: 31,
                bits: 5,
                baseline: 0
            }
        );
        assert!(table.row(32).is_err());
        for header in [vec![], vec![15], vec![0], vec![0x10, 0, 0, 0, 0]] {
            assert!(FseTable::read_header(&header, 11, 6).is_err());
        }
        // log=5; probability 16 is encoded as value 17 in five bits.
        // The remaining probability 16 uses value 17 encoded as value 31 in five bits.
        let header = [0x10, 0x3f];
        let (table, used) = FseTable::read_header(&header, 1, 6).unwrap();
        assert_eq!(used, 2);
        assert_eq!(table.rows.iter().filter(|row| row.symbol == 0).count(), 16);
        assert!(FseTable::read_header(&header[..1], 1, 6).is_err());
    }
}
