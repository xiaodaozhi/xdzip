// crates/xdzip-runtime/src/updating/tests.rs

use super::*;
use std::{
    fs,
    sync::atomic::{AtomicU64, Ordering},
};
use xdzip_codec_api::Format;
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
            solid: false,
            encrypt_headers: false,
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

fn row_id(archive: &Archive, name: &str) -> String {
    archive
        .rows
        .iter()
        .find(|r| r.raw_name == name.as_bytes())
        .unwrap()
        .id
        .get()
        .to_string()
}
fn move_options(
    archive: &Archive,
    selected: &[&str],
    target: &str,
    conflict: ConflictPolicy,
) -> MoveOptions {
    MoveOptions {
        selected: selected.iter().map(|name| row_id(archive, name)).collect(),
        directory: row_id(archive, "资料/"),
        target: if target.is_empty() {
            String::new()
        } else {
            row_id(archive, target)
        },
        conflict,
    }
}
#[test]
fn three_formats_move_mixed_files_folders_empty_directories_and_move_up() {
    for extension in ["zip", "tar", "tar.gz"] {
        let f = Fixture::new();
        fs::create_dir_all(f.0.join("inputs/资料/目标")).unwrap();
        fs::create_dir_all(f.0.join("inputs/资料/嵌套/子目录")).unwrap();
        fs::write(f.0.join("inputs/资料/嵌套/子目录/a.txt"), b"nested data").unwrap();
        let path = f.archive(extension);
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        let options = move_options(
            &archive,
            &["资料/原文件.txt", "资料/空目录/", "资料/嵌套/"],
            "资料/目标/",
            ConflictPolicy::Ask,
        );
        let job = archive.move_job(options, &Cancellation::default()).unwrap();
        drop(archive);
        let result = execute(job, &Control::default(), |_| {});
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        assert_eq!(result.kind, TaskKind::Move);
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        for name in [
            "资料/目标/原文件.txt",
            "资料/目标/空目录/",
            "资料/目标/嵌套/子目录/a.txt",
        ] {
            assert!(
                archive.rows.iter().any(|r| r.raw_name == name.as_bytes()),
                "{name}"
            );
        }
        assert!(
            !archive
                .rows
                .iter()
                .any(|r| r.raw_name == "资料/原文件.txt".as_bytes()
                    || r.raw_name.starts_with("资料/嵌套/".as_bytes()))
        );
        if let Some(output) = std::env::var_os("XDZIP_MOVE_INTEROP_OUTPUT") {
            fs::create_dir_all(&output).unwrap();
            fs::copy(
                &path,
                PathBuf::from(output).join(format!("moved.{extension}")),
            )
            .unwrap();
        }
        let options = move_options(&archive, &["资料/目标/"], "", ConflictPolicy::Ask);
        let job = archive.move_job(options, &Cancellation::default()).unwrap();
        drop(archive);
        assert_eq!(
            execute(job, &Control::default(), |_| {}).status,
            Status::Succeeded
        );
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        assert!(
            archive
                .rows
                .iter()
                .any(|r| r.raw_name == "目标/原文件.txt".as_bytes())
        );
        assert!(
            !archive
                .rows
                .iter()
                .any(|r| r.raw_name.starts_with("资料/目标/".as_bytes()))
        );
    }
}
#[test]
fn moves_merge_directories_and_keep_skipped_sources() {
    for extension in ["zip", "tar", "tar.gz"] {
        for policy in [
            ConflictPolicy::Skip,
            ConflictPolicy::Rename,
            ConflictPolicy::Overwrite,
        ] {
            let f = Fixture::new();
            fs::create_dir_all(f.0.join("inputs/资料/源")).unwrap();
            fs::create_dir_all(f.0.join("inputs/资料/目标/源")).unwrap();
            fs::write(f.0.join("inputs/资料/源/a.txt"), b"incoming").unwrap();
            fs::write(f.0.join("inputs/资料/源/b.txt"), b"other").unwrap();
            fs::write(f.0.join("inputs/资料/目标/源/a.txt"), b"existing").unwrap();
            let path = f.archive(extension);
            let archive = Archive::open(&path, &Cancellation::default()).unwrap();
            let job = archive
                .move_job(
                    move_options(&archive, &["资料/源/"], "资料/目标/", policy),
                    &Cancellation::default(),
                )
                .unwrap();
            drop(archive);
            let result = execute(job, &Control::default(), |_| {});
            assert_eq!(result.status, Status::Succeeded, "{}", result.error);
            let archive = Archive::open(&path, &Cancellation::default()).unwrap();
            let has = |name: &str| archive.rows.iter().any(|r| r.raw_name == name.as_bytes());
            assert!(has("资料/目标/源/b.txt"));
            assert!(!has("资料/源/b.txt"));
            assert_eq!(has("资料/源/a.txt"), policy == ConflictPolicy::Skip);
            assert_eq!(has("资料/源/"), policy == ConflictPolicy::Skip);
            assert_eq!(
                has("资料/目标/源/a (2).txt"),
                policy == ConflictPolicy::Rename
            );
            assert_eq!(
                archive
                    .rows
                    .iter()
                    .filter(|r| r.raw_name == "资料/目标/源/".as_bytes())
                    .count(),
                1
            );
        }
    }
}
#[test]
fn move_rejects_selected_descendant_original_location_and_stale_ids() {
    let f = Fixture::new();
    let path = f.archive("zip");
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    let mut options = MoveOptions {
        selected: vec![row_id(&archive, "资料/")],
        directory: String::new(),
        target: row_id(&archive, "资料/空目录/"),
        conflict: ConflictPolicy::Ask,
    };
    assert_eq!(
        archive
            .move_job(options.clone(), &Cancellation::default())
            .err(),
        Some("move_selected_directory")
    );
    options.target = row_id(&archive, "资料/");
    assert_eq!(
        archive
            .move_job(options.clone(), &Cancellation::default())
            .err(),
        Some("move_selected_directory")
    );
    options.target.clear();
    assert_eq!(
        archive.move_job(options, &Cancellation::default()).err(),
        Some("move_same_directory")
    );
    let mut options = move_options(
        &archive,
        &["资料/原文件.txt"],
        "资料/空目录/",
        ConflictPolicy::Ask,
    );
    options.selected = vec!["99999999".into()];
    assert_eq!(
        archive.move_job(options, &Cancellation::default()).err(),
        Some("invalid_selection")
    );
}
#[test]
fn move_cancel_before_commit_preserves_original_bytes() {
    let f = Fixture::new();
    let path = f.archive("zip");
    let before = fs::read(&path).unwrap();
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    let job = archive
        .move_job(
            move_options(
                &archive,
                &["资料/原文件.txt"],
                "资料/空目录/",
                ConflictPolicy::Ask,
            ),
            &Cancellation::default(),
        )
        .unwrap();
    drop(archive);
    let control = Control::default();
    let result = execute(job, &control, |s| {
        if s.status == Status::Verifying {
            control.cancel();
        }
    });
    assert_eq!(result.status, Status::Cancelled);
    assert_eq!(fs::read(path).unwrap(), before);
}

