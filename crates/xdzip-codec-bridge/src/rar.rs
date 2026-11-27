//! RAR read-only admission on borrowed, host-authorized streams. No filesystem
//! or output path crosses this module; the codec owns format interpretation.
use super::{Source, decode_rows_with_password, scan_wire};
use xdzip_codec_api::{Cancellation, ScanLimits};
use xdzip_codec_rar::RarCodec;

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    operation: u8,
    length: u64,
    selected: u64,
    entries: u32,
    name_bytes: u32,
    volumes: &[u64],
    comment_ids: &[u64],
    password: &str,
    has_password: bool,
) -> Result<(), u8> {
    if !matches!(operation, 0 | 1 | 3 | 4) {
        return Err(3);
    }
    if volumes.len() > 1024
        || (!volumes.is_empty()
            && volumes
                .iter()
                .try_fold(0u64, |sum, part| sum.checked_add(*part))
                != Some(length))
        || password.len() > 4096
    {
        return Err(2);
    }
    let mut source = Source {
        length,
        position: 0,
        volumes: volumes.to_vec(),
        volume_names: Vec::new(),
    };
    let control = Cancellation::default();
    let password = has_password.then_some(password);
    let codec = RarCodec;
    let mut index = xdzip_codec_rar::read_index(
        &mut source,
        &control,
        ScanLimits {
            entries: entries as usize,
            name_bytes: name_bytes as usize,
            ..Default::default()
        },
        password,
    )
    .map_err(scan_wire)?;
    if operation == 4 {
        let ids = comment_ids
            .iter()
            .map(|id| id.checked_add(1).ok_or(2))
            .collect::<Result<Vec<_>, _>>()?;
        for (id, comment) in index
            .entry_comments(&mut source, &ids, &control, password)
            .map_err(scan_wire)?
        {
            let position = usize::try_from(id - 1).map_err(|_| 2)?;
            let mut row = index.entries().get(position).ok_or(2)?.clone();
            row.details.comment = comment;
            row.details.comment_region = None;
            let name = if std::str::from_utf8(row.name_bytes()).is_ok() {
                row.name_bytes()
            } else {
                row.display_name.as_bytes()
            };
            let error = crate::emit_metadata(id - 1, 0, &row, name);
            if error != 0 {
                return Err(error);
            }
        }
        return Ok(());
    }
    let rows = index.entries();
    if operation == 0 {
        for (index, row) in rows.iter().enumerate() {
            // The existing host ROW namespace is UTF-8 for RAR. The complete
            // original record/name identity stays in the Rust codec and is
            // rechecked before payload decoding. Unresolved names stay unsafe.
            let effective = row.name_bytes();
            let valid_utf8 = std::str::from_utf8(effective).is_ok();
            let name = if valid_utf8 {
                effective
            } else {
                row.display_name.as_bytes()
            };
            let flags = u8::from(row.directory)
                | (u8::from(row.encrypted) << 1)
                | (u8::from(row.link) << 2)
                | (u8::from(row.warning || !valid_utf8) << 3)
                | (u8::from(!row.size_known) << 6)
                | (u8::from(row.details.comment_region.is_some()) << 7);
            let error = crate::emit_metadata(index as u64, flags, row, name);
            if error != 0 {
                return Err(error);
            }
        }
        return Ok(());
    }
    decode_rows_with_password(
        operation,
        selected,
        &codec,
        &mut source,
        rows,
        &control,
        password,
    )
}
