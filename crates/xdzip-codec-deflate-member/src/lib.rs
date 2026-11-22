//! Bounded Store and raw Deflate payloads shared by the internal ALZ and EGG codecs.

use std::io::{self, Read, SeekFrom};
use xdzip_codec_api::{
    Cancellation, ReadSource, ScanError,
    payload::{DecodeError, DecodedSink},
};
use xdzip_compression::{Crc32, DeflateReader};

const INPUT_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
const WORK_LIMIT: u64 = 96 * 1024 * 1024 * 1024;

/// Preserve terminal host read errors without mistaking other I/O failures for a format miss.
pub fn source_error(error: &io::Error) -> ScanError {
    match ScanError::from_source_error(error) {
        stop @ (ScanError::Cancelled
        | ScanError::Limit
        | ScanError::Io
        | ScanError::SourceChanged) => stop,
        _ => ScanError::Io,
    }
}

/// Map a format scan failure to the payload error returned when opening a reader.
pub fn scan_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}

fn io_error(error: io::Error) -> DecodeError {
    if let Some(typed) = error.get_ref().and_then(|e| e.downcast_ref::<ScanError>()) {
        return scan_error(*typed);
    }
    match error.kind() {
        io::ErrorKind::InvalidData | io::ErrorKind::UnexpectedEof => DecodeError::Integrity,
        io::ErrorKind::OutOfMemory => DecodeError::Limit,
        _ => DecodeError::SourceIo,
    }
}

fn check_cancel(opened: &Cancellation, task: &Cancellation) -> Result<(), ScanError> {
    opened.check().and_then(|()| task.check())
}

struct Span<'a> {
    source: &'a mut dyn ReadSource,
    left: u64,
    input: &'a mut u64,
    opened: &'a Cancellation,
    task: &'a Cancellation,
}

impl Read for Span<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        check_cancel(self.opened, self.task).map_err(io::Error::other)?;
        if out.is_empty() || self.left == 0 {
            return Ok(0);
        }
        let size = (out.len().min(8192) as u64).min(self.left).min(*self.input) as usize;
        if size == 0 {
            return Err(io::Error::other(ScanError::Limit));
        }
        let n = loop {
            match self.source.read(&mut out[..size]) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted && e.get_ref().is_none() => {
                    continue;
                }
                Err(e) => break Err(io::Error::other(source_error(&e))),
                Ok(n) if n <= size => break Ok(n),
                Ok(_) => break Err(io::Error::other(ScanError::Io)),
            }
        }?;
        if n == 0 {
            return Err(io::Error::other(ScanError::SourceChanged));
        }
        self.left -= n as u64;
        *self.input -= n as u64;
        Ok(n)
    }
}

/// Payload method selected by the owning container after validating its wire method number.
pub enum Method {
    /// Uncompressed bytes.
    Store,
    /// Raw RFC 1951 Deflate.
    Deflate,
}

/// An admitted physical payload from the owning container's frozen index.
pub struct Member {
    /// Payload offset from the start of the source.
    pub offset: u64,
    /// Exact number of compressed bytes.
    pub packed_size: u64,
    /// Exact number of decoded bytes.
    pub size: u64,
    /// CRC-32 of the decoded bytes.
    pub crc32: u32,
    /// Admitted compression method.
    pub method: Method,
}

/// Reader with cumulative input, work and output budgets for a single frozen container.
pub struct MemberReader<'a> {
    source: &'a mut dyn ReadSource,
    opened: &'a Cancellation,
    input: u64,
    work: u64,
    decoded: u64,
    failure: Option<DecodeError>,
}

impl<'a> MemberReader<'a> {
    /// Reserve input used by the owning container's complete metadata scan.
    pub fn new(
        source: &'a mut dyn ReadSource,
        opened: &'a Cancellation,
        metadata_input: u64,
    ) -> Result<Self, DecodeError> {
        Ok(Self {
            source,
            opened,
            input: INPUT_LIMIT
                .checked_sub(metadata_input)
                .ok_or(DecodeError::Limit)?,
            work: WORK_LIMIT,
            decoded: INPUT_LIMIT,
            failure: None,
        })
    }

    /// Deliver the selected member and keep the first failure for the reader's lifetime.
    pub fn decode(
        &mut self,
        member: Result<Member, DecodeError>,
        sink: &mut dyn DecodedSink,
        task: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.deliver(member, sink, task);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }

    fn deliver(
        &mut self,
        member: Result<Member, DecodeError>,
        sink: &mut dyn DecodedSink,
        task: &Cancellation,
    ) -> Result<(), DecodeError> {
        check_cancel(self.opened, task).map_err(scan_error)?;
        let member = member?;
        if member.packed_size > self.input || member.size > self.decoded {
            return Err(DecodeError::Limit);
        }
        let cost = member
            .packed_size
            .checked_add(member.size)
            .ok_or(DecodeError::Limit)?;
        if cost > self.work {
            return Err(DecodeError::Limit);
        }
        self.decoded -= member.size;
        self.work -= cost;
        sink.charge_auxiliary(cost)?;
        self.source
            .seek(SeekFrom::Start(member.offset))
            .map_err(|e| scan_error(source_error(&e)))?;
        let span = Span {
            source: self.source,
            left: member.packed_size,
            input: &mut self.input,
            opened: self.opened,
            task,
        };
        let (total, checksum) = match member.method {
            Method::Store => drain(span, self.opened, task, member.size, sink)?,
            Method::Deflate => {
                let mut reader = DeflateReader::new(span, member.packed_size);
                let result = drain(&mut reader, self.opened, task, member.size, sink)?;
                if reader.consumed_bytes() != member.packed_size {
                    return Err(DecodeError::Integrity);
                }
                result
            }
        };
        if total != member.size || checksum != member.crc32 {
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
}

fn drain(
    mut reader: impl Read,
    opened: &Cancellation,
    task: &Cancellation,
    size: u64,
    sink: &mut dyn DecodedSink,
) -> Result<(u64, u32), DecodeError> {
    let mut crc = Crc32::new();
    let mut total = 0u64;
    let mut bytes = [0u8; 8192];
    loop {
        check_cancel(opened, task).map_err(scan_error)?;
        let n = reader.read(&mut bytes).map_err(io_error)?;
        if n == 0 {
            break;
        }
        total = total.checked_add(n as u64).ok_or(DecodeError::Limit)?;
        if total > size {
            return Err(DecodeError::Integrity);
        }
        crc.update(&bytes[..n]);
        sink.write_chunk(&bytes[..n])?;
    }
    Ok((total, crc.checksum()))
}

#[cfg(test)]
mod tests;
