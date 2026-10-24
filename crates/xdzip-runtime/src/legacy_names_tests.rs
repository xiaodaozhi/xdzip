// crates/xdzip-runtime/src/legacy_names_tests.rs

use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Snapshot, Status};
use std::{fs, path::PathBuf};
#[path = "../../../tests/support/legacy_names.rs"]
mod fixture;
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "xdzip-legacy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn options(&self, target: &str) -> Options {
        let target = self.0.join(target);
        fs::create_dir(&target).unwrap();
        Options {
            target: target.to_string_lossy().into(),
            all: true,
            selected: vec![],
            directory: String::new(),
            create_folder: false,
            conflict: ConflictPolicy::Ask,
            allow_high_ratio: false,
        }
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn extract(job: extraction::Job) -> Snapshot {
    let mut result = None;
    extraction::run(job, 1, &Control::default(), &mut |s| result = Some(s));
    result.unwrap()
}
fn update(job: updating::Job) -> Snapshot {
    let mut result = None;
    updating::run(job, 2, &Control::default(), &mut |s| result = Some(s));
    result.unwrap()
}
#[test]
fn legacy_names_flow_through_navigation_queries_preflight_extract_add_and_move() {
    for (encoding, sample) in fixture::samples() {
        for extension in ["zip", "tar", "tar.gz"] {
            // CP437 is ZIP's specified fallback; Unicode Path is a ZIP-only declaration.
            if extension != "zip" && matches!(encoding, "CP437" | "UnicodePath") {
                continue;
            }
            let temp = Temp::new();
            let path = temp.0.join(format!("owned.{extension}"));
            let mut names: Vec<_> = sample.iter().map(|(raw, _)| *raw).collect();
            names.push(b"target/");
            let bytes = if extension == "zip" {
                fixture::zip_sample(&sample, encoding, true)
            } else {
                fixture::tar(&names)
            };
            let bytes = if extension == "tar.gz" {
                use std::io::Write;
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
                encoder.write_all(&bytes).unwrap();
                encoder.finish().unwrap()
            } else {
                bytes
            };
            fs::write(&path, &bytes).unwrap();
            let cancel = Cancellation::default();
            let archive = Archive::open(&path, &cancel).unwrap();
            for (row, (raw, expected)) in archive.rows.iter().zip(&sample) {
                assert_eq!(row.raw_name, *raw);
                assert_eq!(row.display_name, *expected, "{encoding}/{extension}");
                assert_eq!(row.name_bytes(), expected.as_bytes());
                assert!(!row.warning);
            }
            let first = &archive.rows[0];
            let directory = if first.directory {
                first.id.get().to_string()
            } else {
                String::new()
            };
            let report = archive.preflight(&[first.id], &cancel).unwrap();
            assert_eq!(report.finding_count, 0);
            assert!(report.unicode_review);
            if first.directory {
                assert!(
                    archive
                        .directories
                        .as_ref()
                        .unwrap()
                        .iter()
                        .any(|d| d.path == sample[0].1)
                );
                let (count, rows) = archive
                    .listing_sorted(&directory, "", 0, 128, 0, &cancel)
                    .unwrap();
                assert_eq!(count, sample.len() - 1);
                assert!(rows.iter().all(|r| r.path().starts_with(sample[0].1)));
                let (_, matches) = archive
                    .listing_sorted(
                        &directory,
                        sample[1].1.rsplit('/').next().unwrap(),
                        0,
                        128,
                        0,
                        &cancel,
                    )
                    .unwrap();
                assert_eq!(matches.len(), 1);
            }
            let mut options = temp.options("extract");
            options.all = false;
            options.selected = vec![first.id.get().to_string()];
            let manifest = archive.export_manifest(&options, &cancel).unwrap();
            assert!(manifest.iter().any(|e| e.name == sample.last().unwrap().1));
            let done = extract(archive.extraction_job(options, &cancel).unwrap());
            assert_eq!(
                done.status,
                Status::Succeeded,
                "{encoding}/{extension}: {}",
                done.error
            );
            for (_, expected) in &sample {
                if !expected.ends_with('/') {
                    assert_eq!(
                        fs::read(temp.0.join("extract").join(expected)).unwrap(),
                        fixture::CONTENT
                    );
                }
            }
            assert_eq!(fs::read(&path).unwrap(), bytes);
            let file = archive.rows.iter().find(|r| !r.directory).unwrap();
            fs::create_dir(temp.0.join("cache")).unwrap();
            let open = archive
                .open_entry_job(file.id.get(), &temp.0.join("cache"), &cancel)
                .unwrap();
            assert_eq!(
                open.planned[0].path.display(),
                file.display_name.rsplit('/').next().unwrap()
            );
            let lease = open.temporary.clone().unwrap();
            let done = extract(open);
            assert_eq!(done.status, Status::Succeeded, "{}", done.error);
            assert_eq!(fs::read(&done.open_path).unwrap(), fixture::CONTENT);
            drop(lease);
            let add = temp.0.join("added.txt");
            fs::write(&add, b"new owned input").unwrap();
            let job = archive
                .update_job(
                    updating::Options {
                        inputs: vec![add.to_string_lossy().into()],
                        directory: directory.clone(),
                        conflict: ConflictPolicy::Ask,
                    },
                    &cancel,
                )
                .unwrap();
            let done = update(job);
            assert_eq!(
                done.status,
                Status::Succeeded,
                "{encoding}/{extension}: {}",
                done.error
            );
            drop(archive);
            let archive = Archive::open(&path, &cancel).unwrap();
            let added = format!(
                "{}added.txt",
                if first_directory(&sample) {
                    sample[0].1
                } else {
                    ""
                }
            );
            assert!(archive.rows.iter().any(|r| r.display_name == added));
            assert!(
                archive
                    .rows
                    .iter()
                    .all(|r| std::str::from_utf8(&r.raw_name).is_ok() && r.decoded_name.is_none())
            );
            let moving = archive
                .rows
                .iter()
                .find(|r| {
                    r.display_name == sample.iter().find(|(_, n)| !n.ends_with('/')).unwrap().1
                })
                .unwrap();
            let target = archive
                .rows
                .iter()
                .find(|r| r.display_name == "target/")
                .unwrap()
                .id
                .get()
                .to_string();
            let source_directory = if first_directory(&sample) {
                archive
                    .rows
                    .iter()
                    .find(|r| r.display_name == sample[0].1)
                    .unwrap()
                    .id
                    .get()
                    .to_string()
            } else {
                String::new()
            };
            let moved_name = format!("target/{}", moving.display_name.rsplit('/').next().unwrap());
            let job = archive
                .move_job(
                    updating::MoveOptions {
                        selected: vec![moving.id.get().to_string()],
                        directory: source_directory,
                        target,
                        conflict: ConflictPolicy::Ask,
                    },
                    &cancel,
                )
                .unwrap();
            let done = update(job);
            assert_eq!(
                done.status,
                Status::Succeeded,
                "{encoding}/{extension}: {}",
                done.error
            );
            drop(archive);
            let archive = Archive::open(&path, &cancel).unwrap();
            assert!(archive.rows.iter().any(|r| r.display_name == moved_name));
            assert!(
                archive
                    .rows
                    .iter()
                    .all(|r| std::str::from_utf8(&r.raw_name).is_ok())
            );
            let options = temp.options("verify");
            let done = extract(archive.extraction_job(options, &cancel).unwrap());
            assert_eq!(done.status, Status::Succeeded, "{}", done.error);
            assert_eq!(
                fs::read(temp.0.join("verify").join(moved_name)).unwrap(),
                fixture::CONTENT
            );
            if let Some(output) = std::env::var_os("XDZIP_LEGACY_INTEROP_OUTPUT") {
                let output = PathBuf::from(output).join(encoding).join(extension);
                fs::create_dir_all(&output).unwrap();
                fs::write(
                    output.join(format!("rewritten.{extension}")),
                    fs::read(&path).unwrap(),
                )
                .unwrap();
                fs::write(
                    output.join("names.txt"),
                    archive
                        .rows
                        .iter()
                        .map(|r| r.display_name.as_str())
                        .collect::<Vec<_>>()
                        .join("\n"),
                )
                .unwrap();
            }
        }
    }
}
fn first_directory(sample: &[(&[u8], &str)]) -> bool {
    sample[0].1.ends_with('/')
}
#[test]
fn decoded_aliases_reenter_collision_policy_and_unsafe_names_never_start_writes() {
    let temp = Temp::new();
    let path = temp.0.join("aliases.zip");
    let cancel = Cancellation::default();
    let bytes = fixture::zip(&[(b"caf\x82.txt", 0, &[]), ("café.txt".as_bytes(), 2048, &[])]);
    fs::write(&path, &bytes).unwrap();
    let archive = Archive::open(&path, &cancel).unwrap();
    let report = archive
        .preflight(&[archive.rows[0].id, archive.rows[1].id], &cancel)
        .unwrap();
    assert!(report.findings.iter().any(|f| f.reason == "collision"));
    let mut options = temp.options("aliases");
    options.conflict = ConflictPolicy::Skip;
    let done = extract(archive.extraction_job(options, &cancel).unwrap());
    assert_eq!(done.status, Status::Succeeded);
    assert_eq!(done.committed_files, 1);
    assert_eq!(done.skipped_files, 1);
    assert_eq!(fs::read_dir(temp.0.join("aliases")).unwrap().count(), 1);
    let add = temp.0.join("added.txt");
    fs::write(&add, b"new").unwrap();
    let done = update(
        archive
            .update_job(
                updating::Options {
                    inputs: vec![add.to_string_lossy().into()],
                    directory: String::new(),
                    conflict: ConflictPolicy::Ask,
                },
                &cancel,
            )
            .unwrap(),
    );
    assert_eq!(done.status, Status::Failed);
    assert_eq!(done.error, "name_collision");
    assert_eq!(fs::read(&path).unwrap(), bytes);
    drop(archive);
    let extra = fixture::unicode_extra(b"safe.txt", b"../outside.txt");
    fs::write(&path, fixture::zip(&[(b"safe.txt", 0, &extra)])).unwrap();
    let archive = Archive::open(&path, &cancel).unwrap();
    let options = temp.options("refused");
    assert!(matches!(
        archive.extraction_job(options, &cancel),
        Err("unsupported_entry")
    ));
    assert_eq!(fs::read_dir(temp.0.join("refused")).unwrap().count(), 0);
}

