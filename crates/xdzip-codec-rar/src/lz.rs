use crate::MEMORY_LIMIT;
use crate::algorithm::WorkOutput;
#[cfg(test)]
#[path = "filter_author_modern_tests.rs"]
mod filter_author_modern_tests;
use crate::decode_error;
use crate::index::Descriptor;
use crate::lz_bits::{Bits, Input, Tables};
use crate::lz_filters::{Filter, Output};
use crate::paged_history::PagedHistory;
use std::io::Read;
use xdzip_codec_api::Cancellation;
use xdzip_codec_api::payload::DecodeError;

// Includes retained metadata, filter input/copies, pending descriptors and tables.
// It is a conservative worker admission allowance, not a format dictionary limit.
const OTHER_MEMORY: u64 = 288 * 1024 * 1024;

fn dense_fits(window: u64) -> bool {
    window
        .checked_add(OTHER_MEMORY)
        .and_then(|total| total.checked_add(crate::protection::Protection::MEMORY_BYTES))
        .is_some_and(|total| total <= MEMORY_LIMIT)
}

// Only the modern algorithm 1 compressed profile can use a large logical
// dictionary. Admission of its exact representation does not authorize output.
pub(crate) fn paged_memory_required(d: &Descriptor) -> Result<u64, DecodeError> {
    if d.legacy
        || d.ancient
        || d.method != 70
        || d.effective != 1
        || !(1..=5).contains(&d.compression)
    {
        return Err(DecodeError::Limit);
    }
    PagedHistory::memory_required(d.dictionary.max(256 * 1024))
}

