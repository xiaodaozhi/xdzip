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
            method: match extension {
                "tar" => 0,
                "7z" => 32,
                _ => 8,
            },
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
        shell_layout: 0,
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
        .find(|r| r.name_bytes() == name.as_bytes())
        .unwrap_or_else(|| panic!("missing entry {name}"))
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

fn edit_options(
    archive: &Archive,
    directory: &str,
    names: &[&str],
    action: EntryEdit,
) -> EntryEditOptions {
    EntryEditOptions {
        selected: names.iter().map(|name| row_id(archive, name)).collect(),
        directory: if directory.is_empty() {
            String::new()
        } else {
            row_id(archive, directory)
        },
        action,
    }
}

fn apply_entry_edit(
    path: &std::path::Path,
    options: EntryEditOptions,
    archive: Archive,
) -> Archive {
    let cancel = Cancellation::default();
    let job = archive.edit_entries_job(options, &cancel).unwrap();
    drop(archive);
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    Archive::open(path, &cancel).unwrap()
}

fn exercise_entry_edits(extension: &str) {
    let fixture = Fixture::new();
    fs::create_dir_all(fixture.0.join("inputs/资料/rename me/child")).unwrap();
    fs::write(
        fixture.0.join("inputs/资料/rename me/child/data.bin"),
        b"a nested record",
    )
    .unwrap();
    let path = fixture.archive(extension);
    let path = if extension == "tar.gz" {
        let renamed = path.with_file_name("alias.gz");
        fs::rename(path, &renamed).unwrap();
        renamed
    } else {
        path
    };
    let archive = Archive::open(&path, &Cancellation::default()).unwrap();
    let format = archive.format;
    let options = edit_options(
        &archive,
        "资料/",
        &["资料/原文件.txt"],
        EntryEdit::Rename("new.txt".into()),
    );
    let archive = apply_entry_edit(&path, options, archive);
    let options = edit_options(
        &archive,
        "资料/",
        &["资料/rename me/"],
        EntryEdit::Rename("Rename Me".into()),
    );
    let archive = apply_entry_edit(&path, options, archive);
    assert_eq!(
        crate::native_backend_tests::known_contents(&archive),
        BTreeMap::from([
            (
                "资料/Rename Me/child/data.bin".to_owned(),
                b"a nested record".to_vec()
            ),
            ("资料/new.txt".to_owned(), b"original bytes".to_vec()),
        ])
    );
    let options = edit_options(
        &archive,
        "资料/",
        &["资料/new.txt", "资料/Rename Me/"],
        EntryEdit::Delete,
    );
    let archive = apply_entry_edit(&path, options, archive);
    assert!(archive.rows.iter().all(|row| row.directory));
    assert!(
        archive
            .rows
            .iter()
            .any(|row| row.name_bytes() == "资料/空目录/".as_bytes())
    );
    let options = edit_options(&archive, "", &["资料/"], EntryEdit::Delete);
    let archive = apply_entry_edit(&path, options, archive);
    assert_eq!(archive.format, format);
    assert!(archive.rows.is_empty());
}

#[test]
fn zip_and_tar_rename_and_delete_preserve_content_and_empty_archives() {
    for format in ["zip", "tar", "tar.gz"] {
        exercise_entry_edits(format);
    }
}

#[test]
#[ignore = "requires built isolated codec worker"]
fn seven_zip_rename_and_delete_preserve_content_and_empty_archive() {
    exercise_entry_edits("7z");
}

