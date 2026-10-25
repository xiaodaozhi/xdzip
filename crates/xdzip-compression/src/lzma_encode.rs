//! Independent inverse range coder and bounded greedy LZMA2 compressor.
//! Parsing strategy is project-owned: 64 KiB chunks, a shared dictionary and
//! independently reset models per compressed chunk. Levels select search depth.
use crate::{
    invalid,
    lzma::{self, Dictionary, Lengths, Models, Properties},
};
use std::io::{self, Write};

const DICTIONARY: usize = 8 * 1024 * 1024;
const CHUNK: usize = 65536;
const HASHES: usize = 1 << 18;
const PROPERTY: u8 = 22;
const PROPERTIES: Properties = Properties {
    lc: 3,
    lp: 0,
    pb: 2,
};

/// Allocation admission for the LZMA2 writer: dictionary, bounded hash chains,
/// probability models, input chunk and a bounded trial entropy buffer.
pub fn lzma2_writer_memory_required(level: u8) -> io::Result<u64> {
    if ![1, 6, 9].contains(&level) {
        return Err(invalid());
    }
    Ok(lzma::memory_required(DICTIONARY, 8) + (DICTIONARY * 4 + HASHES * 8 + CHUNK * 2) as u64)
}

pub(crate) struct RangeOutput {
    low: u64,
    width: u32,
    pending: u8,
    postponed: usize,
    bytes: Vec<u8>,
}
impl RangeOutput {
    pub(crate) fn new() -> io::Result<Self> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(CHUNK)
            .map_err(|_| lzma::resource())?;
        Ok(Self {
            low: 0,
            width: u32::MAX,
            pending: 0,
            postponed: 0,
            bytes,
        })
    }
    fn emit(&mut self, byte: u8) -> io::Result<()> {
        if self.bytes.len() == CHUNK {
            return Err(io::Error::other("lzma_trial_full"));
        }
        self.bytes.push(byte);
        Ok(())
    }
    fn advance(&mut self) -> io::Result<()> {
        let carry = (self.low >> 32) as u8;
        let low = self.low as u32;
        let top = (low >> 24) as u8;
        if top != 255 || carry != 0 {
            self.emit(self.pending.wrapping_add(carry))?;
            for _ in 0..self.postponed {
                self.emit(255u8.wrapping_add(carry))?;
            }
            self.pending = top;
            self.postponed = 0;
        } else {
            self.postponed += 1;
        }
        self.low = u64::from(low & 0x00ff_ffff) << 8;
        Ok(())
    }
    fn normalize(&mut self) -> io::Result<()> {
        if self.width < 1 << 24 {
            self.width <<= 8;
            self.advance()?;
        }
        Ok(())
    }
    pub(crate) fn bit(&mut self, probability: &mut u16, bit: u32) -> io::Result<()> {
        let split = (self.width >> 11) * u32::from(*probability);
        if bit == 0 {
            self.width = split;
            *probability += (2048 - *probability) >> 5;
        } else {
            self.low += u64::from(split);
            self.width -= split;
            *probability -= *probability >> 5;
        }
        self.normalize()
    }
    fn direct(&mut self, value: u32, count: u8) -> io::Result<()> {
        for shift in (0..count).rev() {
            self.width >>= 1;
            if (value >> shift) & 1 != 0 {
                self.low += u64::from(self.width);
            }
            self.normalize()?;
        }
        Ok(())
    }
    fn tree(&mut self, models: &mut [u16], bits: u8, value: u32, reverse: bool) -> io::Result<()> {
        let mut prefix = 1usize;
        for index in 0..bits {
            let shift = if reverse { index } else { bits - index - 1 };
            let bit = (value >> shift) & 1;
            self.bit(&mut models[prefix], bit)?;
            prefix = prefix * 2 + bit as usize;
        }
        Ok(())
    }
    pub(crate) fn finish(mut self) -> io::Result<Vec<u8>> {
        for _ in 0..5 {
            self.advance()?;
        }
        Ok(self.bytes)
    }
}

