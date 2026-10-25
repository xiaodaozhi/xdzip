//! Independent RAR20/26 known-size LZ/audio candidate over shared host streams.
//! No RAR compression encoder or third-party algorithm implementation is used.

use crate::algorithm::WorkOutput;
use crate::index::Descriptor;
use crate::legacy_audio20::Audio20;
use crate::legacy_stream::{Bits, Delivery, History};
use crate::lz_bits::{BitRead, Huffman};
use crate::{FILE_LIMIT, MEMORY_LIMIT, decode_error};
use std::io::Read;
use xdzip_codec_api::{Cancellation, payload::DecodeError};

// Retained metadata and all surrounding bounded codec state share the worker's
// memory ceiling. This allowance is policy, not a format dictionary maximum.
const OTHER_MEMORY: u64 = 288 * 1024 * 1024;
const MAX_WINDOW: usize = 1024 * 1024;

enum Tables {
    Normal {
        main: Box<Huffman>,
        distance: Box<Huffman>,
        repeat: Box<Huffman>,
    },
    Audio(Vec<Huffman>),
}
enum Action {
    Byte(u8),
    Copy(u64, usize),
    Table,
}

pub(crate) struct Lz20 {
    history: History,
    tables: Option<Tables>,
    baseline: [u8; 1028],
    recent: [u64; 4],
    last_distance: u64,
    last_length: usize,
    audio: Audio20,
    failed: bool,
}
impl Default for Lz20 {
    fn default() -> Self {
        Self {
            history: History::default(),
            tables: None,
            baseline: [0; 1028],
            recent: [0; 4],
            last_distance: 0,
            last_length: 0,
            audio: Audio20::default(),
            failed: false,
        }
    }
}
impl Lz20 {
    pub fn reset(&mut self) {
        self.history.clear();
        self.tables = None;
        self.baseline.fill(0);
        self.recent.fill(0);
        self.last_distance = 0;
        self.last_length = 0;
        self.audio.reset();
        // Poison is not a format reset. A failed payload needs a new instance.
    }
    fn window(d: &Descriptor) -> Result<usize, DecodeError> {
        if !matches!(d.method, 20 | 26)
            || !(1..=5).contains(&d.compression)
            || d.encrypted
            || d.hash.is_some()
        {
            return Err(DecodeError::Unsupported);
        }
        let window = usize::try_from(d.dictionary.max(65536)).map_err(|_| DecodeError::Limit)?;
        if window > MAX_WINDOW || !window.is_power_of_two() {
            return Err(DecodeError::Unsupported);
        }
        Ok(window)
    }
    pub fn memory_required(&self, d: &Descriptor) -> Result<u64, DecodeError> {
        let window = Self::window(d)?.max(self.history.len());
        (window as u64)
            .checked_add(OTHER_MEMORY)
            .filter(|n| *n <= MEMORY_LIMIT)
            .ok_or(DecodeError::Limit)
    }
    fn begin(
        &mut self,
        d: &Descriptor,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.memory_required(d)?;
        if !d.solid {
            self.reset();
        }
        self.history
            .prepare(Self::window(d)?, d.solid, sink, control)
    }
    fn read_table<B: BitRead + ?Sized>(
        &mut self,
        bits: &mut B,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        let audio = bits.get(1)? != 0;
        let retain = bits.get(1)? != 0;
        let channels = if audio { bits.get(2)? as usize + 1 } else { 0 };
        let count = if audio { channels * 257 } else { 374 };
        sink.reserve_aux((1028 + 19 + count) as u64)?;
        let mut done = 0u64;
        let result = (|| {
            // Full-capacity baseline survives shorter retained table updates.
            let mut next = if retain { self.baseline } else { [0u8; 1028] };
            done += next.len() as u64;
            let mut meta = [0u8; 19];
            for length in &mut meta {
                *length = bits.get(4)? as u8;
                done += 1;
            }
            let meta = Huffman::new(&meta)?;
            let mut at = 0usize;
            while at < count {
                control.check().map_err(decode_error)?;
                let symbol = meta.symbol(bits)?;
                match symbol {
                    0..=15 => {
                        next[at] = (next[at] + symbol as u8) & 15;
                        at += 1;
                        done += 1;
                    }
                    16..=18 => {
                        let (length, value) = match symbol {
                            16 => {
                                if at == 0 {
                                    return Err(DecodeError::Integrity);
                                }
                                (bits.get(2)? as usize + 3, next[at - 1])
                            }
                            17 => (bits.get(3)? as usize + 3, 0),
                            18 => (bits.get(7)? as usize + 11, 0),
                            _ => unreachable!(),
                        };
                        let end = at
                            .checked_add(length)
                            .filter(|&end| end <= count)
                            .ok_or(DecodeError::Integrity)?;
                        next[at..end].fill(value);
                        done += length as u64;
                        at = end;
                    }
                    _ => return Err(DecodeError::Integrity),
                }
            }
            let tables = if audio {
                let mut trees = Vec::with_capacity(channels);
                for lengths in next[..count].chunks(257) {
                    trees.push(Huffman::new(lengths)?);
                }
                Tables::Audio(trees)
            } else {
                Tables::Normal {
                    main: Box::new(Huffman::new(&next[..298])?),
                    distance: Box::new(Huffman::new(&next[298..346])?),
                    repeat: Box::new(Huffman::new(&next[346..374])?),
                }
            };
            Ok((next, tables))
        })();
        // Both paths settle only completed work, without delivery credit. A
        // failed table cannot commit partial baseline or predictor selection.
        let settled = sink.produced_aux(done);
        let (next, tables) = result.and_then(|value| settled.map(|()| value))?;
        if audio {
            self.audio.configure(channels)?;
        }
        self.baseline = next;
        self.tables = Some(tables);
        Ok(())
    }
    fn symbol<B: BitRead + ?Sized>(&self, bits: &mut B) -> Result<(u16, bool), DecodeError> {
        match self.tables.as_ref().ok_or(DecodeError::Integrity)? {
            Tables::Normal { main, .. } => Ok((main.symbol(bits)?, false)),
            Tables::Audio(trees) => Ok((
                trees
                    .get(self.audio.channel())
                    .ok_or(DecodeError::Integrity)?
                    .symbol(bits)?,
                true,
            )),
        }
    }
    fn length<B: BitRead + ?Sized>(slot: u16, bits: &mut B) -> Result<usize, DecodeError> {
        if slot >= 28 {
            return Err(DecodeError::Integrity);
        }
        if slot < 8 {
            return Ok(usize::from(slot));
        }
        let extra = (slot / 4 - 1) as u8;
        Ok(((4 + usize::from(slot % 4)) << extra) + bits.get(extra)? as usize)
    }
    fn distance<B: BitRead + ?Sized>(slot: u16, bits: &mut B) -> Result<u64, DecodeError> {
        let (base, extra) = match slot {
            0..=3 => (u64::from(slot), 0),
            4..=33 => {
                let extra = (slot / 2 - 1) as u8;
                ((2 + u64::from(slot % 2)) << extra, extra)
            }
            34..=47 => (131072 + u64::from(slot - 34) * 65536, 16),
            _ => return Err(DecodeError::Integrity),
        };
        Ok(base + 1 + bits.get(extra)?)
    }
    fn action<B: BitRead + ?Sized>(&mut self, bits: &mut B) -> Result<Action, DecodeError> {
        let (symbol, audio) = self.symbol(bits)?;
        if audio {
            return match symbol {
                0..=255 => Ok(Action::Byte(self.audio.sample(symbol as u8))),
                256 => Ok(Action::Table),
                _ => Err(DecodeError::Integrity),
            };
        }
        Ok(match symbol {
            0..=255 => Action::Byte(symbol as u8),
            256 => {
                if self.last_length == 0 || self.last_distance == 0 {
                    return Err(DecodeError::Integrity);
                }
                Action::Copy(self.last_distance, self.last_length)
            }
            257..=260 => {
                let distance = self.recent[usize::from(symbol - 257)];
                if distance == 0 {
                    return Err(DecodeError::Integrity);
                }
                let Tables::Normal { repeat, .. } =
                    self.tables.as_ref().ok_or(DecodeError::Integrity)?
                else {
                    return Err(DecodeError::Integrity);
                };
                let mut length = Self::length(repeat.symbol(bits)?, bits)? + 2;
                length += usize::from(distance >= 257)
                    + usize::from(distance >= 8192)
                    + usize::from(distance >= 262144);
                Action::Copy(distance, length)
            }
            261..=268 => {
                let (base, extra) = match symbol - 261 {
                    0 => (0, 2),
                    1 => (4, 2),
                    2 => (8, 3),
                    3 => (16, 4),
                    4 => (32, 5),
                    5 => (64, 6),
                    6 => (128, 6),
                    7 => (192, 6),
                    _ => unreachable!(),
                };
                Action::Copy(base + 1 + bits.get(extra)?, 2)
            }
            269 => Action::Table,
            270..=297 => {
                let mut length = Self::length(symbol - 270, bits)? + 3;
                let Tables::Normal { distance, .. } =
                    self.tables.as_ref().ok_or(DecodeError::Integrity)?
                else {
                    return Err(DecodeError::Integrity);
                };
                let distance = Self::distance(distance.symbol(bits)?, bits)?;
                length += usize::from(distance >= 8192) + usize::from(distance >= 262144);
                Action::Copy(distance, length)
            }
            _ => return Err(DecodeError::Integrity),
        })
    }
    fn remember(&mut self, distance: u64, length: usize) {
        self.recent.rotate_right(1);
        self.recent[0] = distance;
        self.last_distance = distance;
        self.last_length = length;
    }
    fn run(
        &mut self,
        d: &Descriptor,
        input: &mut dyn Read,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
        delivery: &mut Delivery,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        self.begin(d, sink, control)?;
        let mut bits = Bits::new(input);
        let expected = d.size.ok_or(DecodeError::Unsupported)?;
        if self.tables.is_none() && expected != 0 {
            self.read_table(&mut bits, sink, control)?;
        }
        while delivery.position() < expected {
            control.check().map_err(decode_error)?;
            match self.action(&mut bits)? {
                Action::Byte(value) => delivery.byte(value, &mut self.history, sink, control)?,
                Action::Copy(distance, length) => {
                    delivery.copy(distance, length, &mut self.history, sink, control)?;
                    self.remember(distance, length);
                }
                Action::Table => self.read_table(&mut bits, sink, control)?,
            }
        }
        // Known N ends content; there is no fabricated EOF symbol. A recognized
        // optional next-table marker must have a complete, valid new table.
        if self.tables.is_some() && bits.byte_tail_at_least(5)? {
            let (symbol, audio) = self.symbol(&mut bits)?;
            if (audio && symbol == 256) || (!audio && symbol == 269) {
                self.read_table(&mut bits, sink, control)?;
            }
        }
        bits.drain(control)?;
        delivery.finish(sink, control)
    }
    pub fn decode(
        &mut self,
        d: &Descriptor,
        input: &mut dyn Read,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
        padding: u8,
    ) -> Result<(), DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let expected = d.size.ok_or(DecodeError::Unsupported);
        let mut delivery = Delivery::new(expected.unwrap_or(0));
        let result = match expected {
            Err(error) => Err(error),
            Ok(n) if n > FILE_LIMIT => Err(DecodeError::Limit),
            Ok(_) if padding > 15 => Err(DecodeError::Unsupported),
            Ok(_) => self.run(d, input, sink, control, &mut delivery),
        };
        let settled = delivery.settle_error(sink);
        let result = result.and(settled);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

#[cfg(test)]
#[path = "legacy_lz20_tests.rs"]
mod tests;
