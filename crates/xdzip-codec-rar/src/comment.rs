//! Bounded, display-only decoding of historical per-file comments.
use crate::algorithm::{Algorithm, Engine, WorkOutput};
use crate::index::{Descriptor, Part};
use crate::legacy::InlineComment;
use crate::source::Reader;
use std::io::{Cursor, Read};
use xdzip_codec_api::payload::{DecodeError, DecodedSink};
use xdzip_codec_api::{Cancellation, ScanError};

const WORK_LIMIT: u64 = 32 * 1024 * 1024;

struct Output {
    bytes: Vec<u8>,
    limit: usize,
    work: u64,
}
impl DecodedSink for Output {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self.bytes.len().saturating_add(bytes.len()) > self.limit {
            return Err(DecodeError::Limit);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}
impl WorkOutput for Output {
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
        WORK_LIMIT.saturating_sub(self.work)
    }
}
impl Output {
    fn charge(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.work = self
            .work
            .checked_add(bytes)
            .filter(|n| *n <= WORK_LIMIT)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
}

fn decode(
    description: &Descriptor,
    packed: &[u8],
    padding: u8,
    control: &Cancellation,
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
        work: 0,
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
    Ok(
        decode(&description, &raw.packed, 0, control)?.filter(|bytes| {
            raw.crc
                .is_none_or(|crc| xdzip_compression::crc32(bytes) as u16 == crc)
        }),
    )
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
    let mut coding = first.description.clone();
    let decoded = if coding.encrypted {
        let cipher = match protection.legacy_content(&coding, password, control, reader) {
            Ok(cipher) => cipher,
            Err(DecodeError::Cancelled) => return Err(ScanError::Cancelled),
            Err(_) => return Ok(None),
        };
        let mut input = match crate::legacy_crypto::LegacyReader::new(
            Cursor::new(&packed),
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
        decode(&coding, content, padding, control)?
    } else {
        decode(&coding, &packed, 0, control)?
    };
    Ok(decoded.filter(|bytes| {
        last.description
            .crc
            .is_some_and(|crc| xdzip_compression::crc32(bytes) == crc)
    }))
}
