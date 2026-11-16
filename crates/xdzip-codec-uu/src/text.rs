use crate::line::{self, Encoding};
use std::io::SeekFrom;
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits};
pub(crate) const WORK_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
const LINE_LIMIT: usize = 8192;
/// An independently charged buffered cursor; its logical position excludes read-ahead.
pub(crate) struct Input<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    buffer: [u8; 8192],
    cursor: usize,
    filled: usize,
    pub position: u64,
    end: u64,
    budget: u64,
    pub work: u64,
    /// Distinguish a local weak-probe line bound from a typed host resource failure.
    pub line_too_long: bool,
}
impl<'a> Input<'a> {
    pub fn new(
        source: &'a mut dyn ReadSource,
        cancel: &'a Cancellation,
        start: u64,
        end: u64,
        budget: u64,
    ) -> Result<Self, ScanError> {
        cancel.check()?;
        if start > end
            || source
                .seek(SeekFrom::Start(start))
                .map_err(|e| ScanError::from_source_error(&e))?
                != start
        {
            return Err(ScanError::SourceChanged);
        }
        Ok(Self {
            source,
            cancel,
            buffer: [0; 8192],
            cursor: 0,
            filled: 0,
            position: start,
            end,
            budget,
            work: 0,
            line_too_long: false,
        })
    }
    pub fn line(&mut self) -> Result<Option<Vec<u8>>, ScanError> {
        self.cancel.check()?;
        self.line_too_long = false;
        let mut line = Vec::new();
        loop {
            if self.cursor == self.filled {
                if self.position == self.end {
                    return Ok((!line.is_empty()).then_some(line));
                }
                let count = (self.end - self.position)
                    .min(self.buffer.len() as u64)
                    .min(self.budget) as usize;
                if count == 0 {
                    return Err(ScanError::Limit);
                }
                self.cancel.check()?;
                self.filled = self
                    .source
                    .read(&mut self.buffer[..count])
                    .map_err(|e| ScanError::from_source_error(&e))?;
                if self.filled == 0 {
                    return Err(ScanError::SourceChanged);
                }
                self.cursor = 0;
                self.budget -= self.filled as u64;
                self.work += self.filled as u64;
            }
            let byte = self.buffer[self.cursor];
            self.cursor += 1;
            self.position += 1;
            if byte == b'\n' {
                if line.last() == Some(&b'\r') {
                    line.pop();
                }
                return Ok(Some(line));
            }
            if line.len() == LINE_LIMIT {
                self.line_too_long = true;
                return Err(ScanError::Limit);
            }
            line.push(byte);
        }
    }
}
pub(crate) struct Header {
    pub name: Vec<u8>,
    pub base64: bool,
}
fn is_header(line: &[u8]) -> bool {
    line.starts_with(b"begin ") || line.starts_with(b"begin-")
}
fn header(line: &[u8], limits: ScanLimits) -> Result<Option<Header>, ScanError> {
    let (rest, base64) = if let Some(rest) = line.strip_prefix(b"begin ") {
        (rest, false)
    } else if let Some(rest) = line.strip_prefix(b"begin-base64 ") {
        (rest, true)
    } else if is_header(line) {
        return Err(ScanError::Unsupported);
    } else {
        return Ok(None);
    };
    let split = rest
        .iter()
        .position(|byte| *byte == b' ')
        .ok_or(ScanError::Invalid)?;
    if !(1..=4).contains(&split) || !rest[..split].iter().all(|byte| matches!(byte, b'0'..=b'7')) {
        return Err(ScanError::Invalid);
    }
    let name = &rest[split + 1..];
    if name.is_empty() {
        return Err(ScanError::Invalid);
    }
    if name.len() > limits.name_bytes {
        return Err(ScanError::Limit);
    }
    Ok(Some(Header {
        name: name.to_vec(),
        base64,
    }))
}
/// Bounded text candidate discovery. A match still requires complete envelope validation.
/// Existing strong containers and TAR headers must take precedence in the host.
pub fn candidate(source: &mut dyn ReadSource, cancel: &Cancellation) -> Result<bool, ScanError> {
    let result = probe(source, cancel);
    if source
        .seek(SeekFrom::Start(0))
        .map_err(|e| ScanError::from_source_error(&e))?
        != 0
    {
        return Err(ScanError::SourceChanged);
    }
    result
}
fn probe(source: &mut dyn ReadSource, cancel: &Cancellation) -> Result<bool, ScanError> {
    let length = source
        .seek(SeekFrom::End(0))
        .map_err(|e| ScanError::from_source_error(&e))?;
    let mut input = Input::new(source, cancel, 0, length.min(64 * 1024), 64 * 1024)?;
    loop {
        match input.line() {
            Ok(Some(line)) if is_header(&line) => return Ok(true),
            Ok(Some(line)) if line.contains(&0) => return Ok(false),
            Ok(Some(_)) => {}
            Ok(None) => return Ok(false),
            Err(ScanError::Limit) if input.line_too_long => return Ok(false),
            Err(error) => return Err(error),
        }
    }
}
pub(crate) struct Record {
    pub name: Vec<u8>,
    pub encoding: Encoding,
    pub size: u64,
    pub start: u64,
    pub end: u64,
}
pub(crate) struct Index {
    pub records: Vec<Record>,
    pub work: u64,
}
/// Parse and validate one body's terminal markers; output is delivered only to a caller sink.
pub(crate) fn body(
    input: &mut Input<'_>,
    base64: bool,
    mut deliver: impl FnMut(&[u8]) -> Result<(), ScanError>,
) -> Result<(Encoding, u64), ScanError> {
    let mut encoding = if base64 { Some(Encoding::Base64) } else { None };
    let mut size = 0u64;
    let mut padded = false;
    loop {
        let line = input.line()?.ok_or(ScanError::Invalid)?;
        let encoding = match encoding {
            Some(value) => value,
            None => {
                let value = line::identify(&line)?;
                encoding = Some(value);
                value
            }
        };
        let bytes = if base64 {
            if line == b"====" {
                return Ok((encoding, size));
            }
            if padded {
                return Err(ScanError::Invalid);
            }
            let (bytes, terminal) = line::base64(&line)?;
            padded = terminal;
            bytes
        } else {
            let bytes = line::counted(&line, encoding)?;
            if bytes.is_empty() {
                if input.line()?.as_deref() != Some(b"end".as_slice()) {
                    return Err(ScanError::Invalid);
                }
                return Ok((encoding, size));
            }
            bytes
        };
        size = size
            .checked_add(bytes.len() as u64)
            .filter(|size| *size <= WORK_LIMIT)
            .ok_or(ScanError::Limit)?;
        deliver(&bytes)?;
    }
}
pub(crate) fn inspect(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    let length = source
        .seek(SeekFrom::End(0))
        .map_err(|e| ScanError::from_source_error(&e))?;
    let read_budget = limits.read_bytes.min(ScanLimits::default().read_bytes);
    if length > read_budget {
        return Err(ScanError::Limit);
    }
    let mut input = Input::new(source, cancel, 0, length, read_budget)?;
    let mut records = Vec::new();
    let mut names = 0usize;
    loop {
        let line_start = input.position;
        let Some(line) = input.line()? else {
            break;
        };
        if let Some(header) = header(&line, limits)? {
            if records.is_empty() && line_start >= 64 * 1024 {
                return Err(ScanError::Limit);
            }
            if records.len() == limits.entries {
                return Err(ScanError::Limit);
            }
            names = names
                .checked_add(header.name.len())
                .filter(|names| *names <= limits.total_name_bytes)
                .ok_or(ScanError::Limit)?;
            let start = input.position;
            let (encoding, size) = body(&mut input, header.base64, |_| Ok(()))?;
            records.push(Record {
                name: header.name,
                encoding,
                size,
                start,
                end: input.position,
            });
        } else if line.iter().any(|byte| *byte == 0 || *byte == b'\r') {
            return Err(ScanError::Invalid);
        }
    }
    if records.is_empty() {
        return Err(ScanError::Unsupported);
    }
    Ok(Index {
        records,
        work: input.work,
    })
}
