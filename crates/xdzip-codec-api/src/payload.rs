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
}
