use super::*;

fn opaque_service(flags: u16, packed_high: u32) -> Vec<u8> {
    let mut fields = 3u32.to_le_bytes().to_vec();
    fields.extend(7u32.to_le_bytes());
    fields.push(2);
    fields.extend(crc32(b"aux").to_le_bytes());
    fields.extend(0x58210000u32.to_le_bytes());
    fields.extend([29, 0x33]);
    fields.extend(2u16.to_le_bytes());
    fields.extend(0x20u32.to_le_bytes());
    if flags & 0x100 != 0 {
        fields.extend(packed_high.to_le_bytes());
        fields.extend(1u32.to_le_bytes());
    }
    fields.extend(b"RR");
    // Deliberately uninterpretable as extended-time or cipher parameters.
    fields.extend([0xff; 17]);
    let mut record = old_header(0x7a, flags | 0x8000, &fields);
    record.extend(b"aux");
    record
}

#[test]
fn service_large_and_time_tails_never_become_public_files_or_file_times() {
    for flags in [0, 0x100, 0x1000, 0x1100, 0x400, 0x500, 0x1400, 0x1500] {
        let service = opaque_service(flags, 0);
        let archive = old_archive(&[
            old_file(b"before", b"abc", 0, 20),
            service.clone(),
            old_file(b"after", b"def", 0, 20),
        ]);
        let rows = scan(archive.clone()).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.size_known && row.size == 3));
        assert!(rows.iter().all(|row| row.modified_dos == 0x58210000));
        assert_eq!(unpack(archive), [b"abc", b"def"]);
        let mut bad_header = service.clone();
        let end = usize::from(u16::from_le_bytes(bad_header[5..7].try_into().unwrap()));
        bad_header[end - 1] ^= 1;
        assert_eq!(scan(old_archive(&[bad_header])), Err(ScanError::Invalid));
        // Opaque service contents are not claimed to be authenticated.
        let mut ignored_content = service;
        ignored_content[end] ^= 1;
        assert_eq!(
            unpack(old_archive(&[
                ignored_content,
                old_file(b"only", b"abc", 0, 20)
            ])),
            [b"abc"]
        );
    }
}

#[test]
fn skipped_service_ranges_and_required_large_fields_remain_bounded() {
    assert_eq!(
        scan(old_archive(&[opaque_service(0x100, 1)])),
        Err(ScanError::Invalid)
    );
    let mut short = opaque_service(0x100, 0);
    short.truncate(36);
    short[5..7].copy_from_slice(&36u16.to_le_bytes());
    let checksum = crc32(&short[2..]) as u16;
    short[..2].copy_from_slice(&checksum.to_le_bytes());
    assert_eq!(scan(old_archive(&[short])), Err(ScanError::Invalid));
    let mut bad_name = opaque_service(0x1100, 0);
    bad_name[26..28].copy_from_slice(&1024u16.to_le_bytes());
    let end = usize::from(u16::from_le_bytes(bad_name[5..7].try_into().unwrap()));
    let checksum = crc32(&bad_name[2..end]) as u16;
    bad_name[..2].copy_from_slice(&checksum.to_le_bytes());
    assert_eq!(scan(old_archive(&[bad_name])), Err(ScanError::Invalid));
}

fn comment(bytes: &[u8]) -> Vec<u8> {
    let mut fields = (bytes.len() as u16).to_le_bytes().to_vec();
    fields.extend([20, 0x30]);
    fields.extend((crc32(bytes) as u16).to_le_bytes());
    fields.extend(bytes);
    let mut out = old_header(0x75, 0, &fields);
    let crc = crc32(&out[2..13]) as u16;
    out[..2].copy_from_slice(&crc.to_le_bytes());
    out
}
fn main_with_comment(bytes: &[u8]) -> Vec<u8> {
    let mut fields = vec![0; 6];
    fields.extend(comment(bytes));
    let mut out = old_header(0x73, 2, &fields);
    let crc = crc32(&out[2..13]) as u16;
    out[..2].copy_from_slice(&crc.to_le_bytes());
    out
}
fn file_with_comment(bytes: &[u8]) -> Vec<u8> {
    let original = old_file(b"entry", b"abc", 8, 20);
    let before = usize::from(u16::from_le_bytes(original[5..7].try_into().unwrap()));
    let mut out = original[..before].to_vec();
    out[5..7].copy_from_slice(&((before + 13 + bytes.len()) as u16).to_le_bytes());
    let crc = crc32(&out[2..]) as u16;
    out[..2].copy_from_slice(&crc.to_le_bytes());
    out.extend(comment(bytes));
    out.extend(&original[before..]);
    out
}

