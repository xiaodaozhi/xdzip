use std::io::{self, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};

pub(crate) struct Volume {
    pub base: u64,
    pub length: u64,
}
pub(crate) struct Reader<'a> {
    input: &'a mut dyn ReadSource,
    control: &'a Cancellation,
    pub volumes: Vec<Volume>,
    used: u64,
    limit: u64,
}
fn failure(error: io::Error) -> ScanError {
    if let Some(error) = error.get_ref().and_then(|e| e.downcast_ref::<ScanError>()) {
        return *error;
    }
    if error.kind() == io::ErrorKind::UnexpectedEof {
        ScanError::Invalid
    } else {
        ScanError::Io
    }
}
impl<'a> Reader<'a> {
    pub fn new(
        input: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        limit: u64,
    ) -> Result<Self, ScanError> {
        control.check()?;
        let length = input.seek(SeekFrom::End(0)).map_err(failure)?;
        let lengths = input
            .volume_lengths()
            .map_or_else(|| vec![length], <[u64]>::to_vec);
        if lengths.is_empty() || lengths.len() > 1024 {
            return Err(ScanError::Invalid);
        }
        if input.volume_names().is_some_and(|names| {
            names.len() != lengths.len()
                || names
                    .iter()
                    .any(|n| n.is_empty() || n.len() > 4096 || n.contains(&0))
        }) {
            return Err(ScanError::Invalid);
        }
        let mut base = 0u64;
        let mut volumes = Vec::with_capacity(lengths.len());
        for length in lengths {
            if length == 0 {
                return Err(ScanError::Invalid);
            }
            volumes.push(Volume { base, length });
            base = base.checked_add(length).ok_or(ScanError::Invalid)?;
        }
        if base != length {
            return Err(ScanError::Invalid);
        }
        Ok(Self {
            input,
            control,
            volumes,
            used: 0,
            limit,
        })
    }
    pub fn set_limit(&mut self, limit: u64) {
        self.limit = limit;
    }
    pub fn check(&self) -> Result<(), ScanError> {
        self.control.check()
    }
    pub fn control(&self) -> &Cancellation {
        self.control
    }
    pub fn range(&self, volume: usize, at: u64, count: u64) -> Result<u64, ScanError> {
        self.check()?;
        let volume = self.volumes.get(volume).ok_or(ScanError::Invalid)?;
        if at.checked_add(count).is_none_or(|end| end > volume.length) {
            return Err(ScanError::Invalid);
        }
        volume.base.checked_add(at).ok_or(ScanError::Invalid)
    }
    pub fn bytes(&mut self, volume: usize, at: u64, count: usize) -> Result<Vec<u8>, ScanError> {
        let mut bytes = Vec::new();
        if count as u64 > self.limit.saturating_sub(self.used) {
            return Err(ScanError::Limit);
        }
        self.range(volume, at, count as u64)?;
        bytes
            .try_reserve_exact(count)
            .map_err(|_| ScanError::Limit)?;
        bytes.resize(count, 0);
        self.read_into(volume, at, &mut bytes)?;
        Ok(bytes)
    }
    pub fn read_into(&mut self, volume: usize, at: u64, bytes: &mut [u8]) -> Result<(), ScanError> {
        let absolute = self.range(volume, at, bytes.len() as u64)?;
        if bytes.len() as u64 > self.limit.saturating_sub(self.used) {
            return Err(ScanError::Limit);
        }
        self.input
            .seek(SeekFrom::Start(absolute))
            .map_err(failure)?;
        let mut done = 0;
        while done < bytes.len() {
            self.check()?;
            match self.input.read(&mut bytes[done..]) {
                Ok(0) => return Err(ScanError::Invalid),
                Ok(n) => {
                    done += n;
                    self.used += n as u64;
                }
                Err(error)
                    if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none() => {}
                Err(error) => return Err(failure(error)),
            }
        }
        Ok(())
    }
}
