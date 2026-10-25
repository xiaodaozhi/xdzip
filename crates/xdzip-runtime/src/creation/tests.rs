//! Real creation/publication regressions; all inputs are owned temporary fixtures.
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use xdzip_codec_api::{MetadataCodec, ScanLimits, payload::PayloadCodec};
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
                volume_bytes: 0,
                solid: false,
                encrypt_headers: false,
                password: None,
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
fn archive_contents(bytes: Vec<u8>, format: Format) -> std::collections::BTreeMap<String, Vec<u8>> {
    struct Bytes(Vec<u8>);
    impl DecodedSink for Bytes {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }
    }
    let cancel = Cancellation::default();
    let mut reader = std::io::Cursor::new(bytes);
    let rows = codec(format)
        .scan(&mut reader, &cancel, crate::formats::limits(format))
        .unwrap();
    let mut decoder = codec(format)
        .open_payload(&mut reader, &rows, &cancel)
        .unwrap();
    let mut contents = std::collections::BTreeMap::new();
    for row in rows {
        let mut bytes = Bytes(vec![]);
        decoder.decode(&row, &mut bytes, &cancel).unwrap();
        if !row.directory {
            assert!(contents.insert(row.display_name, bytes.0).is_none());
        }
    }
    contents
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
                        work: &Cell::new(0),
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
fn aes_creation_authenticates_all_files_and_rejects_wrong_password() {
    for method in [0, 8] {
        let f = Fixture::new();
        let mut job = f.job(method);
        job.options.password = Some(Password::new("中文 password 123".into()).unwrap());
        let result = execute(job, &Control::default(), |_| {});
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        assert!(!result.external_reader);
        assert_eq!(result.output_parts, 1);
        let names = [
            ("输入", true, 0),
            ("输入/empty.bin", false, 0),
            (
                "输入/文件.txt",
                false,
                std::fs::metadata(f.0.join("输入/文件.txt")).unwrap().len(),
            ),
            ("输入/空目录", true, 0),
        ];
        // InputTree determines ordering, so reuse the frozen entries instead of
        // presuming a filesystem enumeration or ZIP central-directory order.
        let tree = InputTree::scan(&[f.0.join("输入")], vec![], &|| false).unwrap();
        assert_eq!(tree.entries().len(), names.len());
        let entries: Vec<_> = tree
            .entries()
            .iter()
            .map(|e| EncodeEntry {
                name: e.name.clone(),
                directory: e.directory,
                size: e.size,
                modified_unix: 0,
            })
            .collect();
        let cancel = Cancellation::default();
        assert!(
            ZipCodec
                .verify_encrypted(
                    std::fs::File::open(f.output()).unwrap(),
                    &entries,
                    "中文 password 123",
                    &cancel
                )
                .is_ok()
        );
        assert!(
            ZipCodec
                .verify_encrypted(
                    std::fs::File::open(f.output()).unwrap(),
                    &entries,
                    "wrong",
                    &cancel
                )
                .is_err()
        );
        assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
    }
}
#[test]
fn split_parts_reassemble_to_a_verified_zip_and_never_replace_old_parts() {
    let f = Fixture::new();
    let mut job = f.job(0);
    job.options.volume_bytes = 1024;
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert!(result.output_parts > 1);
    let mut joined = Vec::new();
    for index in 1..=result.output_parts {
        let bytes = std::fs::read(f.0.join(format!("output/archive.zip.{index:03}"))).unwrap();
        assert!(bytes.len() <= 1024);
        if index < result.output_parts {
            assert_eq!(bytes.len(), 1024);
        }
        joined.extend(bytes);
    }
    let rows = ZipCodec
        .scan(
            &mut std::io::Cursor::new(joined),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap();
    assert_eq!(rows.len(), 4);
    assert!(!f.output().exists());
    let first = std::fs::read(f.0.join("output/archive.zip.001")).unwrap();
    let mut job = f.job(0);
    job.options.volume_bytes = 1024;
    job.options.conflict = ConflictPolicy::Rename;
    let again = execute(job, &Control::default(), |_| {});
    assert_eq!(again.status, Status::Succeeded, "{}", again.error);
    assert!(f.0.join("output/archive (2).zip.001").is_file());
    assert_eq!(
        std::fs::read(f.0.join("output/archive.zip.001")).unwrap(),
        first
    );
}
#[test]
fn tiny_volume_budget_and_invalid_secrets_fail_without_publication() {
    for value in ["", "a\nb", "\0", &"x".repeat(129)] {
        assert!(Password::new(value.into()).is_err());
    }
    let secret = Password::new("do not log me".into()).unwrap();
    assert_eq!(format!("{secret:?}"), "[REDACTED]");
    let f = Fixture::new();
    let mut job = f.job(0);
    job.options.volume_bytes = 1;
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.error, "resource_limit");
    assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 0);
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
fn output_inside_input_keeps_all_files_and_excludes_only_the_private_workspace() {
    let f = Fixture::new();
    let directory = f.0.join("输入");
    let output = directory.join("archive.zip");
    std::fs::write(&output, b"previous archive").unwrap();
    std::fs::write(directory.join(".xdzip-keep.txt"), b"ordinary input").unwrap();
    let mut job = f.job(8);
    job.options.target = directory.to_string_lossy().into_owned();
    let control = Control::default();
    let mut prompts = 0;
    let result = execute(job, &control, |s| {
        if s.status == Status::Waiting {
            prompts += 1;
            let prompt = s.prompt.as_ref().unwrap();
            assert!(!prompt.overwrite_allowed);
            assert!(control.decide(prompt.id, ConflictPolicy::Rename, false));
        }
    });
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(prompts, 1);
    assert_eq!(result.committed_files, 4);
    assert_eq!(result.created_directories, 2);
    assert_eq!(std::fs::read(&output).unwrap(), b"previous archive");
    let contents = archive_contents(std::fs::read(&result.target).unwrap(), Format::Zip);
    assert_eq!(contents.len(), 4);
    assert_eq!(contents["输入/archive.zip"], b"previous archive");
    assert_eq!(contents["输入/.xdzip-keep.txt"], b"ordinary input");
    assert_eq!(contents["输入/empty.bin"], b"");
    assert_eq!(
        contents["输入/文件.txt"],
        std::fs::read(directory.join("文件.txt")).unwrap()
    );
    assert!(!contents.keys().any(|name| name.contains(".xdzip-work-")));
}

