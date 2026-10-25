use crate::algorithm::WorkOutput;
use crate::decode_error;
use crate::filter29_transform::{self, Filter, Kind};
use crate::lz_bits::BitRead;
use crate::lz29_bits::Stream;
use std::collections::VecDeque;
use std::io::Cursor;
use xdzip_codec_api::{Cancellation, payload::DecodeError};
use xdzip_compression::Crc32;

fn compact(input: &mut Stream<'_>) -> Result<u32, DecodeError> {
    match input.get(2)? {
        0 => Ok(input.get(4)? as u32),
        1 => {
            let high = input.get(4)? as u32;
            if high == 0 {
                Ok(0xffffff00 | input.get(8)? as u32)
            } else {
                Ok((high << 4) | input.get(4)? as u32)
            }
        }
        2 => Ok(input.get(16)? as u32),
        3 => Ok(input.get(32)? as u32),
        _ => Err(DecodeError::Integrity),
    }
}
struct Definition {
    kind: Kind,
    length: usize,
}
#[derive(Default)]
pub(crate) struct Definitions {
    items: Vec<Definition>,
    last: usize,
}
impl Definitions {
    pub fn clear(&mut self) {
        self.items.clear();
        self.last = 0;
    }
    pub fn declare(
        &mut self,
        flags: u8,
        packet: &[u8],
        position: u64,
        output: &mut Output,
    ) -> Result<(), DecodeError> {
        if packet.is_empty() || packet.len() > 32768 {
            return Err(DecodeError::Limit);
        }
        let mut cursor = Cursor::new(packet);
        let mut input = Stream::new(&mut cursor);
        let selected = if flags & 0x80 != 0 {
            let selected = compact(&mut input)?;
            if selected == 0 {
                output.reset()?;
                self.clear();
                0
            } else {
                usize::try_from(selected - 1).map_err(|_| DecodeError::Integrity)?
            }
        } else {
            self.last
        };
        if selected > self.items.len() {
            return Err(DecodeError::Integrity);
        }
        let start = position
            .checked_add(u64::from(compact(&mut input)?))
            .and_then(|n| n.checked_add(if flags & 0x40 != 0 { 258 } else { 0 }))
            .ok_or(DecodeError::Integrity)?;
        let length = if flags & 0x20 != 0 {
            usize::try_from(compact(&mut input)?).map_err(|_| DecodeError::Integrity)?
        } else {
            self.items.get(selected).map_or(0, |item| item.length)
        };
        if length == 0 || length >= 262144 {
            return Err(DecodeError::Integrity);
        }
        let mut parameters = [0; 7];
        parameters[4] = length as u32;
        if flags & 0x10 != 0 {
            let mask = input.get(7)?;
            for (i, value) in parameters.iter_mut().enumerate() {
                if mask & (1 << i) != 0 {
                    *value = compact(&mut input)?;
                }
            }
        }
        if parameters[4] != length as u32 {
            return Err(DecodeError::Integrity);
        }
        let kind = if selected == self.items.len() {
            if self.items.len() >= 8192 {
                return Err(DecodeError::Limit);
            }
            let length = compact(&mut input)?;
            if length == 0 || length >= 65536 {
                return Err(DecodeError::Integrity);
            }
            let mut check = 0;
            let mut crc = Crc32::new();
            for _ in 0..length {
                let byte = input.get(8)? as u8;
                check ^= byte;
                crc.update(&[byte]);
            }
            if check != 0 {
                return Err(DecodeError::Integrity);
            }
            let kind = Kind::identify(length as usize, crc.checksum())?;
            self.items.push(Definition { kind, length: 0 });
            kind
        } else {
            self.items[selected].kind
        };
        input.finish(0)?;
        self.items[selected].length = length;
        self.last = selected;
        output.register(Filter {
            start,
            length,
            kind,
            parameters,
        })
    }
}

