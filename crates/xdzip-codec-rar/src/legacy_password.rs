//! Legacy native text projection, separate from cipher mathematics.
use crate::{decode_error, index::Descriptor, legacy_crypto::NarrowPassword, source::Reader};
use xdzip_codec_api::payload::DecodeError;
use zeroize::Zeroizing;

pub(crate) const MEMORY_BYTES: u64 = 4096;

pub(crate) fn admit(password: &str) -> Result<(), DecodeError> {
    if password.len() > 4096
        || if cfg!(windows) {
            password.encode_utf16().count()
        } else {
            password.chars().count()
        } > 511
    {
        return Err(DecodeError::Limit);
    }
    Ok(())
}

fn valid_count<T: Default + PartialEq>(output: &[T], count: usize) -> Result<usize, DecodeError> {
    if count >= output.len()
        || output[count] != T::default()
        || output[..count].contains(&T::default())
    {
        return Err(DecodeError::Unsupported);
    }
    Ok(count)
}

fn windows_wide(
    password: &str,
    d: &Descriptor,
    reader: &Reader<'_>,
) -> Result<Zeroizing<[u16; 512]>, DecodeError> {
    let mut wide = Zeroizing::new([0u16; 512]);
    for (at, unit) in password.encode_utf16().enumerate() {
        if at >= 511 {
            return Err(DecodeError::Limit);
        }
        wide[at] = unit;
    }
    let end = wide
        .iter()
        .position(|u| *u == 0)
        .ok_or(DecodeError::Limit)?;
    wide[end..].fill(0);
    if !d.ancient && d.host == 0 {
        let mut projected = Zeroizing::new([0u16; 512]);
        match reader
            .oem_to_ansi_wide(&wide[..], &mut projected[..])
            .map_err(decode_error)?
        {
            Some(count) => {
                valid_count(&projected[..], count)?;
                wide.copy_from_slice(&projected[..]);
            }
            None if wide[..end].iter().all(|u| *u < 128) => (),
            None => return Err(DecodeError::Unsupported),
        }
    }
    wide[127] = 0;
    Ok(wide)
}

pub(crate) struct Wide {
    units: Zeroizing<[u16; 127]>,
    length: usize,
}
impl Wide {
    pub fn as_slice(&self) -> &[u16] {
        &self.units[..self.length]
    }
}
pub(crate) fn wide(
    password: &str,
    d: &Descriptor,
    reader: &Reader<'_>,
) -> Result<Wide, DecodeError> {
    admit(password)?;
    let mut out = Wide {
        units: Zeroizing::new([0; 127]),
        length: 0,
    };
    if cfg!(windows) {
        let native = windows_wide(password, d, reader)?;
        out.length = native[..127].iter().position(|u| *u == 0).unwrap_or(127);
        out.units[..out.length].copy_from_slice(&native[..out.length]);
    } else {
        for scalar in password.chars().take(127).take_while(|c| *c != '\0') {
            out.units[out.length] = scalar as u32 as u16;
            out.length += 1;
        }
    }
    Ok(out)
}

fn ascii_wide(input: &[u32], output: &mut [u8]) -> Option<usize> {
    let end = input.iter().position(|u| *u == 0)?;
    if input[..end].iter().any(|u| *u >= 128) {
        return None;
    }
    for (byte, unit) in output.iter_mut().zip(&input[..end]) {
        *byte = *unit as u8;
    }
    output[end] = 0;
    Some(end)
}

pub(crate) fn narrow(
    password: &str,
    d: &Descriptor,
    reader: &Reader<'_>,
) -> Result<NarrowPassword, DecodeError> {
    admit(password)?;
    let mut bytes = Zeroizing::new([0u8; 512]);
    if cfg!(windows) {
        let input = windows_wide(password, d, reader)?;
        let end = input[..128].iter().position(|u| *u == 0).unwrap_or(127);
        match reader
            .wide_to_ansi(&input[..=end], &mut bytes[..])
            .map_err(decode_error)?
        {
            Some(count) => {
                valid_count(&bytes[..], count)?;
            }
            None if input[..end].iter().all(|u| *u < 128) => {
                for (byte, unit) in bytes.iter_mut().zip(&input[..end]) {
                    *byte = *unit as u8;
                }
            }
            None => return Err(DecodeError::Unsupported),
        }
    } else {
        let mut input = Zeroizing::new([0u32; 128]);
        let mut length = 0;
        for scalar in password.chars().take(127).take_while(|c| *c != '\0') {
            input[length] = u32::from(scalar);
            length += 1;
        }
        if cfg!(target_os = "macos") {
            let mut at = 0;
            for scalar in &input[..length] {
                let scalar = char::from_u32(*scalar).ok_or(DecodeError::Unsupported)?;
                let mut encoded = Zeroizing::new([0u8; 4]);
                let part = scalar.encode_utf8(&mut encoded[..]);
                bytes[at..at + part.len()].copy_from_slice(part.as_bytes());
                at += part.len();
            }
        } else if input[..length].contains(&0xfffe) {
            // Marker recovery belongs to this native compatibility profile;
            // ordinary locale conversion never invents raw private-use bytes.
            let mut at = 0usize;
            for &unit in &input[..length] {
                if unit == 0xfffe {
                    continue;
                }
                if (0xe080..=0xe0ff).contains(&unit) {
                    if at >= 511 {
                        break;
                    }
                    bytes[at] = unit as u8;
                    at += 1;
                    continue;
                }
                let mut encoded = Zeroizing::new([0u8; 512]);
                let one = Zeroizing::new([unit, 0]);
                let count = reader
                    .wide_to_local_prefix(&one[..], &mut encoded[..])
                    .map_err(decode_error)?
                    .or_else(|| ascii_wide(&one[..], &mut encoded[..]))
                    .ok_or(DecodeError::Unsupported)?;
                valid_count(&encoded[..], count)?;
                let count = if count == 0 {
                    encoded[0] = b'_';
                    1
                } else {
                    count
                };
                if at.checked_add(count).is_none_or(|n| n >= 512) {
                    break;
                }
                bytes[at..at + count].copy_from_slice(&encoded[..count]);
                at += count;
            }
        } else {
            let count = reader
                .wide_to_local_prefix(&input[..=length], &mut bytes[..])
                .map_err(decode_error)?
                .or_else(|| ascii_wide(&input[..=length], &mut bytes[..]))
                .ok_or(DecodeError::Unsupported)?;
            valid_count(&bytes[..], count)?;
        }
    }
    bytes[127] = 0;
    NarrowPassword::from_projected(&bytes[..])
}

#[cfg(test)]
#[path = "legacy_password_tests.rs"]
mod tests;