#[test]
fn explicitly_selected_output_reaches_rename_and_keeps_its_original_bytes_in_each_format() {
    for (extension, method, format) in [
        (".zip", 8, Format::Zip),
        (".tar", 0, Format::Tar),
        (".tar.gz", 8, Format::TarGzip),
    ] {
        for policy in [ConflictPolicy::Ask, ConflictPolicy::Rename] {
            let f = Fixture::new();
            let name = format!("archive{extension}");
            let output = f.0.join("output").join(&name);
            std::fs::write(&output, b"previous archive").unwrap();
            let mut job = f.job(method);
            job.options.name = name.clone();
            job.options.inputs = [
                f.0.join("输入/文件.txt"),
                f.0.join("输入/empty.bin"),
                output.clone(),
            ]
            .into_iter()
            .map(|p| p.to_string_lossy().into_owned())
            .collect();
            job.options.conflict = policy;
            let control = Control::default();
            let mut prompts = 0;
            let result = execute(job, &control, |s| {
                if s.status == Status::Waiting {
                    prompts += 1;
                    let prompt = s.prompt.as_ref().unwrap();
                    assert!(!prompt.overwrite_allowed);
                    assert!(control.decide(prompt.id, ConflictPolicy::Rename, false));
                }
            });
            assert_eq!(result.status, Status::Succeeded, "{}", result.error);
            assert_eq!(prompts, u8::from(policy == ConflictPolicy::Ask));
            assert_eq!(result.committed_files, 3);
            assert_eq!(std::fs::read(&output).unwrap(), b"previous archive");
            assert_eq!(
                Path::new(&result.target),
                f.0.join("output").join(format!("archive (2){extension}"))
            );
            let contents = archive_contents(std::fs::read(&result.target).unwrap(), format);
            assert_eq!(contents.len(), 3);
            assert_eq!(contents[&name], b"previous archive");
            assert_eq!(
                contents["文件.txt"],
                std::fs::read(f.0.join("输入/文件.txt")).unwrap()
            );
            assert_eq!(contents["empty.bin"], b"");
        }
    }
}

