//! CAB-specific admission on the same borrowed stream and worker callbacks.
use super::{Source, decode_rows, ffi, scan_wire};
use xdzip_codec_api::{Cancellation, ScanLimits};
use xdzip_codec_cab::CabCodec;

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    operation: u8,
    length: u64,
    selected: u64,
    entries: u32,
    name_bytes: u32,
    volumes: &[u64],
    names: &[u8],
) -> Result<(), u8> {
    if !matches!(operation, 0 | 1 | 3) {
        return Err(3);
    }
    if volumes.is_empty()
        || volumes.len() > 1024
        || volumes
            .iter()
            .try_fold(0u64, |sum, part| sum.checked_add(*part))
            != Some(length)
        || names.len() > 1024 * (4096 + 4)
    {
        return Err(2);
    }
    let mut volume_names = Vec::with_capacity(volumes.len());
    let mut at = 0usize;
    for _ in volumes {
        let end = at.checked_add(4).ok_or(2)?;
        let width =
            u32::from_le_bytes(names.get(at..end).ok_or(2)?.try_into().map_err(|_| 2)?) as usize;
        at = end;
        if width == 0 || width > 4096 {
            return Err(2);
        }
        let end = at.checked_add(width).ok_or(2)?;
        let raw = names.get(at..end).ok_or(2)?;
        if raw.contains(&0) {
            return Err(2);
        }
        volume_names.push(raw.to_vec());
        at = end;
    }
    if at != names.len() {
        return Err(2);
    }
    let mut source = Source {
        length,
        position: 0,
        volumes: volumes.to_vec(),
        volume_names,
    };
    let control = Cancellation::default();
    let limits = ScanLimits {
        entries: entries as usize,
        name_bytes: name_bytes as usize,
        ..Default::default()
    };
    let codec = CabCodec;
    let (rows, declared_utf8) = codec
        .scan_with_encoding_flags(&mut source, &control, limits)
        .map_err(scan_wire)?;
    if rows.len() != declared_utf8.len() {
        return Err(2);
    }
    if operation == 0 {
        for (index, (row, utf8)) in rows.iter().zip(declared_utf8).enumerate() {
            let flags = 0x10
                | if utf8 { 0x20 } else { 0 }
                | u8::from(row.directory)
                | (u8::from(row.link) << 2)
                | (u8::from(row.warning) << 3);
            let error = ffi::emit_row(
                index as u64,
                flags,
                row.size,
                row.packed_size,
                row.modified_dos,
                row.method,
                &row.raw_name,
            );
            if error != 0 {
                return Err(error);
            }
        }
        return Ok(());
    }
    decode_rows(operation, selected, &codec, &mut source, &rows, &control)
}
