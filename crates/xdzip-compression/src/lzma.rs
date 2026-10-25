//! Raw LZMA, implemented from the LZMA wire grammar and mathematical facts.
//! The dictionary and model arena are admitted before allocation. No reference
//! implementation is linked or used by the production decoder.
use crate::invalid;
pub(crate) use crate::stream_io::{
    Input, READ_QUANTUM, filled, preserve_io_error, resource, retryable_io,
};
use std::io::{self, Read};

const FIXED_MEMORY: u64 = 32768;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Properties {
    pub(crate) lc: u8,
    pub(crate) lp: u8,
    pub(crate) pb: u8,
}
impl Properties {
    pub(crate) fn parse(value: u8) -> io::Result<Self> {
        if value >= 225 {
            return Err(invalid());
        }
        Ok(Self {
            lc: value % 9,
            lp: (value / 9) % 5,
            pb: value / 45,
        })
    }
    pub(crate) fn contexts(self) -> usize {
        1usize << (self.lc + self.lp)
    }
}

fn raw_properties(bytes: &[u8]) -> io::Result<(Properties, usize)> {
    let [property, a, b, c, d] = *bytes else {
        return Err(invalid());
    };
    let dictionary = u32::from_le_bytes([a, b, c, d]).max(4096);
    Ok((Properties::parse(property)?, dictionary as usize))
}

/// Conservative per-reader allocation admission, including dictionary, all
/// probability contexts and bounded input/state storage. Properties must be
/// exactly five bytes; the result is an implementation budget, not a wire limit.
pub fn lzma_memory_required(properties: &[u8]) -> io::Result<u64> {
    let (properties, dictionary) = raw_properties(properties)?;
    Ok(memory_required(dictionary, properties.contexts()))
}

pub(crate) fn memory_required(dictionary: usize, contexts: usize) -> u64 {
    dictionary as u64 + contexts as u64 * 768 * 2 + FIXED_MEMORY
}

pub(crate) struct Range {
    width: u32,
    code: u32,
    boundary: u64,
}
impl Range {
    pub(crate) fn new<R: Read>(input: &mut Input<R>, boundary: u64) -> io::Result<Self> {
        if input.byte(boundary)? != 0 {
            return Err(invalid());
        }
        let mut code = 0u32;
        for _ in 0..4 {
            code = (code << 8) | u32::from(input.byte(boundary)?);
        }
        if code == u32::MAX {
            return Err(invalid());
        }
        Ok(Self {
            width: u32::MAX,
            code,
            boundary,
        })
    }
    fn normalize<R: Read>(&mut self, input: &mut Input<R>) -> io::Result<()> {
        if self.width < 1 << 24 {
            self.width <<= 8;
            self.code = (self.code << 8) | u32::from(input.byte(self.boundary)?);
        }
        if self.code >= self.width {
            return Err(invalid());
        }
        Ok(())
    }
    pub(crate) fn bit<R: Read>(
        &mut self,
        input: &mut Input<R>,
        probability: &mut u16,
    ) -> io::Result<u32> {
        let split = (self.width >> 11) * u32::from(*probability);
        let bit = if self.code < split {
            self.width = split;
            *probability += (2048 - *probability) >> 5;
            0
        } else {
            self.width -= split;
            self.code -= split;
            *probability -= *probability >> 5;
            1
        };
        self.normalize(input)?;
        Ok(bit)
    }
    fn direct<R: Read>(&mut self, input: &mut Input<R>, count: u8) -> io::Result<u32> {
        let mut value = 0;
        for _ in 0..count {
            self.width >>= 1;
            let bit = u32::from(self.code >= self.width);
            if bit != 0 {
                self.code -= self.width;
            }
            value = (value << 1) | bit;
            self.normalize(input)?;
        }
        Ok(value)
    }
    pub(crate) fn tree<R: Read>(
        &mut self,
        input: &mut Input<R>,
        models: &mut [u16],
        bits: u8,
        reverse: bool,
    ) -> io::Result<u32> {
        let mut prefix = 1usize;
        let mut value = 0u32;
        for index in 0..bits {
            let bit = self.bit(input, &mut models[prefix])?;
            prefix = prefix * 2 + bit as usize;
            if reverse {
                value |= bit << index;
            } else {
                value = (value << 1) | bit;
            }
        }
        Ok(value)
    }
    pub(crate) fn complete<R>(&self, input: &Input<R>) -> bool {
        self.code == 0 && input.consumed == self.boundary
    }
}

