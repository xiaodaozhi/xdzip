//! Read-only authority for the immediately adjacent parts of a selected archive.
use crate::FileIdentity;
use cap_fs_ext::{FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::io;
use std::path::{Component, Path, PathBuf};

/// Held directory used to enumerate, admit and reopen sibling archive volumes.
pub struct VolumeDirectory {
    dir: Dir,
    path: PathBuf,
    identity: FileIdentity,
}

impl VolumeDirectory {
    /// Hold the parent named by the selected archive path.
    pub fn open(source: &Path) -> io::Result<Self> {
        let parent = source
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let dir = Dir::open_ambient_dir(parent, cap_std::ambient_authority())?;
        let identity = FileIdentity::read(&dir.try_clone()?.into_std_file())?;
        Ok(Self {
            dir,
            path: parent.to_owned(),
            identity,
        })
    }

    /// Enumerate names from the held directory. Callers apply their format's limits.
    pub fn names(&self) -> io::Result<impl Iterator<Item = io::Result<OsString>>> {
        Ok(self
            .dir
            .entries()?
            .map(|entry| entry.map(|entry| entry.file_name())))
    }

    /// Display path belonging to the held directory.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Open a plain regular file by its single leaf name without following links.
    pub fn open_part(&self, name: &OsStr) -> io::Result<File> {
        let mut components = Path::new(name).components();
        if components.next() != Some(Component::Normal(name)) || components.next().is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid volume leaf",
            ));
        }
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32);
        }
        crate::classify_regular(self.dir.open_with(name, &options)?.into_std(), true)
    }

    /// Confirm that the caller's directory path still names the admitted object.
    pub fn validate(&self) -> io::Result<()> {
        let current = Dir::open_ambient_dir(&self.path, cap_std::ambient_authority())?;
        if FileIdentity::read(&current.into_std_file())? != self.identity {
            return Err(io::Error::other("volume directory changed"));
        }
        Ok(())
    }
}
