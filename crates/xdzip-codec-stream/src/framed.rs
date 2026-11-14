//! Concatenated BZip2 and Lzip framing over the independent member decoders.
use std::io::{self, BufRead, BufReader, Read};
use xdzip_compression::{Bzip2Reader, Crc32, LzmaReader};

const MAX_MEMBERS: usize = 100_000;
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid_compressed_stream")
}
fn limit() -> io::Error {
    io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit")
}

#[derive(Clone, Copy)]
pub(crate) enum Kind {
    Bzip2,
    Lzip,
}
enum Member<R> {
    Bzip2(Box<Bzip2Reader<R>>),
    Lzip(Box<LzmaReader<R>>),
}
impl<R: Read> Member<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Bzip2(reader) => reader.read(output),
            Self::Lzip(reader) => reader.read(output),
        }
    }
    fn consumed(&self) -> u64 {
        match self {
            Self::Bzip2(reader) => reader.consumed_bytes(),
            Self::Lzip(reader) => reader.consumed_bytes(),
        }
    }
    fn decoded(&self) -> u64 {
        match self {
            Self::Bzip2(reader) => reader.decoded_bytes(),
            Self::Lzip(reader) => reader.decoded_bytes(),
        }
    }
    fn into_source(self) -> io::Result<R> {
        match self {
            Self::Bzip2(reader) => reader.into_member_source(),
            Self::Lzip(reader) => reader.into_member_source(),
        }
    }
}

pub(crate) struct FramedReader<R> {
    source: Option<BufReader<R>>,
    member: Option<Member<BufReader<R>>>,
    kind: Kind,
    remaining: u64,
    members: usize,
    memory_limit: u64,
    crc: Crc32,
    failed: bool,
    ended: bool,
}
impl<R: Read> FramedReader<R> {
    pub(crate) fn new(source: R, packed: u64, kind: Kind, memory_limit: u64) -> Self {
        Self {
            source: Some(BufReader::new(source)),
            member: None,
            kind,
            remaining: packed,
            members: 0,
            memory_limit,
            crc: Crc32::new(),
            failed: false,
            ended: false,
        }
    }
    fn start(&mut self) -> io::Result<bool> {
        let source = self.source.as_mut().ok_or_else(invalid)?;
        if source.fill_buf()?.is_empty() {
            if self.members == 0 || self.remaining != 0 {
                return Err(invalid());
            }
            self.ended = true;
            return Ok(false);
        }
        if self.members >= MAX_MEMBERS {
            return Err(limit());
        }
        let mut source = self.source.take().ok_or_else(invalid)?;
        self.member = Some(match self.kind {
            Kind::Bzip2 => Member::Bzip2(Box::new(Bzip2Reader::new_member(
                source,
                self.remaining,
                self.memory_limit,
            )?)),
            Kind::Lzip => {
                let mut header = [0; 6];
                source.read_exact(&mut header)?;
                if header[..4] != *b"LZIP" {
                    return Err(invalid());
                }
                if header[4] != 1 {
                    return Err(io::Error::new(
                        io::ErrorKind::Unsupported,
                        "unsupported_lzip_version",
                    ));
                }
                let power = 1u64 << (header[5] & 31);
                let dictionary = power - (power / 16) * u64::from(header[5] >> 5);
                if !(4096..=512 * 1024 * 1024).contains(&dictionary) {
                    return Err(invalid());
                }
                let mut properties = [0x5d, 0, 0, 0, 0];
                properties[1..].copy_from_slice(&(dictionary as u32).to_le_bytes());
                let packed = self.remaining.checked_sub(6).ok_or_else(invalid)?;
                Member::Lzip(Box::new(LzmaReader::new_member(
                    source,
                    packed,
                    &properties,
                    self.memory_limit,
                )?))
            }
        });
        self.crc = Crc32::new();
        Ok(true)
    }
    fn finish_member(&mut self) -> io::Result<()> {
        let member = self.member.take().ok_or_else(invalid)?;
        let consumed = member.consumed();
        let decoded = member.decoded();
        let mut source = member.into_source()?;
        let size = match self.kind {
            Kind::Bzip2 => consumed,
            Kind::Lzip => {
                let size = consumed.checked_add(26).ok_or_else(invalid)?;
                if size > self.remaining {
                    return Err(invalid());
                }
                let mut trailer = [0; 20];
                source.read_exact(&mut trailer)?;
                if u32::from_le_bytes(trailer[..4].try_into().expect("fixed slice"))
                    != self.crc.checksum()
                    || u64::from_le_bytes(trailer[4..12].try_into().expect("fixed slice"))
                        != decoded
                    || u64::from_le_bytes(trailer[12..].try_into().expect("fixed slice")) != size
                {
                    return Err(invalid());
                }
                size
            }
        };
        self.remaining = self.remaining.checked_sub(size).ok_or_else(invalid)?;
        self.members += 1;
        self.source = Some(source);
        Ok(())
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.member.is_none() && !self.start()? {
                return Ok(0);
            }
            let n = self.member.as_mut().ok_or_else(invalid)?.read(output)?;
            if n != 0 {
                self.crc.update(&output[..n]);
                return Ok(n);
            }
            self.finish_member()?;
        }
    }
}
impl<R: Read> Read for FramedReader<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(0);
        }
        self.read_inner(output).inspect_err(|_| self.failed = true)
    }
}

pub(crate) fn lzma_alone<R: Read>(
    mut source: R,
    packed: u64,
    memory_limit: u64,
) -> io::Result<LzmaReader<R>> {
    let mut header = [0; 13];
    source.read_exact(&mut header)?;
    let size = u64::from_le_bytes(header[5..].try_into().expect("fixed slice"));
    let size = (size != u64::MAX).then_some(size);
    LzmaReader::new(
        source,
        packed.checked_sub(13).ok_or_else(invalid)?,
        size,
        &header[..5],
        memory_limit,
    )
}
