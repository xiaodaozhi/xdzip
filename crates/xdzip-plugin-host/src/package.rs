use crate::Error;
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};
use xdzip_io::{FileVersion, open_regular_nofollow};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    manifest_version: u32,
    plugin_id: String,
    version: String,
    author: String,
    protocol_majors: Vec<u64>,
    targets: Vec<String>,
    entrypoint: String,
    sha256: String,
    formats: Vec<String>,
    permissions: Permissions,
    resources: Resources,
    license_files: Vec<String>,
    source_url: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Permissions {
    broker_source_read: bool,
    broker_output_write: bool,
    network: bool,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Resources {
    memory_bytes: u64,
    threads: u32,
}

pub(crate) struct Package {
    pub id: String,
    pub executable: PathBuf,
    // A private verified copy remains alive until the child and its pipes are gone.
    _root: tempfile::TempDir,
}
impl Package {
    pub fn load(path: &Path) -> Result<Self, Error> {
        let mut file = open_regular_nofollow(path).map_err(|_| Error::Package)?;
        if file.metadata()?.len() > 32 * 1024 {
            return Err(Error::Package);
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let m: Manifest = serde_json::from_slice(&bytes).map_err(|_| Error::Package)?;
        let target = format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH);
        if m.manifest_version != 1
            || m.protocol_majors != [1]
            || m.formats != ["xdp1"]
            || !m.targets.contains(&target)
            || m.targets.len() > 8
            || m.plugin_id.is_empty()
            || m.plugin_id.len() > 64
            || !m
                .plugin_id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b".-_".contains(&b))
            || m.version.is_empty()
            || m.version.len() > 32
            || m.author.is_empty()
            || m.author.len() > 128
            || !m.permissions.broker_source_read
            || !m.permissions.broker_output_write
            || m.permissions.network
            || m.resources.memory_bytes != 512 * 1024 * 1024
            || m.resources.threads != 1
            || m.license_files.is_empty()
            || m.license_files.len() > 4
            || !m.source_url.starts_with("https://")
            || m.source_url.len() > 512
            || m.sha256.len() != 64
            || !m
                .sha256
                .bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
        {
            return Err(Error::Package);
        }
        let parent = path.parent().ok_or(Error::Package)?.canonicalize()?;
        let root_dir = Dir::open_ambient_dir(&parent, cap_std::ambient_authority())?;
        for name in &m.license_files {
            let license = member(&root_dir, name)?;
            if license.metadata()?.len() > 128 * 1024 {
                return Err(Error::Package);
            }
        }
        let mut input = member(&root_dir, &m.entrypoint)?;
        let version = FileVersion::read(&input)?;
        if version.length == 0 || version.length > 64 * 1024 * 1024 {
            return Err(Error::Package);
        }
        let root = tempfile::Builder::new().prefix("xdzip-sdk-").tempdir()?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700))?;
        }
        let executable =
            root.path()
                .canonicalize()?
                .join(if cfg!(windows) { "peer.exe" } else { "peer" });
        let mut output = File::options()
            .write(true)
            .create_new(true)
            .open(&executable)?;
        let mut hash = Sha256::new();
        let mut buffer = [0; 64 * 1024];
        let mut count = 0u64;
        loop {
            let n = input.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            count = count.checked_add(n as u64).ok_or(Error::Limit)?;
            if count > 64 * 1024 * 1024 {
                return Err(Error::Package);
            }
            output.write_all(&buffer[..n])?;
            hash.update(&buffer[..n]);
        }
        if count != version.length
            || FileVersion::read(&input)? != version
            || hex(&hash.finalize()) != m.sha256
        {
            return Err(Error::Package);
        }
        output.sync_all()?;
        drop(output);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o500))?;
        }
        Ok(Self {
            id: m.plugin_id,
            executable,
            _root: root,
        })
    }
}
fn member(parent: &Dir, name: &str) -> Result<File, Error> {
    if name.is_empty() || name.len() > 256 || name.contains('\\') {
        return Err(Error::Package);
    }
    let mut dir = parent.try_clone()?;
    let components: Vec<_> = Path::new(name).components().collect();
    for (index, part) in components.iter().enumerate() {
        let Component::Normal(part) = part else {
            return Err(Error::Package);
        };
        if dir.symlink_metadata(part)?.file_type().is_symlink() {
            return Err(Error::Package);
        }
        if index + 1 < components.len() {
            dir = dir.open_dir_nofollow(part).map_err(|_| Error::Package)?;
        } else {
            let mut options = OpenOptions::new();
            options.read(true).follow(FollowSymlinks::No);
            #[cfg(unix)]
            {
                use cap_std::fs::OpenOptionsExt;
                options.custom_flags(libc::O_NONBLOCK);
            }
            let file = dir
                .open_with(part, &options)
                .map_err(|_| Error::Package)?
                .into_std();
            if !file.metadata()?.is_file() {
                return Err(Error::Package);
            }
            return Ok(file);
        }
    }
    Err(Error::Package)
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
