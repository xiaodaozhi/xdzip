// crates/xdzip-update-helper/src/windows.rs

//! Held Windows handles protect verified inputs while the interactive installer runs.

use super::*;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::sync::Arc;
use windows_sys::Win32::Foundation::{
    CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE, WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT, FILE_SHARE_READ,
};
use windows_sys::Win32::System::Registry::{HKEY_CURRENT_USER, RRF_RT_REG_SZ, RegGetValueW};
use windows_sys::Win32::System::Threading::{
    CreateMutexW, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE,
    QueryFullProcessImageNameW, WaitForSingleObject,
};

pub(super) const PLATFORM: &str = "windows";
pub(super) const PACKAGE: &str = "exe";
const INSTALL_ID: &str = "56DF0D26-2B26-40C4-9E85-FF0F8C8937CA";

fn wide(value: impl AsRef<OsStr>) -> Vec<u16> {
    value.as_ref().encode_wide().chain(Some(0)).collect()
}

struct Handle(HANDLE);
impl Drop for Handle {
    fn drop(&mut self) {
        // SAFETY: this owner only receives successful, owned native handles.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub(super) struct Job {
    path: PathBuf,
    grant: Arc<xdzip_io::Grant>,
    _directory: File,
}
impl Job {
    pub(super) fn load(request: &Path) -> Result<Self> {
        if !request.is_absolute() || request.file_name() != Some(OsStr::new("request.json")) {
            return Err("invalid_request_path");
        }
        let path = request
            .parent()
            .ok_or("invalid_request_path")?
            .to_path_buf();
        let grant = xdzip_io::Grant::for_creation(&path).map_err(|_| "unsafe_job_directory")?;
        let directory = fs::OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT)
            .open(&path)
            .map_err(|_| "unsafe_job_directory")?;
        if directory
            .metadata()
            .map_err(|_| "unsafe_job_directory")?
            .file_attributes()
            & 0x400
            != 0
        {
            return Err("unsafe_job_directory");
        }
        Ok(Self {
            path,
            grant,
            _directory: directory,
        })
    }
    pub(super) fn path(&self) -> &Path {
        &self.path
    }
    pub(super) fn open(&self, name: &OsStr) -> Result<File> {
        self.grant.validate().map_err(|_| "job_directory_changed")?;
        let path = self.path.join(name);
        if path.parent() != Some(&self.path) {
            return Err("invalid_request_path");
        }
        stable_file(&path)
    }
    pub(super) fn control(&self, name: &str) -> Result<Option<File>> {
        match fs::symlink_metadata(self.path.join(name)) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(_) => Err("control_io"),
            Ok(_) => self.open(OsStr::new(name)).map(Some),
        }
    }
    pub(super) fn write(&self, name: &str, bytes: &[u8]) -> Result<()> {
        self.grant.validate().map_err(|_| "job_directory_changed")?;
        let temporary = self.path.join(format!("{name}.helper-new"));
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| "control_io")?;
        file.write_all(bytes)
            .and_then(|()| file.sync_all())
            .map_err(|_| "control_io")?;
        drop(file);
        fs::rename(temporary, self.path.join(name)).map_err(|_| "control_io")
    }
}

fn stable_file(path: &Path) -> Result<File> {
    let file = fs::OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
        .open(path)
        .map_err(|_| "source_io")?;
    let metadata = file.metadata().map_err(|_| "source_io")?;
    if !metadata.is_file() || metadata.file_attributes() & 0x400 != 0 {
        return Err("unsafe_source");
    }
    Ok(file)
}

fn registry(name: &str) -> Result<String> {
    let key = wide("Software\\xdzip\\Install");
    let name = wide(name);
    let mut buffer = [0u16; 32768];
    let mut length = std::mem::size_of_val(&buffer) as u32;
    // SAFETY: all strings are NUL terminated; buffer and length are valid outputs.
    let status = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            key.as_ptr(),
            name.as_ptr(),
            RRF_RT_REG_SZ,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut length,
        )
    };
    if status != 0 || length < 2 || length as usize > std::mem::size_of_val(&buffer) {
        return Err("unsupported_installation");
    }
    let length = length as usize / 2;
    if buffer[length - 1] != 0 {
        return Err("unsupported_installation");
    }
    String::from_utf16(&buffer[..length - 1]).map_err(|_| "unsupported_installation")
}

fn equal_path(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left
            .to_string_lossy()
            .eq_ignore_ascii_case(&right.to_string_lossy()),
        _ => false,
    }
}

