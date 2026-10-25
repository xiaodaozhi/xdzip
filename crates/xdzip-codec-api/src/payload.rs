//! Payload decoding is separate from discovery and filesystem authority.
use crate::{Cancellation, Entry, ReadSource};

/// Stable failures at the untrusted compressed-data boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecodeError {
    /// Source I/O failed.
    SourceIo,
    /// Headers, stream termination, descriptor, size or checksum disagree.
    Integrity,
    /// Method, encryption or special entry is not supported.
    Unsupported,
    /// This supported encrypted record requires a task-scoped password.
    PasswordRequired,
    /// The encryption header's password verifier did not match.
    WrongPassword,
    /// Encrypted data failed without a verifier distinguishing its password from corruption.
    PasswordOrIntegrity,
    /// Host resource budget was reached.
    Limit,
    /// Cooperative cancellation was observed.
    Cancelled,
    /// The host rejected an output chunk; inspect its retained error for detail.
    Output,
}
impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SourceIo => "source_io",
            Self::Integrity => "integrity_error",
            Self::Unsupported => "unsupported_feature",
            Self::PasswordRequired => "password_required",
            Self::WrongPassword => "wrong_password",
            Self::PasswordOrIntegrity => "password_or_integrity",
            Self::Limit => "resource_limit",
            Self::Cancelled => "cancelled",
            Self::Output => "output_io",
        })
    }
}
impl std::error::Error for DecodeError {}

/// Host-owned output. Each chunk is metered before filesystem writes.
pub trait DecodedSink {
    /// Accept actual decoded bytes, or stop decoding. No destination is exposed.
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError>;
    /// Emit bytes whose decoding work was already admitted by `charge_skipped`.
    /// A sink that exempts this write must retain and consume private prepaid
    /// credit, and still enforce file and cumulative output limits. The default
    /// conservatively charges the bytes again through the ordinary write path.
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.write_chunk(bytes)
    }
    /// Admit actual decoding work without emitting output, including solid skips
    /// and a newly decoded cached frame. Successful admission may fund later
    /// precharged writes; callers may not assume that the default retains credit.
    fn charge_skipped(&mut self, _bytes: u64) -> Result<(), DecodeError> {
        Ok(())
    }
    /// Account algorithm work already performed, independently of decoded or
    /// delivered file bytes. This never grants prepaid output credit. Task
    /// hosts may impose a separate cumulative limit across records and retries.
    /// Callers must admit future work privately before reporting actual work.
    fn charge_auxiliary(&mut self, _bytes: u64) -> Result<(), DecodeError> {
        Ok(())
    }
}

/// One opened format instance. Its metadata identities match the host snapshot.
pub trait PayloadReader {
    /// Decode to EOF and validate before returning success, including empty records.
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError>;

    /// Decode with a borrowed task secret. Unsupported formats retain normal decoding.
    /// Authentication failures are integrity errors, not proof of a wrong password.
    fn decode_with_password(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        _password: Option<&str>,
    ) -> Result<(), DecodeError> {
        self.decode(entry, sink, control)
    }
}
/// Built-in extension point independent of Qt and output filesystem policy.
pub trait PayloadCodec {
    /// Check the source's container metadata against the immutable host snapshot.
    /// The decoder borrows input, retains bounded locators, and opens no paths.
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError>;
    /// Open a header-encrypted archive using the secret retained by its immutable session.
    fn open_payload_with_password<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
        _password: Option<&str>,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        self.open_payload(source, snapshot, control)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_precharged_write_keeps_ordinary_admission_and_errors() {
        struct Limited(usize);
        impl DecodedSink for Limited {
            fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
                self.0 = self.0.checked_sub(bytes.len()).ok_or(DecodeError::Limit)?;
                Ok(())
            }
        }
        let mut sink = Limited(3);
        sink.charge_auxiliary(u64::MAX).unwrap();
        sink.charge_skipped(100).unwrap();
        sink.write_precharged_chunk(b"abc").unwrap();
        assert_eq!(sink.write_precharged_chunk(b"x"), Err(DecodeError::Limit));
    }

    #[test]
    fn ambiguous_password_errors_have_distinct_stable_identifiers() {
        assert_eq!(
            DecodeError::PasswordOrIntegrity.to_string(),
            "password_or_integrity"
        );
        assert_eq!(
            crate::ScanError::PasswordOrIntegrity.to_string(),
            "password_or_integrity"
        );
        assert_ne!(DecodeError::PasswordOrIntegrity, DecodeError::WrongPassword);
        assert_ne!(
            crate::ScanError::PasswordOrIntegrity,
            crate::ScanError::WrongPassword
        );
    }
}
