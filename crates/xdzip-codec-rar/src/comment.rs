//! Bounded, display-only decoding of historical per-file comments.
use crate::algorithm::{Algorithm, Engine, WorkOutput};
use crate::index::{Descriptor, Part};
use crate::legacy::InlineComment;
use crate::source::Reader;
use std::io::{Cursor, Read};
use xdzip_codec_api::payload::{DecodeError, DecodedSink};
use xdzip_codec_api::{Cancellation, ScanError};

const WORK_LIMIT: u64 = 32 * 1024 * 1024;

struct Output<'a> {
    bytes: Vec<u8>,
    limit: usize,
    work: &'a mut u64,
    exhausted: bool,
}
impl DecodedSink for Output<'_> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self.bytes.len().saturating_add(bytes.len()) > self.limit {
            return Err(DecodeError::Limit);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}
impl WorkOutput for Output<'_> {
    fn reserve_work(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.charge(bytes)
    }
    fn produced_work(&mut self, _bytes: u64) -> Result<(), DecodeError> {
        Ok(())
    }
    fn reserve_aux(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.charge(bytes)
    }
    fn produced_aux(&mut self, _bytes: u64) -> Result<(), DecodeError> {
        Ok(())
    }
    fn remaining_aux_work(&self) -> u64 {
        WORK_LIMIT.saturating_sub(*self.work)
    }
}
impl Output<'_> {
    fn charge(&mut self, bytes: u64) -> Result<(), DecodeError> {
        match self.work.checked_add(bytes).filter(|n| *n <= WORK_LIMIT) {
            Some(work) => {
                *self.work = work;
                Ok(())
            }
            None => {
                self.exhausted = true;
                Err(DecodeError::Limit)
            }
        }
    }
}

fn decode(
    description: &Descriptor,
    packed: &[u8],
    padding: u8,
    control: &Cancellation,
    work: &mut u64,
) -> Result<Option<Vec<u8>>, ScanError> {
    control.check()?;
    if description.encrypted
        || description.solid
        || description.special
        || description.redirection
        || description.compression > 5
        || (description.compression != 0 && !matches!(description.method, 13 | 15 | 20 | 26 | 29))
    {
        return Ok(None);
    }
    let Some(size) = description.size.filter(|size| *size <= u64::from(u16::MAX)) else {
        return Ok(None);
    };
    let mut output = Output {
        bytes: Vec::new(),
        limit: size as usize,
        work,
        exhausted: false,
    };
    let mut engine = Engine::default();
    if engine
        .memory_required(description)
        .ok()
        .and_then(|n| n.checked_add(crate::protection::Protection::MEMORY_BYTES))
        .is_none_or(|n| n > crate::MEMORY_LIMIT)
    {
        return Ok(None);
    }
    let result = engine.decode_padded(
        description,
        &mut Cursor::new(packed),
        &mut output,
        control,
        padding,
    );
    if matches!(result, Err(DecodeError::Cancelled)) {
        return Err(ScanError::Cancelled);
    }
    if matches!(result, Err(DecodeError::Limit)) && output.exhausted {
        return Err(ScanError::Limit);
    }
    if result.is_err() || output.bytes.len() != size as usize {
        return Ok(None);
    }
    Ok(Some(output.bytes))
}

pub(crate) fn inline(
    raw: &InlineComment,
    control: &Cancellation,
) -> Result<Option<Vec<u8>>, ScanError> {
    let Some(compression) = raw.method.checked_sub(0x30).filter(|value| *value <= 5) else {
        return Ok(None);
    };
    let description = Descriptor {
        size: Some(u64::from(raw.size)),
        legacy: true,
        method: u16::from(raw.version),
        compression,
        dictionary: 65536,
        ..Descriptor::default()
    };
    Ok(decode(&description, &raw.packed, 0, control, &mut 0)
        .or_else(|error| match error {
            ScanError::Limit => Ok(None),
            other => Err(other),
        })?
        .filter(|bytes| {
            raw.crc
                .is_none_or(|crc| xdzip_compression::crc32(bytes) as u16 == crc)
        }))
}

// CMT subdata is opaque and has no length field. Enumerate only suffixes
// whose time flags consume the entire remaining header, without retaining or
// interpreting timestamps as file metadata. The fourth (archive time) slot
// may be absent in older readers or physically present in the wire format.
fn time_salts(header: &[u8], name_len: usize) -> Vec<[u8; 8]> {
    let Some(flags) = header.get(3..5).map(|s| u16::from_le_bytes([s[0], s[1]])) else {
        return Vec::new();
    };
    let Some(start) = 32usize
        .checked_add(if flags & 0x100 != 0 { 8 } else { 0 })
        .and_then(|v| v.checked_add(name_len))
    else {
        return Vec::new();
    };
    let Some(tail) = header.get(start..) else {
        return Vec::new();
    };
    let mut salts = Vec::new();
    for time_at in 8.max(tail.len().saturating_sub(26))..tail.len().saturating_sub(1) {
        let time = &tail[time_at..];
        if time.len() > 26 {
            continue;
        }
        let flags = u16::from_le_bytes([time[0], time[1]]);
        let mut length = 2usize;
        for i in 0..3 {
            let nibble = (flags >> (12 - i * 4)) & 15;
            if nibble & 8 != 0 {
                length += usize::from(nibble & 3) + if i == 0 { 0 } else { 4 };
            }
        }
        let archive = flags & 15;
        let archive_bytes = if archive & 8 != 0 {
            4 + usize::from(archive & 3)
        } else {
            0
        };
        if time.len() == length || time.len() == length + archive_bytes {
            let salt: [u8; 8] = tail[time_at - 8..time_at].try_into().expect("eight bytes");
            if !salts.contains(&salt) {
                salts.push(salt);
            }
        }
    }
    salts
}

