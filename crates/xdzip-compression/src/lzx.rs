//! CAB LZX decoding from Microsoft's 1997 LZX format and [MS-PATCH] sections
//! 2.1, 2.3–2.7 for shared tree/offset definitions. This is CAB LZX, without
//! LZXD reference dictionaries, long-match extensions or chunk-size prefixes.
use std::io;

const FRAME: usize = 32768;
const MAX_PACKED: usize = FRAME + 6144;

fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid_lzx_stream")
}

fn reserve<T>(values: &mut Vec<T>, additional: usize) -> io::Result<()> {
    values
        .try_reserve_exact(additional)
        .map_err(|_| io::Error::from(io::ErrorKind::OutOfMemory))
}

/// Stateful bounded LZX decoder for one logical CAB folder.
///
/// Each supplied frame is complete after CAB fragment assembly and checksum
/// verification. Its output is at most 32 KiB. Dictionary, trees, repeated
/// distances and Intel translation position survive frame boundaries. Any
/// failure poisons the instance; only a fresh folder may create a new decoder.
pub struct LzxDecoder {
    window: Vec<u8>,
    position: usize,
    history: usize,
    slots: usize,
    repeated: [usize; 3],
    main_lengths: Vec<u8>,
    length_lengths: [u8; 249],
    main: Huffman,
    length: Huffman,
    aligned: Huffman,
    block_type: u8,
    block_remaining: usize,
    raw_odd: bool,
    raw_padding: bool,
    header: bool,
    translation_size: u32,
    total: u64,
    frames: u32,
    failed: bool,
    finished: bool,
}

impl LzxDecoder {
    /// Admit a power-of-two CAB dictionary from 32 KiB through 2 MiB.
    pub fn new(window_bits: u8) -> io::Result<Self> {
        if !(15..=21).contains(&window_bits) {
            return Err(invalid());
        }
        // Sum position-slot ranges from the complete 1997 position table.
        // From slot 36 onward the footer remains 17 bits. This yields 42/50
        // slots for 1/2 MiB, correcting that document's earlier overview table.
        let size = 1usize << window_bits;
        let mut slots = 0;
        let mut covered = 0;
        while covered < size {
            covered += 1usize << footer_bits(slots);
            slots += 1;
        }
        let mut window = Vec::new();
        reserve(&mut window, size)?;
        window.resize(size, 0);
        let mut main_lengths = Vec::new();
        reserve(&mut main_lengths, 256 + 8 * slots)?;
        main_lengths.resize(256 + 8 * slots, 0);
        Ok(Self {
            window,
            position: 0,
            history: 0,
            slots,
            repeated: [1; 3],
            main_lengths,
            length_lengths: [0; 249],
            main: Huffman::empty(),
            length: Huffman::empty(),
            aligned: Huffman::empty(),
            block_type: 0,
            block_remaining: 0,
            raw_odd: false,
            raw_padding: false,
            header: false,
            translation_size: 0,
            total: 0,
            frames: 0,
            failed: false,
            finished: false,
        })
    }

    /// Decode exactly one CFDATA frame, preserving the folder's state.
    pub fn decode_frame(&mut self, packed: &[u8], expected: usize) -> io::Result<Vec<u8>> {
        if self.failed || self.finished {
            self.failed = true;
            return Err(invalid());
        }
        let result = self.frame(packed, expected);
        if result.is_err() {
            self.failed = true;
        }
        result
    }

