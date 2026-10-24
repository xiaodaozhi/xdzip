// crates/xdzip-runtime/src/updating/tests.rs

use super::*;
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir()
            .join("xdzip-update-tests")
            .join(format!(
                "{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
        fs::create_dir_all(path.join("inputs/资料/空目录")).unwrap();
        fs::create_dir(path.join("add")).unwrap();
        fs::write(path.join("inputs/资料/原文件.txt"), b"original bytes").unwrap();
        fs::write(path.join("add/新文件.txt"), b"new bytes").unwrap();
        Self(path)
    }
    fn archive(&self, extension: &str) -> PathBuf {
        let name = format!("archive.{extension}");
        let path = self.0.join(&name);
        let options = creation::Options {
            inputs: vec![self.0.join("inputs/资料").to_string_lossy().into()],
            target: self.0.to_string_lossy().into(),
            name,
            method: if extension == "tar" { 0 } else { 8 },
            level: 6,
            conflict: ConflictPolicy::Ask,
            volume_bytes: 0,
            password: None,
        };
        let mut result = None;
        creation::run(
            creation::Job { options },
            1,
            &Control::default(),
            &mut |s| {
                if s.status.terminal() {
                    result = Some(s);
                }
            },
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        path
    }
    fn job(&self, archive: &Archive, directory: &str, conflict: ConflictPolicy) -> Job {
        archive
            .update_job(
                Options {
                    inputs: vec![self.0.join("add/新文件.txt").to_string_lossy().into()],
                    directory: directory.into(),
                    conflict,
                },
                &Cancellation::default(),
            )
            .unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn execute(job: Job, control: &Control, mut hook: impl FnMut(&Snapshot)) -> Snapshot {
    let mut result = None;
    run(job, 2, control, &mut |s| {
        hook(&s);
        if s.status.terminal() {
            result = Some(s);
        }
    });
    result.unwrap()
}
#[test]
fn three_formats_preserve_old_content_and_add_to_current_directory() {
    for extension in ["zip", "tar", "tar.gz"] {
        let f = Fixture::new();
        let path = f.archive(extension);
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        let result = execute(
            f.job(&archive, "1", ConflictPolicy::Ask),
            &Control::default(),
            |_| {},
        );
        assert_eq!(
            result.status,
            Status::Succeeded,
            "{extension}: {}",
            result.error
        );
        let updated = Archive::open(&path, &Cancellation::default()).unwrap();
        assert!(
            updated
                .rows
                .iter()
                .any(|r| r.raw_name == "资料/新文件.txt".as_bytes())
        );
        assert!(
            updated
                .rows
                .iter()
                .any(|r| r.raw_name == "资料/原文件.txt".as_bytes())
        );
        assert!(
            updated
                .rows
                .iter()
                .any(|r| r.raw_name == "资料/空目录/".as_bytes())
        );
        assert_eq!(result.verified_files, 2);
        if let Some(output) = std::env::var_os("XDZIP_UPDATE_INTEROP_OUTPUT") {
            fs::create_dir_all(&output).unwrap();
            fs::copy(
                &path,
                PathBuf::from(output).join(format!("updated.{extension}")),
            )
            .unwrap();
        }
    }
}
#[test]
fn conflicts_skip_rename_replace_and_prompt_cancellation_preserve_original() {
    for policy in [
        ConflictPolicy::Skip,
        ConflictPolicy::Rename,
        ConflictPolicy::Overwrite,
        ConflictPolicy::Ask,
    ] {
        let f = Fixture::new();
        let path = f.archive("zip");
        let old = fs::read(&path).unwrap();
        fs::write(f.0.join("add/原文件.txt"), b"replacement").unwrap();
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        let job = archive
            .update_job(
                Options {
                    inputs: vec![f.0.join("add/原文件.txt").to_string_lossy().into()],
                    directory: "1".into(),
                    conflict: policy,
                },
                &Cancellation::default(),
            )
            .unwrap();
        let control = Control::default();
        let result = execute(job, &control, |s| {
            if s.status == Status::Waiting {
                control.cancel();
            }
        });
        if policy == ConflictPolicy::Ask {
            assert_eq!(result.status, Status::Cancelled);
            assert_eq!(fs::read(&path).unwrap(), old);
        } else {
            assert_eq!(result.status, Status::Succeeded, "{}", result.error);
            assert_eq!(
                result.files,
                if policy == ConflictPolicy::Rename {
                    2
                } else {
                    1
                }
            );
        }
    }
}
#[test]
fn cancel_before_commit_keeps_original_bytes_in_all_formats() {
    for extension in ["zip", "tar", "tar.gz"] {
        let f = Fixture::new();
        let path = f.archive(extension);
        let old = fs::read(&path).unwrap();
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        let control = Control::default();
        let result = execute(f.job(&archive, "", ConflictPolicy::Ask), &control, |s| {
            if s.status == Status::Verifying {
                control.cancel();
            }
        });
        assert_eq!(result.status, Status::Cancelled);
        assert_eq!(fs::read(path).unwrap(), old);
    }
}
#[test]
fn changed_source_and_source_selected_as_input_never_replace_archive() {
    let f = Fixture::new();
    let path = f.archive("zip");
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    let job = archive
        .update_job(
            Options {
                inputs: vec![path.to_string_lossy().into()],
                directory: "".into(),
                conflict: ConflictPolicy::Overwrite,
            },
            &Cancellation::default(),
        )
        .unwrap();
    let old = fs::read(&path).unwrap();
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.error, "protected_source");
    assert_eq!(fs::read(&path).unwrap(), old);
    let job = f.job(&archive, "", ConflictPolicy::Ask);
    fs::write(&path, b"changed source").unwrap();
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.error, "source_changed");
    assert_eq!(fs::read(path).unwrap(), b"changed source");
}
#[test]
fn export_manifest_deduplicates_parent_and_child_and_keeps_empty_directories() {
    let f = Fixture::new();
    let path = f.archive("zip");
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    let options = crate::extraction::Options {
        target: f.0.to_string_lossy().into(),
        all: false,
        selected: vec!["1".into(), "3".into()],
        directory: "".into(),
        create_folder: false,
        conflict: ConflictPolicy::Ask,
        allow_high_ratio: false,
    };
    let entries = archive
        .export_manifest(&options, &Cancellation::default())
        .unwrap();
    assert_eq!(entries.len(), 3);
    assert!(
        entries
            .iter()
            .any(|e| e.name == "资料/空目录" && e.directory)
    );
}

#[test]
fn corrupted_original_payload_is_never_published_as_a_successful_update() {
    let f = Fixture::new();
    let path = f.archive("zip");
    let mut bytes = fs::read(&path).unwrap();
    let name = "资料/原文件.txt".as_bytes();
    let offset = bytes
        .windows(name.len())
        .position(|part| part == name)
        .unwrap();
    let extra = u16::from_le_bytes([bytes[offset - 2], bytes[offset - 1]]) as usize;
    let payload = offset + name.len() + extra;
    bytes[payload] ^= 0xff;
    fs::write(&path, &bytes).unwrap();
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    let result = execute(
        f.job(&archive, "", ConflictPolicy::Ask),
        &Control::default(),
        |_| {},
    );
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "integrity_error");
    assert_eq!(fs::read(path).unwrap(), bytes);
}
