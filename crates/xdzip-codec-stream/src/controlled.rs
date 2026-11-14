//! Shared cancellation and byte admission for sequential filters and containers.
use crate::framed::{FramedReader, Kind, lzma_alone};
use crate::xz::XzReader;
use std::io::{self, Read, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};
use xdzip_compression::GzipReader;

const FILTER_MEMORY_LIMIT: u64 = 128 * 1024 * 1024;
fn scan_error(error: &io::Error) -> ScanError {
    error
        .get_ref()
        .and_then(|e| e.downcast_ref::<ScanError>())
        .copied()
        .unwrap_or(match error.kind() {
            io::ErrorKind::OutOfMemory => ScanError::Limit,
            io::ErrorKind::Unsupported => ScanError::Unsupported,
            _ => ScanError::Invalid,
        })
}

/// Outer filter selected by content-aware host routing.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Filter {
    /// Uncompressed bytes.
    Identity,
    /// RFC 1952, including concatenated members.
    Gzip,
    /// Concatenated BZh1..BZh9 members with block and combined CRC validation.
    Bzip2,
    /// One LZMA Alone stream, with a 13-byte header and exact stream ending.
    Lzma,
    /// Concatenated Lzip v1 members, including all trailer fields.
    Lzip,
    /// XZ streams, including declared checks, complete indexes and zero padding.
    Xz,
}

struct Input<'a> {
    source: &'a mut dyn ReadSource,
    control: &'a Cancellation,
    remaining: u64,
    budget: u64,
}
impl Read for Input<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        self.control.check().map_err(io::Error::other)?;
        // The complete seekable input was admitted before construction. At its
        // frozen end no additional byte is read, even for an exact-size budget.
        if self.remaining == 0 {
            return Ok(0);
        }
        if self.budget == 0 {
            return Err(io::Error::other(ScanError::Limit));
        }
        let take = bytes
            .len()
            .min(self.remaining.min(self.budget).min(usize::MAX as u64) as usize);
        let n = self
            .source
            .read(&mut bytes[..take])
            .map_err(|_| io::Error::other(ScanError::Io))?;
        if n == 0 {
            return Err(io::Error::other(ScanError::Invalid));
        }
        self.remaining -= n as u64;
        self.budget -= n as u64;
        Ok(n)
    }
}

/// Bounded decoded stream. Errors retain their stable host classification.
/// One private decoded byte may be inspected at the output boundary to
/// distinguish EOF from overflow; it is never delivered to the caller.
pub struct ControlledStream<'a> {
    reader: Box<dyn Read + 'a>,
    control: &'a Cancellation,
    remaining: u64,
    position: u64,
    failure: Option<ScanError>,
}
impl<'a> ControlledStream<'a> {
    /// Rewind and admit physical length before initializing a filter.
    pub fn new(
        source: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        filter: Filter,
        input_limit: u64,
        output_limit: u64,
    ) -> Result<Self, ScanError> {
        Self::initialize(source, control, filter, input_limit, output_limit, true)
    }
    /// Bounded prefix discovery. A large physical input is allowed, while actual
    /// compressed reads remain capped. Callers must not publish payload metadata.
    pub fn probe(
        source: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        filter: Filter,
        input_limit: u64,
        output_limit: u64,
    ) -> Result<Self, ScanError> {
        Self::initialize(source, control, filter, input_limit, output_limit, false)
    }
    fn initialize(
        source: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        filter: Filter,
        input_limit: u64,
        output_limit: u64,
        admit_length: bool,
    ) -> Result<Self, ScanError> {
        control.check()?;
        let length = source.seek(SeekFrom::End(0)).map_err(|_| ScanError::Io)?;
        if admit_length && length > input_limit {
            return Err(ScanError::Limit);
        }
        source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
        let input = Input {
            source,
            control,
            remaining: length,
            budget: input_limit,
        };
        let reader: Box<dyn Read + 'a> = match filter {
            Filter::Identity => Box::new(input),
            Filter::Gzip => Box::new(GzipReader::new(input)),
            Filter::Xz => Box::new(
                XzReader::new(input, length, FILTER_MEMORY_LIMIT)
                    .map_err(|error| scan_error(&error))?,
            ),
            Filter::Bzip2 => Box::new(FramedReader::new(
                input,
                length,
                Kind::Bzip2,
                FILTER_MEMORY_LIMIT,
            )),
            Filter::Lzma => Box::new(
                lzma_alone(input, length, FILTER_MEMORY_LIMIT).map_err(|e| scan_error(&e))?,
            ),
            Filter::Lzip => Box::new(FramedReader::new(
                input,
                length,
                Kind::Lzip,
                FILTER_MEMORY_LIMIT,
            )),
        };
        Ok(Self {
            reader,
            control,
            remaining: output_limit,
            position: 0,
            failure: None,
        })
    }
    /// Actual bytes delivered, independent of declared container sizes.
    pub fn position(&self) -> u64 {
        self.position
    }
    /// First retained failure; a failed stream cannot be resumed.
    pub fn failure(&self) -> Option<ScanError> {
        self.failure
    }
    /// Control used for nested metadata operations.
    pub fn control(&self) -> &'a Cancellation {
        self.control
    }
    fn read_inner(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.control.check().map_err(io::Error::other)?;
        if self.remaining == 0 {
            return match self.reader.read(&mut [0])? {
                0 => Ok(0),
                _ => Err(io::Error::other(ScanError::Limit)),
            };
        }
        let take = bytes
            .len()
            .min(self.remaining.min(usize::MAX as u64) as usize);
        let n = self.reader.read(&mut bytes[..take])?;
        self.remaining -= n as u64;
        self.position += n as u64;
        Ok(n)
    }
}
impl Read for ControlledStream<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if let Some(error) = self.failure {
            return Err(io::Error::other(error));
        }
        self.read_inner(bytes).inspect_err(|error| {
            self.failure = Some(scan_error(error));
        })
    }
}