    /// Finish the logical folder, rejecting a block with missing output.
    /// No more frames may follow a successful finish.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed || self.block_remaining != 0 {
            self.failed = true;
            return Err(invalid());
        }
        if self.finished {
            return Ok(());
        }
        // Official CAB decoding accepts an absent final odd raw pad. Preserve
        // that EOF-only compatibility; an actual next frame must supply its
        // zero pad before another block header is accepted.
        self.raw_padding = false;
        self.finished = true;
        Ok(())
    }

    fn frame(&mut self, packed: &[u8], expected: usize) -> io::Result<Vec<u8>> {
        if expected == 0 || expected > FRAME || packed.is_empty() || packed.len() > MAX_PACKED {
            return Err(invalid());
        }
        let mut bits = Bits::new(packed);
        if !self.header {
            if bits.get(1)? != 0 {
                self.translation_size = (bits.get(16)? << 16) | bits.get(16)?;
            }
            self.header = true;
        }
        let mut output = Vec::new();
        reserve(&mut output, expected)?;
        while output.len() < expected {
            if self.block_remaining == 0 {
                self.block(&mut bits)?;
            }
            if self.block_type == 3 {
                let count = self.block_remaining.min(expected - output.len());
                for &value in bits.raw(count)? {
                    self.emit(value, &mut output);
                }
                self.block_remaining -= count;
                if self.block_remaining == 0 {
                    self.raw_padding = self.raw_odd;
                }
                continue;
            }
            let symbol = self.main.decode(&mut bits)?;
            if symbol < 256 {
                self.emit(symbol as u8, &mut output);
                self.block_remaining -= 1;
                continue;
            }
            let match_symbol = symbol - 256;
            let primary = match_symbol & 7;
            let length = if primary == 7 {
                9 + self.length.decode(&mut bits)?
            } else {
                2 + primary
            };
            if length > self.block_remaining || length > expected - output.len() {
                return Err(invalid());
            }
            let slot = match_symbol >> 3;
            if slot >= self.slots {
                return Err(invalid());
            }
            let distance = self.distance(slot, &mut bits)?;
            if distance == 0 || distance > self.history || distance > self.window.len() - 3 {
                return Err(invalid());
            }
            for _ in 0..length {
                let from = (self.position + self.window.len() - distance) & (self.window.len() - 1);
                self.emit(self.window[from], &mut output);
            }
            self.block_remaining -= length;
        }
        if self.raw_padding && bits.remaining() == 1 && bits.left == 0 {
            if bits.raw(1)?[0] != 0 {
                return Err(invalid());
            }
            self.raw_padding = false;
        }
        bits.finish()?;
        self.translate(&mut output);
        self.total = self
            .total
            .checked_add(expected as u64)
            .ok_or_else(invalid)?;
        self.frames = self.frames.checked_add(1).ok_or_else(invalid)?;
        Ok(output)
    }

    fn emit(&mut self, value: u8, output: &mut Vec<u8>) {
        self.window[self.position] = value;
        self.position = (self.position + 1) & (self.window.len() - 1);
        self.history = (self.history + 1).min(self.window.len());
        output.push(value);
    }

    fn block(&mut self, bits: &mut Bits<'_>) -> io::Result<()> {
        if self.raw_padding {
            if bits.raw(1)?[0] != 0 {
                return Err(invalid());
            }
            self.raw_padding = false;
        }
        self.block_type = bits.get(3)? as u8;
        self.block_remaining = bits.get(24)? as usize;
        if self.block_remaining == 0 {
            return Err(invalid());
        }
        match self.block_type {
            1 | 2 => {
                if self.block_type == 2 {
                    let mut lengths = [0; 8];
                    for length in &mut lengths {
                        *length = bits.get(3)? as u8;
                    }
                    self.aligned = Huffman::build(&lengths, false)?;
                }
                read_lengths(bits, &mut self.main_lengths[..256])?;
                read_lengths(bits, &mut self.main_lengths[256..])?;
                self.main = Huffman::build(&self.main_lengths, false)?;
                read_lengths(bits, &mut self.length_lengths)?;
                self.length = Huffman::build(&self.length_lengths, true)?;
            }
            3 => {
                bits.align_raw()?;
                for distance in &mut self.repeated {
                    let raw = bits.raw(4)?;
                    *distance = u32::from_le_bytes(raw.try_into().map_err(|_| invalid())?) as usize;
                    if *distance == 0 || *distance > self.window.len() - 3 {
                        return Err(invalid());
                    }
                }
                self.raw_odd = self.block_remaining & 1 != 0;
            }
            _ => return Err(invalid()),
        }
        Ok(())
    }

    fn distance(&mut self, slot: usize, bits: &mut Bits<'_>) -> io::Result<usize> {
        if slot < 3 {
            self.repeated.swap(0, slot);
            return Ok(self.repeated[0]);
        }
        let extra = footer_bits(slot);
        let footer = if self.block_type == 2 && extra >= 3 {
            ((bits.get(extra - 3)? as usize) << 3) + self.aligned.decode(bits)?
        } else {
            bits.get(extra)? as usize
        };
        let distance = position_base(slot) + footer - 2;
        self.repeated = [distance, self.repeated[0], self.repeated[1]];
        Ok(distance)
    }

    fn translate(&self, output: &mut [u8]) {
        if self.translation_size == 0 || self.total >= 1 << 30 || output.len() <= 10 {
            return;
        }
        let mut at = 0;
        while at < output.len() - 10 {
            if output[at] != 0xe8 {
                at += 1;
                continue;
            }
            let value = u32::from_le_bytes(output[at + 1..at + 5].try_into().unwrap());
            let position = (self.total + at as u64) as u32;
            let relative = if value < self.translation_size {
                Some(value.wrapping_sub(position))
            } else if position != 0 && value >= position.wrapping_neg() {
                Some(value.wrapping_add(self.translation_size))
            } else {
                None
            };
            if let Some(relative) = relative {
                output[at + 1..at + 5].copy_from_slice(&relative.to_le_bytes());
            }
            at += 5;
        }
    }
}

