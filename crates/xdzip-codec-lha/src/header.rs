use std::io::SeekFrom;
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits, time};

const MAX_HEADER: usize = 1024 * 1024;
const MAX_EXTENSIONS: usize = 256;

/// Validated physical member framing; content still requires full decoding and CRC.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// Method marker, retained even when its algorithm is unsupported.
    pub method: [u8; 5],
    /// Physical header byte position, independent of the member name.
    pub header_offset: u64,
    /// Start of the exact packed range.
    pub payload_offset: u64,
    /// Declared bytes in this physical member's packed range.
    pub packed_size: u64,
    /// Declared decoded size, not a verified content size.
    pub size: u64,
    /// Header level, distinct from the compression method.
    pub level: u8,
    /// Raw composed name; legacy encoding must precede separator interpretation.
    pub raw_name: Vec<u8>,
    /// Length of the original dirname extension inside `raw_name`; its 0xff
    /// component separators are decoded independently of the following filename.
    pub dirname_bytes: usize,
    /// Optional declared content CRC-16; never a cryptographic authenticator.
    pub crc16: Option<u16>,
    /// Directory declaration, with no output creation authority.
    pub directory: bool,
    /// Original Unix mode, never applied to filesystem output.
    pub unix_mode: u16,
    /// DOS-shaped local timestamp; zero denotes unavailable projection.
    pub modified_dos: u32,
}
/// Complete sequential index, without payload-derived trust or filename identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    /// Physical member order, retaining duplicate names.
    pub records: Vec<Record>,
    /// Actual source bytes read while validating metadata.
    pub input_used: u64,
    /// Frozen physical source length.
    pub source_size: u64,
}
struct Input<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    length: u64,
    remaining: u64,
    used: u64,
}
impl Input<'_> {
    fn read(&mut self, offset: u64, count: usize) -> Result<Vec<u8>, ScanError> {
        self.cancel.check()?;
        let end = offset.checked_add(count as u64).ok_or(ScanError::Invalid)?;
        if end > self.length {
            return Err(ScanError::Invalid);
        }
        if count as u64 > self.remaining {
            return Err(ScanError::Limit);
        }
        if self
            .source
            .seek(SeekFrom::Start(offset))
            .map_err(|e| ScanError::from_source_error(&e))?
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
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted && e.get_ref().is_none() => {
                    continue;
                }
                Err(e) => return Err(ScanError::from_source_error(&e)),
                Ok(n) => n,
            };
            if n == 0 {
                return Err(ScanError::SourceChanged);
            }
            self.remaining -= n as u64;
            self.used += n as u64;
            at += n;
        }
        Ok(bytes)
    }
}
fn word(bytes: &[u8], at: usize) -> Result<u16, ScanError> {
    let bytes = bytes.get(at..at + 2).ok_or(ScanError::Invalid)?;
    Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
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
fn grow(target: &mut Vec<u8>, bytes: &[u8], maximum: usize) -> Result<(), ScanError> {
    if target
        .len()
        .checked_add(bytes.len())
        .ok_or(ScanError::Limit)?
        > maximum
    {
        return Err(ScanError::Limit);
    }
    target
        .try_reserve(bytes.len())
        .map_err(|_| ScanError::Limit)?;
    target.extend_from_slice(bytes);
    Ok(())
}
fn dos(source: &dyn ReadSource, seconds: u32) -> Result<u32, ScanError> {
    Ok(source
        .local_calendar(time::Timestamp::UnixSeconds(u64::from(seconds)))?
        .filter(|c| (1980..=2107).contains(&c.year))
        .and_then(time::Calendar::wrapping_dos)
        .unwrap_or(0))
}
struct Extensions {
    filename: Option<Vec<u8>>,
    dirname: Option<Vec<u8>>,
    crc: Option<(usize, u16)>,
    mode: Option<u16>,
    mtime: Option<u32>,
    seen: [bool; 256],
}
impl Extensions {
    fn new() -> Self {
        Self {
            filename: None,
            dirname: None,
            crc: None,
            mode: None,
            mtime: None,
            seen: [false; 256],
        }
    }
    fn add(
        &mut self,
        header: &[u8],
        start: usize,
        size: usize,
        width: usize,
        name_limit: usize,
    ) -> Result<(), ScanError> {
        if size < width + 1 {
            return Err(ScanError::Invalid);
        }
        let section = header.get(start..start + size).ok_or(ScanError::Invalid)?;
        let kind = section[0];
        if self.seen[usize::from(kind)] {
            return Err(ScanError::Invalid);
        }
        self.seen[usize::from(kind)] = true;
        let data = &section[1..size - width];
        match kind {
            0x00 => {
                if !(2..=3).contains(&data.len()) {
                    return Err(ScanError::Invalid);
                }
                self.crc = Some((start + 1, word(data, 0)?));
            }
            0x01 | 0x02 => {
                if data.len() > name_limit {
                    return Err(ScanError::Limit);
                }
                if kind == 0x01 {
                    self.filename = Some(data.to_vec());
                } else {
                    if data.is_empty() || data.last() != Some(&255) {
                        return Err(ScanError::Invalid);
                    }
                    self.dirname = Some(data.to_vec());
                }
            }
            0x3f | 0x52 | 0x53 => {}
            0x50 => {
                if data.len() != 2 {
                    return Err(ScanError::Invalid);
                }
                self.mode = Some(word(data, 0)?);
            }
            0x51 => {
                if data.len() != 4 {
                    return Err(ScanError::Invalid);
                }
            }
            0x54 => {
                if data.len() != 4 {
                    return Err(ScanError::Invalid);
                }
                self.mtime = Some(dword(data, 0)?);
            }
            _ => return Err(ScanError::Unsupported),
        }
        Ok(())
    }
}
fn member(
    input: &mut Input<'_>,
    offset: u64,
    first: u8,
    limits: ScanLimits,
) -> Result<Record, ScanError> {
    let mut header = vec![first];
    grow(&mut header, &input.read(offset + 1, 20)?, MAX_HEADER)?;
    let level = header[20];
    let base_size = match level {
        0 | 1 => usize::from(first) + 2,
        2 => usize::from(word(&header, 0)?),
        3 => {
            if word(&header, 0)? != 4 {
                return Err(ScanError::Unsupported);
            }
            grow(&mut header, &input.read(offset + 21, 11)?, MAX_HEADER)?;
            usize::try_from(dword(&header, 24)?).map_err(|_| ScanError::Limit)?
        }
        _ => return Err(ScanError::Unsupported),
    };
    if base_size
        < match level {
            2 => 26,
            3 => 32,
            _ => 22,
        }
    {
        return Err(ScanError::Invalid);
    }
    if base_size > MAX_HEADER {
        return Err(ScanError::Limit);
    }
    let already = header.len();
    grow(
        &mut header,
        &input.read(offset + already as u64, base_size - already)?,
        MAX_HEADER,
    )?;
    if level < 2 && header[2..].iter().fold(0u8, |sum, b| sum.wrapping_add(*b)) != header[1] {
        return Err(ScanError::Invalid);
    }
    let method: [u8; 5] = header[2..7].try_into().map_err(|_| ScanError::Invalid)?;
    if method[0] != b'-' || method[4] != b'-' || !method[1..4].iter().all(u8::is_ascii_alphanumeric)
    {
        return Err(ScanError::Invalid);
    }
    let mut packed_size = u64::from(dword(&header, 7)?);
    let size = u64::from(dword(&header, 11)?);
    let mut modified_dos = if level >= 2 {
        dos(input.source, dword(&header, 15)?)?
    } else {
        dword(&header, 15)?
    };
    let mut unix_mode = 0;
    let (base_name, crc16) = if level < 2 {
        let length = usize::from(header[21]);
        if length > limits.name_bytes {
            return Err(ScanError::Limit);
        }
        let end = 22 + length;
        let name = header.get(22..end).ok_or(ScanError::Invalid)?.to_vec();
        if level == 0 {
            let extra = header.get(end..).ok_or(ScanError::Invalid)?;
            let crc = if extra.is_empty() {
                None
            } else {
                Some(word(extra, 0)?)
            };
            if extra.len() > 2 {
                return Err(ScanError::Unsupported);
            }
            (name, crc)
        } else {
            if base_size != end + 5 {
                return Err(ScanError::Unsupported);
            }
            (name, Some(word(&header, end)?))
        }
    } else {
        (Vec::new(), Some(word(&header, 21)?))
    };
    let mut extensions = Extensions::new();
    if level != 0 {
        let width = if level == 3 { 4 } else { 2 };
        let mut next_at = match level {
            1 => base_size - 2,
            2 => 24,
            _ => 28,
        };
        let mut at = match level {
            1 => base_size,
            2 => 26,
            _ => 32,
        };
        let mut count = 0;
        loop {
            let amount = if width == 2 {
                usize::from(word(&header, next_at)?)
            } else {
                usize::try_from(dword(&header, next_at)?).map_err(|_| ScanError::Limit)?
            };
            if amount == 0 {
                break;
            }
            count += 1;
            if count > MAX_EXTENSIONS {
                return Err(ScanError::Limit);
            }
            if amount < width + 1 {
                return Err(ScanError::Invalid);
            }
            if level == 1 {
                let bytes = input.read(
                    offset.checked_add(at as u64).ok_or(ScanError::Invalid)?,
                    amount,
                )?;
                grow(&mut header, &bytes, MAX_HEADER)?;
                packed_size = packed_size
                    .checked_sub(amount as u64)
                    .ok_or(ScanError::Invalid)?;
            }
            let end = at.checked_add(amount).ok_or(ScanError::Invalid)?;
            extensions.add(&header, at, amount, width, limits.name_bytes)?;
            next_at = end - width;
            at = end;
        }
        let boundary_padding =
            level == 2 && at % 256 == 0 && header.len() == at + 1 && header[at] == 0;
        if at != header.len() && !boundary_padding {
            return Err(ScanError::Invalid);
        }
    }
    if let Some((at, expected)) = extensions.crc {
        header[at..at + 2].fill(0);
        if xdzip_compression::crc16(&header) != expected {
            return Err(ScanError::Invalid);
        }
    }
    if let Some(mode) = extensions.mode {
        unix_mode = mode;
    }
    if let Some(seconds) = extensions.mtime {
        modified_dos = dos(input.source, seconds)?;
    }
    let mut raw_name = Vec::new();
    let dirname_bytes = extensions.dirname.as_ref().map_or(0, Vec::len);
    if let Some(dirname) = extensions.dirname {
        grow(&mut raw_name, &dirname, limits.name_bytes)?;
    }
    grow(
        &mut raw_name,
        extensions.filename.as_ref().unwrap_or(&base_name),
        limits.name_bytes,
    )?;
    let directory = method == *b"-lhd-";
    if raw_name.is_empty() {
        return Err(ScanError::Invalid);
    }
    if directory {
        if packed_size != 0 || size != 0 || crc16.is_some_and(|c| c != 0) {
            return Err(ScanError::Invalid);
        }
        if !raw_name.ends_with(b"\\")
            && !raw_name.ends_with(b"/")
            && !(dirname_bytes != 0 && dirname_bytes == raw_name.len())
        {
            grow(&mut raw_name, b"\\", limits.name_bytes)?;
        }
    }
    if matches!(&method, b"-lh0-" | b"-lz4-" | b"-pm0-") && size != packed_size {
        return Err(ScanError::Invalid);
    }
    let payload_offset = offset
        .checked_add(header.len() as u64)
        .ok_or(ScanError::Invalid)?;
    if payload_offset
        .checked_add(packed_size)
        .is_none_or(|end| end > input.length)
    {
        return Err(ScanError::Invalid);
    }
    Ok(Record {
        method,
        header_offset: offset,
        payload_offset,
        packed_size,
        size,
        level,
        raw_name,
        dirname_bytes,
        crc16,
        directory,
        unix_mode,
        modified_dos,
    })
}
/// Scan the complete level-0/1/2/3 container, with no partial index on a later failure.
pub fn inspect(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    cancel.check()?;
    let limits = ScanLimits {
        entries: limits.entries.min(xdzip_codec_api::MAX_ENTRIES),
        name_bytes: limits.name_bytes.min(4096),
        total_name_bytes: limits.total_name_bytes.min(16 * 1024 * 1024),
        display_name_bytes: limits.display_name_bytes.min(16 * 1024 * 1024),
        read_bytes: limits.read_bytes.min(64 * 1024 * 1024),
    };
    let length = source
        .seek(SeekFrom::End(0))
        .map_err(|e| ScanError::from_source_error(&e))?;
    if length == 0 {
        return Err(ScanError::Invalid);
    }
    let mut input = Input {
        source,
        cancel,
        length,
        remaining: limits.read_bytes.min(64 * 1024 * 1024),
        used: 0,
    };
    let mut records = Vec::new();
    let mut offset = 0;
    let mut name_bytes = 0usize;
    loop {
        let first = input.read(offset, 1)?[0];
        if first == 0 {
            if offset + 1 != length {
                return Err(ScanError::Invalid);
            }
            break;
        }
        if records.len() >= limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
            return Err(ScanError::Limit);
        }
        let record = member(&mut input, offset, first, limits)?;
        name_bytes = name_bytes
            .checked_add(record.raw_name.len())
            .ok_or(ScanError::Limit)?;
        if name_bytes > limits.total_name_bytes {
            return Err(ScanError::Limit);
        }
        offset = record.payload_offset + record.packed_size;
        records.try_reserve(1).map_err(|_| ScanError::Limit)?;
        records.push(record);
    }
    Ok(Index {
        records,
        input_used: input.used,
        source_size: length,
    })
}
