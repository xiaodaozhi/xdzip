use crate::{Index, Record, header::source_error, inspect};
use std::io::{self, Read, SeekFrom};
use xdzip_codec_api::{
    Cancellation, ReadSource, ScanError, ScanLimits,
    payload::{DecodeError, DecodedSink},
};
use xdzip_compression::{ArjReader, Crc32};
const INPUT_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
const WORK_LIMIT: u64 = 96 * 1024 * 1024 * 1024;
#[cfg(test)]
#[path = "content_tests.rs"]
mod tests;
fn scan_error(error: ScanError) -> DecodeError {
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
        io::ErrorKind::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::SourceIo,
    }
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
        loop {
            self.opened
                .check()
                .and_then(|()| self.task.check())
                .map_err(io::Error::other)?;
            if out.is_empty() || self.left == 0 {
                return Ok(0);
            }
            let size = out.len().min(8192).min(self.left.min(*self.input) as usize);
            if size == 0 {
                return Err(io::Error::other(ScanError::Limit));
            }
            let n = match self.source.read(&mut out[..size]) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted && e.get_ref().is_none() => {
                    continue;
                }
                Err(e) => return Err(io::Error::other(source_error(&e))),
                Ok(n) => n,
            };
            if n > size {
                return Err(io::Error::other(ScanError::Io));
            }
            if n == 0 {
                return Err(io::Error::other(ScanError::SourceChanged));
            }
            self.left -= n as u64;
            *self.input -= n as u64;
            return Ok(n);
        }
    }
}
trait Algorithm: Read {
    fn work_left(&self) -> u64;
}
impl<R: Read, P: FnMut() -> io::Result<()>> Algorithm for ArjReader<R, P> {
    fn work_left(&self) -> u64 {
        self.work_remaining()
    }
}
struct Stored<R> {
    source: R,
    work: u64,
    left: u64,
}
impl<R: Read> Read for Stored<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() || self.left == 0 {
            return Ok(0);
        }
        let size = out.len().min(self.left.min(self.work).min(8192) as usize);
        if size == 0 {
            return Err(io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit"));
        }
        let n = self.source.read(&mut out[..size])?;
        if n > size {
            return Err(io::Error::other(ScanError::Io));
        }
        self.work -= n as u64;
        self.left -= n as u64;
        Ok(n)
    }
}
impl<R: Read> Algorithm for Stored<R> {
    fn work_left(&self) -> u64 {
        self.work
    }
}
fn drain(
    reader: &mut dyn Algorithm,
    record: &Record,
    work: &mut u64,
    decoded: &mut u64,
    sink: &mut dyn DecodedSink,
    cancel: &Cancellation,
) -> Result<(), DecodeError> {
    let mut bytes = [0; 8192];
    let mut crc = Crc32::new();
    let mut total = 0u64;
    loop {
        cancel.check().map_err(scan_error)?;
        let before = reader.work_left();
        let result = reader.read(&mut bytes);
        let after = reader.work_left();
        *work = after;
        sink.charge_auxiliary(before.checked_sub(after).ok_or(DecodeError::Limit)?)?;
        let n = result.map_err(io_error)?;
        if n == 0 {
            break;
        }
        total = total.checked_add(n as u64).ok_or(DecodeError::Limit)?;
        *decoded = decoded.checked_sub(n as u64).ok_or(DecodeError::Limit)?;
        if total > record.size {
            return Err(DecodeError::Integrity);
        }
        crc.update(&bytes[..n]);
        sink.write_chunk(&bytes[..n])?;
    }
    if total != record.size || crc.checksum() != record.crc32 {
        return Err(DecodeError::Integrity);
    }
    Ok(())
}
/// One frozen container instance, with cumulative budgets and terminal failure state.
pub struct Reader<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    index: Index,
    input: u64,
    work: u64,
    decoded: u64,
    failure: Option<DecodeError>,
}
impl<'a> Reader<'a> {
    /// Rescan complete metadata and bind it to the caller's exact snapshot before output.
    pub fn open(
        source: &'a mut dyn ReadSource,
        snapshot: &Index,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        let result = Self::fresh(source, cancel)?;
        if &result.index != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(result)
    }
    pub(crate) fn fresh(
        source: &'a mut dyn ReadSource,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        let index = inspect(source, cancel, ScanLimits::default()).map_err(scan_error)?;
        let input = INPUT_LIMIT
            .checked_sub(index.input_used)
            .ok_or(DecodeError::Limit)?;
        Ok(Self {
            source,
            cancel,
            index,
            input,
            work: WORK_LIMIT,
            decoded: INPUT_LIMIT,
            failure: None,
        })
    }
    /// Validated immutable physical metadata, without filesystem authority.
    pub fn index(&self) -> &Index {
        &self.index
    }
    /// Decode a physical member completely and validate its required CRC before success.
    pub fn decode(
        &mut self,
        at: usize,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.deliver(at, sink, cancel);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
    fn deliver(
        &mut self,
        at: usize,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.cancel
            .check()
            .and_then(|()| cancel.check())
            .map_err(scan_error)?;
        let record = self.index.records.get(at).ok_or(DecodeError::Integrity)?;
        if !record.payload_supported() {
            return Err(DecodeError::Unsupported);
        }
        if record.kind == 3 {
            return Ok(());
        }
        if record.packed_size > self.input || record.size > self.decoded {
            return Err(DecodeError::Limit);
        }
        // Reserve the exact successful CRC bit-step cost before lending the remaining
        // work to a stateful decoder; later model updates cannot refund this reserve.
        let crc_work = record.size.checked_mul(8).ok_or(DecodeError::Limit)?;
        self.work = self.work.checked_sub(crc_work).ok_or(DecodeError::Limit)?;
        sink.charge_auxiliary(crc_work)?;
        if self
            .source
            .seek(SeekFrom::Start(record.payload_offset))
            .map_err(|e| scan_error(source_error(&e)))?
            != record.payload_offset
        {
            return Err(DecodeError::SourceIo);
        }
        let source = Span {
            source: self.source,
            left: record.packed_size,
            input: &mut self.input,
            opened: self.cancel,
            task: cancel,
        };
        let opened = self.cancel;
        let poll = move || {
            opened
                .check()
                .and_then(|()| cancel.check())
                .map_err(io::Error::other)
        };
        let initial = self.work;
        let mut reader: Box<dyn Algorithm + '_> = if record.method == 0 {
            Box::new(Stored {
                source,
                work: self.work,
                left: record.size,
            })
        } else {
            Box::new(
                ArjReader::with_poll(
                    source,
                    record.method,
                    record.packed_size,
                    record.size,
                    256 * 1024,
                    self.work,
                    poll,
                )
                .map_err(io_error)?,
            )
        };
        self.work = reader.work_left();
        sink.charge_auxiliary(initial.checked_sub(self.work).ok_or(DecodeError::Limit)?)?;
        drain(
            &mut *reader,
            record,
            &mut self.work,
            &mut self.decoded,
            sink,
            cancel,
        )
    }
}