fn footer_bits(slot: usize) -> u8 {
    if slot < 4 {
        0
    } else {
        ((slot / 2) - 1).min(17) as u8
    }
}

fn position_base(slot: usize) -> usize {
    (0..slot)
        .map(|previous| 1usize << footer_bits(previous))
        .sum()
}

/// Reads MSB-first bits from little-endian 16-bit words without look-ahead.
struct Bits<'a> {
    bytes: &'a [u8],
    cursor: usize,
    word: u16,
    left: u8,
}

impl<'a> Bits<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            cursor: 0,
            word: 0,
            left: 0,
        }
    }

    fn get(&mut self, count: u8) -> io::Result<u32> {
        let mut remaining = count;
        let mut value = 0;
        while remaining != 0 {
            if self.left == 0 {
                let word = self
                    .bytes
                    .get(self.cursor..self.cursor + 2)
                    .ok_or_else(invalid)?;
                self.word = u16::from_le_bytes([word[0], word[1]]);
                self.cursor += 2;
                self.left = 16;
            }
            let take = remaining.min(self.left);
            self.left -= take;
            let part = (u32::from(self.word) >> self.left) & ((1u32 << take) - 1);
            value = (value << take) | part;
            remaining -= take;
        }
        Ok(value)
    }

    fn align_raw(&mut self) -> io::Result<()> {
        let skip = if self.left == 0 { 16 } else { self.left };
        if self.get(skip)? != 0 {
            return Err(invalid());
        }
        Ok(())
    }

    fn raw(&mut self, count: usize) -> io::Result<&'a [u8]> {
        if self.left != 0 {
            return Err(invalid());
        }
        let end = self.cursor.checked_add(count).ok_or_else(invalid)?;
        let bytes = self.bytes.get(self.cursor..end).ok_or_else(invalid)?;
        self.cursor = end;
        Ok(bytes)
    }

    fn remaining(&self) -> usize {
        self.bytes.len() - self.cursor
    }

    fn finish(mut self) -> io::Result<()> {
        if self.left != 0 && self.get(self.left)? != 0 {
            return Err(invalid());
        }
        if self.cursor != self.bytes.len() {
            return Err(invalid());
        }
        Ok(())
    }
}

/// Canonical MSB Huffman decoder; tables are bounded by the wire alphabet.
struct Huffman {
    counts: [u16; 17],
    first: [u32; 17],
    index: [usize; 17],
    symbols: Vec<u16>,
}

impl Huffman {
    fn empty() -> Self {
        Self {
            counts: [0; 17],
            first: [0; 17],
            index: [0; 17],
            symbols: Vec::new(),
        }
    }

    fn build(lengths: &[u8], allow_empty: bool) -> io::Result<Self> {
        let mut tree = Self::empty();
        for &length in lengths {
            if length > 16 {
                return Err(invalid());
            }
            if length != 0 {
                tree.counts[length as usize] += 1;
            }
        }
        let used: usize = tree.counts.iter().map(|&count| usize::from(count)).sum();
        if used == 0 {
            return if allow_empty {
                Ok(tree)
            } else {
                Err(invalid())
            };
        }
        let mut available = 1i32;
        let mut code = 0;
        let mut index = 0;
        for length in 1..=16 {
            available = available * 2 - i32::from(tree.counts[length]);
            if available < 0 {
                return Err(invalid());
            }
            code = (code + u32::from(tree.counts[length - 1])) << 1;
            tree.first[length] = code;
            tree.index[length] = index;
            index += usize::from(tree.counts[length]);
        }
        if available != 0 {
            return Err(invalid());
        }
        reserve(&mut tree.symbols, used)?;
        for length in 1..=16 {
            for (symbol, &actual) in lengths.iter().enumerate() {
                if usize::from(actual) == length {
                    tree.symbols.push(symbol as u16);
                }
            }
        }
        Ok(tree)
    }