#[test]
fn overwriting_an_input_is_rejected_even_when_a_prompt_reply_requests_it() {
    for (extension, method) in [(".zip", 8), (".tar", 0), (".tar.gz", 8)] {
        for ask in [false, true] {
            let f = Fixture::new();
            let output = f.0.join("output").join(format!("archive{extension}"));
            std::fs::write(&output, b"previous archive").unwrap();
            let mut job = f.job(method);
            job.options.name = format!("archive{extension}");
            job.options
                .inputs
                .push(output.to_string_lossy().into_owned());
            job.options.conflict = if ask {
                ConflictPolicy::Ask
            } else {
                ConflictPolicy::Overwrite
            };
            let control = Control::default();
            let mut prompts = 0;
            let result = execute(job, &control, |s| {
                if s.status == Status::Waiting {
                    prompts += 1;
                    let prompt = s.prompt.as_ref().unwrap();
                    assert!(!prompt.overwrite_allowed);
                    assert!(control.decide(prompt.id, ConflictPolicy::Overwrite, false));
                }
            });
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.error, "protected_source");
            assert_eq!(prompts, u8::from(ask));
            assert!(result.written_bytes > 0);
            assert_eq!(result.committed_files, 0);
            assert_eq!(std::fs::read(output).unwrap(), b"previous archive");
            assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
        }
    }
}

#[test]
fn output_case_and_unicode_aliases_reach_the_normal_conflict_guard() {
    for (input_name, output_name) in [
        ("ExactCase.zip", "exactcase.zip"),
        ("caf\u{e9}.zip", "cafe\u{301}.zip"),
    ] {
        for select_directory in [false, true] {
            let f = Fixture::new();
            let directory = f.0.join("输入");
            let source = directory.join(input_name);
            let output = directory.join(output_name);
            std::fs::write(&source, b"previous archive").unwrap();
            let same_entry = output.exists();
            let mut job = f.job(8);
            job.options.target = directory.to_string_lossy().into_owned();
            job.options.name = output_name.into();
            if !select_directory {
                job.options.inputs = [directory.join("文件.txt"), source.clone()]
                    .into_iter()
                    .map(|p| p.to_string_lossy().into_owned())
                    .collect();
            }
            let control = Control::default();
            let mut prompts = 0;
            let result = execute(job, &control, |s| {
                if s.status == Status::Waiting {
                    prompts += 1;
                    let prompt = s.prompt.as_ref().unwrap();
                    assert!(!prompt.overwrite_allowed);
                    assert!(control.decide(prompt.id, ConflictPolicy::Rename, false));
                }
            });
            assert_eq!(result.status, Status::Succeeded, "{}", result.error);
            assert_eq!(prompts, u8::from(same_entry));
            assert_eq!(result.committed_files, if select_directory { 3 } else { 2 });
            assert_eq!(std::fs::read(source).unwrap(), b"previous archive");
            let contents = archive_contents(std::fs::read(&result.target).unwrap(), Format::Zip);
            let prefix = if select_directory { "输入/" } else { "" };
            assert_eq!(
                contents[&format!("{prefix}{input_name}")],
                b"previous archive"
            );
            assert_eq!(
                contents[&format!("{prefix}文件.txt")],
                std::fs::read(directory.join("文件.txt")).unwrap()
            );
            if select_directory {
                assert_eq!(contents["输入/empty.bin"], b"");
            }
        }
    }
}

#[test]
fn distinct_case_sensitive_targets_are_not_mistaken_for_selected_input_aliases() {
    let f = Fixture::new();
    let source = f.0.join("output/ExactCase.zip");
    let output = f.0.join("output/exactcase.zip");
    std::fs::write(&source, b"selected source bytes").unwrap();
    let same_entry = output.exists();
    if !same_entry {
        std::fs::write(&output, b"unselected target bytes").unwrap();
    }
    let mut job = f.job(8);
    job.options.name = "exactcase.zip".into();
    job.options.inputs = vec![source.to_string_lossy().into_owned()];
    let control = Control::default();
    let mut prompts = 0;
    let result = execute(job, &control, |s| {
        if s.status == Status::Waiting {
            prompts += 1;
            let prompt = s.prompt.as_ref().unwrap();
            assert_eq!(prompt.overwrite_allowed, !same_entry);
            let decision = if same_entry {
                ConflictPolicy::Rename
            } else {
                ConflictPolicy::Overwrite
            };
            assert!(control.decide(prompt.id, decision, false));
        }
    });
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(prompts, 1);
    assert_eq!(std::fs::read(source).unwrap(), b"selected source bytes");
    let contents = archive_contents(std::fs::read(result.target).unwrap(), Format::Zip);
    assert_eq!(contents.len(), 1);
    assert_eq!(contents["ExactCase.zip"], b"selected source bytes");
}

