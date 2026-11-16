use std::io::{self, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits};
use xdzip_crypto::SecretSha256;

const MAX_INPUT: u64 = 24 * 1024 * 1024 * 1024;
const ALZ_HEADER: [u8; 4] = [0x41, 0x4c, 0x5a, 0x01];
const LOCAL_HEADER: [u8; 4] = [0x42, 0x4c, 0x5a, 0x01];
const END_HEADER: [u8; 4] = [0x43, 0x4c, 0x5a, 0x02];

pub(crate) fn source_error(error: &io::Error) -> ScanError {
    match ScanError::from_source_error(error) {
        stop @ (ScanError::Cancelled
        | ScanError::Limit
        | ScanError::Io
        | ScanError::SourceChanged) => stop,
        _ => ScanError::Io,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub header_offset: u64,
    pub payload_offset: u64,
    pub packed_size: u64,
    pub size: u64,
    pub method: u8,
    pub raw_name: Vec<u8>,
    pub crc32: u32,
    pub modified_dos: u32,
    pub directory: bool,
    pub encrypted: bool,
}
impl Record {
    pub fn payload_supported(&self) -> bool {
        !self.directory && !self.encrypted && matches!(self.method, 0 | 2)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    pub records: Vec<Record>,
    pub source_size: u64,
    pub input_used: u64,
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
    u16::from_le_bytes(bytes.try_into().expect("fixed ALZ field"))
}
fn dword(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("fixed ALZ field"))
}
fn qword(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().expect("fixed ALZ field"))
}

fn read_sized(bytes: &[u8], width: usize) -> u64 {
    match width {
        1 => u64::from(bytes[0]),
        2 => u64::from(u16::from_le_bytes([bytes[0], bytes[1]])),
        4 => u64::from(dword(bytes)),
        8 => qword(bytes),
        _ => unreachable!(),
    }
}

pub fn inspect(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    cancel.check()?;
    let size = source
        .seek(SeekFrom::End(0))
        .map_err(|e| source_error(&e))?;
    if !(8..=MAX_INPUT).contains(&size) {
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
    let mut fixed = [0u8; 8];
    input.read(0, &mut fixed)?;
    if fixed[..4] != ALZ_HEADER {
        return Err(ScanError::Invalid);
    }
    let tail_start = size.checked_sub(16).ok_or(ScanError::Invalid)?;
    if tail_start < 12 {
        return Err(ScanError::Invalid);
    }
    let mut tail = [0u8; 16];
    input.read(tail_start, &mut tail)?;
    let comment_size = u64::from(dword(&tail[4..8]));
    if comment_size > tail_start.saturating_sub(8) {
        return Err(ScanError::Invalid);
    }
    let mut records = Vec::new();
    let mut at = 8u64;
    let mut names = 0usize;
    while at < tail_start {
        let mut sig = [0u8; 4];
        input.read(at, &mut sig)?;
        if sig == END_HEADER {
            if at + 4 != tail_start {
                return Err(ScanError::Invalid);
            }
            break;
        }
        if sig != LOCAL_HEADER {
            return Err(ScanError::Invalid);
        }
        let mut fixed = [0u8; 9];
        input.read(at + 4, &mut fixed)?;
        let name_len = usize::from(word(&fixed[..2]));
        let attribute = fixed[2];
        let modified_dos = dword(&fixed[3..7]);
        let descriptor = fixed[7];
        let width = match descriptor & 0xf0 {
            0 => 0,
            0x10 => 1,
            0x20 => 2,
            0x40 => 4,
            0x80 => 8,
            _ => return Err(ScanError::Invalid),
        };
        let encrypted = descriptor & 1 != 0;
        let has_descriptor = descriptor & 8 != 0;
        if has_descriptor || name_len == 0 || name_len > limits.name_bytes.min(4096) {
            return Err(if name_len == 0 {
                ScanError::Invalid
            } else {
                ScanError::Limit
            });
        }
        let variable_len = if width == 0 {
            0
        } else {
            6usize.checked_add(width * 2).ok_or(ScanError::Limit)?
        };
        let variable_at = at + 13;
        let mut variable = vec![0u8; variable_len];
        if variable_len != 0 {
            input.read(variable_at, &mut variable)?;
        }
        let (method, crc32, packed_size, decoded_size) = if width == 0 {
            (0, 0, 0, 0)
        } else {
            (
                variable[0],
                dword(&variable[2..6]),
                read_sized(&variable[6..6 + width], width),
                read_sized(&variable[6 + width..6 + width * 2], width),
            )
        };
        let name_at = variable_at + variable_len as u64;
        let mut raw_name = vec![0u8; name_len];
        input.read(name_at, &mut raw_name)?;
        names = names.checked_add(name_len).ok_or(ScanError::Limit)?;
        if names > limits.total_name_bytes.min(16 * 1024 * 1024) {
            return Err(ScanError::Limit);
        }
        let payload_offset = name_at
            .checked_add(name_len as u64)
            .ok_or(ScanError::Invalid)?;
        let end = payload_offset
            .checked_add(packed_size)
            .ok_or(ScanError::Invalid)?;
        if end > tail_start || records.len() >= limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
            return Err(if end > tail_start {
                ScanError::Invalid
            } else {
                ScanError::Limit
            });
        }
        records.push(Record {
            header_offset: at,
            payload_offset,
            packed_size,
            size: decoded_size,
            method,
            raw_name,
            crc32,
            modified_dos,
            directory: attribute & 0x10 != 0 || width == 0,
            encrypted,
        });
        at = end;
    }
    if records.is_empty() || at + 4 != tail_start {
        return Err(ScanError::Invalid);
    }
    Ok(Index {
        records,
        source_size: size,
        input_used: input.used,
        metadata_digest: *input.hash.finish(),
    })
}
