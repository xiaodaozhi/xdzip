use std::io::{self, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits};
use xdzip_crypto::SecretSha256;
const MAX_INPUT: u64 = 24 * 1024 * 1024 * 1024;
pub(crate) fn source_error(error: &io::Error) -> ScanError {
    match ScanError::from_source_error(error) {
        stop @ (ScanError::Cancelled
        | ScanError::Limit
        | ScanError::Io
        | ScanError::SourceChanged) => stop,
        _ => ScanError::Io,
    }
}
/// One physical file, independent of names and duplicate records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// Beginning of its complete old or ordinary header.
    pub header_offset: u64,
    /// Beginning of the exact stored/compressed span.
    pub payload_offset: u64,
    /// Declared packed bytes, without the next member marker.
    pub packed_size: u64,
    /// Declared decoded bytes; old Store inherits packed size.
    pub size: u64,
    /// Original method byte, without Spark flag stripping or ZIP aliases.
    pub method: u8,
    /// Original complete name before the required NUL, without path authority.
    pub raw_name: Vec<u8>,
    /// Required reflected CRC-16/IBM, initial zero.
    pub crc16: u16,
    /// DOS local display time, without host calendar conversion.
    pub modified_dos: u32,
}
impl Record {
    /// Admit method-specific data profiles independently of name review.
    pub fn payload_supported(&self) -> bool {
        matches!(self.method, 1..=9) && (self.method > 2 || self.packed_size == self.size)
    }
}
/// Complete frozen physical framing, not payload authentication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    /// Complete physical order, including duplicates and unknown methods.
    pub records: Vec<Record>,
    /// Physical length including the exact terminal marker.
    pub source_size: u64,
    /// Every metadata byte actually read, excluding skipped payload spans.
    pub input_used: u64,
    /// All header and terminal bytes, including otherwise ignored name padding.
    pub metadata_digest: [u8; 32],
}
struct Input<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    size: u64,
    left: u64,
    used: u64,
    hash: SecretSha256,
}
impl Input<'_> {
    fn read(&mut self, at: u64, out: &mut [u8]) -> Result<(), ScanError> {
        self.cancel.check()?;
        if at
            .checked_add(out.len() as u64)
            .is_none_or(|end| end > self.size)
        {
            return Err(ScanError::Invalid);
        }
        if out.len() as u64 > self.left {
            return Err(ScanError::Limit);
        }
        if self
            .source
            .seek(SeekFrom::Start(at))
            .map_err(|e| source_error(&e))?
            != at
        {
            return Err(ScanError::SourceChanged);
        }
        let mut used = 0;
        while used < out.len() {
            self.cancel.check()?;
            let n = match self.source.read(&mut out[used..]) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted && e.get_ref().is_none() => {
                    continue;
                }
                Err(e) => return Err(source_error(&e)),
                Ok(n) if n <= out.len() - used => n,
                Ok(_) => return Err(ScanError::Io),
            };
            if n == 0 {
                return Err(ScanError::SourceChanged);
            }
            self.used += n as u64;
            self.left -= n as u64;
            used += n;
        }
        self.hash.update(out);
        Ok(())
    }
}
fn word(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes[..2].try_into().expect("fixed admitted header"))
}
fn dword(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes[..4].try_into().expect("fixed admitted header"))
}
/// Validate every header/span and the exact terminator before publishing any records.
pub fn inspect(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    cancel.check()?;
    let size = source
        .seek(SeekFrom::End(0))
        .map_err(|e| source_error(&e))?;
    if size > MAX_INPUT {
        return Err(ScanError::Limit);
    }
    let mut input = Input {
        source,
        cancel,
        size,
        left: limits.read_bytes.min(64 * 1024 * 1024),
        used: 0,
        hash: SecretSha256::new(),
    };
    let mut records = Vec::new();
    let mut at = 0u64;
    let mut names = 0usize;
    loop {
        let mut marker = [0; 2];
        input.read(at, &mut marker)?;
        if marker[0] != 0x1a {
            return Err(ScanError::Invalid);
        }
        let method = marker[1];
        if method == 0 {
            if at + 2 != size {
                return Err(ScanError::Invalid);
            }
            return Ok(Index {
                records,
                source_size: size,
                input_used: input.used,
                metadata_digest: *input.hash.finish(),
            });
        }
        if records.len() >= limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
            return Err(ScanError::Limit);
        }
        let width = if method == 1 { 25 } else { 29 };
        let mut bytes = [0; 27];
        input.read(at + 2, &mut bytes[..width - 2])?;
        let length = bytes[..13]
            .iter()
            .position(|b| *b == 0)
            .ok_or(ScanError::Invalid)?;
        if length == 0 {
            return Err(ScanError::Invalid);
        }
        let mut raw_name = Vec::new();
        raw_name
            .try_reserve_exact(length)
            .map_err(|_| ScanError::Limit)?;
        raw_name.extend_from_slice(&bytes[..length]);
        names = names
            .checked_add(raw_name.capacity())
            .ok_or(ScanError::Limit)?;
        if length > limits.name_bytes.min(4096)
            || names > limits.total_name_bytes.min(16 * 1024 * 1024)
        {
            return Err(ScanError::Limit);
        }
        let packed_size = u64::from(dword(&bytes[13..]));
        let decoded = if method == 1 {
            packed_size
        } else {
            u64::from(dword(&bytes[23..]))
        };
        if method <= 2 && packed_size != decoded {
            return Err(ScanError::Invalid);
        }
        let payload_offset = at.checked_add(width as u64).ok_or(ScanError::Invalid)?;
        let end = payload_offset
            .checked_add(packed_size)
            .ok_or(ScanError::Invalid)?;
        if end.checked_add(2).is_none_or(|needed| needed > size) {
            return Err(ScanError::Invalid);
        }
        records.try_reserve(1).map_err(|_| ScanError::Limit)?;
        records.push(Record {
            header_offset: at,
            payload_offset,
            packed_size,
            size: decoded,
            method,
            raw_name,
            crc16: word(&bytes[21..]),
            modified_dos: (u32::from(word(&bytes[17..])) << 16) | u32::from(word(&bytes[19..])),
        });
        at = end;
    }
}