enum Instruction {
    Literal(u8),
    Copy { distance: u64, length: usize },
    Filter(Filter),
    Noop,
}
impl Instruction {
    fn length(&self) -> usize {
        match self {
            Self::Literal(_) => 1,
            Self::Copy { length, .. } => *length,
            _ => 0,
        }
    }
}
#[derive(Default)]
pub(crate) struct Lz {
    window: Vec<u8>,
    paged: Option<PagedHistory>,
    position: u64,
    available: u64,
    recent: [Option<u64>; 4],
    last_length: usize,
    tables: Option<Tables>,
    algorithm: Option<u8>,
    failed: bool,
    #[cfg(test)]
    filter_counts: [u64; 4],
}
impl Lz {
    pub fn memory_required(&self, d: &Descriptor) -> Result<u64, DecodeError> {
        if !matches!(d.method, 50 | 70)
            || d.effective > 1
            || d.compression == 0
            || d.compression > 5
            || d.encrypted
            || d.hash.is_some()
        {
            return Err(DecodeError::Unsupported);
        }
        let history = if d.solid && self.window_len() != 0 {
            if let Some(paged) = &self.paged {
                PagedHistory::memory_required(paged.logical_len())?
            } else {
                self.window.len() as u64
            }
        } else {
            let window = d.dictionary.max(256 * 1024);
            if dense_fits(window) {
                window
            } else {
                paged_memory_required(d)?
            }
        };
        history
            .checked_add(OTHER_MEMORY)
            .filter(|n| *n <= MEMORY_LIMIT)
            .ok_or(DecodeError::Limit)
    }
    pub fn reset(&mut self) {
        self.position = 0;
        self.available = 0;
        self.recent = [None; 4];
        self.last_length = 0;
        self.tables = None;
        self.algorithm = None;
    }
    fn window_len(&self) -> u64 {
        self.paged
            .as_ref()
            .map_or(self.window.len() as u64, PagedHistory::logical_len)
    }
    #[cfg(test)]
    pub(crate) fn paged_history_stats(&self) -> Option<(u64, u64, u64)> {
        self.paged.as_ref().map(|history| {
            (
                history.logical_len(),
                history.allocated_bytes(),
                history.raw_allocated_bytes(),
            )
        })
    }
    fn begin(
        &mut self,
        d: &Descriptor,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.memory_required(d)?;
        if d.solid {
            if self.algorithm != Some(d.effective) || self.window_len() == 0 {
                return Err(DecodeError::Unsupported);
            }
            if d.dictionary.max(256 * 1024) > self.window_len() {
                return Err(DecodeError::Unsupported);
            }
        } else {
            self.reset();
            let logical_len = d.dictionary.max(256 * 1024);
            let use_paged = !dense_fits(logical_len);
            if use_paged {
                // Non-solid history cannot reuse old bytes or a partial rewrite.
                // Drop both representations before admitting the new directory.
                self.window = Vec::new();
                self.paged = None;
                self.paged = Some(PagedHistory::new(logical_len, sink, control)?);
            } else {
                self.paged = None;
                let size = usize::try_from(logical_len).map_err(|_| DecodeError::Limit)?;
                if self.window.len() != size {
                    // A non-solid reset no longer needs the old dictionary. Release
                    // it before admitting a replacement, avoiding a two-window peak.
                    self.window = Vec::new();
                    let mut window = Vec::new();
                    window
                        .try_reserve_exact(size)
                        .map_err(|_| DecodeError::Limit)?;
                    window.resize(size, 0);
                    self.window = window;
                }
            }
            self.algorithm = Some(d.effective);
        }
        Ok(())
    }
    fn put(
        &mut self,
        byte: u8,
        output: &mut Output,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(paged) = &mut self.paged {
            let write = paged.prepare_write_at(self.position, byte, sink, control)?;
            output.put(byte)?;
            write.commit();
        } else {
            output.put(byte)?;
            self.window[self.position as usize] = byte;
        }
        self.position += 1;
        let window_len = self.window_len();
        if self.position == window_len {
            self.position = 0;
        }
        self.available = (self.available + 1).min(window_len);
        Ok(())
    }
    fn copy(
        &mut self,
        distance: u64,
        length: usize,
        output: &mut Output,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        let window_len = self.window_len();
        if distance == 0 || distance > window_len || distance > self.available {
            return Err(DecodeError::Integrity);
        }
        for _ in 0..length {
            let source = if self.position >= distance {
                self.position - distance
            } else {
                window_len - (distance - self.position)
            };
            let byte = if let Some(paged) = &self.paged {
                paged.read_at(source)?
            } else {
                self.window[source as usize]
            };
            self.put(byte, output, sink, control)?;
        }
        Ok(())
    }
    fn length(slot: u16, bits: &mut Bits<'_, '_>) -> Result<usize, DecodeError> {
        if slot > 43 {
            return Err(DecodeError::Integrity);
        }
        if slot < 8 {
            return Ok(2 + usize::from(slot));
        }
        let extra = (slot / 4 - 1) as u8;
        let base = 4 + usize::from(slot % 4);
        Ok(2 + (base << extra) + bits.get(extra)? as usize)
    }
    fn distance(&self, bits: &mut Bits<'_, '_>) -> Result<u64, DecodeError> {
        let tables = self.tables.as_ref().ok_or(DecodeError::Integrity)?;
        let slot = tables.distance.symbol(bits)?;
        if slot < 4 {
            return Ok(u64::from(slot) + 1);
        }
        let extra = (slot / 2 - 1) as u8;
        let base = (2 + u64::from(slot % 2))
            .checked_shl(u32::from(extra))
            .ok_or(DecodeError::Integrity)?;
        let value = if extra < 4 {
            bits.get(extra)?
        } else {
            (bits.get(extra - 4)? << 4) | u64::from(tables.low.symbol(bits)?)
        };
        1u64.checked_add(base)
            .and_then(|n| n.checked_add(value))
            .ok_or(DecodeError::Integrity)
    }
    fn filter(bits: &mut Bits<'_, '_>, position: u64) -> Result<Filter, DecodeError> {
        fn integer(bits: &mut Bits<'_, '_>) -> Result<u64, DecodeError> {
            let count = bits.get(2)? + 1;
            let mut value = 0;
            for i in 0..count {
                value |= bits.get(8)? << (i * 8);
            }
            Ok(value)
        }
        let distance = integer(bits)?;
        let length = integer(bits)?;
        let kind = bits.get(3)? as u8;
        let channels = if kind == 0 { bits.get(5)? as u8 + 1 } else { 0 };
        Ok(Filter {
            start: position
                .checked_add(distance)
                .ok_or(DecodeError::Integrity)?,
            length: usize::try_from(length).map_err(|_| DecodeError::Limit)?,
            kind,
            channels,
        })
    }
    fn instruction(
        &mut self,
        bits: &mut Bits<'_, '_>,
        position: u64,
    ) -> Result<Instruction, DecodeError> {
        let symbol = self
            .tables
            .as_ref()
            .ok_or(DecodeError::Integrity)?
            .main
            .symbol(bits)?;
        Ok(match symbol {
            0..=255 => Instruction::Literal(symbol as u8),
            256 => {
                let filter = Self::filter(bits, position)?;
                #[cfg(test)]
                if filter.kind < 4 {
                    self.filter_counts[usize::from(filter.kind)] += 1;
                }
                Instruction::Filter(filter)
            }
            257 => {
                if self.last_length == 0 {
                    Instruction::Noop
                } else {
                    Instruction::Copy {
                        distance: self.recent[0].ok_or(DecodeError::Integrity)?,
                        length: self.last_length,
                    }
                }
            }
            258..=261 => {
                let at = usize::from(symbol - 258);
                let distance = self.recent[at].ok_or(DecodeError::Integrity)?;
                let slot = self
                    .tables
                    .as_ref()
                    .ok_or(DecodeError::Integrity)?
                    .repeat
                    .symbol(bits)?;
                let length = Self::length(slot, bits)?;
                for i in (1..=at).rev() {
                    self.recent[i] = self.recent[i - 1];
                }
                self.recent[0] = Some(distance);
                self.last_length = length;
                Instruction::Copy { distance, length }
            }
            262..=305 => {
                let mut length = Self::length(symbol - 262, bits)?;
                let distance = self.distance(bits)?;
                length += usize::from(distance > 256)
                    + usize::from(distance > 8192)
                    + usize::from(distance > 262144);
                self.recent.rotate_right(1);
                self.recent[0] = Some(distance);
                self.last_length = length;
                Instruction::Copy { distance, length }
            }
            _ => return Err(DecodeError::Integrity),
        })
    }
    fn execute(
        &mut self,
        batch: &mut Vec<Instruction>,
        output: &mut Output,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        let planned = batch.iter().map(Instruction::length).sum::<usize>();
        if planned > 8192 {
            return Err(DecodeError::Limit);
        }
        sink.reserve_work(planned as u64)?;
        let before = output.position();
        let result = (|| {
            for instruction in batch.drain(..) {
                control.check().map_err(decode_error)?;
                match instruction {
                    Instruction::Literal(byte) => self.put(byte, output, sink, control)?,
                    Instruction::Copy { distance, length } => {
                        self.copy(distance, length, output, sink, control)?
                    }
                    Instruction::Filter(filter) => output.register(filter)?,
                    Instruction::Noop => (),
                }
            }
            Ok(())
        })();
        // Only executed bytes are settled externally and become deliverable.
        // Cancellation or a partially failed instruction cannot refund work or
        // turn its unexecuted local reservation into prepaid output authority.
        sink.produced_work(output.position() - before)?;
        result?;
        output.flush(sink, control, false)
    }
    fn run(
        &mut self,
        d: &Descriptor,
        input: &mut dyn Read,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
        padding: u8,
    ) -> Result<(), DecodeError> {
        self.begin(d, sink, control)?;
        let mut input = Input::new(input);
        let mut output = Output::new(d.size.ok_or(DecodeError::Unsupported)?);
        loop {
            control.check().map_err(decode_error)?;
            let flags = input.byte()?;
            let check = input.byte()?;
            let width = ((flags >> 3) & 3) + 1;
            if width == 4 {
                return Err(DecodeError::Integrity);
            }
            let mut bytes = 0u64;
            for i in 0..width {
                bytes |= u64::from(input.byte()?) << (i * 8);
            }
            let expected =
                0x5a ^ flags ^ (bytes as u8) ^ ((bytes >> 8) as u8) ^ ((bytes >> 16) as u8);
            if check != expected {
                return Err(DecodeError::Integrity);
            }
            let mut bits = Bits::new(&mut input, bytes, (flags & 7) + 1)?;
            if flags & 128 != 0 {
                self.tables = Some(Tables::read(&mut bits, d.effective)?);
            }
            let mut batch = Vec::with_capacity(256);
            let mut planned = 0usize;
            while bits.remaining() != 0 {
                control.check().map_err(decode_error)?;
                let instruction =
                    self.instruction(&mut bits, output.position() + planned as u64)?;
                let length = instruction.length();
                if planned + length > 8192 || batch.len() == 256 {
                    self.execute(&mut batch, &mut output, sink, control)?;
                    planned = 0;
                }
                planned += length;
                batch.push(instruction);
            }
            self.execute(&mut batch, &mut output, sink, control)?;
            if flags & 64 != 0 {
                break;
            }
        }
        input.end_padding(padding)?;
        output.flush(sink, control, true)?;
        control.check().map_err(decode_error)
    }
    pub fn decode(
        &mut self,
        d: &Descriptor,
        input: &mut dyn Read,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.decode_padded(d, input, sink, control, 0)
    }
    pub fn decode_padded(
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
        let result = if padding > 15 {
            Err(DecodeError::Unsupported)
        } else {
            self.run(d, input, sink, control, padding)
        };
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lz_bits::Huffman;
    use std::io::Cursor;
    use xdzip_codec_api::payload::DecodedSink;
    #[derive(Default)]
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
            self.0.extend(b);
            Ok(())
        }
    }
    impl WorkOutput for Sink {}
    fn description(algorithm: u8, size: u64, solid: bool) -> Descriptor {
        Descriptor {
            size: Some(size),
            method: if algorithm == 0 { 50 } else { 70 },
            compression: 3,
            effective: algorithm,
            dictionary: if algorithm == 0 { 131072 } else { 270336 },
            solid,
            ..Descriptor::default()
        }
    }
    // A fixed mathematical wire vector, not a compression encoder. Twenty
    // nibble lengths admit meta symbols0/1, then one-bit lengths admit A/B.
    fn literal_vector(algorithm: u8, last: bool) -> Vec<u8> {
        let n = if algorithm == 0 { 65 } else { 67 };
        let mut bytes = vec![0; n];
        bytes[0] = 0x11;
        bytes[18] = 0x60;
        bytes[n - 2] = 1;
        bytes[n - 1] = 0x80;
        let flags = 0x81 | if last { 64 } else { 0 };
        let mut out = vec![flags, 0x5a ^ flags ^ (n as u8), n as u8];
        out.extend(bytes);
        out
    }
    fn tree(n: usize, items: &[(usize, u8)]) -> Huffman {
        let mut lengths = vec![0; n];
        for (i, v) in items {
            lengths[*i] = *v;
        }
        Huffman::new(&lengths).unwrap()
    }
    fn tables(main: &[(usize, u8)], distance: usize, slot: usize) -> Tables {
        Tables {
            main: tree(306, main),
            distance: tree(distance, &[(slot, 1)]),
            low: tree(16, &[(15, 1)]),
            repeat: tree(44, &[]),
        }
    }
    #[test]
    fn fixed_literal_blocks_decode_tables_and_exact_last_bits() {
        for algorithm in [0, 1] {
            let mut lz = Lz::default();
            let mut sink = Sink::default();
            lz.decode(
                &description(algorithm, 4, false),
                &mut Cursor::new(literal_vector(algorithm, true)),
                &mut sink,
                &Cancellation::default(),
            )
            .unwrap();
            assert_eq!(sink.0, b"ABBA");
            assert_eq!(
                lz.window.len(),
                if algorithm == 0 { 262144 } else { 270336 }
            );
        }
    }
    #[test]
    fn encrypted_alignment_tail_is_bounded_and_separate_from_lz_termination() {
        for algorithm in [0, 1] {
            for tail in 0..=16 {
                let mut bytes = literal_vector(algorithm, true);
                bytes.extend((0..tail).map(|n| 0x91 ^ n));
                let mut lz = Lz::default();
                let mut sink = Sink::default();
                let result = lz.decode_padded(
                    &description(algorithm, 4, false),
                    &mut Cursor::new(bytes.clone()),
                    &mut sink,
                    &Cancellation::default(),
                    15,
                );
                if tail <= 15 {
                    result.unwrap();
                    assert_eq!(sink.0, b"ABBA");
                } else {
                    assert_eq!(result, Err(DecodeError::Integrity));
                }
                if tail != 0 {
                    assert_eq!(
                        Lz::default().decode(
                            &description(algorithm, 4, false),
                            &mut Cursor::new(bytes),
                            &mut Sink::default(),
                            &Cancellation::default(),
                        ),
                        Err(DecodeError::Integrity)
                    );
                }
            }
            for final_block in [false, true] {
                let mut bytes = literal_vector(algorithm, final_block);
                if final_block {
                    bytes.pop();
                }
                assert_eq!(
                    Lz::default().decode_padded(
                        &description(algorithm, 4, false),
                        &mut Cursor::new(bytes),
                        &mut Sink::default(),
                        &Cancellation::default(),
                        15,
                    ),
                    Err(DecodeError::Integrity)
                );
            }
        }
    }
    #[test]
    fn block_and_solid_file_boundaries_reuse_tables() {
        let c = Cancellation::default();
        let mut packed = literal_vector(0, false);
        packed.extend([0x43, 0x18, 1, 0xa0]);
        let mut lz = Lz::default();
        let mut sink = Sink::default();
        lz.decode(
            &description(0, 8, false),
            &mut Cursor::new(packed),
            &mut sink,
            &c,
        )
        .unwrap();
        assert_eq!(sink.0, b"ABBABABA");
        let mut sink = Sink::default();
        lz.decode(
            &description(0, 4, true),
            &mut Cursor::new([0x43, 0x18, 1, 0x60]),
            &mut sink,
            &c,
        )
        .unwrap();
        assert_eq!(sink.0, b"ABBA");
    }
    #[test]
    fn repeated_match_uses_recent_distance_and_previous_length() {
        let mut lz = Lz {
            window: vec![0; 262144],
            tables: Some(tables(&[(65, 1), (257, 2), (262, 2)], 64, 0)),
            algorithm: Some(0),
            ..Lz::default()
        };
        let mut sink = Sink::default();
        lz.decode(
            &description(0, 5, true),
            &mut Cursor::new([0x45, 0x1e, 1, 0x68]),
            &mut sink,
            &Cancellation::default(),
        )
        .unwrap();
        assert_eq!(sink.0, b"AAAAA");
    }
    #[test]
    fn distance_math_keeps_32_and_40_bit_extremes() {
        for (algorithm, n, slot, bytes, bits, expected) in [
            (0, 64, 63, vec![0x7f, 0xff, 0xff, 0xe0], 28, 1u64 << 32),
            (
                1,
                80,
                79,
                vec![0x7f, 0xff, 0xff, 0xff, 0xe0],
                36,
                1u64 << 40,
            ),
        ] {
            let lz = Lz {
                tables: Some(tables(&[], n, slot)),
                algorithm: Some(algorithm),
                ..Lz::default()
            };
            let mut cursor = Cursor::new(bytes.clone());
            let mut input = Input::new(&mut cursor);
            let mut b =
                Bits::new(&mut input, bytes.len() as u64, ((bits - 1) % 8 + 1) as u8).unwrap();
            assert_eq!(lz.distance(&mut b).unwrap(), expected);
        }
    }
    #[test]
    fn length_slots_preserve_extreme_extra_bits() {
        for (slot, byte, n, expected) in [
            (0, 0, 0, 2),
            (7, 0, 0, 9),
            (8, 0, 1, 10),
            (8, 0x80, 1, 11),
            (43, 0xff, 9, 4097),
        ] {
            let mut cursor = Cursor::new([byte, byte]);
            let mut input = Input::new(&mut cursor);
            let mut bits = Bits::new(
                &mut input,
                if n > 8 { 2 } else { 1 },
                if n > 8 { 1 } else { 8 },
            )
            .unwrap();
            assert_eq!(Lz::length(slot, &mut bits).unwrap(), expected);
        }
    }
    #[test]
    fn true_modulo_and_overlap_preserve_prefilter_history() {
        let c = Cancellation::default();
        let mut lz = Lz {
            window: b"HICDEFG".to_vec(),
            position: 2,
            available: 7,
            ..Lz::default()
        };
        let mut out = Output::new(14);
        lz.copy(7, 14, &mut out, &mut Sink::default(), &c).unwrap();
        let mut sink = Sink::default();
        out.flush(&mut sink, &c, true).unwrap();
        assert_eq!(sink.0, b"CDEFGHICDEFGHI");
        let mut lz = Lz {
            window: vec![0; 7],
            ..Lz::default()
        };
        let mut out = Output::new(6);
        out.register(Filter {
            start: 0,
            length: 3,
            kind: 0,
            channels: 1,
        })
        .unwrap();
        for b in [1, 2, 3] {
            lz.put(b, &mut out, &mut Sink::default(), &c).unwrap();
        }
        out.flush(&mut Sink::default(), &c, false).unwrap();
        lz.copy(3, 3, &mut out, &mut Sink::default(), &c).unwrap();
        let mut sink = Sink::default();
        out.flush(&mut sink, &c, true).unwrap();
        assert_eq!(sink.0, [1, 2, 3]);
    }
    #[test]
    fn invalid_tail_header_zero_block_and_poison_are_checked() {
        let c = Cancellation::default();
        for (n, mut bytes) in [
            literal_vector(0, true),
            literal_vector(0, true),
            vec![0xc0, 0x9a, 0],
        ]
        .into_iter()
        .enumerate()
        {
            if bytes.len() > 3 {
                if n == 0 {
                    bytes[1] ^= 1;
                } else {
                    bytes.push(0);
                }
            }
            let mut lz = Lz::default();
            assert!(
                lz.decode(
                    &description(0, 4, false),
                    &mut Cursor::new(bytes),
                    &mut Sink::default(),
                    &c
                )
                .is_err()
            );
            assert_eq!(
                lz.decode(
                    &description(0, 0, false),
                    &mut Cursor::new([0x40, 0x1a, 0]),
                    &mut Sink::default(),
                    &c
                ),
                Err(DecodeError::Integrity)
            );
        }
        let mut lz = Lz::default();
        lz.decode(
            &description(0, 0, false),
            &mut Cursor::new([0x40, 0x1a, 0]),
            &mut Sink::default(),
            &c,
        )
        .unwrap();
    }

    #[test]
    #[ignore = "explicit fixed-fixture branch inventory, not independent filter conformance"]
    fn fixed_fixture_filter_inventory() {
        use crate::{index, source::Reader};
        use std::io::{self, Read, Seek, SeekFrom};
        use xdzip_codec_api::{ReadSource, ScanLimits};
        struct Parts {
            bytes: Cursor<Vec<u8>>,
            lengths: Vec<u64>,
        }
        impl Read for Parts {
            fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
                self.bytes.read(b)
            }
        }
        impl Seek for Parts {
            fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
                self.bytes.seek(p)
            }
        }
        impl ReadSource for Parts {
            fn volume_lengths(&self) -> Option<&[u64]> {
                Some(&self.lengths)
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/native-archives");
        let mut groups = vec![
            vec!["test_read_format_rar5_compressed.rar".to_string()],
            vec!["test_read_format_rar5_multiple_files_solid.rar".to_string()],
            vec!["rar-7.23-rar5-compatible.rar".to_string()],
        ];
        groups.push(
            (1..=4)
                .map(|i| format!("test_read_format_rar5_multiarchive_solid.part{i:02}.rar"))
                .collect(),
        );
        if let Some(extra) = std::env::var_os("XDZIP_RAR_LZ_AUTHORED") {
            for file in ["e8-only.rar", "e8e9.rar"] {
                groups.push(vec![
                    std::path::PathBuf::from(&extra)
                        .join(file)
                        .to_string_lossy()
                        .into_owned(),
                ]);
            }
        }
        for group in groups {
            let chunks = group
                .iter()
                .map(|p| std::fs::read(root.join(p)).unwrap())
                .collect::<Vec<_>>();
            let mut source = Parts {
                lengths: chunks.iter().map(|p| p.len() as u64).collect(),
                bytes: Cursor::new(chunks.concat()),
            };
            let control = Cancellation::default();
            let mut reader = Reader::new(&mut source, &control, crate::METADATA_LIMIT).unwrap();
            let index = index::parse(&mut reader, ScanLimits::default(), None).unwrap();
            reader.set_limit(crate::INPUT_LIMIT);
            let mut lz = Lz::default();
            for file in &index.files {
                let mut d = index.parts[file.parts[0]].description.clone();
                if d.directory {
                    continue;
                }
                if d.compression == 0 {
                    lz.reset();
                    continue;
                }
                d.size = file.size;
                let mut packed = Vec::new();
                for p in &file.parts {
                    let part = &index.parts[*p];
                    packed.extend(
                        reader
                            .bytes(
                                part.volume,
                                part.data_at,
                                usize::try_from(part.packed).unwrap(),
                            )
                            .unwrap(),
                    );
                }
                lz.decode(&d, &mut Cursor::new(packed), &mut Sink::default(), &control)
                    .unwrap();
            }
            println!(
                "{} filters delta/e8/e8e9/arm={:?}",
                std::path::Path::new(&group[0])
                    .file_name()
                    .unwrap()
                    .to_string_lossy(),
                lz.filter_counts
            );
        }
    }

    #[test]
    fn failed_batch_settles_only_executed_bytes_and_never_outputs_reservation() {
        #[derive(Default)]
        struct Meter {
            reserved: u64,
            produced: u64,
            bytes: Vec<u8>,
        }
        impl DecodedSink for Meter {
            fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
                self.bytes.extend(b);
                Ok(())
            }
        }
        impl WorkOutput for Meter {
            fn reserve_work(&mut self, n: u64) -> Result<(), DecodeError> {
                self.reserved += n;
                Ok(())
            }
            fn produced_work(&mut self, n: u64) -> Result<(), DecodeError> {
                self.produced += n;
                Ok(())
            }
        }
        let mut lz = Lz {
            window: vec![0; 7],
            ..Lz::default()
        };
        let mut output = Output::new(3);
        let mut meter = Meter::default();
        let mut batch = vec![
            Instruction::Literal(1),
            Instruction::Filter(Filter {
                start: 1,
                length: 1,
                kind: 7,
                channels: 0,
            }),
            Instruction::Literal(2),
            Instruction::Literal(3),
        ];
        assert_eq!(
            lz.execute(
                &mut batch,
                &mut output,
                &mut meter,
                &Cancellation::default()
            ),
            Err(DecodeError::Unsupported)
        );
        assert_eq!((meter.reserved, meter.produced), (3, 1));
        assert!(meter.bytes.is_empty());
    }

    #[test]
    fn truncated_streams_and_unfunded_batches_never_succeed() {
        let packed = literal_vector(0, true);
        let control = Cancellation::default();
        for n in 0..packed.len() {
            let mut lz = Lz::default();
            assert!(
                lz.decode(
                    &description(0, 4, false),
                    &mut Cursor::new(&packed[..n]),
                    &mut Sink::default(),
                    &control
                )
                .is_err(),
                "truncation {n}"
            );
        }
        struct Reject;
        impl DecodedSink for Reject {
            fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
                panic!("unfunded output")
            }
        }
        impl WorkOutput for Reject {
            fn reserve_work(&mut self, _: u64) -> Result<(), DecodeError> {
                Err(DecodeError::Limit)
            }
            fn produced_work(&mut self, _: u64) -> Result<(), DecodeError> {
                panic!("unfunded execution")
            }
        }
        let mut lz = Lz::default();
        assert_eq!(
            lz.decode(
                &description(0, 4, false),
                &mut Cursor::new(packed),
                &mut Reject,
                &control
            ),
            Err(DecodeError::Limit)
        );
        assert_eq!(lz.available, 0);
    }

    #[test]
    fn solid_smaller_and_non_power_declarations_keep_initial_modulus() {
        let c = Cancellation::default();
        let mut lz = Lz::default();
        let mut first = description(0, 4, false);
        first.dictionary = 1024 * 1024;
        lz.decode(
            &first,
            &mut Cursor::new(literal_vector(0, true)),
            &mut Sink::default(),
            &c,
        )
        .unwrap();
        for dictionary in [512 * 1024, 528 * 1024] {
            let mut d = description(0, 4, true);
            d.dictionary = dictionary;
            lz.decode(
                &d,
                &mut Cursor::new([0x43, 0x18, 1, 0x60]),
                &mut Sink::default(),
                &c,
            )
            .unwrap();
            assert_eq!(lz.window.len(), 1024 * 1024);
        }
        let mut grown = description(0, 4, true);
        grown.dictionary = 2 * 1024 * 1024;
        assert_eq!(
            lz.decode(
                &grown,
                &mut Cursor::new([0x43, 0x18, 1, 0x60]),
                &mut Sink::default(),
                &c
            ),
            Err(DecodeError::Unsupported)
        );
        let mut oversized = description(0, 4, false);
        oversized.dictionary = MEMORY_LIMIT;
        let mut lz = Lz::default();
        assert_eq!(
            lz.decode(
                &oversized,
                &mut Cursor::new(literal_vector(0, true)),
                &mut Sink::default(),
                &c
            ),
            Err(DecodeError::Limit)
        );
        assert!(lz.window.is_empty());
    }

    #[test]
    fn paged_admission_preserves_large_solid_window_and_releases_on_new_profile() {
        let control = Cancellation::default();
        let mut lz = Lz::default();
        let mut first = description(1, 4, false);
        first.dictionary = 6 * 1024 * 1024 * 1024;
        assert_eq!(
            lz.memory_required(&first).unwrap(),
            crate::paged_history::MEMORY_BYTES + OTHER_MEMORY
        );
        let mut sink = Sink::default();
        lz.decode(
            &first,
            &mut Cursor::new(literal_vector(1, true)),
            &mut sink,
            &control,
        )
        .unwrap();
        assert_eq!(sink.0, b"ABBA");
        assert!(lz.window.is_empty());
        let (logical, allocated, raw) = lz.paged_history_stats().unwrap();
        assert_eq!((logical, raw), (first.dictionary, 64 * 1024));
        assert!(allocated < 18 * 1024 * 1024);
        let mut sink = Sink::default();
        lz.decode(
            &description(1, 4, true),
            &mut Cursor::new([0x43, 0x18, 1, 0xa0]),
            &mut sink,
            &control,
        )
        .unwrap();
        assert_eq!(sink.0, b"BABA");
        assert_eq!(lz.window_len(), first.dictionary);
        assert_eq!(lz.position, 8);
        assert_eq!(lz.paged_history_stats(), Some((logical, allocated, raw)));

        lz.decode(
            &description(0, 4, false),
            &mut Cursor::new(literal_vector(0, true)),
            &mut Sink::default(),
            &control,
        )
        .unwrap();
        assert!(lz.paged.is_none());
        assert_eq!((lz.window_len(), lz.position), (256 * 1024, 4));
    }

    #[test]
    fn paged_output_failure_does_not_commit_history_and_stays_poisoned() {
        let control = Cancellation::default();
        let mut lz = Lz::default();
        let mut d = description(1, 0, false);
        d.dictionary = 6 * 1024 * 1024 * 1024;
        assert_eq!(
            lz.decode(
                &d,
                &mut Cursor::new(literal_vector(1, true)),
                &mut Sink::default(),
                &control,
            ),
            Err(DecodeError::Integrity)
        );
        assert_eq!((lz.position, lz.available), (0, 0));
        assert_eq!(lz.paged.as_ref().unwrap().read_at(0).unwrap(), 0);
        assert_eq!(lz.paged_history_stats().unwrap().2, 64 * 1024);
        lz.reset();
        d.size = Some(4);
        assert_eq!(
            lz.decode(
                &d,
                &mut Cursor::new(literal_vector(1, true)),
                &mut Sink::default(),
                &control,
            ),
            Err(DecodeError::Integrity)
        );
    }

    #[test]
    fn paged_dictionary_admission_is_not_a_general_large_dictionary_exception() {
        let mut d = description(1, 4, false);
        // The container's separately retained crypto allowance is part of the
        // dense admission threshold even for an unencrypted entry.
        d.dictionary = MEMORY_LIMIT - OTHER_MEMORY;
        assert!(!dense_fits(d.dictionary));
        assert_eq!(
            Lz::default().memory_required(&d).unwrap(),
            crate::paged_history::MEMORY_BYTES + OTHER_MEMORY
        );
        assert!(dense_fits(
            d.dictionary - crate::protection::Protection::MEMORY_BYTES
        ));
        d.dictionary = crate::paged_history::MAX_LOGICAL_LEN;
        assert!(paged_memory_required(&d).is_ok());
        d.dictionary += 1;
        assert_eq!(paged_memory_required(&d), Err(DecodeError::Limit));
        d.dictionary = 6 * 1024 * 1024 * 1024;
        for field in 0..5 {
            let mut other = d.clone();
            match field {
                0 => other.legacy = true,
                1 => other.ancient = true,
                2 => other.method = 50,
                3 => other.effective = 0,
                _ => other.compression = 0,
            }
            assert_eq!(paged_memory_required(&other), Err(DecodeError::Limit));
        }
    }
}