#[test]
fn numbered_split_legacy_zip_keeps_decoded_names_and_read_only_edit_boundary() {
    let temp = Temp::new();
    let cancel = Cancellation::default();
    let sample = fixture::samples()
        .into_iter()
        .find(|(label, _)| *label == "GBK")
        .unwrap()
        .1;
    let bytes = fixture::zip_sample(&sample, "GBK", false);
    let mut volumes = Vec::new();
    for (i, chunk) in bytes.chunks(64).enumerate() {
        let path = temp.0.join(format!("owned.zip.{:03}", i + 1));
        fs::write(&path, chunk).unwrap();
        volumes.push(path);
    }
    let archive = Archive::open(&volumes[1], &cancel).unwrap();
    assert!(archive.source.is_split());
    assert!(!archive.can_update());
    assert_eq!(archive.rows[1].display_name, sample[1].1);
    let options = temp.options("split");
    let done = extract(archive.extraction_job(options, &cancel).unwrap());
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(
        fs::read(temp.0.join("split").join(sample[1].1)).unwrap(),
        fixture::CONTENT
    );
}

#[test]
fn decoded_directory_aliases_repeat_unicode_and_case_checks_before_publication() {
    let temp = Temp::new();
    let path = temp.0.join("aliases.zip");
    let cancel = Cancellation::default();
    let bytes = fixture::zip(&[
        (b"caf\x82/a.txt", 0, &[]),
        ("CAFE\u{301}/b.txt".as_bytes(), 2048, &[]),
    ]);
    fs::write(&path, &bytes).unwrap();
    let archive = Archive::open(&path, &cancel).unwrap();
    assert!(archive.rows.iter().all(|r| !r.warning));
    let options = temp.options("refused");
    assert!(matches!(
        archive.extraction_job(options, &cancel),
        Err("ambiguous_name")
    ));
    assert_eq!(fs::read_dir(temp.0.join("refused")).unwrap().count(), 0);
    assert_eq!(fs::read(path).unwrap(), bytes);
}

