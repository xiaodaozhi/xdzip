//! Bounded four-byte hash chains shared by independently framed block encoders.
use crate::invalid;
use std::io;
const WINDOW: usize = 65536;
const ABSENT: u32 = u32::MAX;
pub(crate) fn search_depth(level: u8) -> io::Result<usize> {
    match level {
        1 => Ok(1),
        6 => Ok(16),
        9 => Ok(64),
        _ => Err(invalid()),
    }
}
pub(crate) struct MatchFinder {
    heads: Vec<u32>,
    previous: Vec<u32>,
    depth: usize,
}
impl MatchFinder {
    pub(crate) const MEMORY: usize = WINDOW * 8;
    pub(crate) fn new(depth: usize) -> io::Result<Self> {
        Self::with_work(depth, &mut |_| Ok(()))
    }
    pub(crate) fn with_work(
        depth: usize,
        charge: &mut impl FnMut(u64) -> io::Result<()>,
    ) -> io::Result<Self> {
        fn table(charge: &mut impl FnMut(u64) -> io::Result<()>) -> io::Result<Vec<u32>> {
            let mut table = Vec::new();
            table
                .try_reserve_exact(WINDOW)
                .map_err(|_| crate::stream_io::resource())?;
            for _ in 0..WINDOW / 1024 {
                charge(1024)?;
                table.extend_from_slice(&[ABSENT; 1024]);
            }
            Ok(table)
        }
        Ok(Self {
            heads: table(charge)?,
            previous: table(charge)?,
            depth,
        })
    }
    fn insert(&mut self, input: &[u8], position: usize) -> io::Result<u32> {
        let end = position.checked_add(4).ok_or_else(invalid)?;
        let bytes = input.get(position..end).ok_or_else(invalid)?;
        let position32 = u32::try_from(position).map_err(|_| invalid())?;
        let word = u32::from_le_bytes(bytes.try_into().expect("four-byte key"));
        let key = (word.wrapping_mul(0x9e37_79b1) >> 16) as usize;
        let previous = self.heads[key];
        self.previous[position % WINDOW] = previous;
        self.heads[key] = position32;
        Ok(previous)
    }
    /// The caller sets the format's last allowed match byte, independently of history.
    pub(crate) fn find(
        &mut self,
        input: &[u8],
        position: usize,
        maximum_end: usize,
    ) -> io::Result<(usize, usize)> {
        self.find_bounded(input, position, maximum_end, 65535, &mut |_| Ok(()))
    }
    /// A narrower wire dictionary and cumulative work admission remain caller-owned.
    pub(crate) fn find_bounded(
        &mut self,
        input: &[u8],
        position: usize,
        maximum_end: usize,
        maximum_distance: usize,
        charge: &mut impl FnMut(u64) -> io::Result<()>,
    ) -> io::Result<(usize, usize)> {
        if maximum_end > input.len() || position.checked_add(4).is_none_or(|n| n > maximum_end) {
            return Err(invalid());
        }
        if !(1..=65535).contains(&maximum_distance) {
            return Err(invalid());
        }
        charge(5)?;
        let mut candidate = self.insert(input, position)?;
        let mut best = (0, 0);
        for _ in 0..self.depth {
            if candidate == ABSENT {
                break;
            }
            let at = candidate as usize;
            charge(1)?;
            if at >= position || position - at > maximum_distance {
                break;
            }
            charge(4)?;
            if input[at..at + 4] == input[position..position + 4] {
                let maximum = maximum_end - position;
                let mut length = 4;
                while length < maximum {
                    charge(1)?;
                    if input[at + length] != input[position + length] {
                        break;
                    }
                    length += 1;
                }
                if length > best.1 {
                    best = (position - at, length);
                }
                if length == maximum {
                    break;
                }
            }
            let next = self.previous[at % WINDOW];
            if next >= candidate {
                break;
            }
            candidate = next;
        }
        Ok(best)
    }
    pub(crate) fn seed(&mut self, input: &[u8], begin: usize, end: usize) -> io::Result<()> {
        self.seed_with_work(input, begin, end, &mut |_| Ok(()))
    }
    pub(crate) fn seed_with_work(
        &mut self,
        input: &[u8],
        begin: usize,
        end: usize,
        charge: &mut impl FnMut(u64) -> io::Result<()>,
    ) -> io::Result<()> {
        if begin > end || end > input.len() {
            return Err(invalid());
        }
        for at in begin..end.min(input.len().saturating_sub(3)) {
            charge(5)?;
            self.insert(input, at)?;
        }
        Ok(())
    }
}
