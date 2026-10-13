//! Best-effort source version checks, never an atomic filesystem snapshot claim.
use std::fs::File;
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
    #[cfg(windows)]
    identity: (u64, u64),
}
impl SourceVersion {
    pub fn read(file: &File) -> Result<Self, ScanError> {
        let metadata = file.metadata().map_err(|_| ScanError::Io)?;
        if !metadata.is_file() {
            return Err(ScanError::SourceChanged);
        }
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        #[cfg(windows)]
        let info = winapi_util::file::information(file).map_err(|_| ScanError::Io)?;
        Ok(Self {
            length: metadata.len(),
            modified: metadata.modified().map_err(|_| ScanError::Io)?,
            created: metadata.created().ok(),
            #[cfg(unix)]
            identity: (metadata.dev(), metadata.ino()),
            // Creation time can survive a same-name replacement on NTFS. Compare
            // the opened file's volume/index through the dependency's safe wrapper.
            #[cfg(windows)]
            identity: (info.volume_serial_number(), info.file_index()),
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
        // Reopen read-only to compare handle identities, not path timestamps alone.
        // Failure to reopen or inspect also invalidates the old snapshot.
        let current = File::open(&self.path).map_err(|_| ScanError::SourceChanged)?;
        let version = |file: &File| SourceVersion::read(file).map_err(|_| ScanError::SourceChanged);
        if version(&self.file)? != self.version || version(&current)? != self.version {
            Err(ScanError::SourceChanged)
        } else {
            Ok(())
        }
    }
}
