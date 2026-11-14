//! Bounded four-byte hash chains shared by independently framed block encoders.
use crate::{invalid, stream_io::filled};
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
        Ok(Self {
            heads: filled(WINDOW, ABSENT)?,
            previous: filled(WINDOW, ABSENT)?,
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
        if maximum_end > input.len() || position.checked_add(4).is_none_or(|n| n > maximum_end) {
            return Err(invalid());
        }
        let mut candidate = self.insert(input, position)?;
        let mut best = (0, 0);
        for _ in 0..self.depth {
            if candidate == ABSENT {
                break;
            }
            let at = candidate as usize;
            if at >= position || position - at > 65535 {
                break;
            }
            if input[at..at + 4] == input[position..position + 4] {
                let maximum = maximum_end - position;
                let mut length = 4;
                while length < maximum && input[at + length] == input[position + length] {
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
        if begin > end || end > input.len() {
            return Err(invalid());
        }
        for at in begin..end.min(input.len().saturating_sub(3)) {
            self.insert(input, at)?;
        }
        Ok(())
    }
}
