use std::io::SeekFrom;
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};
pub(crate) const MAXIMUM: u64 = 24 * 1024 * 1024 * 1024;
#[derive(Clone, Copy)]
/// Stricter reader-private quotas. Each field has a hard 24 GiB maximum.
pub struct Limits {
    /// Actual input bytes, including authentication and repeated payload reads.
    pub input: u64,
    /// Actual hash compression-block bytes, including KDF, pads and finalization.
    pub hash: u64,
    /// Actual AES ciphertext-block bytes decrypted, including session material.
    pub aes: u64,
    /// Decoded bytes admitted for delivery to the host sink.
    pub decoded: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            input: MAXIMUM,
            hash: MAXIMUM,
            aes: MAXIMUM,
            decoded: MAXIMUM,
        }
    }
}
#[derive(Clone, Copy, Default)]
pub(crate) struct Usage {
    pub input: u64,
    pub hash: u64,
    pub aes: u64,
    pub decoded: u64,
}
pub(crate) struct Work {
    pub left: Limits,
    pub used: Usage,
    #[cfg(test)]
    pub hash_observer: Option<Box<dyn FnMut(u64)>>,
}
fn charge(left: &mut u64, used: &mut u64, amount: u64) -> Result<(), ScanError> {
    *left = left.checked_sub(amount).ok_or(ScanError::Limit)?;
    *used = used.checked_add(amount).ok_or(ScanError::Limit)?;
    Ok(())
}
impl Work {
    pub fn new(mut limits: Limits) -> Self {
        limits.input = limits.input.min(MAXIMUM);
        limits.hash = limits.hash.min(MAXIMUM);
        limits.aes = limits.aes.min(MAXIMUM);
        limits.decoded = limits.decoded.min(MAXIMUM);
        Self {
            left: limits,
            used: Usage::default(),
            #[cfg(test)]
            hash_observer: None,
        }
    }
    pub fn input(&mut self, amount: u64) -> Result<(), ScanError> {
        charge(&mut self.left.input, &mut self.used.input, amount)
    }
    pub fn hash(&mut self, amount: u64) -> Result<(), ScanError> {
        charge(&mut self.left.hash, &mut self.used.hash, amount)?;
        #[cfg(test)]
        if let Some(observer) = self.hash_observer.as_mut() {
            observer(self.used.hash);
        }
        Ok(())
    }
    pub fn aes(&mut self, amount: u64) -> Result<(), ScanError> {
        charge(&mut self.left.aes, &mut self.used.aes, amount)
    }
    pub fn decoded(&mut self, amount: u64) -> Result<(), ScanError> {
        charge(&mut self.left.decoded, &mut self.used.decoded, amount)
    }
}
pub(crate) fn seek(source: &mut dyn ReadSource, position: u64) -> Result<(), ScanError> {
    if source
        .seek(SeekFrom::Start(position))
        .map_err(|e| ScanError::from_source_error(&e))?
        != position
    {
        return Err(ScanError::SourceChanged);
    }
    Ok(())
}
pub(crate) fn read(
    source: &mut dyn ReadSource,
    mut bytes: &mut [u8],
    cancel: &Cancellation,
    work: &mut Work,
) -> Result<(), ScanError> {
    while !bytes.is_empty() {
        cancel.check()?;
        let take = bytes
            .len()
            .min(work.left.input.min(usize::MAX as u64) as usize);
        if take == 0 {
            return Err(ScanError::Limit);
        }
        let count = match source.read(&mut bytes[..take]) {
            Err(error)
                if error.kind() == std::io::ErrorKind::Interrupted && error.get_ref().is_none() =>
            {
                continue;
            }
            Err(error) => return Err(ScanError::from_source_error(&error)),
            Ok(0) => return Err(ScanError::SourceChanged),
            Ok(count) if count <= take => count,
            Ok(_) => return Err(ScanError::Io),
        };
        work.input(count as u64)?;
        bytes = &mut bytes[count..];
    }
    Ok(())
}