fn length(
    range: &mut RangeOutput,
    models: &mut Lengths,
    pos: usize,
    length: usize,
) -> io::Result<()> {
    if length < 10 {
        range.bit(&mut models.choice[0], 0)?;
        range.tree(&mut models.low[pos], 3, (length - 2) as u32, false)
    } else {
        range.bit(&mut models.choice[0], 1)?;
        if length < 18 {
            range.bit(&mut models.choice[1], 0)?;
            range.tree(&mut models.middle[pos], 3, (length - 10) as u32, false)
        } else {
            range.bit(&mut models.choice[1], 1)?;
            range.tree(&mut models.high, 8, (length - 18) as u32, false)
        }
    }
}

fn literal(
    range: &mut RangeOutput,
    models: &mut Models,
    dictionary: &Dictionary,
    byte: u8,
) -> io::Result<()> {
    let state = models.state;
    let pos = models.position_state(dictionary.position);
    range.bit(&mut models.is_match[state][pos], 0)?;
    let context = models.literal_context(dictionary.position, dictionary.previous);
    let tree = &mut models.literals[context * 768..(context + 1) * 768];
    let mut matched = state >= 7;
    let prediction = if matched {
        dictionary.get(models.reps[0])?
    } else {
        0
    };
    let mut prefix = 1usize;
    for shift in (0..8).rev() {
        let bit = u32::from((byte >> shift) & 1);
        let guess = (prediction >> shift) & 1;
        let offset = if matched {
            (1 + guess as usize) * 256
        } else {
            0
        };
        range.bit(&mut tree[offset + prefix], bit)?;
        prefix = prefix * 2 + bit as usize;
        matched &= bit == u32::from(guess);
    }
    models.literal_state();
    Ok(())
}

fn repetition(
    range: &mut RangeOutput,
    models: &mut Models,
    position: u64,
    index: usize,
    count: usize,
) -> io::Result<()> {
    let state = models.state;
    let pos = models.position_state(position);
    range.bit(&mut models.is_match[state][pos], 1)?;
    range.bit(&mut models.is_rep[state], 1)?;
    range.bit(&mut models.rep_g0[state], u32::from(index != 0))?;
    if index == 0 {
        range.bit(&mut models.rep_long[state][pos], u32::from(count != 1))?;
        if count == 1 {
            models.state = if state < 7 { 9 } else { 11 };
            return Ok(());
        }
    } else {
        range.bit(&mut models.rep_g1[state], u32::from(index != 1))?;
        if index > 1 {
            range.bit(&mut models.rep_g2[state], u32::from(index == 3))?;
        }
    }
    length(range, &mut models.lengths[1], pos, count)?;
    models.select_rep(index);
    models.state = if state < 7 { 8 } else { 11 };
    Ok(())
}

fn distance_match(
    range: &mut RangeOutput,
    models: &mut Models,
    position: u64,
    distance: usize,
    count: usize,
) -> io::Result<()> {
    let state = models.state;
    let pos = models.position_state(position);
    range.bit(&mut models.is_match[state][pos], 1)?;
    range.bit(&mut models.is_rep[state], 0)?;
    length(range, &mut models.lengths[0], pos, count)?;
    let zero_based = (distance - 1) as u32;
    let slot = if zero_based < 4 {
        zero_based
    } else {
        let highest = 31 - zero_based.leading_zeros();
        highest * 2 + ((zero_based >> (highest - 1)) & 1)
    };
    range.tree(&mut models.slots[(count - 2).min(3)], 6, slot, false)?;
    if slot >= 4 {
        let bits = (slot / 2 - 1) as u8;
        let base = (2 + (slot & 1)) << bits;
        let residual = zero_based - base;
        if slot < 14 {
            range.tree(
                &mut models.distances[(slot - 4) as usize],
                bits,
                residual,
                true,
            )?;
        } else {
            range.direct(residual >> 4, bits - 4)?;
            range.tree(&mut models.alignment, 4, residual & 15, true)?;
        }
    }
    models.reps.rotate_right(1);
    models.reps[0] = distance;
    models.state = if state < 7 { 7 } else { 10 };
    Ok(())
}

