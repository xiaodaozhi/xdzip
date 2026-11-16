//! Real host task coverage for owned ASAR inputs, without application execution.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-asar-{}-{}",
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
        .join("../../tests/fixtures/asar")
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
fn asar_owned_profiles_browse_check_and_export_actual_stored_files() {
    let cancel = Cancellation::default();
    for (name, count) in [
        ("ordinary.asar", 6),
        ("without-integrity.asar", 6),
        ("shared.asar", 3),
        ("executable.asar", 1),
        ("empty.asar", 0),
    ] {
        let scratch = Scratch::new();
        let archive = Archive::open(&fixture(name), &cancel).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::Asar);
        assert!(!archive.update_allowed);
        assert_eq!(archive.rows.len(), count);
        assert!(
            archive
                .rows
                .iter()
                .all(|row| row.unix_mode == 0 && row.payload_supported())
        );
        let mut result = None;
        checking::run(
            archive.check_job(&cancel).unwrap(),
            1,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(
            result.verified_bytes,
            archive.rows.iter().map(|row| row.size).sum::<u64>()
        );
        if count == 0 {
            continue;
        }
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
        for row in archive.rows.iter().filter(|row| !row.directory) {
            let actual = std::fs::read(
                scratch
                    .0
                    .join(std::str::from_utf8(row.name_bytes()).unwrap()),
            )
            .unwrap();
            let expected = if row.display_name.contains("资料") {
                "owned UTF-8 ASAR content\n资料\n".as_bytes().to_vec()
            } else if row.size == 0 {
                vec![]
            } else {
                b"abracadabra abracadabra\n".repeat(6)
            };
            assert_eq!(actual, expected);
        }
    }
}
#[test]
fn shared_range_selection_keeps_distinct_physical_identities() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let archive = Archive::open(&fixture("shared.asar"), &cancel).unwrap();
    assert_ne!(archive.rows[0].id, archive.rows[1].id);
    let mut selected = options(&scratch.0);
    selected.all = false;
    selected.selected = vec![archive.rows[1].id.get().to_string()];
    let mut result = None;
    extraction::run(
        archive.extraction_job(selected, &cancel).unwrap(),
        1,
        &Control::default(),
        &mut |s| result = Some(s),
    );
    assert_eq!(result.unwrap().status, Status::Succeeded);
    assert_eq!(
        std::fs::read(scratch.0.join("copy.txt")).unwrap(),
        b"abracadabra abracadabra\n".repeat(6)
    );
    assert!(!scratch.0.join("hello.txt").exists());
}
#[test]
fn external_and_link_declarations_fail_before_output_grants() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    for name in ["external.asar", "link.asar"] {
        let archive = Archive::open(&fixture(name), &cancel).unwrap();
        assert!(archive.rows[0].warning && !archive.rows[0].payload_supported());
        assert!(
            archive
                .extraction_job(options(&scratch.0), &cancel)
                .is_err()
        );
        assert_eq!(std::fs::read_dir(&scratch.0).unwrap().count(), 0);
    }
}
#[test]
fn damaged_checks_cancelled_jobs_and_source_changes_preserve_existing_files() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let source = scratch.0.join("source.asar");
    let mut bytes = std::fs::read(fixture("shared.asar")).unwrap();
    let data = 8 + u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
    bytes[data] ^= 1;
    std::fs::write(&source, &bytes).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    let target = scratch.0.join("target");
    std::fs::create_dir(&target).unwrap();
    std::fs::write(target.join("hello.txt"), b"keep").unwrap();
    let mut result = None;
    extraction::run(
        archive.extraction_job(options(&target), &cancel).unwrap(),
        1,
        &Control::default(),
        &mut |s| result = Some(s),
    );
    let result = result.unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "integrity_error");
    assert_eq!(std::fs::read(target.join("hello.txt")).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
    std::fs::copy(fixture("shared.asar"), &source).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    let control = Control::default();
    assert!(control.cancel());
    let mut result = None;
    extraction::run(
        archive.extraction_job(options(&target), &cancel).unwrap(),
        2,
        &control,
        &mut |s| result = Some(s),
    );
    assert_eq!(result.unwrap().status, Status::Cancelled);
    assert_eq!(std::fs::read(target.join("hello.txt")).unwrap(), b"keep");
    let job = archive.extraction_job(options(&target), &cancel).unwrap();
    bytes.push(0);
    std::fs::write(&source, bytes).unwrap();
    let mut result = None;
    extraction::run(job, 3, &Control::default(), &mut |s| result = Some(s));
    let result = result.unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert_eq!(std::fs::read(target.join("hello.txt")).unwrap(), b"keep");
}
