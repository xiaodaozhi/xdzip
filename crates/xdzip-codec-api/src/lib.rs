//! Internal format contracts. Payload adapters receive input and a host-owned sink, never paths.
pub mod encode;
pub mod payload;
use std::io::{Read, Seek};
use std::sync::atomic::{AtomicBool, Ordering};
use xdzip_domain::identity::EntryId;

/// Seekable input supplied by the host; codecs do not open arbitrary paths.
pub trait ReadSource: Read + Seek {}
impl<T: Read + Seek> ReadSource for T {}

/// Cooperative control shared between the GUI and its single read worker.
#[derive(Default)]
pub struct Cancellation(AtomicBool);
impl Cancellation {
    /// Idempotent, lock-free request. Does not interrupt a blocked OS read.
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
    /// Check at record boundaries and before each input read.
    pub fn check(&self) -> Result<(), ScanError> {
        if self.0.load(Ordering::Relaxed) {
            Err(ScanError::Cancelled)
        } else {
            Ok(())
        }
    }
}

/// Physical-record admission shared by codecs, indexes and the native model.
pub const MAX_ENTRIES: usize = 100_000;

/// Bounded M2 metadata admission; sizes never grant payload decode capability.
#[derive(Clone, Copy)]
pub struct ScanLimits {
    /// Maximum physical records, including duplicates and directories.
    pub entries: usize,
    /// Maximum bytes in one raw filename.
    pub name_bytes: usize,
    /// Aggregate raw filename allocation budget.
    pub total_name_bytes: usize,
    /// Aggregate escaped display-name allocation budget.
    pub display_name_bytes: usize,
    /// Total input bytes read, including repeated reads and the end-record search.
    pub read_bytes: u64,
}
impl Default for ScanLimits {
    fn default() -> Self {
        Self {
            entries: MAX_ENTRIES,
            name_bytes: 4096,
            total_name_bytes: 16 * 1024 * 1024,
            display_name_bytes: 16 * 1024 * 1024,
            read_bytes: 64 * 1024 * 1024,
        }
    }
}

/// Owned metadata of a physical record. Declared sizes are not verified payload sizes.
#[derive(Clone, Debug)]
pub struct Entry {
    /// Stable within this immutable scan, independent of the name and view row.
    pub id: EntryId,
    /// Original filename bytes retained for future encoding and output policy.
    pub raw_name: Vec<u8>,
    /// Safe text only; never a filesystem output path.
    pub display_name: String,
    /// Directory marker from metadata; no filesystem object is created.
    pub directory: bool,
    /// Raw declared uncompressed bytes.
    pub size: u64,
    /// Raw declared compressed bytes.
    pub packed_size: u64,
    /// ZIP method number; listing a method does not imply decoding support.
    pub method: u16,
    /// ZIP local DOS date/time bits; zero denotes unavailable or invalid metadata.
    pub modified_dos: u32,
    /// Metadata encryption flag. No password is requested in this read-only slice.
    pub encrypted: bool,
    /// Suspicious name, non-UTF8 bytes, link or special record; never safe-to-extract approval.
    pub warning: bool,
}

/// Stable failures without source paths or archive-supplied messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanError {
    /// Source cannot be read or is not a regular file.
    Io,
    /// Invalid or inconsistent archive metadata.
    Invalid,
    /// Archive feature not admitted by this slice (e.g. multi-volume).
    Unsupported,
    /// A resource admission limit was reached.
    Limit,
    /// Cooperative cancellation was observed.
    Cancelled,
    /// File size or modification time changed during the scan.
    SourceChanged,
}
impl std::fmt::Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Io => "source_io",
            Self::Invalid => "invalid_archive",
            Self::Unsupported => "unsupported_feature",
            Self::Limit => "resource_limit",
            Self::Cancelled => "cancelled",
            Self::SourceChanged => "source_changed",
        })
    }
}
impl std::error::Error for ScanError {}

/// Internal source extension point; future extraction will use a separate host-owned sink.
pub trait MetadataCodec {
    /// Scan completely or return an error. Never return partial rows as a successful index.
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError>;
}
