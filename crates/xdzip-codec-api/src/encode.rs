//! Internal creation extension contract. Paths and filesystem commits belong to the host.
use crate::Cancellation;
use std::io::{Read, Write};

/// Frozen format-visible record, independent of host paths or GUI objects.
#[derive(Clone, Debug)]
pub struct EncodeEntry {
    /// Validated portable relative name without a directory slash.
    pub name: String,
    /// Explicit ordinary directory record.
    pub directory: bool,
    /// Exact admitted input bytes; directories use zero.
    pub size: u64,
    /// UTC modification seconds; dates outside ZIP's DOS range use extended time.
    pub modified_unix: i64,
}
/// Stable failure class at the source/format/output boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EncodeError {
    /// Source read failed or its frozen version changed.
    Source,
    /// Host-owned output failed; the host retains the actual OS failure class.
    Output,
    /// Input size, encoding or format invariants were violated.
    Invalid,
    /// Requested method or level is not supported.
    Unsupported,
    /// Cooperative cancellation observed before publication.
    Cancelled,
}
/// Host-controlled read access by physical input index, never by an archive name.
pub trait EncodeInput {
    /// Frozen, bounded records, in deterministic archive order.
    fn entries(&self) -> &[EncodeEntry];
    /// Borrow one ordinary-file stream with host version checks.
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError>;
}
/// Writable staging stream. It grants no path, directory or publication authority.
pub trait EncodeOutput: Write {}
impl<T: Write + ?Sized> EncodeOutput for T {}
/// One actual-work progress observation; the host controls event throttling.
pub struct EncodeProgress<'a> {
    /// Current portable name from the frozen record.
    pub name: &'a str,
    /// Actual input bytes read so far across files.
    pub input_bytes: u64,
}
/// Codec-defined settings. Method identifiers are independent of ZIP header numbers.
#[derive(Clone, Copy)]
pub struct EncodeSettings<'a> {
    /// Capability identifier, such as "store" or "deflate".
    pub method: &'a str,
    /// Admitted backend level; its meaning belongs to that codec's capability.
    pub level: u8,
    /// Optional borrowed password. Never include it in progress, metadata or errors.
    pub password: Option<&'a str>,
}
/// Reusable source extension point for archive creation, separate from decoding.
pub trait CreationCodec {
    /// Write one complete archive, including compression termination and central metadata.
    /// Unsupported codec capabilities must fail explicitly.
    fn encode(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError>;
}
