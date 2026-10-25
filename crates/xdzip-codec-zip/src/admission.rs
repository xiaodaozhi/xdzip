// crates/xdzip-codec-zip/src/admission.rs

//! Host-normalized streams must contain single-volume ZIP metadata.
use xdzip_codec_api::ScanError;

pub(super) fn single_volume(
    disk: u32,
    central_disk: u32,
    on_disk: u64,
    total: u64,
) -> Result<(), ScanError> {
    if disk != 0 || central_disk != 0 || on_disk != total {
        Err(ScanError::Unsupported)
    } else {
        Ok(())
    }
}
pub(super) fn entry_disk(disk: u32) -> Result<(), ScanError> {
    if disk == 0 {
        Ok(())
    } else {
        Err(ScanError::Unsupported)
    }
}