#[derive(Clone)]
pub(crate) struct Lengths {
    pub(crate) choice: [u16; 2],
    pub(crate) low: [[u16; 8]; 16],
    pub(crate) middle: [[u16; 8]; 16],
    pub(crate) high: [u16; 256],
}
impl Lengths {
    fn new() -> Self {
        Self {
            choice: [1024; 2],
            low: [[1024; 8]; 16],
            middle: [[1024; 8]; 16],
            high: [1024; 256],
        }
    }
    fn decode<R: Read>(
        &mut self,
        range: &mut Range,
        input: &mut Input<R>,
        position: usize,
    ) -> io::Result<usize> {
        Ok(if range.bit(input, &mut self.choice[0])? == 0 {
            2 + range.tree(input, &mut self.low[position], 3, false)? as usize
        } else if range.bit(input, &mut self.choice[1])? == 0 {
            10 + range.tree(input, &mut self.middle[position], 3, false)? as usize
        } else {
            18 + range.tree(input, &mut self.high, 8, false)? as usize
        })
    }
}

#[derive(Clone)]
pub(crate) struct Models {
    pub(crate) properties: Properties,
    pub(crate) literals: Vec<u16>,
    pub(crate) is_match: [[u16; 16]; 12],
    pub(crate) is_rep: [u16; 12],
    pub(crate) rep_g0: [u16; 12],
    pub(crate) rep_g1: [u16; 12],
    pub(crate) rep_g2: [u16; 12],
    pub(crate) rep_long: [[u16; 16]; 12],
    pub(crate) lengths: [Lengths; 2],
    pub(crate) slots: [[u16; 64]; 4],
    pub(crate) distances: [[u16; 32]; 10],
    pub(crate) alignment: [u16; 16],
    pub(crate) state: usize,
    pub(crate) reps: [usize; 4],
}
impl Models {
    pub(crate) fn new(properties: Properties) -> io::Result<Self> {
        Ok(Self {
            properties,
            literals: filled(properties.contexts() * 768, 1024)?,
            is_match: [[1024; 16]; 12],
            is_rep: [1024; 12],
            rep_g0: [1024; 12],
            rep_g1: [1024; 12],
            rep_g2: [1024; 12],
            rep_long: [[1024; 16]; 12],
            lengths: [Lengths::new(), Lengths::new()],
            slots: [[1024; 64]; 4],
            distances: [[1024; 32]; 10],
            alignment: [1024; 16],
            state: 0,
            reps: [1; 4],
        })
    }
    pub(crate) fn reset(&mut self, properties: Properties) -> io::Result<()> {
        // Reuse the already admitted maximum arena; changing lc/lp cannot
        // allocate beyond the caller's admission.
        let count = properties.contexts() * 768;
        if count > self.literals.capacity() {
            self.literals
                .try_reserve_exact(count - self.literals.len())
                .map_err(|_| resource())?;
        }
        self.literals.resize(count, 1024);
        self.literals.fill(1024);
        self.properties = properties;
        self.is_match.fill([1024; 16]);
        self.is_rep.fill(1024);
        self.rep_g0.fill(1024);
        self.rep_g1.fill(1024);
        self.rep_g2.fill(1024);
        self.rep_long.fill([1024; 16]);
        self.lengths = [Lengths::new(), Lengths::new()];
        self.slots.fill([1024; 64]);
        self.distances.fill([1024; 32]);
        self.alignment.fill(1024);
        self.state = 0;
        self.reps = [1; 4];
        Ok(())
    }
    pub(crate) fn position_state(&self, position: u64) -> usize {
        (position & ((1 << self.properties.pb) - 1)) as usize
    }
    pub(crate) fn literal_context(&self, position: u64, previous: u8) -> usize {
        (((position & ((1 << self.properties.lp) - 1)) as usize) << self.properties.lc)
            + (usize::from(previous) >> (8 - self.properties.lc))
    }
    pub(crate) fn literal_state(&mut self) {
        self.state = match self.state {
            0..=3 => 0,
            4..=9 => self.state - 3,
            _ => self.state - 6,
        };
    }
    pub(crate) fn select_rep(&mut self, index: usize) -> usize {
        let distance = self.reps[index];
        self.reps[..=index].rotate_right(1);
        distance
    }
}

