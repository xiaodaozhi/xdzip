use crate::algorithm::WorkOutput;
use crate::decode_error;
use xdzip_codec_api::{Cancellation, payload::DecodeError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Kind {
    E8,
    E8E9,
    Itanium,
    Delta,
    Rgb,
    Audio,
}
impl Kind {
    /// Fixed functional signatures from RAR29-FILTERS-2026-10-26-v1, not an
    /// arbitrary VM interpreter or an authentication promise for program CRC.
    pub fn identify(length: usize, crc: u32) -> Result<Self, DecodeError> {
        match (length, crc) {
            (53, 0xad576887) => Ok(Self::E8),
            (57, 0x3cd7e57e) => Ok(Self::E8E9),
            (120, 0x3769893f) => Ok(Self::Itanium),
            (29, 0x0e06077d) => Ok(Self::Delta),
            (149, 0x1c2c5dc8) => Ok(Self::Rgb),
            (216, 0xbc85e701) => Ok(Self::Audio),
            _ => Err(DecodeError::Unsupported),
        }
    }
}
pub(crate) struct Filter {
    pub start: u64,
    pub length: usize,
    pub kind: Kind,
    pub parameters: [u32; 7],
}
fn copy(bytes: &[u8]) -> Result<Vec<u8>, DecodeError> {
    let mut out = Vec::new();
    out.try_reserve_exact(bytes.len())
        .map_err(|_| DecodeError::Limit)?;
    out.extend_from_slice(bytes);
    Ok(out)
}
fn field(bytes: &[u8], bit: usize, width: usize) -> u32 {
    (0..width).fold(0, |value, i| {
        value | (u32::from((bytes[(bit + i) / 8] >> ((bit + i) % 8)) & 1) << i)
    })
}
fn set_field(bytes: &mut [u8], bit: usize, width: usize, value: u32) {
    for i in 0..width {
        let at = (bit + i) / 8;
        let shift = (bit + i) % 8;
        bytes[at] = (bytes[at] & !(1 << shift)) | ((((value >> i) & 1) as u8) << shift);
    }
}
fn adapt_audio(
    coefficients: &mut [i32; 3],
    errors: &mut [i32; 7],
    sample: usize,
) -> Result<(), DecodeError> {
    if sample.is_multiple_of(32) {
        let index = errors
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| *e)
            .map(|(i, _)| i)
            .ok_or(DecodeError::Integrity)?;
        if index != 0 {
            let coefficient = &mut coefficients[(index - 1) / 2];
            if index % 2 == 1 {
                if *coefficient >= -16 {
                    *coefficient -= 1;
                }
            } else if *coefficient < 16 {
                *coefficient += 1;
            }
        }
        errors.fill(0);
    }
    Ok(())
}
fn transform(
    filter: &Filter,
    bytes: &[u8],
    mut progress: impl FnMut(u64) -> Result<(), DecodeError>,
) -> Result<Vec<u8>, DecodeError> {
    let n = bytes.len();
    if n != filter.length || n == 0 || n >= 262144 || filter.parameters[4] != n as u32 {
        return Err(DecodeError::Integrity);
    }
    let mut out = copy(bytes)?;
    match filter.kind {
        Kind::E8 | Kind::E8E9 => {
            if n < 4 {
                return Err(DecodeError::Integrity);
            }
            let mut i = 0;
            while i + 4 < n {
                if bytes[i] == 0xe8 || (filter.kind == Kind::E8E9 && bytes[i] == 0xe9) {
                    let a = u32::from_le_bytes(
                        bytes[i + 1..i + 5]
                            .try_into()
                            .map_err(|_| DecodeError::Integrity)?,
                    );
                    let q = (filter.start as u32).wrapping_add(i as u32).wrapping_add(1);
                    let value = if a & 0x80000000 != 0 && a.wrapping_add(q) & 0x80000000 == 0 {
                        a.wrapping_add(1 << 24)
                    } else if a & 0x80000000 == 0 && a.wrapping_sub(1 << 24) & 0x80000000 != 0 {
                        a.wrapping_sub(q)
                    } else {
                        a
                    };
                    out[i + 1..i + 5].copy_from_slice(&value.to_le_bytes());
                    i += 5;
                    progress(5)?;
                } else {
                    i += 1;
                    progress(1)?;
                }
            }
            progress((n - i) as u64)?;
        }
        Kind::Itanium => {
            if n < 21 {
                return Err(DecodeError::Integrity);
            }
            let mut at = 0;
            while at < n - 21 {
                let template = bytes[at] & 31;
                for slot in 0..3 {
                    let selected = match template {
                        16 | 17 | 24 | 25 | 28 | 29 => slot == 2,
                        18 | 19 => slot >= 1,
                        22 | 23 => true,
                        _ => false,
                    };
                    let start = 5 + 41 * slot;
                    if selected && field(&out[at..at + 16], start + 37, 4) == 5 {
                        let a = field(&out[at..at + 16], start + 13, 20);
                        let pc = (filter.start as u32 / 16).wrapping_add((at / 16) as u32);
                        set_field(
                            &mut out[at..at + 16],
                            start + 13,
                            20,
                            a.wrapping_sub(pc) & 0xfffff,
                        );
                    }
                }
                at += 16;
                progress(16)?;
            }
            progress((n - at) as u64)?;
        }
        Kind::Delta | Kind::Audio => {
            let maximum = if filter.kind == Kind::Delta {
                1024
            } else {
                128
            };
            let channels =
                usize::try_from(filter.parameters[0]).map_err(|_| DecodeError::Integrity)?;
            if n > 131072 || channels == 0 || channels > maximum {
                return Err(DecodeError::Integrity);
            }
            let mut source = 0;
            // Empty channels cannot alter state or output; skipping them keeps
            // work proportional to the actual input even for malicious C>N.
            for channel in 0..channels.min(n) {
                let mut previous = 0u8;
                let mut previous_delta = 0i32;
                let (mut old_d1, mut old_d2) = (0i32, 0i32);
                let mut coefficients = [0i32; 3];
                let mut errors = [0i32; 7];
                for (sample, at) in (channel..n).step_by(channels).enumerate() {
                    let residual = bytes[source];
                    source += 1;
                    let value = if filter.kind == Kind::Delta {
                        previous.wrapping_sub(residual)
                    } else {
                        let differences = [previous_delta, previous_delta - old_d1, old_d2];
                        old_d1 = differences[0];
                        old_d2 = differences[1];
                        let prediction = (8 * i32::from(previous)
                            + coefficients
                                .iter()
                                .zip(differences)
                                .map(|(k, d)| k * d)
                                .sum::<i32>())
                        .div_euclid(8) as u8;
                        let value = prediction.wrapping_sub(residual);
                        previous_delta = i32::from(value.wrapping_sub(previous) as i8);
                        let error = 8 * i32::from(residual as i8);
                        errors[0] += error.abs();
                        for (i, d) in differences.into_iter().enumerate() {
                            errors[1 + 2 * i] += (error - d).abs();
                            errors[2 + 2 * i] += (error + d).abs();
                        }
                        adapt_audio(&mut coefficients, &mut errors, sample)?;
                        value
                    };
                    out[at] = value;
                    previous = value;
                    progress(1)?;
                }
            }
        }
        Kind::Rgb => {
            let width = filter.parameters[0]
                .checked_sub(3)
                .ok_or(DecodeError::Integrity)? as usize;
            let red = filter.parameters[1] as usize;
            if !(3..=131072).contains(&n) || width > n || red > 2 {
                return Err(DecodeError::Integrity);
            }
            let mut initialized = vec![false; n];
            let mut source = 0;
            for channel in 0..3 {
                let mut previous = 0u8;
                for at in (channel..n).step_by(3) {
                    let prediction = if at < width + 3 {
                        previous
                    } else {
                        let up = at - width;
                        let diagonal = up - 3;
                        if !initialized[up] || !initialized[diagonal] {
                            return Err(DecodeError::Integrity);
                        }
                        let choices = [
                            i32::from(previous),
                            i32::from(out[up]),
                            i32::from(out[diagonal]),
                        ];
                        let p = choices[0] + choices[1] - choices[2];
                        let index = (0..3)
                            .min_by_key(|&i| (p - choices[i]).abs())
                            .ok_or(DecodeError::Integrity)?;
                        choices[index] as u8
                    };
                    out[at] = prediction.wrapping_sub(bytes[source]);
                    source += 1;
                    previous = out[at];
                    initialized[at] = true;
                    progress(1)?;
                }
            }
            for (step, at) in (red..n - 2).step_by(3).enumerate() {
                out[at] = out[at].wrapping_add(out[at + 1]);
                out[at + 2] = out[at + 2].wrapping_add(out[at + 1]);
                if step & 2047 == 0 {
                    progress(0)?;
                }
            }
        }
    }
    Ok(out)
}
fn run_observed(
    filter: &Filter,
    bytes: &[u8],
    sink: &mut dyn WorkOutput,
    control: &Cancellation,
    mut observe: impl FnMut(u64),
) -> Result<Vec<u8>, DecodeError> {
    control.check().map_err(decode_error)?;
    sink.reserve_aux(bytes.len() as u64)?;
    let mut processed = 0u64;
    let mut checked = 0;
    let result = transform(filter, bytes, |n| {
        processed += n;
        observe(processed);
        if n == 0 || processed - checked >= 8192 {
            checked = processed;
            control.check().map_err(decode_error)?;
        }
        Ok(())
    });
    sink.produced_aux(processed)?;
    let result = result?;
    control.check().map_err(decode_error)?;
    Ok(result)
}
pub(crate) fn run(
    filter: &Filter,
    bytes: &[u8],
    sink: &mut dyn WorkOutput,
    control: &Cancellation,
) -> Result<Vec<u8>, DecodeError> {
    run_observed(filter, bytes, sink, control, |_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use xdzip_codec_api::payload::DecodedSink;
    fn filter(kind: Kind, length: usize, first: u32, second: u32, start: u64) -> Filter {
        let mut parameters = [0; 7];
        parameters[0] = first;
        parameters[1] = second;
        parameters[4] = length as u32;
        Filter {
            start,
            length,
            kind,
            parameters,
        }
    }
    fn math(filter: Filter, input: &[u8]) -> Vec<u8> {
        transform(&filter, input, |_| Ok(())).unwrap()
    }
    #[test]
    fn old_x86_pc_wrap_and_tail_are_not_the_modern_16mib_profile() {
        let input = [0xe8, 16, 0, 0, 0];
        assert_eq!(
            math(filter(Kind::E8, 5, 0, 0, 0x1000007), &input),
            [0xe8, 8, 0, 0, 0xff]
        );
        assert_eq!(
            math(
                filter(Kind::E8, 5, 0, 0, 1),
                &[0xe8, 0xfe, 0xff, 0xff, 0xff]
            ),
            [0xe8, 0xfe, 0xff, 0xff, 0]
        );
        assert_eq!(
            math(filter(Kind::E8, 5, 0, 0, 100), &[0xe9, 16, 0, 0, 0]),
            [0xe9, 16, 0, 0, 0]
        );
        assert_eq!(
            math(filter(Kind::E8E9, 5, 0, 0, 100), &[0xe9, 16, 0, 0, 0]),
            [0xe9, 0xab, 0xff, 0xff, 0xff]
        );
        assert_eq!(
            math(filter(Kind::E8, 4, 0, 0, 100), &[0xe8, 16, 0, 0]),
            [0xe8, 16, 0, 0]
        );
    }
    #[test]
    fn channel_reconstruction_and_rgb_predictor_use_only_initialized_history() {
        assert_eq!(
            math(filter(Kind::Delta, 6, 2, 0, 0), &[1, 2, 3, 4, 5, 6]),
            [255, 252, 253, 247, 250, 241]
        );
        assert_eq!(
            math(filter(Kind::Audio, 4, 2, 0, 0), &[1, 2, 3, 4]),
            [255, 253, 253, 249]
        );
        assert_eq!(
            math(filter(Kind::Rgb, 9, 6, 0, 0), &[1, 2, 3, 4, 5, 6, 7, 8, 9]),
            [251, 252, 245, 244, 247, 232, 235, 241, 217]
        );
        assert!(transform(&filter(Kind::Rgb, 9, 3, 0, 0), &[0; 9], |_| Ok(())).is_err());
        for kind in [Kind::Delta, Kind::Audio] {
            assert!(transform(&filter(kind, 4, 0, 0, 0), &[0; 4], |_| Ok(())).is_err());
        }
    }
    #[test]
    fn itanium_protected_tail_and_slot_bits_are_exact() {
        let mut input = vec![0; 22];
        input[0] = 22;
        set_field(&mut input, 5 + 41 + 37, 4, 5);
        set_field(&mut input, 5 + 41 + 13, 20, 100);
        let output = math(filter(Kind::Itanium, 22, 0, 0, 16), &input);
        assert_eq!(field(&output, 5 + 41 + 13, 20), 99);
        assert_eq!(field(&output, 5 + 41 + 37, 4), 5);
        assert_eq!(&output[16..], &input[16..]);
        assert_eq!(
            math(filter(Kind::Itanium, 21, 0, 0, 16), &input[..21]),
            input[..21]
        );
    }
    #[derive(Default)]
    struct Work {
        reserved: u64,
        actual: u64,
    }
    impl DecodedSink for Work {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            panic!("transform does not deliver output")
        }
    }
    impl WorkOutput for Work {
        fn reserve_aux(&mut self, n: u64) -> Result<(), DecodeError> {
            self.reserved += n;
            Ok(())
        }
        fn produced_aux(&mut self, n: u64) -> Result<(), DecodeError> {
            self.reserved -= n;
            self.actual += n;
            Ok(())
        }
    }
    #[test]
    fn cancelled_filter_settles_only_the_executed_prefix_without_delivery() {
        let cancel = Cancellation::default();
        let mut sink = Work::default();
        let result = run_observed(
            &filter(Kind::Delta, 20000, 2, 0, 0),
            &vec![1; 20000],
            &mut sink,
            &cancel,
            |n| {
                if n >= 8192 {
                    cancel.cancel();
                }
            },
        );
        assert!(matches!(result, Err(DecodeError::Cancelled)));
        assert_eq!(sink.actual, 8192);
        assert_eq!(sink.reserved, 20000 - 8192);
    }
    #[test]
    fn audio_adapts_at_sample_zero_and_the_negative_limit_is_minus_seventeen() {
        let mut coefficients = [-16, 16, 0];
        let mut errors = [9, 1, 2, 3, 4, 5, 6];
        adapt_audio(&mut coefficients, &mut errors, 0).unwrap();
        assert_eq!(coefficients, [-17, 16, 0]);
        assert_eq!(errors, [0; 7]);
        errors = [9, 1, 2, 3, 4, 5, 6];
        adapt_audio(&mut coefficients, &mut errors, 31).unwrap();
        assert_eq!(coefficients, [-17, 16, 0]);
        assert_ne!(errors, [0; 7]);
        adapt_audio(&mut coefficients, &mut errors, 32).unwrap();
        assert_eq!(coefficients, [-17, 16, 0]);
        errors = [9, 9, 9, 9, 1, 9, 9];
        adapt_audio(&mut coefficients, &mut errors, 64).unwrap();
        assert_eq!(coefficients, [-17, 16, 0]);
    }
}
