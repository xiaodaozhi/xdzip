//! ARC through host discovery, checks, immutable physical selections and publication.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
#[path = "../../xdzip-compression/src/arc/fixture_cases.rs"]
mod cases;
#[path = "../../../tests/fixtures/arc/known.rs"]
mod known;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-arc-{}-{}",
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
        .join("../../tests/fixtures/arc")
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
fn stored(payload: &[u8]) -> Vec<u8> {
    let mut bytes = std::fs::read(fixture("m2-phrase.arc")).unwrap();
    bytes.truncate(29);
    bytes[15..19].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes[25..29].copy_from_slice(&(payload.len() as u32).to_le_bytes());
    bytes[23..25].copy_from_slice(&xdzip_compression::crc16(payload).to_le_bytes());
    bytes.extend_from_slice(payload);
    bytes.extend_from_slice(&[0x1a, 0]);
    bytes
}
#[test]
fn all_independent_arc_profiles_use_actual_browse_full_check_and_selected_extraction() {
    let cancel = Cancellation::default();
    for case in cases::CASES {
        let scratch = Scratch::new();
        let archive = Archive::open(&fixture(case.name), &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Arc);
        assert!(!archive.update_allowed);
        assert_eq!(archive.rows.len(), 1);
        let row = &archive.rows[0];
        assert_eq!(row.name_bytes(), b"HELLO.BIN");
        assert_eq!(
            (row.method, row.size, row.packed_size),
            (
                u16::from(case.method),
                case.decoded as u64,
                case.packed as u64
            )
        );
        assert!(case.method == case.requested || matches!(case.requested, 8 | 9));
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
        assert_eq!(xdzip_compression::crc16(&actual), case.crc);
    }
}
#[test]
fn mandatory_crc_and_unknown_method_leave_old_output_and_no_staging_files() {
    let cancel = Cancellation::default();
    for unknown in [false, true] {
        let scratch = Scratch::new();
        let mut bytes = stored(b"known archive content");
        if unknown {
            bytes[1] = 14;
        } else {
            bytes[29] ^= 1;
        }
        let source = scratch.0.join("wrong-suffix.zip");
        std::fs::write(&source, bytes).unwrap();
        let archive = Archive::open(&source, &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Arc);
        let target = scratch.0.join("output");
        let opts = options(&target);
        std::fs::write(target.join("HELLO.BIN"), b"old file").unwrap();
        let job = archive.extraction_job(opts, &cancel);
        if unknown {
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
            b"old file"
        );
        assert_eq!(std::fs::read_dir(target).unwrap().count(), 1);
    }
}
#[test]
fn physical_duplicate_namespace_survives_raw_tar_marker_and_nested_zip_payloads() {
    let cancel = Cancellation::default();
    let scratch = Scratch::new();
    let mut marker = vec![b'A'; 600];
    marker[257 - 29..262 - 29].copy_from_slice(b"ustar");
    let zip = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip"),
    )
    .unwrap();
    for (at, payload) in [marker, zip].into_iter().enumerate() {
        let mut bytes = stored(&payload);
        bytes.truncate(bytes.len() - 2);
        bytes.extend_from_slice(&stored(b"second record"));
        let source = scratch.0.join(format!("input-{at}.dat"));
        std::fs::write(&source, bytes).unwrap();
        let archive = Archive::open(&source, &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Arc);
        assert_eq!(archive.rows.len(), 2);
        assert_ne!(archive.rows[0].id, archive.rows[1].id);
        let target = scratch.0.join(format!("out-{at}"));
        let mut selected = options(&target);
        selected.all = false;
        selected.selected = vec![archive.rows[1].id.get().to_string()];
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
fn unknown_neighbor_blocks_whole_check_but_supported_physical_selection_remains_valid() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let mut bytes = stored(b"safe selection");
    bytes.truncate(bytes.len() - 2);
    let mut unknown = stored(b"unsupported neighbor");
    unknown[1] = 14;
    bytes.extend_from_slice(&unknown);
    let source = scratch.0.join("mixed.arc");
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
#[test]
fn empty_arc_is_content_detected_and_later_corruption_never_publishes_partial_rows() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let source = scratch.0.join("empty.bin");
    std::fs::write(&source, [0x1a, 0]).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    assert_eq!(archive.format, xdzip_codec_api::Format::Arc);
    assert!(archive.rows.is_empty());
    let mut bytes = stored(b"first");
    bytes.truncate(bytes.len() - 2);
    let later = bytes.len();
    bytes.extend_from_slice(&stored(b"second"));
    bytes[later] = 0;
    std::fs::write(&source, bytes).unwrap();
    assert!(Archive::open(&source, &cancel).is_err());
}
