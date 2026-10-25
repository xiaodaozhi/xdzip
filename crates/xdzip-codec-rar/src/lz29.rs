use crate::algorithm::WorkOutput;
use crate::filter29::{Definitions, Output};
use crate::index::Descriptor;
use crate::lz_bits::BitRead;
use crate::lz29_bits::{Stream, Tables};
use crate::{MEMORY_LIMIT, decode_error};
use std::io::Read;
use xdzip_codec_api::{Cancellation, payload::DecodeError};

const OTHER_MEMORY: u64 = 288 * 1024 * 1024;
enum Action {
    Byte(u8),
    Copy(u64, usize),
    None,
    Table,
    End(bool),
    Filter,
}

impl Action {
    fn length(&self) -> usize {
        match self {
            Self::Byte(_) => 1,
            Self::Copy(_, n) => *n,
            _ => 0,
        }
    }
}

/// Generation29's state is distinct from modern LZ even when the window size
/// agrees. Its differential code lengths, low-distance repeat and EOF controls
/// have independent lifetimes. Store never invokes this state machine.
pub(crate) struct Lz29 {
    window: Vec<u8>,
    position: usize,
    available: u64,
    recent: [Option<u64>; 4],
    last_length: usize,
    tables: Option<Tables>,
    baseline: [u8; 404],
    low_previous: u64,
    low_repeats: u8,
    reuse: bool,
    failed: bool,
    filters: Definitions,
    ppm: crate::ppm29::Model,
    ppm_mode: bool,
    escape: u8,
}
impl Default for Lz29 {
    fn default() -> Self {
        Self {
            window: Vec::new(),
            position: 0,
            available: 0,
            recent: [None; 4],
            last_length: 0,
            tables: None,
            baseline: [0; 404],
            low_previous: 0,
            low_repeats: 0,
            reuse: false,
            failed: false,
            filters: Definitions::default(),
            ppm: crate::ppm29::Model::default(),
            ppm_mode: false,
            escape: 2,
        }
    }
}
impl Lz29 {
    pub fn reset(&mut self) {
        self.ppm.clear();
        self.reset_lz();
    }
    fn reset_lz(&mut self) {
        self.position = 0;
        self.available = 0;
        self.recent = [None; 4];
        self.last_length = 0;
        self.tables = None;
        self.baseline.fill(0);
        self.low_previous = 0;
        self.low_repeats = 0;
        self.reuse = false;
        self.filters.clear();
        self.ppm_mode = false;
        self.escape = 2;
    }
    pub fn memory_required(&self, d: &Descriptor) -> Result<u64, DecodeError> {
        if d.method != 29 || !(1..=5).contains(&d.compression) || d.encrypted || d.hash.is_some() {
            return Err(DecodeError::Unsupported);
        }
        let window = if d.solid && !self.window.is_empty() {
            self.window.len() as u64
        } else {
            d.dictionary.max(65536)
        };
        if window > 4 * 1024 * 1024 {
            return Err(DecodeError::Unsupported);
        }
        window
            .checked_add(OTHER_MEMORY)
            .and_then(|n| n.checked_add(self.ppm.allocated_bytes()))
            .filter(|n| *n <= MEMORY_LIMIT)
            .ok_or(DecodeError::Limit)
    }
    fn begin(&mut self, d: &Descriptor) -> Result<(), DecodeError> {
        self.memory_required(d)?;
        if d.solid {
            if self.window.is_empty() || d.dictionary.max(65536) > self.window.len() as u64 {
                return Err(DecodeError::Unsupported);
            }
        } else {
            self.reset_lz();
            let size = usize::try_from(d.dictionary.max(65536)).map_err(|_| DecodeError::Limit)?;
            if self.window.len() != size {
                self.window = Vec::new();
                self.window
                    .try_reserve_exact(size)
                    .map_err(|_| DecodeError::Limit)?;
                self.window.resize(size, 0);
            }
        }
        Ok(())
    }
    fn read_table(
        &mut self,
        input: &mut Stream<'_>,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        input.align();
        if input.peek_byte()? & 0x80 != 0 {
            let maximum = MEMORY_LIMIT
                .checked_sub(OTHER_MEMORY)
                .and_then(|n| n.checked_sub(self.window.len() as u64))
                .and_then(|n| n.checked_sub(crate::protection::Protection::MEMORY_BYTES))
                .ok_or(DecodeError::Limit)?;
            let before = self.ppm.initialization_work();
            let result = self
                .ppm
                .init(input, control, maximum, sink.remaining_aux_work());
            self.settle_initialization(before, sink)?;
            if let Some(escape) = result? {
                self.escape = escape;
            }
            self.ppm_mode = true;
            self.reuse = false;
            return Ok(());
        }
        self.ppm_mode = false;
        self.tables = Some(Tables::read(input, &mut self.baseline)?);
        self.low_previous = 0;
        self.low_repeats = 0;
        self.reuse = true;
        Ok(())
    }
    fn settle_initialization(
        &self,
        before: u64,
        sink: &mut dyn WorkOutput,
    ) -> Result<(), DecodeError> {
        let actual = self
            .ppm
            .initialization_work()
            .checked_sub(before)
            .ok_or(DecodeError::Integrity)?;
        if actual != 0 {
            // The model admitted this exact call against remaining_aux_work
            // before touching its arena. No outer work changes while borrowed.
            // Only its monotonic executed delta is settled, including failure.
            sink.reserve_aux(actual)?;
            sink.produced_aux(actual)?;
        }
        Ok(())
    }
    fn ppm_byte(
        &mut self,
        input: &mut Stream<'_>,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<u8, DecodeError> {
        sink.reserve_aux(1)?;
        let before = self.ppm.initialization_work();
        let result = self
            .ppm
            .decode_byte(input, control, sink.remaining_aux_work());
        self.settle_initialization(before, sink)?;
        sink.produced_aux(u64::from(result.is_ok()))?;
        result
    }
    fn filter_byte(
        &mut self,
        input: &mut Stream<'_>,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<u8, DecodeError> {
        if self.ppm_mode {
            self.ppm_byte(input, sink, control)
        } else {
            Ok(input.get(8)? as u8)
        }
    }
    fn ppm_action(
        &mut self,
        input: &mut Stream<'_>,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<Action, DecodeError> {
        let byte = self.ppm_byte(input, sink, control)?;
        if byte != self.escape {
            return Ok(Action::Byte(byte));
        }
        match self.ppm_byte(input, sink, control)? {
            0 => Ok(Action::Table),
            1 => Ok(Action::Byte(self.escape)),
            2 => Ok(Action::End(false)),
            3 => Ok(Action::Filter),
            4 => {
                let mut distance = 0;
                for _ in 0..3 {
                    distance = (distance << 8) | u64::from(self.ppm_byte(input, sink, control)?);
                }
                let length = usize::from(self.ppm_byte(input, sink, control)?) + 32;
                Ok(Action::Copy(distance + 2, length))
            }
            5 => Ok(Action::Copy(
                1,
                usize::from(self.ppm_byte(input, sink, control)?) + 4,
            )),
            _ => Ok(Action::Byte(self.escape)),
        }
    }
    fn length(slot: u16, input: &mut Stream<'_>) -> Result<usize, DecodeError> {
        if slot >= 28 {
            return Err(DecodeError::Integrity);
        }
        if slot < 8 {
            return Ok(usize::from(slot));
        }
        let width = (slot / 4 - 1) as u8;
        Ok(((4 + usize::from(slot % 4)) << width) + input.get(width)? as usize)
    }
    fn distance(&mut self, input: &mut Stream<'_>) -> Result<u64, DecodeError> {
        let table = self.tables.as_ref().ok_or(DecodeError::Integrity)?;
        let slot = u64::from(table.distance.symbol(input)?);
        let (base, width) = match slot {
            0..=3 => (slot, 0),
            4..=33 => {
                let width = (slot / 2 - 1) as u8;
                ((2 + slot % 2) << width, width)
            }
            34..=47 => (131072 + (slot - 34) * 65536, 16),
            48..=59 => (1048576 + (slot - 48) * 262144, 18),
            _ => return Err(DecodeError::Integrity),
        };
        let extra = if slot <= 9 {
            input.get(width)?
        } else {
            let high = input.get(width - 4)?;
            let low = if self.low_repeats != 0 {
                self.low_repeats -= 1;
                self.low_previous
            } else {
                let low = table.low.symbol(input)?;
                if low == 16 {
                    self.low_repeats = 15;
                    self.low_previous
                } else {
                    self.low_previous = u64::from(low);
                    u64::from(low)
                }
            };
            high * 16 + low
        };
        Ok(base + 1 + extra)
    }
    fn remember(&mut self, distance: u64, length: usize) -> Action {
        self.recent.rotate_right(1);
        self.recent[0] = Some(distance);
        self.last_length = length;
        Action::Copy(distance, length)
    }
    fn action(&mut self, input: &mut Stream<'_>) -> Result<Action, DecodeError> {
        let table = self.tables.as_ref().ok_or(DecodeError::Integrity)?;
        let symbol = table.main.symbol(input)?;
        match symbol {
            0..=255 => Ok(Action::Byte(symbol as u8)),
            256 => {
                if input.get(1)? != 0 {
                    Ok(Action::Table)
                } else {
                    Ok(Action::End(input.get(1)? == 0))
                }
            }
            257 => Ok(Action::Filter),
            258 => {
                if self.last_length == 0 {
                    Ok(Action::None)
                } else {
                    Ok(Action::Copy(
                        self.recent[0].ok_or(DecodeError::Integrity)?,
                        self.last_length,
                    ))
                }
            }
            259..=262 => {
                let index = usize::from(symbol - 259);
                let distance = self.recent[index].ok_or(DecodeError::Integrity)?;
                let length = Self::length(table.repeat.symbol(input)?, input)? + 2;
                self.recent[..=index].rotate_right(1);
                self.last_length = length;
                Ok(Action::Copy(distance, length))
            }
            263..=270 => {
                let (base, width) = [
                    (1, 2),
                    (5, 2),
                    (9, 3),
                    (17, 4),
                    (33, 5),
                    (65, 6),
                    (129, 6),
                    (193, 6),
                ][usize::from(symbol - 263)];
                let distance = base + input.get(width)?;
                Ok(self.remember(distance, 2))
            }
            271..=298 => {
                let mut length = Self::length(symbol - 271, input)? + 3;
                let distance = self.distance(input)?;
                length += usize::from(distance >= 8192) + usize::from(distance >= 262144);
                Ok(self.remember(distance, length))
            }
            _ => Err(DecodeError::Integrity),
        }
    }
    fn put(&mut self, byte: u8, output: &mut Output) -> Result<(), DecodeError> {
        output.put(byte)?;
        self.window[self.position] = byte;
        self.position += 1;
        if self.position == self.window.len() {
            self.position = 0;
        }
        self.available = (self.available + 1).min(self.window.len() as u64);
        Ok(())
    }
    fn execute(
        &mut self,
        batch: &mut Vec<Action>,
        output: &mut Output,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        let planned = batch.iter().map(Action::length).sum::<usize>();
        if planned > 8192 {
            return Err(DecodeError::Limit);
        }
        sink.reserve_work(planned as u64)?;
        let before = output.position();
        let result = (|| {
            for action in batch.drain(..) {
                control.check().map_err(decode_error)?;
                match action {
                    Action::Byte(byte) => self.put(byte, output)?,
                    Action::Copy(distance, length) => {
                        if distance == 0
                            || distance > self.available
                            || distance > self.window.len() as u64
                        {
                            return Err(DecodeError::Integrity);
                        }
                        for _ in 0..length {
                            let source = (self.position + self.window.len() - distance as usize)
                                % self.window.len();
                            self.put(self.window[source], output)?;
                        }
                    }
                    Action::None => (),
                    _ => return Err(DecodeError::Integrity),
                }
            }
            Ok(())
        })();
        sink.produced_work(output.position() - before)?;
        result?;
        output.flush(sink, control, false)
    }
    fn run(
        &mut self,
        d: &Descriptor,
        source: &mut dyn Read,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
        padding: u8,
    ) -> Result<(), DecodeError> {
        self.begin(d)?;
        let mut input = Stream::new(source);
        if !self.reuse {
            self.read_table(&mut input, sink, control)?;
        }
        let mut output = Output::new(d.size.ok_or(DecodeError::Unsupported)?);
        let mut batch = Vec::with_capacity(256);
        let mut planned = 0;
        loop {
            control.check().map_err(decode_error)?;
            let action = if self.ppm_mode {
                self.ppm_action(&mut input, sink, control)?
            } else {
                self.action(&mut input)?
            };
            match action {
                Action::Filter => {
                    self.execute(&mut batch, &mut output, sink, control)?;
                    planned = 0;
                    let flags = self.filter_byte(&mut input, sink, control)?;
                    let length = match (flags & 7) + 1 {
                        7 => u64::from(self.filter_byte(&mut input, sink, control)?) + 7,
                        8 => {
                            (u64::from(self.filter_byte(&mut input, sink, control)?) << 8)
                                | u64::from(self.filter_byte(&mut input, sink, control)?)
                        }
                        n => u64::from(n),
                    };
                    if length == 0 || length > 32768 {
                        return Err(DecodeError::Limit);
                    }
                    let mut packet = Vec::new();
                    packet
                        .try_reserve_exact(length as usize)
                        .map_err(|_| DecodeError::Limit)?;
                    for _ in 0..length {
                        packet.push(self.filter_byte(&mut input, sink, control)?);
                    }
                    self.filters
                        .declare(flags, &packet, output.position(), &mut output)?;
                }
                Action::Table => {
                    self.execute(&mut batch, &mut output, sink, control)?;
                    planned = 0;
                    self.read_table(&mut input, sink, control)?;
                }
                Action::End(reuse) => {
                    self.reuse = reuse;
                    break;
                }
                action => {
                    let length = action.length();
                    if batch.len() == 256 || planned + length > 8192 {
                        self.execute(&mut batch, &mut output, sink, control)?;
                        planned = 0;
                    }
                    planned += length;
                    batch.push(action);
                }
            }
        }
        self.execute(&mut batch, &mut output, sink, control)?;
        output.flush(sink, control, true)?;
        if self.ppm_mode {
            // RAR29 PPM terminates by escape+2, with no range/code-zero or
            // fixed trailing-byte rule. Drain only this admitted packed range;
            // trailing bytes never enter the dictionary, content hash or sink.
            input.discard_remaining(control)?;
        } else {
            input.finish(padding)?;
        }
        control.check().map_err(decode_error)
    }
    pub fn decode(
        &mut self,
        d: &Descriptor,
        input: &mut dyn Read,
        output: &mut dyn WorkOutput,
        control: &Cancellation,
        padding: u8,
    ) -> Result<(), DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let result = if padding > 15 {
            Err(DecodeError::Unsupported)
        } else {
            self.run(d, input, output, control, padding)
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
    use std::io::Cursor;
    use xdzip_codec_api::payload::DecodedSink;
    #[derive(Default)]
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend(bytes);
            Ok(())
        }
    }
    impl WorkOutput for Sink {}
    fn description(size: u64, solid: bool) -> Descriptor {
        Descriptor {
            method: 29,
            compression: 3,
            dictionary: 65536,
            size: Some(size),
            solid,
            ..Descriptor::default()
        }
    }
    // Fixed declarative mathematical vector: meta0=1/meta1=2/meta2=2;
    // main A/B length2, EOF length1, all other404 lengths0. Data ABBA.
    // This does not accept input or implement an archive compression encoder.
    fn literal(control: u8) -> Vec<u8> {
        let mut bytes = vec![0; 63];
        bytes[0] = 0x04;
        bytes[1] = 0x88;
        bytes[18] = 0x1e;
        bytes[42] = 0x08;
        bytes[61] = 0x5f;
        bytes[62] = control;
        bytes
    }
    #[test]
    fn literal_tables_file_controls_and_solid_baseline_have_distinct_lifetimes() {
        let cancel = Cancellation::default();
        let mut codec = Lz29::default();
        let mut sink = Sink::default();
        codec
            .decode(
                &description(4, false),
                &mut Cursor::new(literal(0)),
                &mut sink,
                &cancel,
                0,
            )
            .unwrap();
        assert_eq!(sink.0, b"ABBA");
        sink.0.clear();
        codec
            .decode(
                &description(4, true),
                &mut Cursor::new([0xbe, 0]),
                &mut sink,
                &cancel,
                0,
            )
            .unwrap();
        assert_eq!(sink.0, b"ABBA");
        let mut first = literal(0x20); // EOF followed1: new table in same file.
        first.extend(literal(0));
        let mut sink = Sink::default();
        Lz29::default()
            .decode(
                &description(8, false),
                &mut Cursor::new(first),
                &mut sink,
                &cancel,
                0,
            )
            .unwrap();
        assert_eq!(sink.0, b"ABBAABBA");
        let mut codec = Lz29::default();
        codec
            .decode(
                &description(4, false),
                &mut Cursor::new(literal(0x10)),
                &mut Sink::default(),
                &cancel,
                0,
            )
            .unwrap();
        let mut retained = vec![0; 63];
        retained[0] = 0x44;
        retained[60] = 2;
        retained[61] = 0xf8;
        let mut sink = Sink::default();
        codec
            .decode(
                &description(4, true),
                &mut Cursor::new(retained),
                &mut sink,
                &cancel,
                0,
            )
            .unwrap();
        assert_eq!(sink.0, b"ABBA");
    }
    fn tree(length: usize, symbol: usize) -> crate::lz_bits::Huffman {
        let mut lengths = vec![0; length];
        lengths[symbol] = 1;
        crate::lz_bits::Huffman::new(&lengths).unwrap()
    }
    #[test]
    fn low_nibble_repeat_is_sixteen_requests_and_old_lengths_are_not_modern() {
        let mut codec = Lz29::default();
        let mut low = vec![0; 17];
        low[15] = 1;
        low[16] = 1;
        codec.tables = Some(Tables {
            main: tree(299, 0),
            distance: tree(60, 10),
            low: crate::lz_bits::Huffman::new(&low).unwrap(),
            repeat: tree(28, 0),
        });
        let mut cursor = Cursor::new([0x10, 0, 0]);
        let mut input = Stream::new(&mut cursor);
        for request in 0..18 {
            assert_eq!(codec.distance(&mut input).unwrap(), 48);
            if request == 1 {
                assert_eq!(codec.low_repeats, 15);
            }
            if request == 16 {
                assert_eq!(codec.low_repeats, 0);
            }
        }
        for (slot, minimum, maximum) in [(0, 0, 0), (7, 7, 7), (8, 8, 9), (27, 224, 255)] {
            let mut zeros = Cursor::new([0; 4]);
            assert_eq!(
                Lz29::length(slot, &mut Stream::new(&mut zeros)).unwrap(),
                minimum
            );
            let mut ones = Cursor::new([255; 4]);
            assert_eq!(
                Lz29::length(slot, &mut Stream::new(&mut ones)).unwrap(),
                maximum
            );
        }
    }
    #[test]
    fn truncation_missing_terminator_and_tail_failures_poison_the_state() {
        let cancel = Cancellation::default();
        for end in 0..63 {
            assert!(
                Lz29::default()
                    .decode(
                        &description(4, false),
                        &mut Cursor::new(&literal(0)[..end]),
                        &mut Sink::default(),
                        &cancel,
                        0
                    )
                    .is_err()
            );
        }
        for tail in 1..=16 {
            let mut bytes = literal(0);
            bytes.extend(vec![0xa5; tail]);
            let mut codec = Lz29::default();
            assert_eq!(
                codec.decode(
                    &description(4, false),
                    &mut Cursor::new(&bytes),
                    &mut Sink::default(),
                    &cancel,
                    0
                ),
                Err(DecodeError::Integrity)
            );
            assert_eq!(
                codec.decode(
                    &description(4, false),
                    &mut Cursor::new(literal(0)),
                    &mut Sink::default(),
                    &cancel,
                    0
                ),
                Err(DecodeError::Integrity)
            );
            let result = Lz29::default().decode(
                &description(4, false),
                &mut Cursor::new(bytes),
                &mut Sink::default(),
                &cancel,
                15,
            );
            assert_eq!(result.is_ok(), tail <= 15);
        }
        assert_eq!(
            Lz29::default().decode(
                &description(4, true),
                &mut Cursor::new([0xbe, 0]),
                &mut Sink::default(),
                &cancel,
                0
            ),
            Err(DecodeError::Unsupported)
        );
    }
}
