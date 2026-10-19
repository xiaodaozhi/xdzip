//! Host-only output capabilities, private staging and whole-file publication.
//! Path syntax is not authority. Every relative operation is anchored to held directories.
mod identity;
mod input;
mod private;
mod target;
#[cfg(test)]
mod tests;
pub use identity::{FileIdentity, FileVersion};
pub use input::{InputEntry, InputError, InputReader, InputTree};
pub use target::{CommitReceipt, ExistingFile, Grant, SetCommitError, Stage, Workspace};

/// Stable filesystem failures without arbitrary OS strings or user paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputError {
    /// Permissions or an ordinary filesystem operation failed.
    Io,
    /// Disk space was exhausted.
    NoSpace,
    /// A path is a link, special object, foreign-owned or broadly writable.
    UnsafeTarget,
    /// The granted root, parent or expected leaf changed.
    TargetChanged,
    /// A destination already exists; no overwrite occurred.
    Conflict,
    /// The filesystem cannot provide the required publication primitive.
    Unsupported,
    /// Transaction or name admission budget was exceeded.
    Limit,
}
impl std::fmt::Display for OutputError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Io => "output_io",
            Self::NoSpace => "disk_full",
            Self::UnsafeTarget => "unsafe_target",
            Self::TargetChanged => "target_changed",
            Self::Conflict => "output_conflict",
            Self::Unsupported => "unsupported_filesystem",
            Self::Limit => "resource_limit",
        })
    }
}
impl std::error::Error for OutputError {}
pub(crate) fn failure(error: std::io::Error) -> OutputError {
    match error.kind() {
        std::io::ErrorKind::AlreadyExists => OutputError::Conflict,
        std::io::ErrorKind::StorageFull => OutputError::NoSpace,
        std::io::ErrorKind::Unsupported | std::io::ErrorKind::CrossesDevices => {
            OutputError::Unsupported
        }
        _ => OutputError::Io,
    }
}
