use crate::decode_error;
use std::collections::VecDeque;
use xdzip_codec_api::Cancellation;
use xdzip_codec_api::payload::{DecodeError, DecodedSink};

const BLOCK_LIMIT: usize = 4 * 1024 * 1024;
pub(crate) struct Filter {
    pub start: u64,
    pub length: usize,
    pub kind: u8,
    pub channels: u8,
}
fn transform(
    filter: &Filter,
    mut bytes: Vec<u8>,
    control: &Cancellation,
) -> Result<Vec<u8>, DecodeError> {
    match filter.kind {
        0 => {
            let mut out = Vec::new();
            out.try_reserve_exact(bytes.len())
                .map_err(|_| DecodeError::Limit)?;
            out.resize(bytes.len(), 0);
            let mut source = 0;
            for channel in 0..usize::from(filter.channels) {
                let mut value = 0u8;
                for at in (channel..bytes.len()).step_by(usize::from(filter.channels)) {
                    if at & 8191 == 0 {
                        control.check().map_err(decode_error)?;
                    }
                    value = value.wrapping_sub(bytes[source]);
                    out[at] = value;
                    source += 1;
                }
            }
            Ok(out)
        }
        1 | 2 => {
            let mut at = 0usize;
            while at + 4 < bytes.len() {
                if at & 8191 == 0 {
                    control.check().map_err(decode_error)?;
                }
                if bytes[at] != 0xe8 && (filter.kind != 2 || bytes[at] != 0xe9) {
                    at += 1;
                    continue;
                }
                let a = i32::from_le_bytes(
                    bytes[at + 1..at + 5]
                        .try_into()
                        .map_err(|_| DecodeError::Integrity)?,
                ) as i64;
                let q = ((filter.start + at as u64 + 1) & 0xffffff) as i64;
                let value = if a < 0 && a + q >= 0 {
                    a + (1 << 24)
                } else if (0..1 << 24).contains(&a) {
                    a - q
                } else {
                    a
                };
                bytes[at + 1..at + 5].copy_from_slice(&(value as i32).to_le_bytes());
                at += 5;
            }
            Ok(bytes)
        }
        3 => {
            for (i, instruction) in bytes.as_chunks_mut::<4>().0.iter_mut().enumerate() {
                if i & 2047 == 0 {
                    control.check().map_err(decode_error)?;
                }
                if instruction[3] == 0xeb {
                    let a = u32::from_le_bytes([instruction[0], instruction[1], instruction[2], 0]);
                    let value =
                        a.wrapping_sub(((filter.start + i as u64 * 4) / 4) as u32) & 0xffffff;
                    instruction[..3].copy_from_slice(&value.to_le_bytes()[..3]);
                }
            }
            Ok(bytes)
        }
        _ => Err(DecodeError::Unsupported),
    }
}
pub(crate) struct Output {
    pending: Vec<u8>,
    used: usize,
    emitted: u64,
    produced: u64,
    expected: u64,
    filters: VecDeque<Filter>,
    last_end: u64,
}
impl Output {
    pub fn new(expected: u64) -> Self {
        Self {
            pending: Vec::with_capacity(8192),
            used: 0,
            emitted: 0,
            produced: 0,
            expected,
            filters: VecDeque::new(),
            last_end: 0,
        }
    }
    pub fn position(&self) -> u64 {
        self.produced
    }
    pub fn register(&mut self, filter: Filter) -> Result<(), DecodeError> {
        if filter.kind > 3 {
            return Err(DecodeError::Unsupported);
        }
        if filter.kind == 0 && !(1..=32).contains(&filter.channels) {
            return Err(DecodeError::Integrity);
        }
        if filter.length > BLOCK_LIMIT || self.filters.len() >= 8192 {
            return Err(DecodeError::Limit);
        }
        let end = filter
            .start
            .checked_add(filter.length as u64)
            .filter(|n| *n <= self.expected)
            .ok_or(DecodeError::Integrity)?;
        if filter.start < self.produced
            || filter.start < self.emitted
            || filter.start < self.last_end
        {
            return Err(DecodeError::Integrity);
        }
        if filter.length == 0 {
            return Ok(());
        }
        self.last_end = end;
        self.filters.push_back(filter);
        Ok(())
    }
    pub fn put(&mut self, byte: u8) -> Result<(), DecodeError> {
        if self.produced >= self.expected {
            return Err(DecodeError::Integrity);
        }
        if self.pending.len() == self.pending.capacity() {
            self.pending
                .try_reserve(8192)
                .map_err(|_| DecodeError::Limit)?;
        }
        if self.pending.len() - self.used > BLOCK_LIMIT + 8192 {
            return Err(DecodeError::Limit);
        }
        self.pending.push(byte);
        self.produced += 1;
        Ok(())
    }
    fn compact(&mut self) {
        if self.used == self.pending.len() {
            self.pending.clear();
            self.used = 0;
        } else if self.used >= 8192 {
            self.pending.drain(..self.used);
            self.used = 0;
        }
    }
    pub fn flush(
        &mut self,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        final_output: bool,
    ) -> Result<(), DecodeError> {
        loop {
            control.check().map_err(decode_error)?;
            let available = self.pending.len() - self.used;
            if available == 0 {
                break;
            }
            if let Some(filter) = self.filters.front() {
                if self.emitted == filter.start {
                    if available < filter.length {
                        break;
                    }
                    let filter = self.filters.pop_front().ok_or(DecodeError::Integrity)?;
                    let mut bytes = Vec::new();
                    bytes
                        .try_reserve_exact(filter.length)
                        .map_err(|_| DecodeError::Limit)?;
                    bytes.extend_from_slice(&self.pending[self.used..self.used + filter.length]);
                    let bytes = transform(&filter, bytes, control)?;
                    for chunk in bytes.chunks(8192) {
                        control.check().map_err(decode_error)?;
                        sink.write_precharged_chunk(chunk)?;
                    }
                    self.used += filter.length;
                    self.emitted += filter.length as u64;
                    self.compact();
                    continue;
                }
                if self.emitted > filter.start {
                    return Err(DecodeError::Integrity);
                }
            }
            let distance = self
                .filters
                .front()
                .map_or(available as u64, |f| f.start - self.emitted);
            let n = available
                .min(8192)
                .min(usize::try_from(distance).unwrap_or(usize::MAX));
            if n == 0 {
                break;
            }
            if !final_output && n < 8192 && self.filters.is_empty() {
                break;
            }
            sink.write_precharged_chunk(&self.pending[self.used..self.used + n])?;
            self.used += n;
            self.emitted += n as u64;
            self.compact();
        }
        if final_output
            && (self.produced != self.expected
                || self.emitted != self.produced
                || !self.filters.is_empty())
        {
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Default)]
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
            self.0.extend(b);
            Ok(())
        }
    }
    #[test]
    fn delta_channels_are_independent_and_unequal() {
        let c = Cancellation::default();
        let f = Filter {
            start: 0,
            length: 7,
            kind: 0,
            channels: 3,
        };
        assert_eq!(
            transform(&f, vec![1, 2, 3, 4, 5, 6, 7], &c).unwrap(),
            [255, 252, 250, 253, 247, 243, 250]
        );
        let f = Filter {
            start: 0,
            length: 3,
            kind: 0,
            channels: 32,
        };
        assert_eq!(transform(&f, vec![1, 2, 3], &c).unwrap(), [255, 254, 253]);
    }
    #[test]
    fn x86_signed_branches_and_period_use_immediate_position() {
        let c = Cancellation::default();
        let f = Filter {
            start: 0xfffffe,
            length: 10,
            kind: 2,
            channels: 0,
        };
        let mut bytes = vec![0xe8, 0, 0, 0, 0, 0xe9];
        bytes.extend((-2i32).to_le_bytes());
        let out = transform(&f, bytes, &c).unwrap();
        assert_eq!(out, [0xe8, 1, 0, 0, 0xff, 0xe9, 0xfe, 0xff, 0xff, 0]);
        let f = Filter {
            start: 12,
            length: 5,
            kind: 1,
            channels: 0,
        };
        assert_eq!(
            transform(&f, vec![0xe9, 1, 2, 3, 4], &c).unwrap(),
            [0xe9, 1, 2, 3, 4]
        );
    }
    #[test]
    fn arm_uses_file_offset_and_keeps_partial_tail() {
        let f = Filter {
            start: 12,
            length: 9,
            kind: 3,
            channels: 0,
        };
        assert_eq!(
            transform(
                &f,
                vec![10, 0, 0, 0xeb, 0, 0, 0, 0xea, 99],
                &Cancellation::default()
            )
            .unwrap(),
            [7, 0, 0, 0xeb, 0, 0, 0, 0xea, 99]
        );
    }
    #[test]
    fn complete_filter_can_cross_flush_boundaries() {
        let mut out = Output::new(9000);
        out.register(Filter {
            start: 0,
            length: 9000,
            kind: 0,
            channels: 2,
        })
        .unwrap();
        let mut sink = Sink::default();
        let c = Cancellation::default();
        for i in 0..9000 {
            out.put(1).unwrap();
            if i % 1024 == 0 {
                out.flush(&mut sink, &c, false).unwrap();
                assert!(sink.0.is_empty());
            }
        }
        out.flush(&mut sink, &c, true).unwrap();
        let expected = (1..=4500)
            .flat_map(|v| [0u8.wrapping_sub(v as u8); 2])
            .collect::<Vec<_>>();
        assert_eq!(sink.0, expected);
    }
    #[test]
    fn pending_overlap_unknown_and_incomplete_fail_closed() {
        let mut out = Output::new(20);
        out.register(Filter {
            start: 4,
            length: 4,
            kind: 1,
            channels: 0,
        })
        .unwrap();
        assert_eq!(
            out.register(Filter {
                start: 6,
                length: 1,
                kind: 3,
                channels: 0
            }),
            Err(DecodeError::Integrity)
        );
        assert_eq!(
            out.register(Filter {
                start: 8,
                length: 1,
                kind: 7,
                channels: 0
            }),
            Err(DecodeError::Unsupported)
        );
        for _ in 0..6 {
            out.put(1).unwrap();
        }
        assert_eq!(
            out.flush(&mut Sink::default(), &Cancellation::default(), true),
            Err(DecodeError::Integrity)
        );
        let mut out = Output::new(8 * 1024 * 1024);
        assert_eq!(
            out.register(Filter {
                start: 0,
                length: BLOCK_LIMIT + 1,
                kind: 1,
                channels: 0
            }),
            Err(DecodeError::Limit)
        );
    }
}
