use super::*;

#[path = "../../../tests/support/rar_cmt_time.rs"]
mod rar_cmt_time;

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

fn service_comment(bytes: &[u8], child: bool) -> Vec<u8> {
    let mut record = old_file(b"CMT", bytes, if child { 0x10 } else { 0 }, 29);
    record[2] = 0x7a;
    let size = usize::from(u16::from_le_bytes(record[5..7].try_into().unwrap()));
    let checksum = crc32(&record[2..size]) as u16;
    record[..2].copy_from_slice(&checksum.to_le_bytes());
    record
}

#[test]
fn archive_comment_is_independent_of_inline_and_service_entry_comments() {
    let control = Cancellation::default();
    let file = file_with_comment(b"entry");
    let child = service_comment(b"child", true);
    let mut inline = b"Rar!\x1a\x07\x00".to_vec();
    inline.extend(main_with_comment("压缩包".as_bytes()));
    inline.extend(&file);
    inline.extend(&child);
    for (bytes, expected) in [
        (inline, Some("压缩包")),
        (
            old_archive(&[
                file.clone(),
                child.clone(),
                service_comment(b"archive", false),
            ]),
            Some("archive"),
        ),
        (old_archive(&[file, child]), None),
    ] {
        let mut source = Cursor::new(bytes);
        let mut index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
        assert_eq!(
            index
                .archive_comment(&mut source, &control)
                .unwrap()
                .as_deref(),
            expected
        );
    }
}

fn service_comment_with_time(
    bytes: &[u8],
    subdata: &[u8],
    salt: Option<[u8; 8]>,
    time: &[u8],
) -> Vec<u8> {
    let mut record = service_comment(bytes, true);
    let size = usize::from(u16::from_le_bytes(record[5..7].try_into().unwrap()));
    let mut tail = subdata.to_vec();
    if let Some(salt) = salt {
        tail.extend(salt);
    }
    tail.extend(time);
    record.splice(size..size, tail.iter().copied());
    let flags = u16::from_le_bytes(record[3..5].try_into().unwrap())
        | 0x1000
        | if salt.is_some() { 0x400 } else { 0 };
    record[3..5].copy_from_slice(&flags.to_le_bytes());
    let new_size = size + tail.len();
    record[5..7].copy_from_slice(&(new_size as u16).to_le_bytes());
    let checksum = crc32(&record[2..new_size]) as u16;
    record[..2].copy_from_slice(&checksum.to_le_bytes());
    record
}

#[test]
fn service_comments_with_extended_times_preserve_file_content_and_encoding() {
    let raw: Vec<u8> = "中文备注"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    for (salt, time) in [
        (None, vec![0, 0]),
        (Some([0x37; 8]), vec![0, 0xb0, 1, 2, 3]),
        (None, vec![0x80, 0x08, 0, 0, 0x21, 0x58, 0, 0, 0x21, 0x58]),
    ] {
        let mut child = service_comment_with_time(&raw, b"opaque-subdata", salt, &time);
        let header_size = usize::from(u16::from_le_bytes(child[5..7].try_into().unwrap()));
        child[28..32].copy_from_slice(&1u32.to_le_bytes());
        let checksum = crc32(&child[2..header_size]) as u16;
        child[..2].copy_from_slice(&checksum.to_le_bytes());
        let archive = old_archive(&[old_file(b"entry", b"file data", 0, 29), child]);
        let control = Cancellation::default();
        let mut index = read_index(
            &mut Cursor::new(&archive),
            &control,
            ScanLimits::default(),
            None,
        )
        .unwrap();
        assert_eq!(
            index
                .entry_comments(&mut Cursor::new(&archive), &[1], &control, None)
                .unwrap()[0]
                .1,
            "中文备注".as_bytes()
        );
        assert_eq!(index.entries()[0].modified_dos, 0x58210000);
        assert_eq!(unpack(archive), [b"file data"]);
    }
}