struct MatchFinder {
    heads: Vec<u64>,
    links: Vec<u32>,
}
impl MatchFinder {
    fn new() -> io::Result<Self> {
        Ok(Self {
            heads: lzma::filled(HASHES, u64::MAX)?,
            links: lzma::filled(DICTIONARY, 0)?,
        })
    }
    fn hash(input: &[u8]) -> usize {
        ((usize::from(input[0]) * 251 + usize::from(input[1])) * 251 + usize::from(input[2]))
            & (HASHES - 1)
    }
    fn insert(&mut self, input: &[u8], position: u64) {
        if input.len() < 3 {
            return;
        }
        let hash = Self::hash(input);
        let previous = self.heads[hash];
        let delta = position
            .checked_sub(previous)
            .filter(|&value| value != 0 && value <= DICTIONARY as u64)
            .unwrap_or(0);
        self.links[position as usize % DICTIONARY] = delta as u32;
        self.heads[hash] = position;
    }
    fn find(&self, dictionary: &Dictionary, input: &[u8], depth: usize) -> (usize, usize) {
        if input.len() < 3 {
            return (0, 0);
        }
        let mut candidate = self.heads[Self::hash(input)];
        let mut best = (0, 0);
        for _ in 0..depth {
            let Some(distance) = dictionary.position.checked_sub(candidate) else {
                break;
            };
            if distance == 0 || distance > DICTIONARY as u64 {
                break;
            }
            let distance = distance as usize;
            let count = match_length(dictionary, input, distance);
            if count > best.1 {
                best = (distance, count);
            }
            if count == input.len().min(273) {
                break;
            }
            let delta = self.links[candidate as usize % DICTIONARY];
            if delta == 0 {
                break;
            }
            let Some(previous) = candidate.checked_sub(u64::from(delta)) else {
                break;
            };
            candidate = previous;
        }
        best
    }
}

fn match_length(dictionary: &Dictionary, input: &[u8], distance: usize) -> usize {
    let mut count = 0;
    while count < input.len().min(273) {
        let byte = if count < distance {
            let Ok(byte) = dictionary.get(distance - count) else {
                break;
            };
            byte
        } else {
            input[count - distance]
        };
        if byte != input[count] {
            break;
        }
        count += 1;
    }
    count
}

