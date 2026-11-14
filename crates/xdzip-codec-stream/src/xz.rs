//! XZ 1.2.1 framing over independent LZMA2 and size-preserving byte filters.
//! Every index record is compared with observed block work before stream success.
mod writer;
use std::io::{self, BufRead, BufReader, Read, Take};
pub use writer::XzWriter;
use xdzip_compression::{ByteFilter, ByteFilterKind, Crc32, Lzma2Reader, crc32};
use xdzip_crypto::SecretSha256;

const MAGIC: &[u8; 6] = b"\xfd7zXZ\0";
const MAX_RECORDS: usize = 100_000;
const CHUNK: usize = 32 * 1024;
// Covers bounded index/record storage and all parsing/filter buffers together.
const FIXED_MEMORY: u64 = 8 * 1024 * 1024;
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid_xz_stream")
}
fn unsupported() -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, "unsupported_xz_profile")
}
fn limit() -> io::Error {
    io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit")
}
fn terminal(error: io::Error) -> io::Error {
    if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_some() {
        io::Error::other(error.into_inner().expect("payload checked"))
    } else {
        error
    }
}
fn checked_crc(bytes: &[u8]) -> io::Result<()> {
    let at = bytes.len().checked_sub(4).ok_or_else(invalid)?;
    if crc32(&bytes[..at]).to_le_bytes() != bytes[at..] {
        return Err(invalid());
    }
    Ok(())
}
fn vli(input: &mut &[u8]) -> io::Result<u64> {
    let mut value = 0u64;
    for index in 0..9 {
        let (&byte, rest) = input.split_first().ok_or_else(invalid)?;
        *input = rest;
        if index != 0 && byte == 0 {
            return Err(invalid());
        }
        value |= u64::from(byte & 127) << (index * 7);
        if byte & 128 == 0 {
            return Ok(value);
        }
    }
    Err(invalid())
}
fn append_vli(output: &mut Vec<u8>, mut value: u64) -> io::Result<()> {
    if value > i64::MAX as u64 {
        return Err(invalid());
    }
    while value >= 128 {
        output.push((value as u8) | 128);
        value >>= 7;
    }
    output.push(value as u8);
    Ok(())
}
fn index_bytes(records: &[(u64, u64)]) -> io::Result<Vec<u8>> {
    if records.len() > MAX_RECORDS {
        return Err(limit());
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(16 + records.len() * 18)
        .map_err(|_| limit())?;
    bytes.push(0);
    append_vli(&mut bytes, records.len() as u64)?;
    for &(packed, decoded) in records {
        append_vli(&mut bytes, packed)?;
        append_vli(&mut bytes, decoded)?;
    }
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    bytes.extend_from_slice(&crc32(&bytes).to_le_bytes());
    Ok(bytes)
}

// Reflected ECMA-182 polynomial, initial/final inversion per XZ section 6.
const fn crc64_table() -> [u64; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut value = i as u64;
        let mut bit = 0;
        while bit < 8 {
            value = (value >> 1)
                ^ if value & 1 != 0 {
                    0xc96c_5795_d787_0f42
                } else {
                    0
                };
            bit += 1;
        }
        table[i] = value;
        i += 1;
    }
    table
}
const CRC64_TABLE: [u64; 256] = crc64_table();
enum Check {
    None,
    Crc32(Crc32),
    Crc64(u64),
    Sha256(SecretSha256),
}
impl Check {
    fn new(id: u8) -> io::Result<Self> {
        match id {
            0 => Ok(Self::None),
            1 => Ok(Self::Crc32(Crc32::new())),
            4 => Ok(Self::Crc64(!0)),
            10 => Ok(Self::Sha256(SecretSha256::new())),
            _ => Err(unsupported()),
        }
    }
    fn update(&mut self, bytes: &[u8]) {
        match self {
            Self::None => {}
            Self::Crc32(crc) => crc.update(bytes),
            Self::Sha256(sha) => sha.update(bytes),
            Self::Crc64(crc) => {
                for &byte in bytes {
                    *crc = (*crc >> 8) ^ CRC64_TABLE[((*crc ^ u64::from(byte)) & 255) as usize];
                }
            }
        }
    }
    fn finish(self) -> Vec<u8> {
        match self {
            Self::None => Vec::new(),
            Self::Crc32(crc) => crc.checksum().to_le_bytes().to_vec(),
            Self::Crc64(crc) => (!crc).to_le_bytes().to_vec(),
            Self::Sha256(sha) => sha.finish().to_vec(),
        }
    }
}
struct Source<R> {
    input: BufReader<Take<R>>,
    position: u64,
}
impl<R: Read> Read for Source<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let n = self.input.read(bytes).map_err(terminal)?;
        self.position = self.position.checked_add(n as u64).ok_or_else(invalid)?;
        Ok(n)
    }
}
impl<R: Read> Source<R> {
    fn byte(&mut self) -> io::Result<u8> {
        let mut byte = [0];
        self.read_exact(&mut byte)?;
        Ok(byte[0])
    }
    fn peek(&mut self) -> io::Result<Option<u8>> {
        Ok(self.input.fill_buf()?.first().copied())
    }
}
struct Stage {
    filter: ByteFilter,
    pending: Vec<u8>,
}
fn byte_filter(id: u64, properties: &[u8]) -> io::Result<ByteFilter> {
    let kind = match id {
        3 if properties.len() == 1 => ByteFilterKind::Delta(usize::from(properties[0]) + 1),
        4 => ByteFilterKind::X86,
        5 => ByteFilterKind::Ppc,
        6 => ByteFilterKind::Ia64,
        7 => ByteFilterKind::Arm,
        8 => ByteFilterKind::Thumb,
        9 => ByteFilterKind::Sparc,
        10 => ByteFilterKind::Arm64,
        11 => ByteFilterKind::Riscv,
        _ => return Err(unsupported()),
    };
    let position = if id == 3 || properties.is_empty() {
        0
    } else if properties.len() == 4 {
        u32::from_le_bytes(properties.try_into().expect("fixed properties"))
    } else {
        return Err(invalid());
    };
    ByteFilter::new(kind, position)
}
fn buffer() -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    bytes.try_reserve_exact(CHUNK + 64).map_err(|_| limit())?;
    Ok(bytes)
}
struct Block<R> {
    raw: Lzma2Reader<Source<R>>,
    stages: Vec<Stage>,
    output: Vec<u8>,
    offset: usize,
    eof: bool,
    header_size: u64,
    packed_assertion: Option<u64>,
    check: Check,
    decoded: u64,
}
impl<R: Read> Block<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        while self.offset == self.output.len() {
            if self.eof {
                return Ok(0);
            }
            self.output.resize(CHUNK, 0);
            let n = self.raw.read(&mut self.output)?;
            self.eof = n == 0;
            self.output.truncate(n);
            for stage in &mut self.stages {
                stage.pending.extend_from_slice(&self.output);
                let used = stage
                    .filter
                    .transform(&mut stage.pending, self.eof, &mut || Ok(()))?;
                self.output.clear();
                self.output.extend_from_slice(&stage.pending[..used]);
                stage.pending.drain(..used);
            }
            self.decoded = self
                .decoded
                .checked_add(self.output.len() as u64)
                .ok_or_else(invalid)?;
            if matches!(self.check, Check::Sha256(_)) && self.decoded > u64::MAX / 8 {
                return Err(invalid());
            }
            self.check.update(&self.output);
            self.offset = 0;
        }
        let n = output.len().min(self.output.len() - self.offset);
        output[..n].copy_from_slice(&self.output[self.offset..self.offset + n]);
        self.offset += n;
        Ok(n)
    }
}

