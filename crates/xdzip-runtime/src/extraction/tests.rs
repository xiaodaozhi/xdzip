use super::*;
use crate::Archive;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(1);
pub(super) struct Target(pub(super) PathBuf);
impl Target {
    pub(super) fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "xdzip-task-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&p).unwrap();
        Self(p)
    }
    pub(super) fn options(&self) -> Options {
        Options {
            target: self.0.to_string_lossy().into(),
            all: true,
            selected: vec![],
            directory: String::new(),
            create_folder: false,
            shell_layout: 0,
            conflict: ConflictPolicy::Ask,
            allow_high_ratio: false,
        }
    }
}
impl Drop for Target {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
pub(super) fn archive(name: &str) -> Archive {
    Archive::open(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/extract")
            .join(name),
        &Cancellation::default(),
    )
    .unwrap()
}
pub(super) fn execute(
    archive: &Archive,
    options: Options,
    control: &Control,
    mut on_event: impl FnMut(&Snapshot),
) -> Snapshot {
    let job = archive
        .extraction_job(options, &Cancellation::default())
        .unwrap();
    let mut last = None;
    run(job, 1, control, &mut |s| {
        on_event(&s);
        last = Some(s);
    });
    last.unwrap()
}
#[test]
fn actual_store_deflate_and_both_descriptor_widths_match_independent_fixture_bytes() {
    for name in [
        "stored.zip",
        "deflate.zip",
        "descriptor.zip",
        "descriptor64.zip",
    ] {
        let target = Target::new();
        let a = archive(name);
        let done = execute(&a, target.options(), &Control::default(), |_| {});
        assert_eq!(done.status, Status::Succeeded, "{name}: {}", done.error);
        let file = a.rows.iter().find(|e| !e.directory).unwrap();
        let actual =
            std::fs::read(target.0.join(std::str::from_utf8(&file.raw_name).unwrap())).unwrap();
        assert_eq!(actual.len() as u64, file.size);
        assert_eq!(done.committed_bytes, file.size);
        assert_eq!(done.committed_files, 1);
        assert!(!done.cleanup_warning);
        assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 1);
    }
}
#[test]
fn integrity_failures_never_replace_old_files_and_remove_private_staging() {
    for name in [
        "crc_bad.zip",
        "trailing.zip",
        "no_end.zip",
        "header_bad.zip",
        "descriptor_bad.zip",
        "descriptor64_bad.zip",
    ] {
        let target = Target::new();
        let a = archive(name);
        let path = target
            .0
            .join(std::str::from_utf8(&a.rows[0].raw_name).unwrap());
        std::fs::write(&path, b"original important content").unwrap();
        let mut options = target.options();
        options.conflict = ConflictPolicy::Overwrite;
        let done = execute(&a, options, &Control::default(), |_| {});
        assert_eq!(done.status, Status::Failed, "{name}");
        assert_eq!(done.error, "integrity_error", "{name}");
        assert_eq!(done.committed_files, 0);
        assert_eq!(std::fs::read(path).unwrap(), b"original important content");
        assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 1);
        assert!(!done.cleanup_warning);
    }
}
#[test]
fn virtual_directory_selection_and_portable_tree_do_not_flatten_paths() {
    let target = Target::new();
    let a = archive("tree.zip");
    let mut options = target.options();
    options.all = false;
    options.selected = vec![format!(
        "d{}",
        a.directories()
            .unwrap()
            .iter()
            .find(|d| d.name == "照片")
            .unwrap()
            .id
    )];
    let done = execute(&a, options, &Control::default(), |_| {});
    assert_eq!(done.status, Status::Succeeded);
    assert!(target.0.join("照片/说明.txt").is_file());
    assert!(!target.0.join("data.bin").exists());
    assert_eq!(done.created_directories, 1);
    let target = Target::new();
    let done = execute(&a, target.options(), &Control::default(), |_| {});
    assert_eq!(done.status, Status::Succeeded);
    assert_eq!(done.committed_files, 3);
    assert_eq!(
        std::fs::read(target.0.join("data.bin")).unwrap(),
        (0..=255u8).cycle().take(262144).collect::<Vec<_>>()
    );
}
#[test]
fn ordinary_conflict_responses_are_single_use_and_cancel_wakes_waiter() {
    let target = Target::new();
    let a = archive("stored.zip");
    let leaf = std::str::from_utf8(&a.rows[0].raw_name).unwrap();
    std::fs::write(target.0.join(leaf), b"old").unwrap();
    let control = Control::default();
    let done = execute(&a, target.options(), &control, |s| {
        if let Some(p) = &s.prompt {
            assert!(!control.decide(p.id + 1, ConflictPolicy::Overwrite, false));
            assert!(control.decide(p.id, ConflictPolicy::Rename, true));
            assert!(!control.decide(p.id, ConflictPolicy::Overwrite, false));
        }
    });
    assert_eq!(done.status, Status::Succeeded);
    assert_eq!(done.committed_files, 1);
    assert_eq!(std::fs::read(target.0.join(leaf)).unwrap(), b"old");
    let control = Control::default();
    let done = execute(&a, target.options(), &control, |s| {
        if s.prompt.is_some() {
            control.cancel();
        }
    });
    assert_eq!(done.status, Status::Cancelled);
    assert_eq!(done.committed_files, 0);
    assert_eq!(std::fs::read(target.0.join(leaf)).unwrap(), b"old");
}
#[test]
fn native_filename_equivalence_preserves_content_paths_and_reports_changes() {
    let target = Target::new();
    assert_native_filename_output(&target);
    if let Some(path) = std::env::var_os("XDZIP_TEST_CASE_SENSITIVE_TARGET") {
        let target = Target(PathBuf::from(path).join("native-names"));
        std::fs::create_dir(&target.0).unwrap();
        assert!(distinct_names(&target, "CaseProbe", "caseprobe"));
        assert_native_filename_output(&target);
    }
}

