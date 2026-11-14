use crate::work::{self, Work};
use std::io::SeekFrom;
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};
pub(crate) const ITERATION_LIMIT: u32 = 2_000_000;
const HEADER_LIMIT: u64 = 1024 * 1024;
#[derive(Clone, Eq, PartialEq)]
pub(crate) struct Header {
    pub version: u8,
    pub length: u64,
    pub iterations: u32,
    pub iv: [u8; 16],
    pub wrapped: Option<[u8; 48]>,
    pub session_tag: Option<[u8; 32]>,
    pub data_tag: [u8; 32],
    pub start: u64,
    pub cipher: u64,
    pub legacy_size: Option<u64>,
}
pub(crate) fn parse(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    work: &mut Work,
) -> Result<Header, ScanError> {
    cancel.check()?;
    let length = source
        .seek(SeekFrom::End(0))
        .map_err(|e| ScanError::from_source_error(&e))?;
    if length < 5 {
        return Err(ScanError::Invalid);
    }
    if length > work::MAXIMUM {
        return Err(ScanError::Limit);
    }
    work::seek(source, 0)?;
    let mut fixed = [0; 5];
    work::read(source, &mut fixed, cancel, work)?;
    if fixed[..3] != *b"AES" {
        return Err(ScanError::Invalid);
    }
    let version = fixed[3];
    let minimum = match version {
        0 => 53,
        1 => 134,
        2 => 136,
        3 => 155,
        _ => return Err(ScanError::Unsupported),
    };
    if length < minimum || (version == 0 && fixed[4] > 15) || (version != 0 && fixed[4] != 0) {
        return Err(ScanError::Invalid);
    }
    let mut position = 5u64;
    if version >= 2 {
        let mut count = 0usize;
        let mut scratch = [0; 65535];
        loop {
            if position + 2 > length {
                return Err(ScanError::Invalid);
            }
            let mut size = [0; 2];
            work::read(source, &mut size, cancel, work)?;
            position += 2;
            let size = usize::from(u16::from_be_bytes(size));
            if size == 0 {
                break;
            }
            count += 1;
            if count > 256
                || position
                    .checked_add(size as u64)
                    .is_none_or(|end| end > HEADER_LIMIT)
            {
                return Err(ScanError::Limit);
            }
            if position + size as u64 > length {
                return Err(ScanError::Invalid);
            }
            work::read(source, &mut scratch[..size], cancel, work)?;
            if !scratch[..size].contains(&0) {
                return Err(ScanError::Invalid);
            }
            position += size as u64;
        }
    }
    let required = if version == 3 {
        148
    } else if version == 1 || version == 2 {
        129
    } else {
        48
    };
    if position
        .checked_add(required)
        .is_none_or(|end| end > length)
    {
        return Err(ScanError::Invalid);
    }
    let iterations = if version == 3 {
        let mut bytes = [0; 4];
        work::read(source, &mut bytes, cancel, work)?;
        position += 4;
        let value = u32::from_be_bytes(bytes);
        if value == 0 {
            return Err(ScanError::Invalid);
        }
        if value > ITERATION_LIMIT {
            return Err(ScanError::Limit);
        }
        value
    } else {
        8192
    };
    let mut iv = [0; 16];
    work::read(source, &mut iv, cancel, work)?;
    position += 16;
    let (wrapped, session_tag) = if version != 0 {
        let mut bytes = [0; 48];
        let mut tag = [0; 32];
        work::read(source, &mut bytes, cancel, work)?;
        work::read(source, &mut tag, cancel, work)?;
        position += 80;
        (Some(bytes), Some(tag))
    } else {
        (None, None)
    };
    let footer = if version == 1 || version == 2 { 33 } else { 32 };
    if position > HEADER_LIMIT {
        return Err(ScanError::Limit);
    }
    let cipher = length
        .checked_sub(position + footer)
        .ok_or(ScanError::Invalid)?;
    if !cipher.is_multiple_of(16) || (version == 3 && cipher == 0) {
        return Err(ScanError::Invalid);
    }
    let mut modulo = fixed[4];
    work::seek(source, length - footer)?;
    if version == 1 || version == 2 {
        let mut byte = [0];
        work::read(source, &mut byte, cancel, work)?;
        modulo = byte[0];
    }
    let legacy_size = if version < 3 {
        if modulo > 15 || (cipher == 0 && modulo != 0) {
            return Err(ScanError::Invalid);
        }
        Some(
            cipher
                .checked_sub(if modulo == 0 {
                    0
                } else {
                    16 - u64::from(modulo)
                })
                .ok_or(ScanError::Invalid)?,
        )
    } else {
        None
    };
    let mut data_tag = [0; 32];
    work::read(source, &mut data_tag, cancel, work)?;
    if cipher > work.left.input {
        return Err(ScanError::Limit);
    }
    Ok(Header {
        version,
        length,
        iterations,
        iv,
        wrapped,
        session_tag,
        data_tag,
        start: position,
        cipher,
        legacy_size,
    })
}