/// Sequential, poisoned-on-error XZ reader; checks every declared check and index.
/// `packed_limit` is the complete physical range. All allocations are admitted.
pub struct XzReader<R> {
    source: Option<Source<R>>,
    block: Option<Box<Block<R>>>,
    records: Vec<(u64, u64)>,
    packed_limit: u64,
    memory_limit: u64,
    streams: usize,
    check_id: u8,
    in_stream: bool,
    ended: bool,
    failed: bool,
    decoded: u64,
    checksum_present: bool,
}
impl<R: Read> XzReader<R> {
    /// Initialize without decoding or allocating a dictionary.
    pub fn new(source: R, packed_limit: u64, memory_limit: u64) -> io::Result<Self> {
        if memory_limit < FIXED_MEMORY {
            return Err(limit());
        }
        if packed_limit > i64::MAX as u64 {
            return Err(invalid());
        }
        Ok(Self {
            source: Some(Source {
                input: BufReader::new(source.take(packed_limit)),
                position: 0,
            }),
            block: None,
            records: Vec::new(),
            packed_limit,
            memory_limit,
            streams: 0,
            check_id: 0,
            in_stream: false,
            ended: false,
            failed: false,
            decoded: 0,
            checksum_present: true,
        })
    }
    /// True only after validated EOF when every stream declares a content check.
    pub fn has_content_check(&self) -> bool {
        self.ended && !self.failed && self.checksum_present
    }
    fn start_stream(&mut self) -> io::Result<()> {
        if self.streams >= MAX_RECORDS {
            return Err(limit());
        }
        let source = self.source.as_mut().ok_or_else(invalid)?;
        let mut header = [0; 12];
        source.read_exact(&mut header)?;
        if &header[..6] != MAGIC {
            return Err(invalid());
        }
        checked_crc(&header[6..])?;
        if header[6] != 0 || header[7] & 0xf0 != 0 {
            return Err(unsupported());
        }
        Check::new(header[7])?;
        self.check_id = header[7];
        self.checksum_present &= self.check_id != 0;
        self.streams += 1;
        self.in_stream = true;
        self.records.clear();
        Ok(())
    }
    fn start_block(&mut self, first: u8) -> io::Result<()> {
        if self.records.len() >= MAX_RECORDS {
            return Err(limit());
        }
        let mut source = self.source.take().ok_or_else(invalid)?;
        let length = (usize::from(first) + 1) * 4;
        let mut header = [0; 1024];
        header[0] = first;
        source.read_exact(&mut header[1..length])?;
        checked_crc(&header[..length])?;
        let flags = header[1];
        if flags & 0x3c != 0 {
            return Err(unsupported());
        }
        let mut fields = &header[2..length - 4];
        let packed = if flags & 64 != 0 {
            Some(vli(&mut fields)?)
        } else {
            None
        };
        if packed == Some(0) {
            return Err(invalid());
        }
        let decoded = if flags & 128 != 0 {
            Some(vli(&mut fields)?)
        } else {
            None
        };
        let count = usize::from(flags & 3) + 1;
        let mut stages = Vec::with_capacity(count - 1);
        let mut property = None;
        for index in 0..count {
            let id = vli(&mut fields)?;
            let size = usize::try_from(vli(&mut fields)?).map_err(|_| invalid())?;
            if size > fields.len() {
                return Err(invalid());
            }
            let (properties, rest) = fields.split_at(size);
            fields = rest;
            if index == count - 1 {
                if id != 0x21 || properties.len() != 1 {
                    return Err(unsupported());
                }
                property = Some(properties[0]);
            } else {
                stages.push(Stage {
                    filter: byte_filter(id, properties)?,
                    pending: buffer()?,
                });
            }
        }
        if fields.iter().any(|byte| *byte != 0) {
            return Err(invalid());
        }
        stages.reverse();
        let remaining = self
            .packed_limit
            .checked_sub(source.position)
            .ok_or_else(invalid)?;
        let raw = Lzma2Reader::new_member(
            source,
            remaining,
            decoded,
            property.ok_or_else(invalid)?,
            self.memory_limit - FIXED_MEMORY,
        )?;
        self.block = Some(Box::new(Block {
            raw,
            stages,
            output: buffer()?,
            offset: 0,
            eof: false,
            header_size: length as u64,
            packed_assertion: packed,
            check: Check::new(self.check_id)?,
            decoded: 0,
        }));
        Ok(())
    }
    fn finish_block(&mut self) -> io::Result<()> {
        let block = self.block.take().ok_or_else(invalid)?;
        let packed = block.raw.consumed_bytes();
        let decoded = block.raw.decoded_bytes();
        if block.packed_assertion.is_some_and(|size| size != packed) || block.decoded != decoded {
            return Err(invalid());
        }
        let mut source = block.raw.into_member_source()?;
        for _ in 0..(4 - packed % 4) % 4 {
            if source.byte()? != 0 {
                return Err(invalid());
            }
        }
        let check = block.check.finish();
        let mut stored = [0; 32];
        source.read_exact(&mut stored[..check.len()])?;
        if stored[..check.len()] != check {
            return Err(invalid());
        }
        let unpadded = block
            .header_size
            .checked_add(packed)
            .and_then(|size| size.checked_add(check.len() as u64))
            .ok_or_else(invalid)?;
        if unpadded > i64::MAX as u64 {
            return Err(invalid());
        }
        self.decoded = self
            .decoded
            .checked_add(decoded)
            .filter(|size| *size <= i64::MAX as u64)
            .ok_or_else(invalid)?;
        self.records.try_reserve(1).map_err(|_| limit())?;
        self.records.push((unpadded, decoded));
        self.source = Some(source);
        Ok(())
    }
    fn finish_stream(&mut self) -> io::Result<()> {
        let source = self.source.as_mut().ok_or_else(invalid)?;
        let index = index_bytes(&self.records)?;
        // Indicator was consumed while deciding whether another block follows.
        for &expected in &index[1..] {
            if source.byte()? != expected {
                return Err(invalid());
            }
        }
        let mut footer = [0; 12];
        source.read_exact(&mut footer)?;
        if crc32(&footer[4..10]).to_le_bytes() != footer[..4]
            || u32::from_le_bytes(footer[4..8].try_into().expect("fixed footer")) as usize
                != index.len() / 4 - 1
            || footer[8..10] != [0, self.check_id]
            || footer[10..] != *b"YZ"
        {
            return Err(invalid());
        }
        self.in_stream = false;
        let mut padding = 0u8;
        while source.peek()? == Some(0) {
            source.byte()?;
            padding = (padding + 1) % 4;
        }
        if padding != 0 {
            return Err(invalid());
        }
        if source.peek()?.is_none() {
            if source.position != self.packed_limit {
                return Err(invalid());
            }
            self.ended = true;
        }
        Ok(())
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.ended {
                return Ok(0);
            }
            if let Some(block) = &mut self.block {
                let n = block.read(output)?;
                if n != 0 {
                    return Ok(n);
                }
                self.finish_block()?;
            } else {
                if !self.in_stream {
                    self.start_stream()?;
                }
                let first = self.source.as_mut().ok_or_else(invalid)?.byte()?;
                if first == 0 {
                    self.finish_stream()?;
                } else {
                    self.start_block(first)?;
                }
            }
        }
    }
}
impl<R: Read> Read for XzReader<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        self.read_inner(output).inspect_err(|_| self.failed = true)
    }
}

#[cfg(test)]
mod tests;