struct EmptyFiles(Vec<EncodeEntry>);
impl EncodeInput for EmptyFiles {
    fn entries(&self) -> &[EncodeEntry] {
        &self.0
    }
    fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(std::io::empty()))
    }
}

#[test]
fn move_at_namespace_limit_counts_final_names_without_stale_sources() {
    let f = Fixture::new();
    let path = f.0.join("limit.zip");
    let mut entries = vec![EncodeEntry {
        name: "target".into(),
        directory: true,
        size: 0,
        modified_unix: 0,
    }];
    entries.extend((0..99_999).map(|n| EncodeEntry {
        name: format!("f{n:05}"),
        directory: false,
        size: 0,
        modified_unix: 0,
    }));
    let mut bytes = Vec::new();
    xdzip_codec_zip::ZipCodec
        .encode(
            &EmptyFiles(entries),
            &mut bytes,
            EncodeSettings {
                solid: false,
                encrypt_headers: false,
                method: "store",
                level: 6,
                password: None,
            },
            &Cancellation::default(),
            &mut |_| {},
        )
        .unwrap();
    fs::write(&path, bytes).unwrap();
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    let options = MoveOptions {
        selected: vec![row_id(&archive, "f00000")],
        directory: String::new(),
        target: row_id(&archive, "target/"),
        conflict: ConflictPolicy::Ask,
    };
    let job = archive.move_job(options, &Cancellation::default()).unwrap();
    drop(archive);
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    assert_eq!(archive.rows.len(), 100_000);
    assert!(archive.rows.iter().any(|r| r.raw_name == b"target/f00000"));
}

