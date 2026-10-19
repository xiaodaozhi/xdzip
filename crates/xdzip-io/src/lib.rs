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

/// Open a source without allowing a special file to block the caller.
/// Unix uses a nonblocking classification open before normal reads.
pub fn open_regular_readonly(path: &std::path::Path) -> std::io::Result<std::fs::File> {
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use rustix::fs::OFlags;
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(OFlags::NONBLOCK.bits() as i32);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "source is not a regular file",
        ));
    }
    Ok(file)
}

#[cfg(test)]
mod source_tests {
    use super::open_regular_readonly;

    #[test]
    fn regular_source_opens_and_directory_is_rejected() {
        let root = std::env::temp_dir().join(format!(
            "xdzip-regular-source-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let file = root.join("archive.bin");
        std::fs::write(&file, b"source").unwrap();
        assert!(open_regular_readonly(&file).is_ok());
        assert!(open_regular_readonly(&root).is_err());
        std::fs::remove_file(file).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
}

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
