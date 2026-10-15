//! Best-effort source version checks, never an atomic filesystem snapshot claim.
use std::fs::File;
use std::path::PathBuf;
use std::time::SystemTime;
use xdzip_codec_api::ScanError;

#[derive(Clone, Eq, PartialEq)]
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
    /// Independent input cursor, checked against the original immutable scan.
    pub fn fork(&self) -> Result<Self, ScanError> {
        self.validate()?;
        let file = File::open(&self.path).map_err(|_| ScanError::SourceChanged)?;
        let source = Self::new(file, self.path.clone(), self.version.clone());
        source.validate()?;
        Ok(source)
    }
    pub fn file(&mut self) -> &mut File {
        &mut self.file
    }
    pub fn identity(&self) -> Result<xdzip_io::FileIdentity, ScanError> {
        xdzip_io::FileIdentity::read(&self.file).map_err(|_| ScanError::SourceChanged)
    }
    pub fn name(&self) -> &str {
        source_name(&self.path)
    }
}
fn source_name(path: &std::path::Path) -> &str {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("archive");
    let stem = if path
        .extension()
        .is_some_and(|s| s.eq_ignore_ascii_case("gz"))
        && stem.to_ascii_lowercase().ends_with(".tar")
    {
        &stem[..stem.len() - 4]
    } else {
        stem
    };
    if stem.is_empty() { "archive" } else { stem }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compound_suffixes_use_the_same_nonempty_folder_as_the_ui_intent() {
        for (input, expected) in [
            ("归档.tar.gz", "归档"),
            ("归档.TAR.GZ", "归档"),
            ("archive.tgz", "archive"),
            ("archive.tar", "archive"),
            ("archive.zip", "archive"),
            (".tar.gz", "archive"),
        ] {
            assert_eq!(source_name(std::path::Path::new(input)), expected);
        }
    }
}
