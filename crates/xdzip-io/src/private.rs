use crate::{OutputError, failure};
use std::fs::File;

pub(crate) fn check(file: &File, directory: bool) -> Result<(), OutputError> {
    let m = file.metadata().map_err(failure)?;
    if (directory && !m.is_dir()) || (!directory && !m.is_file()) {
        return Err(OutputError::UnsafeTarget);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if m.uid() != rustix::process::geteuid().as_raw()
            || m.mode() & 0o022 != 0
            || (!directory && m.nlink() != 1)
        {
            return Err(OutputError::UnsafeTarget);
        }
        #[cfg(target_os = "macos")]
        macos::check_acl(file)?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if m.file_attributes() & 0x400 != 0 {
            return Err(OutputError::UnsafeTarget);
        }
        if !directory
            && winapi_util::file::information(file)
                .map_err(failure)?
                .number_of_links()
                != 1
        {
            return Err(OutputError::UnsafeTarget);
        }
        windows::check_acl(file)?;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
#[allow(unsafe_code)]
mod macos {
    use super::*;
    use std::os::fd::AsRawFd;
    use std::os::raw::{c_int, c_void};
    use std::ptr::null_mut;

    // macOS exposes extended ACLs through acl_get_fd_np, but libc does not
    // provide a portable Rust abstraction for this platform-specific API.
    unsafe extern "C" {
        fn acl_get_fd_np(fd: c_int, acl_type: c_int) -> *mut c_void;
        fn acl_valid(acl: *mut c_void) -> c_int;
        fn acl_get_entry(acl: *mut c_void, entry_id: c_int, entry: *mut *mut c_void) -> c_int;
        fn acl_get_tag_type(entry: *mut c_void, tag: *mut c_int) -> c_int;
        fn acl_get_permset_mask_np(entry: *mut c_void, permissions: *mut u64) -> c_int;
        fn acl_get_qualifier(entry: *mut c_void) -> *mut c_void;
        fn acl_free(obj: *mut c_void) -> c_int;
        fn mbr_uuid_to_id(uuid: *const u8, id: *mut libc::id_t, kind: *mut c_int) -> c_int;
        fn __error() -> *mut c_int;
    }
    const ACL_TYPE_EXTENDED: c_int = 0x0000_0100;
    const ACL_FIRST_ENTRY: c_int = 0;
    const ACL_NEXT_ENTRY: c_int = -1;
    const ACL_EXTENDED_ALLOW: c_int = 1;
    const ACL_EXTENDED_DENY: c_int = 2;
    const ID_TYPE_UID: c_int = 0;
    // sys/acl.h: read/list, execute/search, read attributes, read extended
    // attributes, read security and synchronize do not authorize modification.
    const READ_ONLY: u64 = (1 << 1) | (1 << 3) | (1 << 7) | (1 << 9) | (1 << 11) | (1 << 20);
    const KNOWN_PERMISSIONS: u64 = 0x3ffe | (1 << 20);

    struct Allocation(*mut c_void);
    impl Allocation {
        fn release(mut self) -> Result<(), OutputError> {
            let pointer = std::mem::replace(&mut self.0, null_mut());
            // SAFETY: only acl_get_fd_np/acl_get_qualifier allocations enter
            // this owner. Taking the pointer prevents a second free in Drop.
            if unsafe { acl_free(pointer) } == 0 {
                Ok(())
            } else {
                Err(OutputError::UnsafeTarget)
            }
        }
    }
    impl Drop for Allocation {
        fn drop(&mut self) {
            if !self.0.is_null() {
                // SAFETY: error paths still own this unfreed ACL allocation;
                // success explicitly releases it and checks the cleanup result.
                unsafe { acl_free(self.0) };
            }
        }
    }

    fn current_user(entry: *mut c_void) -> Result<bool, OutputError> {
        // SAFETY: entry borrows the validated ACL and remains live throughout;
        // its returned qualifier is a separately owned 16-byte UUID allocation.
        let qualifier = unsafe { acl_get_qualifier(entry) };
        if qualifier.is_null() {
            return Err(OutputError::UnsafeTarget);
        }
        let qualifier = Allocation(qualifier);
        let mut id = 0;
        let mut kind = -1;
        // SAFETY: the qualifier holds a complete UUID; outputs are initialized
        // stack values. Resolve both real and synthetic UUIDs to their UID.
        let status = unsafe { mbr_uuid_to_id(qualifier.0.cast(), &mut id, &mut kind) };
        qualifier.release()?;
        if status != 0 {
            return Err(OutputError::UnsafeTarget);
        }
        Ok(kind == ID_TYPE_UID && id == rustix::process::geteuid().as_raw())
    }

    pub(super) fn check_acl(file: &File) -> Result<(), OutputError> {
        // SAFETY: the descriptor stays borrowed and live; this returns an owned
        // ACL snapshot, not pointers into filesystem state that can be mutated.
        let acl = unsafe { acl_get_fd_np(file.as_raw_fd(), ACL_TYPE_EXTENDED) };
        if acl.is_null() {
            // SAFETY: __error is the current thread's errno slot. Read it before
            // another native call can overwrite the failed query's error.
            let errno = unsafe { *__error() };
            return if errno == libc::ENOENT || errno == libc::ENODATA {
                Ok(())
            } else {
                Err(OutputError::UnsafeTarget)
            };
        }
        let acl = Allocation(acl);
        // SAFETY: this live ACL was allocated by acl_get_fd_np. Validate before
        // using Darwin's EINVAL end-of-iteration convention below.
        if unsafe { acl_valid(acl.0) } != 0 {
            return Err(OutputError::UnsafeTarget);
        }
        let mut entry_id = ACL_FIRST_ENTRY;
        // sys/acl.h bounds extended ACLs at ACL_MAX_ENTRIES (128). Include one
        // final query for the terminator; any other iteration error fails closed.
        for _ in 0..=128 {
            let mut entry = null_mut();
            // SAFETY: acl owns all borrowed entries; output storage is valid.
            if unsafe { acl_get_entry(acl.0, entry_id, &mut entry) } != 0 {
                if unsafe { *__error() } == libc::EINVAL {
                    return acl.release();
                }
                return Err(OutputError::UnsafeTarget);
            }
            if entry.is_null() {
                return Err(OutputError::UnsafeTarget);
            }
            entry_id = ACL_NEXT_ENTRY;
            let mut tag = 0;
            let mut permissions = 0;
            // SAFETY: validated, borrowed entry and initialized output slots.
            if unsafe { acl_get_tag_type(entry, &mut tag) } != 0
                || unsafe { acl_get_permset_mask_np(entry, &mut permissions) } != 0
                || permissions & !KNOWN_PERMISSIONS != 0
            {
                return Err(OutputError::UnsafeTarget);
            }
            match tag {
                ACL_EXTENDED_DENY => {}
                ACL_EXTENDED_ALLOW => {
                    // Inspect effective and inherit-only ACEs alike: permitting
                    // outsider mutation on future children is also unsafe. A
                    // current-user group can include other accounts, so only a
                    // user UUID resolving to the effective UID may modify.
                    if permissions & !READ_ONLY != 0 && !current_user(entry)? {
                        return Err(OutputError::UnsafeTarget);
                    }
                }
                _ => return Err(OutputError::UnsafeTarget),
            }
        }
        Err(OutputError::UnsafeTarget)
    }
}

#[cfg(windows)]
#[allow(unsafe_code)] // Borrowed handles and kernel-owned ACL buffers, confined to this module.
mod windows {
    use super::*;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::ptr::{addr_of, null_mut};
    use windows_sys::Win32::Foundation::{GENERIC_EXECUTE, GENERIC_READ, LocalFree};
    use windows_sys::Win32::Security::Authorization::{GetSecurityInfo, SE_FILE_OBJECT};
    use windows_sys::Win32::Security::*;
    use windows_sys::Win32::Storage::FileSystem::{FILE_GENERIC_EXECUTE, FILE_GENERIC_READ};
    use windows_sys::Win32::System::SystemServices::{
        ACCESS_ALLOWED_ACE_TYPE, ACCESS_DENIED_ACE_TYPE,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    struct Descriptor(PSECURITY_DESCRIPTOR);
    impl Drop for Descriptor {
        fn drop(&mut self) {
            // SAFETY: GetSecurityInfo allocated this descriptor with LocalAlloc;
            // its ACL pointers are no longer used when this owner is dropped.
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    pub(super) fn check_acl(file: &File) -> Result<(), OutputError> {
        // SAFETY: file is borrowed for the whole operation; output pointers target
        // initialized stack storage. The returned descriptor is RAII-owned below.
        let mut acl = null_mut();
        let mut owner = null_mut();
        let mut descriptor = null_mut();
        let status = unsafe {
            GetSecurityInfo(
                file.as_raw_handle(),
                SE_FILE_OBJECT,
                DACL_SECURITY_INFORMATION | OWNER_SECURITY_INFORMATION,
                &mut owner,
                null_mut(),
                &mut acl,
                null_mut(),
                &mut descriptor,
            )
        };
        if status != 0 {
            return Err(OutputError::UnsafeTarget);
        }
        let _descriptor = Descriptor(descriptor);
        if acl.is_null() {
            return Err(OutputError::UnsafeTarget);
        }
        let mut token = null_mut();
        // SAFETY: pseudo process handle is valid; token receives an owned handle
        // and is closed by OwnedHandle even when later checks fail.
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(OutputError::UnsafeTarget);
        }
        let token = unsafe { OwnedHandle::from_raw_handle(token) };
        let mut needed = 0;
        // SAFETY: zero-length query only writes the required size.
        unsafe {
            GetTokenInformation(token.as_raw_handle(), TokenUser, null_mut(), 0, &mut needed);
        }
        if needed == 0 || needed > 65536 {
            return Err(OutputError::UnsafeTarget);
        }
        let mut user = vec![0u64; (needed as usize).div_ceil(8)];
        // SAFETY: aligned allocation contains at least needed bytes; successful
        // TOKEN_USER and its SID remain valid while user is alive.
        if unsafe {
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                user.as_mut_ptr().cast(),
                needed,
                &mut needed,
            )
        } == 0
        {
            return Err(OutputError::UnsafeTarget);
        }
        let user_sid = unsafe { (*user.as_ptr().cast::<TOKEN_USER>()).User.Sid };
        let mut allowed = Vec::new();
        for kind in [
            WinLocalSystemSid,
            WinBuiltinAdministratorsSid,
            WinCreatorOwnerSid,
            // OWNER RIGHTS applies to the object's owner, checked separately
            // below; it never makes an unrelated owner trusted.
            WinCreatorOwnerRightsSid,
        ] {
            let mut sid = vec![0u32; 17];
            let mut length = 68;
            // SAFETY: documented SECURITY_MAX_SID_SIZE allocation with DWORD
            // alignment; no domain SID is required for these well-known types.
            if unsafe { CreateWellKnownSid(kind, null_mut(), sid.as_mut_ptr().cast(), &mut length) }
                == 0
            {
                return Err(OutputError::UnsafeTarget);
            }
            allowed.push(sid);
        }
        // The owner can change a DACL even without an explicit allow ACE. An
        // unrelated owner therefore cannot make this a private grant by briefly
        // exposing only our SID. SYSTEM/Administrators are trusted platform actors.
        if owner.is_null() || unsafe { IsValidSid(owner) } == 0 {
            return Err(OutputError::UnsafeTarget);
        }
        let trusted_owner = unsafe { EqualSid(owner, user_sid) != 0 }
            || allowed[..2]
                .iter()
                .any(|known| unsafe { EqualSid(owner, known.as_ptr().cast_mut().cast()) != 0 });
        if !trusted_owner {
            return Err(OutputError::UnsafeTarget);
        }
        // SAFETY: kernel returned an ACL in its owned descriptor. IsValidAcl and
        // GetAce validate the records before any type-specific field is read.
        if unsafe { IsValidAcl(acl) } == 0 {
            return Err(OutputError::UnsafeTarget);
        }
        let count = unsafe { (*acl).AceCount };
        for index in 0..u32::from(count) {
            let mut ace = null_mut();
            if unsafe { GetAce(acl, index, &mut ace) } == 0 {
                return Err(OutputError::UnsafeTarget);
            }
            let header = unsafe { &*ace.cast::<ACE_HEADER>() };
            if u32::from(header.AceType) == ACCESS_DENIED_ACE_TYPE {
                continue;
            }
            if u32::from(header.AceType) != ACCESS_ALLOWED_ACE_TYPE || header.AceSize < 16 {
                return Err(OutputError::UnsafeTarget);
            }
            let entry = ace.cast::<ACCESS_ALLOWED_ACE>();
            let sid = unsafe { addr_of!((*entry).SidStart).cast_mut().cast() };
            if unsafe { IsValidSid(sid) } == 0 {
                return Err(OutputError::UnsafeTarget);
            }
            if unsafe { GetLengthSid(sid) } > u32::from(header.AceSize) - 8 {
                return Err(OutputError::UnsafeTarget);
            }
            // Both effective and inherit-only grants matter. Read/execute access
            // cannot mutate the authorized tree. Existing read-only sharing is
            // preserved, including on newly inherited output; it is not a promise
            // of confidentiality. Generic read/execute have the same bounded
            // mapping for file objects; write, delete, ACL and unknown bits fail.
            let trusted = unsafe { EqualSid(sid, user_sid) != 0 }
                || allowed
                    .iter()
                    .any(|known| unsafe { EqualSid(sid, known.as_ptr().cast_mut().cast()) != 0 });
            let read_only =
                FILE_GENERIC_READ | FILE_GENERIC_EXECUTE | GENERIC_READ | GENERIC_EXECUTE;
            if !trusted && unsafe { (*entry).Mask } & !read_only != 0 {
                return Err(OutputError::UnsafeTarget);
            }
        }
        Ok(())
    }
}
