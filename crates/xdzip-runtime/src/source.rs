//! Best-effort source version checks, never an atomic filesystem snapshot claim.
use std::fs::{File, Metadata};
use std::path::PathBuf;
use std::time::SystemTime;
use xdzip_codec_api::ScanError;

#[derive(Eq, PartialEq)]
pub(crate) struct SourceVersion {
    length: u64,
    modified: SystemTime,
    created: Option<SystemTime>,
    #[cfg(unix)]
    identity: (u64, u64),
}
impl SourceVersion {
    pub fn read(metadata: &Metadata) -> Result<Self, ScanError> {
        if !metadata.is_file() {
            return Err(ScanError::SourceChanged);
        }
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        Ok(Self {
            length: metadata.len(),
            modified: metadata.modified().map_err(|_| ScanError::Io)?,
            created: metadata.created().ok(),
            #[cfg(unix)]
            identity: (metadata.dev(), metadata.ino()),
        })
    }
}
pub(crate) struct Source {
    file: File,
    path: PathBuf,
    version: SourceVersion,
}
impl Source {
    pub fn new(file: File, path: PathBuf, version: SourceVersion) -> Self {
        Self {
            file,
            path,
            version,
        }
    }
    pub fn validate(&self) -> Result<(), ScanError> {
        let handle = self.file.metadata().map_err(|_| ScanError::SourceChanged)?;
        let current = std::fs::metadata(&self.path).map_err(|_| ScanError::SourceChanged)?;
        if SourceVersion::read(&handle)? != self.version
            || SourceVersion::read(&current)? != self.version
        {
            Err(ScanError::SourceChanged)
        } else {
            Ok(())
        }
    }
}
