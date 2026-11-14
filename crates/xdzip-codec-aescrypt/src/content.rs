use crate::{
    index::{self, Admitted, Metadata},
    keys::{Cbc, Mac},
    work::{self, Work},
};
use xdzip_codec_api::{
    Cancellation, ReadSource, ScanError, ScanLimits,
    payload::{DecodeError, DecodedSink},
};
use zeroize::{Zeroize, Zeroizing};
pub(crate) fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Unsupported => DecodeError::Unsupported,
        ScanError::PasswordRequired => DecodeError::PasswordRequired,
        ScanError::WrongPassword => DecodeError::WrongPassword,
        ScanError::PasswordOrIntegrity => DecodeError::PasswordOrIntegrity,
        _ => DecodeError::Integrity,
    }
}
/// One admitted source and opening credential. It retains keys, never a plaintext password.
pub struct Reader<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    index: Admitted,
    work: Work,
    pending_auxiliary: u64,
    failure: Option<DecodeError>,
}
impl<'a> Reader<'a> {
    /// Authenticate the complete envelope before exposing its exact-size metadata.
    pub fn open(
        source: &'a mut dyn ReadSource,
        password: Option<&str>,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        Self::open_with_limits(source, password, cancel, work::Limits::default())
    }
    /// Apply stricter private budgets. Limits cannot relax any hard maximum.
    pub fn open_with_limits(
        source: &'a mut dyn ReadSource,
        password: Option<&str>,
        cancel: &'a Cancellation,
        limits: work::Limits,
    ) -> Result<Self, DecodeError> {
        let mut work = Work::new(limits);
        let scan = ScanLimits {
            read_bytes: work.left.input,
            ..ScanLimits::default()
        };
        let index =
            index::inspect(source, password, cancel, scan, &mut work).map_err(decode_error)?;
        let pending_auxiliary = work
            .used
            .hash
            .checked_add(work.used.aes)
            .ok_or(DecodeError::Limit)?;
        Ok(Self {
            source,
            cancel,
            index,
            work,
            pending_auxiliary,
            failure: None,
        })
    }
    /// Immutable virtual-file metadata after complete cipher authentication.
    pub fn metadata(&self) -> &Metadata {
        &self.index.metadata
    }
    /// Export into a host sink, reauthenticating exactly the ciphertext being decrypted.
    pub fn decode(
        &mut self,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.deliver(sink, cancel);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
    fn deliver(
        &mut self,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.cancel.check().map_err(decode_error)?;
        cancel.check().map_err(decode_error)?;
        if self.pending_auxiliary != 0 {
            sink.charge_auxiliary(self.pending_auxiliary)?;
            self.pending_auxiliary = 0;
        }
        let header = &self.index.header;
        work::seek(self.source, header.start).map_err(decode_error)?;
        let mut mac = Mac::new(&self.index.keys.key, &mut self.work).map_err(decode_error)?;
        let mut cbc = Cbc::new(&self.index.keys.key, *self.index.keys.iv);
        let mut remaining = header.cipher;
        let mut delivered = 0u64;
        let mut buffer = Zeroizing::new([0; 65536]);
        let mut final_block = Zeroizing::new([0; 16]);
        while remaining != 0 {
            self.cancel.check().map_err(decode_error)?;
            cancel.check().map_err(decode_error)?;
            let take = remaining.min(buffer.len() as u64) as usize;
            work::read(
                self.source,
                &mut buffer[..take],
                self.cancel,
                &mut self.work,
            )
            .map_err(decode_error)?;
            mac.update(&buffer[..take], &mut self.work)
                .map_err(decode_error)?;
            cbc.decrypt(&mut buffer[..take], cancel, &mut self.work)
                .map_err(decode_error)?;
            remaining -= take as u64;
            if remaining == 0 && header.version == 3 {
                final_block.copy_from_slice(&buffer[take - 16..take]);
            }
            let wanted = (self.index.metadata.size - delivered).min(take as u64) as usize;
            self.work.decoded(wanted as u64).map_err(decode_error)?;
            if wanted != 0 {
                sink.write_chunk(&buffer[..wanted])?;
            }
            delivered += wanted as u64;
            buffer.zeroize();
        }
        if !constant_time_eq::constant_time_eq_32(
            mac.finish(&mut self.work)
                .map_err(decode_error)?
                .as_ref()
                .try_into()
                .expect("MAC width"),
            &header.data_tag,
        ) || delivered != self.index.metadata.size
        {
            return Err(DecodeError::Integrity);
        }
        if header.version == 3
            && index::padding_size(&final_block, header.cipher).map_err(decode_error)?
                != self.index.metadata.size
        {
            return Err(DecodeError::Integrity);
        }
        self.cancel.check().map_err(decode_error)?;
        cancel.check().map_err(decode_error)?;
        Ok(())
    }
}
#[cfg(test)]
mod tests;
