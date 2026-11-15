use std::io::{self, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits, time};
use xdzip_compression::crc32;
use xdzip_crypto::SecretSha256;
const MAX_HEADER: usize = 2600;
const MAX_EXTENSIONS: usize = 256;

pub(crate) fn source_error(error: &io::Error) -> ScanError {
    host_error(ScanError::from_source_error(error))
}
fn host_error(error: ScanError) -> ScanError {
    match error {
        stop @ (ScanError::Cancelled
        | ScanError::Limit
        | ScanError::SourceChanged
        | ScanError::Io) => stop,
        _ => ScanError::Io,
    }
}
/// One validated physical member. Metadata grants no path or content authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// Physical base-header position, independent of duplicate names.
    pub header_offset: u64,
    /// Exact packed span begins here, after every extended header.
    pub payload_offset: u64,
    /// Declared packed length, excluding headers and following records.
    pub packed_size: u64,
    /// Declared decoded length, verified only by full decoding and CRC.
    pub size: u64,
    /// Original method byte, including unknown methods.
    pub method: u8,
    /// Binary 0, legacy text 1, directory 3; other types remain explicit.
    pub kind: u8,
    /// Original physical flags, including encryption and continuation.
    pub flags: u8,
    /// Original archive version, distinct from required decoder version.
    pub version: u8,
    /// Required decoder version; future profiles do not gain payload support.
    pub required_version: u8,
    /// Original host identifier, with no platform execution authority.
    pub host: u8,
    /// Original first-header width, not the complete base-header size.
    pub first_size: u8,
    /// Complete filename bytes, before decoding or separator interpretation.
    pub raw_name: Vec<u8>,
    /// Declared filename position, retained without basename truncation.
    pub name_position: u16,
    /// Required IEEE CRC-32 of the complete decoded payload.
    pub crc32: u32,
    /// Host-projected DOS-shaped local display time; zero is unavailable.
    pub modified_dos: u32,
    /// Original access mode or platform attributes, never applied to output.
    pub attributes: u16,
    /// Original modifier, without password admission or credential authority.
    pub password_modifier: u8,
    /// Original encryption/extended flags, without implicit support.
    pub extended_flags: u8,
    /// Original chapter byte, without cross-chapter interpretation.
    pub chapter: u8,
    /// Physical continuation position, never a filesystem offset authority.
    pub resume_position: u32,
    /// CRC-validated opaque attribute bytes, never restored or executed.
    pub extended_bytes: u64,
}
impl Record {
    /// Whether this physical record has an admitted data profile, before name review.
    pub fn payload_supported(&self) -> bool {
        if self.required_version > 11
            || !matches!(self.host, 0 | 2 | 5 | 10 | 11)
            || self.flags & !(0x02 | 0x10 | 0x20) != 0
            || self.extended_flags != 0
            || self.chapter != 0
            || self.resume_position != 0
        {
            return false;
        }
        let unix_kind = if self.host == 2 {
            self.attributes & 0xf000
        } else {
            0
        };
        match self.kind {
            0 | 1 => {
                matches!(self.method, 0..=4)
                    && matches!(unix_kind, 0 | 0x8000)
                    && (self.method != 0 || self.packed_size == self.size)
            }
            3 => {
                self.method == 0
                    && self.size == 0
                    && self.packed_size == 0
                    && self.crc32 == 0
                    && matches!(unix_kind, 0 | 0x4000)
            }
            _ => false,
        }
    }
}
/// Complete physical framing, including every record and exact terminal marker.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    /// Physical member order, retaining duplicates and unsupported records.
    pub records: Vec<Record>,
    /// Frozen physical length, including all metadata and terminal framing.
    pub source_size: u64,
    /// Actual metadata source bytes read, including extended attributes.
    pub input_used: u64,
    /// Complete metadata digest, not payload authentication.
    pub metadata_digest: [u8; 32],
}
struct Input<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    length: u64,
    remaining: u64,
    used: u64,
    hash: SecretSha256,
}
impl Input<'_> {
    fn read(&mut self, offset: u64, count: usize) -> Result<Vec<u8>, ScanError> {
        self.cancel.check()?;
        if offset
            .checked_add(count as u64)
            .is_none_or(|end| end > self.length)
        {
            return Err(ScanError::Invalid);
        }
        if count as u64 > self.remaining {
            return Err(ScanError::Limit);
        }
        if self
            .source
            .seek(SeekFrom::Start(offset))
            .map_err(|e| source_error(&e))?
            != offset
        {
            return Err(ScanError::SourceChanged);
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(count)
            .map_err(|_| ScanError::Limit)?;
        bytes.resize(count, 0);
        let mut at = 0;
        while at < count {
            self.cancel.check()?;
            let end = (at + 8192).min(count);
            let n = match self.source.read(&mut bytes[at..end]) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted && e.get_ref().is_none() => {
                    continue;
                }
                Err(e) => return Err(source_error(&e)),
                Ok(n) => n,
            };
            if n > end - at {
                return Err(ScanError::Io);
            }
            if n == 0 {
                return Err(ScanError::SourceChanged);
            }
            self.remaining -= n as u64;
            self.used += n as u64;
            self.hash.update(&bytes[at..at + n]);
            at += n;
        }
        Ok(bytes)
    }
}
fn word(bytes: &[u8], at: usize) -> Result<u16, ScanError> {
    Ok(u16::from_le_bytes(
        bytes
            .get(at..at + 2)
            .ok_or(ScanError::Invalid)?
            .try_into()
            .map_err(|_| ScanError::Invalid)?,
    ))
}
fn dword(bytes: &[u8], at: usize) -> Result<u32, ScanError> {
    Ok(u32::from_le_bytes(
        bytes
            .get(at..at + 4)
            .ok_or(ScanError::Invalid)?
            .try_into()
            .map_err(|_| ScanError::Invalid)?,
    ))
}
struct Header {
    body: Vec<u8>,
    payload: u64,
    extended_bytes: u64,
}
fn block(input: &mut Input<'_>, offset: u64) -> Result<Option<Header>, ScanError> {
    let preamble = input.read(offset, 4)?;
    if preamble[..2] != [0x60, 0xea] {
        return Err(ScanError::Invalid);
    }
    let size = usize::from(word(&preamble, 2)?);
    if size == 0 {
        if offset + 4 != input.length {
            return Err(ScanError::Invalid);
        }
        return Ok(None);
    }
    if size < 30 {
        return Err(ScanError::Invalid);
    }
    if size > MAX_HEADER {
        return Err(ScanError::Limit);
    }
    let mut body = input.read(offset + 4, size + 4)?;
    if crc32(&body[..size]) != dword(&body, size)? {
        return Err(ScanError::Invalid);
    }
    body.truncate(size);
    if usize::from(body[0]) > size || body[0] < 30 {
        return Err(ScanError::Invalid);
    }
    let mut at = offset + size as u64 + 8;
    let mut total = 0;
    for count in 0..=MAX_EXTENSIONS {
        let size = usize::from(word(&input.read(at, 2)?, 0)?);
        at += 2;
        if size == 0 {
            return Ok(Some(Header {
                body,
                payload: at,
                extended_bytes: total,
            }));
        }
        if count == MAX_EXTENSIONS {
            return Err(ScanError::Limit);
        }
        let bytes = input.read(at, size + 4)?;
        if crc32(&bytes[..size]) != dword(&bytes, size)? {
            return Err(ScanError::Invalid);
        }
        total += size as u64;
        at += size as u64 + 4;
    }
    Err(ScanError::Limit)
}
fn name(body: &[u8]) -> Result<&[u8], ScanError> {
    let fields = body.get(usize::from(body[0])..).ok_or(ScanError::Invalid)?;
    let end = fields
        .iter()
        .position(|b| *b == 0)
        .ok_or(ScanError::Invalid)?;
    let comment = &fields[end + 1..];
    if comment.last() != Some(&0) || comment[..comment.len() - 1].contains(&0) {
        return Err(ScanError::Invalid);
    }
    if usize::from(word(body, 24)?) > end {
        return Err(ScanError::Invalid);
    }
    Ok(&fields[..end])
}
/// Validate complete raw ARJ framing at offset zero, without SFX/volume recovery or partial rows.
pub fn inspect(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    cancel.check()?;
    let length = source
        .seek(SeekFrom::End(0))
        .map_err(|e| source_error(&e))?;
    let mut input = Input {
        source,
        cancel,
        length,
        remaining: limits.read_bytes.min(64 * 1024 * 1024),
        used: 0,
        hash: SecretSha256::new(),
    };
    let main = block(&mut input, 0)?.ok_or(ScanError::Invalid)?;
    let body = &main.body;
    if body[6] != 2 {
        return Err(ScanError::Invalid);
    }
    if !matches!(body[0], 30 | 34) {
        return Err(ScanError::Unsupported);
    }
    name(body)?;
    if body[2] > 11
        || body[4] & !(0x02 | 0x10) != 0
        || (body[1] < 9 && body[4] & 2 != 0)
        || body[28] != 0
        || body[29] != 0
        || dword(body, 20)? != 0
        || word(body, 26)? != 0
        || main.extended_bytes != 0
        || (body[0] >= 34 && body[30..34] != [0; 4])
    {
        return Err(ScanError::Unsupported);
    }
    let archive_size = u64::from(dword(body, 16)?);
    if archive_size != 0 && archive_size != length {
        return Err(ScanError::Invalid);
    }
    let mut offset = main.payload;
    let mut records = Vec::new();
    let mut names = 0usize;
    while let Some(header) = block(&mut input, offset)? {
        cancel.check()?;
        if records.len() >= limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
            return Err(ScanError::Limit);
        }
        let body = &header.body;
        if !matches!(body[0], 30 | 34 | 46) {
            return Err(ScanError::Unsupported);
        }
        if body[4] & 0x80 != 0 {
            return Err(ScanError::Unsupported);
        }
        let raw = name(body)?;
        if raw.len() > limits.name_bytes.min(4096) {
            return Err(ScanError::Limit);
        }
        let mut raw_name = Vec::new();
        raw_name
            .try_reserve_exact(raw.len())
            .map_err(|_| ScanError::Limit)?;
        raw_name.extend_from_slice(raw);
        names = names
            .checked_add(raw_name.capacity())
            .ok_or(ScanError::Limit)?;
        if names > limits.total_name_bytes.min(16 * 1024 * 1024) {
            return Err(ScanError::Limit);
        }
        let packed_size = u64::from(dword(body, 12)?);
        let size = u64::from(dword(body, 16)?);
        let next = header
            .payload
            .checked_add(packed_size)
            .ok_or(ScanError::Invalid)?;
        if next > length {
            return Err(ScanError::Invalid);
        }
        if body[5] == 0 && matches!(body[6], 0 | 1) && body[4] & 1 == 0 && packed_size != size {
            return Err(ScanError::Invalid);
        }
        let timestamp = dword(body, 8)?;
        let modified_dos = if body[3] == 2 {
            input
                .source
                .local_calendar(time::Timestamp::UnixSeconds(u64::from(timestamp)))
                .map_err(host_error)?
                .filter(|c| (1980..=2107).contains(&c.year))
                .and_then(time::Calendar::wrapping_dos)
                .unwrap_or(0)
        } else {
            timestamp
        };
        let resume_position = if body[0] >= 34 { dword(body, 30)? } else { 0 };
        records.try_reserve(1).map_err(|_| ScanError::Limit)?;
        records.push(Record {
            header_offset: offset,
            payload_offset: header.payload,
            packed_size,
            size,
            method: body[5],
            kind: body[6],
            flags: body[4],
            version: body[1],
            required_version: body[2],
            host: body[3],
            first_size: body[0],
            raw_name,
            name_position: word(body, 24)?,
            crc32: dword(body, 20)?,
            modified_dos,
            attributes: word(body, 26)?,
            password_modifier: body[7],
            extended_flags: body[28],
            chapter: body[29],
            resume_position,
            extended_bytes: header.extended_bytes,
        });
        offset = next;
    }
    Ok(Index {
        records,
        source_size: length,
        input_used: input.used,
        metadata_digest: *input.hash.finish(),
    })
}
