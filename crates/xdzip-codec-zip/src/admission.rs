//! One-volume admission using the parser's validated record locations.
use rawzip::{ReaderAt, ZipArchive, ZipFileHeaderRecord};
use xdzip_codec_api::ScanError;

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().expect("fixed header"))
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("fixed header"))
}
fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().expect("fixed header"))
}

pub(super) fn single_volume<R: ReaderAt>(archive: &ZipArchive<R>) -> Result<(), ScanError> {
    let source = archive.get_ref();
    let end_at = archive.eocd_offset();
    let mut end = [0; 22];
    source
        .read_exact_at(&mut end, end_at)
        .map_err(|_| ScanError::Invalid)?;
    if u16_at(&end, 4) != 0 || u16_at(&end, 6) != 0 {
        return Err(ScanError::Unsupported);
    }
    let mut locator = [0; 20];
    let locator_at = end_at.checked_sub(20);
    let has_locator = locator_at.is_some_and(|at| {
        source.read_exact_at(&mut locator, at).is_ok() && &locator[..4] == b"PK\x06\x07"
    });
    if !has_locator {
        if u16_at(&end, 8) != u16_at(&end, 10) {
            return Err(ScanError::Unsupported);
        }
        // 65,535 records also fit classic ZIP. The scanner checks the actual
        // count; only overflowing directory size/offset require a ZIP64 footer.
        if u32_at(&end, 12) == u32::MAX || u32_at(&end, 16) == u32::MAX {
            return Err(ScanError::Invalid);
        }
        return Ok(());
    }
    if u32_at(&locator, 4) != 0 || u32_at(&locator, 16) != 1 {
        return Err(ScanError::Unsupported);
    }
    let offset = u64_at(&locator, 8);
    let mut zip64 = [0; 56];
    source
        .read_exact_at(&mut zip64, offset)
        .map_err(|_| ScanError::Invalid)?;
    if &zip64[..4] != b"PK\x06\x06"
        || u64_at(&zip64, 4) < 44
        || offset
            .checked_add(12)
            .and_then(|at| at.checked_add(u64_at(&zip64, 4)))
            != locator_at
        || u64_at(&zip64, 32) != archive.entries_hint()
    {
        return Err(ScanError::Invalid);
    }
    if u32_at(&zip64, 16) != 0
        || u32_at(&zip64, 20) != 0
        || u64_at(&zip64, 24) != u64_at(&zip64, 32)
    {
        return Err(ScanError::Unsupported);
    }
    Ok(())
}

pub(super) fn entry_disk<R: ReaderAt>(
    archive: &ZipArchive<R>,
    entry: &ZipFileHeaderRecord<'_>,
) -> Result<(), ScanError> {
    // ZIP64's disk value follows only the fields whose classic values are sentinels.
    let mut fixed = [0; 46];
    archive
        .get_ref()
        .read_exact_at(&mut fixed, entry.central_directory_offset())
        .map_err(|_| ScanError::Invalid)?;
    let disk = u16_at(&fixed, 34);
    let disk = if disk == u16::MAX {
        let data = entry
            .extra_fields()
            .find(|(id, _)| id.as_u16() == 1)
            .map(|(_, data)| data)
            .ok_or(ScanError::Invalid)?;
        let offset = [24, 20, 42]
            .into_iter()
            .filter(|&at| u32_at(&fixed, at) == u32::MAX)
            .count()
            * 8;
        let value = data.get(offset..offset + 4).ok_or(ScanError::Invalid)?;
        u32::from_le_bytes(value.try_into().expect("checked disk field"))
    } else {
        u32::from(disk)
    };
    if disk == 0 {
        Ok(())
    } else {
        Err(ScanError::Unsupported)
    }
}
