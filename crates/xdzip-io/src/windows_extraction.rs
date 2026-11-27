//! Windows-only authorization for a selected directory alias or shared parent.
use crate::target::extraction_failure;
use crate::{FileIdentity, OutputError, private};
use std::fs::{File, OpenOptions};
use std::io;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;
use std::os::windows::fs::{MetadataExt, OpenOptionsExt};
use std::os::windows::io::{AsRawHandle, FromRawHandle};
use std::path::{Component, Path, PathBuf, Prefix};
use std::ptr::{null, null_mut};
use windows_sys::Wdk::Foundation::OBJECT_ATTRIBUTES;
use windows_sys::Wdk::Storage::FileSystem::{
    FILE_CREATE, FILE_DIRECTORY_FILE, FILE_SYNCHRONOUS_IO_NONALERT, NtCreateFile,
};
use windows_sys::Win32::Foundation::{
    OBJ_CASE_INSENSITIVE, OBJ_DONT_REPARSE, RtlNtStatusToDosError, STATUS_NOT_A_DIRECTORY,
    STATUS_OBJECT_NAME_COLLISION, STATUS_REPARSE_POINT_ENCOUNTERED, UNICODE_STRING,
};
use windows_sys::Win32::Storage::FileSystem::{
    FILE_ADD_SUBDIRECTORY, FILE_ATTRIBUTE_DIRECTORY, FILE_ATTRIBUTE_REPARSE_POINT,
    FILE_ATTRIBUTE_TAG_INFO, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
    FILE_GENERIC_READ, FILE_GENERIC_WRITE, FILE_READ_ATTRIBUTES, FILE_SHARE_READ, FILE_SHARE_WRITE,
    FILE_TRAVERSE, FileAttributeTagInfo, GetFileInformationByHandleEx, GetFinalPathNameByHandleW,
    GetVolumeInformationByHandleW, SYNCHRONIZE, VOLUME_NAME_DOS,
};
use windows_sys::Win32::System::IO::IO_STATUS_BLOCK;
use windows_sys::Win32::System::SystemServices::{
    FILE_PERSISTENT_ACLS, FILE_SUPPORTS_HARD_LINKS, IO_REPARSE_TAG_MOUNT_POINT,
    IO_REPARSE_TAG_SYMLINK,
};

const SHARE: u32 = FILE_SHARE_READ | FILE_SHARE_WRITE;

pub(crate) struct Selected {
    requested: PathBuf,
    resolved: PathBuf,
    file: File,
    identity: FileIdentity,
    ancestors: Vec<File>,
}