pub(crate) struct Dictionary {
    bytes: Vec<u8>,
    cursor: usize,
    valid: usize,
    pub(crate) position: u64,
    pub(crate) previous: u8,
}
impl Dictionary {
    pub(crate) fn new(size: usize) -> io::Result<Self> {
        Ok(Self {
            bytes: filled(size, 0)?,
            cursor: 0,
            valid: 0,
            position: 0,
            previous: 0,
        })
    }
    pub(crate) fn reset(&mut self) {
        self.cursor = 0;
        self.valid = 0;
        self.position = 0;
        self.previous = 0;
    }
    pub(crate) fn get(&self, distance: usize) -> io::Result<u8> {
        if distance == 0 || distance > self.valid {
            return Err(invalid());
        }
        let index = if distance > self.cursor {
            self.bytes.len() - (distance - self.cursor)
        } else {
            self.cursor - distance
        };
        Ok(self.bytes[index])
    }
    pub(crate) fn push(&mut self, byte: u8) -> io::Result<()> {
        self.position = self.position.checked_add(1).ok_or_else(invalid)?;
        self.bytes[self.cursor] = byte;
        self.cursor += 1;
        if self.cursor == self.bytes.len() {
            self.cursor = 0;
        }
        self.valid = self.valid.saturating_add(1).min(self.bytes.len());
        self.previous = byte;
        Ok(())
    }
}

pub(crate) enum Token {
    Literal(u8),
    Match { distance: usize, length: usize },
    End,
}

pub(crate) fn token<R: Read>(
    models: &mut Models,
    dictionary: &Dictionary,
    range: &mut Range,
    input: &mut Input<R>,
) -> io::Result<Token> {
    let state = models.state;
    let pos = models.position_state(dictionary.position);
    if range.bit(input, &mut models.is_match[state][pos])? == 0 {
        let context = models.literal_context(dictionary.position, dictionary.previous);
        let tree = &mut models.literals[context * 768..(context + 1) * 768];
        let mut prefix = 1usize;
        let mut matched = state >= 7;
        let match_byte = if matched {
            dictionary.get(models.reps[0])?
        } else {
            0
        };
        for shift in (0..8).rev() {
            let prediction = (match_byte >> shift) & 1;
            let offset = if matched {
                (1 + prediction as usize) * 256
            } else {
                0
            };
            let bit = range.bit(input, &mut tree[offset + prefix])? as u8;
            prefix = prefix * 2 + usize::from(bit);
            matched &= prediction == bit;
        }
        models.literal_state();
        return Ok(Token::Literal((prefix - 256) as u8));
    }
    if range.bit(input, &mut models.is_rep[state])? != 0 {
        let index = if range.bit(input, &mut models.rep_g0[state])? == 0 {
            if range.bit(input, &mut models.rep_long[state][pos])? == 0 {
                models.state = if state < 7 { 9 } else { 11 };
                dictionary.get(models.reps[0])?;
                return Ok(Token::Match {
                    distance: models.reps[0],
                    length: 1,
                });
            }
            0
        } else if range.bit(input, &mut models.rep_g1[state])? == 0 {
            1
        } else if range.bit(input, &mut models.rep_g2[state])? == 0 {
            2
        } else {
            3
        };
        let distance = models.select_rep(index);
        let length = models.lengths[1].decode(range, input, pos)?;
        models.state = if state < 7 { 8 } else { 11 };
        dictionary.get(distance)?;
        return Ok(Token::Match { distance, length });
    }
    let length = models.lengths[0].decode(range, input, pos)?;
    let slot = range.tree(input, &mut models.slots[(length - 2).min(3)], 6, false)?;
    let zero_based = if slot < 4 {
        slot
    } else {
        let bits = (slot / 2 - 1) as u8;
        let base = (2 + (slot & 1)) << bits;
        base + if slot < 14 {
            range.tree(
                input,
                &mut models.distances[(slot - 4) as usize],
                bits,
                true,
            )?
        } else {
            (range.direct(input, bits - 4)? << 4)
                + range.tree(input, &mut models.alignment, 4, true)?
        }
    };
    if zero_based == u32::MAX {
        return Ok(Token::End);
    }
    let distance = zero_based as usize + 1;
    dictionary.get(distance)?;
    models.reps.rotate_right(1);
    models.reps[0] = distance;
    models.state = if state < 7 { 7 } else { 10 };
    Ok(Token::Match { distance, length })
}