pub(super) fn installed_executable(request: &Request) -> Result<PathBuf> {
    if registry("InstallId")? != INSTALL_ID
        || !equal_path(Path::new(&registry("Location")?), &request.install_root)
    {
        return Err("unsupported_installation");
    }
    xdzip_io::Grant::for_creation(&request.install_root).map_err(|_| "unsafe_installation")?;
    let manifest = fs::symlink_metadata(request.install_root.join("install-manifest.txt"))
        .map_err(|_| "unsupported_installation")?;
    if !manifest.is_file() || manifest.file_attributes() & 0x400 != 0 {
        return Err("unsafe_installation");
    }
    Ok(request.install_root.join("xdzip.exe"))
}
pub(super) fn updated_executable(request: &Request) -> Result<PathBuf> {
    installed_executable(request)
}

pub(super) struct Parent(Handle);
impl Parent {
    pub(super) fn open(_job: &Job, request: &Request) -> Result<Self> {
        // SAFETY: OpenProcess returns an owned handle or NULL; no access grants process mutation.
        let process = unsafe {
            OpenProcess(
                PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
                0,
                request.parent_pid,
            )
        };
        if process.is_null() {
            return Err("parent_missing");
        }
        let process = Handle(process);
        let mut path = [0u16; 32768];
        let mut length = path.len() as u32;
        // SAFETY: the live process handle is queried into a bounded initialized buffer.
        if unsafe { QueryFullProcessImageNameW(process.0, 0, path.as_mut_ptr(), &mut length) } == 0
        {
            return Err("parent_identity");
        }
        let path = String::from_utf16(&path[..length as usize]).map_err(|_| "parent_identity")?;
        if !equal_path(Path::new(&path), &request.install_root.join("xdzip.exe")) {
            return Err("parent_identity");
        }
        Ok(Self(process))
    }
    pub(super) fn exited(&self) -> Result<bool> {
        // SAFETY: waiting with zero timeout only observes this held process object.
        match unsafe { WaitForSingleObject(self.0.0, 0) } {
            WAIT_OBJECT_0 => Ok(true),
            WAIT_TIMEOUT => Ok(false),
            _ => Err("parent_wait"),
        }
    }
}

pub(super) struct Gate {
    _main: Handle,
    _transaction: Handle,
}
impl Gate {
    pub(super) fn acquire(request: &Request) -> Result<Self> {
        fn create(name: &str) -> Result<Handle> {
            let name = wide(name);
            // SAFETY: name is terminated, default token ACL applies, handle ownership is retained.
            let handle = unsafe { CreateMutexW(std::ptr::null(), 0, name.as_ptr()) };
            if handle.is_null() {
                return Err("update_gate");
            }
            let handle = Handle(handle);
            // SAFETY: captured immediately after CreateMutex; existing objects reject parallel updates.
            if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
                return Err("update_in_progress");
            }
            Ok(handle)
        }
        let main = create("Local\\xdzip.desktop.updating")?;
        let transaction = create(&format!(
            "Local\\xdzip.desktop.update.{}",
            request.transaction
        ))?;
        Ok(Self {
            _main: main,
            _transaction: transaction,
        })
    }
}

pub(super) struct Staged {
    pub(super) inspection: xdzip_update::Inspection,
    pub(super) revision: String,
    _package: File,
}
pub(super) fn stage(_job: &Job, _request: &Request, prepared: Prepared) -> Result<Staged> {
    // The read-only, no-write/no-delete-sharing package handle remains live through Setup.
    Ok(Staged {
        inspection: prepared.inspection,
        revision: prepared.revision,
        _package: prepared.package,
    })
}
pub(super) fn install(_job: &Job, request: &Request, _staged: &Staged) -> Result<()> {
    let status = Command::new(&request.package_file)
        .arg("/NORESTART")
        .arg("/RESTARTEXITCODE=3010")
        .arg("/NOCLOSEAPPLICATIONS")
        .arg("/NORESTARTAPPLICATIONS")
        .arg(format!("/XDZIPUPDATE={}", request.transaction))
        .arg(format!("/DIR={}", request.install_root.display()))
        .stdin(Stdio::null())
        .status()
        .map_err(|_| "installer_start")?;
    match status.code() {
        Some(0) => Ok(()),
        Some(2 | 5) => Err("installation_cancelled"),
        Some(20) => Err("cleanup_incomplete"),
        Some(3010) => Err("restart_required"),
        _ => Err("installation_failed"),
    }
}
pub(super) fn restart(_job: &Job, request: &Request, _parent: &Parent) -> Result<()> {
    Command::new(installed_executable(request)?)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(|_| ())
        .map_err(|_| "restart_failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn verified_package_handle_refuses_replacement_and_concurrent_writes() {
        let path = std::env::temp_dir().join(format!(
            "xdzip-held-package-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, b"owned fixture").unwrap();
        let held = stable_file(&path).unwrap();
        assert!(fs::OpenOptions::new().write(true).open(&path).is_err());
        assert!(fs::remove_file(&path).is_err());
        assert_eq!(read_bounded(held, 32).unwrap(), b"owned fixture");
        fs::remove_file(&path).unwrap();
    }
}