fn distinct_names(target: &Target, first: &str, second: &str) -> bool {
    let first = target.0.join(first);
    let second = target.0.join(second);
    std::fs::File::create_new(&first).unwrap();
    let distinct = match std::fs::File::create_new(&second) {
        Ok(file) => {
            drop(file);
            std::fs::remove_file(&second).unwrap();
            true
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => false,
        Err(e) => panic!("{e}"),
    };
    std::fs::remove_file(first).unwrap();
    distinct
}

fn assert_native_filename_output(target: &Target) {
    let case = distinct_names(target, "CaseProbe", "caseprobe");
    let accented_case = distinct_names(target, "Ä-probe", "ä-probe");
    let unicode = distinct_names(target, "é-probe", "e\u{301}-probe");
    let a = archive("native_names.zip");
    let mut options = target.options();
    options.conflict = ConflictPolicy::Rename;
    let done = execute(&a, options, &Control::default(), |_| {});
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(done.committed_files, 11);
    assert_eq!(done.skipped_files, 0);
    let expected: [(&str, &str, &[u8]); 11] = [
        ("A.txt", "A.txt", b"first"),
        ("a.txt", if case { "a.txt" } else { "a (2).txt" }, b"second"),
        ("a (1).txt", "a (1).txt", b"original numbered name"),
        ("Docs/first.txt", "Docs/first.txt", b"first branch"),
        (
            "docs/second.txt",
            if case {
                "docs/second.txt"
            } else {
                "docs (1)/second.txt"
            },
            b"second branch",
        ),
        ("café.txt", "café.txt", b"composed"),
        ("Ä.txt", "Ä.txt", b"uppercase accented"),
        (
            "ä.txt",
            if accented_case {
                "ä.txt"
            } else {
                "ä (1).txt"
            },
            b"lowercase accented",
        ),
        (
            "cafe\u{301}.txt",
            if unicode {
                "cafe\u{301}.txt"
            } else {
                "cafe\u{301} (1).txt"
            },
            b"decomposed",
        ),
        ("Résumé/a.txt", "Résumé/a.txt", b"composed parent"),
        (
            "Re\u{301}sume\u{301}/b.txt",
            if unicode {
                "Re\u{301}sume\u{301}/b.txt"
            } else {
                "Re\u{301}sume\u{301} (1)/b.txt"
            },
            b"decomposed parent",
        ),
    ];
    for (original, output, contents) in expected {
        assert_eq!(
            std::fs::read(target.0.join(output)).unwrap(),
            contents,
            "{output}"
        );
        assert_eq!(
            done.details
                .iter()
                .any(|d| d.original_name.as_deref() == Some(original) && d.name == output),
            original != output,
            "{original}"
        );
    }
    if !case {
        assert!(!target.0.join("Docs/second.txt").exists());
        assert!(
            done.details
                .iter()
                .any(|d| d.original_name.as_deref() == Some("docs") && d.name == "docs (1)")
        );
    }
    if !unicode {
        assert!(!target.0.join("Résumé/b.txt").exists());
    }
}

#[test]
fn special_punctuation_preserves_or_renames_files_and_directory_trees() {
    let a = archive("special_names.zip");
    let target = Target::new();
    let existing = target.0.join("rating_ (1).txt");
    std::fs::write(&existing, b"keep existing").unwrap();
    let win32 = cfg!(windows);
    let mut options = target.options();
    // A forced replacement must not inherit permission to overwrite its fallback name.
    options.conflict = ConflictPolicy::Overwrite;
    let done = execute(&a, options, &Control::default(), |_| {});
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(done.committed_files, 5);
    assert_eq!(done.skipped_files, 0);
    assert_eq!(std::fs::read(existing).unwrap(), b"keep existing");
    let expected: [(&str, &str, &[u8]); 5] = [
        ("rating?.txt", "rating_ (2).txt", b"question"),
        ("rating*.txt", "rating_ (3).txt", b"asterisk"),
        ("rating_.txt", "rating_.txt", b"original underscore"),
        (
            "Album?|/photo<1>.txt",
            "Album__ (1)/photo_1_.txt",
            b"angle brackets",
        ),
        ("Album__/\"label\".txt", "Album__/_label_.txt", b"quotes"),
    ];
    for (original, replacement, bytes) in expected {
        let output = if win32 { replacement } else { original };
        assert_eq!(
            std::fs::read(target.0.join(output)).unwrap(),
            bytes,
            "{output}"
        );
        assert_eq!(
            done.details
                .iter()
                .any(|row| row.original_name.as_deref() == Some(original) && row.name == output),
            original != output,
            "{original}"
        );
    }
    let branch = if win32 { "Album__ (1)" } else { "Album?|" };
    assert_eq!(std::fs::read_dir(target.0.join(branch)).unwrap().count(), 1);
    assert_eq!(
        std::fs::read_dir(target.0.join("Album__")).unwrap().count(),
        1
    );
    assert_eq!(
        done.details
            .iter()
            .any(|row| row.original_name.as_deref() == Some("Album?|") && row.name == branch),
        win32,
    );
    let cancel = Cancellation::default();
    assert!(a.export_manifest(&target.options(), &cancel).is_err());
    let names = a
        .resolve_members(&["Album?|".into()], &[], &cancel)
        .unwrap();
    assert_eq!(names.len(), 1);
    let target = Target::new();
    let mut opts = target.options();
    opts.all = false;
    opts.selected = names;
    let done = execute(&a, opts, &Control::default(), |_| {});
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    let path = if win32 {
        "Album__/photo_1_.txt"
    } else {
        "Album?|/photo<1>.txt"
    };
    assert_eq!(
        std::fs::read(target.0.join(path)).unwrap(),
        b"angle brackets"
    );
}

#[test]
fn unsafe_plans_create_nothing() {
    let target = Target::new();
    let a = archive("unsafe.zip");
    assert!(
        a.extraction_job(target.options(), &Cancellation::default())
            .is_err()
    );
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
}
#[test]
fn source_replacement_after_planning_invalidates_task_before_destination_creation() {
    let target = Target::new();
    let source = Target::new();
    let path = source.0.join("source.zip");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/extract/stored.zip"),
        &path,
    )
    .unwrap();
    let a = Archive::open(&path, &Cancellation::default()).unwrap();
    let job = a
        .extraction_job(target.options(), &Cancellation::default())
        .unwrap();
    std::fs::rename(&path, source.0.join("old.zip")).unwrap();
    std::fs::write(&path, b"changed").unwrap();
    let mut last = None;
    run(job, 1, &Control::default(), &mut |s| last = Some(s));
    assert_eq!(last.unwrap().error, "source_changed");
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
}
#[test]
fn commit_cancellation_has_explicit_linearization_and_never_claims_rollback() {
    let c = Control::default();
    assert!(c.cancel());
    assert_eq!(c.begin_commit(), Err("cancelled"));
    let c = Control::default();
    c.begin_commit().unwrap();
    assert!(c.cancel());
    assert_eq!(c.end_commit(true), Err("cancelled"));
    let c = Control::default();
    c.begin_commit().unwrap();
    c.end_commit(true).unwrap();
    assert!(!c.cancel());
    c.complete().unwrap();
    let target = Target::new();
    let a = archive("tree.zip");
    let c = Control::default();
    let done = execute(&a, target.options(), &c, |s| {
        if s.committed_files > 0 && !s.status.terminal() {
            c.cancel();
        }
    });
    assert_eq!(done.status, Status::Cancelled);
    assert_eq!(done.committed_files, 1);
    assert!(target.0.join("照片/说明.txt").is_file());
    assert!(!target.0.join("data.bin").exists());
}