#[test]
fn directory_selection_aliases_keep_old_output_and_exclude_the_private_workspace() {
    let f = Fixture::new();
    let directory = f.0.join("CaseParent");
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(directory.join("payload.txt"), b"ordinary input").unwrap();
    std::fs::write(directory.join("ExactCase.zip"), b"previous archive").unwrap();
    let alias = f.0.join("caseparent");
    let selected = if alias.is_dir() {
        alias
    } else {
        directory.clone()
    };
    let prefix = selected.file_name().unwrap().to_str().unwrap().to_owned();
    let mut job = f.job(8);
    job.options.inputs = vec![selected.to_string_lossy().into_owned()];
    job.options.target = directory.to_string_lossy().into_owned();
    job.options.name = "ExactCase.zip".into();
    job.options.conflict = ConflictPolicy::Rename;
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(result.committed_files, 2);
    assert_eq!(
        std::fs::read(directory.join("ExactCase.zip")).unwrap(),
        b"previous archive"
    );
    let contents = archive_contents(std::fs::read(result.target).unwrap(), Format::Zip);
    assert_eq!(contents.len(), 2);
    assert_eq!(
        contents[&format!("{prefix}/payload.txt")],
        b"ordinary input"
    );
    assert_eq!(
        contents[&format!("{prefix}/ExactCase.zip")],
        b"previous archive"
    );
}

#[test]
fn selected_directory_and_explicit_output_are_deduplicated_without_excluding_the_file() {
    let f = Fixture::new();
    let directory = f.0.join("输入");
    let output = directory.join("archive.zip");
    std::fs::write(&output, b"previous archive").unwrap();
    let mut job = f.job(8);
    job.options.target = directory.to_string_lossy().into_owned();
    job.options
        .inputs
        .push(output.to_string_lossy().into_owned());
    job.options.conflict = ConflictPolicy::Rename;
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(result.committed_files, 3);
    assert_eq!(std::fs::read(output).unwrap(), b"previous archive");
    let contents = archive_contents(std::fs::read(result.target).unwrap(), Format::Zip);
    assert_eq!(contents.len(), 3);
    assert_eq!(contents["输入/archive.zip"], b"previous archive");
}

#[test]
fn selecting_only_the_old_target_is_valid_input_and_can_rename_for_each_format() {
    for (extension, method, format) in [
        (".zip", 8, Format::Zip),
        (".tar", 0, Format::Tar),
        (".tar.gz", 8, Format::TarGzip),
    ] {
        let f = Fixture::new();
        let name = format!("archive{extension}");
        let output = f.0.join("output").join(&name);
        std::fs::write(&output, b"previous archive").unwrap();
        let mut job = f.job(method);
        job.options.name = name.clone();
        job.options.inputs = vec![output.to_string_lossy().into_owned()];
        let control = Control::default();
        let mut prompts = 0;
        let result = execute(job, &control, |s| {
            if s.status == Status::Waiting {
                prompts += 1;
                let prompt = s.prompt.as_ref().unwrap();
                assert!(!prompt.overwrite_allowed);
                assert!(control.decide(prompt.id, ConflictPolicy::Rename, false));
            }
        });
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        assert_eq!(prompts, 1);
        assert_eq!(result.committed_files, 1);
        assert_eq!(std::fs::read(output).unwrap(), b"previous archive");
        let contents = archive_contents(std::fs::read(result.target).unwrap(), format);
        assert_eq!(contents.len(), 1);
        assert_eq!(contents[&name], b"previous archive");
    }
}

#[test]
fn explicitly_selecting_an_identity_bound_workspace_is_still_rejected() {
    let f = Fixture::new();
    let root = Grant::for_creation(&f.0.join("output")).unwrap();
    let workspace = root.workspace(1).unwrap();
    let (path, identity) = workspace.input_exclusion();
    let result = InputTree::scan(
        std::slice::from_ref(&path),
        vec![(path.clone(), identity)],
        &|| false,
    );
    assert!(matches!(result, Err(xdzip_io::InputError::Unsafe)));
    let alias = path.with_file_name(path.file_name().unwrap().to_string_lossy().to_uppercase());
    if alias.is_dir() {
        assert!(matches!(
            InputTree::scan(&[alias], vec![(path, identity)], &|| false),
            Err(xdzip_io::InputError::Unsafe)
        ));
    }
    workspace.cleanup().unwrap();
    assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 0);
}

