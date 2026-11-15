//! ARC method-scoped RLE, explicit squeeze trees and historical LZW wire profiles.
use crate::{
    bitstream::Bits,
    invalid,
    stream_io::{filled, preserve_io_error, resource},
};
use std::io::{self, Read};
const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_WORK: u64 = 96 * 1024 * 1024 * 1024;
const NONE: u16 = u16::MAX;

/// Store belongs to framing. Each compressed method admits its complete private storage.
pub fn arc_memory_required(method: u8) -> io::Result<u64> {
    match method {
        3 => Ok(16 * 1024),
        4 => Ok(32 * 1024),
        5..=7 => Ok(64 * 1024),
        8 | 9 => Ok(96 * 1024),
        _ => Err(io::Error::new(io::ErrorKind::Unsupported, "arc_method")),
    }
}
struct Dictionary {
    prefixes: Vec<u16>,
    suffixes: Vec<u8>,
    occupied: Vec<bool>,
    links: Vec<u16>,
    count: usize,
    improved: bool,
}
impl Dictionary {
    fn new(size: usize, fixed: bool, improved: bool) -> io::Result<Self> {
        Ok(Self {
            prefixes: filled(size, NONE)?,
            suffixes: filled(size, 0)?,
            occupied: filled(size, false)?,
            links: if fixed { filled(size, NONE)? } else { vec![] },
            count: 0,
            improved,
        })
    }
    fn slot<R: Read, P: FnMut() -> io::Result<()>>(
        &self,
        prefix: u16,
        suffix: u8,
        bits: &mut Bits<R, P>,
    ) -> io::Result<(usize, Option<usize>)> {
        let key = u32::from(prefix.wrapping_add(u16::from(suffix)));
        let mut at = if self.improved {
            key.wrapping_mul(15073) & 4095
        } else {
            (key | 0x800).wrapping_mul(key | 0x800) >> 6 & 4095
        } as usize;
        bits.charge(1)?;
        let mut parent = None;
        if self.occupied[at] {
            let mut hops = 0;
            while self.links[at] != NONE {
                bits.charge(1)?;
                at = usize::from(self.links[at]);
                hops += 1;
                if hops >= 4096 {
                    return Err(invalid());
                }
            }
            parent = Some(at);
            at = (at + 101) & 4095;
            while self.occupied[at] {
                bits.charge(1)?;
                at = (at + 1) & 4095;
                hops += 1;
                if hops >= 8192 {
                    return Err(invalid());
                }
            }
        }
        Ok((at, parent))
    }
    fn add_fixed<R: Read, P: FnMut() -> io::Result<()>>(
        &mut self,
        prefix: u16,
        suffix: u8,
        bits: &mut Bits<R, P>,
    ) -> io::Result<usize> {
        let (at, parent) = self.slot(prefix, suffix, bits)?;
        if let Some(link) = parent {
            self.links[link] = at as u16;
        }
        self.prefixes[at] = prefix;
        self.suffixes[at] = suffix;
        self.occupied[at] = true;
        self.links[at] = NONE;
        self.count += 1;
        bits.charge(1)?;
        Ok(at)
    }
}
enum Wire {
    Rle,
    Squeeze(Vec<[i16; 2]>),
    Fixed {
        table: Dictionary,
        nibble: Option<u8>,
    },
    Dynamic {
        table: Dictionary,
        next: usize,
        width: u8,
        maximum: u8,
        group_codes: u8,
    },
}
/// Bounded raw decoding. The container must verify decoded size and its required CRC.
pub struct ArcReader<R, P = fn() -> io::Result<()>>
where
    P: FnMut() -> io::Result<()>,
{
    bits: Bits<R, P>,
    wire: Wire,
    stack: Vec<u8>,
    previous: Option<u16>,
    first: u8,
    remaining: u64,
    rle: bool,
    escaped: bool,
    history: Option<u8>,
    repeat: u16,
    ended: bool,
    finished: bool,
    failure: Option<io::ErrorKind>,
    #[cfg(test)]
    widths: u32,
    #[cfg(test)]
    resets: usize,
}
impl<R: Read> ArcReader<R> {
    /// Admit limits before allocating any dictionary or reading a tree.
    pub fn new(
        source: R,
        method: u8,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
    ) -> io::Result<Self> {
        Self::with_poll(source, method, packed, decoded, memory, work, || Ok(()))
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> ArcReader<R, P> {
    /// Polling covers source-independent tree, dictionary, expansion and repeat work.
    pub fn with_poll(
        source: R,
        method: u8,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
        mut poll: P,
    ) -> io::Result<Self> {
        if memory < arc_memory_required(method)? || packed > MAX_BYTES || decoded > MAX_BYTES {
            return Err(resource());
        }
        poll().map_err(preserve_io_error)?;
        let mut bits = Bits::with_poll(source, packed, work.min(MAX_WORK), poll);
        let wire = match method {
            3 => Wire::Rle,
            4 => Wire::Squeeze(read_tree(&mut bits)?),
            5..=7 => {
                bits.charge(4096 * 6)?;
                let mut table = Dictionary::new(4096, true, method == 7)?;
                for value in 0..=255 {
                    table.add_fixed(NONE, value, &mut bits)?;
                }
                Wire::Fixed {
                    table,
                    nibble: None,
                }
            }
            8 | 9 => {
                if method == 8 && bits.get(8)? != 12 {
                    return Err(invalid());
                }
                let capacity = if method == 8 { 4096 } else { 8192 };
                bits.charge(capacity as u64 * 4)?;
                Wire::Dynamic {
                    table: Dictionary::new(capacity, false, false)?,
                    next: 257,
                    width: 9,
                    maximum: if method == 8 { 12 } else { 13 },
                    group_codes: 0,
                }
            }
            _ => unreachable!("method checked before construction"),
        };
        let mut stack = Vec::new();
        if method >= 5 {
            stack
                .try_reserve_exact(if method == 9 { 8193 } else { 4097 })
                .map_err(|_| resource())?;
        }
        Ok(Self {
            bits,
            wire,
            stack,
            previous: None,
            first: 0,
            remaining: decoded,
            rle: !matches!(method, 5 | 9),
            escaped: false,
            history: None,
            repeat: 0,
            ended: false,
            finished: false,
            failure: None,
            #[cfg(test)]
            widths: 1 << 9,
            #[cfg(test)]
            resets: 0,
        })
    }
    /// Actual remaining work, including constructor work, for a shared member ledger.
    pub fn work_remaining(&self) -> u64 {
        self.bits.work
    }
    fn symbol(&mut self) -> io::Result<Option<u8>> {
        if let Some(value) = self.stack.pop() {
            self.bits.charge(1)?;
            return Ok(Some(value));
        }
        match &mut self.wire {
            Wire::Rle => {
                if self.bits.input.consumed == self.bits.input.packed {
                    return Ok(None);
                }
                Ok(Some(self.bits.get(8)? as u8))
            }
            Wire::Squeeze(nodes) => {
                if self.ended {
                    return Ok(None);
                }
                let mut at = 0usize;
                loop {
                    self.bits.charge(1)?;
                    let child = nodes[at][self.bits.get(1)? as usize];
                    if child < 0 {
                        let value = -(i32::from(child) + 1);
                        if value == 256 {
                            self.ended = true;
                            self.bits.finish_ignored_padding()?;
                            return Ok(None);
                        }
                        return Ok(Some(value as u8));
                    }
                    at = child as usize;
                }
            }
            Wire::Fixed { table, nibble } => {
                let needed = if self.previous.is_some() { 0 } else { 1 };
                let code = match fixed_code(&mut self.bits, nibble)? {
                    Some(code) => code,
                    None => return Ok(None),
                };
                let known = table.occupied[usize::from(code)];
                if needed != 0 && (!known || table.prefixes[usize::from(code)] != NONE) {
                    return Err(invalid());
                }
                if !known {
                    self.stack.push(self.first);
                }
                let start = if known {
                    code
                } else {
                    self.previous.ok_or_else(invalid)?
                };
                expand(table, start, &mut self.stack, &mut self.bits, true)?;
                let first = *self.stack.last().ok_or_else(invalid)?;
                if let Some(previous) = self.previous
                    && table.count < 4096
                {
                    let inserted = table.add_fixed(previous, first, &mut self.bits)?;
                    if !known && inserted != usize::from(code) {
                        return Err(invalid());
                    }
                } else if !known {
                    return Err(invalid());
                }
                self.first = first;
                self.previous = Some(code);
                self.bits.charge(1)?;
                Ok(self.stack.pop())
            }
            Wire::Dynamic {
                table,
                next,
                width,
                maximum,
                group_codes,
            } => {
                if *next == 1usize << *width && *width < *maximum {
                    align_group(&mut self.bits, group_codes, *width)?;
                    *width += 1;
                    #[cfg(test)]
                    {
                        self.widths |= 1 << *width;
                    }
                }
                let Some(mut code) = dynamic_code(&mut self.bits, *width, group_codes)? else {
                    return Ok(None);
                };
                if code == 256 {
                    if self.previous.is_none() {
                        return Err(invalid());
                    }
                    align_group(&mut self.bits, group_codes, *width)?;
                    *next = 257;
                    *width = 9;
                    self.previous = None;
                    #[cfg(test)]
                    {
                        self.resets += 1;
                    }
                    code =
                        dynamic_code(&mut self.bits, *width, group_codes)?.ok_or_else(invalid)?;
                }
                if self.previous.is_none() {
                    if code > 255 {
                        return Err(invalid());
                    }
                    self.previous = Some(code);
                    self.first = code as u8;
                    return Ok(Some(code as u8));
                }
                if usize::from(code) > *next || code == 256 {
                    return Err(invalid());
                }
                let previous = self.previous.ok_or_else(invalid)?;
                let special = usize::from(code) == *next;
                if special {
                    self.stack.push(self.first);
                }
                expand(
                    table,
                    if special { previous } else { code },
                    &mut self.stack,
                    &mut self.bits,
                    false,
                )?;
                let first = *self.stack.last().ok_or_else(invalid)?;
                if *next < table.prefixes.len() {
                    self.bits.charge(1)?;
                    table.prefixes[*next] = previous;
                    table.suffixes[*next] = first;
                    table.occupied[*next] = true;
                    *next += 1;
                } else if special {
                    return Err(invalid());
                }
                self.first = first;
                self.previous = Some(code);
                self.bits.charge(1)?;
                Ok(self.stack.pop())
            }
        }
    }
    fn byte(&mut self) -> io::Result<Option<u8>> {
        loop {
            self.bits.charge(1)?;
            if self.repeat != 0 {
                self.repeat -= 1;
                return self.history.map(Some).ok_or_else(invalid);
            }
            let Some(value) = self.symbol()? else {
                if self.escaped {
                    return Err(invalid());
                }
                return Ok(None);
            };
            if !self.rle {
                return Ok(Some(value));
            }
            if self.escaped {
                self.escaped = false;
                if value == 0 {
                    return Ok(Some(0x90));
                }
                if self.history.is_none() {
                    return Err(invalid());
                }
                self.repeat = u16::from(value) - 1;
            } else if value == 0x90 {
                self.escaped = true;
            } else {
                self.history = Some(value);
                return Ok(Some(value));
            }
        }
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for ArcReader<R, P> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if let Some(kind) = self.failure {
            return Err(io::Error::new(kind, "terminal_arc_error"));
        }
        if out.is_empty() {
            return Ok(0);
        }
        if self.finished {
            return Ok(0);
        }
        let result = (|| {
            let mut used = 0;
            for slot in out.iter_mut().take(8192) {
                let Some(value) = self.byte()? else {
                    if self.remaining != 0 {
                        return Err(invalid());
                    }
                    self.finished = true;
                    return Ok(used);
                };
                self.remaining = self.remaining.checked_sub(1).ok_or_else(invalid)?;
                *slot = value;
                used += 1;
            }
            Ok(used)
        })();
        if let Err(error) = &result {
            self.failure = Some(error.kind());
        }
        result
    }
}
fn fixed_code<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Bits<R, P>,
    nibble: &mut Option<u8>,
) -> io::Result<Option<u16>> {
    if bits.input.consumed == bits.input.packed {
        bits.finish_ignored_padding()?;
        return Ok(None);
    }
    if let Some(high) = nibble.take() {
        return Ok(Some((u16::from(high) << 8) | bits.get(8)? as u16));
    }
    let high = bits.get(8)? as u16;
    let low = bits.get(8)? as u8;
    *nibble = Some(low & 15);
    Ok(Some((high << 4) | u16::from(low >> 4)))
}
fn align_group<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Bits<R, P>,
    used: &mut u8,
    width: u8,
) -> io::Result<()> {
    while *used != 0 {
        bits.get(width)?;
        *used = (*used + 1) % 8;
    }
    Ok(())
}
fn dynamic_code<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Bits<R, P>,
    width: u8,
    used: &mut u8,
) -> io::Result<Option<u16>> {
    // A code may start inside the final already-read byte. Count buffered bits
    // explicitly instead of manufacturing an EOF code from missing bytes.
    if bits.available_bits() < u64::from(width) {
        if bits.available_bits() >= 8 {
            return Err(invalid());
        }
        bits.finish_ignored_padding()?;
        return Ok(None);
    }
    let code = bits.get(width)? as u16;
    *used = (*used + 1) % 8;
    Ok(Some(code))
}
fn expand<R: Read, P: FnMut() -> io::Result<()>>(
    table: &Dictionary,
    mut code: u16,
    stack: &mut Vec<u8>,
    bits: &mut Bits<R, P>,
    fixed: bool,
) -> io::Result<()> {
    let mut hops = 0;
    loop {
        bits.charge(1)?;
        let at = usize::from(code);
        if at >= table.prefixes.len() {
            return Err(invalid());
        }
        if !fixed && code < 256 {
            stack.push(code as u8);
            return Ok(());
        }
        if !table.occupied[at] || stack.len() >= table.prefixes.len() {
            return Err(invalid());
        }
        stack.push(table.suffixes[at]);
        if table.prefixes[at] == NONE {
            return Ok(());
        }
        code = table.prefixes[at];
        hops += 1;
        if hops >= table.prefixes.len() {
            return Err(invalid());
        }
    }
}
fn read_tree<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Bits<R, P>,
) -> io::Result<Vec<[i16; 2]>> {
    let count = bits.get(16)? as usize;
    if count > 256 {
        return Err(invalid());
    }
    bits.charge(count.max(1) as u64 * 2)?;
    let mut nodes = filled(count.max(1), [-257; 2])?;
    for node in nodes.iter_mut().take(count) {
        for child in node {
            *child = bits.get(16)? as i16;
            if *child >= count as i16 || *child < -257 {
                return Err(invalid());
            }
        }
    }
    let mut seen = [false; 256];
    let mut pending = vec![0usize];
    let mut eof = false;
    while let Some(at) = pending.pop() {
        bits.charge(1)?;
        if seen[at] {
            return Err(invalid());
        }
        seen[at] = true;
        for child in nodes[at] {
            if child >= 0 {
                pending.push(child as usize);
            } else {
                eof |= child == -257;
            }
        }
    }
    if !eof || (count != 0 && seen[..count].iter().any(|used| !used)) {
        return Err(invalid());
    }
    Ok(nodes)
}
#[cfg(test)]
mod tests;
