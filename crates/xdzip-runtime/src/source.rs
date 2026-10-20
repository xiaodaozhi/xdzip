//! Best-effort source version checks, never an atomic filesystem snapshot claim.
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::time::SystemTime;
use xdzip_codec_api::ScanError;

#[derive(Clone, Eq, PartialEq)]
pub(crate) struct SourceVersion {
    protected_identity: xdzip_io::FileIdentity,
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
            protected_identity: xdzip_io::FileIdentity::read(file).map_err(|_| ScanError::Io)?,
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
    parts: Vec<Part>,
    cached: Option<(usize, File)>,
    split: bool,
    name: PathBuf,
    position: u64,
    tail: Option<std::sync::Arc<xdzip_codec_zip::split::NormalizedTail>>,
}
#[derive(Clone)]
struct Part {
    path: PathBuf,
    version: SourceVersion,
    start: u64,
}
impl Source {
    pub fn new(file: File, path: PathBuf, version: SourceVersion) -> Self {
        Self {
            cached: Some((0, file)),
            split: false,
            name: path.clone(),
            parts: vec![Part {
                path,
                version,
                start: 0,
            }],
            position: 0,
            tail: None,
        }
    }
    pub fn open(
        path: &std::path::Path,
        control: &xdzip_codec_api::Cancellation,
    ) -> Result<Self, ScanError> {
        let mut file = xdzip_io::open_regular_readonly(path).map_err(|_| ScanError::Io)?;
        let version = SourceVersion::read(&file)?;
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .ok_or(ScanError::Io)?;
        let lower = filename.to_ascii_lowercase();
        let (base, width, pkzip) = if let Some((base, ext)) = filename.rsplit_once('.') {
            if ext.len() >= 3
                && ext.bytes().all(|b| b.is_ascii_digit())
                && base.to_ascii_lowercase().ends_with(".zip")
            {
                (base.to_owned(), 3, false)
            } else if (ext.len() >= 3
                && matches!(ext.as_bytes().first(), Some(b'z' | b'Z'))
                && ext[1..].bytes().all(|b| b.is_ascii_digit()))
                || (lower.ends_with(".zip")
                    && xdzip_codec_zip::split::end_disk(&mut file).map_or(true, |disk| disk > 0))
            {
                (base.to_owned(), 2, true)
            } else {
                return Ok(Self::new(file, path.into(), version));
            }
        } else {
            return Ok(Self::new(file, path.into(), version));
        };
        let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
        let prefix = if pkzip {
            format!("{}.z", base.to_ascii_lowercase())
        } else {
            format!("{}.", base.to_ascii_lowercase())
        };
        let mut numbered = std::collections::BTreeMap::new();
        let mut final_part = None;
        for (scanned, item) in std::fs::read_dir(parent)
            .map_err(|_| ScanError::Io)?
            .enumerate()
        {
            control.check()?;
            if scanned >= 100_000 {
                return Err(ScanError::Limit);
            }
            let item = item.map_err(|_| ScanError::Io)?;
            let Some(name) = item.file_name().to_str().map(str::to_owned) else {
                continue;
            };
            let folded = name.to_ascii_lowercase();
            if pkzip && folded == format!("{}.zip", base.to_ascii_lowercase()) {
                if final_part.replace(item.path()).is_some() {
                    return Err(ScanError::Invalid);
                }
            } else if let Some(ext) = folded.strip_prefix(&prefix) {
                if ext.len() < width || !ext.bytes().all(|b| b.is_ascii_digit()) {
                    continue;
                }
                let number = ext.parse::<usize>().map_err(|_| ScanError::Limit)?;
                if number == 0 || number > 1024 || ext != format!("{number:0width$}") {
                    return Err(ScanError::Invalid);
                }
                if numbered.insert(number, item.path()).is_some() {
                    return Err(ScanError::Invalid);
                }
            }
        }
        if numbered.is_empty() && lower.ends_with(".zip") {
            return Ok(Self::new(file, path.into(), version));
        }
        if numbered.is_empty()
            || numbered.len() > 1024
            || *numbered.last_key_value().unwrap().0 != numbered.len()
        {
            return Err(ScanError::MissingVolume);
        }
        let mut paths: Vec<_> = numbered.into_values().collect();
        if pkzip {
            paths.push(final_part.ok_or(ScanError::MissingVolume)?);
        }
        if paths.len() > 1024 {
            return Err(ScanError::Limit);
        }
        let mut parts = Vec::new();
        let mut start = 0u64;
        for path in paths {
            control.check()?;
            let metadata =
                std::fs::symlink_metadata(&path).map_err(|_| ScanError::MissingVolume)?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err(ScanError::Io);
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if metadata.file_attributes() & 0x400 != 0 {
                    return Err(ScanError::Io);
                }
            }
            let file = xdzip_io::open_regular_readonly(&path).map_err(|_| ScanError::Io)?;
            let version = SourceVersion::read(&file)?;
            if version.length == 0 {
                return Err(ScanError::Invalid);
            }
            let next = start.checked_add(version.length).ok_or(ScanError::Limit)?;
            parts.push(Part {
                path,
                version,
                start,
            });
            start = next;
        }
        let name = parent.join(if pkzip { format!("{base}.zip") } else { base });
        let mut source = Self {
            parts,
            cached: None,
            split: true,
            name,
            position: 0,
            tail: None,
        };
        if pkzip {
            let lengths = source
                .parts
                .iter()
                .map(|p| p.version.length)
                .collect::<Vec<_>>();
            source.tail = Some(std::sync::Arc::new(xdzip_codec_zip::split::normalize(
                &mut source,
                &lengths,
                control,
            )?));
        }
        source.validate()?;
        source.position = 0;
        Ok(source)
    }
    pub fn validate(&self) -> Result<(), ScanError> {
        // Reopen read-only to compare handle identities, not path timestamps alone.
        // Failure to reopen or inspect also invalidates the old snapshot.
        let version = |file: &File| SourceVersion::read(file).map_err(|_| ScanError::SourceChanged);
        if let Some((index, file)) = &self.cached
            && version(file)? != self.parts[*index].version
        {
            return Err(ScanError::SourceChanged);
        }
        for part in &self.parts {
            if self.split {
                let m =
                    std::fs::symlink_metadata(&part.path).map_err(|_| ScanError::SourceChanged)?;
                if m.file_type().is_symlink() {
                    return Err(ScanError::SourceChanged);
                }
                #[cfg(windows)]
                {
                    use std::os::windows::fs::MetadataExt;
                    if m.file_attributes() & 0x400 != 0 {
                        return Err(ScanError::SourceChanged);
                    }
                }
            }
            let current = xdzip_io::open_regular_readonly(&part.path)
                .map_err(|_| ScanError::SourceChanged)?;
            if version(&current)? != part.version {
                return Err(ScanError::SourceChanged);
            }
        }
        Ok(())
    }
    /// Independent input cursor, checked against the original immutable scan.
    pub fn fork(&self) -> Result<Self, ScanError> {
        self.validate()?;
        let source = Self {
            parts: self.parts.clone(),
            cached: None,
            split: self.split,
            name: self.name.clone(),
            position: 0,
            tail: self.tail.clone(),
        };
        source.validate()?;
        Ok(source)
    }
    pub fn file(&mut self) -> &mut Self {
        self
    }
    pub fn identities(&self) -> Result<Vec<xdzip_io::FileIdentity>, ScanError> {
        Ok(self
            .parts
            .iter()
            .map(|p| p.version.protected_identity)
            .collect())
    }
    pub fn retained_bytes(&self) -> usize {
        self.parts
            .iter()
            .map(|p| std::mem::size_of::<Part>() + p.path.as_os_str().len() * 2)
            .sum::<usize>()
            + self.tail.as_ref().map_or(0, |t| t.bytes.capacity())
    }
    pub fn length(&self) -> u64 {
        self.tail.as_ref().map_or_else(
            || self.parts.last().map_or(0, |p| p.start + p.version.length),
            |t| t.start + t.bytes.len() as u64,
        )
    }
    pub fn name(&self) -> &str {
        source_name(&self.name)
    }
    pub fn is_split(&self) -> bool {
        self.split
    }
}
impl Read for Source {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() || self.position >= self.length() {
            return Ok(0);
        }
        if let Some(tail) = &self.tail
            && self.position >= tail.start
        {
            let at = (self.position - tail.start) as usize;
            let n = output.len().min(tail.bytes.len() - at);
            output[..n].copy_from_slice(&tail.bytes[at..at + n]);
            self.position += n as u64;
            return Ok(n);
        }
        let at = self
            .parts
            .partition_point(|p| p.start <= self.position)
            .saturating_sub(1);
        if self.cached.as_ref().is_none_or(|(index, _)| *index != at) {
            self.cached = None;
            let file = xdzip_io::open_regular_readonly(&self.parts[at].path)?;
            if SourceVersion::read(&file).map_err(io::Error::other)? != self.parts[at].version {
                return Err(io::Error::other("source_changed"));
            }
            self.cached = Some((at, file));
        }
        let part = &self.parts[at];
        let offset = self.position - part.start;
        let mut n = output.len().min((part.version.length - offset) as usize);
        if let Some(tail) = &self.tail {
            n = n.min((tail.start - self.position) as usize);
        }
        let file = &mut self.cached.as_mut().expect("opened current volume").1;
        file.seek(SeekFrom::Start(offset))?;
        let n = file.read(&mut output[..n])?;
        if n == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "source_changed",
            ));
        }
        self.position += n as u64;
        Ok(n)
    }
}
impl Seek for Source {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let next = match pos {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
            SeekFrom::End(n) => i128::from(self.length()) + i128::from(n),
        };
        self.position = u64::try_from(next)
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid seek"))?;
        Ok(self.position)
    }
}
#[cfg(all(test, unix))]
impl std::os::fd::AsRawFd for Source {
    fn as_raw_fd(&self) -> std::os::fd::RawFd {
        std::os::fd::AsRawFd::as_raw_fd(&self.cached.as_ref().expect("opened regular source").1)
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
