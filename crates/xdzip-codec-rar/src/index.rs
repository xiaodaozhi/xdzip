use crate::protection::Protection;
use crate::{FILE_LIMIT, MEMORY_LIMIT, legacy, modern, names, source::Reader};
use xdzip_codec_api::names::NameEncoding;
use xdzip_codec_api::{Entry, Format, ScanError, ScanLimits};
use xdzip_domain::identity::EntryId;

const SEARCH: u64 = 0x400000;
const OLD: &[u8] = b"Rar!\x1a\x07\x00";
const NEW: &[u8] = b"Rar!\x1a\x07\x01\x00";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Generation {
    Ancient,
    Legacy,
    Modern,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum RawTime {
    Dos(u32),
    DosExtended(u32, bool, u32),
    Unix(u32, Option<u32>),
    Filetime(u64),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Crypt {
    pub version: u64,
    pub flags: u64,
    pub exponent: u8,
    pub salt: Vec<u8>,
    pub iv: Vec<u8>,
    pub check: Option<[u8; 12]>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Descriptor {
    pub name: Vec<u8>,
    pub unicode: Option<String>,
    pub legacy: bool,
    pub ancient: bool,
    pub size: Option<u64>,
    pub method: u16,
    pub compression: u8,
    pub declared: u8,
    pub effective: u8,
    pub dictionary: u64,
    pub directory: bool,
    pub special: bool,
    pub solid: bool,
    pub encrypted: bool,
    pub attributes: u64,
    pub host: u64,
    pub times: [Option<RawTime>; 3],
    pub crc: Option<u32>,
    pub hash: Option<[u8; 32]>,
    pub crypt: Option<Crypt>,
    pub version: Option<u64>,
    pub redirection: bool,
    pub old_checksum: Option<u16>,
}
#[derive(Clone, Debug)]
pub(crate) struct Part {
    pub volume: usize,
    pub header_at: u64,
    pub header: Vec<u8>,
    pub data_at: u64,
    pub packed: u64,
    pub before: bool,
    pub after: bool,
    pub service: bool,
    pub description: Descriptor,
}
pub(crate) enum Block {
    Main {
        volume: bool,
        number: Option<u64>,
        solid: bool,
        encrypted: bool,
    },
    Record(Box<Part>),
    End {
        next: bool,
        number: Option<u64>,
    },
    Skip,
    HeaderCrypt(Crypt),
}
pub(crate) struct Parsed {
    pub block: Block,
    pub next: u64,
}
pub(crate) struct File {
    pub parts: Vec<usize>,
    pub size: Option<u64>,
}
pub(crate) struct Index {
    pub rows: Vec<Entry>,
    pub parts: Vec<Part>,
    pub files: Vec<File>,
    pub protection: Protection,
}

fn signature(reader: &mut Reader<'_>, volume: usize) -> Result<(Generation, u64), ScanError> {
    let length = reader.volumes[volume].length;
    let prefix = reader.bytes(
        volume,
        0,
        usize::try_from(length.min(8)).map_err(|_| ScanError::Invalid)?,
    )?;
    if prefix.starts_with(NEW) {
        return Ok((Generation::Modern, 8));
    }
    if prefix.starts_with(OLD) {
        return Ok((Generation::Legacy, 7));
    }
    if prefix.starts_with(b"RE~^") {
        return Ok((Generation::Ancient, 0));
    }
    if prefix.starts_with(b"Rar!\x1a\x07") {
        return Err(if prefix.len() >= 8 && prefix[6] > 1 {
            ScanError::Unsupported
        } else {
            ScanError::Invalid
        });
    }
    let limit = length.min(SEARCH + 8);
    let mut at = 0u64;
    while at < limit {
        let count = usize::try_from((limit - at).min(32768)).map_err(|_| ScanError::Limit)?;
        let bytes = reader.bytes(volume, at, count)?;
        for i in 0..bytes.len() {
            if at + i as u64 > SEARCH {
                break;
            }
            let rest = &bytes[i..];
            if rest.starts_with(NEW) {
                return Ok((Generation::Modern, at + i as u64 + 8));
            }
            if rest.starts_with(OLD) {
                return Ok((Generation::Legacy, at + i as u64 + 7));
            }
            if rest.starts_with(b"RE~^") {
                let position = at + i as u64;
                if position > SEARCH - 19 {
                    return Err(ScanError::Unsupported);
                }
                if position >= 2
                    && length.saturating_sub(1).min(SEARCH - 16) > 31
                    && reader.bytes(volume, 28, 4)? != b"RSFX"
                {
                    return Err(ScanError::Unsupported);
                }
                return Ok((Generation::Ancient, position));
            }
            if rest.starts_with(b"Rar!\x1a\x07") && rest.len() >= 8 && rest[6] > 1 {
                return Err(ScanError::Unsupported);
            }
        }
        if count <= 7 {
            break;
        }
        at += count as u64 - 7;
    }
    Err(ScanError::Invalid)
}
fn compatible(a: &Part, b: &Part) -> bool {
    let x = &a.description;
    let y = &b.description;
    a.service == b.service
        && x.name == y.name
        && x.unicode == y.unicode
        && x.method == y.method
        && x.ancient == y.ancient
        && x.compression == y.compression
        && x.declared == y.declared
        && x.effective == y.effective
        && x.dictionary == y.dictionary
        && x.directory == y.directory
        && x.special == y.special
        && x.solid == y.solid
        && x.encrypted == y.encrypted
        && x.attributes == y.attributes
        && x.host == y.host
        && x.version == y.version
        && x.redirection == y.redirection
        && x.hash.is_some() == y.hash.is_some()
        && x.crc.is_some() == y.crc.is_some()
        && x.times == y.times
        && x.crypt == y.crypt
}
fn append(
    index: &mut Index,
    part: Part,
    pending: &mut Option<usize>,
    services: &mut Option<usize>,
    limits: ScanLimits,
    names_used: &mut usize,
) -> Result<(), ScanError> {
    if part.description.name.len() > limits.name_bytes {
        return Err(ScanError::Limit);
    }
    *names_used = names_used
        .checked_add(part.description.name.len())
        .ok_or(ScanError::Limit)?;
    if *names_used > limits.total_name_bytes {
        return Err(ScanError::Limit);
    }
    if part.description.size.is_some_and(|size| size > FILE_LIMIT) {
        return Err(ScanError::Limit);
    }
    if !part.description.directory && part.description.dictionary > MEMORY_LIMIT {
        return Err(ScanError::Limit);
    }
    let at = index.parts.len();
    let chain = if part.service { services } else { pending };
    if part.before {
        let previous = chain.take().ok_or(ScanError::Invalid)?;
        let prior = &index.parts[previous];
        if !prior.after || prior.volume + 1 != part.volume || !compatible(prior, &part) {
            return Err(ScanError::Invalid);
        }
        if part.service
            && prior
                .description
                .size
                .zip(part.description.size)
                .is_some_and(|(a, b)| a != b)
        {
            return Err(ScanError::Invalid);
        }
        if !part.service {
            let file = index.files.last_mut().ok_or(ScanError::Invalid)?;
            if file.parts.last().copied() != Some(previous) {
                return Err(ScanError::Invalid);
            }
            if let Some(size) = part.description.size {
                if file.size.is_some_and(|old| old != size) {
                    return Err(ScanError::Invalid);
                }
                file.size = Some(size);
            }
            file.parts.push(at);
        }
    } else {
        if chain.is_some() {
            return Err(ScanError::Invalid);
        }
        if !part.service {
            index.files.push(File {
                parts: vec![at],
                size: part.description.size,
            });
        }
    }
    if part.after {
        *chain = Some(at);
    }
    index.parts.push(part);
    Ok(())
}
pub(crate) fn parse(
    reader: &mut Reader<'_>,
    limits: ScanLimits,
    password: Option<&str>,
) -> Result<Index, ScanError> {
    let mut index = Index {
        rows: Vec::new(),
        parts: Vec::new(),
        files: Vec::new(),
        protection: Protection::default(),
    };
    let mut generation = None;
    let mut pending = None;
    let mut services = None;
    let mut records = 0usize;
    let mut names_used = 0usize;
    let mut previous_next = false;
    let mut encrypted_state = None;
    for volume in 0..reader.volumes.len() {
        reader.check()?;
        if volume > 0 && !previous_next && pending.is_none() && services.is_none() {
            return Err(ScanError::Invalid);
        }
        let (family, mut at) = signature(reader, volume)?;
        if generation.is_some_and(|old| old != family) {
            return Err(ScanError::Invalid);
        }
        generation = Some(family);
        let mut main = false;
        let mut ended = false;
        let mut next = false;
        let mut global_solid = false;
        let mut header_encrypted = false;
        let mut modern_state = modern::State::default();
        let mut legacy_state = legacy::State::default();
        while at < reader.volumes[volume].length {
            records = records.checked_add(1).ok_or(ScanError::Limit)?;
            if records > limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
                return Err(ScanError::Limit);
            }
            let Parsed {
                mut block,
                next: position,
            } = match family {
                Generation::Modern => modern_state.block(
                    reader,
                    volume,
                    at,
                    limits,
                    password,
                    &mut index.protection,
                )?,
                Generation::Legacy => legacy_state.block(
                    reader,
                    volume,
                    at,
                    limits,
                    password,
                    &mut index.protection,
                )?,
                Generation::Ancient => legacy::ancient(reader, volume, at, !main, limits)?,
            };
            if position <= at {
                return Err(ScanError::Invalid);
            }
            reader.range(volume, at, position - at)?;
            match &mut block {
                Block::Main {
                    volume: multi,
                    number,
                    solid,
                    encrypted,
                } => {
                    if main || index.parts.iter().any(|part| part.volume == volume) {
                        return Err(ScanError::Invalid);
                    }
                    if (reader.volumes.len() > 1 || volume > 0) && !*multi {
                        return Err(ScanError::Invalid);
                    }
                    if number.is_some_and(|n| n != volume as u64) {
                        return Err(ScanError::Invalid);
                    }
                    if encrypted_state.is_some_and(|old| old != *encrypted) {
                        return Err(ScanError::Invalid);
                    }
                    encrypted_state = Some(*encrypted);
                    header_encrypted = *encrypted;
                    main = true;
                    global_solid = *solid;
                }
                Block::Record(part) => {
                    if !main {
                        return Err(ScanError::Invalid);
                    }
                    part.description.ancient = family == Generation::Ancient;
                    if family != Generation::Modern && part.description.method <= 15 {
                        part.description.solid = if part.before {
                            pending
                                .map(|n: usize| index.parts[n].description.solid)
                                .unwrap_or(false)
                        } else {
                            !part.service && global_solid && !index.files.is_empty()
                        };
                    }
                    append(
                        &mut index,
                        *part.clone(),
                        &mut pending,
                        &mut services,
                        limits,
                        &mut names_used,
                    )?;
                }
                Block::End { next: n, number } => {
                    if !main || number.is_some_and(|n| n != volume as u64) {
                        return Err(ScanError::Invalid);
                    }
                    next = *n;
                    ended = true;
                    break;
                }
                Block::Skip => {
                    if !main {
                        return Err(ScanError::Invalid);
                    }
                }
                Block::HeaderCrypt(_) => {
                    if main || family != Generation::Modern || header_encrypted {
                        return Err(ScanError::Invalid);
                    }
                    header_encrypted = true;
                }
            }
            at = position;
        }
        if !main || (family == Generation::Modern && !ended) {
            return Err(ScanError::Invalid);
        }
        if family == Generation::Modern
            && ended
            && !next
            && (pending.is_some() || services.is_some())
        {
            return Err(ScanError::Invalid);
        }
        if (next || pending.is_some() || services.is_some()) && volume + 1 == reader.volumes.len() {
            return Err(ScanError::MissingVolume);
        }
        previous_next = next;
    }
    if pending.is_some() || services.is_some() {
        return Err(ScanError::MissingVolume);
    }
    let mut legacy_flags = Vec::with_capacity(index.files.len());
    let mut warnings = Vec::with_capacity(index.files.len());
    for (n, file) in index.files.iter().enumerate() {
        let part = &index.parts[file.parts[0]];
        let d = &part.description;
        let size = file.size.unwrap_or(0);
        let warning = d.special
            || d.redirection
            || (d.compression != 0
                && !matches!(d.method, 13 | 15 | 20 | 26 | 29 | 50 | 70)
                && !d.directory);
        let mut decoded = d.unicode.clone();
        if let Some(version) = d.version.filter(|v| *v != 0) {
            if decoded.is_none() {
                decoded = std::str::from_utf8(&d.name).ok().map(str::to_owned);
            }
            if let Some(name) = &mut decoded {
                use std::fmt::Write;
                write!(name, ";{version}").map_err(|_| ScanError::Limit)?;
            } else {
                return Err(ScanError::Unsupported);
            }
        }
        index.rows.push(Entry {
            format: Format::Rar,
            id: EntryId::try_from(n as u64 + 1).map_err(|_| ScanError::Limit)?,
            raw_name: d.name.clone(),
            decoded_name: decoded,
            name_encoding: NameEncoding::Utf8,
            display_name: String::new(),
            directory: d.directory,
            link: d.special || d.redirection,
            unix_mode: 0,
            size,
            size_known: file.size.is_some(),
            packed_size: part.packed,
            method: d.method,
            modified_dos: crate::time::modified(d.times[0].as_ref(), reader)?,
            encrypted: d.encrypted,
            password_supported: d.encrypted
                && (d.legacy || (matches!(d.method, 50 | 70) && d.crypt.is_some())),
            warning,
        });
        legacy_flags.push(d.legacy && d.unicode.is_none());
        warnings.push(warning);
    }
    names::resolve_rows(&mut index.rows, &legacy_flags, &warnings, reader, limits)?;
    Ok(index)
}