#[test]
fn declining_a_conflict_with_selected_output_keeps_every_input_and_cleans_staging() {
    let f = Fixture::new();
    std::fs::write(f.output(), b"previous archive").unwrap();
    let mut job = f.job(8);
    job.options
        .inputs
        .push(f.output().to_string_lossy().into_owned());
    let control = Control::default();
    let mut prompts = 0;
    let result = execute(job, &control, |s| {
        if s.status == Status::Waiting {
            prompts += 1;
            assert!(!s.prompt.as_ref().unwrap().overwrite_allowed);
            control.cancel();
        }
    });
    assert_eq!(prompts, 1);
    assert_eq!(result.status, Status::Cancelled);
    assert_eq!(result.files, 3);
    assert!(result.written_bytes > 0);
    assert_eq!(std::fs::read(f.output()).unwrap(), b"previous archive");
    assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
}

#[test]
fn changing_a_selected_target_while_waiting_prevents_publishing_stale_input() {
    let f = Fixture::new();
    std::fs::write(f.output(), b"previous archive").unwrap();
    let mut job = f.job(8);
    job.options.inputs = vec![f.output().to_string_lossy().into_owned()];
    let control = Control::default();
    let mut prompts = 0;
    let result = execute(job, &control, |s| {
        if s.status == Status::Waiting {
            prompts += 1;
            let prompt = s.prompt.as_ref().unwrap();
            assert!(!prompt.overwrite_allowed);
            std::fs::write(f.output(), b"new externally modified input bytes").unwrap();
            assert!(control.decide(prompt.id, ConflictPolicy::Rename, false));
        }
    });
    assert_eq!(prompts, 1);
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert_eq!(result.committed_files, 0);
    assert_eq!(
        std::fs::read(f.output()).unwrap(),
        b"new externally modified input bytes"
    );
    assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
}

#[test]
fn selected_numbered_outputs_and_neighbor_names_all_remain_archive_inputs() {
    let f = Fixture::new();
    let names = [
        "archive.zip.001",
        "archive.zip.1024",
        "archive.zip.1025",
        "archive.zip",
    ];
    let mut expected = std::collections::BTreeMap::new();
    let mut job = f.job(0);
    for name in names {
        let path = f.0.join("output").join(name);
        let bytes = format!("previous bytes of {name}").into_bytes();
        std::fs::write(&path, &bytes).unwrap();
        expected.insert(name.to_owned(), bytes);
        job.options.inputs.push(path.to_string_lossy().into_owned());
    }
    expected.insert(
        "输入/文件.txt".into(),
        std::fs::read(f.0.join("输入/文件.txt")).unwrap(),
    );
    expected.insert("输入/empty.bin".into(), vec![]);
    job.options.volume_bytes = 1024;
    let control = Control::default();
    let mut prompts = 0;
    let result = execute(job, &control, |s| {
        if s.status == Status::Waiting {
            prompts += 1;
            let prompt = s.prompt.as_ref().unwrap();
            assert!(!prompt.overwrite_allowed);
            assert!(control.decide(prompt.id, ConflictPolicy::Rename, false));
        }
    });
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(prompts, 1);
    assert_eq!(result.committed_files, 6);
    let mut joined = Vec::new();
    for index in 1..=result.output_parts {
        joined
            .extend(std::fs::read(f.0.join(format!("output/archive (2).zip.{index:03}"))).unwrap());
    }
    assert_eq!(archive_contents(joined, Format::Zip), expected);
    for name in names {
        assert_eq!(
            std::fs::read(f.0.join("output").join(name)).unwrap(),
            expected[name]
        );
    }
}

#[test]
fn selected_volume_conflicts_allow_cancel_and_still_reject_forced_overwrite() {
    for (policy, decision) in [
        (ConflictPolicy::Ask, ConflictPolicy::Skip),
        (ConflictPolicy::Ask, ConflictPolicy::Overwrite),
        (ConflictPolicy::Overwrite, ConflictPolicy::Overwrite),
    ] {
        let f = Fixture::new();
        let first = f.0.join("output/archive.zip.001");
        std::fs::write(&first, vec![5; 4096]).unwrap();
        let mut job = f.job(0);
        job.options.inputs = vec![first.to_string_lossy().into_owned()];
        job.options.volume_bytes = 1024;
        job.options.conflict = policy;
        let control = Control::default();
        let mut prompts = 0;
        let result = execute(job, &control, |s| {
            if s.status == Status::Waiting {
                prompts += 1;
                let prompt = s.prompt.as_ref().unwrap();
                assert!(!prompt.overwrite_allowed);
                assert!(control.decide(prompt.id, decision, false));
            }
        });
        assert_eq!(prompts, u8::from(policy == ConflictPolicy::Ask));
        if decision == ConflictPolicy::Skip {
            assert_eq!(result.status, Status::Cancelled);
        } else {
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.error, "split_overwrite_not_supported");
        }
        assert!(result.written_bytes > 0);
        assert_eq!(std::fs::read(first).unwrap(), vec![5; 4096]);
        assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 1);
    }
}