#[test]
fn entry_edits_reject_wrong_selection_invalid_names_and_conflicts() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("inputs/资料/existing.txt"), b"sibling").unwrap();
    let path = fixture.archive("zip");
    let original = fs::read(&path).unwrap();
    let cancel = Cancellation::default();
    let archive = Archive::open(&path, &cancel).unwrap();
    for (name, code) in [
        ("..", "entry_name_invalid"),
        ("blocked/here", "entry_name_invalid"),
        ("CON.txt", "entry_name_invalid"),
        ("原文件.txt", "entry_name_unchanged"),
    ] {
        let options = edit_options(
            &archive,
            "资料/",
            &["资料/原文件.txt"],
            EntryEdit::Rename(name.into()),
        );
        assert_eq!(archive.edit_entries_job(options, &cancel).err(), Some(code));
    }
    let options = edit_options(&archive, "", &["资料/原文件.txt"], EntryEdit::Delete);
    assert_eq!(
        archive.edit_entries_job(options, &cancel).err(),
        Some("invalid_selection")
    );
    let mut options = edit_options(
        &archive,
        "资料/",
        &["资料/原文件.txt", "资料/空目录/"],
        EntryEdit::Delete,
    );
    options.selected.push("99999".into());
    assert_eq!(
        archive.edit_entries_job(options, &cancel).err(),
        Some("invalid_selection")
    );
    for name in ["Existing.txt", "空目录"] {
        let options = edit_options(
            &archive,
            "资料/",
            &["资料/原文件.txt"],
            EntryEdit::Rename(name.into()),
        );
        let result = execute(
            archive.edit_entries_job(options, &cancel).unwrap(),
            &Control::default(),
            |_| {},
        );
        assert_eq!(
            (result.kind, result.status, result.error.as_str()),
            (TaskKind::Rename, Status::Failed, "entry_name_exists")
        );
    }
    assert_eq!(fs::read(&path).unwrap(), original);
}

#[test]
fn deleting_last_child_keeps_an_implicit_source_folder_and_rejects_cancelled_or_changed_sources() {
    let fixture = Fixture::new();
    let path = fixture.0.join("implicit.zip");
    let entries = EmptyFiles(vec![
        EncodeEntry {
            name: "implicit/last.txt".into(),
            directory: false,
            size: 0,
            modified_unix: 0,
        },
        EncodeEntry {
            name: "existing/other.txt".into(),
            directory: false,
            size: 0,
            modified_unix: 0,
        },
    ]);
    let mut bytes = Vec::new();
    let cancel = Cancellation::default();
    codec(Format::Zip)
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
            &cancel,
            &mut |_| {},
        )
        .unwrap();
    fs::write(&path, &bytes).unwrap();
    let archive = Archive::open(&path, &cancel).unwrap();
    let id = |name| {
        format!(
            "d{}",
            archive
                .directories
                .as_ref()
                .unwrap()
                .iter()
                .find(|dir| dir.path == name)
                .unwrap()
                .id
        )
    };
    let implicit = id("implicit/");
    let options = EntryEditOptions {
        selected: vec![implicit.clone()],
        directory: String::new(),
        action: EntryEdit::Rename("existing".into()),
    };
    assert_eq!(
        execute(
            archive.edit_entries_job(options, &cancel).unwrap(),
            &Control::default(),
            |_| {}
        )
        .error,
        "entry_name_exists"
    );
    let options = EntryEditOptions {
        selected: vec![implicit.clone()],
        directory: String::new(),
        action: EntryEdit::Rename("renamed".into()),
    };
    let archive = apply_entry_edit(&path, options, archive);
    let prefix = format!(
        "d{}",
        archive
            .directories
            .as_ref()
            .unwrap()
            .iter()
            .find(|dir| dir.path == "renamed/")
            .unwrap()
            .id
    );
    assert!(
        archive
            .rows
            .iter()
            .any(|row| row.name_bytes() == b"renamed/last.txt")
    );
    let options = EntryEditOptions {
        selected: vec![row_id(&archive, "renamed/last.txt")],
        directory: prefix,
        action: EntryEdit::Delete,
    };
    let original = fs::read(&path).unwrap();
    let control = Control::default();
    let result = execute(
        archive.edit_entries_job(options.clone(), &cancel).unwrap(),
        &control,
        |snapshot| {
            if snapshot.status == Status::Verifying {
                control.cancel();
            }
        },
    );
    assert_eq!(result.status, Status::Cancelled);
    assert_eq!(fs::read(&path).unwrap(), original);
    let archive = apply_entry_edit(&path, options, archive);
    assert!(
        archive
            .rows
            .iter()
            .any(|row| row.name_bytes() == b"renamed/" && row.directory)
    );
    assert!(
        !archive
            .rows
            .iter()
            .any(|row| row.name_bytes() == b"renamed/last.txt")
    );
    let options = edit_options(&archive, "", &["renamed/"], EntryEdit::Delete);
    let job = archive.edit_entries_job(options, &cancel).unwrap();
    let mut changed = fs::read(&path).unwrap();
    changed.push(0);
    fs::write(&path, &changed).unwrap();
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(
        (result.kind, result.status, result.error.as_str()),
        (TaskKind::Delete, Status::Failed, "source_changed")
    );
    assert_eq!(fs::read(&path).unwrap(), changed);
}