pub(crate) fn service(
    reader: &mut Reader<'_>,
    parts: &[Part],
    indices: &[usize],
    control: &Cancellation,
    password: Option<&str>,
    protection: &mut crate::protection::Protection,
) -> Result<Option<Vec<u8>>, ScanError> {
    let Some(first) = indices.first().and_then(|at| parts.get(*at)) else {
        return Ok(None);
    };
    let Some(last) = indices.last().and_then(|at| parts.get(*at)) else {
        return Ok(None);
    };
    if first.description.service_comment_utf16.is_none() {
        return Ok(None);
    }
    let total = indices
        .iter()
        .try_fold(0u64, |sum, at| sum.checked_add(parts.get(*at)?.packed));
    let Some(total) = total.filter(|size| *size <= 1024 * 1024) else {
        return Ok(None);
    };
    let mut packed = Vec::with_capacity(total as usize);
    for at in indices {
        let part = parts.get(*at).ok_or(ScanError::Invalid)?;
        control.check()?;
        if reader.bytes(part.volume, part.header_at, part.header.len())? != part.header {
            return Err(ScanError::SourceChanged);
        }
        packed.extend(reader.bytes(part.volume, part.data_at, part.packed as usize)?);
    }
    let coding = &first.description;
    let header_flags = u16::from_le_bytes([first.header[3], first.header[4]]);
    let candidates: Vec<Option<[u8; 8]>> = if coding.encrypted
        && !matches!(coding.method, 13 | 15 | 20 | 26)
        && header_flags & 0x1400 == 0x1400
    {
        time_salts(&first.header, coding.name.len())
            .into_iter()
            .map(Some)
            .collect()
    } else {
        vec![None]
    };
    let mut work = 0;
    let mut verified = None;
    for salt in candidates {
        let mut coding = coding.clone();
        if let Some(salt) = salt {
            coding.crypt = Some(crate::index::Crypt {
                version: u64::from(coding.method),
                flags: 0,
                exponent: 0,
                salt: salt.to_vec(),
                iv: Vec::new(),
                check: None,
            });
        }
        let decoded = match attempt(
            &mut coding,
            &packed,
            total,
            control,
            password,
            protection,
            reader,
            &mut work,
        ) {
            Err(ScanError::Limit) => return Ok(None),
            other => other?,
        };
        if let Some(bytes) = decoded.filter(|bytes| {
            last.description
                .crc
                .is_some_and(|crc| xdzip_compression::crc32(bytes) == crc)
        }) {
            if verified.is_some() {
                return Ok(None);
            }
            verified = Some(bytes);
        }
    }
    Ok(verified)
}

#[allow(clippy::too_many_arguments)]
fn attempt(
    coding: &mut Descriptor,
    packed: &[u8],
    total: u64,
    control: &Cancellation,
    password: Option<&str>,
    protection: &mut crate::protection::Protection,
    reader: &Reader<'_>,
    work: &mut u64,
) -> Result<Option<Vec<u8>>, ScanError> {
    if coding.encrypted {
        let cipher = match protection.legacy_content(coding, password, control, reader) {
            Ok(cipher) => cipher,
            Err(DecodeError::Cancelled) => return Err(ScanError::Cancelled),
            Err(DecodeError::Limit) => return Err(ScanError::Limit),
            Err(_) => return Ok(None),
        };
        let mut input = match crate::legacy_crypto::LegacyReader::new(
            Cursor::new(packed),
            total,
            cipher,
            control,
        ) {
            Ok(input) => input,
            Err(_) => return Ok(None),
        };
        let mut plain = zeroize::Zeroizing::new(Vec::with_capacity(total as usize));
        if input.read_to_end(&mut plain).is_err() || input.finish().is_err() {
            control.check()?;
            return Ok(None);
        }
        coding.encrypted = false;
        coding.crypt = None;
        let (content, padding) = if coding.compression == 0 {
            let Some(size) = coding.size.and_then(|size| usize::try_from(size).ok()) else {
                return Ok(None);
            };
            if size > plain.len() || plain.len() - size > 15 {
                return Ok(None);
            }
            (&plain[..size], 0)
        } else {
            (&plain[..], 15)
        };
        decode(coding, content, padding, control, work)
    } else {
        decode(coding, packed, 0, control, work)
    }
}