#[test]
fn preserving_target_inputs_does_not_relax_directory_or_hard_link_target_guards() {
    let f = Fixture::new();
    std::fs::create_dir(f.output()).unwrap();
    let mut job = f.job(8);
    job.options
        .inputs
        .push(f.output().to_string_lossy().into_owned());
    assert_eq!(
        execute(job, &Control::default(), |_| {}).error,
        "unsafe_target"
    );
    std::fs::remove_dir(f.output()).unwrap();
    std::fs::write(f.output(), b"previous archive").unwrap();
    let alias = f.0.join("输入/output-alias.zip");
    std::fs::hard_link(f.output(), &alias).unwrap();
    let mut job = f.job(8);
    job.options
        .inputs
        .push(f.output().to_string_lossy().into_owned());
    job.options.conflict = ConflictPolicy::Overwrite;
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "unsafe_target");
    assert_eq!(std::fs::read(f.output()).unwrap(), b"previous archive");
    assert_eq!(std::fs::read(alias).unwrap(), b"previous archive");
}

#[cfg(unix)]
#[test]
fn selected_output_parent_alias_reaches_rename_but_its_leaf_link_is_still_rejected() {
    let f = Fixture::new();
    std::fs::write(f.output(), b"previous archive").unwrap();
    std::os::unix::fs::symlink(f.0.join("output"), f.0.join("alias")).unwrap();
    let mut job = f.job(8);
    job.options
        .inputs
        .push(f.0.join("alias/archive.zip").to_string_lossy().into_owned());
    job.options.conflict = ConflictPolicy::Rename;
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.status, Status::Succeeded, "{}", result.error);
    assert_eq!(std::fs::read(f.output()).unwrap(), b"previous archive");
    let contents = archive_contents(std::fs::read(result.target).unwrap(), Format::Zip);
    assert_eq!(contents["archive.zip"], b"previous archive");
    std::fs::remove_file(f.output()).unwrap();
    std::os::unix::fs::symlink(f.0.join("输入/文件.txt"), f.output()).unwrap();
    let mut job = f.job(8);
    job.options
        .inputs
        .push(f.output().to_string_lossy().into_owned());
    let result = execute(job, &Control::default(), |_| {});
    assert_eq!(result.error, "unsafe_input");
    assert!(std::fs::symlink_metadata(f.output()).unwrap().is_symlink());
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
fn parent_aliases_match_private_workspace_exclusion_without_following_leaf_links() {
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

#[test]
fn tar_and_gzip_use_the_same_safe_creation_extraction_rename_and_cancel_barriers() {
    for (extension, method, format) in [(".tar", 0, Format::Tar), (".tar.gz", 8, Format::TarGzip)] {
        let f = Fixture::new();
        let mut job = f.job(method);
        job.options.name = format!("archive{extension}");
        let options = job.options.clone();
        let done = execute(job, &Control::default(), |_| {});
        assert_eq!(done.status, Status::Succeeded, "{}", done.error);
        let path = f.0.join("output").join(&options.name);
        let original = std::fs::read(&path).unwrap();
        let archive = crate::Archive::open(&path, &Cancellation::default()).unwrap();
        assert_eq!(archive.format(), format);
        let destination = f.0.join("extracted");
        std::fs::create_dir(&destination).unwrap();
        let extraction = crate::extraction::Options {
            target: destination.to_string_lossy().into_owned(),
            all: true,
            selected: vec![],
            directory: String::new(),
            create_folder: false,
            conflict: ConflictPolicy::Ask,
            allow_high_ratio: false,
        };
        let mut terminal = None;
        crate::extraction::run(
            archive
                .extraction_job(extraction, &Cancellation::default())
                .unwrap(),
            2,
            &Control::default(),
            &mut |s| terminal = Some(s),
        );
        assert_eq!(terminal.unwrap().status, Status::Succeeded);
        assert_eq!(
            std::fs::read(destination.join("输入/文件.txt")).unwrap(),
            std::fs::read(f.0.join("输入/文件.txt")).unwrap()
        );
        let mut renamed = options.clone();
        renamed.conflict = ConflictPolicy::Rename;
        assert_eq!(
            execute(Job { options: renamed }, &Control::default(), |_| {}).status,
            Status::Succeeded
        );
        assert!(
            f.0.join("output")
                .join(format!("archive (2){extension}"))
                .is_file()
        );
        let mut overwrite = options;
        overwrite.conflict = ConflictPolicy::Overwrite;
        let cancel = Control::default();
        let stopped = execute(Job { options: overwrite }, &cancel, |s| {
            if s.status == Status::Verifying {
                cancel.cancel();
            }
        });
        assert_eq!(stopped.status, Status::Cancelled);
        assert_eq!(std::fs::read(path).unwrap(), original);
        assert_eq!(std::fs::read_dir(f.0.join("output")).unwrap().count(), 2);
    }
}

#[test]
fn gzip_ratio_requires_explicit_intent_and_opt_in_still_uses_the_safe_output_job() {
    let f = Fixture::new();
    let input = f.0.join("输入/文件.txt");
    std::fs::write(&input, vec![0; 32 * 1024 * 1024]).unwrap();
    let mut create = f.job(8);
    create.options.name = "ratio.tar.gz".into();
    assert_eq!(
        execute(create, &Control::default(), |_| {}).status,
        Status::Succeeded
    );
    let path = f.0.join("output/ratio.tar.gz");
    let archive = crate::Archive::open(&path, &Cancellation::default()).unwrap();
    let target = f.0.join("ratio-output");
    std::fs::create_dir(&target).unwrap();
    let mut options = crate::extraction::Options {
        target: target.to_string_lossy().into_owned(),
        all: true,
        selected: vec![],
        directory: String::new(),
        create_folder: false,
        conflict: ConflictPolicy::Ask,
        allow_high_ratio: false,
    };
    let admission = archive.extraction_job(options.clone(), &Cancellation::default());
    assert!(
        matches!(admission, Err("high_ratio")),
        "expected high-ratio admission, got {:?}, archive bytes {}",
        admission.err(),
        std::fs::metadata(&path).unwrap().len()
    );
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
    options.allow_high_ratio = true;
    let mut terminal = None;
    crate::extraction::run(
        archive
            .extraction_job(options, &Cancellation::default())
            .unwrap(),
        3,
        &Control::default(),
        &mut |s| terminal = Some(s),
    );
    assert_eq!(terminal.unwrap().status, Status::Succeeded);
    assert_eq!(
        std::fs::metadata(target.join("输入/文件.txt"))
            .unwrap()
            .len(),
        32 * 1024 * 1024
    );
}

#[test]
fn macos_metadata_is_filtered_from_creation_in_all_formats() {
    for (extension, format) in [
        ("zip", Format::Zip),
        ("tar", Format::Tar),
        ("tar.gz", Format::TarGzip),
    ] {
        let f = Fixture::new();
        for path in [
            "输入/__MACOSX",
            "输入/子目录/_MACOSX",
            "输入/仅元数据/下级",
            "__MACOSX",
        ] {
            std::fs::create_dir_all(f.0.join(path)).unwrap();
        }
        for path in [
            ".DS_Store",
            "__MACOSX/._root",
            "输入/.DS_Store",
            "输入/__MACOSX/._file",
            "输入/子目录/.DS_Store",
            "输入/子目录/_MACOSX/._file",
            "输入/仅元数据/下级/.DS_Store",
        ] {
            std::fs::write(f.0.join(path), b"metadata").unwrap();
        }
        let mut job = f.job(if extension == "tar" { 0 } else { 8 });
        job.options.name = format!("archive.{extension}");
        job.options.inputs.extend([
            f.0.join(".DS_Store").to_string_lossy().into_owned(),
            f.0.join("__MACOSX").to_string_lossy().into_owned(),
        ]);
        let result = execute(job, &Control::default(), |_| {});
        assert_eq!(
            result.status,
            Status::Succeeded,
            "{extension}: {}",
            result.error
        );
        assert_eq!(result.committed_files, 2);
        assert_eq!(result.created_directories, 2);
        let path = f.0.join(format!("output/archive.{extension}"));
        let bytes = std::fs::read(&path).unwrap();
        let rows = codec(format)
            .scan(
                &mut std::io::Cursor::new(&bytes),
                &Cancellation::default(),
                crate::formats::limits(format),
            )
            .unwrap();
        assert_eq!(rows.len(), 4);
        let contents = archive_contents(bytes, format);
        assert_eq!(
            contents["输入/文件.txt"],
            "中文与 ZIP 互操作\n".repeat(1000).as_bytes()
        );
        assert!(contents["输入/empty.bin"].is_empty());
        assert_eq!(
            std::fs::read(f.0.join("输入/__MACOSX/._file")).unwrap(),
            b"metadata"
        );
        if let Some(output) = std::env::var_os("XDZIP_METADATA_INTEROP_OUTPUT") {
            std::fs::create_dir_all(&output).unwrap();
            std::fs::copy(
                &path,
                PathBuf::from(output).join(format!("created.{extension}")),
            )
            .unwrap();
        }
    }
}
#[test]
fn metadata_only_creation_fails_before_encoding_or_conflict_and_preserves_outputs() {
    for extension in ["zip", "tar", "tar.gz"] {
        for selected in [".DS_Store", "__MACOSX", "仅元数据"] {
            for existing in [false, true] {
                let f = Fixture::new();
                std::fs::create_dir_all(f.0.join("仅元数据/下级/__MACOSX")).unwrap();
                std::fs::create_dir(f.0.join("__MACOSX")).unwrap();
                std::fs::write(f.0.join(".DS_Store"), b"metadata").unwrap();
                std::fs::write(f.0.join("__MACOSX/._file"), b"metadata").unwrap();
                std::fs::write(f.0.join("仅元数据/下级/__MACOSX/._file"), b"metadata").unwrap();
                let name = format!("archive.{extension}");
                let path = f.0.join("output").join(&name);
                if existing {
                    std::fs::write(&path, b"owned original").unwrap();
                }
                let mut job = f.job(if extension == "tar" { 0 } else { 8 });
                job.options.name = name;
                job.options.inputs = vec![f.0.join(selected).to_string_lossy().into_owned()];
                job.options.conflict = ConflictPolicy::Overwrite;
                let result = execute(job, &Control::default(), |s| {
                    assert!(!matches!(
                        s.status,
                        Status::Running | Status::Verifying | Status::Waiting
                    ));
                });
                assert_eq!(result.status, Status::Failed);
                assert_eq!(result.error, "empty_input");
                assert_eq!(result.committed_files, 0);
                assert_eq!(result.written_bytes, 0);
                if existing {
                    assert_eq!(std::fs::read(path).unwrap(), b"owned original");
                } else {
                    assert!(!path.exists());
                }
                assert_eq!(
                    std::fs::read_dir(f.0.join("output")).unwrap().count(),
                    usize::from(existing)
                );
            }
        }
    }
}
#[test]
fn metadata_filter_also_applies_to_encrypted_and_numbered_split_creation() {
    for (encrypted, split) in [(true, false), (false, true), (true, true)] {
        let f = Fixture::new();
        std::fs::write(f.0.join("输入/.DS_Store"), b"metadata").unwrap();
        std::fs::create_dir(f.0.join("输入/__MACOSX")).unwrap();
        std::fs::write(f.0.join("输入/__MACOSX/._file"), b"metadata").unwrap();
        let mut job = f.job(8);
        if encrypted {
            job.options.password = Some(Password::new("test password".into()).unwrap());
        }
        if split {
            job.options.volume_bytes = 128;
        }
        let result = execute(job, &Control::default(), |_| {});
        assert_eq!(result.status, Status::Succeeded, "{}", result.error);
        let mut bytes = Vec::new();
        if split {
            assert!(result.output_parts > 1);
            for part in 1..=result.output_parts {
                bytes.extend(
                    std::fs::read(f.0.join(format!("output/archive.zip.{part:03}"))).unwrap(),
                );
            }
        } else {
            bytes = std::fs::read(f.output()).unwrap();
        }
        let rows = ZipCodec
            .scan(
                &mut std::io::Cursor::new(bytes),
                &Cancellation::default(),
                ScanLimits::default(),
            )
            .unwrap();
        assert_eq!(rows.len(), 4);
        assert!(
            rows.iter()
                .all(|r| !r.raw_name.windows(9).any(|s| s == b".DS_Store")
                    && !r.raw_name.windows(8).any(|s| s == b"__MACOSX"))
        );
    }
}

#[test]
fn native_creation_capabilities_are_format_specific() {
    let fixture = Fixture::new();
    let mut options = fixture.job(32).options;
    options.name = "archive.7z".into();
    assert!(options.bounded());
    options.solid = true;
    assert!(options.bounded());
    options.encrypt_headers = true;
    assert!(!options.bounded());
    options.password = Some(Password::new("test secret".into()).unwrap());
    assert!(options.bounded());
    options.volume_bytes = 4096;
    assert!(options.bounded());
    options.name = "archive.rar".into();
    assert!(!options.bounded());
    options.name = "archive.zip".into();
    options.method = 8;
    assert!(!options.bounded());
    options.solid = false;
    options.encrypt_headers = false;
    assert!(options.bounded());
}