    fn decode(&self, bits: &mut Bits<'_>) -> io::Result<usize> {
        if self.symbols.is_empty() {
            return Err(invalid());
        }
        let mut code = 0;
        for length in 1..=16 {
            code = (code << 1) | bits.get(1)?;
            if let Some(offset) = code.checked_sub(self.first[length])
                && offset < u32::from(self.counts[length])
            {
                return Ok(usize::from(
                    self.symbols[self.index[length] + offset as usize],
                ));
            }
        }
        Err(invalid())
    }
}

fn read_lengths(bits: &mut Bits<'_>, lengths: &mut [u8]) -> io::Result<()> {
    let mut code_lengths = [0; 20];
    for length in &mut code_lengths {
        *length = bits.get(4)? as u8;
    }
    let pretree = Huffman::build(&code_lengths, false)?;
    let mut at = 0;
    while at < lengths.len() {
        let symbol = pretree.decode(bits)?;
        if symbol <= 16 {
            lengths[at] = (lengths[at] + 17 - symbol as u8) % 17;
            at += 1;
            continue;
        }
        let (count, value) = match symbol {
            17 => (4 + bits.get(4)? as usize, 0),
            18 => (20 + bits.get(5)? as usize, 0),
            19 => {
                let count = 4 + bits.get(1)? as usize;
                let delta = pretree.decode(bits)?;
                if delta > 16 {
                    return Err(invalid());
                }
                (count, (lengths[at] + 17 - delta as u8) % 17)
            }
            _ => return Err(invalid()),
        };
        let end = at.checked_add(count).ok_or_else(invalid)?;
        let run = lengths.get_mut(at..end).ok_or_else(invalid)?;
        run.fill(value);
        at = end;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tiny wire-vector author: emits the documented MSB/LE16 layout rather
    // than using a second LZX implementation as the oracle.
    #[derive(Default)]
    struct Writer {
        bytes: Vec<u8>,
        word: u16,
        used: u8,
    }
    impl Writer {
        fn put(&mut self, value: u32, count: u8) {
            for bit in (0..count).rev() {
                self.word |= (((value >> bit) & 1) as u16) << (15 - self.used);
                self.used += 1;
                if self.used == 16 {
                    self.bytes.extend_from_slice(&self.word.to_le_bytes());
                    self.word = 0;
                    self.used = 0;
                }
            }
        }
        fn finish(mut self) -> Vec<u8> {
            if self.used != 0 {
                self.put(0, 16 - self.used);
            }
            self.bytes
        }
        fn raw_header(&mut self, length: usize) {
            self.put(3, 3);
            self.put(length as u32, 24);
            self.put(0, 16 - self.used);
            for _ in 0..3 {
                self.bytes.extend_from_slice(&1u32.to_le_bytes());
            }
        }
        fn code(&mut self, lengths: &[u8], symbol: usize) {
            let length = lengths[symbol];
            assert!(length != 0);
            let mut code = 0;
            for depth in 1..=length {
                code <<= 1;
                if depth == length {
                    code += lengths[..symbol].iter().filter(|&&n| n == depth).count() as u32;
                } else {
                    code += lengths.iter().filter(|&&n| n == depth).count() as u32;
                }
            }
            self.put(code, length);
        }
        fn lengths(&mut self, previous: &[u8], current: &[u8]) {
            // 12 codes of length4 and 8 of length5 form a complete 20-symbol
            // pretree. Encode individual deltas to exercise nonzero history.
            let pretree = [4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5];
            for &length in &pretree {
                self.put(u32::from(length), 4);
            }
            for (&old, &new) in previous.iter().zip(current) {
                self.code(&pretree, usize::from((old + 17 - new) % 17));
            }
        }
        fn compressed_header(
            &mut self,
            length: usize,
            previous: &[u8],
            current: &[u8],
            secondary: &[u8],
        ) {
            self.put(1, 3);
            self.put(length as u32, 24);
            self.lengths(&previous[..256], &current[..256]);
            self.lengths(&previous[256..], &current[256..]);
            self.lengths(&[0; 249], secondary);
        }
    }

    fn raw_frame(payload: &[u8], translation: Option<u32>, pad: bool) -> Vec<u8> {
        let mut writer = Writer::default();
        writer.put(u32::from(translation.is_some()), 1);
        if let Some(size) = translation {
            writer.put(size >> 16, 16);
            writer.put(size & 0xffff, 16);
        }
        writer.raw_header(payload.len());
        writer.bytes.extend_from_slice(payload);
        if pad && payload.len() & 1 != 0 {
            writer.bytes.push(0);
        }
        writer.finish()
    }

    #[test]
    fn window_ranges_follow_complete_position_table() {
        for (bits, slots) in [
            (15, 30),
            (16, 32),
            (17, 34),
            (18, 36),
            (19, 38),
            (20, 42),
            (21, 50),
        ] {
            let decoder = LzxDecoder::new(bits).unwrap();
            assert_eq!(decoder.slots, slots);
            assert_eq!(position_base(slots), 1 << bits);
        }
        assert!(LzxDecoder::new(14).is_err());
        assert!(LzxDecoder::new(22).is_err());
    }

    #[test]
    fn raw_blocks_survive_frames_and_require_finish_for_missing_output() {
        let mut writer = Writer::default();
        writer.put(0, 1);
        writer.raw_header(FRAME + 7);
        writer.bytes.extend(std::iter::repeat_n(42, FRAME));
        let first = writer.finish();
        let mut decoder = LzxDecoder::new(15).unwrap();
        assert_eq!(
            decoder.decode_frame(&first, FRAME).unwrap(),
            vec![42; FRAME]
        );
        assert_eq!(
            decoder.decode_frame(&[1, 2, 3, 4, 5, 6, 7, 0], 7).unwrap(),
            [1, 2, 3, 4, 5, 6, 7]
        );
        decoder.finish().unwrap();
        decoder.finish().unwrap();
        assert!(decoder.decode_frame(&[0, 0], 1).is_err());
        assert!(decoder.finish().is_err());
        let mut decoder = LzxDecoder::new(15).unwrap();
        decoder.decode_frame(&first, FRAME).unwrap();
        assert!(decoder.finish().is_err());
        assert!(decoder.decode_frame(&[1, 2, 3, 4, 5, 6, 7, 0], 7).is_err());
    }

    #[test]
    fn raw_odd_pad_is_optional_only_at_folder_eof() {
        for pad in [false, true] {
            let mut decoder = LzxDecoder::new(15).unwrap();
            assert_eq!(
                decoder
                    .decode_frame(&raw_frame(b"odd", None, pad), 3)
                    .unwrap(),
                b"odd"
            );
            decoder.finish().unwrap();
        }
        let mut decoder = LzxDecoder::new(15).unwrap();
        decoder
            .decode_frame(&raw_frame(b"odd", None, false), 3)
            .unwrap();
        // Another block needs the deferred zero byte even across a frame.
        let mut next = Writer::default();
        next.bytes.push(0);
        next.raw_header(2);
        next.bytes.extend_from_slice(b"ok");
        let next = next.finish();
        assert_eq!(decoder.decode_frame(&next, 2).unwrap(), b"ok");
        decoder.finish().unwrap();
        for suffix in [[0u8, 0], [1, 0]] {
            let mut decoder = LzxDecoder::new(15).unwrap();
            let mut packed = raw_frame(b"odd", None, false);
            packed.extend_from_slice(&suffix);
            assert!(decoder.decode_frame(&packed, 3).is_err());
        }
    }

    #[test]
    fn verbatim_tree_deltas_are_subtracted_across_blocks() {
        let mut first = vec![0; 496];
        first[b'A' as usize] = 1;
        first[b'B' as usize] = 1;
        let mut second = vec![0; 496];
        second[b'B' as usize] = 1;
        second[b'C' as usize] = 1;
        let mut writer = Writer::default();
        writer.put(0, 1);
        writer.compressed_header(3, &[0; 496], &first, &[0; 249]);
        for symbol in b"ABA" {
            writer.code(&first, usize::from(*symbol));
        }
        writer.compressed_header(4, &first, &second, &[0; 249]);
        for symbol in b"BCCB" {
            writer.code(&second, usize::from(*symbol));
        }
        let mut decoder = LzxDecoder::new(15).unwrap();
        assert_eq!(
            decoder.decode_frame(&writer.finish(), 7).unwrap(),
            b"ABABCCB"
        );
        decoder.finish().unwrap();
    }

    fn long_match_frame(literal_first: bool, count: usize) -> Vec<u8> {
        let mut main = vec![0; 496];
        main[b'A' as usize] = 1;
        main[256 + 7] = 1; // R0, secondary length.
        let mut secondary = [0; 249];
        secondary[0] = 1;
        secondary[248] = 1;
        let mut writer = Writer::default();
        writer.put(0, 1);
        writer.compressed_header(count, &[0; 496], &main, &secondary);
        if literal_first {
            writer.code(&main, b'A' as usize);
        }
        writer.code(&main, 256 + 7);
        writer.code(&secondary, 248);
        writer.finish()
    }

    #[test]
    fn overlapping_maximum_match_cannot_read_uninitialized_or_cross_frame_end() {
        let mut decoder = LzxDecoder::new(15).unwrap();
        assert_eq!(
            decoder
                .decode_frame(&long_match_frame(true, 258), 258)
                .unwrap(),
            vec![b'A'; 258]
        );
        decoder.finish().unwrap();
        let mut decoder = LzxDecoder::new(15).unwrap();
        assert!(
            decoder
                .decode_frame(&long_match_frame(false, 257), 257)
                .is_err()
        );
        assert!(
            decoder
                .decode_frame(&long_match_frame(true, 258), 258)
                .is_err()
        );
        for (declared, frame) in [(258, 257), (257, 258)] {
            let mut decoder = LzxDecoder::new(15).unwrap();
            assert!(
                decoder
                    .decode_frame(&long_match_frame(true, declared), frame)
                    .is_err()
            );
        }
    }

    #[test]
    fn explicit_distance_and_repeat_slots_update_the_wire_queue() {
        let mut main = vec![0; 496];
        for slot in 1..=4 {
            main[256 + slot * 8] = 2;
        }
        let mut writer = Writer::default();
        writer.put(0, 1);
        writer.raw_header(40);
        let start = writer.bytes.len() - 12;
        for (index, distance) in [5u32, 7, 9].into_iter().enumerate() {
            writer.bytes[start + index * 4..start + index * 4 + 4]
                .copy_from_slice(&distance.to_le_bytes());
        }
        writer.bytes.extend(0..40);
        writer.compressed_header(12, &[0; 496], &main, &[0; 249]);
        for slot in [1, 2, 4, 3, 4, 2] {
            writer.code(&main, 256 + slot * 8);
            if slot == 4 {
                writer.put(1, 1); // Explicit distance3, including when R1 is3.
            }
        }
        let mut decoder = LzxDecoder::new(15).unwrap();
        let output = decoder.decode_frame(&writer.finish(), 52).unwrap();
        assert_eq!(
            &output[40..],
            &[33, 34, 33, 34, 34, 33, 33, 33, 33, 33, 33, 33]
        );
        assert_eq!(decoder.repeated, [3, 1, 3]);
        decoder.finish().unwrap();
    }

    #[test]
    fn a_later_frame_matches_unfiltered_intel_history() {
        let mut payload = vec![b'A'; FRAME];
        let instruction = FRAME - 15;
        payload[instruction] = 0xe8;
        payload[instruction + 1..instruction + 5].copy_from_slice(&100u32.to_le_bytes());
        let mut decoder = LzxDecoder::new(15).unwrap();
        let first = decoder
            .decode_frame(&raw_frame(&payload, Some(1000), false), FRAME)
            .unwrap();
        assert_eq!(
            &first[instruction + 1..instruction + 5],
            &100u32.wrapping_sub(instruction as u32).to_le_bytes()
        );
        let mut main = vec![0; 496];
        main[0] = 1;
        main[256 + 8 * 8 + 7] = 1; // Slot8, secondary length16, distance16.
        let mut secondary = [0; 249];
        secondary[0] = 1;
        secondary[7] = 1;
        let mut writer = Writer::default();
        writer.compressed_header(16, &[0; 496], &main, &secondary);
        writer.code(&main, 256 + 8 * 8 + 7);
        writer.code(&secondary, 7);
        writer.put(2, 3);
        let second = decoder.decode_frame(&writer.finish(), 16).unwrap();
        let mut expected = payload[FRAME - 16..].to_vec();
        expected[2..6].copy_from_slice(&100u32.wrapping_sub((FRAME + 1) as u32).to_le_bytes());
        assert_eq!(second, expected);
        decoder.finish().unwrap();
    }

    #[test]
    fn canonical_trees_and_length_runs_reject_invalid_shapes() {
        for lengths in [&[1u8][..], &[1, 1, 1], &[2, 2], &[17, 1], &[]] {
            assert!(Huffman::build(lengths, false).is_err());
        }
        assert!(Huffman::build(&[], true).is_ok());
        assert!(Huffman::build(&[1, 1], false).is_ok());
        let pretree = [4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 5, 5, 5, 5, 5, 5, 5, 5];
        for (symbol, extra, width) in [(17, 0, 4), (18, 0, 5), (19, 0, 1)] {
            let mut writer = Writer::default();
            for &length in &pretree {
                writer.put(u32::from(length), 4);
            }
            writer.code(&pretree, symbol);
            writer.put(extra, width);
            if symbol == 19 {
                writer.code(&pretree, 2);
            }
            let data = writer.finish();
            assert!(read_lengths(&mut Bits::new(&data), &mut [3; 3]).is_err());
        }
        let mut writer = Writer::default();
        for &length in &pretree {
            writer.put(u32::from(length), 4);
        }
        writer.code(&pretree, 19);
        writer.put(0, 1);
        writer.code(&pretree, 2);
        let data = writer.finish();
        let mut previous = [3, 4, 5, 6];
        read_lengths(&mut Bits::new(&data), &mut previous).unwrap();
        assert_eq!(previous, [1; 4]);
    }

    #[test]
    fn frame_end_is_exact_and_nonzero_padding_or_extra_words_fail() {
        let packed = raw_frame(b"plain", None, true);
        for cut in 0..packed.len() - 1 {
            let mut decoder = LzxDecoder::new(15).unwrap();
            assert!(decoder.decode_frame(&packed[..cut], 5).is_err());
        }
        let mut bad_pad = packed.clone();
        *bad_pad.last_mut().unwrap() = 1;
        let mut decoder = LzxDecoder::new(15).unwrap();
        assert!(decoder.decode_frame(&bad_pad, 5).is_err());
        let mut extra = packed;
        extra.extend_from_slice(&[0, 0]);
        let mut decoder = LzxDecoder::new(15).unwrap();
        assert!(decoder.decode_frame(&extra, 5).is_err());
        for expected in [0, FRAME + 1] {
            assert!(
                LzxDecoder::new(15)
                    .unwrap()
                    .decode_frame(&[0, 0], expected)
                    .is_err()
            );
        }
        let mut compressed = long_match_frame(true, 258);
        let last_word = compressed.len() - 2;
        compressed[last_word] |= 1; // An unused low bit of the final LE16 word.
        assert!(
            LzxDecoder::new(15)
                .unwrap()
                .decode_frame(&compressed, 258)
                .is_err()
        );
        let mut aligned = Bits::new(&[0, 0, 5, 0]);
        aligned.align_raw().unwrap(); // Already aligned still consumes a word.
        assert_eq!(aligned.raw(2).unwrap(), &[5, 0]);
        assert!(Bits::new(&[1, 0]).align_raw().is_err());
    }

    #[test]
    fn intel_translation_uses_u32_range_and_keeps_history_unfiltered() {
        let mut payload = vec![0; 32];
        payload[1] = 0xe8;
        payload[2..6].copy_from_slice(&0x9000_0000u32.to_le_bytes());
        payload[22] = 0xe8; // Last ten bytes are excluded from opcode scanning.
        payload[23..27].copy_from_slice(&10u32.to_le_bytes());
        let mut decoder = LzxDecoder::new(15).unwrap();
        let output = decoder
            .decode_frame(&raw_frame(&payload, Some(0xf000_0000), true), payload.len())
            .unwrap();
        assert_eq!(&output[2..6], &0x8fff_ffffu32.to_le_bytes());
        assert_eq!(&output[22..], &payload[22..]);
        assert_eq!(&decoder.window[..payload.len()], &payload);
        decoder.finish().unwrap();
        decoder.total = 100;
        let mut negative = vec![0; 16];
        negative[0] = 0xe8;
        negative[1..5].copy_from_slice(&(-50i32).to_le_bytes());
        decoder.translation_size = 1000;
        decoder.translate(&mut negative);
        assert_eq!(&negative[1..5], &950u32.to_le_bytes());
        decoder.total = 1 << 30;
        let before = negative.clone();
        decoder.translate(&mut negative);
        assert_eq!(negative, before);
    }

    #[test]
    fn intel_boundary_vectors_match_the_independent_official_worker() {
        // Numeric discriminators independently verified by the reference
        // analyst with complete CAB inputs and official 7-Zip26.05.
        for (size, at, value, expected) in [
            (1000u32, 0, 100u32, 100u32),
            (1000, 1, 100, 99),
            (1000, 21, 100, 79),
            (1000, 22, 100, 100),
            (1000, 1, u32::MAX, 999),
            (1000, 1, u32::MAX - 1, u32::MAX - 1),
            (0x8000_0010, 1, 0x8000_0000, 0x7fff_ffff),
        ] {
            let mut payload = [b'A'; 32];
            payload[at] = 0xe8;
            payload[at + 1..at + 5].copy_from_slice(&value.to_le_bytes());
            let mut decoder = LzxDecoder::new(15).unwrap();
            let output = decoder
                .decode_frame(&raw_frame(&payload, Some(size), false), 32)
                .unwrap();
            assert_eq!(&output[at + 1..at + 5], &expected.to_le_bytes());
            decoder.finish().unwrap();
        }
    }

    #[test]
    fn fixed_independent_cab_lzx_payload() {
        let cab = include_bytes!("../../../tests/fixtures/cab-iso/mszip_lzx_qtm.cab");
        // The manifest's second folder is the independent LZX18 sample.
        let offset = u32::from_le_bytes(cab[44..48].try_into().unwrap()) as usize;
        let packed = u16::from_le_bytes(cab[offset + 4..offset + 6].try_into().unwrap()) as usize;
        let expected = u16::from_le_bytes(cab[offset + 6..offset + 8].try_into().unwrap()) as usize;
        let mut decoder = LzxDecoder::new(18).unwrap();
        let output = decoder
            .decode_frame(&cab[offset + 8..offset + 8 + packed], expected)
            .unwrap();
        assert_eq!(output, b"-----------------------------------------------------------------\nIf you can read this, the LZX decompressor is working!\n-----------------------------------------------------------------\n");
        decoder.finish().unwrap();
    }

    #[test]
    #[ignore = "independent makecab-generated multi-frame fixtures under build/cab-lzx"]
    fn microsoft_makecab_all_windows_and_aligned_blocks() {
        let root = std::path::PathBuf::from(
            std::env::var_os("XDZIP_LZX_FIXTURE_DIR").expect("fixture directory"),
        );
        for (label, bits, input) in (15..=21)
            .map(|bits| (format!("window{bits}"), bits, "payload.bin"))
            .chain([
                ("aligned15".to_owned(), 15, "aligned.bin"),
                ("aligned21".to_owned(), 21, "aligned.bin"),
            ])
        {
            let packed = std::fs::read(root.join(format!("{label}.frames"))).unwrap();
            let expected = std::fs::read(root.join(input)).unwrap();
            let mut decoder = LzxDecoder::new(bits).unwrap();
            let mut offset = 0;
            let mut output = Vec::new();
            while offset < packed.len() {
                let size =
                    u32::from_le_bytes(packed[offset..offset + 4].try_into().unwrap()) as usize;
                let unpacked =
                    u32::from_le_bytes(packed[offset + 4..offset + 8].try_into().unwrap()) as usize;
                offset += 8;
                output.extend_from_slice(
                    &decoder
                        .decode_frame(&packed[offset..offset + size], unpacked)
                        .unwrap(),
                );
                offset += size;
            }
            decoder.finish().unwrap();
            assert_eq!(output, expected, "{label}");
        }
    }
}
