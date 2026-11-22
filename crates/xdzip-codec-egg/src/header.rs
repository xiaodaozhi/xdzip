use std::io::{self, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits};
use xdzip_codec_deflate_member::source_error;
use xdzip_crypto::SecretSha256;

const MAX_INPUT: u64 = 24 * 1024 * 1024 * 1024;
const EGG_MAGIC: [u8; 4] = [0x45, 0x47, 0x47, 0x41];
const END: [u8; 4] = [0x22, 0x82, 0xe2, 0x08];
const FILE: [u8; 4] = [0xe3, 0x90, 0x85, 0x0a];
const FILENAME: [u8; 4] = [0xac, 0x91, 0x85, 0x0a];
const COMMENT: [u8; 4] = [0x72, 0x36, 0xc6, 0x04];
const WINDOWS_INFO: [u8; 4] = [0x0b, 0x95, 0x86, 0x2c];
const POSIX_INFO: [u8; 4] = [0xe5, 0x22, 0xe9, 0x1e];
const ENCRYPT_INFO: [u8; 4] = [0x0f, 0x47, 0xd1, 0x08];
const BLOCK: [u8; 4] = [0x13, 0x0c, 0xb5, 0x02];
const SPLIT: [u8; 4] = [0x62, 0xa2, 0xf5, 0x24];
const SOLID: [u8; 4] = [0x60, 0xa0, 0xe5, 0x24];
const GLOBAL_ENCRYPT: [u8; 4] = [0xa8, 0x44, 0xd1, 0x08];
const SKIP: [u8; 4] = [0x00, 0x00, 0xff, 0xff];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub header_offset: u64,
    pub payload_offset: u64,
    pub packed_size: u64,
    pub size: u64,
    pub method: u8,
    pub raw_name: Vec<u8>,
    pub crc32: u32,
    pub directory: bool,
    pub encrypted: bool,
    pub solid: bool,
    pub split: bool,
}
impl Record {
    pub fn payload_supported(&self) -> bool {
        !self.directory
            && !self.encrypted
            && !self.solid
            && !self.split
            && matches!(self.method, 0 | 1)
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
    u16::from_le_bytes(bytes.try_into().expect("fixed EGG field"))
}
fn dword(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("fixed EGG field"))
}
fn qword(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().expect("fixed EGG field"))
}

