//! Actual host paths for LHA, with immutable identity and private output publication.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-lha-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/lha")
        .join(name)
}
fn options(target: &Path) -> Options {
    std::fs::create_dir_all(target).unwrap();
    Options {
        target: target.to_string_lossy().into_owned(),
        all: true,
        selected: vec![],
        directory: String::new(),
        create_folder: false,
        shell_layout: 0,
        conflict: ConflictPolicy::Overwrite,
        allow_high_ratio: false,
    }
}
#[test]
fn lha_common_header_and_method_profiles_use_actual_browse_check_and_extract() {
    let cancel = Cancellation::default();
    for name in [
        "lh0-h0.lzh",
        "lh1-h1.lzh",
        "lh2-h2.lzh",
        "lh3-h3.lzh",
        "lh4-h0.lzh",
        "lh5-h1.lzh",
        "lh6-h2.lzh",
        "lh7-h3.lzh",
        "lzs-h0.lzh",
        "lz4-h1.lzh",
        "lz5-h2.lzh",
    ] {
        let scratch = Scratch::new();
        let archive =
            Archive::open(&fixture(name), &cancel).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(archive.format, xdzip_codec_api::Format::Lha);
        assert!(!archive.update_allowed);
        assert_eq!(archive.rows.len(), 1);
        assert_eq!(archive.rows[0].name_bytes(), b"hello.bin");
        assert!(archive.rows[0].payload_supported());
        let mut checked = None;
        checking::run(
            archive.check_job(&cancel).unwrap(),
            1,
            &Control::default(),
            &mut |s| checked = Some(s),
        );
        let checked = checked.unwrap();
        assert_eq!(
            checked.status,
            Status::Succeeded,
            "{name}: {}",
            checked.error
        );
        assert_eq!(checked.verified_bytes, 144);
        let mut result = None;
        extraction::run(
            archive
                .extraction_job(options(&scratch.0), &cancel)
                .unwrap(),
            2,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(
            std::fs::read(scratch.0.join("hello.bin")).unwrap(),
            b"abracadabra abracadabra\n".repeat(6)
        );
    }
}
#[test]
fn damaged_stored_crc_cannot_replace_existing_output_and_empty_container_is_content_detected() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let mut bytes = std::fs::read(fixture("lh0-h0.lzh")).unwrap();
    bytes[33] ^= 1;
    let source = scratch.0.join("source.dat");
    std::fs::write(&source, bytes).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    assert_eq!(archive.format, xdzip_codec_api::Format::Lha);
    let target = scratch.0.join("output");
    let options = options(&target);
    std::fs::write(target.join("hello.bin"), b"old-content").unwrap();
    let mut result = None;
    extraction::run(
        archive.extraction_job(options, &cancel).unwrap(),
        3,
        &Control::default(),
        &mut |s| result = Some(s),
    );
    let result = result.unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(
        std::fs::read(target.join("hello.bin")).unwrap(),
        b"old-content"
    );
    let empty = scratch.0.join("empty.bin");
    std::fs::write(&empty, [0]).unwrap();
    let archive = Archive::open(&empty, &cancel).unwrap();
    assert_eq!(archive.format, xdzip_codec_api::Format::Lha);
    assert!(archive.rows.is_empty());
}
#[test]
fn independently_encoded_names_use_the_same_legacy_decoding_directory_expansion_and_output() {
    let cancel = Cancellation::default();
    for (name, path) in [
        ("named-utf8.lzh", "nested/资料/说明.txt"),
        (
            "named-shiftjis.lzh",
            "nested/ソフトウェアの設定/ソフトウェアの設定ファイルと日本語.txt",
        ),
    ] {
        let scratch = Scratch::new();
        let archive = Archive::open(&fixture(name), &cancel).unwrap();
        assert_eq!(archive.rows[0].name_bytes(), path.as_bytes());
        assert!(!archive.rows[0].warning);
        assert!(archive.rows[0].payload_supported());
        let mut options = options(&scratch.0);
        options.all = false;
        let prefix = format!("{}/", path.rsplit_once('/').unwrap().0);
        let directory = archive
            .directories
            .as_ref()
            .unwrap()
            .iter()
            .find(|node| node.path == prefix)
            .unwrap();
        options.selected = vec![format!("d{}", directory.id)];
        let mut result = None;
        extraction::run(
            archive.extraction_job(options, &cancel).unwrap(),
            4,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(
            std::fs::read(scratch.0.join(path)).unwrap(),
            b"abracadabra abracadabra\n".repeat(6)
        );
    }
}
#[test]
fn duplicate_names_remain_distinct_physical_entries_and_selected_export_uses_the_requested_one() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let mut first = std::fs::read(fixture("lh0-h0.lzh")).unwrap();
    first.pop();
    let mut second = std::fs::read(fixture("lh0-h0.lzh")).unwrap();
    second[33..177].fill(b'A');
    let crc = 0x143au16; // Independent CRC-16 of the owned 144 A bytes.
    second[31..33].copy_from_slice(&crc.to_le_bytes());
    second[1] = second[2..33]
        .iter()
        .fold(0u8, |sum, b| sum.wrapping_add(*b));
    first.extend_from_slice(&second);
    let source = scratch.0.join("duplicates.bin");
    std::fs::write(&source, first).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    assert_eq!(archive.rows.len(), 2);
    assert_ne!(archive.rows[0].id, archive.rows[1].id);
    assert_eq!(archive.rows[0].name_bytes(), archive.rows[1].name_bytes());
    let target = scratch.0.join("output");
    let mut options = options(&target);
    options.all = false;
    options.selected = vec![archive.rows[1].id.get().to_string()];
    let mut result = None;
    extraction::run(
        archive.extraction_job(options, &cancel).unwrap(),
        5,
        &Control::default(),
        &mut |s| result = Some(s),
    );
    let result = result.unwrap();
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(
        std::fs::read(target.join("hello.bin")).unwrap(),
        vec![b'A'; 144]
    );
}
fn stored(payload: &[u8]) -> Vec<u8> {
    let original = std::fs::read(fixture("lh0-h0.lzh")).unwrap();
    let mut result = original[..33].to_vec();
    result[7..11].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    result[11..15].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    result[31..33].copy_from_slice(&xdzip_compression::crc16(payload).to_le_bytes());
    result[1] = result[2..33]
        .iter()
        .fold(0u8, |sum, b| sum.wrapping_add(*b));
    result.extend_from_slice(payload);
    result.push(0);
    result
}
fn pma_fixture(name: &str) -> PathBuf {
    fixture(name)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("pma")
        .join(name)
}
#[path = "../../xdzip-compression/src/pm1/fixture_cases.rs"]
mod pm1_cases;
#[test]
fn every_pm1_reference_profile_uses_actual_runtime_check_and_selected_publication() {
    let cancel = Cancellation::default();
    let known = std::fs::read(pma_fixture("pm1-known.bin")).unwrap();
    for &(name, packed, size, _, offset) in pm1_cases::CASES {
        let scratch = Scratch::new();
        let archive = Archive::open(&pma_fixture(name), &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Pma);
        assert!(!archive.update_allowed);
        assert_eq!(archive.rows[0].method, 1);
        assert_eq!(archive.rows[0].packed_size, packed as u64);
        let mut checked = None;
        checking::run(
            archive.check_job(&cancel).unwrap(),
            11,
            &Control::default(),
            &mut |s| checked = Some(s),
        );
        let checked = checked.unwrap();
        assert_eq!(
            checked.status,
            Status::Succeeded,
            "{name}: {}",
            checked.error
        );
        assert_eq!(checked.verified_bytes, size as u64);
        let mut selected = options(&scratch.0);
        selected.all = false;
        selected.selected = vec![archive.rows[0].id.get().to_string()];
        let mut result = None;
        extraction::run(
            archive.extraction_job(selected, &cancel).unwrap(),
            12,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(
            std::fs::read(scratch.0.join("HELLO.BIN")).unwrap(),
            known[offset..offset + size]
        );
    }
}
#[test]
fn pma_common_profiles_use_actual_readonly_check_and_private_extraction() {
    let cancel = Cancellation::default();
    for (name, expected) in [
        ("pm0-allbytes.pma", (0..=255).collect::<Vec<u8>>()),
        ("pm2-zeros.pma", vec![0; 65536]),
        (
            "pm2-deep28.pma",
            (32..=39).rev().cycle().take(128).collect(),
        ),
        (
            "pm2-boundary-1024.pma",
            [
                (0..=255).collect::<Vec<u8>>(),
                b"abracadabra abracadabra\n".repeat(128),
            ]
            .concat()
            .into_iter()
            .cycle()
            .take(1024)
            .collect(),
        ),
    ] {
        let scratch = Scratch::new();
        let archive = Archive::open(&pma_fixture(name), &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Pma);
        assert!(!archive.update_allowed);
        assert_eq!(archive.rows[0].name_bytes(), b"HELLO.BIN");
        let mut checked = None;
        checking::run(
            archive.check_job(&cancel).unwrap(),
            7,
            &Control::default(),
            &mut |s| checked = Some(s),
        );
        let checked = checked.unwrap();
        assert_eq!(
            checked.status,
            Status::Succeeded,
            "{name}: {}",
            checked.error
        );
        assert_eq!(checked.verified_bytes, expected.len() as u64);
        let mut result = None;
        extraction::run(
            archive
                .extraction_job(options(&scratch.0), &cancel)
                .unwrap(),
            8,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(
            std::fs::read(scratch.0.join("HELLO.BIN")).unwrap(),
            expected
        );
    }
    let empty = Archive::open(&pma_fixture("empty.pma"), &cancel).unwrap();
    assert_eq!(empty.format, xdzip_codec_api::Format::Pma);
    assert!(empty.rows.is_empty());
}
#[test]
fn pma_crc_invalid_pm1_and_unknown_methods_leave_existing_output_and_no_staging_files() {
    let cancel = Cancellation::default();
    for method in [0, 1, 3] {
        let scratch = Scratch::new();
        let mut bytes = std::fs::read(pma_fixture("pm0-allbytes.pma")).unwrap();
        bytes[5] = b'0' + method;
        if method == 0 {
            bytes[33] ^= 1;
        }
        bytes[1] = bytes[2..33].iter().fold(0u8, |s, b| s.wrapping_add(*b));
        let source = scratch.0.join("input.dat");
        std::fs::write(&source, bytes).unwrap();
        let archive = Archive::open(&source, &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Pma);
        assert_eq!(archive.rows[0].payload_supported(), method != 3);
        let target = scratch.0.join("output");
        let options = options(&target);
        std::fs::write(target.join("HELLO.BIN"), b"old-content").unwrap();
        let job = archive.extraction_job(options, &cancel);
        if method == 3 {
            assert!(matches!(job, Err("unsupported_entry")));
        } else {
            let mut result = None;
            extraction::run(job.unwrap(), 9, &Control::default(), &mut |s| {
                result = Some(s)
            });
            assert_eq!(result.unwrap().status, Status::Failed);
        }
        assert_eq!(
            std::fs::read(target.join("HELLO.BIN")).unwrap(),
            b"old-content"
        );
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
    }
}
fn pma_stored(payload: &[u8]) -> Vec<u8> {
    let mut bytes = stored(payload);
    bytes[2..7].copy_from_slice(b"-pm0-");
    bytes[1] = bytes[2..33].iter().fold(0u8, |s, b| s.wrapping_add(*b));
    bytes.resize(bytes.len().div_ceil(128) * 128, 0x1a);
    bytes
}
#[test]
fn valid_lha_pma_header_checksums_cannot_masquerade_as_gzip_or_unix_compress() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    for pma in [false, true] {
        for signature in [0x8bu8, 0x9d] {
            let payload = b"independent known collision content";
            let mut bytes = if pma {
                pma_stored(payload)
            } else {
                stored(payload)
            };
            let mut found = false;
            for minute in 0..1440u32 {
                let dos = (43 << 25)
                    | (11 << 21)
                    | (14 << 16)
                    | ((minute / 60) << 11)
                    | ((minute % 60) << 5);
                bytes[15..19].copy_from_slice(&dos.to_le_bytes());
                bytes[1] = bytes[2..33].iter().fold(0u8, |s, b| s.wrapping_add(*b));
                if bytes[1] == signature {
                    found = true;
                    break;
                }
            }
            assert!(found);
            assert_eq!(bytes[..2], [0x1f, signature]);
            let path = scratch.0.join(format!("collision-{pma}-{signature}.dat"));
            std::fs::write(&path, &bytes).unwrap();
            let archive = Archive::open(&path, &cancel).unwrap();
            assert_eq!(
                archive.format,
                if pma {
                    xdzip_codec_api::Format::Pma
                } else {
                    xdzip_codec_api::Format::Lha
                }
            );
            let target = scratch.0.join(format!("output-{pma}-{signature}"));
            let mut result = None;
            extraction::run(
                archive.extraction_job(options(&target), &cancel).unwrap(),
                13,
                &Control::default(),
                &mut |s| result = Some(s),
            );
            let result = result.unwrap();
            assert_eq!(result.status, Status::Succeeded, "{}", result.error);
            assert_eq!(std::fs::read(target.join("hello.bin")).unwrap(), payload);
        }
    }
}
#[test]
fn pma_duplicates_source_marker_collisions_and_nested_zip_keep_the_physical_namespace() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let mut content = vec![b'A'; 300];
    content[224..229].copy_from_slice(b"ustar");
    let nested = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip"),
    )
    .unwrap();
    for (at, payload) in [content, nested].into_iter().enumerate() {
        let mut bytes = pma_stored(&payload);
        bytes.truncate(33 + payload.len());
        bytes.extend_from_slice(&pma_stored(b"second-record")[..33 + 13]);
        bytes.push(0);
        bytes.resize(bytes.len().div_ceil(128) * 128, 0x1a);
        let path = scratch.0.join(format!("collision-{at}.bin"));
        std::fs::write(&path, &bytes).unwrap();
        let archive = Archive::open(&path, &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Pma);
        assert_eq!(archive.rows.len(), 2);
        assert_eq!(archive.rows[0].name_bytes(), archive.rows[1].name_bytes());
        assert_ne!(archive.rows[0].id, archive.rows[1].id);
        let target = scratch.0.join(format!("output-{at}"));
        let mut options = options(&target);
        options.all = false;
        options.selected = vec![archive.rows[1].id.get().to_string()];
        let mut result = None;
        extraction::run(
            archive.extraction_job(options, &cancel).unwrap(),
            10,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        assert_eq!(
            std::fs::read(target.join("hello.bin")).unwrap(),
            b"second-record"
        );
    }
}
#[test]
fn source_marker_collisions_and_nested_zip_content_keep_the_validated_lha_namespace() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let mut content = vec![b'A'; 300];
    content[224..229].copy_from_slice(b"ustar");
    let nested = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip"),
    )
    .unwrap();
    for (name, payload) in [("raw-tar-marker.bin", content), ("nested-zip.bin", nested)] {
        let path = scratch.0.join(name);
        std::fs::write(&path, stored(&payload)).unwrap();
        let archive = Archive::open(&path, &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Lha);
        assert_eq!(archive.rows[0].size, payload.len() as u64);
        let target = scratch.0.join(format!("out-{name}"));
        let mut result = None;
        extraction::run(
            archive.extraction_job(options(&target), &cancel).unwrap(),
            6,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(std::fs::read(target.join("hello.bin")).unwrap(), payload);
    }
}
