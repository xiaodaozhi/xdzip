//! Independent LHA static-Huffman/LZSS member streams; containers retain CRC and file authority.
use crate::{
    bitstream::Bits,
    entropy::Canonical,
    invalid,
    stream_io::{filled, preserve_io_error, resource},
};
use std::io::{self, Read};
#[cfg(test)]
pub(crate) mod fixture_cases;
#[cfg(test)]
pub(crate) mod tests;
type Input<R, P> = Bits<R, P, true>;
const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_WORK: u64 = 96 * 1024 * 1024 * 1024;
fn parameters(method: u8) -> io::Result<(u8, usize, u8)> {
    match method {
        3 => Ok((13, 128, 7)),
        4 => Ok((12, 14, 4)),
        5 => Ok((13, 14, 4)),
        6 => Ok((15, 16, 5)),
        7 => Ok((16, 17, 5)),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "unsupported_lzh_method",
        )),
    }
}
/// Conservative admission for the actual dictionary, canonical tables, buffered I/O and scratch.
pub fn lzh_memory_required(method: u8) -> io::Result<u64> {
    let (bits, _, _) = parameters(method)?;
    Ok((1u64 << bits) + 64 * 1024)
}
fn short_tree<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Input<R, P>,
    alphabet: usize,
    width: u8,
    special: bool,
) -> io::Result<Canonical> {
    bits.charge(alphabet as u64)?;
    let count = bits.get(width)? as usize;
    if count == 0 {
        let symbol = bits.get(width)? as usize;
        if symbol >= alphabet {
            return Err(invalid());
        }
        return Ok(Canonical::single(symbol as u16));
    }
    if count > alphabet {
        return Err(invalid());
    }
    let mut lengths = filled(alphabet, 0u8)?;
    let mut at = 0;
    while at < count {
        let mut length = bits.get(3)? as u8;
        if length == 7 {
            while bits.get(1)? != 0 {
                length += 1;
                if length > 16 {
                    return Err(invalid());
                }
            }
        }
        lengths[at] = length;
        at += 1;
        if special && at == 3 {
            let extra = bits.get(2)? as usize;
            at = at
                .checked_add(extra)
                .filter(|&n| n <= count)
                .ok_or_else(invalid)?;
        }
    }
    Canonical::from_lengths(&lengths, 16, false)
}
fn symbol<R: Read, P: FnMut() -> io::Result<()>>(
    tree: &Canonical,
    bits: &mut Input<R, P>,
) -> io::Result<u16> {
    if tree.is_single() {
        bits.charge(1)?;
    }
    tree.symbol(|| Ok(bits.get(1)? as u8))
}
fn literal_tree<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Input<R, P>,
    lengths: &Canonical,
) -> io::Result<Canonical> {
    bits.charge(510)?;
    let count = bits.get(9)? as usize;
    if count == 0 {
        let code = bits.get(9)? as u16;
        if code >= 510 {
            return Err(invalid());
        }
        return Ok(Canonical::single(code));
    }
    if count > 510 {
        return Err(invalid());
    }
    let mut result = filled(510, 0u8)?;
    let mut at = 0;
    while at < count {
        let value = symbol(lengths, bits)?;
        match value {
            0..=2 => {
                let repeat = match value {
                    0 => 1,
                    1 => 3 + bits.get(4)? as usize,
                    _ => 20 + bits.get(9)? as usize,
                };
                bits.charge(repeat as u64)?;
                at = at
                    .checked_add(repeat)
                    .filter(|&n| n <= count)
                    .ok_or_else(invalid)?;
            }
            3..=18 => {
                result[at] = (value - 2) as u8;
                at += 1;
            }
            _ => return Err(invalid()),
        }
    }
    Canonical::from_lengths(&result, 16, false)
}
fn legacy_tree<R: Read, P: FnMut() -> io::Result<()>>(
    bits: &mut Input<R, P>,
    positions: bool,
) -> io::Result<Canonical> {
    let alphabet = if positions { 128 } else { 286 };
    bits.charge(alphabet as u64)?;
    let mut lengths = filled(alphabet, 0u8)?;
    for at in 0..alphabet {
        lengths[at] = if positions {
            bits.get(4)? as u8
        } else if bits.get(1)? != 0 {
            bits.get(4)? as u8 + 1
        } else {
            0
        };
        if at == 2 && lengths[..3] == [1, 1, 1] {
            let code = bits.get(if positions { 7 } else { 9 })? as usize;
            if code >= alphabet {
                return Err(invalid());
            }
            return Ok(Canonical::single(code as u16));
        }
    }
    Canonical::from_lengths(&lengths, 16, false)
}
/// Exact-size static LH3/LH4/LH5/LH6/LH7 stream, without container framing or CRC.
/// Its declared span and decoded size must come from a validated caller-owned container.
pub struct LzhReader<R, P = fn() -> io::Result<()>> {
    bits: Input<R, P>,
    dictionary: Vec<u8>,
    position: usize,
    remaining: u64,
    position_symbols: usize,
    position_bits: u8,
    commands: u16,
    method: u8,
    literals: Canonical,
    positions: Canonical,
    copy: usize,
    distance: usize,
    ended: bool,
    failed: bool,
}
impl<R: Read> LzhReader<R> {
    /// Admit method, known decoded size, packed span and all dictionary memory before use.
    pub fn new(
        source: R,
        method: u8,
        packed: u64,
        decoded: u64,
        memory_limit: u64,
        work_limit: u64,
    ) -> io::Result<Self> {
        Self::with_poll(
            source,
            method,
            packed,
            decoded,
            memory_limit,
            work_limit,
            || Ok(()),
        )
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> LzhReader<R, P> {
    /// Unconsumed actual-work allowance, including after a terminal error.
    pub fn work_remaining(&self) -> u64 {
        self.bits.work
    }
    /// Cooperative control also covers tree decoding and dictionary/output work without new input.
    pub fn with_poll(
        source: R,
        method: u8,
        packed: u64,
        decoded: u64,
        memory_limit: u64,
        work_limit: u64,
        poll: P,
    ) -> io::Result<Self> {
        let (width, position_symbols, position_bits) = parameters(method)?;
        if packed > MAX_BYTES || decoded > MAX_BYTES || lzh_memory_required(method)? > memory_limit
        {
            return Err(resource());
        }
        if decoded == 0 && packed != 0 {
            return Err(invalid());
        }
        let mut bits = Bits::with_poll(source, packed, work_limit.min(MAX_WORK), poll);
        bits.charge(1u64 << width)?;
        Ok(Self {
            bits,
            dictionary: filled(1usize << width, b' ')?,
            position: 0,
            remaining: decoded,
            position_symbols,
            position_bits,
            commands: 0,
            method,
            literals: Canonical::single(0),
            positions: Canonical::single(0),
            copy: 0,
            distance: 0,
            ended: false,
            failed: false,
        })
    }
    fn next_command(&mut self) -> io::Result<u16> {
        if self.commands == 0 {
            self.commands = self.bits.get(16)? as u16;
            if self.commands == 0 {
                return Err(invalid());
            }
            if self.method == 3 {
                self.literals = legacy_tree(&mut self.bits, false)?;
                self.positions = if self.bits.get(1)? != 0 {
                    legacy_tree(&mut self.bits, true)?
                } else {
                    self.bits.charge(128)?;
                    let lengths: Vec<u8> = (0..128)
                        .map(|at| {
                            2 + [1, 1, 3, 6, 13, 31, 78]
                                .iter()
                                .filter(|&&boundary| at >= boundary)
                                .count() as u8
                        })
                        .collect();
                    Canonical::from_lengths(&lengths, 16, false)?
                };
            } else {
                let lengths = short_tree(&mut self.bits, 19, 5, true)?;
                self.literals = literal_tree(&mut self.bits, &lengths)?;
                self.positions = short_tree(
                    &mut self.bits,
                    self.position_symbols,
                    self.position_bits,
                    false,
                )?;
            }
        }
        self.commands -= 1;
        let mut code = symbol(&self.literals, &mut self.bits)?;
        if self.method == 3 && code == 285 {
            code += self.bits.get(8)? as u16;
            if code > 509 {
                return Err(invalid());
            }
        }
        Ok(code)
    }
    fn decode(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let maximum = output.len().min(8192);
        let mut at = 0;
        while at < maximum && !self.ended {
            if self.remaining == 0 {
                if self.copy != 0 || self.commands != 0 {
                    return Err(invalid());
                }
                self.bits.finish_ignored_padding()?;
                self.ended = true;
                break;
            }
            let byte = if self.copy != 0 {
                let from = (self.position + self.dictionary.len() - self.distance)
                    & (self.dictionary.len() - 1);
                self.copy -= 1;
                self.dictionary[from]
            } else {
                let code = self.next_command()?;
                if code < 256 {
                    code as u8
                } else {
                    let count = usize::from(code) - 253;
                    if count as u64 > self.remaining {
                        return Err(invalid());
                    }
                    let position = symbol(&self.positions, &mut self.bits)?;
                    let offset = if self.method == 3 {
                        (usize::from(position) << 6) + self.bits.get(6)? as usize
                    } else if position == 0 {
                        0
                    } else {
                        (1usize << (position - 1)) + self.bits.get((position - 1) as u8)? as usize
                    };
                    self.distance = offset + 1;
                    if self.distance > self.dictionary.len() {
                        return Err(invalid());
                    }
                    self.copy = count;
                    continue;
                }
            };
            self.bits.charge(1)?;
            self.dictionary[self.position] = byte;
            self.position = (self.position + 1) & (self.dictionary.len() - 1);
            self.remaining -= 1;
            output[at] = byte;
            at += 1;
        }
        Ok(at)
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for LzhReader<R, P> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        match self.decode(output) {
            Ok(n) => Ok(n),
            Err(error) => {
                self.failed = true;
                Err(preserve_io_error(error))
            }
        }
    }
}
