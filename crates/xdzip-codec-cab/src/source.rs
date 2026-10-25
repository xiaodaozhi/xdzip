use std::io::{self, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};

pub(crate) struct Volume {
    pub(crate) base: u64,
    pub(crate) length: u64,
    pub(crate) name: Vec<u8>,
}
pub(crate) struct Reader<'a> {
    source: &'a mut dyn ReadSource,
    pub(crate) control: &'a Cancellation,
    pub(crate) volumes: Vec<Volume>,
    limit: u64,
    read: u64,
}
fn source_error(error: io::Error) -> ScanError {
    if let Some(error) = error
        .get_ref()
        .and_then(|error| error.downcast_ref::<ScanError>())
    {
        return *error;
    }
    if error.kind() == io::ErrorKind::UnexpectedEof {
        ScanError::Invalid
    } else {
        ScanError::Io
    }
}
impl<'a> Reader<'a> {
    pub(crate) fn new(
        source: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        limit: u64,
    ) -> Result<Self, ScanError> {
        control.check()?;
        let length = source.seek(SeekFrom::End(0)).map_err(source_error)?;
        if source
            .volume_lengths()
            .is_some_and(|items| items.is_empty() || items.len() > 1024)
            || source.volume_names().is_some_and(|items| {
                items.len() > 1024
                    || items
                        .iter()
                        .any(|name| name.is_empty() || name.len() > 4096 || name.contains(&0))
            })
        {
            return Err(ScanError::Invalid);
        }
        let lengths = source.volume_lengths().map(<[u64]>::to_vec);
        let names = source.volume_names().map(<[Vec<u8>]>::to_vec);
        let (lengths, names) = match (lengths, names) {
            (None, None) => (vec![length], vec![b"archive.cab".to_vec()]),
            (Some(lengths), Some(names)) => (lengths, names),
            _ => return Err(ScanError::Invalid),
        };
        if lengths.is_empty() || lengths.len() > 1024 || names.len() != lengths.len() {
            return Err(ScanError::Invalid);
        }
        let mut base = 0u64;
        let mut volumes = Vec::with_capacity(lengths.len());
        for (length, name) in lengths.into_iter().zip(names) {
            if name.is_empty() || name.len() > 4096 || name.contains(&0) {
                return Err(ScanError::Invalid);
            }
            volumes.push(Volume { base, length, name });
            base = base.checked_add(length).ok_or(ScanError::Invalid)?;
        }
        if base != length {
            return Err(ScanError::Invalid);
        }
        Ok(Self {
            source,
            control,
            volumes,
            limit,
            read: 0,
        })
    }
    pub(crate) fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }
    pub(crate) fn check(&self) -> Result<(), ScanError> {
        self.control.check()
    }
    pub(crate) fn range(&self, volume: usize, offset: u64, count: u64) -> Result<u64, ScanError> {
        self.check()?;
        let part = self.volumes.get(volume).ok_or(ScanError::Invalid)?;
        if offset
            .checked_add(count)
            .is_none_or(|end| end > part.length)
        {
            return Err(ScanError::Invalid);
        }
        part.base.checked_add(offset).ok_or(ScanError::Invalid)
    }
    pub(crate) fn bytes(
        &mut self,
        volume: usize,
        offset: u64,
        count: usize,
    ) -> Result<Vec<u8>, ScanError> {
        let absolute = self.range(volume, offset, count as u64)?;
        if count as u64 > self.limit.saturating_sub(self.read) {
            return Err(ScanError::Limit);
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(count)
            .map_err(|_| ScanError::Limit)?;
        output.resize(count, 0);
        self.source
            .seek(SeekFrom::Start(absolute))
            .map_err(source_error)?;
        let mut at = 0;
        while at < count {
            self.check()?;
            let n = match self.source.read(&mut output[at..]) {
                Err(error) if error.get_ref().is_some_and(|error| error.is::<ScanError>()) => {
                    return Err(source_error(error));
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => result.map_err(source_error)?,
            };
            if n == 0 {
                return Err(ScanError::Invalid);
            }
            self.read += n as u64;
            at += n;
        }
        Ok(output)
    }
    pub(crate) fn string(
        &mut self,
        volume: usize,
        offset: &mut u64,
        max: usize,
    ) -> Result<Vec<u8>, ScanError> {
        let mut result = Vec::new();
        // Read-ahead beyond NUL is charged as input, but never retained as a name.
        loop {
            let remaining = self.volumes[volume]
                .length
                .checked_sub(*offset)
                .ok_or(ScanError::Invalid)?;
            let count = 32usize.min(max + 1 - result.len()).min(remaining as usize);
            if count == 0 {
                return Err(ScanError::Invalid);
            }
            let bytes = self.bytes(volume, *offset, count)?;
            if let Some(end) = bytes.iter().position(|byte| *byte == 0) {
                result.extend_from_slice(&bytes[..end]);
                *offset = offset
                    .checked_add(end as u64 + 1)
                    .ok_or(ScanError::Invalid)?;
                return Ok(result);
            }
            result.extend_from_slice(&bytes);
            *offset = offset.checked_add(count as u64).ok_or(ScanError::Invalid)?;
            if result.len() > max {
                return Err(ScanError::Limit);
            }
        }
    }
}