impl Selected {
    pub(crate) fn open(path: &Path) -> Result<Self, OutputError> {
        let requested = std::path::absolute(path).map_err(extraction_failure)?;
        let file = selected_directory(&requested)?;
        let identity = FileIdentity::read(&file).map_err(extraction_failure)?;
        let resolved = final_path(&file)?;
        verify_local_ntfs(&file)?;

        // cap-std on Windows uses pathname-based filesystem APIs for some mutations.
        // Hold real path ancestors without delete sharing, and disallow reparse
        // components in the resolved chain. The user alias is checked separately.
        let mut ancestors = Vec::new();
        for path in resolved.ancestors().skip(1) {
            ancestors.push(path);
        }
        let ancestors = ancestors
            .into_iter()
            .rev()
            .map(|path| {
                let handle = open_directory(
                    path,
                    FILE_TRAVERSE | FILE_READ_ATTRIBUTES | SYNCHRONIZE,
                    true,
                )?;
                check_ordinary(&handle)?;
                Ok(handle)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let selected = Self {
            requested,
            resolved,
            file,
            identity,
            ancestors,
        };
        selected.validate()?;
        Ok(selected)
    }

    pub(crate) fn resolved(&self) -> &Path {
        &self.resolved
    }
    pub(crate) fn file(&self) -> &File {
        &self.file
    }
    pub(crate) fn identity(&self) -> FileIdentity {
        self.identity
    }

    pub(crate) fn validate(&self) -> Result<(), OutputError> {
        check_ordinary(&self.file).map_err(|_| OutputError::TargetChanged)?;
        for ancestor in &self.ancestors {
            check_ordinary(ancestor).map_err(|_| OutputError::TargetChanged)?;
        }
        let current =
            selected_directory(&self.requested).map_err(|_| OutputError::TargetChanged)?;
        let resolved = open_directory(&self.resolved, FILE_GENERIC_READ, true)
            .map_err(|_| OutputError::TargetChanged)?;
        check_ordinary(&resolved).map_err(|_| OutputError::TargetChanged)?;
        self.check_identity(&current)?;
        self.check_identity(&resolved)
    }

    pub(crate) fn check_create_access(&self) -> Result<(), OutputError> {
        self.validate()?;
        let file = open_directory(
            &self.resolved,
            FILE_ADD_SUBDIRECTORY | FILE_READ_ATTRIBUTES,
            true,
        )?;
        check_ordinary(&file)?;
        self.check_identity(&file)
    }

    pub(crate) fn create_child(&self, leaf: &str) -> Result<(PathBuf, File), OutputError> {
        self.validate()?;
        let dir = create_protected_directory(&self.file, leaf)?;
        private::check_protected_directory(&dir)?;
        self.validate()?;
        Ok((self.resolved.join(leaf), dir))
    }

    fn check_identity(&self, file: &File) -> Result<(), OutputError> {
        if FileIdentity::read(file).map_err(|_| OutputError::TargetChanged)? != self.identity {
            return Err(OutputError::TargetChanged);
        }
        Ok(())
    }
}

fn open_directory(path: &Path, access: u32, nofollow: bool) -> Result<File, OutputError> {
    let flags = FILE_FLAG_BACKUP_SEMANTICS
        | if nofollow {
            FILE_FLAG_OPEN_REPARSE_POINT
        } else {
            0
        };
    OpenOptions::new()
        .access_mode(access)
        .share_mode(SHARE)
        .custom_flags(flags)
        .open(path)
        .map_err(extraction_failure)
}

fn selected_directory(path: &Path) -> Result<File, OutputError> {
    let leaf = open_directory(path, FILE_GENERIC_READ, true)?;
    let mut info = FILE_ATTRIBUTE_TAG_INFO::default();
    // SAFETY: live file handle, writable buffer for the selected information class.
    if unsafe {
        GetFileInformationByHandleEx(
            leaf.as_raw_handle(),
            FileAttributeTagInfo,
            (&mut info as *mut FILE_ATTRIBUTE_TAG_INFO).cast(),
            size_of::<FILE_ATTRIBUTE_TAG_INFO>() as u32,
        )
    } == 0
    {
        return Err(extraction_failure(io::Error::last_os_error()));
    }
    if info.FileAttributes & FILE_ATTRIBUTE_REPARSE_POINT == 0 {
        if info.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0 {
            return Err(OutputError::TargetType);
        }
        return Ok(leaf);
    }
    if info.FileAttributes & FILE_ATTRIBUTE_DIRECTORY == 0
        || !matches!(
            info.ReparseTag,
            IO_REPARSE_TAG_MOUNT_POINT | IO_REPARSE_TAG_SYMLINK
        )
    {
        return Err(OutputError::TargetType);
    }
    let resolved = open_directory(path, FILE_GENERIC_READ, false)?;
    check_ordinary(&resolved).map_err(|error| match error {
        OutputError::UnsafeTarget => OutputError::TargetType,
        error => error,
    })?;
    Ok(resolved)
}

fn check_ordinary(file: &File) -> Result<(), OutputError> {
    let metadata = file.metadata().map_err(extraction_failure)?;
    if !metadata.is_dir() || metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
        return Err(OutputError::UnsafeTarget);
    }
    Ok(())
}

fn final_path(file: &File) -> Result<PathBuf, OutputError> {
    let mut wide = vec![0; 32_768];
    // SAFETY: the borrowed handle and the writable UTF-16 buffer stay live.
    let len = unsafe {
        GetFinalPathNameByHandleW(
            file.as_raw_handle(),
            wide.as_mut_ptr(),
            wide.len() as u32,
            VOLUME_NAME_DOS,
        )
    } as usize;
    if len == 0 {
        return Err(extraction_failure(io::Error::last_os_error()));
    }
    if len >= wide.len() {
        return Err(OutputError::Limit);
    }
    let verbatim = PathBuf::from(std::ffi::OsString::from_wide(&wide[..len]));
    let Some(Component::Prefix(prefix)) = verbatim.components().next() else {
        return Err(OutputError::Unsupported);
    };
    if !matches!(prefix.kind(), Prefix::VerbatimDisk(_)) {
        return Err(OutputError::Unsupported);
    }
    // The display path should use standard drive syntax when lossless.
    let normal = PathBuf::from(std::ffi::OsString::from_wide(&wide[4..len]));
    if normal.components().any(|part| match part {
        Component::Normal(name) => name.to_string_lossy().ends_with([' ', '.']),
        _ => false,
    }) {
        Ok(verbatim)
    } else {
        Ok(normal)
    }
}

fn verify_local_ntfs(file: &File) -> Result<(), OutputError> {
    let mut flags = 0;
    let mut filesystem = [0u16; 16];
    // SAFETY: the borrowed handle and both output buffers remain live.
    if unsafe {
        GetVolumeInformationByHandleW(
            file.as_raw_handle(),
            null_mut(),
            0,
            null_mut(),
            null_mut(),
            &mut flags,
            filesystem.as_mut_ptr(),
            filesystem.len() as u32,
        )
    } == 0
    {
        return Err(extraction_failure(io::Error::last_os_error()));
    }
    let required = FILE_PERSISTENT_ACLS | FILE_SUPPORTS_HARD_LINKS;
    if filesystem[..5] != [b'N' as u16, b'T' as u16, b'F' as u16, b'S' as u16, 0]
        || flags & required != required
    {
        return Err(OutputError::Unsupported);
    }
    Ok(())
}

fn create_protected_directory(parent: &File, leaf: &str) -> Result<File, OutputError> {
    let leaf_path =
        xdzip_domain::path::PortablePath::parse(leaf).ok_or(OutputError::UnsafeTarget)?;
    if leaf_path.parts().len() != 1 {
        return Err(OutputError::UnsafeTarget);
    }
    let descriptor = private::protected_directory_descriptor()?;
    let mut wide: Vec<_> = leaf.encode_utf16().collect();
    let bytes = (wide.len() * size_of::<u16>()) as u16;
    let mut name = UNICODE_STRING {
        Length: bytes,
        MaximumLength: bytes,
        Buffer: wide.as_mut_ptr(),
    };
    let attributes = OBJECT_ATTRIBUTES {
        Length: size_of::<OBJECT_ATTRIBUTES>() as u32,
        RootDirectory: parent.as_raw_handle(),
        ObjectName: &mut name,
        Attributes: OBJ_CASE_INSENSITIVE | OBJ_DONT_REPARSE,
        SecurityDescriptor: descriptor.raw(),
        SecurityQualityOfService: null_mut(),
    };
    let mut handle = null_mut();
    let mut status_block = IO_STATUS_BLOCK::default();
    // SAFETY: one validated relative component, a live parent capability, a
    // protected DACL and initialized outputs. FILE_CREATE cannot open or replace
    // an existing object. A successful synchronous call returns an owned handle
    // without delete sharing from the instant the new directory is created.
    let status = unsafe {
        NtCreateFile(
            &mut handle,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            &attributes,
            &mut status_block,
            null(),
            FILE_ATTRIBUTE_DIRECTORY,
            SHARE,
            FILE_CREATE,
            FILE_DIRECTORY_FILE | FILE_SYNCHRONOUS_IO_NONALERT,
            null(),
            0,
        )
    };
    // The ordinary, held parent and single leaf make these statuses local
    // name collisions, including an existing non-directory or reparse leaf.
    if matches!(
        status,
        STATUS_OBJECT_NAME_COLLISION | STATUS_NOT_A_DIRECTORY | STATUS_REPARSE_POINT_ENCOUNTERED
    ) {
        return Err(OutputError::Conflict);
    }
    if status < 0 {
        // SAFETY: maps a numeric NTSTATUS without accessing pointers or handles.
        let error = unsafe { RtlNtStatusToDosError(status) };
        return Err(extraction_failure(io::Error::from_raw_os_error(
            error as i32,
        )));
    }
    if handle.is_null() {
        return Err(OutputError::Io);
    }
    // SAFETY: NtCreateFile transferred ownership of this live directory handle.
    let directory = unsafe { File::from_raw_handle(handle) };
    Ok(directory)
}
