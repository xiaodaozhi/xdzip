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
        let current =
            xdzip_io::open_regular_readonly(&self.path).map_err(|_| ScanError::SourceChanged)?;
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
        let file =
            xdzip_io::open_regular_readonly(&self.path).map_err(|_| ScanError::SourceChanged)?;
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
    pub fn length(&self) -> u64 {
        self.version.length
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

    #[cfg(unix)]
    #[test]
    #[allow(unsafe_code)]
    fn replaced_source_and_special_aliases_are_rejected_without_blocking() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::fs::symlink;
        use std::process::{Command, Stdio};

        const VARIABLE: &str = "XDZIP_OWNED_SPECIAL_SOURCE";
        if let Some(root) = std::env::var_os(VARIABLE) {
            let root = PathBuf::from(root);
            let path = root.join("source.zip");
            let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/zip/normal.zip");
            let original = std::fs::read(fixture).unwrap();
            std::fs::write(&path, &original).unwrap();
            let archive = crate::Archive::open(&path, &Default::default()).unwrap();
            let alias = root.join("alias.zip");
            symlink(&path, &alias).unwrap();
            let alias_archive = crate::Archive::open(&alias, &Default::default()).unwrap();
            assert_eq!(archive.len(), alias_archive.len());
            let source = archive.source;
            source.validate().unwrap();
            source.fork().unwrap();
            std::fs::rename(&path, root.join("original.zip")).unwrap();
            let raw = CString::new(path.as_os_str().as_bytes()).unwrap();
            // SAFETY: a valid NUL-terminated path under the owned fixture.
            assert_eq!(unsafe { libc::mkfifo(raw.as_ptr(), 0o600) }, 0);
            assert!(xdzip_io::open_regular_readonly(&path).is_err());
            assert_eq!(source.validate(), Err(ScanError::SourceChanged));
            assert!(matches!(source.fork(), Err(ScanError::SourceChanged)));
            assert!(matches!(
                crate::Archive::open(&path, &Default::default()),
                Err(ScanError::Io)
            ));
            assert!(xdzip_io::open_regular_readonly(&alias).is_err());
            assert!(matches!(
                crate::Archive::open(&alias, &Default::default()),
                Err(ScanError::Io)
            ));
            assert_eq!(std::fs::read(root.join("original.zip")).unwrap(), original);
            return;
        }
        let root = std::env::temp_dir().join(format!(
            "xdzip-source-fifo-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        // A regression must fail in bounded time rather than hanging the test
        // runner on FIFO open. The child can be killed without leaking a thread.
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "source::tests::replaced_source_and_special_aliases_are_rejected_without_blocking",
            ])
            .env(VARIABLE, &root)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break Some(status);
            }
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                break None;
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        };
        std::fs::remove_dir_all(root).unwrap();
        assert!(
            status.is_some_and(|status| status.success()),
            "special source rejected within deadline"
        );
    }
}
