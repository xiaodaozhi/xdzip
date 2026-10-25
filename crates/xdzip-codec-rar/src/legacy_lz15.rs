//! Original adaptive RAR15-family decoder from the frozen mathematical facts.
use crate::algorithm::WorkOutput;
use crate::index::Descriptor;
use crate::legacy_stream::{Bits, Delivery, History};
use crate::{MEMORY_LIMIT, decode_error};
use std::io::Read;
use xdzip_codec_api::{Cancellation, payload::DecodeError};
#[path = "legacy_lz15_codes.rs"]
mod codes;
use codes::Books;

#[derive(Clone, Copy, Default)]
struct Item {
    value: u8,
    score: u8,
}

#[cfg(test)]
#[path = "legacy_lz15_tests.rs"]
mod tests;
struct Ranking {
    items: [Item; 256],
    positions: [u8; 256],
}
impl Ranking {
    fn new(flags: bool) -> Self {
        Self {
            items: std::array::from_fn(|i| Item {
                value: if flags {
                    (i as u8).wrapping_neg()
                } else {
                    i as u8
                },
                score: 0,
            }),
            positions: [0; 256],
        }
    }
    fn rescale(&mut self) {
        for (rank, item) in self.items.iter_mut().enumerate() {
            item.score = 7 - (rank / 32) as u8;
        }
        self.positions.fill(0);
        for (i, position) in self.positions[..7].iter_mut().enumerate() {
            *position = (224 - i * 32) as u8;
        }
    }
    fn select(&mut self, rank: usize, literal: bool) -> Result<(u8, u8), DecodeError> {
        if rank >= 256 {
            return Err(DecodeError::Integrity);
        }
        for _ in 0..2 {
            let item = self.items[rank];
            let target = usize::from(self.positions[usize::from(item.score)]);
            self.positions[usize::from(item.score)] =
                self.positions[usize::from(item.score)].wrapping_add(1);
            let packed = ((u16::from(item.value) << 8) | u16::from(item.score)).wrapping_add(1);
            let candidate = Item {
                value: (packed >> 8) as u8,
                score: packed as u8,
            };
            if (literal && candidate.score > 161) || (!literal && candidate.score == 0) {
                self.rescale();
                continue;
            }
            self.items[rank] = self.items[target];
            self.items[target] = candidate;
            return Ok((item.value, candidate.value));
        }
        Err(DecodeError::Integrity)
    }
}

enum Action {
    Byte(u8),
    Copy(u64, usize),
    None,
}