#[test]
fn moving_last_child_keeps_an_implicit_source_directory_empty() {
    let f = Fixture::new();
    let path = f.0.join("implicit.zip");
    let entries = EmptyFiles(vec![
        EncodeEntry {
            name: "implicit/file.txt".into(),
            directory: false,
            size: 0,
            modified_unix: 0,
        },
        EncodeEntry {
            name: "target".into(),
            directory: true,
            size: 0,
            modified_unix: 0,
        },
    ]);
    let mut bytes = Vec::new();
    xdzip_codec_zip::ZipCodec
        .encode(
            &entries,
            &mut bytes,
            EncodeSettings {
                solid: false,
                encrypt_headers: false,
                method: "store",
                level: 6,
                password: None,
            },
            &Cancellation::default(),
            &mut |_| {},
        )
        .unwrap();
    fs::write(&path, bytes).unwrap();
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    let directory = archive
        .directories
        .as_ref()
        .unwrap()
        .iter()
        .find(|d| d.path == "implicit/")
        .unwrap();
    let options = MoveOptions {
        selected: vec![row_id(&archive, "implicit/file.txt")],
        directory: format!("d{}", directory.id),
        target: row_id(&archive, "target/"),
        conflict: ConflictPolicy::Ask,
    };
    let job = archive.move_job(options, &Cancellation::default()).unwrap();
    drop(archive);
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    assert!(
        archive
            .rows
            .iter()
            .any(|r| r.raw_name == b"implicit/" && r.directory)
    );
    assert!(
        archive
            .rows
            .iter()
            .any(|r| r.raw_name == b"target/file.txt")
    );
    assert!(
        !archive
            .rows
            .iter()
            .any(|r| r.raw_name == b"implicit/file.txt")
    );
}

#[test]
fn skipped_child_preserves_explicit_source_directory_when_destination_is_implicit() {
    use xdzip_codec_api::Format;
    for format in [Format::Zip, Format::Tar, Format::TarGzip] {
        let f = Fixture::new();
        let path = f.0.join("implicit-merge.archive");
        let entries = EmptyFiles(
            [
                ("source", true),
                ("source/a.txt", false),
                ("source/b.txt", false),
                ("target/source/a.txt", false),
            ]
            .into_iter()
            .map(|(name, directory)| EncodeEntry {
                name: name.into(),
                directory,
                size: 0,
                modified_unix: 1_700_000_000,
            })
            .collect(),
        );
        let mut bytes = Vec::new();
        codec(format)
            .encode(
                &entries,
                &mut bytes,
                EncodeSettings {
                    solid: false,
                    encrypt_headers: false,
                    method: if format == Format::Tar {
                        "store"
                    } else {
                        "deflate"
                    },
                    level: 6,
                    password: None,
                },
                &Cancellation::default(),
                &mut |_| {},
            )
            .unwrap();
        fs::write(&path, bytes).unwrap();
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        let original = archive
            .rows
            .iter()
            .find(|r| r.raw_name == b"source/")
            .unwrap()
            .clone();
        let job = archive
            .move_job(
                MoveOptions {
                    selected: vec![row_id(&archive, "source/")],
                    directory: String::new(),
                    target: format!(
                        "d{}",
                        archive
                            .directories
                            .as_ref()
                            .unwrap()
                            .iter()
                            .find(|d| d.path == "target/")
                            .unwrap()
                            .id
                    ),
                    conflict: ConflictPolicy::Skip,
                },
                &Cancellation::default(),
            )
            .unwrap();
        drop(archive);
        let result = execute(job, &Control::default(), |_| {});
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        let retained = archive
            .rows
            .iter()
            .find(|r| r.raw_name == b"source/")
            .unwrap();
        assert_eq!(retained.modified_dos, original.modified_dos);
        assert!(archive.rows.iter().any(|r| r.raw_name == b"source/a.txt"));
        assert!(
            archive
                .rows
                .iter()
                .any(|r| r.raw_name == b"target/source/b.txt")
        );
        assert!(!archive.rows.iter().any(|r| r.raw_name == b"source/b.txt"));
    }
}

