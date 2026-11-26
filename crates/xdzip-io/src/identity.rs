use std::fs::File;
use std::io;
use std::time::SystemTime;

/// Native identity; handle-based and never inferred from a display pathname.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FileIdentity(u64, u64);
impl FileIdentity {
    /// An opaque representation for a native identity carried across the desktop bridge.
    pub fn encode(self) -> String {
        format!("{:016x}:{:016x}", self.0, self.1)
    }

    /// Parse a previously issued identity. A match never replaces filesystem authorization.
    pub fn decode(value: &str) -> Option<Self> {
        let (volume, object) = value.split_once(':')?;
        if volume.len() != 16 || object.len() != 16 {
            return None;
        }
        Some(Self(
            u64::from_str_radix(volume, 16).ok()?,
            u64::from_str_radix(object, 16).ok()?,
        ))
    }

    /// Read the identity of an already opened file or directory.
    pub fn read(file: &File) -> io::Result<Self> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let m = file.metadata()?;
            Ok(Self(m.dev(), m.ino()))
        }
        #[cfg(windows)]
        {
            let m = winapi_util::file::information(file)?;
            Ok(Self(m.volume_serial_number(), m.file_index()))
        }
        #[cfg(not(any(unix, windows)))]
        {
            let _ = file;
            Err(io::ErrorKind::Unsupported.into())
        }
    }
}
/// Best-effort version fingerprint for conditional overwrite decisions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileVersion {
    /// Native object identity.
    pub identity: FileIdentity,
    /// Current length.
    pub length: u64,
    /// Last modification timestamp.
    pub modified: SystemTime,
}
impl FileVersion {
    /// Read all fields through the owned handle. This is not a content CAS or atomic snapshot.
    pub fn read(file: &File) -> io::Result<Self> {
        let m = file.metadata()?;
        Ok(Self {
            identity: FileIdentity::read(file)?,
            length: m.len(),
            modified: m.modified()?,
        })
    }
}