/// Exact-span raw LZMA1 reader. A known output size permits either a correctly
/// terminated EOS or a no-EOS range ending; unknown size requires EOS. Consumers
/// must drain to EOF or call `finish` after reading the full expected output.
/// Each nonempty `read` emits at most 8 KiB, bounding cancellation latency.
pub struct LzmaReader<R> {
    input: Input<R>,
    dictionary: Dictionary,
    models: Models,
    range: Option<Range>,
    expected: Option<u64>,
    decoded: u64,
    pending: (usize, usize),
    ended: bool,
    failed: bool,
}
impl<R: Read> LzmaReader<R> {
    /// Admit dictionary and model memory before allocation. The five-byte
    /// properties and packed/output lengths belong to the authorizing container.
    pub fn new(
        source: R,
        packed: u64,
        expected: Option<u64>,
        properties: &[u8],
        memory_limit: u64,
    ) -> io::Result<Self> {
        let (properties, dictionary) = raw_properties(properties)?;
        if memory_required(dictionary, properties.contexts()) > memory_limit {
            return Err(resource());
        }
        Ok(Self {
            input: Input::new(source, packed),
            dictionary: Dictionary::new(dictionary)?,
            models: Models::new(properties)?,
            range: None,
            expected,
            decoded: 0,
            pending: (0, 0),
            ended: false,
            failed: false,
        })
    }
    /// Logical compressed bytes consumed, excluding buffered read-ahead.
    pub fn consumed_bytes(&self) -> u64 {
        self.input.consumed
    }
    /// Bytes returned to the consumer, including a partially emitted match.
    pub fn decoded_bytes(&self) -> u64 {
        self.decoded
    }
    fn initialize(&mut self) -> io::Result<()> {
        if self.range.is_none() {
            let boundary = self.input.packed;
            self.range = Some(Range::new(&mut self.input, boundary)?);
        }
        Ok(())
    }
    fn terminal(&mut self) -> io::Result<()> {
        self.initialize()?;
        if self.pending.1 != 0 || self.expected.is_some_and(|size| size != self.decoded) {
            return Err(invalid());
        }
        let range = self.range.as_mut().ok_or_else(invalid)?;
        if self.expected.is_some() && range.complete(&self.input) {
            self.ended = true;
            return Ok(());
        }
        if !matches!(
            token(&mut self.models, &self.dictionary, range, &mut self.input)?,
            Token::End
        ) || !range.complete(&self.input)
        {
            return Err(invalid());
        }
        self.ended = true;
        Ok(())
    }
    /// Validate EOF without discarding unread output. An early finish poisons
    /// the reader, as does any input or integrity error.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(());
        }
        let result = if self.expected.is_none() {
            Err(invalid())
        } else {
            self.terminal()
        };
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.initialize()?;
        let limit = output.len().min(READ_QUANTUM);
        let mut count = 0;
        while count < limit {
            if self.expected == Some(self.decoded) {
                self.terminal()?;
                break;
            }
            let byte = if self.pending.1 != 0 {
                self.pending.1 -= 1;
                self.dictionary.get(self.pending.0)?
            } else {
                let range = self.range.as_mut().ok_or_else(invalid)?;
                match token(&mut self.models, &self.dictionary, range, &mut self.input)? {
                    Token::Literal(byte) => byte,
                    Token::Match { distance, length } => {
                        if self
                            .expected
                            .is_some_and(|size| length as u64 > size - self.decoded)
                        {
                            return Err(invalid());
                        }
                        self.pending = (distance, length - 1);
                        self.dictionary.get(distance)?
                    }
                    Token::End => {
                        if self.expected.is_some() || !range.complete(&self.input) {
                            return Err(invalid());
                        }
                        self.ended = true;
                        break;
                    }
                }
            };
            self.dictionary.push(byte)?;
            output[count] = byte;
            self.decoded = self.decoded.checked_add(1).ok_or_else(invalid)?;
            count += 1;
        }
        Ok(count)
    }
}
impl<R: Read> Read for LzmaReader<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(0);
        }
        let result = self.read_inner(output);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