fn extended_archive(
    fixture: &Fixture,
    suffix: &str,
    filter: xdzip_codec_stream::controlled::Filter,
) -> PathBuf {
    use std::io::Write;
    use xdzip_codec_stream::{FrameWriter, controlled::Filter};
    let host = fixture.0.join("project");
    fs::create_dir_all(host.join("target")).unwrap();
    fs::create_dir_all(host.join("empty")).unwrap();
    fs::write(host.join("source.bin"), extended_payload()).unwrap();
    fs::write(fixture.0.join("add/extra.txt"), b"extra bytes").unwrap();
    let name = if filter == Filter::Identity {
        suffix
    } else {
        "unfiltered.tar"
    };
    let mut result = None;
    creation::run(
        creation::Job {
            options: creation::Options {
                inputs: vec![host.to_string_lossy().into_owned()],
                target: fixture.0.to_string_lossy().into_owned(),
                name: name.into(),
                method: if filter == Filter::Identity { 259 } else { 0 },
                level: 6,
                conflict: ConflictPolicy::Ask,
                volume_bytes: 0,
                solid: false,
                encrypt_headers: false,
                password: None,
            },
        },
        1,
        &Control::default(),
        &mut |snapshot| result = Some(snapshot),
    );
    let result = result.unwrap();
    assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
    if filter == Filter::Identity {
        return fixture.0.join(name);
    }
    let path = fixture.0.join(suffix);
    let mut writer = FrameWriter::new(fs::File::create(&path).unwrap(), filter, 6).unwrap();
    writer
        .write_all(&fs::read(fixture.0.join(name)).unwrap())
        .unwrap();
    writer.finish().unwrap();
    path
}

fn extended_payload() -> Vec<u8> {
    // Large enough to exercise multiple outer blocks, dictionary resets and
    // range state across calls, with long runs and high-entropy sections.
    let mut x = 1729u32;
    (0..180_003)
        .map(|i| {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            if i % 4096 < 800 { 0x61 } else { x as u8 }
        })
        .collect()
}

