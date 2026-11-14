//! Actual read-only host tasks for independent UU/XXE transport inputs.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-uu-{}-{}",
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
        .join("../../tests/fixtures/uu")
        .join(name)
}
fn payload() -> Vec<u8> {
    let mut bytes = b"Owned UU/XXE payload.\n".repeat(17);
    bytes.extend(0..=255);
    bytes
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
fn owned_transport_profiles_use_actual_browse_check_extract_tasks() {
    let cancel = Cancellation::default();
    for name in [
        "ordinary.uue",
        "ordinary.xxe",
        "spaces.uu",
        "tails.uue",
        "tails.xxe",
        "empty.uue",
        "empty.xxe",
        "base64.uue",
    ] {
        let scratch = Scratch::new();
        let archive = Archive::open(&fixture(name), &cancel).unwrap();
        assert_eq!(
            archive.format,
            if name.ends_with("xxe") {
                xdzip_codec_api::Format::Xxe
            } else {
                xdzip_codec_api::Format::Uu
            }
        );
        assert!(!archive.update_allowed);
        assert_eq!(archive.rows.len(), 1);
        assert_eq!(archive.rows[0].unix_mode, 0);
        let expected = if name.starts_with("empty") {
            vec![]
        } else if name.starts_with("tails") {
            (0..46).collect()
        } else {
            payload()
        };
        let mut result = None;
        checking::run(
            archive.check_job(&cancel).unwrap(),
            1,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(result.verified_bytes, expected.len() as u64);
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
            std::fs::read(scratch.0.join(if name == "base64.uue" {
                "data.bin"
            } else {
                "hello.bin"
            }))
            .unwrap(),
            expected
        );
    }
}
#[test]
fn duplicate_names_keep_selected_member_identity_without_suffix_trust() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let renamed = scratch.0.join("source.zip");
    std::fs::copy(fixture("multi.uue"), &renamed).unwrap();
    let archive = Archive::open(&renamed, &cancel).unwrap();
    assert_eq!(archive.format, xdzip_codec_api::Format::Uu);
    assert_eq!(archive.rows.len(), 3);
    assert_eq!(archive.rows[0].raw_name, archive.rows[1].raw_name);
    assert_ne!(archive.rows[0].id, archive.rows[1].id);
    let target = scratch.0.join("output");
    let mut selected = options(&target);
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
        std::fs::read(target.join("hello.bin")).unwrap(),
        (0..46).collect::<Vec<u8>>()
    );
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
}
#[test]
fn unsafe_header_paths_and_malformed_members_cannot_publish_files() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let source = scratch.0.join("source.uue");
    let target = scratch.0.join("output");
    for name in ["../outside", "/absolute", "C:\\outside"] {
        std::fs::write(&source, format!("begin 7777 {name}\n`\nend\n")).unwrap();
        let archive = Archive::open(&source, &cancel).unwrap();
        assert!(archive.rows[0].warning);
        assert!(archive.extraction_job(options(&target), &cancel).is_err());
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
    }
    std::fs::write(&source, b"begin 644 hello.bin\n!0000\n`\nended\n").unwrap();
    assert!(Archive::open(&source, &cancel).is_err());
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
}
#[test]
fn queued_cancellation_and_source_changes_preserve_existing_output() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let source = scratch.0.join("source.xxe");
    std::fs::copy(fixture("ordinary.xxe"), &source).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    let target = scratch.0.join("output");
    let opts = options(&target);
    std::fs::write(target.join("hello.bin"), b"keep").unwrap();
    let control = Control::default();
    assert!(control.cancel());
    let mut result = None;
    extraction::run(
        archive.extraction_job(opts.clone(), &cancel).unwrap(),
        1,
        &control,
        &mut |s| result = Some(s),
    );
    assert_eq!(result.unwrap().status, Status::Cancelled);
    let job = archive.extraction_job(opts, &cancel).unwrap();
    let mut changed = std::fs::read(&source).unwrap();
    changed.push(b'\n');
    std::fs::write(&source, changed).unwrap();
    let mut result = None;
    extraction::run(job, 2, &Control::default(), &mut |s| result = Some(s));
    let result = result.unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert_eq!(std::fs::read(target.join("hello.bin")).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
}
