use crate::{Index, header::source_error, inspect};
use std::io::{self, Read, SeekFrom};
use xdzip_codec_api::{
    Cancellation, ReadSource, ScanError, ScanLimits,
    payload::{DecodeError, DecodedSink},
};
use xdzip_compression::{Crc32, DeflateReader};

const INPUT_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
const WORK_LIMIT: u64 = 96 * 1024 * 1024 * 1024;

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

/// One frozen ALZ container instance with cumulative input, work and output budgets.
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
    /// Return the frozen physical index.
    pub fn index(&self) -> &Index {
        &self.index
    }
    /// Decode one physical member and validate its size, termination and CRC-32.
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
        if record.packed_size > self.input || record.size > self.decoded {
            return Err(DecodeError::Limit);
        }
        let cost = record
            .packed_size
            .checked_add(record.size)
            .ok_or(DecodeError::Limit)?;
        if cost > self.work {
            return Err(DecodeError::Limit);
        }
        self.decoded -= record.size;
        self.work -= cost;
        sink.charge_auxiliary(cost)?;
        self.source
            .seek(SeekFrom::Start(record.payload_offset))
            .map_err(|e| scan_error(source_error(&e)))?;
        let mut source = Span {
            source: self.source,
            left: record.packed_size,
            input: &mut self.input,
            opened: self.cancel,
            task: cancel,
        };
        let mut crc = Crc32::new();
        let mut total = 0u64;
        let mut bytes = [0u8; 8192];
        if record.method == 0 {
            loop {
                self.cancel
                    .check()
                    .and_then(|()| cancel.check())
                    .map_err(scan_error)?;
                let n = source.read(&mut bytes).map_err(io_error)?;
                if n == 0 {
                    break;
                }
                total = total.checked_add(n as u64).ok_or(DecodeError::Limit)?;
                if total > record.size {
                    return Err(DecodeError::Integrity);
                }
                crc.update(&bytes[..n]);
                sink.write_chunk(&bytes[..n])?;
            }
        } else {
            let mut reader = DeflateReader::new(source, record.packed_size);
            loop {
                self.cancel
                    .check()
                    .and_then(|()| cancel.check())
                    .map_err(scan_error)?;
                let n = reader.read(&mut bytes).map_err(io_error)?;
                if n == 0 {
                    break;
                }
                total = total.checked_add(n as u64).ok_or(DecodeError::Limit)?;
                if total > record.size {
                    return Err(DecodeError::Integrity);
                }
                crc.update(&bytes[..n]);
                sink.write_chunk(&bytes[..n])?;
            }
            if reader.consumed_bytes() != record.packed_size {
                return Err(DecodeError::Integrity);
            }
        }
        if total != record.size || crc.checksum() != record.crc32 {
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
}
