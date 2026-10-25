use crate::HeaderSource;
use std::io::{self, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};
pub(crate) const METADATA_LIMIT: u64 = 64 * 1024 * 1024;
pub(crate) struct Budget {
    allocated: u64,
}
impl Budget {
    pub(crate) fn new() -> Self {
        Self { allocated: 0 }
    }
    pub(crate) fn claim(&mut self, bytes: usize) -> Result<(), ScanError> {
        self.allocated = self
            .allocated
            .checked_add(bytes as u64)
            .filter(|n| *n <= METADATA_LIMIT)
            .ok_or(ScanError::Limit)?;
        Ok(())
    }
    pub(crate) fn vec<T>(&mut self, count: usize) -> Result<Vec<T>, ScanError> {
        self.claim(
            count
                .checked_mul(std::mem::size_of::<T>())
                .ok_or(ScanError::Limit)?,
        )?;
        let mut values = Vec::new();
        values
            .try_reserve_exact(count)
            .map_err(|_| ScanError::Limit)?;
        Ok(values)
    }
}
pub(crate) fn io_error(error: io::Error) -> ScanError {
    error
        .get_ref()
        .and_then(|e| e.downcast_ref::<ScanError>())
        .copied()
        .unwrap_or_else(|| {
            if error.kind() == io::ErrorKind::UnexpectedEof {
                ScanError::Invalid
            } else {
                ScanError::Io
            }
        })
}
pub(crate) struct Reader<'a> {
    source: &'a mut dyn ReadSource,
    pub(crate) control: &'a Cancellation,
    length: u64,
    remaining: u64,
}
impl<'a> Reader<'a> {
    pub(crate) fn set_limit(&mut self, limit: u64) {
        self.remaining = limit;
    }
    pub(crate) fn new(
        source: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        limit: u64,
    ) -> Result<Self, ScanError> {
        control.check()?;
        let length = source.seek(SeekFrom::End(0)).map_err(io_error)?;
        Ok(Self {
            source,
            control,
            length,
            remaining: limit.min(METADATA_LIMIT),
        })
    }
    pub(crate) fn range(&self, offset: u64, length: u64) -> Result<(), ScanError> {
        self.check()?;
        if offset.checked_add(length).is_some_and(|n| n <= self.length) {
            Ok(())
        } else {
            Err(ScanError::Invalid)
        }
    }
    pub(crate) fn bytes(
        &mut self,
        offset: u64,
        count: usize,
        budget: &mut Budget,
    ) -> Result<Vec<u8>, ScanError> {
        self.range(offset, count as u64)?;
        if count as u64 > self.remaining {
            return Err(ScanError::Limit);
        }
        let mut bytes = budget.vec(count)?;
        bytes.resize(count, 0);
        self.read_exact_at(offset, &mut bytes)?;
        Ok(bytes)
    }
}
impl HeaderSource for Reader<'_> {
    fn length(&self) -> u64 {
        self.length
    }
    fn check(&self) -> Result<(), ScanError> {
        self.control.check()
    }
    fn read_exact_at(&mut self, offset: u64, output: &mut [u8]) -> Result<(), ScanError> {
        self.range(offset, output.len() as u64)?;
        if output.len() as u64 > self.remaining {
            return Err(ScanError::Limit);
        }
        self.source
            .seek(SeekFrom::Start(offset))
            .map_err(io_error)?;
        let mut at = 0;
        while at < output.len() {
            self.check()?;
            let size = (output.len() - at).min(64 * 1024);
            let n = match self.source.read(&mut output[at..at + size]) {
                Err(error) if error.get_ref().is_some_and(|e| e.is::<ScanError>()) => {
                    return Err(io_error(error));
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                value => value.map_err(io_error)?,
            };
            if n == 0 {
                return Err(ScanError::Invalid);
            }
            self.remaining -= n as u64;
            at += n;
        }
        self.check()
    }
}