#[test]
fn three_formats_filter_added_metadata_and_preserve_existing_metadata() {
    for (extension, format) in [
        ("zip", Format::Zip),
        ("tar", Format::Tar),
        ("tar.gz", Format::TarGzip),
    ] {
        let f = Fixture::new();
        let path = f.0.join(format!("archive.{extension}"));
        // Existing records are deliberately encoded without the host's new-input filter.
        let originals = EmptyFiles(
            ["原文件.txt", ".DS_Store", "__MACOSX/._original"]
                .iter()
                .map(|name| EncodeEntry {
                    name: (*name).into(),
                    directory: false,
                    size: 0,
                    modified_unix: 0,
                })
                .collect(),
        );
        let mut bytes = Vec::new();
        codec(format)
            .encode(
                &originals,
                &mut bytes,
                EncodeSettings {
                    solid: false,
                    encrypt_headers: false,
                    method: if extension == "tar" {
                        "store"
                    } else {
                        "deflate"
                    },
                    level: 6,
                    password: None,
                },
                &Cancellation::default(),
                &mut |_| {},
            )
            .unwrap();
        fs::write(&path, bytes).unwrap();
        for directory in [
            "add/空目录",
            "add/__MACOSX",
            "add/仅元数据/子目录",
            "_MACOSX",
        ] {
            fs::create_dir_all(f.0.join(directory)).unwrap();
        }
        for name in [
            ".DS_Store",
            "_MACOSX/._root",
            "add/.DS_Store",
            "add/__MACOSX/._new",
            "add/仅元数据/子目录/.DS_Store",
        ] {
            fs::write(f.0.join(name), b"new metadata").unwrap();
        }
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        let job = archive
            .update_job(
                Options {
                    inputs: ["add", ".DS_Store", "_MACOSX"]
                        .iter()
                        .map(|p| f.0.join(p).to_string_lossy().into_owned())
                        .collect(),
                    directory: String::new(),
                    conflict: ConflictPolicy::Ask,
                },
                &Cancellation::default(),
            )
            .unwrap();
        drop(archive);
        let result = execute(job, &Control::default(), |_| {});
        assert_eq!(
            result.status,
            Status::Succeeded,
            "{extension}: {}",
            result.error
        );
        let updated = Archive::open(&path, &Cancellation::default()).unwrap();
        let names: Vec<_> = updated
            .rows
            .iter()
            .map(|r| r.display_name.as_str())
            .collect();
        assert_eq!(
            names,
            vec![
                "原文件.txt",
                ".DS_Store",
                "__MACOSX/._original",
                "add/",
                "add/新文件.txt",
                "add/空目录/"
            ]
        );
        assert_eq!(
            fs::read(f.0.join("add/__MACOSX/._new")).unwrap(),
            b"new metadata"
        );
        if let Some(output) = std::env::var_os("XDZIP_METADATA_INTEROP_OUTPUT") {
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
fn metadata_only_addition_fails_without_rewriting_any_archive() {
    for extension in ["zip", "tar", "tar.gz"] {
        let f = Fixture::new();
        let path = f.archive(extension);
        let original = fs::read(&path).unwrap();
        fs::create_dir_all(f.0.join("仅元数据/子目录/_MACOSX")).unwrap();
        fs::create_dir(f.0.join("__MACOSX")).unwrap();
        fs::write(f.0.join(".DS_Store"), b"metadata").unwrap();
        fs::write(f.0.join("__MACOSX/._root"), b"metadata").unwrap();
        fs::write(f.0.join("仅元数据/子目录/_MACOSX/._file"), b"metadata").unwrap();
        for inputs in [
            vec![".DS_Store"],
            vec!["__MACOSX"],
            vec!["仅元数据"],
            vec![".DS_Store", "__MACOSX", "仅元数据"],
        ] {
            let archive = Archive::open(&path, &Cancellation::default()).unwrap();
            let job = archive
                .update_job(
                    Options {
                        inputs: inputs
                            .iter()
                            .map(|p| f.0.join(p).to_string_lossy().into_owned())
                            .collect(),
                        directory: "1".into(),
                        conflict: ConflictPolicy::Overwrite,
                    },
                    &Cancellation::default(),
                )
                .unwrap();
            drop(archive);
            let result = execute(job, &Control::default(), |s| {
                assert!(!matches!(
                    s.status,
                    Status::Running | Status::Verifying | Status::Waiting
                ));
            });
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.error, "empty_input");
            assert_eq!(result.written_bytes, 0);
            assert_eq!(fs::read(&path).unwrap(), original);
            assert!(!result.cleanup_warning);
        }
    }
}
