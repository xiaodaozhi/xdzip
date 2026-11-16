use std::io::{self, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits};
use xdzip_crypto::SecretSha256;

const MAX_INPUT: u64 = 24 * 1024 * 1024 * 1024;
const FIXED_HEADER: usize = 37;

pub(crate) fn source_error(error: &io::Error) -> ScanError {
    match ScanError::from_source_error(error) {
        stop @ (ScanError::Cancelled
        | ScanError::Limit
        | ScanError::Io
        | ScanError::SourceChanged) => stop,
        _ => ScanError::Io,
    }
}

/// One physical BlakHole member, independent of names and duplicate records.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// Beginning of the complete member header.
    pub header_offset: u64,
    /// Beginning of the exact raw Deflate payload.
    pub payload_offset: u64,
    /// Declared compressed bytes.
    pub packed_size: u64,
    /// Declared decoded bytes.
    pub size: u64,
    /// Original BH method byte. The current product profile admits method 8.
    pub method: u8,
    /// Original filename bytes.
    pub raw_name: Vec<u8>,
    /// Required IEEE CRC-32 of decoded bytes.
    pub crc32: u32,
    /// DOS timestamp as stored by the format.
    pub modified_dos: u32,
}
impl Record {
    /// Admit only the independently confirmed ordinary raw Deflate profile.
    pub fn payload_supported(&self) -> bool {
        self.method == 8
    }
}

/// Complete frozen physical framing, not payload authentication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    /// Complete physical order, including duplicates and unknown methods.
    pub records: Vec<Record>,
    /// Physical source length.
    pub source_size: u64,
    /// Header and name bytes actually read, excluding payload spans.
    pub input_used: u64,
    /// Hash of all metadata bytes read during inspection.
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
        let mut used = 0usize;
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
            self.used = self.used.checked_add(n as u64).ok_or(ScanError::Limit)?;
            self.left = self.left.checked_sub(n as u64).ok_or(ScanError::Limit)?;
            used += n;
        }
        self.hash.update(out);
        Ok(())
    }
}

fn word(bytes: &[u8]) -> u16 {
    u16::from_le_bytes(bytes.try_into().expect("fixed BH field"))
}
fn dword(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("fixed BH field"))
}

/// Validate every member header and payload span before publishing any records.
pub fn inspect(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    cancel.check()?;
    let size = source
        .seek(SeekFrom::End(0))
        .map_err(|e| source_error(&e))?;
    if size == 0 || size > MAX_INPUT {
        return Err(ScanError::Invalid);
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
    while at < size {
        let mut fixed = [0u8; FIXED_HEADER];
        input.read(at, &mut fixed)?;
        if &fixed[..2] != b"BH" || fixed[2..4] != [5, 7] {
            return Err(ScanError::Invalid);
        }
        let header_len = usize::from(word(&fixed[4..6]));
        if fixed[6..10] != [0x25, 0xd8, 0x0c, 0x00]
            || fixed[27] != 0x20
            || fixed[28..33].iter().any(|b| *b != 0)
        {
            return Err(ScanError::Invalid);
        }
        let method = fixed[10];
        let packed_size = u64::from(dword(&fixed[15..19]));
        let size_decoded = u64::from(dword(&fixed[19..23]));
        let crc32 = dword(&fixed[23..27]);
        let name_len = usize::try_from(dword(&fixed[33..37])).map_err(|_| ScanError::Limit)?;
        if header_len != FIXED_HEADER.checked_add(name_len).ok_or(ScanError::Limit)?
            || header_len < FIXED_HEADER
            || header_len as u64 > size.saturating_sub(at)
        {
            return Err(ScanError::Invalid);
        }
        if records.len() >= limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
            return Err(ScanError::Limit);
        }
        if name_len == 0 || name_len > limits.name_bytes.min(4096) {
            return Err(if name_len == 0 {
                ScanError::Invalid
            } else {
                ScanError::Limit
            });
        }
        let mut raw_name = Vec::new();
        raw_name
            .try_reserve_exact(name_len)
            .map_err(|_| ScanError::Limit)?;
        raw_name.resize(name_len, 0);
        input.read(at + FIXED_HEADER as u64, &mut raw_name)?;
        names = names
            .checked_add(raw_name.capacity())
            .ok_or(ScanError::Limit)?;
        if names > limits.total_name_bytes.min(16 * 1024 * 1024) {
            return Err(ScanError::Limit);
        }
        let payload_offset = at
            .checked_add(header_len as u64)
            .ok_or(ScanError::Invalid)?;
        let end = payload_offset
            .checked_add(packed_size)
            .ok_or(ScanError::Invalid)?;
        if end > size {
            return Err(ScanError::Invalid);
        }
        records.try_reserve(1).map_err(|_| ScanError::Limit)?;
        records.push(Record {
            header_offset: at,
            payload_offset,
            packed_size,
            size: size_decoded,
            method,
            raw_name,
            crc32,
            modified_dos: dword(&fixed[11..15]),
        });
        at = end;
    }
    if records.is_empty() || at != size {
        return Err(ScanError::Invalid);
    }
    Ok(Index {
        records,
        source_size: size,
        input_used: input.used,
        metadata_digest: *input.hash.finish(),
    })
}