#[test]
fn extended_formats_rename_move_add_replace_and_delete_transactionally() {
    use xdzip_codec_stream::controlled::Filter;
    let cancel = Cancellation::default();
    for (name, format, filter) in [
        ("edit.txz", Format::TarXz, Filter::Xz),
        ("edit.tar.xz", Format::TarXz, Filter::Xz),
        ("edit.tzst", Format::TarZstd, Filter::Zstd),
        ("edit.tar.zst", Format::TarZstd, Filter::Zstd),
        ("edit.tar.lz4", Format::TarLz4, Filter::Lz4),
        ("edit.tar.bz2", Format::TarBzip2, Filter::Bzip2),
        ("edit.tar.lzma", Format::TarLzma, Filter::Lzma),
        ("edit.tar.lz", Format::TarLzip, Filter::Lzip),
        ("edit.tar.Z", Format::TarCompress, Filter::Compress),
        ("edit.tar.br", Format::TarBrotli, Filter::Brotli),
        ("edit.lzh", Format::Lha, Filter::Identity),
        ("edit.lha", Format::Lha, Filter::Identity),
    ] {
        let fixture = Fixture::new();
        let path = extended_archive(&fixture, name, filter);
        let archive = Archive::open(&path, &cancel).unwrap();
        assert_eq!(archive.format, format, "{name}");
        assert!(archive.can_update(), "{name}");
        let options = edit_options(
            &archive,
            "project/",
            &["project/source.bin"],
            EntryEdit::Rename("renamed.bin".into()),
        );
        let archive = apply_entry_edit(&path, options, archive);

        let options = MoveOptions {
            selected: vec![row_id(&archive, "project/renamed.bin")],
            directory: row_id(&archive, "project/"),
            target: row_id(&archive, "project/target/"),
            conflict: ConflictPolicy::Ask,
        };
        let job = archive.move_job(options, &cancel).unwrap();
        drop(archive);
        let result = execute(job, &Control::default(), |_| {});
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);

        let extra = fixture.0.join("add/extra.txt");
        for expected in [b"extra bytes".as_slice(), b"replacement bytes".as_slice()] {
            fs::write(&extra, expected).unwrap();
            let archive = Archive::open(&path, &cancel).unwrap();
            let job = archive
                .update_job(
                    Options {
                        inputs: vec![extra.to_string_lossy().into_owned()],
                        directory: row_id(&archive, "project/target/"),
                        conflict: ConflictPolicy::Overwrite,
                    },
                    &cancel,
                )
                .unwrap();
            drop(archive);
            let result = execute(job, &Control::default(), |_| {});
            assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
            let archive = Archive::open(&path, &cancel).unwrap();
            let contents = crate::native_backend_tests::known_contents(&archive);
            assert_eq!(contents["project/target/extra.txt"], expected);
            assert_eq!(contents["project/target/renamed.bin"], extended_payload());
            assert_eq!(contents.len(), 2);
        }
        if let Some(out) = std::env::var_os("XDZIP_EXTENDED_EDIT_OUTPUT") {
            let out = PathBuf::from(out);
            fs::create_dir_all(&out).unwrap();
            fs::copy(&path, out.join(name)).unwrap();
            fs::write(out.join("expected.bin"), extended_payload()).unwrap();
        }

        let archive = Archive::open(&path, &cancel).unwrap();
        let options = edit_options(
            &archive,
            "project/",
            &["project/target/", "project/empty/"],
            EntryEdit::Delete,
        );
        let original = fs::read(&path).unwrap();
        let stop = Control::default();
        let job = archive.edit_entries_job(options.clone(), &cancel).unwrap();
        let result = execute(job, &stop, |snapshot| {
            if snapshot.status == Status::Verifying {
                stop.cancel();
            }
        });
        assert_eq!(result.status, Status::Cancelled, "{name}");
        assert_eq!(fs::read(&path).unwrap(), original);
        let archive = apply_entry_edit(&path, options, archive);
        assert_eq!(archive.rows.len(), 1, "{name}");
        let options = edit_options(&archive, "", &["project/"], EntryEdit::Delete);
        let archive = apply_entry_edit(&path, options, archive);
        assert_eq!(archive.format, format, "{name}");
        assert!(archive.rows.is_empty(), "{name}");
    }
}

#[test]
fn bare_stream_suffix_rejects_empty_tar_before_commit() {
    use xdzip_codec_stream::controlled::Filter;
    let fixture = Fixture::new();
    let path = extended_archive(&fixture, "bare.xz", Filter::Xz);
    let cancel = Cancellation::default();
    let original = fs::read(&path).unwrap();
    let archive = Archive::open(&path, &cancel).unwrap();
    assert_eq!(archive.format, Format::TarXz);
    let options = edit_options(&archive, "", &["project/"], EntryEdit::Delete);
    let result = execute(
        archive.edit_entries_job(options, &cancel).unwrap(),
        &Control::default(),
        |_| {},
    );
    assert_eq!(result.error, "empty_tar_suffix");
    assert_eq!(fs::read(&path).unwrap(), original);
}