#[test]
fn partial_disk_full_write_never_replaces_an_existing_destination() {
    let target = Target::new();
    let a = archive("deflate.zip");
    let path = target.0.join("说明.txt");
    std::fs::write(&path, b"old safe bytes").unwrap();
    let mut options = target.options();
    options.conflict = ConflictPolicy::Overwrite;
    execute::inject_disk_full(Some(2));
    let done = execute(&a, options, &Control::default(), |_| {});
    execute::inject_disk_full(None);
    assert_eq!(done.error, "disk_full");
    assert_eq!(done.written_bytes, 2);
    assert_eq!(done.committed_files, 0);
    assert_eq!(std::fs::read(path).unwrap(), b"old safe bytes");
    assert!(!done.cleanup_warning);
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 1);
}
#[test]
fn source_changed_at_commit_boundary_preserves_existing_destination() {
    let target = Target::new();
    let source = Target::new();
    let path = source.0.join("source.zip");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/extract/deflate.zip"),
        &path,
    )
    .unwrap();
    let a = Archive::open(&path, &Cancellation::default()).unwrap();
    std::fs::write(target.0.join("说明.txt"), b"old safe bytes").unwrap();
    let mut options = target.options();
    options.conflict = ConflictPolicy::Overwrite;
    let done = execute(&a, options, &Control::default(), |s| {
        if s.status == Status::Committing {
            std::fs::write(&path, b"changed before publication").unwrap();
        }
    });
    assert_eq!(done.error, "source_changed");
    assert_eq!(done.committed_files, 0);
    assert_eq!(
        std::fs::read(target.0.join("说明.txt")).unwrap(),
        b"old safe bytes"
    );
    assert!(!done.cleanup_warning);
}

