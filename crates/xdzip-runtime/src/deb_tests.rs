//! Product-host tasks for owned DEB inputs, without installing their contents.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-deb-{}-{}",
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
        .join("../../tests/fixtures/deb")
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
        conflict: ConflictPolicy::Ask,
        allow_high_ratio: false,
    }
}
#[test]
fn deb_all_profiles_browse_check_and_export_raw_control_and_data_views() {
    let cancel = Cancellation::default();
    for control in ["none", "gz", "xz", "zst"] {
        for data in ["none", "gz", "xz", "zst", "bz2", "lzma"] {
            let scratch = Scratch::new();
            let name = format!("control-{control}-data-{data}.deb");
            let archive = Archive::open(&fixture(&name), &cancel).unwrap();
            assert_eq!(archive.format, xdzip_codec_api::Format::Deb);
            assert!(!archive.update_allowed);
            assert_eq!(archive.rows.len(), 9);
            for (row, listed) in archive
                .rows
                .iter()
                .zip(archive.catalog(0, 128, &cancel).unwrap())
            {
                assert_eq!(listed.packed_size.is_some(), row.method == 0);
            }
            let roots: Vec<_> = archive
                .directories
                .as_ref()
                .unwrap()
                .iter()
                .filter(|d| d.parent == 0)
                .map(|d| d.path.as_str())
                .collect();
            assert_eq!(roots, ["members/", "control/", "data/"]);
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
            assert_eq!(checked.verified_files, 8);
            assert_eq!(
                checked.verified_bytes,
                archive
                    .rows
                    .iter()
                    .filter(|r| !r.directory)
                    .map(|r| r.size)
                    .sum::<u64>()
            );
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
                std::fs::read(scratch.0.join("members/debian-binary")).unwrap(),
                b"2.0\n"
            );
            assert!(
                std::fs::read(scratch.0.join("control/control"))
                    .unwrap()
                    .starts_with(b"Package: xdzip-owned-fixture\n")
            );
            assert_eq!(
                std::fs::read(scratch.0.join("control/postinst")).unwrap(),
                b"#!/bin/sh\necho owned-test-only\n"
            );
            assert_eq!(
                std::fs::read(scratch.0.join("data/usr/share/xdzip/hello.txt")).unwrap(),
                b"abracadabra abracadabra\n".repeat(6)
            );
            assert_eq!(
                std::fs::read(scratch.0.join("data/usr/share/xdzip/资料.txt")).unwrap(),
                b"owned UTF-8 content\n"
            );
            assert!(
                std::fs::read(scratch.0.join("data/usr/share/xdzip/empty.bin"))
                    .unwrap()
                    .is_empty()
            );
        }
    }
}
#[test]
fn deb_virtual_view_and_physical_duplicate_selections_export_only_the_requested_content() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let archive = Archive::open(&fixture("control-gz-data-zst.deb"), &cancel).unwrap();
    let data = archive
        .directories
        .as_ref()
        .unwrap()
        .iter()
        .find(|d| d.path == "data/")
        .unwrap();
    let mut selected = options(&scratch.0.join("view"));
    selected.all = false;
    selected.selected = vec![format!("d{}", data.id)];
    let mut result = None;
    extraction::run(
        archive.extraction_job(selected, &cancel).unwrap(),
        1,
        &Control::default(),
        &mut |s| result = Some(s),
    );
    let result = result.unwrap();
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert!(
        scratch
            .0
            .join("view/data/usr/share/xdzip/hello.txt")
            .is_file()
    );
    assert!(!scratch.0.join("view/members").exists());
    assert!(!scratch.0.join("view/control").exists());
    let archive = Archive::open(&fixture("duplicates.deb"), &cancel).unwrap();
    let duplicate = archive
        .rows
        .iter()
        .rfind(|r| r.display_name == "data/usr/share/xdzip/hello.txt")
        .unwrap();
    let mut selected = options(&scratch.0.join("physical"));
    selected.all = false;
    selected.selected = vec![duplicate.id.get().to_string()];
    let mut result = None;
    extraction::run(
        archive.extraction_job(selected, &cancel).unwrap(),
        2,
        &Control::default(),
        &mut |s| result = Some(s),
    );
    let result = result.unwrap();
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(
        std::fs::read(scratch.0.join("physical/data/usr/share/xdzip/hello.txt")).unwrap(),
        b"second physical record\n"
    );
}
#[test]
fn deb_cancelled_and_changed_source_jobs_do_not_publish_files() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let source = scratch.0.join("source.deb");
    std::fs::copy(fixture("control-gz-data-zst.deb"), &source).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    let target = scratch.0.join("cancelled");
    let job = archive.extraction_job(options(&target), &cancel).unwrap();
    let control = Control::default();
    assert!(control.cancel());
    let mut result = None;
    extraction::run(job, 1, &control, &mut |s| result = Some(s));
    assert_eq!(result.unwrap().status, Status::Cancelled);
    assert!(!target.join("members/debian-binary").exists());
    let target = scratch.0.join("changed");
    let job = archive.extraction_job(options(&target), &cancel).unwrap();
    let mut bytes = std::fs::read(&source).unwrap();
    bytes.push(0);
    std::fs::write(&source, bytes).unwrap();
    let mut result = None;
    extraction::run(job, 2, &Control::default(), &mut |s| result = Some(s));
    let result = result.unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert!(!target.join("members/debian-binary").exists());
}