/// Bounded streaming LZMA2 compressor, nominal 8 MiB dictionary, one thread.
/// Levels 1, 6 and 9 perform actual LZ matching with different search depths.
/// Incompressible chunks use the format's raw fallback. Drop never finishes.
pub struct Lzma2Writer<W> {
    sink: W,
    dictionary: Dictionary,
    models: Models,
    finder: MatchFinder,
    buffer: Vec<u8>,
    level: u8,
    input: u64,
    encoded: u64,
    dictionary_ready: bool,
    properties_ready: bool,
    failed: bool,
}
impl<W: Write> Lzma2Writer<W> {
    /// Admit all dictionary, matching, model and trial-buffer allocations.
    pub fn new(sink: W, level: u8, memory_limit: u64) -> io::Result<Self> {
        if lzma2_writer_memory_required(level)? > memory_limit {
            return Err(lzma::resource());
        }
        let mut buffer = Vec::new();
        buffer
            .try_reserve_exact(CHUNK)
            .map_err(|_| lzma::resource())?;
        Ok(Self {
            sink,
            dictionary: Dictionary::new(DICTIONARY)?,
            models: Models::new(PROPERTIES)?,
            finder: MatchFinder::new()?,
            buffer,
            level,
            input: 0,
            encoded: 0,
            dictionary_ready: false,
            properties_ready: false,
            failed: false,
        })
    }
    /// Raw LZMA2 dictionary property for the enclosing coder descriptor.
    pub fn property(&self) -> u8 {
        PROPERTY
    }
    /// Uncompressed bytes accepted through `Write`.
    pub fn input_bytes(&self) -> u64 {
        self.input
    }
    /// Bytes successfully written to the owned sink, including headers.
    pub fn encoded_bytes(&self) -> u64 {
        self.encoded
    }
    fn emit(&mut self, bytes: &[u8]) -> io::Result<()> {
        let mut remaining = bytes;
        while !remaining.is_empty() {
            match self.sink.write(remaining) {
                Err(error) if lzma::retryable_io(&error) => continue,
                Err(error) => return Err(lzma::preserve_io_error(error)),
                Ok(0) => {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "compressed_sink_write_zero",
                    ));
                }
                Ok(count) if count <= remaining.len() => {
                    self.encoded = self.encoded.checked_add(count as u64).ok_or_else(invalid)?;
                    remaining = &remaining[count..];
                }
                Ok(_) => return Err(invalid()),
            }
        }
        Ok(())
    }
    fn trial(&mut self) -> io::Result<Option<Vec<u8>>> {
        self.models.reset(PROPERTIES)?;
        let mut range = RangeOutput::new()?;
        let depth = match self.level {
            1 => 4,
            6 => 32,
            _ => 128,
        };
        let mut position = 0;
        let mut active = true;
        while position < self.buffer.len() {
            let input = &self.buffer[position..];
            let mut count = 1;
            if active {
                let (distance, found) = self.finder.find(&self.dictionary, input, depth);
                let mut repetition_choice = (0, 0);
                for (index, &rep) in self.models.reps.iter().enumerate() {
                    let length = match_length(&self.dictionary, input, rep);
                    if length > repetition_choice.1 {
                        repetition_choice = (index, length);
                    }
                }
                let (index, repeated) = repetition_choice;
                let result = if repeated >= 2 && repeated.saturating_add(1) >= found {
                    count = repeated;
                    repetition(
                        &mut range,
                        &mut self.models,
                        self.dictionary.position,
                        index,
                        count,
                    )
                } else if found >= 3 {
                    count = found;
                    distance_match(
                        &mut range,
                        &mut self.models,
                        self.dictionary.position,
                        distance,
                        count,
                    )
                } else if index == 0 && repeated == 1 {
                    repetition(&mut range, &mut self.models, self.dictionary.position, 0, 1)
                } else {
                    literal(&mut range, &mut self.models, &self.dictionary, input[0])
                };
                if let Err(error) = result {
                    if error.to_string() != "lzma_trial_full" {
                        return Err(error);
                    }
                    active = false;
                }
            }
            for offset in 0..count {
                self.finder
                    .insert(&self.buffer[position + offset..], self.dictionary.position);
                self.dictionary.push(self.buffer[position + offset])?;
            }
            position += count;
        }
        if !active {
            return Ok(None);
        }
        match range.finish() {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.to_string() == "lzma_trial_full" => Ok(None),
            Err(error) => Err(error),
        }
    }
    fn chunk(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let size = self.buffer.len();
        let compressed = self.trial()?;
        if let Some(packed) = compressed.filter(|bytes| bytes.len() + 3 < size) {
            // Each compressed chunk resets models. Keeping dictionary history
            // across chunks permits long-distance matches without snapshots or
            // undoing probability mutations when a trial falls back to raw.
            let control = if !self.dictionary_ready {
                0xe0
            } else if !self.properties_ready {
                0xc0
            } else {
                0xa0
            };
            let unpacked = ((size - 1) as u16).to_be_bytes();
            let packed_size = ((packed.len() - 1) as u16).to_be_bytes();
            self.emit(&[
                control,
                unpacked[0],
                unpacked[1],
                packed_size[0],
                packed_size[1],
            ])?;
            if control >= 0xc0 {
                self.emit(&[0x5d])?;
            }
            self.emit(&packed)?;
            self.dictionary_ready = true;
            self.properties_ready = true;
        } else {
            let control = if self.dictionary_ready { 2 } else { 1 };
            let bytes = ((size - 1) as u16).to_be_bytes();
            self.emit(&[control, bytes[0], bytes[1]])?;
            // Avoid a second chunk-sized allocation solely to split borrows.
            let mut position = 0;
            while position < size {
                match self.sink.write(&self.buffer[position..]) {
                    Err(error) if lzma::retryable_io(&error) => continue,
                    Err(error) => return Err(lzma::preserve_io_error(error)),
                    Ok(0) => {
                        return Err(io::Error::new(
                            io::ErrorKind::WriteZero,
                            "compressed_sink_write_zero",
                        ));
                    }
                    Ok(count) if count <= size - position => {
                        position += count;
                        self.encoded =
                            self.encoded.checked_add(count as u64).ok_or_else(invalid)?;
                    }
                    Ok(_) => return Err(invalid()),
                }
            }
            if !self.dictionary_ready {
                self.properties_ready = false;
            }
            self.dictionary_ready = true;
        }
        self.buffer.clear();
        Ok(())
    }
    /// Finish pending data and emit the overall end marker. An earlier error
    /// prevents finish, and no failed write is replayed.
    pub fn finish(mut self) -> io::Result<W> {
        if self.failed {
            return Err(invalid());
        }
        self.chunk()?;
        self.emit(&[0])?;
        Ok(self.sink)
    }
}
impl<W: Write> Write for Lzma2Writer<W> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if input.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        if self.buffer.len() == CHUNK
            && let Err(error) = self.chunk()
        {
            self.failed = true;
            return Err(error);
        }
        let count = input.len().min(CHUNK - self.buffer.len());
        let Some(total) = self.input.checked_add(count as u64) else {
            self.failed = true;
            return Err(invalid());
        };
        self.input = total;
        self.buffer.extend_from_slice(&input[..count]);
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        let result = self.chunk().and_then(|_| {
            loop {
                match self.sink.flush() {
                    Err(error) if lzma::retryable_io(&error) => continue,
                    Err(error) => break Err(lzma::preserve_io_error(error)),
                    Ok(()) => break Ok(()),
                }
            }
        });
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Lzma2Reader, LzmaReader};
    use std::io::Read;

    fn literal_bytes(
        range: &mut RangeOutput,
        models: &mut Models,
        dictionary: &mut Dictionary,
        plain: &mut Vec<u8>,
        bytes: &[u8],
    ) {
        for &byte in bytes {
            literal(range, models, dictionary, byte).unwrap();
            dictionary.push(byte).unwrap();
            plain.push(byte);
        }
    }
    fn copy(
        range: &mut RangeOutput,
        models: &mut Models,
        dictionary: &mut Dictionary,
        plain: &mut Vec<u8>,
        distance: usize,
        count: usize,
        rep: Option<usize>,
    ) {
        if let Some(index) = rep {
            assert_eq!(models.reps[index], distance);
            repetition(range, models, dictionary.position, index, count).unwrap();
        } else {
            distance_match(range, models, dictionary.position, distance, count).unwrap();
        }
        for _ in 0..count {
            let byte = dictionary.get(distance).unwrap();
            dictionary.push(byte).unwrap();
            plain.push(byte);
        }
    }
    fn compressed(output: &mut Vec<u8>, control: u8, plain: usize, packed: &[u8], property: u8) {
        output.push(control | (((plain - 1) >> 16) as u8));
        output.extend_from_slice(&((plain - 1) as u16).to_be_bytes());
        output.extend_from_slice(&((packed.len() - 1) as u16).to_be_bytes());
        if control >= 0xc0 {
            output.push(property);
        }
        output.extend_from_slice(packed);
    }
    fn reset_vector() -> (Vec<u8>, Vec<u8>) {
        let mut models = Models::new(PROPERTIES).unwrap();
        let mut dictionary = Dictionary::new(4096).unwrap();
        let mut plain = Vec::new();
        let mut encoded = Vec::new();
        let mut range = RangeOutput::new().unwrap();
        literal_bytes(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            b"reset-vector-initial-history\n",
        );
        copy(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            7,
            18,
            None,
        );
        compressed(
            &mut encoded,
            0xe0,
            plain.len(),
            &range.finish().unwrap(),
            0x5d,
        );
        // Raw02 keeps models, repetitions and finite state, while replacing the
        // previous byte/history seen by the following matched literal.
        let raw = b"raw-preserved-state";
        encoded.extend_from_slice(&[2, 0, (raw.len() - 1) as u8]);
        encoded.extend_from_slice(raw);
        for &byte in raw {
            dictionary.push(byte).unwrap();
            plain.push(byte);
        }
        let before = plain.len();
        let mut range = RangeOutput::new().unwrap();
        literal_bytes(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            b"matched-after-raw",
        );
        copy(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            7,
            9,
            Some(0),
        );
        compressed(
            &mut encoded,
            0x80,
            plain.len() - before,
            &range.finish().unwrap(),
            0x5d,
        );
        for (control, property) in [(0xa0, 0x5d), (0xc0, 0x38)] {
            models.reset(Properties::parse(property).unwrap()).unwrap();
            let before = plain.len();
            let mut range = RangeOutput::new().unwrap();
            copy(
                &mut range,
                &mut models,
                &mut dictionary,
                &mut plain,
                13,
                273,
                None,
            );
            literal_bytes(
                &mut range,
                &mut models,
                &mut dictionary,
                &mut plain,
                b"retained-position",
            );
            compressed(
                &mut encoded,
                control,
                plain.len() - before,
                &range.finish().unwrap(),
                property,
            );
        }
        dictionary.reset();
        let raw = b"dictionary-reset";
        encoded.extend_from_slice(&[1, 0, (raw.len() - 1) as u8]);
        encoded.extend_from_slice(raw);
        for &byte in raw {
            dictionary.push(byte).unwrap();
            plain.push(byte);
        }
        models.reset(PROPERTIES).unwrap();
        let before = plain.len();
        let mut range = RangeOutput::new().unwrap();
        copy(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            16,
            32,
            None,
        );
        compressed(
            &mut encoded,
            0xc0,
            plain.len() - before,
            &range.finish().unwrap(),
            0x5d,
        );
        dictionary.reset();
        models.reset(PROPERTIES).unwrap();
        let before = plain.len();
        let mut range = RangeOutput::new().unwrap();
        literal_bytes(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            b"full-dictionary-reset",
        );
        compressed(
            &mut encoded,
            0xe0,
            plain.len() - before,
            &range.finish().unwrap(),
            0x5d,
        );
        encoded.push(0);
        (encoded, plain)
    }
    fn boundary_vector(eos: bool) -> (Vec<u8>, Vec<u8>) {
        let mut dictionary = Dictionary::new(4096).unwrap();
        let mut models = Models::new(PROPERTIES).unwrap();
        let mut range = RangeOutput::new().unwrap();
        let mut plain = Vec::new();
        let seed: Vec<u8> = (0..4096)
            .map(|value| (value * 251 + value / 251) as u8)
            .collect();
        literal_bytes(&mut range, &mut models, &mut dictionary, &mut plain, &seed);
        for (distance, length) in [(4096, 273), (4, 2), (5, 9), (128, 10), (129, 17), (1, 18)] {
            copy(
                &mut range,
                &mut models,
                &mut dictionary,
                &mut plain,
                distance,
                length,
                None,
            );
            literal_bytes(&mut range, &mut models, &mut dictionary, &mut plain, b"X");
        }
        for (index, length) in [(0, 1), (1, 273), (2, 2), (3, 9), (0, 10), (1, 17), (2, 18)] {
            let distance = models.reps[index];
            copy(
                &mut range,
                &mut models,
                &mut dictionary,
                &mut plain,
                distance,
                length,
                Some(index),
            );
            literal_bytes(&mut range, &mut models, &mut dictionary, &mut plain, b"Z");
        }
        // Consecutive matches reach the high finite states (10/11), rather
        // than returning to the literal states between every repetition.
        copy(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            2,
            3,
            None,
        );
        copy(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            3,
            4,
            None,
        );
        for (index, count) in [(0, 1), (1, 2), (2, 18), (3, 273)] {
            let distance = models.reps[index];
            copy(
                &mut range,
                &mut models,
                &mut dictionary,
                &mut plain,
                distance,
                count,
                Some(index),
            );
        }
        literal_bytes(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            b"matched-literal-after-state11",
        );
        if eos {
            // Mathematical EOS is the all-ones zero-based distance. Its length
            // is not constrained to two by the decoder contract.
            distance_match(
                &mut range,
                &mut models,
                dictionary.position,
                u32::MAX as usize + 1,
                273,
            )
            .unwrap();
        }
        (range.finish().unwrap(), plain)
    }

    #[test]
    fn lzma_mathematical_all_rep_length_distance_and_reset_vectors() {
        let (encoded, plain) = reset_vector();
        let mut reader = Lzma2Reader::new(
            encoded.as_slice(),
            encoded.len() as u64,
            plain.len() as u64,
            0,
            1 << 20,
        )
        .unwrap();
        let mut decoded = Vec::new();
        reader.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, plain);
        for eos in [false, true] {
            let (encoded, plain) = boundary_vector(eos);
            let mut reader = LzmaReader::new(
                encoded.as_slice(),
                encoded.len() as u64,
                Some(plain.len() as u64),
                &[0x5d, 0, 0x10, 0, 0],
                1 << 20,
            )
            .unwrap();
            let mut decoded = Vec::new();
            reader.read_to_end(&mut decoded).unwrap();
            assert_eq!(decoded, plain);
        }
    }

    #[test]
    fn lzma2_maximum_chunk_bounded_read_and_eos_rejection() {
        let mut models = Models::new(PROPERTIES).unwrap();
        let mut dictionary = Dictionary::new(4096).unwrap();
        let mut range = RangeOutput::new().unwrap();
        let mut plain = Vec::new();
        literal_bytes(&mut range, &mut models, &mut dictionary, &mut plain, b"A");
        while plain.len() < 2 * 1024 * 1024 {
            let count = (2 * 1024 * 1024 - plain.len()).min(273);
            copy(
                &mut range,
                &mut models,
                &mut dictionary,
                &mut plain,
                1,
                count,
                Some(0),
            );
        }
        let packed = range.finish().unwrap();
        let mut encoded = Vec::new();
        compressed(&mut encoded, 0xe0, plain.len(), &packed, 0x5d);
        assert_eq!(encoded[0], 0xff);
        encoded.push(0);
        let mut reader = Lzma2Reader::new(
            encoded.as_slice(),
            encoded.len() as u64,
            plain.len() as u64,
            0,
            1 << 20,
        )
        .unwrap();
        let mut buffer = [0; 32768];
        let mut count = 0;
        loop {
            let next = reader.read(&mut buffer).unwrap();
            if next == 0 {
                break;
            }
            assert!(next <= 8192);
            assert!(buffer[..next].iter().all(|&byte| byte == b'A'));
            count += next;
        }
        assert_eq!(count, plain.len());
        reader.finish().unwrap();
        assert_eq!(reader.consumed_bytes(), encoded.len() as u64);
        let (packed, plain) = boundary_vector(true);
        for size in [plain.len(), plain.len() + 1] {
            let mut encoded = Vec::new();
            compressed(&mut encoded, 0xe0, size, &packed, 0x5d);
            encoded.push(0);
            let mut reader = Lzma2Reader::new(
                encoded.as_slice(),
                encoded.len() as u64,
                size as u64,
                0,
                1 << 20,
            )
            .unwrap();
            assert!(
                reader.read_to_end(&mut Vec::new()).is_err(),
                "LZMA2 chunk must not accept EOS"
            );
            assert!(reader.finish().is_err());
        }
    }

    #[test]
    fn lzma1_full_literal_context_range_and_history_rejection() {
        for property in [0, 8, 44, 224] {
            let mut models = Models::new(Properties::parse(property).unwrap()).unwrap();
            let mut dictionary = Dictionary::new(4096).unwrap();
            let mut range = RangeOutput::new().unwrap();
            let mut plain = Vec::new();
            let seed: Vec<u8> = (0..5000)
                .map(|value| (value * 61 + value / 256) as u8)
                .collect();
            literal_bytes(&mut range, &mut models, &mut dictionary, &mut plain, &seed);
            let packed = range.finish().unwrap();
            let mut reader = LzmaReader::new(
                packed.as_slice(),
                packed.len() as u64,
                Some(plain.len() as u64),
                &[property, 0, 0x10, 0, 0],
                16 << 20,
            )
            .unwrap();
            let mut decoded = Vec::new();
            reader.read_to_end(&mut decoded).unwrap();
            assert_eq!(decoded, plain);
        }
        // A newly allocated zero-filled dictionary is not valid history.
        let mut models = Models::new(PROPERTIES).unwrap();
        let mut range = RangeOutput::new().unwrap();
        distance_match(&mut range, &mut models, 0, 1, 2).unwrap();
        let packed = range.finish().unwrap();
        let mut reader = LzmaReader::new(
            packed.as_slice(),
            packed.len() as u64,
            Some(2),
            &[0x5d, 0, 0x10, 0, 0],
            1 << 20,
        )
        .unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
        let mut models = Models::new(PROPERTIES).unwrap();
        let mut dictionary = Dictionary::new(4096).unwrap();
        let mut range = RangeOutput::new().unwrap();
        let mut plain = Vec::new();
        literal_bytes(
            &mut range,
            &mut models,
            &mut dictionary,
            &mut plain,
            &vec![b'A'; 4097],
        );
        distance_match(&mut range, &mut models, dictionary.position, 4097, 2).unwrap();
        let packed = range.finish().unwrap();
        let mut reader = LzmaReader::new(
            packed.as_slice(),
            packed.len() as u64,
            Some(4099),
            &[0x5d, 0, 0x10, 0, 0],
            1 << 20,
        )
        .unwrap();
        assert!(
            reader.read_to_end(&mut Vec::new()).is_err(),
            "distance exceeds dictionary even when total output is larger"
        );
    }

    #[test]
    #[ignore = "exports self-authored mathematical streams for independent official-tool readback"]
    fn lzma_export_mathematical_vectors() {
        let directory = std::path::PathBuf::from(
            std::env::var_os("XDZIP_LZMA_VECTOR_DIR").expect("output directory"),
        )
        .join("math-vectors");
        std::fs::create_dir_all(&directory).unwrap();
        let (packed, plain) = reset_vector();
        std::fs::write(directory.join("reset.raw"), packed).unwrap();
        std::fs::write(directory.join("reset.plain"), plain).unwrap();
        for eos in [false, true] {
            let (packed, plain) = boundary_vector(eos);
            let mut bytes = vec![0x5d, 0, 0x10, 0, 0];
            bytes.extend_from_slice(&(plain.len() as u64).to_le_bytes());
            bytes.extend_from_slice(&packed);
            std::fs::write(directory.join(format!("boundary-{eos}.lzma")), bytes).unwrap();
            std::fs::write(directory.join(format!("boundary-{eos}.plain")), plain).unwrap();
        }
        for property in [0, 8, 44, 224] {
            let mut models = Models::new(Properties::parse(property).unwrap()).unwrap();
            let mut dictionary = Dictionary::new(4096).unwrap();
            let mut range = RangeOutput::new().unwrap();
            let mut plain = Vec::new();
            let seed: Vec<u8> = (0..5000)
                .map(|value| (value * 61 + value / 256) as u8)
                .collect();
            literal_bytes(&mut range, &mut models, &mut dictionary, &mut plain, &seed);
            let packed = range.finish().unwrap();
            let mut header = vec![property, 0, 0x10, 0, 0];
            header.extend_from_slice(&(plain.len() as u64).to_le_bytes());
            header.extend_from_slice(&packed);
            std::fs::write(directory.join(format!("context-{property}.lzma")), header).unwrap();
            std::fs::write(directory.join(format!("context-{property}.plain")), plain).unwrap();
        }
    }
    #[test]
    fn lzma_inverse_interval_and_delayed_carry_facts() {
        assert_eq!(RangeOutput::new().unwrap().finish().unwrap(), [0; 5]);
        let mut range = RangeOutput::new().unwrap();
        range.direct(1, 1).unwrap();
        assert_eq!(range.finish().unwrap(), [0, 0x7f, 0xff, 0xff, 0xff]);
        let mut range = RangeOutput::new().unwrap();
        range.pending = 0x7a;
        range.postponed = 2;
        range.low = 0x1_12ab_cdef;
        range.advance().unwrap();
        assert_eq!(range.bytes, [0x7b, 0, 0]);
        assert_eq!(range.pending, 0x12);
        assert_eq!(range.low, 0xabcd_ef00);
    }
}