fn extra(input: &mut Input<'_>, at: u64) -> Result<(u8, u32, u64), ScanError> {
    let mut prefix = [0u8; 5];
    input.read(at, &mut prefix[..3])?;
    let flags = prefix[0];
    let (prefix_len, size) = if flags & 1 == 0 {
        (3, u64::from(word(&prefix[1..3])))
    } else {
        input.read(at + 3, &mut prefix[3..5])?;
        (5, u64::from(dword(&prefix[1..5])))
    };
    Ok((
        flags,
        u32::try_from(size).map_err(|_| ScanError::Limit)?,
        prefix_len,
    ))
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
    if !(14..=MAX_INPUT).contains(&size) {
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
    let mut header = [0u8; 14];
    input.read(0, &mut header)?;
    if header[..4] != EGG_MAGIC || header[10..14].iter().any(|b| *b != 0) {
        return Err(ScanError::Invalid);
    }
    let mut at = 14u64;
    let mut split = false;
    let mut solid = false;
    let mut encrypted = false;
    let mut marker = [0u8; 4];
    loop {
        input.read(at, &mut marker)?;
        if marker == END {
            at += 4;
            break;
        }
        let (flags, length, prefix_len) = match marker {
            SPLIT | SOLID | GLOBAL_ENCRYPT | SKIP => extra(&mut input, at + 4)?,
            _ => return Err(ScanError::Invalid),
        };
        if flags != 0 || length > limits.read_bytes.min(64 * 1024 * 1024) as u32 {
            return Err(ScanError::Invalid);
        }
        if marker == SPLIT {
            if length != 8 {
                return Err(ScanError::Invalid);
            }
            split = true;
        }
        if marker == SOLID {
            if length != 0 {
                return Err(ScanError::Invalid);
            }
            solid = true;
        }
        if marker == GLOBAL_ENCRYPT {
            encrypted = true;
        }
        at = at
            .checked_add(4 + prefix_len + u64::from(length))
            .ok_or(ScanError::Invalid)?;
    }
    let mut records = Vec::new();
    let mut names = 0usize;
    loop {
        input.read(at, &mut marker)?;
        if marker == END {
            at += 4;
            if at != size {
                return Err(ScanError::Invalid);
            }
            break;
        }
        if marker != FILE {
            return Err(ScanError::Invalid);
        }
        let header_at = at;
        let mut fh = [0u8; 12];
        input.read(at + 4, &mut fh)?;
        let decoded_size = qword(&fh[4..12]);
        at += 16;
        let mut raw_name = None;
        let mut entry_encrypted = encrypted;
        let mut directory = false;
        loop {
            input.read(at, &mut marker)?;
            if marker == END {
                at += 4;
                break;
            }
            if !matches!(
                marker,
                FILENAME | COMMENT | WINDOWS_INFO | POSIX_INFO | ENCRYPT_INFO
            ) {
                return Err(ScanError::Invalid);
            }
            let (flags, length, prefix_len) = extra(&mut input, at + 4)?;
            let payload = at + 4 + prefix_len;
            if payload
                .checked_add(u64::from(length))
                .is_none_or(|end| end > size)
            {
                return Err(ScanError::Invalid);
            }
            if marker == FILENAME {
                if raw_name.is_some() || length == 0 {
                    return Err(ScanError::Invalid);
                }
                let mut body = vec![0u8; length as usize];
                input.read(payload, &mut body)?;
                let mut cursor = 0usize;
                if flags & 8 != 0 {
                    if body.len() < 2 {
                        return Err(ScanError::Invalid);
                    }
                    cursor += 2;
                }
                if flags & 16 != 0 {
                    if body.len() < cursor + 4 {
                        return Err(ScanError::Invalid);
                    }
                    cursor += 4;
                }
                if flags & 4 != 0 {
                    entry_encrypted = true;
                }
                if flags & 1 != 0 {
                    return Err(ScanError::Invalid);
                }
                if cursor == body.len() {
                    return Err(ScanError::Invalid);
                }
                raw_name = Some(body[cursor..].to_vec());
            } else if marker == WINDOWS_INFO && length >= 4 {
                let mut attrs = [0u8; 4];
                input.read(payload, &mut attrs)?;
                directory = dword(&attrs) & 0x10 != 0;
            } else if marker == ENCRYPT_INFO {
                entry_encrypted = true;
            }
            at = payload + u64::from(length);
        }
        let raw_name = raw_name.ok_or(ScanError::Invalid)?;
        if raw_name.len() > limits.name_bytes.min(4096) {
            return Err(ScanError::Limit);
        }
        names = names.checked_add(raw_name.len()).ok_or(ScanError::Limit)?;
        if names > limits.total_name_bytes.min(16 * 1024 * 1024) {
            return Err(ScanError::Limit);
        }
        let mut blocks = 0usize;
        let mut method = 0u8;
        let mut packed_size = 0u64;
        let mut size_decoded = 0u64;
        let mut crc32 = 0u32;
        let mut payload_offset = at;
        loop {
            input.read(at, &mut marker)?;
            if marker != BLOCK {
                break;
            }
            let mut bh = [0u8; 14];
            input.read(at + 4, &mut bh)?;
            let block_method = bh[0];
            let unpack = u64::from(dword(&bh[2..6]));
            let pack = u64::from(dword(&bh[6..10]));
            let block_crc = dword(&bh[10..14]);
            let mut end = [0u8; 4];
            input.read(at + 18, &mut end)?;
            if end != END {
                return Err(ScanError::Invalid);
            }
            let data = at + 22;
            if data.checked_add(pack).is_none_or(|end| end > size) {
                return Err(ScanError::Invalid);
            }
            blocks += 1;
            if blocks == 1 {
                method = block_method;
                size_decoded = unpack;
                packed_size = pack;
                crc32 = block_crc;
                payload_offset = data;
            } else {
                return Err(ScanError::Unsupported);
            }
            at = data + pack;
        }
        if blocks == 0 {
            directory = true;
            size_decoded = 0;
            packed_size = 0;
            payload_offset = at;
        }
        if decoded_size != size_decoded {
            return Err(ScanError::Invalid);
        }
        if records.len() >= limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
            return Err(ScanError::Limit);
        }
        records.push(Record {
            header_offset: header_at,
            payload_offset,
            packed_size,
            size: size_decoded,
            method,
            raw_name,
            crc32,
            directory,
            encrypted: entry_encrypted,
            solid,
            split,
        });
    }
    if records.is_empty() {
        return Err(ScanError::Invalid);
    }
    Ok(Index {
        records,
        source_size: size,
        input_used: input.used,
        metadata_digest: *input.hash.finish(),
    })
}