#[test]
fn damaged_comment_with_extended_time_remains_empty() {
    let child = service_comment_with_time(b"note", b"metadata", None, &[0, 0]);
    let mut archive = old_archive(&[old_file(b"entry", b"payload", 0, 29), child]);
    let at = archive
        .windows(4)
        .position(|bytes| bytes == b"note")
        .unwrap();
    archive[at] ^= 1;
    let control = Cancellation::default();
    let mut index = read_index(
        &mut Cursor::new(&archive),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert!(
        index
            .entry_comments(&mut Cursor::new(&archive), &[1], &control, None)
            .unwrap()[0]
            .1
            .is_empty()
    );
    assert_eq!(unpack(archive), [b"payload"]);
}

#[test]
fn encrypted_service_comment_with_extended_time_finds_verified_salt() {
    for subdata in [b"".as_slice(), b"opaque\x00\x00metadata".as_slice()] {
        let archive = rar_cmt_time::encrypted_comment(subdata);
        let control = Cancellation::default();
        let mut index = read_index(
            &mut Cursor::new(&archive),
            &control,
            ScanLimits::default(),
            None,
        )
        .unwrap();
        assert_eq!(
            index
                .entry_comments(&mut Cursor::new(&archive), &[1], &control, Some("password"))
                .unwrap()[0]
                .1,
            b"This is from b.txt"
        );
        assert!(
            index
                .entry_comments(&mut Cursor::new(&archive), &[1], &control, Some("wrong"))
                .unwrap()[0]
                .1
                .is_empty()
        );
        assert_eq!(unpack(archive), [b"This is from a.txt"]);
    }
    let mut longest_time = vec![0xbb, 0xbb];
    longest_time.extend([0x52; 24]);
    let archive = rar_cmt_time::encrypted_comment_with_time(b"subdata", Some(&longest_time));
    let control = Cancellation::default();
    let mut index = read_index(
        &mut Cursor::new(&archive),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(
        index
            .entry_comments(&mut Cursor::new(&archive), &[1], &control, Some("password"))
            .unwrap()[0]
            .1,
        b"This is from b.txt"
    );
    let mut archive = rar_cmt_time::encrypted_comment(b"opaque\x00\x00metadata");
    let data_at = 80 + usize::from(u16::from_le_bytes(archive[85..87].try_into().unwrap()));
    archive[data_at] ^= 1;
    let control = Cancellation::default();
    let mut index = read_index(
        &mut Cursor::new(&archive),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert!(
        index
            .entry_comments(&mut Cursor::new(&archive), &[1], &control, Some("password"))
            .unwrap()[0]
            .1
            .is_empty()
    );
    assert_eq!(unpack(archive.clone()), [b"This is from a.txt"]);
    control.cancel();
    assert_eq!(
        index.entry_comments(&mut Cursor::new(&archive), &[1], &control, Some("password")),
        Err(ScanError::Cancelled)
    );
}

#[test]
fn unicode_service_comment_is_decoded_as_utf16() {
    let raw: Vec<u8> = "中文备注"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut child = service_comment(&raw, true);
    let old_size = usize::from(u16::from_le_bytes(child[5..7].try_into().unwrap()));
    child[28..32].copy_from_slice(&1u32.to_le_bytes());
    let checksum = crc32(&child[2..old_size]) as u16;
    child[..2].copy_from_slice(&checksum.to_le_bytes());
    let archive = old_archive(&[old_file(b"entry", b"abc", 0, 29), child]);
    let control = Cancellation::default();
    let mut index = read_index(
        &mut Cursor::new(&archive),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(
        index
            .entry_comments(&mut Cursor::new(&archive), &[1], &control, None)
            .unwrap()[0]
            .1,
        "中文备注".as_bytes()
    );
    assert_eq!(unpack(archive), [b"abc"]);
}

#[test]
fn service_comments_follow_physical_files_not_the_archive() {
    let archive = old_archive(&[
        service_comment(b"archive comment", false),
        old_file(b"duplicate", b"one", 0, 29),
        service_comment(b"first comment", true),
        old_file(b"duplicate", b"two", 0, 29),
        service_comment(b"second comment", true),
        old_file(b"plain", b"three", 0, 29),
        service_comment(b"another archive comment", false),
    ]);
    let control = Cancellation::default();
    let mut index = read_index(
        &mut Cursor::new(archive.clone()),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert!(index.entries()[0].details.comment_region.is_some());
    assert!(index.entries()[1].details.comment_region.is_some());
    assert!(index.entries()[2].details.comment_region.is_none());
    assert_eq!(
        index
            .entry_comments(
                &mut Cursor::new(archive.clone()),
                &[1, 2, 3],
                &control,
                None
            )
            .unwrap(),
        [
            (1, b"first comment".to_vec()),
            (2, b"second comment".to_vec()),
            (3, Vec::new())
        ]
    );
    assert_eq!(
        unpack(archive),
        [b"one".as_slice(), b"two".as_slice(), b"three".as_slice()]
    );
}

#[test]
fn damaged_service_comment_is_empty_without_changing_file_payload() {
    let mut archive = old_archive(&[
        old_file(b"entry", b"file data", 0, 29),
        service_comment(b"visible comment", true),
    ]);
    let at = archive
        .windows(15)
        .position(|w| w == b"visible comment")
        .unwrap();
    archive[at] ^= 1;
    let control = Cancellation::default();
    let mut index = read_index(
        &mut Cursor::new(&archive),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(
        index
            .entry_comments(&mut Cursor::new(&archive), &[1], &control, None)
            .unwrap()[0]
            .1,
        b""
    );
    assert_eq!(unpack(archive), [b"file data"]);
}

#[test]
#[ignore = "requires separately verified public RAR comment fixtures"]
fn independent_comment_fixtures_read_expected_entry_text() {
    let root = std::path::PathBuf::from(std::env::var_os("XDZIP_RAR_COMMENT_FIXTURES").unwrap());
    let control = Cancellation::default();
    for (name, password, expected) in [
        (
            "rar202-comment-nopsw.rar",
            None,
            ["file1comment", "file2comment"],
        ),
        (
            "rar3-comment-plain.rar",
            None,
            ["Comment1v2\n", "Comment2v2\n"],
        ),
        (
            "rar3-comment-hpsw.rar",
            Some("password"),
            ["Comment1v2\n", "Comment2v2\n"],
        ),
        (
            "rar3-comment-psw.rar",
            Some("password"),
            ["Comment1v2\n", "Comment2v2\n"],
        ),
    ] {
        let bytes = std::fs::read(root.join(name)).unwrap();
        let mut index = read_index(
            &mut Cursor::new(&bytes),
            &control,
            ScanLimits::default(),
            password,
        )
        .unwrap();
        let comments = index
            .entry_comments(&mut Cursor::new(&bytes), &[1, 2], &control, password)
            .unwrap();
        for (actual, expected) in comments.iter().zip(expected) {
            assert_eq!(actual.1, expected.as_bytes(), "{name}");
        }
    }
    let bytes = std::fs::read(root.join("FCOMM.RAR")).unwrap();
    let mut index = read_index(
        &mut Cursor::new(&bytes),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(
        index
            .entry_comments(&mut Cursor::new(&bytes), &[1], &control, None)
            .unwrap()[0]
            .1,
        b"FCOM\r\n"
    );
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
fn embedded_file_comments_are_decoded_only_when_requested() {
    let archive = old_archive(&[
        file_with_comment(b"file comment"),
        old_file(b"plain", b"xyz", 0, 20),
    ]);
    let control = Cancellation::default();
    let mut index = read_index(
        &mut Cursor::new(archive.clone()),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert!(index.entries()[0].details.comment_region.is_some());
    assert!(index.entries()[0].details.comment.is_empty());
    assert!(index.entries()[1].details.comment_region.is_none());
    assert_eq!(
        index
            .entry_comments(&mut Cursor::new(archive.clone()), &[1, 2], &control, None)
            .unwrap(),
        [(1, b"file comment".to_vec()), (2, Vec::new())]
    );
    assert_eq!(
        unpack(archive.clone()),
        [b"abc".as_slice(), b"xyz".as_slice()]
    );
    let mut damaged = archive;
    let at = damaged
        .windows(12)
        .position(|w| w == b"file comment")
        .unwrap();
    damaged[at] ^= 1;
    let mut index = read_index(
        &mut Cursor::new(damaged.clone()),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert!(
        index
            .entry_comments(&mut Cursor::new(damaged.clone()), &[1], &control, None)
            .unwrap()[0]
            .1
            .is_empty()
    );
    assert_eq!(unpack(damaged), [b"abc".as_slice(), b"xyz".as_slice()]);
}

#[test]
fn legacy_comment_encoding_can_be_detected_when_filename_is_ascii() {
    let raw = [
        213, 226, 202, 199, 210, 187, 184, 246, 214, 208, 206, 196, 215, 162, 202, 205, 163, 172,
        211, 195, 211, 218, 209, 233, 214, 164, 206, 196, 188, 254, 212, 164, 192, 192, 196, 220,
        185, 187, 213, 253, 200, 183, 207, 212, 202, 190, 209, 185, 203, 245, 176, 252, 214, 208,
        181, 196, 214, 208, 206, 196, 196, 218, 200, 221, 161, 163,
    ];
    let archive = old_archive(&[file_with_comment(&raw)]);
    let control = Cancellation::default();
    let mut index = read_index(
        &mut Cursor::new(&archive),
        &control,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    let result = index
        .entry_comments(&mut Cursor::new(&archive), &[1], &control, None)
        .unwrap();
    assert_eq!(
        result[0].1,
        "这是一个中文注释，用于验证文件预览能够正确显示压缩包中的中文内容。".as_bytes()
    );
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