#[test]
fn every_terminal_outcome_rejects_late_control_requests() {
    for (fixture, status) in [
        ("deflate.zip", Status::Succeeded),
        ("crc_bad.zip", Status::Failed),
    ] {
        let target = Target::new();
        let a = archive(fixture);
        let c = Control::default();
        let done = execute(&a, target.options(), &c, |_| {});
        assert_eq!(done.status, status);
        assert!(!c.cancel());
    }
    let target = Target::new();
    let a = archive("deflate.zip");
    let c = Control::default();
    c.cancel();
    let done = execute(&a, target.options(), &c, |_| {});
    assert_eq!(done.status, Status::Cancelled);
    assert!(!c.cancel());
}

#[test]
fn declared_links_have_explicit_diagnostic_before_output_admission() {
    let target = Target::new();
    let mut a = archive("stored.zip");
    let entries = Arc::make_mut(&mut a.rows);
    entries[0].link = true;
    entries[0].warning = true;
    assert!(matches!(
        a.extraction_job(target.options(), &Cancellation::default()),
        Err("unsupported_link")
    ));
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
}

#[cfg(unix)]
#[test]
fn app_zip_same_folder_is_a_normal_directory_containing_the_real_bundle() {
    let source = Target::new();
    let path = source.0.join("xdzip-0.1.2-macos-arm64.app.zip");
    std::fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/extract/unix_bundle.zip"),
        &path,
    )
    .unwrap();
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    assert_eq!(archive.output_folder(), "xdzip-0.1.2-macos-arm64");
    let target = Target::new();
    let mut options = target.options();
    options.create_folder = true;
    let done = execute(&archive, options, &Control::default(), |_| {});
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    let folder = target.0.join("xdzip-0.1.2-macos-arm64");
    assert_eq!(Path::new(&done.target), folder);
    assert!(!target.0.join("xdzip-0.1.2-macos-arm64.app").exists());
    let app = folder.join("Demo.app/Contents");
    assert!(app.join("MacOS/demo").is_file());
    assert_eq!(
        std::fs::read_link(app.join("Frameworks/F.framework/Versions/Current")).unwrap(),
        Path::new("A")
    );
}

