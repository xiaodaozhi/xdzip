use crate::index::{Block, Crypt, Descriptor, Parsed, Part, RawTime};
use crate::legacy_crypto::LegacyReader;
use crate::protection::{Protection, scan_error};
use crate::{names, source::Reader, wire::Wire};
use std::io::{Cursor, Read};
use xdzip_codec_api::{ScanError, ScanLimits};
use xdzip_compression::crc32;
use zeroize::Zeroizing;

fn extended(w: &mut Wire<'_>, d: &mut Descriptor) -> Result<(), ScanError> {
    let flags = w.u16()?;
    for i in 0..4 {
        let nibble = (flags >> (12 - i * 4)) & 15;
        if nibble & 8 == 0 {
            continue;
        }
        let time = if i == 0 {
            match d.times[0] {
                Some(RawTime::Dos(time)) => time,
                _ => 0,
            }
        } else {
            w.u32()?
        };
        // Preserve DOS's local-wall-clock representation; fractional source
        // bytes remain in the frozen physical header for later time projection.
        let c = usize::from(nibble & 3);
        let fraction = w
            .take(c)?
            .iter()
            .enumerate()
            .fold(0u32, |v, (j, b)| v | (u32::from(*b) << (8 * (j + 3 - c))));
        if i < 3 {
            d.times[i] = Some(RawTime::DosExtended(time, nibble & 4 != 0, fraction));
        }
    }
    Ok(())
}
fn file(
    header: &[u8],
    flags: u16,
    limits: ScanLimits,
    service: bool,
) -> Result<(Descriptor, u64), ScanError> {
    let mut w = Wire::new(&header[7..]);
    let packed = w.u32()?;
    let unpacked = w.u32()?;
    let host = u64::from(w.byte()?);
    let crc = w.u32()?;
    let time = w.u32()?;
    let method = u16::from(w.byte()?);
    let compression = w
        .byte()?
        .checked_sub(0x30)
        .filter(|m| *m <= 5)
        .ok_or(ScanError::Unsupported)?;
    let n = usize::from(w.u16()?);
    let attributes = u64::from(w.u32()?);
    let (packed, unpacked, unknown) = if flags & 0x100 != 0 {
        let packed_high = w.u32()?;
        let unpacked_high = w.u32()?;
        (
            (u64::from(packed_high) << 32) | u64::from(packed),
            (u64::from(unpacked_high) << 32) | u64::from(unpacked),
            unpacked_high == u32::MAX && unpacked == u32::MAX,
        )
    } else {
        (u64::from(packed), u64::from(unpacked), unpacked == u32::MAX)
    };
    if n > limits.name_bytes {
        return Err(ScanError::Limit);
    }
    let name = w.take(n)?.to_vec();
    if flags & 8 != 0 {
        return Err(ScanError::Unsupported);
    }
    if service && flags & (0x100 | 0x1000) != 0 {
        return Err(ScanError::Unsupported);
    }
    let directory = (!service && (flags & 0xe0) == 0xe0) || (method < 20 && attributes & 0x10 != 0);
    let mut d = Descriptor {
        name,
        legacy: true,
        size: if unknown { None } else { Some(unpacked) },
        method,
        compression,
        host,
        attributes,
        directory,
        solid: !service && flags & 0x10 != 0,
        encrypted: flags & 4 != 0,
        crc: Some(crc),
        dictionary: if directory {
            0
        } else {
            65536u64 << ((flags >> 5) & 7)
        },
        times: [Some(RawTime::Dos(time)), None, None],
        ..Descriptor::default()
    };
    if host == 3 {
        let mode = attributes & 0xf000;
        if mode != 0 && mode != 0x8000 && mode != 0x4000 {
            d.special = true;
        }
    }
    if flags & 0x200 != 0 {
        d.unicode = names::compressed(&d.name)?;
    }
    if service {
        // The legacy service's optional subdata is bounded by this exact header.
        let salt = if flags & 0x400 != 0 { 8 } else { 0 };
        if w.remaining() < salt {
            return Err(ScanError::Invalid);
        }
        w.take(w.remaining() - salt)?;
    }
    if flags & 0x400 != 0 {
        let salt = w.take(8)?.to_vec();
        d.crypt = Some(Crypt {
            version: u64::from(method),
            flags: 0,
            exponent: 0,
            salt,
            iv: Vec::new(),
            check: None,
        });
    }
    if flags & 0x1000 != 0 {
        extended(&mut w, &mut d)?;
    }
    w.end()?;
    Ok((d, packed))
}
fn parse_header(
    reader: &Reader<'_>,
    volume: usize,
    at: u64,
    header: &[u8],
    span: usize,
    limits: ScanLimits,
) -> Result<Parsed, ScanError> {
    let common = header.get(..7).ok_or(ScanError::Invalid)?;
    let mut w = Wire::new(common);
    let expected = w.u16()?;
    let kind = w.byte()?;
    let flags = w.u16()?;
    let n = usize::from(w.u16()?);
    if n < 7 || n != header.len() {
        return Err(ScanError::Invalid);
    }
    let exceptional = matches!(kind, 0x76 | 0x79)
        || (kind == 0x73 && flags & 2 != 0)
        || (matches!(kind, 0x74 | 0x7a) && flags & 8 != 0)
        || (kind == 0x7b && flags & 4 != 0)
        || (kind == 0x77 && header.get(11..13) == Some(&101u16.to_le_bytes()));
    if exceptional {
        return Err(ScanError::Unsupported);
    }
    if (crc32(&header[2..]) & 65535) as u16 != expected {
        return Err(ScanError::Invalid);
    }
    let mut packed = 0;
    let block = match kind {
        0x73 => {
            if n < 13 {
                return Err(ScanError::Invalid);
            }
            if flags & 2 != 0 {
                return Err(ScanError::Unsupported);
            }
            if n != 13 {
                return Err(ScanError::Unsupported);
            }
            Block::Main {
                volume: flags & 1 != 0,
                number: if flags & 0x100 != 0 { Some(0) } else { None },
                solid: flags & 8 != 0,
                encrypted: flags & 0x80 != 0,
            }
        }
        0x74 | 0x7a => {
            let (description, size) = file(header, flags, limits, kind == 0x7a)?;
            packed = size;
            Block::Record(Box::new(Part {
                volume,
                header_at: at,
                header: Vec::new(),
                data_at: at.checked_add(span as u64).ok_or(ScanError::Invalid)?,
                packed,
                before: flags & 1 != 0,
                after: flags & 2 != 0,
                service: kind == 0x7a,
                description,
            }))
        }
        0x7b => {
            let mut w = Wire::new(&header[7..]);
            if flags & 2 != 0 {
                w.u32()?;
            }
            let number = if flags & 8 != 0 {
                Some(u64::from(w.u16()?))
            } else {
                None
            };
            if flags & 4 != 0 {
                return Err(ScanError::Unsupported);
            }
            w.end()?;
            Block::End {
                next: flags & 1 != 0,
                number,
            }
        }
        0x75 => {
            if n < 13 {
                return Err(ScanError::Invalid);
            }
            Block::Skip
        }
        0x76 | 0x79 => return Err(ScanError::Unsupported),
        0x77 => {
            if n < 14 {
                return Err(ScanError::Invalid);
            }
            let mut w = Wire::new(&header[7..]);
            packed = u64::from(w.u32()?);
            if w.u16()? == 101 {
                return Err(ScanError::Unsupported);
            }
            w.byte()?;
            Block::Skip
        }
        0x78 => {
            if n < 26 {
                return Err(ScanError::Invalid);
            }
            packed = u64::from(Wire::new(&header[7..]).u32()?);
            Block::Skip
        }
        _ => return Err(ScanError::Unsupported),
    };
    let data_at = at.checked_add(span as u64).ok_or(ScanError::Invalid)?;
    reader.range(volume, data_at, packed)?;
    let next = data_at.checked_add(packed).ok_or(ScanError::Invalid)?;
    if matches!(
        block,
        Block::Main {
            encrypted: true,
            ..
        }
    ) {
        reader.range(volume, next, 24)?;
    }
    Ok(Parsed { block, next })
}
#[derive(Default)]
pub(crate) struct State {
    encrypted: bool,
    main: bool,
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
        let encrypted = self.encrypted;
        self.block_inner(reader, volume, at, limits, password, protection)
            .map_err(|error| crate::protection::encrypted_scan_error(error, encrypted))
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
        let (plain, physical) = if self.encrypted {
            let prefix = reader.bytes(volume, at, 24)?;
            let salt = prefix[..8].try_into().map_err(|_| ScanError::Invalid)?;
            let mut first = Zeroizing::new([0u8; 16]);
            {
                let cipher = protection
                    .legacy_header(password, salt, reader.control())
                    .map_err(scan_error)?;
                let mut input =
                    LegacyReader::new(Cursor::new(&prefix[8..]), 16, cipher, reader.control())
                        .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
                input
                    .read_exact(first.as_mut())
                    .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
                input
                    .finish()
                    .map_err(|e| scan_error(crate::algorithm::input_error(e)))?;
            }
            let length = usize::from(u16::from_le_bytes(
                first[5..7].try_into().map_err(|_| ScanError::Invalid)?,
            ));
            if length < 7 {
                return Err(ScanError::Invalid);
            }
            let cipher_length = length.checked_add(15).ok_or(ScanError::Invalid)? / 16 * 16;
            let physical = reader.bytes(
                volume,
                at,
                cipher_length.checked_add(8).ok_or(ScanError::Invalid)?,
            )?;
            let mut plain = Zeroizing::new(vec![0u8; cipher_length]);
            {
                let cipher = protection
                    .legacy_header(password, salt, reader.control())
                    .map_err(scan_error)?;
                let mut input = LegacyReader::new(
                    Cursor::new(&physical[8..]),
                    cipher_length as u64,
                    cipher,
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
            let common = reader.bytes(volume, at, 7)?;
            let length = usize::from(u16::from_le_bytes(
                common[5..7].try_into().map_err(|_| ScanError::Invalid)?,
            ));
            if length < 7 {
                return Err(ScanError::Invalid);
            }
            let physical = reader.bytes(volume, at, length)?;
            (Zeroizing::new(physical.clone()), physical)
        };
        let mut parsed = parse_header(reader, volume, at, &plain, physical.len(), limits)?;
        match &mut parsed.block {
            Block::Main { encrypted, .. } => {
                if self.main {
                    return Err(ScanError::Invalid);
                }
                self.main = true;
                if *encrypted && password.is_none() {
                    return Err(ScanError::PasswordRequired);
                }
                self.encrypted = *encrypted;
            }
            Block::Record(part) => part.header = physical,
            _ => (),
        }
        Ok(parsed)
    }
}
pub(crate) fn ancient(
    reader: &mut Reader<'_>,
    volume: usize,
    at: u64,
    limits: ScanLimits,
) -> Result<Parsed, ScanError> {
    if at == 0 {
        let header = reader.bytes(volume, 0, 7)?;
        let mut w = Wire::new(&header[4..]);
        let n = u64::from(w.u16()?);
        let flags = w.byte()?;
        if n < 7 {
            return Err(ScanError::Invalid);
        }
        reader.range(volume, 0, n)?;
        return Ok(Parsed {
            block: Block::Main {
                volume: flags & 1 != 0,
                number: None,
                solid: flags & 8 != 0,
                encrypted: false,
            },
            next: n,
        });
    }
    let fixed = reader.bytes(volume, at, 21)?;
    let mut w = Wire::new(&fixed);
    let packed = u64::from(w.u32()?);
    let unpacked = u64::from(w.u32()?);
    let checksum = w.u16()?;
    let n = usize::from(w.u16()?);
    let time = w.u32()?;
    let attributes = u64::from(w.byte()?);
    let flags = w.byte()?;
    let method = if w.byte()? == 2 { 13 } else { 10 };
    let name_len = usize::from(w.byte()?);
    let compression = w.byte()?;
    if compression > 5 {
        return Err(ScanError::Unsupported);
    }
    if n < 21 || name_len > n - 21 {
        return Err(ScanError::Invalid);
    }
    if name_len > limits.name_bytes {
        return Err(ScanError::Limit);
    }
    let header = reader.bytes(volume, at, n)?;
    let data_at = at.checked_add(n as u64).ok_or(ScanError::Invalid)?;
    reader.range(volume, data_at, packed)?;
    let d = Descriptor {
        name: header[21..21 + name_len].to_vec(),
        legacy: true,
        ancient: true,
        size: Some(unpacked),
        method,
        compression,
        attributes,
        directory: attributes & 16 != 0,
        times: [Some(RawTime::Dos(time)), None, None],
        old_checksum: Some(checksum),
        dictionary: 65536,
        encrypted: flags & 4 != 0,
        ..Descriptor::default()
    };
    Ok(Parsed {
        block: Block::Record(Box::new(Part {
            volume,
            header_at: at,
            header,
            data_at,
            packed,
            before: flags & 1 != 0,
            after: flags & 2 != 0,
            service: false,
            description: d,
        })),
        next: data_at.checked_add(packed).ok_or(ScanError::Invalid)?,
    })
}