#[test]
fn first_edit_can_move_legacy_entries_without_a_prior_addition() {
    let sample = fixture::samples()
        .into_iter()
        .find(|(label, _)| *label == "GBK")
        .unwrap()
        .1;
    for extension in ["zip", "tar"] {
        let temp = Temp::new();
        let path = temp.0.join(format!("owned.{extension}"));
        let cancel = Cancellation::default();
        let mut names: Vec<_> = sample.iter().map(|(raw, _)| *raw).collect();
        names.push(b"target/");
        let bytes = if extension == "zip" {
            fixture::zip_sample(&sample, "GBK", true)
        } else {
            fixture::tar(&names)
        };
        fs::write(&path, bytes).unwrap();
        let archive = Archive::open(&path, &cancel).unwrap();
        let target = archive.rows.last().unwrap().id.get().to_string();
        let job = archive
            .move_job(
                updating::MoveOptions {
                    selected: vec![archive.rows[1].id.get().to_string()],
                    directory: archive.rows[0].id.get().to_string(),
                    target,
                    conflict: ConflictPolicy::Ask,
                },
                &cancel,
            )
            .unwrap();
        let done = update(job);
        assert_eq!(done.status, Status::Succeeded, "{}", done.error);
        drop(archive);
        let archive = Archive::open(&path, &cancel).unwrap();
        assert!(archive.rows.iter().any(
            |r| r.display_name == format!("target/{}", sample[1].1.rsplit('/').next().unwrap())
        ));
        assert!(archive.rows.iter().any(|r| r.display_name == sample[2].1));
        assert!(
            archive
                .rows
                .iter()
                .all(|r| std::str::from_utf8(&r.raw_name).is_ok())
        );
    }
}

