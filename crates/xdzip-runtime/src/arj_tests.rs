//! ARJ through actual host identity, checking, selected extraction and publication.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
#[path = "../../xdzip-compression/src/arj/fixture_cases.rs"]
mod cases;
#[path = "../../../tests/fixtures/arj/known.rs"]
mod known;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "xdzip-arj-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        Self(root)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/arj")
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
fn refresh(bytes: &mut [u8], offset: usize) {
    let size = u16::from_le_bytes(bytes[offset + 2..offset + 4].try_into().unwrap()) as usize;
    let crc = xdzip_compression::crc32(&bytes[offset + 4..offset + 4 + size]);
    bytes[offset + 4 + size..offset + 8 + size].copy_from_slice(&crc.to_le_bytes());
}
fn stored(payload: &[u8]) -> Vec<u8> {
    let mut bytes = std::fs::read(fixture("m0-phrase.arj")).unwrap();
    // The owned, CRC-verified template has main/header spans 56/67 bytes.
    bytes[65] = 0;
    bytes[72..76].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes[76..80].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes[80..84].copy_from_slice(&xdzip_compression::crc32(payload).to_le_bytes());
    refresh(&mut bytes, 56);
    bytes.truncate(123);
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&[0x60, 0xea, 0, 0]);
    bytes
}
#[test]
fn all_native_profiles_use_real_browse_full_check_and_selected_publication() {
    let cancel = Cancellation::default();
    for case in cases::CASES {
        let scratch = Scratch::new();
        let archive = Archive::open(&fixture(case.name), &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Arj);
        assert!(!archive.update_allowed);
        let row = &archive.rows[0];
        assert_eq!(
            (row.name_bytes(), row.method, row.size, row.packed_size),
            (
                b"HELLO.BIN".as_slice(),
                u16::from(case.method),
                case.decoded as u64,
                case.packed as u64
            )
        );
        assert!(case.method == 0 || case.method == case.requested);
        assert_eq!(
            std::fs::metadata(fixture(case.name)).unwrap().len(),
            (case.offset + case.packed + 4) as u64
        );
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
            "{}: {}",
            case.name,
            checked.error
        );
        assert_eq!(checked.verified_bytes, case.decoded as u64);
        let mut selected = options(&scratch.0);
        selected.all = false;
        selected.selected = vec![row.id.get().to_string()];
        let mut result = None;
        extraction::run(
            archive.extraction_job(selected, &cancel).unwrap(),
            2,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(
            result.status,
            Status::Succeeded,
            "{}: {}",
            case.name,
            result.error
        );
        let actual = std::fs::read(scratch.0.join("HELLO.BIN")).unwrap();
        assert_eq!(actual, known::plain(case.kind));
        assert_eq!(xdzip_compression::crc32(&actual), case.crc);
    }
}
#[test]
fn crc_failures_and_unknown_methods_preserve_old_output_without_staging_leftovers() {
    let cancel = Cancellation::default();
    for method in [0, 8] {
        let scratch = Scratch::new();
        let mut bytes = stored(b"known content");
        bytes[65] = method;
        if method == 0 {
            bytes[123] ^= 1;
        }
        refresh(&mut bytes, 56);
        let source = scratch.0.join("wrong-extension.zip");
        std::fs::write(&source, &bytes).unwrap();
        let archive = Archive::open(&source, &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Arj);
        assert_eq!(archive.rows[0].method, u16::from(method));
        let target = scratch.0.join("output");
        let opts = options(&target);
        std::fs::write(target.join("HELLO.BIN"), b"old content").unwrap();
        let job = archive.extraction_job(opts, &cancel);
        if method == 8 {
            assert!(matches!(job, Err("unsupported_entry")));
        } else {
            let mut result = None;
            extraction::run(job.unwrap(), 3, &Control::default(), &mut |s| {
                result = Some(s)
            });
            assert_eq!(result.unwrap().status, Status::Failed);
        }
        assert_eq!(
            std::fs::read(target.join("HELLO.BIN")).unwrap(),
            b"old content"
        );
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
    }
}
#[test]
fn duplicates_and_unvalidated_tar_or_nested_zip_keep_the_arj_physical_namespace() {
    let cancel = Cancellation::default();
    let scratch = Scratch::new();
    let mut marker = vec![b'A'; 600];
    marker[257 - 123..262 - 123].copy_from_slice(b"ustar");
    let zip = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip"),
    )
    .unwrap();
    for (i, payload) in [marker, zip].into_iter().enumerate() {
        let mut bytes = stored(&payload);
        bytes.truncate(bytes.len() - 4);
        bytes.extend_from_slice(&stored(b"second record")[56..]);
        let source = scratch.0.join(format!("input{i}.dat"));
        std::fs::write(&source, bytes).unwrap();
        let archive = Archive::open(&source, &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Arj);
        assert_eq!(archive.rows.len(), 2);
        assert_eq!(archive.rows[0].id.get(), 1);
        assert_eq!(archive.rows[1].id.get(), 2);
        let target = scratch.0.join(format!("out{i}"));
        let mut selected = options(&target);
        selected.all = false;
        selected.selected = vec!["2".into()];
        let mut result = None;
        extraction::run(
            archive.extraction_job(selected, &cancel).unwrap(),
            4,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        assert_eq!(
            std::fs::read(target.join("HELLO.BIN")).unwrap(),
            b"second record"
        );
    }
}
#[test]
fn later_corruption_and_cancel_never_publish_partial_arj_metadata() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let mut bytes = stored(b"first");
    bytes.truncate(bytes.len() - 4);
    let later = bytes.len();
    bytes.extend_from_slice(&stored(b"second")[56..]);
    bytes[later + 4] ^= 1;
    let source = scratch.0.join("broken.arj");
    std::fs::write(&source, bytes).unwrap();
    assert!(Archive::open(&source, &cancel).is_err());
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert!(Archive::open(&fixture("m1-phrase.arj"), &cancelled).is_err());
}

#[test]
fn unknown_neighbor_prevents_whole_check_but_known_selection_retains_its_identity() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let mut bytes = stored(b"safe selection");
    bytes.truncate(bytes.len() - 4);
    let mut unknown = stored(b"unknown neighbor");
    unknown[65] = 8;
    refresh(&mut unknown, 56);
    bytes.extend_from_slice(&unknown[56..]);
    let source = scratch.0.join("mixed.arj");
    std::fs::write(&source, bytes).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    assert_eq!(archive.rows.len(), 2);
    assert!(archive.rows[0].payload_supported() && !archive.rows[1].payload_supported());
    let mut checked = None;
    checking::run(
        archive.check_job(&cancel).unwrap(),
        5,
        &Control::default(),
        &mut |s| checked = Some(s),
    );
    assert_eq!(checked.unwrap().status, Status::Failed);
    let target = scratch.0.join("out");
    let mut selected = options(&target);
    selected.all = false;
    selected.selected = vec!["1".into()];
    let mut result = None;
    extraction::run(
        archive.extraction_job(selected, &cancel).unwrap(),
        6,
        &Control::default(),
        &mut |s| result = Some(s),
    );
    assert_eq!(result.unwrap().status, Status::Succeeded);
    assert_eq!(
        std::fs::read(target.join("HELLO.BIN")).unwrap(),
        b"safe selection"
    );
}
