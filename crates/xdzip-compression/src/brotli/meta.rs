use super::{bits::Bits, dictionary, prefix::Prefix};
use crate::{invalid, stream_io::filled};
use std::io::{self, Read};
pub(super) fn number<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Bits<R, P>,
) -> io::Result<usize> {
    if bits.get(1)? == 0 {
        return Ok(1);
    }
    let n = bits.get(3)? as u8;
    Ok(if n == 0 {
        2
    } else {
        (1usize << n) + 1 + bits.get(n)? as usize
    })
}
fn count<R: Read, P: FnMut() -> io::Result<()>>(
    tree: &Prefix,
    bits: &mut Bits<R, P>,
) -> io::Result<u32> {
    // RFC 7932 section 6: consecutive ranges from 1, with these widths.
    const WIDTHS: [u8; 26] = [
        2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 6, 6, 7, 8, 9, 10, 11, 12, 13, 24,
    ];
    let symbol = usize::from(tree.symbol(bits)?);
    let &width = WIDTHS.get(symbol).ok_or_else(invalid)?;
    let base: u32 = 1 + WIDTHS[..symbol].iter().map(|&n| 1u32 << n).sum::<u32>();
    Ok(base + bits.get(width)?)
}
struct Blocks {
    size: usize,
    previous: usize,
    current: usize,
    remaining: u32,
    types: Option<Prefix>,
    counts: Option<Prefix>,
    seen: Vec<bool>,
}
impl Blocks {
    fn read<R: Read, P: FnMut() -> io::Result<()>>(bits: &mut Bits<R, P>) -> io::Result<Self> {
        let size = number(bits)?;
        let (types, counts, remaining) = if size == 1 {
            (None, None, u32::MAX)
        } else {
            let types = Prefix::read(bits, size + 2)?;
            let counts = Prefix::read(bits, 26)?;
            let remaining = count(&counts, bits)?;
            (Some(types), Some(counts), remaining)
        };
        let mut seen = filled(size, false)?;
        seen[0] = true;
        Ok(Self {
            size,
            previous: 1,
            current: 0,
            remaining,
            types,
            counts,
            seen,
        })
    }
    fn take<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        bits: &mut Bits<R, P>,
    ) -> io::Result<usize> {
        if self.size == 1 {
            return Ok(0);
        }
        if self.remaining == 0 {
            let symbol = self.types.as_ref().ok_or_else(invalid)?.symbol(bits)?;
            let next = match symbol {
                0 => self.previous,
                1 => (self.current + 1) % self.size,
                _ => usize::from(symbol - 2),
            };
            if next >= self.size {
                return Err(invalid());
            }
            self.previous = self.current;
            self.current = next;
            self.seen[next] = true;
            self.remaining = count(self.counts.as_ref().ok_or_else(invalid)?, bits)?;
        }
        self.remaining -= 1;
        Ok(self.current)
    }
    fn complete(&self) -> bool {
        self.seen.iter().all(|&v| v)
    }
}
fn context_map<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Bits<R, P>,
    size: usize,
    trees: usize,
) -> io::Result<Vec<u8>> {
    let mut map = filled(size, 0u8)?;
    if trees == 1 {
        return Ok(map);
    }
    let rle = if bits.get(1)? == 0 {
        0
    } else {
        bits.get(4)? as u8 + 1
    };
    let prefix = Prefix::read(bits, trees + usize::from(rle))?;
    let mut at = 0;
    while at < size {
        let symbol = prefix.symbol(bits)?;
        if symbol == 0 {
            at += 1;
        } else if symbol <= u16::from(rle) {
            let repeat = (1usize << symbol) + bits.get(symbol as u8)? as usize;
            at = at
                .checked_add(repeat)
                .filter(|&n| n <= size)
                .ok_or_else(invalid)?;
            bits.charge(repeat as u64)?;
        } else {
            let value = symbol - u16::from(rle);
            if usize::from(value) >= trees {
                return Err(invalid());
            }
            map[at] = value as u8;
            at += 1;
        }
    }
    if bits.get(1)? != 0 {
        let mut mtf = [0u8; 256];
        for (at, value) in mtf.iter_mut().enumerate() {
            *value = at as u8;
        }
        for value in &mut map {
            let index = usize::from(*value);
            bits.charge(index as u64 + 1)?;
            let next = mtf[index];
            mtf[..=index].rotate_right(1);
            mtf[0] = next;
            *value = next;
        }
    }
    let mut seen = [false; 256];
    for &v in &map {
        seen[usize::from(v)] = true;
    }
    if seen[..trees].iter().any(|&v| !v) || seen[trees..].iter().any(|&v| v) {
        return Err(invalid());
    }
    Ok(map)
}
fn trees<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Bits<R, P>,
    count: usize,
    alphabet: usize,
) -> io::Result<Vec<Prefix>> {
    let mut trees = Vec::new();
    trees
        .try_reserve_exact(count)
        .map_err(|_| crate::stream_io::resource())?;
    for _ in 0..count {
        trees.push(Prefix::read(bits, alphabet)?);
    }
    Ok(trees)
}
pub(super) struct Meta {
    literals: Blocks,
    commands: Blocks,
    distances: Blocks,
    modes: Vec<u8>,
    literal_map: Vec<u8>,
    distance_map: Vec<u8>,
    literal_trees: Vec<Prefix>,
    command_trees: Vec<Prefix>,
    distance_trees: Vec<Prefix>,
    postfix: u8,
    direct: u32,
}
impl Meta {
    pub fn read<R: Read, P: FnMut() -> io::Result<()>>(bits: &mut Bits<R, P>) -> io::Result<Self> {
        let literals = Blocks::read(bits)?;
        let commands = Blocks::read(bits)?;
        let distances = Blocks::read(bits)?;
        let postfix = bits.get(2)? as u8;
        let direct = bits.get(4)? << postfix;
        let mut modes = filled(literals.size, 0u8)?;
        for mode in &mut modes {
            *mode = bits.get(2)? as u8;
        }
        let nl = number(bits)?;
        let literal_map = context_map(bits, literals.size * 64, nl)?;
        let nd = number(bits)?;
        let distance_map = context_map(bits, distances.size * 4, nd)?;
        let literal_trees = trees(bits, nl, 256)?;
        let command_trees = trees(bits, commands.size, 704)?;
        let distance_trees = trees(bits, nd, (16 + direct + (48 << postfix)) as usize)?;
        Ok(Self {
            literals,
            commands,
            distances,
            modes,
            literal_map,
            distance_map,
            literal_trees,
            command_trees,
            distance_trees,
            postfix,
            direct,
        })
    }
    pub fn command<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        bits: &mut Bits<R, P>,
    ) -> io::Result<(usize, usize, bool)> {
        const GROUPS: [(usize, usize); 11] = [
            (0, 0),
            (0, 8),
            (0, 0),
            (0, 8),
            (8, 0),
            (8, 8),
            (0, 16),
            (16, 0),
            (8, 16),
            (16, 8),
            (16, 16),
        ];
        const INSERT: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 12, 14, 24,
        ];
        const COPY: [u8; 24] = [
            0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10, 24,
        ];
        let kind = self.commands.take(bits)?;
        let code = usize::from(self.command_trees[kind].symbol(bits)?);
        let &(insert, copy) = GROUPS.get(code / 64).ok_or_else(invalid)?;
        let insert = insert + ((code >> 3) & 7);
        let copy = copy + (code & 7);
        let base_insert: usize = INSERT[..insert].iter().map(|&n| 1usize << n).sum();
        let base_copy: usize = 2 + COPY[..copy].iter().map(|&n| 1usize << n).sum::<usize>();
        Ok((
            base_insert + bits.get(INSERT[insert])? as usize,
            base_copy + bits.get(COPY[copy])? as usize,
            code < 128,
        ))
    }
    pub fn literal<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        bits: &mut Bits<R, P>,
        last: u8,
        before: u8,
    ) -> io::Result<u8> {
        let kind = self.literals.take(bits)?;
        let context = dictionary::context(self.modes[kind], last, before)?;
        let tree = usize::from(self.literal_map[kind * 64 + context]);
        self.literal_trees[tree]
            .symbol(bits)?
            .try_into()
            .map_err(|_| invalid())
    }
    /// Return both the physical symbol and resolved distance; symbol zero alone preserves the cache.
    pub fn distance<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        bits: &mut Bits<R, P>,
        copy: usize,
        implicit: bool,
        cache: &[u32; 4],
    ) -> io::Result<(u16, u32)> {
        let symbol = if implicit {
            0
        } else {
            let kind = self.distances.take(bits)?;
            let context = copy.saturating_sub(2).min(3);
            let tree = usize::from(self.distance_map[kind * 4 + context]);
            self.distance_trees[tree].symbol(bits)?
        };
        let distance = match symbol {
            0..=3 => cache[usize::from(symbol)],
            4..=15 => {
                let code = usize::from(symbol - 4);
                let base = cache[usize::from(code >= 6)];
                let delta = (code % 6) / 2 + 1;
                if code.is_multiple_of(2) {
                    base.checked_sub(delta as u32).ok_or_else(invalid)?
                } else {
                    base.checked_add(delta as u32).ok_or_else(invalid)?
                }
            }
            _ => {
                let code = u32::from(symbol) - 16;
                if code < self.direct {
                    code + 1
                } else {
                    let code = code - self.direct;
                    let high = code >> self.postfix;
                    let low = code & ((1 << self.postfix) - 1);
                    let width = (1 + (high >> 1)) as u8;
                    if width > 24 {
                        return Err(invalid());
                    }
                    let offset = ((2 + (high & 1)) << width) - 4;
                    ((offset + bits.get(width)?) << self.postfix) + low + self.direct + 1
                }
            }
        };
        if distance == 0 {
            return Err(invalid());
        }
        Ok((symbol, distance))
    }
    pub fn complete(&self) -> io::Result<()> {
        if !self.literals.complete() || !self.commands.complete() || !self.distances.complete() {
            return Err(invalid());
        }
        Ok(())
    }
}
