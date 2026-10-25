// crates/xdzip-codec-zip/src/rewrite.rs

//! Ordinary ZIP updates retain compressed bytes and opaque metadata, never encryption state.
use crate::{
    ZipCodec,
    record::{read_at, read_directory},
    writer::{ArchiveWriter, check, filter_extra, scan_error, settings_valid},
};
use std::io::{SeekFrom, Write};
use xdzip_codec_api::{Cancellation, MetadataCodec, ReadSource, ScanLimits, encode::*};

fn payload_error(error: xdzip_codec_api::payload::DecodeError) -> EncodeError {
    use xdzip_codec_api::payload::DecodeError;
    match error {
        DecodeError::Cancelled => EncodeError::Cancelled,
        DecodeError::Limit => EncodeError::Limit,
        DecodeError::SourceIo => EncodeError::Source,
        DecodeError::Unsupported | DecodeError::PasswordRequired => EncodeError::Unsupported,
        _ => EncodeError::Invalid,
    }
}

impl RewriteCodec for ZipCodec {
    fn rewrite(
        &self,
        original: &mut dyn ReadSource,
        input: RewriteInput<'_>,
        output: &mut dyn RewriteOutput,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        settings_valid(input.settings)?;
        if input.settings.password.is_some()
            || original
                .volume_lengths()
                .is_some_and(|volumes| volumes.len() != 1)
        {
            return Err(EncodeError::Unsupported);
        }
        check(cancel)?;
        let snapshot = self
            .scan(original, cancel, ScanLimits::default())
            .map_err(scan_error)?;
        let directory =
            read_directory(original, cancel, ScanLimits::default()).map_err(scan_error)?;
        if directory.records.len() != input.keep.len()
            || directory.base_offset != 0
            || (!input.renames.is_empty() && input.renames.len() != input.keep.len())
            || snapshot.len() != directory.records.len()
        {
            return Err(EncodeError::Unsupported);
        }
        let mut central_end = directory.central_offset;
        for record in &directory.records {
            if record.central_offset != central_end {
                return Err(EncodeError::Invalid);
            }
            central_end = central_end
                .checked_add(
                    46 + record.name.len() as u64
                        + record.extra.len() as u64
                        + record.comment.len() as u64,
                )
                .ok_or(EncodeError::Limit)?;
        }
        if directory.central_offset.checked_add(directory.central_size) != Some(central_end)
            || !matches!(directory.end_offset.checked_sub(central_end), Some(0 | 76))
            || directory
                .end_offset
                .checked_add(22 + directory.comment.len() as u64)
                != Some(directory.source_length)
        {
            // A central digital signature would be invalidated by a rewrite; do not silently discard it.
            return Err(EncodeError::Unsupported);
        }
        let mut offsets: Vec<_> = directory
            .records
            .iter()
            .map(|record| record.local_offset)
            .collect();
        offsets.sort_unstable();
        // With no local records, the absolute directory start still exposes any prefix.
        if offsets.first().copied().unwrap_or(directory.central_offset) != 0
            || offsets.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(EncodeError::Unsupported);
        }
        // Reject the entire encrypted/special archive, even when an affected record would be removed.
        for (index, (record, entry)) in directory.records.iter().zip(&snapshot).enumerate() {
            check(cancel)?;
            if record.encrypted()
                || record.method == 99
                || filter_extra(&record.extra, &[0x9901])? != record.extra
                || record.flags & !0x080e != 0
                || !matches!(record.method, 0 | 8)
                || entry.warning
                || entry.link
                || (entry.directory && entry.size != 0)
            {
                return Err(EncodeError::Unsupported);
            }
            if let Some(Some(name)) = input.renames.get(index)
                && (name.is_empty()
                    || name.len() > 4096
                    || std::str::from_utf8(&record.comment).is_err())
            {
                // EFS declares both name and comment UTF-8. Preserve opaque legacy comments by refusing this edit.
                return Err(EncodeError::Unsupported);
            }
        }
        // Both streams are host-owned staging/source handles. This codec does not replace the original.
        let mut writer = ArchiveWriter::new(output);
        let mut buffer = [0; 64 * 1024];
        for (index, mut record) in directory.records.into_iter().enumerate() {
            check(cancel)?;
            let physical = offsets
                .binary_search(&record.local_offset)
                .map_err(|_| EncodeError::Invalid)?;
            let boundary = offsets
                .get(physical + 1)
                .copied()
                .unwrap_or(directory.central_offset);
            let local = crate::payload::local_record(
                original,
                &record,
                snapshot[index].name_bytes(),
                boundary,
                cancel,
            )
            .map_err(payload_error)?;
            if local.data_end < local.data_start || local.end != boundary {
                // Unknown inter-record material may carry metadata that cannot safely be relocated.
                return Err(EncodeError::Unsupported);
            }
            if !input.keep[index] {
                continue;
            }
            let mut fixed = [0; 30];
            read_at(original, record.local_offset, &mut fixed).map_err(scan_error)?;
            let mut local_extra = local.extra;
            if let Some(Some(name)) = input.renames.get(index) {
                let name = if record.is_dir() && !name.ends_with('/') {
                    format!("{name}/")
                } else {
                    name.clone()
                };
                if name.len() > 4096 {
                    return Err(EncodeError::Limit);
                }
                record.name = name.into_bytes();
                record.flags |= 0x0800;
                record.extra = filter_extra(&record.extra, &[0x7075])?;
                local_extra = filter_extra(&local_extra, &[0x7075])?;
                fixed[6..8].copy_from_slice(&record.flags.to_le_bytes());
                fixed[26..28].copy_from_slice(&(record.name.len() as u16).to_le_bytes());
                fixed[28..30].copy_from_slice(&(local_extra.len() as u16).to_le_bytes());
            }
            record.local_offset = writer.position();
            writer.write_all(&fixed).map_err(|_| EncodeError::Output)?;
            writer
                .write_all(&record.name)
                .map_err(|_| EncodeError::Output)?;
            writer
                .write_all(&local_extra)
                .map_err(|_| EncodeError::Output)?;
            original
                .seek(SeekFrom::Start(local.data_start))
                .map_err(|_| EncodeError::Source)?;
            let mut remaining = local
                .end
                .checked_sub(local.data_start)
                .ok_or(EncodeError::Invalid)?;
            while remaining != 0 {
                check(cancel)?;
                let bound = remaining.min(buffer.len() as u64) as usize;
                original
                    .read_exact(&mut buffer[..bound])
                    .map_err(|_| EncodeError::Source)?;
                writer
                    .write_all(&buffer[..bound])
                    .map_err(|_| EncodeError::Output)?;
                remaining -= bound as u64;
            }
            let name = String::from_utf8_lossy(&record.name);
            progress(EncodeProgress {
                name: &name,
                input_bytes: 0,
            });
            writer.push(record)?;
        }
        writer.append(input.additions, input.settings, cancel, progress)?;
        writer.finish(&directory.comment, cancel)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Read};
    struct Empty;
    impl EncodeInput for Empty {
        fn entries(&self) -> &[EncodeEntry] {
            &[]
        }
        fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            Err(EncodeError::Invalid)
        }
    }
    #[test]
    fn raw_copy_preserves_compressed_payload_comments_time_mode_and_custom_metadata() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.set_comment("archive comment").unwrap();
        let mut options = zip::write::FullFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .last_modified_time(zip::DateTime::from_date_and_time(2020, 3, 4, 5, 6, 8).unwrap())
            .unix_permissions(0o755)
            .with_file_comment("entry comment");
        options.add_extra_field(0xf00d, b"opaque", false).unwrap();
        writer.start_file("original.txt", options).unwrap();
        writer.write_all(b"unchanged payload").unwrap();
        let original = writer.finish().unwrap().into_inner();
        for rename in [false, true] {
            let renames = [rename.then(|| "移动/renamed.txt".to_owned())];
            let mut output = Cursor::new(Vec::new());
            ZipCodec
                .rewrite(
                    &mut Cursor::new(original.clone()),
                    RewriteInput {
                        keep: &[true],
                        renames: &renames,
                        additions: &Empty,
                        settings: EncodeSettings {
                            solid: false,
                            encrypt_headers: false,
                            method: "deflate",
                            level: 6,
                            password: None,
                        },
                    },
                    &mut output,
                    &Cancellation::default(),
                    &mut |_| {},
                )
                .unwrap();
            let rewritten = output.into_inner();
            let mut old = zip::ZipArchive::new(Cursor::new(&original)).unwrap();
            let mut new = zip::ZipArchive::new(Cursor::new(&rewritten)).unwrap();
            assert_eq!(old.comment(), new.comment());
            let a = old.by_index_raw(0).unwrap();
            let b = new.by_index_raw(0).unwrap();
            assert_eq!(
                b.name().unwrap(),
                if rename {
                    "移动/renamed.txt"
                } else {
                    "original.txt"
                }
            );
            assert_eq!(a.comment(), b.comment());
            assert_eq!(a.last_modified(), b.last_modified());
            assert_eq!(a.unix_mode(), b.unix_mode());
            assert_eq!(a.extra_data(), b.extra_data());
            let a_start = a.data_start().unwrap() as usize;
            let b_start = b.data_start().unwrap() as usize;
            assert_eq!(
                &original[a_start..a_start + a.compressed_size() as usize],
                &rewritten[b_start..b_start + b.compressed_size() as usize]
            );
        }
    }

    fn settings() -> EncodeSettings<'static> {
        EncodeSettings {
            method: "deflate",
            level: 6,
            password: None,
            solid: false,
            encrypt_headers: false,
        }
    }
    fn rewrite(bytes: &[u8], renames: &[Option<String>]) -> Result<Vec<u8>, EncodeError> {
        let mut output = Cursor::new(Vec::new());
        ZipCodec.rewrite(
            &mut Cursor::new(bytes),
            RewriteInput {
                keep: &[true],
                renames,
                additions: &Empty,
                settings: settings(),
            },
            &mut output,
            &Cancellation::default(),
            &mut |_| {},
        )?;
        Ok(output.into_inner())
    }
    fn put16(bytes: &mut [u8], at: usize, value: u16) {
        bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn put32(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn put64(bytes: &mut [u8], at: usize, value: u64) {
        bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }
    fn empty_fixture(prefix: &[u8], absolute: bool, comment: &[u8]) -> Vec<u8> {
        let mut end = [0; 22];
        end[..4].copy_from_slice(b"PK\x05\x06");
        put32(&mut end, 16, if absolute { prefix.len() as u32 } else { 0 });
        put16(&mut end, 20, comment.len() as u16);
        let mut bytes = prefix.to_vec();
        bytes.extend_from_slice(&end);
        bytes.extend_from_slice(comment);
        bytes
    }
    struct Addition {
        entries: Vec<EncodeEntry>,
    }
    impl Addition {
        fn new() -> Self {
            Self {
                entries: vec![EncodeEntry {
                    name: "added.txt".into(),
                    directory: false,
                    size: 3,
                    modified_unix: 0,
                }],
            }
        }
    }
    impl EncodeInput for Addition {
        fn entries(&self) -> &[EncodeEntry] {
            &self.entries
        }
        fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            if index != 0 {
                return Err(EncodeError::Invalid);
            }
            Ok(Box::new(Cursor::new(b"abc")))
        }
    }
    #[test]
    fn empty_archives_preserve_comments_and_accept_additions() {
        let addition = Addition::new();
        for comment in [b"".as_slice(), b"archive comment"] {
            for additions in [&Empty as &dyn EncodeInput, &addition] {
                let mut output = Cursor::new(Vec::new());
                ZipCodec
                    .rewrite(
                        &mut Cursor::new(empty_fixture(b"", false, comment)),
                        RewriteInput {
                            keep: &[],
                            renames: &[],
                            additions,
                            settings: settings(),
                        },
                        &mut output,
                        &Cancellation::default(),
                        &mut |_| {},
                    )
                    .unwrap();
                let mut archive = zip::ZipArchive::new(Cursor::new(output.into_inner())).unwrap();
                assert_eq!(archive.comment(), comment);
                assert_eq!(archive.len(), additions.entries().len());
                if !additions.entries().is_empty() {
                    let mut file = archive.by_index(0).unwrap();
                    assert_eq!(file.name().unwrap(), "added.txt");
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes).unwrap();
                    assert_eq!(bytes, b"abc");
                }
            }
        }
    }
    #[test]
    fn empty_archives_with_relative_or_absolute_prefix_offsets_are_refused() {
        let addition = Addition::new();
        for prefix in [b"MZstub".as_slice(), b"opaque prefix"] {
            for absolute in [false, true] {
                for comment in [b"".as_slice(), b"archive comment"] {
                    let original = empty_fixture(prefix, absolute, comment);
                    let directory = read_directory(
                        &mut Cursor::new(&original),
                        &Cancellation::default(),
                        ScanLimits::default(),
                    )
                    .unwrap();
                    assert!(directory.records.is_empty());
                    assert_eq!(directory.central_offset, prefix.len() as u64);
                    assert_eq!(
                        directory.base_offset,
                        if absolute { 0 } else { prefix.len() as u64 }
                    );
                    for additions in [&Empty as &dyn EncodeInput, &addition] {
                        let mut output = Cursor::new(Vec::new());
                        assert_eq!(
                            ZipCodec.rewrite(
                                &mut Cursor::new(&original),
                                RewriteInput {
                                    keep: &[],
                                    renames: &[],
                                    additions,
                                    settings: settings(),
                                },
                                &mut output,
                                &Cancellation::default(),
                                &mut |_| {},
                            ),
                            Err(EncodeError::Unsupported)
                        );
                        assert!(output.into_inner().is_empty());
                    }
                }
            }
        }
    }
    // Owned APPNOTE byte fixture: different local/central extras, Unicode Path, optional signed ZIP64 descriptor.
    fn fixture(wide: bool, signed: bool, comment: &[u8]) -> Vec<u8> {
        let name = b"old.txt";
        let data = b"abc";
        let crc = xdzip_compression::crc32(data);
        let mut unicode = vec![0x75, 0x70];
        let logical = "旧名.txt".as_bytes();
        unicode.extend_from_slice(&((logical.len() + 5) as u16).to_le_bytes());
        unicode.push(1);
        unicode.extend_from_slice(&xdzip_compression::crc32(name).to_le_bytes());
        unicode.extend_from_slice(logical);
        let mut local_extra = vec![0x0d, 0xf0, 3, 0, b'L', b'O', b'C'];
        local_extra.extend_from_slice(&unicode);
        let mut central_extra = vec![0xef, 0xbe, 3, 0, b'C', b'E', b'N'];
        central_extra.extend_from_slice(&unicode);
        if wide {
            local_extra.extend_from_slice(&[1, 0, 16, 0]);
            local_extra.extend_from_slice(&[0; 16]);
            central_extra.extend_from_slice(&[1, 0, 16, 0]);
            central_extra.extend_from_slice(&3u64.to_le_bytes());
            central_extra.extend_from_slice(&3u64.to_le_bytes());
        }
        let mut local = [0; 30];
        local[..4].copy_from_slice(b"PK\x03\x04");
        put16(&mut local, 4, if wide { 45 } else { 20 });
        put16(&mut local, 6, 8);
        put32(&mut local, 10, 0x5064_28c4);
        if wide {
            put32(&mut local, 18, u32::MAX);
            put32(&mut local, 22, u32::MAX);
        }
        put16(&mut local, 26, name.len() as u16);
        put16(&mut local, 28, local_extra.len() as u16);
        let mut bytes = local.to_vec();
        bytes.extend_from_slice(name);
        bytes.extend_from_slice(&local_extra);
        bytes.extend_from_slice(data);
        if signed {
            bytes.extend_from_slice(b"PK\x07\x08");
        }
        bytes.extend_from_slice(&crc.to_le_bytes());
        if wide {
            bytes.extend_from_slice(&3u64.to_le_bytes());
            bytes.extend_from_slice(&3u64.to_le_bytes());
        } else {
            bytes.extend_from_slice(&3u32.to_le_bytes());
            bytes.extend_from_slice(&3u32.to_le_bytes());
        }
        let central_at = bytes.len();
        let mut central = [0; 46];
        central[..4].copy_from_slice(b"PK\x01\x02");
        put16(&mut central, 4, 0x033f);
        put16(&mut central, 6, if wide { 45 } else { 20 });
        put16(&mut central, 8, 8);
        put32(&mut central, 12, 0x5064_28c4);
        put32(&mut central, 16, crc);
        put32(&mut central, 20, if wide { u32::MAX } else { 3 });
        put32(&mut central, 24, if wide { u32::MAX } else { 3 });
        put16(&mut central, 28, name.len() as u16);
        put16(&mut central, 30, central_extra.len() as u16);
        put16(&mut central, 32, comment.len() as u16);
        put16(&mut central, 36, 1);
        put32(&mut central, 38, 0o100754 << 16);
        bytes.extend_from_slice(&central);
        bytes.extend_from_slice(name);
        bytes.extend_from_slice(&central_extra);
        bytes.extend_from_slice(comment);
        let size = bytes.len() - central_at;
        if wide {
            let mut footer = crate::zip64_fixture::archive(0);
            put64(&mut footer, 24, 1);
            put64(&mut footer, 32, 1);
            put64(&mut footer, 40, size as u64);
            put64(&mut footer, 48, central_at as u64);
            put64(&mut footer, 64, bytes.len() as u64);
            bytes.extend_from_slice(&footer);
        } else {
            let mut end = [0; 22];
            end[..4].copy_from_slice(b"PK\x05\x06");
            put16(&mut end, 8, 1);
            put16(&mut end, 10, 1);
            put32(&mut end, 12, size as u32);
            put32(&mut end, 16, central_at as u32);
            bytes.extend_from_slice(&end);
        }
        bytes
    }
    #[test]
    fn raw_copy_retains_local_opaque_fields_descriptors_and_clears_stale_unicode_path_on_rename() {
        for wide in [false, true] {
            for signed in [false, true] {
                let old = fixture(wide, signed, b"entry comment");
                for rename in [false, true] {
                    let renames = [rename.then(|| "移动/new.txt".to_owned())];
                    let new = rewrite(&old, &renames).unwrap();
                    let cancel = Cancellation::default();
                    let a = read_directory(&mut Cursor::new(&old), &cancel, ScanLimits::default())
                        .unwrap();
                    let b = read_directory(&mut Cursor::new(&new), &cancel, ScanLimits::default())
                        .unwrap();
                    let ar = &a.records[0];
                    let br = &b.records[0];
                    assert_eq!(ar.comment, br.comment);
                    assert_eq!(ar.modified_dos, br.modified_dos);
                    assert_eq!(ar.version_made_by, br.version_made_by);
                    assert_eq!(ar.external_attributes, br.external_attributes);
                    assert_eq!(ar.internal_attributes, br.internal_attributes);
                    let al = crate::payload::local_record(
                        &mut Cursor::new(&old),
                        ar,
                        "旧名.txt".as_bytes(),
                        a.central_offset,
                        &cancel,
                    )
                    .unwrap();
                    let effective = if rename {
                        "移动/new.txt"
                    } else {
                        "旧名.txt"
                    };
                    let bl = crate::payload::local_record(
                        &mut Cursor::new(&new),
                        br,
                        effective.as_bytes(),
                        b.central_offset,
                        &cancel,
                    )
                    .unwrap();
                    assert_eq!(
                        &old[al.data_start as usize..al.end as usize],
                        &new[bl.data_start as usize..bl.end as usize]
                    );
                    if rename {
                        assert_eq!(br.flags & 0x800, 0x800);
                        assert_eq!(filter_extra(&al.extra, &[0x7075]).unwrap(), bl.extra);
                        assert_eq!(
                            filter_extra(&ar.extra, &[0x7075, 1]).unwrap(),
                            filter_extra(&br.extra, &[1]).unwrap()
                        );
                    } else {
                        assert_eq!(ar.flags, br.flags);
                        assert_eq!(al.extra, bl.extra);
                        assert_eq!(ar.extra, br.extra);
                    }
                    let mut archive = zip::ZipArchive::new(Cursor::new(new)).unwrap();
                    let mut file = archive.by_index(0).unwrap();
                    let mut body = Vec::new();
                    file.read_to_end(&mut body).unwrap();
                    assert_eq!(body, b"abc");
                }
            }
        }
    }
    #[test]
    fn legacy_comment_is_preserved_without_efs_and_refuses_utf8_rename() {
        let original = fixture(false, true, &[0x82, 0x80]);
        let bytes = rewrite(&original, &[]).unwrap();
        let directory = read_directory(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap();
        assert_eq!(directory.records[0].comment, &[0x82, 0x80]);
        assert_eq!(directory.records[0].flags & 0x800, 0);
        assert_eq!(
            rewrite(&original, &[Some("new.txt".into())]),
            Err(EncodeError::Unsupported)
        );
    }
    #[test]
    fn encrypted_removed_records_sfx_signatures_and_opaque_zip64_footers_are_refused() {
        let mut encrypted = fixture(false, true, b"");
        let central = encrypted
            .windows(4)
            .position(|v| v == b"PK\x01\x02")
            .unwrap();
        put16(&mut encrypted, 6, 9);
        put16(&mut encrypted, central + 8, 9);
        let mut output = Cursor::new(Vec::new());
        assert_eq!(
            ZipCodec.rewrite(
                &mut Cursor::new(encrypted),
                RewriteInput {
                    keep: &[false],
                    renames: &[],
                    additions: &Empty,
                    settings: settings(),
                },
                &mut output,
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Unsupported)
        );
        assert!(output.into_inner().is_empty());
        let mut sfx = b"MZstub".to_vec();
        sfx.extend_from_slice(&fixture(false, true, b""));
        assert_eq!(rewrite(&sfx, &[]), Err(EncodeError::Unsupported));
        let mut signature = fixture(false, true, b"");
        let end = signature.len() - 22;
        let size = u32::from_le_bytes(signature[end + 12..end + 16].try_into().unwrap());
        signature.splice(end..end, b"PK\x05\x05\x01\x00x".iter().copied());
        put32(&mut signature, end + 7 + 12, size + 7);
        assert_eq!(rewrite(&signature, &[]), Err(EncodeError::Unsupported));
        let mut extension = crate::zip64_fixture::archive(1);
        let zip64 = extension.len() - 98;
        extension.splice(zip64 + 56..zip64 + 56, b"opaque".iter().copied());
        put64(&mut extension, zip64 + 4, 50);
        assert_eq!(rewrite(&extension, &[]), Err(EncodeError::Unsupported));
    }
    #[test]
    fn corrupt_descriptor_and_unpreservable_gaps_never_produce_a_finished_rewrite() {
        for wide in [false, true] {
            let mut bytes = fixture(wide, true, b"");
            let central = bytes.windows(4).position(|v| v == b"PK\x01\x02").unwrap();
            bytes[central - if wide { 20 } else { 12 }] ^= 1;
            assert_eq!(rewrite(&bytes, &[]), Err(EncodeError::Invalid));
        }
        let mut bytes = fixture(false, true, b"");
        let end = bytes.len() - 22;
        let central = u32::from_le_bytes(bytes[end + 16..end + 20].try_into().unwrap());
        bytes.splice(
            central as usize..central as usize,
            b"opaque".iter().copied(),
        );
        put32(&mut bytes, end + 6 + 16, central + 6);
        assert_eq!(rewrite(&bytes, &[]), Err(EncodeError::Unsupported));
    }
    #[test]
    fn duplicate_physical_records_survive_and_cancellation_keeps_rewrite_unfinished() {
        let original = crate::zip64_fixture::archive_names(&["same.txt".into(), "same.txt".into()]);
        let mut output = Cursor::new(Vec::new());
        ZipCodec
            .rewrite(
                &mut Cursor::new(&original),
                RewriteInput {
                    keep: &[true, true],
                    renames: &[],
                    additions: &Empty,
                    settings: settings(),
                },
                &mut output,
                &Cancellation::default(),
                &mut |_| {},
            )
            .unwrap();
        let directory =
            read_directory(&mut output, &Cancellation::default(), ScanLimits::default()).unwrap();
        assert_eq!(directory.records.len(), 2);
        assert_eq!(directory.records[0].name, directory.records[1].name);
        assert!(directory.records[0].local_offset < directory.records[1].local_offset);
        let cancel = Cancellation::default();
        let mut output = Cursor::new(Vec::new());
        assert_eq!(
            ZipCodec.rewrite(
                &mut Cursor::new(original),
                RewriteInput {
                    keep: &[true, true],
                    renames: &[],
                    additions: &Empty,
                    settings: settings(),
                },
                &mut output,
                &cancel,
                &mut |_| cancel.cancel()
            ),
            Err(EncodeError::Cancelled)
        );
        assert!(!output.into_inner().windows(4).any(|v| v == b"PK\x05\x06"));
    }
}
