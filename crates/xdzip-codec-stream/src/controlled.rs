//! Shared cancellation and byte admission for sequential filters and containers.
use crate::framed::{FramedReader, Kind, lzma_alone};
use crate::lz4::Lz4Reader;
use crate::xz::XzReader;
use crate::zstd::ZstdReader;
use std::io::{self, Read, SeekFrom};
use std::{cell::Cell, rc::Rc};
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
    /// Standard LZ4 frames, with declared checks, linking and skippable frames.
    Lz4,
    /// Zstandard standard frames with bounded history, entropy and declared checks.
    Zstd,
}

struct Input<'a> {
    source: Box<dyn Read + 'a>,
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
            .map_err(|error| io::Error::other(ScanError::from_source_error(&error)))?;
        if n > take {
            return Err(io::Error::other(ScanError::Io));
        }
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
    output_budget: Option<Rc<Cell<u64>>>,
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
    /// Own a bounded source capability without borrowing a local adapter or granting paths.
    pub fn new_owned(
        mut source: Box<dyn ReadSource + 'a>,
        control: &'a Cancellation,
        filter: Filter,
        input_limit: u64,
        output_limit: u64,
    ) -> Result<Self, ScanError> {
        let length = Self::admit(source.as_mut(), control, input_limit, true)?;
        Self::from_input(
            Input {
                source,
                control,
                remaining: length,
                budget: input_limit,
            },
            control,
            filter,
            length,
            output_limit,
        )
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
        let length = Self::admit(source, control, input_limit, admit_length)?;
        let input = Input {
            source: Box::new(source),
            control,
            remaining: length,
            budget: input_limit,
        };
        Self::from_input(input, control, filter, length, output_limit)
    }
    fn admit(
        source: &mut dyn ReadSource,
        control: &Cancellation,
        input_limit: u64,
        admit_length: bool,
    ) -> Result<u64, ScanError> {
        control.check()?;
        let length = source
            .seek(SeekFrom::End(0))
            .map_err(|error| ScanError::from_source_error(&error))?;
        if admit_length && length > input_limit {
            return Err(ScanError::Limit);
        }
        let start = source
            .seek(SeekFrom::Start(0))
            .map_err(|error| ScanError::from_source_error(&error))?;
        if start != 0 {
            return Err(ScanError::SourceChanged);
        }
        Ok(length)
    }
    fn from_input(
        input: Input<'a>,
        control: &'a Cancellation,
        filter: Filter,
        length: u64,
        output_limit: u64,
    ) -> Result<Self, ScanError> {
        let reader: Box<dyn Read + 'a> = match filter {
            Filter::Identity => Box::new(input),
            Filter::Gzip => Box::new(GzipReader::new(input)),
            Filter::Xz => Box::new(
                XzReader::new(input, length, FILTER_MEMORY_LIMIT)
                    .map_err(|error| scan_error(&error))?,
            ),
            Filter::Lz4 => Box::new(
                Lz4Reader::new(input, length, FILTER_MEMORY_LIMIT)
                    .map_err(|error| scan_error(&error))?,
            ),
            Filter::Zstd => Box::new(
                ZstdReader::new(input, length, FILTER_MEMORY_LIMIT)
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
            output_budget: None,
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
    /// Attach an additional shared decoded-byte budget without relaxing this stream's limit.
    pub fn with_output_budget(mut self, budget: Rc<Cell<u64>>) -> Self {
        self.output_budget = Some(budget);
        self
    }
    fn read_inner(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.control.check().map_err(io::Error::other)?;
        let admitted = self.remaining.min(
            self.output_budget
                .as_ref()
                .map_or(u64::MAX, |budget| budget.get()),
        );
        if admitted == 0 {
            return match self.reader.read(&mut [0])? {
                0 => Ok(0),
                _ => Err(io::Error::other(ScanError::Limit)),
            };
        }
        let take = bytes.len().min(admitted.min(usize::MAX as u64) as usize);
        let n = self.reader.read(&mut bytes[..take])?;
        if let Some(budget) = &self.output_budget {
            budget.set(
                budget
                    .get()
                    .checked_sub(n as u64)
                    .ok_or_else(|| io::Error::other(ScanError::Limit))?,
            );
        }
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
