//! Real creation/publication regressions; all inputs are owned temporary fixtures.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "xdzip-create-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        std::fs::create_dir(root.join("输入")).unwrap();
        std::fs::create_dir(root.join("输入/空目录")).unwrap();
        std::fs::write(
            root.join("输入/文件.txt"),
            "中文与 ZIP 互操作\n".repeat(1000),
        )
        .unwrap();
        std::fs::write(root.join("输入/empty.bin"), b"").unwrap();
        std::fs::create_dir(root.join("output")).unwrap();
        Self(root)
    }
    fn job(&self, method: u16) -> Job {
        Job {
            options: Options {
                inputs: vec![self.0.join("输入").to_string_lossy().into_owned()],
                target: self.0.join("output").to_string_lossy().into_owned(),
                name: "archive.zip".into(),
                method,
                level: 6,
                conflict: ConflictPolicy::Ask,
            },
        }
    }
    fn output(&self) -> PathBuf {
        self.0.join("output/archive.zip")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn execute(job: Job, control: &Control, mut hook: impl FnMut(&Snapshot)) -> Snapshot {
    let mut final_state = None;
    run(job, 1, control, &mut |s| {
        hook(&s);
        if s.status.terminal() {
            final_state = Some(s);
        }
    });
    final_state.unwrap()
}
#[test]
fn store_and_deflate_write_real_archives_and_decode_every_record() {
    for method in [0, 8] {
        let f = Fixture::new();
        let result = execute(f.job(method), &Control::default(), |_| {});
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        assert_eq!(result.committed_files, 2);
        assert_eq!(result.created_directories, 2);
        assert!(result.total_known && result.creating);
        assert!(!result.cleanup_warning);
        let mut reader = std::fs::File::open(f.output()).unwrap();
        let cancel = Cancellation::default();
        let rows = ZipCodec
            .scan(&mut reader, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 4);
        assert!(rows.iter().any(|r| r.raw_name == "输入/空目录/".as_bytes()));
        let mut decoder = ZipCodec.open_payload(&mut reader, &rows, &cancel).unwrap();
        for r in &rows {
            decoder
                .decode(
                    r,
                    &mut VerifySink {
                        bytes: 0,
                        limit: r.size,
                        cancel: &cancel,
                    },
                    &cancel,
                )
                .unwrap();
        }
        assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
    }
}
#[test]
fn cancelling_a_conflict_preserves_the_old_archive_and_cleans_staging() {
    let f = Fixture::new();
    std::fs::write(f.output(), b"keep old archive").unwrap();
    let control = Control::default();
    let result = execute(f.job(8), &control, |s| {
        if s.status == Status::Waiting {
            control.cancel();
        }
    });
    assert_eq!(result.status, Status::Cancelled);
    assert_eq!(result.committed_files, 0);
    assert_eq!(std::fs::read(f.output()).unwrap(), b"keep old archive");
    assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
}
#[test]
fn rename_and_overwrite_apply_to_one_complete_verified_archive() {
    for policy in [ConflictPolicy::Rename, ConflictPolicy::Overwrite] {
        let f = Fixture::new();
        std::fs::write(f.output(), b"old").unwrap();
        let mut job = f.job(8);
        job.options.conflict = policy;
        let result = execute(job, &Control::default(), |_| {});
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        if policy == ConflictPolicy::Rename {
            assert_eq!(std::fs::read(f.output()).unwrap(), b"old");
            assert!(f.0.join("output/archive (2).zip").is_file());
        } else {
            assert_ne!(std::fs::read(f.output()).unwrap(), b"old");
        }
    }
}
#[test]
fn output_inside_input_excludes_only_this_output_and_private_workspace() {
    let f = Fixture::new();
    let mut job = f.job(8);
    job.options.target = f.0.join("输入").to_string_lossy().into_owned();
    std::fs::write(f.0.join("输入/archive.zip"), b"old").unwrap();
    job.options.conflict = ConflictPolicy::Overwrite;
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    let mut reader = std::fs::File::open(f.0.join("输入/archive.zip")).unwrap();
    let rows = ZipCodec
        .scan(&mut reader, &Cancellation::default(), ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 4);
    assert!(
        !rows
            .iter()
            .any(|r| r.display_name.contains("archive.zip") || r.display_name.contains(".xdzip"))
    );
}
#[test]
fn source_mutation_before_publication_keeps_existing_destination() {
    let f = Fixture::new();
    std::fs::write(f.output(), b"keep").unwrap();
    let mut job = f.job(8);
    job.options.conflict = ConflictPolicy::Overwrite;
    let result = execute(job, &Control::default(), |s| {
        if s.status == Status::Verifying {
            std::fs::write(f.0.join("输入/文件.txt"), b"changed").unwrap();
        }
    });
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert_eq!(std::fs::read(f.output()).unwrap(), b"keep");
}
#[test]
fn whole_archive_commit_wins_over_late_cancellation() {
    let f = Fixture::new();
    let control = Control::default();
    let result = execute(f.job(8), &control, |s| {
        if s.status == Status::Committing {
            control.cancel();
        }
    });
    assert_eq!(result.status, Status::Succeeded);
    assert!(f.output().is_file());
    assert!(!control.cancel());
}
#[test]
fn duplicate_selection_and_added_children_are_detected_without_recursive_rediscovery() {
    let f = Fixture::new();
    let dir = f.0.join("输入");
    let tree = InputTree::scan(
        &[dir.clone(), dir.join("文件.txt"), dir.clone()],
        vec![],
        &|| false,
    )
    .unwrap();
    assert_eq!(tree.entries().len(), 4);
    std::fs::write(dir.join("added.txt"), b"new").unwrap();
    assert_eq!(tree.validate(&|| false), Err(xdzip_io::InputError::Changed));
}
#[test]
fn colliding_names_and_link_inputs_fail_before_publication() {
    let f = Fixture::new();
    std::fs::write(f.0.join("输入/a.txt"), b"a").unwrap();
    std::fs::write(f.0.join("输入/A.TXT"), b"b").unwrap();
    // On case-insensitive filesystems the second path is the same ordinary file.
    let names: Vec<_> = std::fs::read_dir(f.0.join("输入"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    if names.iter().any(|n| n == "a.txt") && names.iter().any(|n| n == "A.TXT") {
        let result = execute(f.job(8), &Control::default(), |_| {});
        assert_eq!(result.error, "input_collision");
        assert!(!f.output().exists());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(f.0.join("输入/文件.txt"), f.0.join("link")).unwrap();
        let mut job = f.job(8);
        job.options.inputs = vec![f.0.join("link").to_string_lossy().into_owned()];
        assert_eq!(
            execute(job, &Control::default(), |_| {}).error,
            "unsafe_input"
        );
    }
}

#[test]
fn cancellation_during_encoding_never_publishes_a_partial_archive() {
    let f = Fixture::new();
    let file = std::fs::File::create(f.0.join("输入/large.bin")).unwrap();
    file.set_len(64 * 1024 * 1024).unwrap();
    std::fs::write(f.output(), b"keep").unwrap();
    let mut job = f.job(8);
    job.options.conflict = ConflictPolicy::Overwrite;
    let control = Control::default();
    let result = execute(job, &control, |s| {
        if s.status == Status::Running && s.decoded_bytes > 0 {
            control.cancel();
        }
    });
    assert_eq!(result.status, Status::Cancelled);
    assert_eq!(result.committed_files, 0);
    assert_eq!(std::fs::read(f.output()).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
}

#[test]
fn corruption_of_staged_bytes_is_rejected_before_publication() {
    use std::io::{Seek, SeekFrom, Write};
    let f = Fixture::new();
    std::fs::write(f.output(), b"keep").unwrap();
    let mut job = f.job(0);
    job.options.conflict = ConflictPolicy::Overwrite;
    let result = execute(job, &Control::default(), |s| {
        if s.status == Status::Verifying {
            let workspace = std::fs::read_dir(f.0.join("output"))
                .unwrap()
                .map(|e| e.unwrap().path())
                .find(|p| p.is_dir())
                .unwrap();
            let stage = std::fs::read_dir(workspace)
                .unwrap()
                .map(|e| e.unwrap().path())
                .find(|p| p.extension().is_some_and(|ext| ext == "part"))
                .unwrap();
            let mut file = std::fs::OpenOptions::new().write(true).open(stage).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
            file.write_all(b"BROKEN").unwrap();
        }
    });
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "integrity_error");
    assert_eq!(std::fs::read(f.output()).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn parent_aliases_match_output_exclusions_without_following_leaf_links() {
    let f = Fixture::new();
    std::os::unix::fs::symlink(&f.0, f.0.join("alias")).unwrap();
    let mut job = f.job(8);
    job.options.inputs = vec![f.0.join("alias/输入").to_string_lossy().into()];
    job.options.target = f.0.join("输入").to_string_lossy().into();
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(result.created_directories, 2);
    assert_eq!(result.committed_files, 2);
    // The selected leaf itself is still a link and must be rejected.
    let mut job = f.job(8);
    job.options.inputs = vec![f.0.join("alias").to_string_lossy().into()];
    assert_eq!(
        execute(job, &Control::default(), |_| {}).error,
        "unsafe_input"
    );
}

#[test]
fn changed_destination_is_not_replaced_after_commit_permission() {
    let f = Fixture::new();
    std::fs::write(f.output(), b"original").unwrap();
    let mut job = f.job(8);
    job.options.conflict = ConflictPolicy::Overwrite;
    let result = execute(job, &Control::default(), |s| {
        if s.status == Status::Committing {
            std::fs::write(f.output(), b"new unrelated version").unwrap();
        }
    });
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "target_changed");
    assert_eq!(std::fs::read(f.output()).unwrap(), b"new unrelated version");
    assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn directory_link_swap_after_scan_is_rejected() {
    let f = Fixture::new();
    let result = execute(f.job(8), &Control::default(), |s| {
        if s.status == Status::Running {
            std::fs::rename(f.0.join("输入"), f.0.join("original-input")).unwrap();
            std::os::unix::fs::symlink(f.0.join("original-input"), f.0.join("输入")).unwrap();
        }
    });
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert!(!f.output().exists());
}
