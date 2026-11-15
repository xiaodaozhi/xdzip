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