#[test]
fn embedded_and_standalone_comments_keep_independent_header_crc_ranges() {
    let mut archive = b"Rar!\x1a\x07\x00".to_vec();
    archive.extend(main_with_comment(b"main text"));
    archive.extend(comment(b"standalone"));
    archive.extend(file_with_comment(b"file comment"));
    archive.extend(old_header(0x7b, 0, &[]));
    assert_eq!(unpack(archive.clone()), [b"abc"]);
    // Comment payload is deliberately skipped; no comment content integrity
    // is advertised. The ordinary file's own final CRC remains mandatory.
    let at = archive
        .windows(12)
        .position(|w| w == b"file comment")
        .unwrap();
    archive[at] ^= 1;
    assert_eq!(unpack(archive.clone()), [b"abc"]);
    archive[at - 13] ^= 1;
    assert_eq!(scan(archive), Err(ScanError::Invalid));
}

#[test]
fn historical_crc_exemptions_never_cover_a_different_old_service_or_file() {
    let mut ignored = Vec::new();
    for kind in [0x76, 0x79, 0x77] {
        let mut fields = 3u32.to_le_bytes().to_vec();
        if kind == 0x77 {
            fields.extend(0x0101u16.to_le_bytes());
            fields.push(0);
        }
        let mut record = old_header(kind, 0x8000, &fields);
        record[0] ^= 1;
        record.extend(b"aux");
        ignored.push(record);
    }
    let mut records = ignored.clone();
    records.push(old_file(b"only", b"abc", 0, 20));
    assert_eq!(unpack(old_archive(&records)), [b"abc"]);
    ignored[2][11..13].copy_from_slice(&0x0100u16.to_le_bytes());
    assert_eq!(scan(old_archive(&ignored)), Err(ScanError::Invalid));
    let mut ordinary = old_file(b"file", b"abc", 0, 20);
    ordinary[0] ^= 1;
    assert_eq!(scan(old_archive(&[ordinary])), Err(ScanError::Invalid));
}

#[test]
fn malformed_comment_sizes_and_auxiliary_data_ranges_never_publish_partial_rows() {
    let mut record = file_with_comment(b"text");
    let child = 32 + b"entry".len();
    record[child + 5..child + 7].copy_from_slice(&100u16.to_le_bytes());
    assert_eq!(scan(old_archive(&[record])), Err(ScanError::Invalid));
    let mut comment = comment(b"x");
    comment.truncate(12);
    assert_eq!(scan(old_archive(&[comment])), Err(ScanError::Invalid));
    let mut auxiliary = old_header(0x76, 0x8000, &999u32.to_le_bytes());
    auxiliary.extend(b"short");
    assert_eq!(scan(old_archive(&[auxiliary])), Err(ScanError::Invalid));
}

#[test]
fn comment_parent_crc_includes_salt_and_processed_times_but_not_archive_time_mode() {
    let original = old_file(b"entry", b"abc", 0x1408, 29);
    let before = usize::from(u16::from_le_bytes(original[5..7].try_into().unwrap()));
    let mut record = original[..before].to_vec();
    record.extend([7; 8]);
    // mtime with one fraction byte, ignored archive-time mode. The latter
    // does not consume a fabricated DOS field from the following CMT header.
    record.extend(0x9008u16.to_le_bytes());
    record.push(0x45);
    let processed = record.len();
    let child = comment(b"ignored");
    record[5..7].copy_from_slice(&((processed + child.len()) as u16).to_le_bytes());
    let checksum = crc32(&record[2..]) as u16;
    record[..2].copy_from_slice(&checksum.to_le_bytes());
    record.extend(child);
    record.extend(&original[before..]);
    assert_eq!(unpack(old_archive(&[record.clone()])), [b"abc"]);
    record[before + 3] ^= 1;
    assert_eq!(scan(old_archive(&[record])), Err(ScanError::Invalid));
}

#[test]
fn rev_crc_exception_uses_only_the_current_end_physical_tail() {
    let ordinary = old_file(b"entry", b"abc", 0, 20);
    for (flags, tail, damage, success) in [
        (4, vec![0; 7], true, true),
        (4, vec![0; 6], true, true),
        (4, vec![0; 5], true, false),
        (0, vec![0; 7], true, false),
        (4, vec![1; 7], true, false),
        (4, vec![1; 3], false, true),
        (0, vec![2; 3], false, true),
    ] {
        let mut end = old_header(0x7b, flags, &tail);
        if damage {
            end[0] ^= 1;
        }
        let mut archive = b"Rar!\x1a\x07\x00".to_vec();
        archive.extend(old_header(0x73, 0, &[0; 6]));
        archive.extend(&ordinary);
        archive.extend(end);
        if success {
            assert_eq!(unpack(archive), [b"abc"]);
        } else {
            assert_eq!(scan(archive), Err(ScanError::Invalid));
        }
    }
}