/// Original prefilter bytes are temporary here; the dictionary separately owns
/// its history. Chained standard transforms operate on bounded result copies.
pub(crate) struct Output {
    bytes: Vec<u8>,
    used: usize,
    emitted: u64,
    produced: u64,
    expected: u64,
    pending: VecDeque<Filter>,
    last: Option<(u64, usize)>,
}
impl Output {
    pub fn new(expected: u64) -> Self {
        Self {
            bytes: Vec::new(),
            used: 0,
            emitted: 0,
            produced: 0,
            expected,
            pending: VecDeque::new(),
            last: None,
        }
    }
    pub fn position(&self) -> u64 {
        self.produced
    }
    fn reset(&mut self) -> Result<(), DecodeError> {
        if !self.pending.is_empty() {
            return Err(DecodeError::Integrity);
        }
        self.last = None;
        Ok(())
    }
    fn register(&mut self, filter: Filter) -> Result<(), DecodeError> {
        if self.pending.len() >= 8192 {
            return Err(DecodeError::Limit);
        }
        let end = filter
            .start
            .checked_add(filter.length as u64)
            .filter(|n| *n <= self.expected)
            .ok_or(DecodeError::Integrity)?;
        if filter.start < self.emitted || filter.length == 0 || end == filter.start {
            return Err(DecodeError::Integrity);
        }
        if let Some((start, length)) = self.last {
            if filter.start == start {
                if filter.length != length {
                    return Err(DecodeError::Integrity);
                }
            } else if filter.start < start + length as u64 {
                return Err(DecodeError::Integrity);
            }
        }
        self.last = Some((filter.start, filter.length));
        self.pending.push_back(filter);
        Ok(())
    }
    pub fn put(&mut self, byte: u8) -> Result<(), DecodeError> {
        self.produced = self
            .produced
            .checked_add(1)
            .filter(|n| *n <= self.expected)
            .ok_or(DecodeError::Integrity)?;
        if self.bytes.len() - self.used >= 262144 + 8192 {
            return Err(DecodeError::Limit);
        }
        self.bytes.try_reserve(1).map_err(|_| DecodeError::Limit)?;
        self.bytes.push(byte);
        Ok(())
    }
    fn deliver(
        bytes: &[u8],
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        for chunk in bytes.chunks(8192) {
            control.check().map_err(decode_error)?;
            sink.write_precharged_chunk(chunk)?;
        }
        Ok(())
    }
    pub fn flush(
        &mut self,
        sink: &mut dyn WorkOutput,
        control: &Cancellation,
        final_block: bool,
    ) -> Result<(), DecodeError> {
        loop {
            control.check().map_err(decode_error)?;
            let available = self.bytes.len() - self.used;
            if available == 0 {
                break;
            }
            let prefix = self
                .pending
                .front()
                .map_or(available as u64, |filter| filter.start - self.emitted)
                .min(available as u64) as usize;
            if prefix != 0 {
                Self::deliver(&self.bytes[self.used..self.used + prefix], sink, control)?;
                self.used += prefix;
                self.emitted += prefix as u64;
                continue;
            }
            let filter = self.pending.front().ok_or(DecodeError::Integrity)?;
            if filter.start != self.emitted {
                return Err(DecodeError::Integrity);
            }
            if filter.length > available {
                break;
            }
            let length = filter.length;
            let mut result = filter29_transform::run(
                filter,
                &self.bytes[self.used..self.used + length],
                sink,
                control,
            )?;
            self.pending.pop_front();
            while self
                .pending
                .front()
                .is_some_and(|next| next.start == self.emitted)
            {
                let next = self.pending.front().ok_or(DecodeError::Integrity)?;
                if next.length != result.len() {
                    return Err(DecodeError::Integrity);
                }
                result = filter29_transform::run(next, &result, sink, control)?;
                self.pending.pop_front();
            }
            Self::deliver(&result, sink, control)?;
            self.used += length;
            self.emitted += length as u64;
        }
        if self.used != 0 {
            self.bytes.drain(..self.used);
            self.used = 0;
        }
        if final_block
            && (self.produced != self.expected
                || self.emitted != self.expected
                || !self.pending.is_empty())
        {
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xdzip_codec_api::payload::DecodedSink;
    fn item(start: u64, length: usize) -> Filter {
        let mut parameters = [0; 7];
        parameters[0] = 1;
        parameters[4] = length as u32;
        Filter {
            start,
            length,
            kind: Kind::Delta,
            parameters,
        }
    }
    #[derive(Default)]
    struct Sink {
        bytes: Vec<u8>,
        work: u64,
        reserved: u64,
        maximum: u64,
    }
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.bytes.extend(bytes);
            Ok(())
        }
    }
    impl WorkOutput for Sink {
        fn reserve_aux(&mut self, n: u64) -> Result<(), DecodeError> {
            if self.maximum != 0 && self.work + self.reserved + n > self.maximum {
                return Err(DecodeError::Limit);
            }
            self.reserved += n;
            Ok(())
        }
        fn produced_aux(&mut self, n: u64) -> Result<(), DecodeError> {
            self.reserved -= n;
            self.work += n;
            Ok(())
        }
    }
    #[test]
    fn compact_values_keep_sign_extension_and_exact_consumption() {
        for (bytes, value) in [
            (vec![0x3c], 15),
            (vec![0x44, 0], 16),
            (vec![0x43, 0xf8], 0xfffffffe),
            (vec![0x84, 0x8d, 0], 0x1234),
        ] {
            assert_eq!(
                compact(&mut Stream::new(&mut Cursor::new(bytes))).unwrap(),
                value
            );
        }
    }
    #[test]
    fn chains_transform_one_region_and_each_transform_consumes_auxiliary_work() {
        let cancel = Cancellation::default();
        for maximum in [0, 4] {
            let mut output = Output::new(3);
            output.register(item(0, 3)).unwrap();
            output.register(item(0, 3)).unwrap();
            for byte in [1, 2, 3] {
                output.put(byte).unwrap();
            }
            let mut sink = Sink {
                maximum,
                ..Sink::default()
            };
            let result = output.flush(&mut sink, &cancel, true);
            if maximum == 0 {
                result.unwrap();
                assert_eq!(sink.bytes, [1, 4, 10]);
                assert_eq!(sink.work, 6);
            } else {
                assert_eq!(result, Err(DecodeError::Limit));
                assert!(sink.bytes.is_empty());
                assert_eq!(sink.work, 3);
            }
            assert_eq!(sink.reserved, 0);
        }
    }
    #[test]
    fn pending_overlap_reset_incomplete_ranges_and_queue_limit_fail_closed() {
        let mut output = Output::new(8);
        output.register(item(1, 3)).unwrap();
        assert_eq!(output.register(item(2, 3)), Err(DecodeError::Integrity));
        assert_eq!(output.register(item(1, 2)), Err(DecodeError::Integrity));
        assert_eq!(output.reset(), Err(DecodeError::Integrity));
        assert_eq!(
            output.flush(&mut Sink::default(), &Cancellation::default(), true),
            Err(DecodeError::Integrity)
        );
        let mut output = Output::new(1);
        for _ in 0..8192 {
            output.register(item(0, 1)).unwrap();
        }
        assert_eq!(output.register(item(0, 1)), Err(DecodeError::Limit));
    }
    #[test]
    fn unknown_program_never_becomes_unfiltered_success_and_cached_lengths_survive() {
        let mut definitions = Definitions::default();
        let mut output = Output::new(5);
        // Fixed container math: reset/select0, start0, length4, program1=00.
        assert_eq!(
            definitions.declare(0xa0, &[0, 1, 1, 0], 0, &mut output),
            Err(DecodeError::Unsupported)
        );
        assert_eq!(
            definitions.declare(0xa0, &[0, 1, 1, 1], 0, &mut output),
            Err(DecodeError::Integrity)
        );
        definitions.items.push(Definition {
            kind: Kind::E8,
            length: 5,
        });
        definitions.declare(0, &[0], 0, &mut output).unwrap();
        assert_eq!(output.pending[0].length, 5);
        assert_eq!(output.pending[0].kind, Kind::E8);
    }
}
