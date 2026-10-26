use crate::crypto::CbcReader;
use crate::index::{Block, Crypt, Descriptor, Parsed, Part, RawTime};
use crate::protection::{Protection, scan_error};
use crate::{source::Reader, wire::Wire};
use std::io::{Cursor, Read};
use xdzip_codec_api::{ScanError, ScanLimits};
use xdzip_compression::crc32;
use zeroize::Zeroizing;

const HEAD_LIMIT: usize = 2 * 1024 * 1024;
fn count(value: u64) -> Result<usize, ScanError> {
    usize::try_from(value).map_err(|_| ScanError::Limit)
}
fn crypt(w: &mut Wire<'_>, iv: bool) -> Result<Crypt, ScanError> {
    let version = w.vint()?;
    let flags = w.vint()?;
    let exponent = w.byte()?;
    if version != 0 || flags & !(if iv { 3 } else { 1 }) != 0 {
        return Err(ScanError::Unsupported);
    }
    if exponent > 24 {
        return Err(ScanError::Limit);
    }
    let salt = w.take(16)?.to_vec();
    let iv = if iv { w.take(16)?.to_vec() } else { Vec::new() };
    let check = if flags & 1 != 0 {
        Some(w.take(12)?.try_into().map_err(|_| ScanError::Invalid)?)
    } else {
        None
    };
    Ok(Crypt {
        version,
        flags,
        exponent,
        salt,
        iv,
        check,
    })
}
fn records(bytes: &[u8]) -> Result<Vec<(u64, &[u8])>, ScanError> {
    let mut w = Wire::new(bytes);
    let mut result = Vec::new();
    while w.remaining() > 0 {
        let n = count(w.vint()?)?;
        if n == 0 {
            return Err(ScanError::Invalid);
        }
        let mut record = Wire::new(w.take(n)?);
        let kind = record.vint()?;
        result.push((kind, record.rest()));
    }
    Ok(result)
}
fn main_extra(bytes: &[u8], limits: ScanLimits) -> Result<(), ScanError> {
    let mut seen = 0u8;
    for (kind, bytes) in records(bytes)? {
        let mut w = Wire::new(bytes);
        if (1..=2).contains(&kind) {
            let bit = 1u8 << kind;
            if seen & bit != 0 {
                return Err(ScanError::Invalid);
            }
            seen |= bit;
        }
        match kind {
            1 => {
                let flags = w.vint()?;
                if flags & !3 != 0 {
                    return Err(ScanError::Unsupported);
                }
                for bit in [1, 2] {
                    if flags & bit != 0 {
                        w.vint()?;
                    }
                }
                w.end()?;
            }
            2 => {
                let flags = w.vint()?;
                if flags & !15 != 0 {
                    return Err(ScanError::Unsupported);
                }
                if flags & 1 != 0 {
                    let n = count(w.vint()?)?;
                    if n > limits.name_bytes {
                        return Err(ScanError::Limit);
                    }
                    w.take(n)?;
                }
                if flags & 2 != 0 {
                    if flags & 4 != 0 && flags & 8 == 0 {
                        w.u32()?;
                    } else {
                        w.u64()?;
                    }
                }
                w.end()?;
            }
            _ => (),
        }
    }
    Ok(())
}
fn extras(bytes: &[u8], d: &mut Descriptor, limits: ScanLimits) -> Result<(), ScanError> {
    let mut seen = 0u8;
    for (kind, bytes) in records(bytes)? {
        let mut w = Wire::new(bytes);
        if (1..=6).contains(&kind) {
            let bit = 1u8 << kind;
            if seen & bit != 0 {
                return Err(ScanError::Invalid);
            }
            seen |= bit;
        }
        match kind {
            1 => {
                d.crypt = Some(crypt(&mut w, true)?);
                d.encrypted = true;
                w.end()?;
            }
            2 => {
                if w.vint()? != 0 {
                    return Err(ScanError::Unsupported);
                }
                d.hash = Some(w.take(32)?.try_into().map_err(|_| ScanError::Invalid)?);
                w.end()?;
            }
            3 => {
                let flags = w.vint()?;
                if flags & !31 != 0 {
                    return Err(ScanError::Unsupported);
                }
                let unix = flags & 1 != 0;
                for (i, bit) in [2, 4, 8].into_iter().enumerate() {
                    if flags & bit != 0 {
                        d.times[i] = Some(if unix {
                            RawTime::Unix(w.u32()?, None)
                        } else {
                            RawTime::Filetime(w.u64()?)
                        });
                    }
                }
                if unix && flags & 16 != 0 {
                    for (i, bit) in [2, 4, 8].into_iter().enumerate() {
                        if flags & bit != 0 {
                            let nanos = w.u32()?;
                            if let Some(RawTime::Unix(_, ref mut n)) = d.times[i] {
                                *n = Some(nanos);
                            }
                        }
                    }
                }
                w.end()?;
            }
            4 => {
                if w.vint()? != 0 {
                    return Err(ScanError::Unsupported);
                }
                d.version = Some(w.vint()?);
                w.end()?;
            }
            5 => {
                d.redirection = true;
                d.special = true;
                w.vint()?;
                w.vint()?;
                let n = count(w.vint()?)?;
                if n > limits.name_bytes {
                    return Err(ScanError::Limit);
                }
                w.take(n)?;
                w.end()?;
            }
            6 => {
                let flags = w.vint()?;
                if flags & !15 != 0 {
                    return Err(ScanError::Unsupported);
                }
                for bit in [1, 2] {
                    if flags & bit != 0 {
                        let n = count(w.vint()?)?;
                        if n > limits.name_bytes {
                            return Err(ScanError::Limit);
                        }
                        w.take(n)?;
                    }
                }
                for bit in [4, 8] {
                    if flags & bit != 0 {
                        w.vint()?;
                    }
                }
                w.end()?;
            }
            _ => (),
        }
    }
    Ok(())
}
fn file(
    w: &mut Wire<'_>,
    extra: &[u8],
    limits: ScanLimits,
    service: bool,
) -> Result<Descriptor, ScanError> {
    let flags = w.vint()?;
    if flags & !15 != 0 {
        return Err(ScanError::Unsupported);
    }
    let size = w.vint()?;
    let attributes = w.vint()?;
    let mut d = Descriptor {
        size: if flags & 8 == 0 { Some(size) } else { None },
        attributes,
        directory: !service && flags & 1 != 0,
        ..Descriptor::default()
    };
    if flags & 2 != 0 {
        d.times[0] = Some(RawTime::Unix(w.u32()?, None));
    }
    if flags & 4 != 0 {
        d.crc = Some(w.u32()?);
    }
    let info = w.vint()?;
    d.declared = (info & 63) as u8;
    d.effective = if d.declared == 1 && info & 0x100000 != 0 {
        0
    } else {
        d.declared
    };
    d.method = match d.effective {
        0 => 50,
        1 => 70,
        _ => 9999,
    };
    d.solid = !service && info & 64 != 0;
    d.compression = ((info >> 7) & 7) as u8;
    if d.compression > 5 {
        return Err(ScanError::Unsupported);
    }
    if d.declared <= 1 {
        let bits = (info >> 10) & if d.declared == 0 { 15 } else { 31 };
        let base = (128u64 * 1024)
            .checked_shl(bits as u32)
            .ok_or(ScanError::Invalid)?;
        d.dictionary = if d.declared == 1 {
            base.checked_add((base / 32) * ((info >> 15) & 31))
                .ok_or(ScanError::Invalid)?
        } else {
            base
        };
        let allowed = if d.declared == 0 { 0x3fff } else { 0x1fffff };
        if info & !allowed != 0 {
            return Err(ScanError::Unsupported);
        }
    } else if d.compression != 0 {
        return Err(ScanError::Unsupported);
    }
    d.host = w.vint()?;
    let n = count(w.vint()?)?;
    if n > limits.name_bytes {
        return Err(ScanError::Limit);
    }
    d.name = w.take(n)?.to_vec();
    let name = std::str::from_utf8(&d.name).map_err(|_| ScanError::Invalid)?;
    if name.contains('\0') {
        return Err(ScanError::Invalid);
    }
    if name.contains('\u{fffe}') {
        d.special = true;
    }
    w.end()?;
    extras(extra, &mut d, limits)?;
    if d.host == 1 {
        let mode = d.attributes & 0xf000;
        if mode != 0 && mode != 0x8000 && mode != 0x4000 {
            d.special = true;
        }
        if mode == 0x4000 && !d.directory {
            d.special = true;
        }
    }
    Ok(d)
}
fn header_size(prefix: &[u8]) -> Result<(usize, usize), ScanError> {
    if prefix.len() < 5 {
        return Err(ScanError::Invalid);
    }
    let mut bytes = 1usize;
    while prefix[4 + bytes - 1] & 128 != 0 {
        if bytes == 3 || prefix.len() <= 4 + bytes {
            return Err(ScanError::Invalid);
        }
        bytes += 1;
    }
    let body = count(Wire::new(&prefix[4..4 + bytes]).vint()?)?;
    let total = (4 + bytes).checked_add(body).ok_or(ScanError::Invalid)?;
    if total > HEAD_LIMIT {
        return Err(ScanError::Limit);
    }
    Ok((bytes, total))
}
fn parse_header(
    reader: &Reader<'_>,
    volume: usize,
    at: u64,
    header: &[u8],
    span: u64,
    limits: ScanLimits,
) -> Result<Parsed, ScanError> {
    let (encoded, total) = header_size(header)?;
    if total != header.len() {
        return Err(ScanError::Invalid);
    }
    if crc32(&header[4..])
        != u32::from_le_bytes(header[..4].try_into().map_err(|_| ScanError::Invalid)?)
    {
        return Err(ScanError::Invalid);
    }
    let mut common = Wire::new(&header[4 + encoded..]);
    let kind = common.vint()?;
    let flags = common.vint()?;
    if flags & !127 != 0 {
        return Err(ScanError::Unsupported);
    }
    let extra = if flags & 1 != 0 {
        count(common.vint()?)?
    } else {
        0
    };
    let packed = if flags & 2 != 0 { common.vint()? } else { 0 };
    if extra > common.remaining() {
        return Err(ScanError::Invalid);
    }
    let fields = common.take(common.remaining() - extra)?;
    let extra = common.rest();
    let mut w = Wire::new(fields);
    let data_at = at.checked_add(span).ok_or(ScanError::Invalid)?;
    reader.range(volume, data_at, packed)?;
    let next = data_at.checked_add(packed).ok_or(ScanError::Invalid)?;
    let block = match kind {
        1 => {
            if packed != 0 || flags & 0x78 != 0 {
                return Err(ScanError::Invalid);
            }
            let flags = w.vint()?;
            if flags & !31 != 0 {
                return Err(ScanError::Unsupported);
            }
            let number = if flags & 2 != 0 {
                Some(w.vint()?)
            } else {
                Some(0)
            };
            w.end()?;
            main_extra(extra, limits)?;
            Block::Main {
                volume: flags & 1 != 0,
                number,
                solid: flags & 4 != 0,
                encrypted: false,
            }
        }
        2 | 3 => {
            let description = file(&mut w, extra, limits, kind == 3)?;
            Block::Record(Box::new(Part {
                volume,
                header_at: at,
                header: Vec::new(),
                data_at,
                packed,
                before: flags & 8 != 0,
                after: flags & 16 != 0,
                service: kind == 3,
                description,
            }))
        }
        4 => {
            if packed != 0 || !extra.is_empty() || flags & !4 != 0 {
                return Err(ScanError::Invalid);
            }
            let crypt = crypt(&mut w, false)?;
            w.end()?;
            // A valid encrypted-header marker must still contain its first IV and block.
            reader.range(volume, data_at, 32)?;
            Block::HeaderCrypt(crypt)
        }
        5 => {
            let end = w.vint()?;
            w.end()?;
            if end & !1 != 0 || packed != 0 {
                return Err(ScanError::Unsupported);
            }
            records(extra)?;
            Block::End {
                next: end & 1 != 0,
                number: None,
            }
        }
        _ => return Err(ScanError::Unsupported),
    };
    Ok(Parsed { block, next })
}
fn plain_header(reader: &mut Reader<'_>, volume: usize, at: u64) -> Result<Vec<u8>, ScanError> {
    let mut prefix = reader.bytes(volume, at, 5)?;
    while prefix.last().is_some_and(|b| b & 128 != 0) {
        if prefix.len() == 7 {
            return Err(ScanError::Invalid);
        }
        prefix.push(reader.bytes(volume, at + prefix.len() as u64, 1)?[0]);
    }
    let (_, total) = header_size(&prefix)?;
    reader.bytes(volume, at, total)
}