#[test]
fn unresolved_descendants_are_not_silently_omitted_from_decoded_directory_selection() {
    let temp = Temp::new();
    let path = temp.0.join("owned.zip");
    let cancel = Cancellation::default();
    let bytes = fixture::zip(&[
        (b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/", 0, &[]),
        (
            b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/\xd6\xd0\xce\xc4.txt",
            0,
            &[],
        ),
        (b"\xd6\xd0\xce\xc4\xd7\xca\xc1\xcf/bad.txt", 2048, &[]),
    ]);
    fs::write(&path, bytes).unwrap();
    let archive = Archive::open(&path, &cancel).unwrap();
    assert_eq!(archive.rows[0].display_name, "中文资料/");
    assert!(archive.rows[2].warning);
    let report = archive.preflight(&[archive.rows[0].id], &cancel).unwrap();
    assert!(
        report
            .findings
            .iter()
            .any(|f| f.entry == archive.rows[2].id && f.reason == "metadata")
    );
    let mut options = temp.options("refused");
    options.all = false;
    options.selected = vec![archive.rows[0].id.get().to_string()];
    assert!(matches!(
        archive.extraction_job(options, &cancel),
        Err("unsupported_entry")
    ));
    assert_eq!(fs::read_dir(temp.0.join("refused")).unwrap().count(), 0);
    let mut options = temp.options("explicit");
    options.all = false;
    options.selected = vec![archive.rows[1].id.get().to_string()];
    let done = extract(archive.extraction_job(options, &cancel).unwrap());
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(
        fs::read(temp.0.join("explicit/中文资料/中文.txt")).unwrap(),
        fixture::CONTENT
    );
}
