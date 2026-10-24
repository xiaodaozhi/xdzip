//! Internal format contracts. Payload adapters receive input and a host-owned sink, never paths.
pub mod encode;
pub mod names;
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
    /// Aggregate decoded and escaped display-name allocation budget.
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

/// Container capability selected from source bytes, never a filename hint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    /// ZIP container.
    Zip,
    /// Uncompressed TAR container.
    Tar,
    /// One GZIP filter around TAR.
    TarGzip,
}
impl Format {
    /// Stable user-visible capability label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Zip => "ZIP",
            Self::Tar => "TAR",
            Self::TarGzip => "TAR.GZ",
        }
    }
}

/// Owned metadata of a physical record. Declared sizes are not verified payload sizes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Entry {
    /// Actual container capability; packed sizes are unknown for TAR.GZ records.
    pub format: Format,
    /// Stable within this immutable scan, independent of the name and view row.
    pub id: EntryId,
    /// Container-resolved filename bytes before output validation. TAR strips leading ./ only.
    pub raw_name: Vec<u8>,
    /// Decoded UTF-8 spelling when it differs from, or declares an encoding for, raw bytes.
    /// Output still requires portable-path validation and destination authorization.
    pub decoded_name: Option<String>,
    /// How the immutable effective name was resolved; not a confidence guarantee.
    pub name_encoding: names::NameEncoding,
    /// Safe text only; never a filesystem output path.
    pub display_name: String,
    /// Directory marker from metadata; no filesystem object is created.
    pub directory: bool,
    /// Declared symbolic/hard link. Only ZIP symbolic links have an extraction policy.
    pub link: bool,
    /// Raw Unix mode for ZIP Unix creators; zero for other creators/formats. Never applied directly.
    pub unix_mode: u32,
    /// Raw declared uncompressed bytes.
    pub size: u64,
    /// Raw declared compressed bytes.
    pub packed_size: u64,
    /// ZIP method number; listing a method does not imply decoding support.
    pub method: u16,
    /// DOS-shaped date/time bits. ZIP uses local time, TAR uses UTC; zero is unavailable.
    pub modified_dos: u32,
    /// Metadata encryption flag. Browsing alone never requests a password.
    pub encrypted: bool,
    /// Supported password profile from container metadata, not payload authentication.
    pub password_supported: bool,
    /// Suspicious/unresolved name or unsupported special record; never safe-to-extract approval.
    pub warning: bool,
}

impl Entry {
    /// Effective archive-relative name. Never use escaped display text as a path.
    pub fn name_bytes(&self) -> &[u8] {
        self.decoded_name
            .as_ref()
            .map_or(self.raw_name.as_slice(), |name| name.as_bytes())
    }
    /// Charge every owned name allocation, including retained original bytes.
    pub fn name_capacity(&self) -> usize {
        self.raw_name.capacity()
            + self.display_name.capacity()
            + self.decoded_name.as_ref().map_or(0, String::capacity)
    }
}

/// Stable failures without source paths or archive-supplied messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanError {
    /// Source cannot be read or is not a regular file.
    Io,
    /// Invalid or inconsistent archive metadata.
    Invalid,
    /// Archive feature not admitted by the host (e.g. an unsupported volume layout).
    Unsupported,
    /// A resource admission limit was reached.
    Limit,
    /// Cooperative cancellation was observed.
    Cancelled,
    /// File size or modification time changed during the scan.
    SourceChanged,
    /// One or more required adjacent archive volumes are absent.
    MissingVolume,
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
            Self::MissingVolume => "missing_volume",
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