#[cfg(unix)]
#[test]
fn unix_bundle_links_and_safe_executable_modes_are_restored_after_regular_files() {
    use std::os::unix::fs::PermissionsExt;
    let target = Target::new();
    let a = archive("unix_bundle.zip");
    let done = execute(&a, target.options(), &Control::default(), |_| {});
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(done.committed_files, 7);
    assert_eq!(done.decoded_bytes, done.declared_bytes);
    assert_eq!(done.written_bytes, done.committed_bytes);
    let app = target.0.join("Demo.app/Contents");
    assert_eq!(
        std::fs::metadata(app.join("MacOS/demo"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o755
    );
    assert_eq!(
        std::fs::metadata(app.join("Resources/info.txt"))
            .unwrap()
            .permissions()
            .mode()
            & 0o7777,
        0o644
    );
    let framework = app.join("Frameworks/F.framework");
    assert_eq!(
        std::fs::read_link(framework.join("Versions/Current")).unwrap(),
        Path::new("A")
    );
    assert_eq!(
        std::fs::read_link(framework.join("F")).unwrap(),
        Path::new("Versions/Current/F")
    );
    assert_eq!(
        std::fs::read(framework.join("F")).unwrap(),
        b"owned library\n"
    );
    assert_eq!(
        std::fs::read(framework.join("Resources/info")).unwrap(),
        b"resource"
    );
    assert!(!done.cleanup_warning);
}

#[test]
fn hostile_or_incomplete_link_graphs_are_rejected_before_any_output() {
    for name in [
        "link_absolute.zip",
        "link_parent.zip",
        "link_backslash.zip",
        "link_ambiguous_name.zip",
        "link_empty.zip",
        "link_nul.zip",
        "link_dangling.zip",
        "link_cycle.zip",
        "link_device.zip",
        "link_parent_write.zip",
        "link_cycle_pair.zip",
        "link_large.zip",
        "link_crc.zip",
    ] {
        let target = Target::new();
        let a = archive(name);
        let result = a.extraction_job(target.options(), &Cancellation::default());
        if cfg!(unix) && name == "link_ambiguous_name.zip" {
            assert!(matches!(result, Err("ambiguous_name")));
        } else {
            assert!(result.is_err(), "{name}");
        }
        assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0, "{name}");
    }
}

#[cfg(unix)]
#[test]
fn bundle_conflicts_and_partial_selections_do_not_publish_broken_links() {
    let a = archive("link_renames.zip");
    for policy in [
        ConflictPolicy::Ask,
        ConflictPolicy::Skip,
        ConflictPolicy::Overwrite,
    ] {
        let target = Target::new();
        let occupied = target.0.join("primary?.txt");
        std::fs::write(&occupied, b"preserve").unwrap();
        let mut options = target.options();
        options.conflict = policy;
        let done = execute(&a, options, &Control::default(), |_| {});
        assert_eq!(done.error, "output_conflict");
        assert_eq!(done.committed_files, 0);
        assert_eq!(std::fs::read(&occupied).unwrap(), b"preserve");
        assert!(!target.0.join("viewer").exists());
    }
    let target = Target::new();
    let mut options = target.options();
    options.all = false;
    options.selected = vec![a.rows.iter().find(|e| e.link).unwrap().id.get().to_string()];
    assert!(matches!(
        a.extraction_job(options, &Cancellation::default()),
        Err("unsupported_link")
    ));
}

#[cfg(unix)]
#[test]
fn native_links_reach_the_same_contents_before_and_after_renaming() {
    use std::fs;
    let a = archive("link_renames.zip");
    for conflicts in [false, true] {
        let target = Target::new();
        if conflicts {
            fs::write(target.0.join("primary?.txt"), b"original host file").unwrap();
            fs::write(target.0.join("entry"), b"original host entry").unwrap();
            fs::create_dir(target.0.join("assets")).unwrap();
            fs::write(target.0.join("assets/old.txt"), b"original host directory").unwrap();
        }
        let mut options = target.options();
        options.conflict = ConflictPolicy::Rename;
        let done = execute(&a, options, &Control::default(), |_| {});
        assert_eq!(done.status, Status::Succeeded, "{}", done.error);
        assert_eq!(done.committed_files, 9);
        assert_eq!(done.skipped_files, 0);
        let (entry, file, dir) = if conflicts {
            ("entry (1)", "primary? (1).txt", "assets (1)")
        } else {
            ("entry", "primary?.txt", "assets")
        };
        for link in [entry, "viewer", "via", "directory-view/pointer"] {
            assert_eq!(
                fs::read(target.0.join(link)).unwrap(),
                b"owned primary",
                "{link}"
            );
        }
        assert_eq!(
            fs::read(target.0.join("mount/local")).unwrap(),
            b"owned asset"
        );
        assert_eq!(
            fs::read_link(target.0.join(entry)).unwrap(),
            Path::new(file)
        );
        assert_eq!(
            fs::read_link(target.0.join("mount")).unwrap(),
            Path::new(dir)
        );
        assert_eq!(
            fs::read_link(target.0.join(dir).join("local")).unwrap(),
            Path::new("data.txt")
        );
        if conflicts {
            assert_eq!(
                fs::read(target.0.join("primary?.txt")).unwrap(),
                b"original host file"
            );
            assert_eq!(
                fs::read(target.0.join("entry")).unwrap(),
                b"original host entry"
            );
            assert_eq!(
                fs::read(target.0.join("assets/old.txt")).unwrap(),
                b"original host directory"
            );
            for (original, output) in [("primary?.txt", file), ("assets", dir), ("entry", entry)] {
                assert!(
                    done.details
                        .iter()
                        .any(|d| d.original_name.as_deref() == Some(original) && d.name == output)
                );
            }
        } else {
            assert_eq!(
                fs::read_link(target.0.join("viewer")).unwrap(),
                Path::new("entry")
            );
            assert_eq!(
                fs::read_link(target.0.join("via")).unwrap(),
                Path::new("mount/../primary?.txt")
            );
            assert!(done.details.iter().all(|d| d.original_name.is_none()));
        }
    }
}

#[cfg(not(unix))]
#[test]
fn unix_links_fail_explicitly_without_windows_privilege_or_file_substitution() {
    let target = Target::new();
    assert!(matches!(
        archive("unix_bundle.zip").extraction_job(target.options(), &Cancellation::default()),
        Err("unsupported_link")
    ));
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
}

#[test]
#[ignore = "Run with an explicitly generated XDZIP_LARGE_INDEX_FIXTURE"]
fn million_record_product_index_and_selected_safe_output() {
    use crate::query::QueryRequest;
    use std::io::{Seek, SeekFrom, Write};
    let path = std::path::PathBuf::from(
        std::env::var_os("XDZIP_LARGE_INDEX_FIXTURE").expect("owned fixture required"),
    );
    let control = Cancellation::default();
    let mut archive = Archive::open(&path, &control).unwrap();
    assert_eq!(archive.len(), 1_000_000);
    assert!(archive.rows.is_empty());
    assert!(archive.index_disk_bytes() > 0);
    let first = archive
        .query_page(
            QueryRequest {
                directory: "",
                text: "",
                sort: 2,
                offset: 0,
                expected: None,
                generation: 1,
            },
            &control,
        )
        .unwrap();
    let total = first.total;
    let key = first.key;
    let last = archive
        .query_page(
            QueryRequest {
                directory: "",
                text: "",
                sort: 2,
                offset: total as u64 - 1,
                expected: Some(key),
                generation: 1,
            },
            &control,
        )
        .unwrap();
    assert_eq!(last.rows.len(), 1);
    let target = Target::new();
    let mut options = target.options();
    options.all = false;
    options.selected = vec!["1000000".into()];
    let job = archive.extraction_job(options, &control).unwrap();
    assert!(job.subset);
    assert_eq!(job.rows.len(), 1);
    let mut result = None;
    run(job, 1, &Control::default(), &mut |snapshot| {
        result = Some(snapshot)
    });
    let result = result.unwrap();
    assert_eq!(result.status, Status::Succeeded);
    assert!(target.0.join("item-999999.txt").is_file());
    let mut options = target.options();
    options.all = true;
    assert!(matches!(
        archive.extraction_job(options, &control),
        Err("resource_limit")
    ));
    let copy = target.0.join("changed.zip");
    std::fs::copy(&path, &copy).unwrap();
    let mut changed = Archive::open(&copy, &control).unwrap();
    let mut file = std::fs::OpenOptions::new().write(true).open(&copy).unwrap();
    file.seek(SeekFrom::End(0)).unwrap();
    file.write_all(&[0]).unwrap();
    assert!(
        changed
            .query_page(
                QueryRequest {
                    directory: "",
                    text: "",
                    sort: 0,
                    offset: 0,
                    expected: None,
                    generation: 1
                },
                &control
            )
            .is_err()
    );
}

#[test]
fn shell_smart_layout_and_exclusive_wrappers_preserve_contents() {
    let a = archive("stored.zip");
    let target = Target::new();
    let mut options = target.options();
    options.shell_layout = 1;
    let first = execute(&a, options.clone(), &Control::default(), |_| {});
    assert_eq!(first.status, Status::Succeeded, "{}", first.error);
    assert_ne!(Path::new(&first.target), target.0);
    let second = execute(&a, options, &Control::default(), |_| {});
    assert_eq!(second.status, Status::Succeeded, "{}", second.error);
    assert_ne!(first.target, second.target);
    assert!(second.target.ends_with(" (2)"));
    // The complete bundle snapshot has a single implicit root directory.
    let bundle = archive("unix_bundle.zip");
    let output = Target::new();
    let mut smart = output.options();
    smart.shell_layout = 1;
    #[cfg(unix)]
    {
        let done = execute(&bundle, smart, &Control::default(), |_| {});
        assert_eq!(done.status, Status::Succeeded, "{}", done.error);
        assert_eq!(Path::new(&done.target), output.0);
        assert!(output.0.join("Demo.app/Contents/MacOS/demo").is_file());
    }
    #[cfg(windows)]
    {
        assert!(matches!(
            bundle.extraction_job(smart, &Cancellation::default()),
            Err("unsupported_link")
        ));
        assert_eq!(std::fs::read_dir(&output.0).unwrap().count(), 0);
    }
}

#[test]
fn shell_layout_rejects_unknown_modes_and_partial_selection() {
    let a = archive("stored.zip");
    let target = Target::new();
    let mut options = target.options();
    options.shell_layout = 3;
    assert!(matches!(
        a.extraction_job(options.clone(), &Cancellation::default()),
        Err("invalid_options")
    ));
    options.shell_layout = 1;
    options.all = false;
    options.selected = vec!["1".into()];
    assert!(matches!(
        a.extraction_job(options, &Cancellation::default()),
        Err("invalid_options")
    ));
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
}