#[derive(Default)]
pub(crate) struct State {
    crypt: Option<Crypt>,
    main: bool,
    trusted: bool,
}
impl State {
    pub fn block(
        &mut self,
        reader: &mut Reader<'_>,
        volume: usize,
        at: u64,
        limits: ScanLimits,
        password: Option<&str>,
        protection: &mut Protection,
    ) -> Result<Parsed, ScanError> {
        let unverified = self.crypt.is_some() && !self.trusted;
        self.block_inner(reader, volume, at, limits, password, protection)
            .map_err(|error| crate::protection::encrypted_scan_error(error, unverified))
    }
    fn block_inner(
        &mut self,
        reader: &mut Reader<'_>,
        volume: usize,
        at: u64,
        limits: ScanLimits,
        password: Option<&str>,
        protection: &mut Protection,
    ) -> Result<Parsed, ScanError> {
        let (plain, physical) = if let Some(crypt) = &self.crypt {
            let prefix = reader.bytes(volume, at, 32)?;
            let iv = prefix[..16].try_into().map_err(|_| ScanError::Invalid)?;
            let key = protection
                .key(crypt, password, false, reader.control())
                .map_err(scan_error)?;
            let mut first = Zeroizing::new([0u8; 16]);
            {
                let mut input =
                    CbcReader::new(Cursor::new(&prefix[16..]), 16, key, iv, reader.control())
                        .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
                input
                    .read_exact(first.as_mut())
                    .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
                input
                    .finish()
                    .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
            }
            let (_, length) = header_size(first.as_ref())?;
            let cipher = length.checked_add(15).ok_or(ScanError::Invalid)? / 16 * 16;
            let span = cipher.checked_add(16).ok_or(ScanError::Invalid)?;
            let physical = reader.bytes(volume, at, span)?;
            let mut plain = Zeroizing::new(vec![0u8; cipher]);
            {
                let mut input = CbcReader::new(
                    Cursor::new(&physical[16..]),
                    cipher as u64,
                    key,
                    iv,
                    reader.control(),
                )
                .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
                input
                    .read_exact(plain.as_mut_slice())
                    .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
                input
                    .finish()
                    .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
            }
            plain.truncate(length);
            (plain, physical)
        } else {
            let physical = plain_header(reader, volume, at)?;
            (Zeroizing::new(physical.clone()), physical)
        };
        let mut parsed = parse_header(reader, volume, at, &plain, physical.len() as u64, limits)?;
        match &mut parsed.block {
            Block::HeaderCrypt(crypt) => {
                if self.main || self.crypt.is_some() {
                    return Err(ScanError::Invalid);
                }
                let key = protection
                    .key(crypt, password, false, reader.control())
                    .map_err(scan_error)?;
                self.trusted = crate::protection::trusted_check(crypt, key, false);
                self.crypt = Some(crypt.clone());
            }
            Block::Main { encrypted, .. } => {
                self.main = true;
                *encrypted = self.crypt.is_some();
            }
            Block::Record(part) => part.header = physical,
            _ => (),
        }
        Ok(parsed)
    }
}
