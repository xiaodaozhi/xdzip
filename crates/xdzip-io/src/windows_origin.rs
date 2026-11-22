//! Windows attachment origin on host-approved files and private staging handles.

use crate::FileIdentity;
use std::ffi::OsString;
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::os::windows::ffi::OsStringExt;
use std::os::windows::fs::OpenOptionsExt;
use std::os::windows::io::AsRawHandle;
use std::ptr::{null, null_mut};
use windows_sys::Win32::Foundation::{
    ERROR_FILE_NOT_FOUND, ERROR_INVALID_FUNCTION, ERROR_INVALID_NAME, ERROR_NOT_SUPPORTED,
};
use windows_sys::Win32::Storage::FileSystem::{
    BACKUP_ALTERNATE_DATA, BackupWrite, FILE_FLAG_OPEN_REPARSE_POINT, FILE_NAME_OPENED,
    GetFinalPathNameByHandleW,
};

const MARKER: &str = ":Zone.Identifier:$DATA";
const MAX_MARKER_BYTES: u64 = 4096;
const MAX_PATH_UNITS: u32 = 32768;

/// Trust zone from the Windows marker attached to an archive file.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct OriginZone(u8);

impl OriginZone {
    /// Read the marker for an already open regular file. The reopened stream must belong to
    /// the same file before any data is read. Only the numeric zone crosses the host boundary.
    pub fn read(file: &File) -> io::Result<Option<Self>> {
        let identity = FileIdentity::read(file)?;
        let handle = file.as_raw_handle();
        // The live file handle remains borrowed throughout both calls. A zero-length null
        // buffer requests the required UTF-16 size. Failure leaves the source untrusted.
        let needed = unsafe { GetFinalPathNameByHandleW(handle, null_mut(), 0, FILE_NAME_OPENED) };
        if needed == 0 {
            return Err(io::Error::last_os_error());
        }
        if needed > MAX_PATH_UNITS {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let mut name = vec![0; needed as usize];
        // The writable array has exactly `needed` initialized UTF-16 slots. Reject an
        // incomplete name or a changed size before deriving the fixed marker stream name.
        let length = unsafe {
            GetFinalPathNameByHandleW(handle, name.as_mut_ptr(), needed, FILE_NAME_OPENED)
        };
        if length == 0 {
            return Err(io::Error::last_os_error());
        }
        if length >= needed {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let mut path = OsString::from_wide(&name[..length as usize]);
        path.push(MARKER);
        let stream = match OpenOptions::new()
            .read(true)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
            .open(path)
        {
            Ok(stream) => stream,
            Err(error)
                if matches!(
                    error.raw_os_error().map(|n| n as u32),
                    Some(
                        ERROR_FILE_NOT_FOUND
                            | ERROR_INVALID_FUNCTION
                            | ERROR_NOT_SUPPORTED
                            | ERROR_INVALID_NAME
                    )
                ) =>
            {
                return Ok(None);
            }
            Err(error) => return Err(error),
        };
        if FileIdentity::read(&stream)? != identity {
            return Err(io::ErrorKind::InvalidData.into());
        }
        let mut contents = Vec::new();
        stream
            .take(MAX_MARKER_BYTES + 1)
            .read_to_end(&mut contents)?;
        Ok(Some(Self::parse(&contents)))
    }

    fn parse(contents: &[u8]) -> Self {
        if contents.len() as u64 > MAX_MARKER_BYTES {
            return Self(3);
        }
        let mut in_zone_transfer = false;
        let mut zone = None;
        for line in contents
            .split(|&byte| byte == b'\n')
            .map(<[u8]>::trim_ascii)
        {
            if line.first() == Some(&b'[') {
                in_zone_transfer = line.eq_ignore_ascii_case(b"[ZoneTransfer]");
            } else if in_zone_transfer
                && let Some(equal) = line.iter().position(|&byte| byte == b'=')
                && line[..equal].trim_ascii().eq_ignore_ascii_case(b"ZoneId")
            {
                let candidate = match line[equal + 1..].trim_ascii() {
                    [digit @ b'0'..=b'4'] => Self(digit - b'0'),
                    _ => Self(3),
                };
                zone = zone.max(Some(candidate));
            }
        }
        zone.unwrap_or(Self(3))
    }

    pub(crate) fn write(self, file: &File) -> io::Result<()> {
        let marker = format!("[ZoneTransfer]\r\nZoneId={}\r\n", self.0);
        let name = MARKER
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect::<Vec<_>>();
        // WIN32_STREAM_ID serialized as its 20-byte header, UTF-16 stream name and data.
        let mut data = Vec::new();
        data.extend_from_slice(&BACKUP_ALTERNATE_DATA.to_le_bytes());
        data.extend_from_slice(&0_u32.to_le_bytes());
        data.extend_from_slice(&(marker.len() as u64).to_le_bytes());
        data.extend_from_slice(&(name.len() as u32).to_le_bytes());
        data.extend_from_slice(&name);
        data.extend_from_slice(marker.as_bytes());
        let handle = file.as_raw_handle();
        let mut context = null_mut();
        let mut written = 0;
        // Stage owns a synchronous read/write handle. The fixed buffer is live for the entire
        // call and contains only a named data stream, with security restoration disabled.
        let result = unsafe {
            BackupWrite(
                handle,
                data.as_ptr(),
                data.len() as u32,
                &mut written,
                0,
                0,
                &mut context,
            )
        };
        let failure = (result == 0).then(io::Error::last_os_error);
        // Release the context on both success and failure; the API ignores the null buffer
        // with abort set. A failed or partial write prevents publication of the private stage.
        let mut ignored = 0;
        let released = unsafe { BackupWrite(handle, null(), 0, &mut ignored, 1, 0, &mut context) };
        if let Some(error) = failure {
            return Err(error);
        }
        if released == 0 {
            return Err(io::Error::last_os_error());
        }
        if written != data.len() as u32 || Self::read(file)? != Some(self) {
            return Err(io::ErrorKind::Unsupported.into());
        }
        Ok(())
    }
}
