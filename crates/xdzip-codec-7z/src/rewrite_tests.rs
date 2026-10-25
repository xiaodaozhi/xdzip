use super::*;
use crate::encode::tests::{create, owned, settings, unpack};
use crate::encode::{FileProperties, encode_internal};
use std::io::{Cursor, Seek, SeekFrom};
fn rewritten(
    original: &[u8],
    keep: &[bool],
    renames: &[Option<String>],
    additions: &dyn EncodeInput,
    level: u8,
) -> Vec<u8> {
    let mut source = Cursor::new(original);
    let mut output = Cursor::new(Vec::new());
    SevenZipCodec
        .rewrite(
            &mut source,
            RewriteInput {
                keep,
                renames,
                additions,
                settings: settings(level),
            },
            &mut output,
            &Cancellation::default(),
            &mut |_| {},
        )
        .unwrap();
    output.into_inner()
}
#[test]
fn rewrite_preserves_physical_duplicates_raw_metadata_order_rename_and_add() {
    let original = owned(&[
        ("same", false, b"first physical content"),
        ("same", false, b"second physical content"),
        ("dir", true, b""),
        ("drop", false, b"removed"),
    ]);
    let properties = [
        FileProperties {
            attributes: Some((0o100640 << 16) | 0x8020),
            creation_time: Some(133_000_000_000_000_001),
            access_time: Some(133_100_000_000_000_003),
            modified_time: Some(133_200_000_000_000_007),
        },
        FileProperties {
            attributes: None,
            creation_time: None,
            access_time: None,
            modified_time: None,
        },
        FileProperties {
            attributes: Some((0o040750 << 16) | 0x8010),
            ..FileProperties::new(&original.entries[2])
        },
        FileProperties::new(&original.entries[3]),
    ];
    let mut archive = Vec::new();
    let mut options = settings(6);
    options.solid = true;
    encode_internal(
        &original,
        &mut archive,
        options,
        &Cancellation::default(),
        &mut |_| {},
        Some(&properties),
        0,
    )
    .unwrap();
    let addition = owned(&[
        ("same", false, b"new third physical record"),
        ("empty", false, b""),
    ]);
    let keep = [true, true, true, false];
    let result = rewritten(
        &archive,
        &keep,
        &[
            Some("moved/first".to_owned()),
            None,
            Some("newdir/".to_owned()),
            None,
        ],
        &addition,
        9,
    );
    let (index, values) = unpack(&result, None);
    assert_eq!(
        values,
        [
            b"first physical content".to_vec(),
            b"second physical content".to_vec(),
            Vec::new(),
            b"new third physical record".to_vec(),
            Vec::new()
        ]
    );
    assert_eq!(
        index
            .rows
            .iter()
            .map(|row| row.raw_name.as_slice())
            .collect::<Vec<_>>(),
        [
            b"moved/first".as_slice(),
            b"same".as_slice(),
            b"newdir/".as_slice(),
            b"same".as_slice(),
            b"empty".as_slice()
        ]
    );
    for (file, expected) in index.files[..3].iter().zip(&properties) {
        assert_eq!(file.attributes, expected.attributes);
        assert_eq!(file.creation_time, expected.creation_time);
        assert_eq!(file.access_time, expected.access_time);
        assert_eq!(file.modified_time, expected.modified_time);
    }
    assert_eq!(
        index.files[3].modified_time,
        FileProperties::new(&addition.entries[0]).modified_time
    );
    // No name-keyed map: two indistinguishable names remain separate records.
    let duplicated = rewritten(&archive, &[true, true, false, false], &[], &owned(&[]), 0);
    let (index, values) = unpack(&duplicated, None);
    assert_eq!(index.rows[0].name_bytes(), index.rows[1].name_bytes());
    assert_ne!(index.rows[0].id, index.rows[1].id);
    assert_eq!(values[0], original.bytes[0]);
    assert_eq!(values[1], original.bytes[1]);
    let selected_second = rewritten(&archive, &[false, true, false, false], &[], &owned(&[]), 1);
    assert_eq!(
        unpack(&selected_second, None).1,
        [original.bytes[1].clone()]
    );
}
#[test]
fn rewrite_all_removed_and_add_only_empty_records() {
    let archive = create(&owned(&[("old", false, b"old")]), settings(0));
    for additions in [
        owned(&[]),
        owned(&[("dir", true, b""), ("empty", false, b"")]),
    ] {
        let output = rewritten(&archive, &[false], &[], &additions, 6);
        assert_eq!(unpack(&output, None).1, additions.bytes);
    }
}
fn reject(archive: &[u8], keep: &[bool], additions: &dyn EncodeInput, expected: EncodeError) {
    let sentinel = b"old target stays owned by host".to_vec();
    let mut output = Cursor::new(sentinel.clone());
    let result = SevenZipCodec.rewrite(
        &mut Cursor::new(archive),
        RewriteInput {
            keep,
            renames: &[],
            additions,
            settings: settings(0),
        },
        &mut output,
        &Cancellation::default(),
        &mut |_| {},
    );
    assert_eq!(result, Err(expected));
    assert_eq!(
        output.into_inner(),
        sentinel,
        "rebuild admission/old-data verification precedes output writes"
    );
}
#[test]
fn rewrite_bad_old_crc_does_not_touch_host_staging_output() {
    let mut archive = create(
        &owned(&[("old", false, b"retained content must be checked")]),
        settings(0),
    );
    archive[33] ^= 1;
    reject(&archive, &[true], &owned(&[]), EncodeError::Invalid);
}
fn modified_header(archive: &[u8], change: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
    let start = u64::from_le_bytes(archive[12..20].try_into().unwrap()) as usize + 32;
    let mut header = archive[start..].to_vec();
    change(&mut header);
    let mut output = archive[..start].to_vec();
    output.extend_from_slice(&header);
    output[20..28].copy_from_slice(&(header.len() as u64).to_le_bytes());
    output[28..32].copy_from_slice(&xdzip_compression::crc32(&header).to_le_bytes());
    let checksum = xdzip_compression::crc32(&output[12..32]);
    output[8..12].copy_from_slice(&checksum.to_le_bytes());
    output
}
#[test]
fn rewrite_rejects_encrypted_unknown_archive_properties_links_and_anti() {
    let input = owned(&[("owned", false, b"bytes")]);
    for headers in [false, true] {
        let mut options = settings(0);
        options.password = Some("owned-password");
        options.encrypt_headers = headers;
        reject(
            &create(&input, options),
            &[true],
            &owned(&[]),
            EncodeError::Unsupported,
        );
    }
    let plain = create(&input, settings(0));
    let unknown = modified_header(&plain, |header| {
        header.splice(1..1, [2, 99, 1, 0, 0]);
    });
    reject(&unknown, &[true], &owned(&[]), EncodeError::Unsupported);
    for attributes in [(0o120777 << 16) | 32, (0o020600 << 16) | 32, 0x420] {
        let mut archive = Vec::new();
        let properties = [FileProperties {
            attributes: Some(attributes),
            ..FileProperties::new(&input.entries[0])
        }];
        encode_internal(
            &input,
            &mut archive,
            settings(0),
            &Cancellation::default(),
            &mut |_| {},
            Some(&properties),
            0,
        )
        .unwrap();
        reject(&archive, &[true], &owned(&[]), EncodeError::Unsupported);
    }
    let empty = create(&owned(&[("deleted", false, b"")]), settings(0));
    let anti = modified_header(&empty, |header| {
        let position = header.len() - 2;
        header.splice(position..position, [16, 1, 128]);
    });
    reject(&anti, &[true], &owned(&[]), EncodeError::Unsupported);
}
struct Volumes {
    source: Cursor<Vec<u8>>,
    volumes: [u64; 2],
    reads: usize,
}
impl Read for Volumes {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.reads += 1;
        self.source.read(output)
    }
}
impl Seek for Volumes {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.source.seek(from)
    }
}
impl ReadSource for Volumes {
    fn volume_lengths(&self) -> Option<&[u64]> {
        Some(&self.volumes)
    }
}
#[test]
fn rewrite_split_and_working_set_limits_fail_before_open_or_write() {
    let archive = create(&owned(&[("old", false, b"small")]), settings(0));
    let mut volumes = Volumes {
        source: Cursor::new(archive.clone()),
        volumes: [1, archive.len() as u64 - 1],
        reads: 0,
    };
    let mut output = Cursor::new(Vec::new());
    let empty = owned(&[]);
    assert_eq!(
        SevenZipCodec.rewrite(
            &mut volumes,
            RewriteInput {
                keep: &[true],
                renames: &[],
                additions: &empty,
                settings: settings(0)
            },
            &mut output,
            &Cancellation::default(),
            &mut |_| {}
        ),
        Err(EncodeError::Unsupported)
    );
    assert_eq!(volumes.reads, 0);
    assert!(output.into_inner().is_empty());
    let mut addition = owned(&[("large", false, b"")]);
    addition.entries[0].size = WORKING_SET;
    reject(&archive, &[true], &addition, EncodeError::Limit);
    assert_eq!(addition.opens.get(), 0);
    reject(&archive, &[], &owned(&[]), EncodeError::Unsupported);
}
#[test]
fn rewrite_cancellation_and_grown_addition_leave_staging_untouched() {
    let archive = create(&owned(&[("old", false, b"small")]), settings(0));
    let mut addition = owned(&[("add", false, b"too many bytes")]);
    addition.entries[0].size = 3;
    reject(&archive, &[true], &addition, EncodeError::Source);
    let control = Cancellation::default();
    control.cancel();
    let empty = owned(&[]);
    let mut output = Cursor::new(Vec::new());
    assert_eq!(
        SevenZipCodec.rewrite(
            &mut Cursor::new(archive),
            RewriteInput {
                keep: &[true],
                renames: &[],
                additions: &empty,
                settings: settings(0)
            },
            &mut output,
            &control,
            &mut |_| {}
        ),
        Err(EncodeError::Cancelled)
    );
    assert!(output.into_inner().is_empty());
}
#[test]
#[ignore = "exports owned rewrite results for pinned official-reader known-byte verification"]
fn rewrite_export_physical_selection_and_raw_properties() {
    let directory = std::path::PathBuf::from(std::env::var_os("XDZIP_7Z_WRITER_OUTPUT").unwrap())
        .canonicalize()
        .unwrap();
    let build = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build")
        .canonicalize()
        .unwrap();
    assert!(directory.starts_with(build));
    let first = b"first original rewritten file\n".repeat(1000);
    let second = b"second retained physical data\n".repeat(800);
    let input = owned(&[
        ("same", false, &first),
        ("same", false, &second),
        ("dir", true, b""),
    ]);
    let properties = [
        FileProperties {
            attributes: Some((0o100640 << 16) | 0x8020),
            creation_time: Some(133_000_000_000_000_001),
            access_time: Some(133_100_000_000_000_003),
            modified_time: Some(133_200_000_000_000_007),
        },
        FileProperties {
            attributes: None,
            creation_time: None,
            access_time: None,
            modified_time: None,
        },
        FileProperties {
            attributes: Some((0o040750 << 16) | 0x8010),
            ..FileProperties::new(&input.entries[2])
        },
    ];
    let mut archive = Vec::new();
    encode_internal(
        &input,
        &mut archive,
        settings(6),
        &Cancellation::default(),
        &mut |_| {},
        Some(&properties),
        0,
    )
    .unwrap();
    let additions = owned(&[
        ("new/中文.txt", false, b"new known file bytes"),
        ("empty", false, b""),
    ]);
    let output = rewritten(
        &archive,
        &[true, true, true],
        &[
            Some("first.txt".to_owned()),
            Some("second.txt".to_owned()),
            None,
        ],
        &additions,
        9,
    );
    let (index, values) = unpack(&output, None);
    assert_eq!(values[..2], input.bytes[..2]);
    assert_eq!(index.rows.len(), 5);
    std::fs::write(directory.join("rewrite-selection.7z"), &output).unwrap();
    std::fs::write(directory.join("rewrite-first.plain"), &first).unwrap();
    std::fs::write(directory.join("rewrite-second.plain"), &second).unwrap();
    eprintln!("rewrite_export archive={} files=5", output.len());
}
#[test]
#[ignore = "rewrites a fixed official-tool authored owned-input archive and exports actual property and known-byte results"]
fn rewrite_official_source_raw_properties_and_known_contents() {
    let path =
        std::path::PathBuf::from(std::env::var_os("XDZIP_7Z_REWRITE_REFERENCE_INPUT").unwrap())
            .canonicalize()
            .unwrap();
    let directory = std::path::PathBuf::from(std::env::var_os("XDZIP_7Z_WRITER_OUTPUT").unwrap())
        .canonicalize()
        .unwrap();
    let build = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build")
        .canonicalize()
        .unwrap();
    assert!(path.starts_with(&build) && directory.starts_with(&build));
    let archive = std::fs::read(path).unwrap();
    let (old_index, old_values) = unpack(&archive, None);
    let keep = vec![true; old_index.rows.len()];
    let renames: Vec<_> = old_index
        .rows
        .iter()
        .map(|row| {
            Some(match row.name_bytes() {
                b"alpha.txt" => "moved/alpha.txt".to_owned(),
                b"nested/beta.bin" => "preserved-dir/beta.bin".to_owned(),
                b"nested/" => "preserved-dir/".to_owned(),
                other => std::str::from_utf8(other).unwrap().to_owned(),
            })
        })
        .collect();
    let additions = owned(&[("new/中文.txt", false, b"known appended original contents")]);
    let output = rewritten(&archive, &keep, &renames, &additions, 6);
    let (new_index, new_values) = unpack(&output, None);
    assert_eq!(new_values[..old_values.len()], old_values);
    assert_eq!(new_values.last().unwrap(), &additions.bytes[0]);
    let mut records = String::new();
    for (physical, ((old, new), value)) in old_index
        .files
        .iter()
        .zip(&new_index.files)
        .zip(&old_values)
        .enumerate()
    {
        assert_eq!(old.attributes, new.attributes);
        assert_eq!(old.creation_time, new.creation_time);
        assert_eq!(old.access_time, new.access_time);
        assert_eq!(old.modified_time, new.modified_time);
        let name = std::str::from_utf8(new_index.rows[physical].name_bytes()).unwrap();
        records.push_str(&format!(
            "{physical}\t{}\t{name}\n",
            u8::from(new_index.rows[physical].directory)
        ));
        std::fs::write(
            directory.join(format!("rewrite-official-{physical}.plain")),
            value,
        )
        .unwrap();
    }
    std::fs::write(directory.join("rewrite-official-input.7z"), output).unwrap();
    std::fs::write(directory.join("rewrite-official-records.tsv"), records).unwrap();
    eprintln!(
        "rewrite_official old_records={} final_records={} raw_property_fields_preserved=4",
        old_index.rows.len(),
        new_index.rows.len()
    );
}
struct ChangingSource {
    source: Cursor<Vec<u8>>,
    first_failure: u64,
    pack_end: u64,
    signal: xdzip_codec_api::ScanError,
    failed_reads: usize,
}
impl Read for ChangingSource {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.source.position() >= self.first_failure && self.source.position() < self.pack_end {
            self.failed_reads += 1;
            return Err(io::Error::new(io::ErrorKind::Interrupted, self.signal));
        }
        self.source.read(output)
    }
}
impl Seek for ChangingSource {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.source.seek(from)
    }
}
impl ReadSource for ChangingSource {}
#[test]
fn rewrite_old_payload_typed_failures_after_scan_never_retry_or_touch_output() {
    let archive = create(&owned(&[("old", false, &vec![7; 9000])]), settings(0));
    let empty = owned(&[]);
    for signal in [
        xdzip_codec_api::ScanError::SourceChanged,
        xdzip_codec_api::ScanError::Cancelled,
        xdzip_codec_api::ScanError::Limit,
    ] {
        for partial in [false, true] {
            let mut source = ChangingSource {
                source: Cursor::new(archive.clone()),
                first_failure: 32 + if partial { 8192 } else { 0 },
                pack_end: 9032,
                signal,
                failed_reads: 0,
            };
            let sentinel = b"host target remains outside codec authority".to_vec();
            let mut output = Cursor::new(sentinel.clone());
            assert_eq!(
                SevenZipCodec.rewrite(
                    &mut source,
                    RewriteInput {
                        keep: &[true],
                        renames: &[],
                        additions: &empty,
                        settings: settings(0)
                    },
                    &mut output,
                    &Cancellation::default(),
                    &mut |_| {}
                ),
                Err(encode::scan_error(signal))
            );
            assert_eq!(source.failed_reads, 1);
            assert_eq!(output.into_inner(), sentinel);
        }
    }
}
#[test]
fn rewrite_private_work_credit_is_admitted_once_and_never_reused() {
    let control = Cancellation::default();
    let mut bytes = Vec::new();
    let mut work = Work {
        remaining: 4,
        credit: 0,
    };
    let mut sink = MemorySink {
        bytes: &mut bytes,
        size: 4,
        work: &mut work,
        control: &control,
    };
    assert_eq!(
        sink.write_precharged_chunk(b"x"),
        Err(DecodeError::Integrity)
    );
    sink.charge_skipped(4).unwrap();
    sink.write_precharged_chunk(b"data").unwrap();
    assert_eq!(sink.work.remaining, 0);
    assert_eq!(sink.work.credit, 0);
    assert_eq!(
        sink.write_precharged_chunk(b"x"),
        Err(DecodeError::Integrity)
    );
    assert_eq!(sink.write_chunk(b"x"), Err(DecodeError::Limit));
    assert_eq!(sink.charge_skipped(1), Err(DecodeError::Limit));
    assert_eq!(sink.work.credit, 0);
    assert_eq!(bytes, b"data");
}
#[test]
fn rewrite_selects_filesinfo_entities_not_unreferenced_folder_blobs() {
    // Reuse the Copy/zero-substream geometry from the container's existing
    // mixed-CRC vector. This is a project-owned grammar regression, not an
    // independent official-author compatibility claim.
    let mut header = vec![1, 4, 6, 0, 2, 9, 1, 1, 10, 1];
    for data in [b"X", b"A"] {
        header.extend_from_slice(&xdzip_compression::crc32(data).to_le_bytes());
    }
    header.extend_from_slice(&[0, 7, 11, 2, 0, 1, 1, 0, 1, 1, 0, 12, 1, 1, 10, 1]);
    for data in [b"X", b"A"] {
        header.extend_from_slice(&xdzip_compression::crc32(data).to_le_bytes());
    }
    header.extend_from_slice(&[0, 8, 13, 0, 1, 0, 0, 5, 1, 17, 11, 0]);
    for unit in "kept".encode_utf16().chain([0]) {
        header.extend_from_slice(&unit.to_le_bytes());
    }
    header.extend_from_slice(&[0, 0]);
    let mut tail = 2u64.to_le_bytes().to_vec();
    tail.extend_from_slice(&(header.len() as u64).to_le_bytes());
    tail.extend_from_slice(&xdzip_compression::crc32(&header).to_le_bytes());
    let mut archive = b"7z\xbc\xaf\x27\x1c\0\x04".to_vec();
    archive.extend_from_slice(&xdzip_compression::crc32(&tail).to_le_bytes());
    archive.extend_from_slice(&tail);
    archive.extend_from_slice(b"XA");
    archive.extend_from_slice(&header);
    let (before, data) = unpack(&archive, None);
    assert_eq!(before.rows.len(), 1);
    assert_eq!(data, [b"A".to_vec()]);
    assert_eq!(before.streams.folders.len(), 2);
    assert!(before.streams.folders[0].substreams.is_empty());
    assert_eq!(before.streams.folders[0].unpacked_size(), 1);
    let output = rewritten(&archive, &[true], &[], &owned(&[]), 0);
    let (after, data) = unpack(&output, None);
    assert_eq!(data, [b"A".to_vec()]);
    assert_eq!(after.rows[0].name_bytes(), b"kept");
    assert_eq!(after.streams.folders.len(), 1);
    assert_eq!(after.streams.packs[0].range.length, 1);
}