pub(crate) struct Lz15 {
    history: History,
    books: Option<Books>,
    literal: Ranking,
    distance: Ranking,
    flags: Ranking,
    near: [u8; 256],
    recent: [Option<u64>; 4],
    last: Option<(u64, usize)>,
    average: u32,
    average1: u32,
    average2: u32,
    average3: u32,
    distance_average: u32,
    threshold: u64,
    huffman_weight: u16,
    long_weight: u16,
    consecutive: u16,
    short_variant: bool,
    special: bool,
    repeats: u8,
    flag_byte: u8,
    flag_left: u8,
    initialized: bool,
    failed: bool,
}
impl Default for Lz15 {
    fn default() -> Self {
        let mut distance = Ranking::new(false);
        distance.rescale();
        Self {
            history: History::default(),
            books: None,
            literal: Ranking::new(false),
            distance,
            flags: Ranking::new(true),
            near: std::array::from_fn(|i| i as u8),
            recent: [None; 4],
            last: None,
            average: 0x3500,
            average1: 0,
            average2: 0,
            average3: 0,
            distance_average: 0,
            threshold: 0x2001,
            huffman_weight: 128,
            long_weight: 128,
            consecutive: 0,
            short_variant: false,
            special: false,
            repeats: 0,
            flag_byte: 0,
            flag_left: 0,
            initialized: false,
            failed: false,
        }
    }
}
impl Lz15 {
    pub fn reset(&mut self) {
        self.history.clear();
        self.initialized = false;
        self.failed = false;
    }
    pub fn memory_required(&self, d: &Descriptor) -> Result<u64, DecodeError> {
        if !matches!(d.method, 13 | 15)
            || !(1..=5).contains(&d.compression)
            || d.encrypted
            || d.hash.is_some()
        {
            return Err(DecodeError::Unsupported);
        }
        (288u64 * 1024 * 1024)
            .checked_add(self.history.len().max(65536) as u64)
            .filter(|n| *n <= MEMORY_LIMIT)
            .ok_or(DecodeError::Limit)
    }
    fn book(&self, table: usize, input: &mut Bits<'_>) -> Result<u16, DecodeError> {
        self.books
            .as_ref()
            .ok_or(DecodeError::Integrity)?
            .number(table, input)
    }
    fn begin(
        &mut self,
        d: &Descriptor,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.memory_required(d)?;
        if d.solid && !self.initialized {
            return Err(DecodeError::Unsupported);
        }
        if self.books.is_none() {
            self.books = Some(Books::new()?);
        }
        if !d.solid {
            self.literal = Ranking::new(false);
            self.flags = Ranking::new(true);
            self.distance = Ranking::new(false);
            self.distance.rescale();
            self.near = std::array::from_fn(|i| i as u8);
            self.recent = [None; 4];
            self.last = None;
            self.average = 0x3500;
            self.average1 = 0;
            self.average2 = 0;
            self.average3 = 0;
            self.distance_average = 0;
            self.threshold = 0x2001;
            self.huffman_weight = 128;
            self.long_weight = 128;
            self.consecutive = 0;
            self.short_variant = false;
        }
        self.history.prepare(65536, d.solid, sink, control)?;
        self.flag_byte = 0;
        self.flag_left = 0;
        self.repeats = 0;
        self.special = false;
        self.initialized = true;
        Ok(())
    }
    fn flag(&mut self, input: &mut Bits<'_>) -> Result<bool, DecodeError> {
        if self.flag_left == 0 {
            let rank = usize::from(self.book(4, input)?);
            self.flag_byte = self.flags.select(rank, false)?.0;
            self.flag_left = 8;
        }
        let value = self.flag_byte & 128 != 0;
        self.flag_byte <<= 1;
        self.flag_left -= 1;
        Ok(value)
    }
    fn huffman(&mut self, input: &mut Bits<'_>) -> Result<Action, DecodeError> {
        let table = if self.average > 0x75ff {
            6
        } else if self.average > 0x5dff {
            5
        } else if self.average > 0x35ff {
            4
        } else if self.average > 0x0dff {
            3
        } else {
            2
        };
        let value = self.book(table, input)?;
        let rank = if self.special {
            // Every zero codeword occupies <=0x0fff of the 16-bit prefix
            // interval, and codeword256 occupies its high interval. Thus the
            // frozen preview test is exactly represented by the decoded value.
            if value == 0 {
                if input.get(1)? != 0 {
                    self.special = false;
                    self.consecutive = 0;
                    return Ok(Action::None);
                }
                let length = 3 + input.get(1)? as usize;
                let distance = 32 * u64::from(self.book(4, input)?) + input.get(5)?;
                return Ok(Action::Copy(distance, length));
            }
            usize::from(value - 1)
        } else {
            if self.consecutive >= 16 && self.flag_left == 0 {
                self.special = true;
            }
            self.consecutive += 1;
            usize::from(value as u8)
        };
        let byte = self.literal.select(rank, true)?.0;
        let sum = self.average + rank as u32;
        self.average = sum - (sum >> 8);
        self.huffman_weight += 16;
        if self.huffman_weight > 255 {
            self.huffman_weight = 144;
            self.long_weight /= 2;
        }
        Ok(Action::Byte(byte))
    }
    fn remember(&mut self, distance: u64, length: usize, recent: bool) -> Action {
        if recent {
            self.recent.copy_within(..3, 1);
            self.recent[0] = Some(distance);
        }
        self.last = Some((distance, length));
        Action::Copy(distance, length)
    }
    fn repeat(&self) -> Result<Action, DecodeError> {
        let (distance, length) = self.last.ok_or(DecodeError::Integrity)?;
        Ok(Action::Copy(distance, length))
    }
    fn short(&mut self, input: &mut Bits<'_>) -> Result<Action, DecodeError> {
        self.consecutive = 0;
        if self.repeats == 2 {
            if input.get(1)? != 0 {
                return self.repeat();
            }
            self.repeats = 0;
        }
        let mode = usize::from(self.average1 >= 37);
        let slot = self.books.as_ref().ok_or(DecodeError::Integrity)?.short(
            mode,
            self.short_variant,
            input,
        )?;
        match slot {
            9 => {
                self.repeats += 1;
                self.repeat()
            }
            14 => {
                self.repeats = 0;
                let length = usize::from(self.book(1, input)?) + 5;
                let distance = 32768 + input.get(15)?;
                Ok(self.remember(distance, length, false))
            }
            10..=13 => {
                self.repeats = 0;
                let distance = self.recent[usize::from(slot - 10)].ok_or(DecodeError::Integrity)?;
                let mut length = usize::from(self.book(0, input)?) + 2;
                if slot == 10 && length == 257 {
                    self.short_variant = !self.short_variant;
                    return Ok(Action::None);
                }
                length += usize::from(distance > 256) + usize::from(distance >= self.threshold);
                Ok(self.remember(distance, length, true))
            }
            0..=8 => {
                self.repeats = 0;
                let sum = self.average1 + u32::from(slot);
                self.average1 = sum - (sum >> 4);
                let rank = usize::from(self.book(4, input)? as u8);
                let distance = u64::from(self.near[rank]) + 1;
                if rank != 0 {
                    self.near.swap(rank, rank - 1);
                }
                Ok(self.remember(distance, usize::from(slot) + 2, true))
            }
            _ => Err(DecodeError::Integrity),
        }
    }
    fn long(&mut self, input: &mut Bits<'_>) -> Result<Action, DecodeError> {
        self.consecutive = 0;
        self.long_weight += 16;
        if self.long_weight > 255 {
            self.long_weight = 144;
            self.huffman_weight /= 2;
        }
        let previous_average2 = self.average2;
        let value = if self.average2 >= 122 {
            self.book(1, input)?
        } else if self.average2 >= 64 {
            self.book(0, input)?
        } else {
            let mut zeros = 0;
            while zeros < 8 && input.get(1)? == 0 {
                zeros += 1;
            }
            if zeros == 8 {
                input.get(8)? as u16
            } else {
                zeros
            }
        };
        let sum = self.average2 + u32::from(value);
        self.average2 = sum - (sum >> 5);
        let table = if self.distance_average > 0x28ff {
            4
        } else if self.distance_average > 0x06ff {
            3
        } else {
            2
        };
        let rank = self.book(table, input)?;
        let sum = self.distance_average + u32::from(rank);
        self.distance_average = sum - (sum >> 8);
        let high = self.distance.select(usize::from(rank as u8), false)?.1;
        let distance = 128 * u64::from(high) + input.get(7)?;
        let previous_average3 = self.average3;
        if value != 1 && value != 4 {
            if value == 0 && distance <= self.threshold {
                let sum = self.average3 + 1;
                self.average3 = sum - (sum >> 8);
            } else {
                self.average3 = self.average3.saturating_sub(1);
            }
        }
        let length = usize::from(value)
            + 3
            + usize::from(distance >= self.threshold)
            + 8 * usize::from(distance <= 256);
        self.threshold =
            if previous_average3 > 0xb0 || (self.average >= 0x2a00 && previous_average2 < 0x40) {
                0x7f00
            } else {
                0x2001
            };
        Ok(self.remember(distance, length, true))
    }
    fn action(&mut self, input: &mut Bits<'_>) -> Result<Action, DecodeError> {
        if self.special {
            return self.huffman(input);
        }
        let preferred = self.long_weight > self.huffman_weight;
        let long = if self.flag(input)? {
            preferred
        } else if self.flag(input)? {
            !preferred
        } else {
            return self.short(input);
        };
        if long {
            self.long(input)
        } else {
            self.huffman(input)
        }
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
        if padding > 15 {
            return Err(DecodeError::Unsupported);
        }
        let expected = d.size.ok_or(DecodeError::Unsupported)?;
        let mut output = Delivery::new(expected);
        let result = (|| {
            self.begin(d, sink, control)?;
            let mut bits = Bits::new(input);
            while output.remaining() != 0 {
                control.check().map_err(decode_error)?;
                match self.action(&mut bits)? {
                    Action::Byte(byte) => output.byte(byte, &mut self.history, sink, control)?,
                    Action::Copy(distance, length) => {
                        output.copy(distance, length, &mut self.history, sink, control)?
                    }
                    Action::None => (),
                }
            }
            bits.drain(control)?;
            output.finish(sink, control)
        })();
        if result.is_err() {
            self.failed = true;
            output.settle_error(sink)?;
        }
        result
    }
}
