// crates/xdzip-runtime/src/extraction/opening_tests.rs

use super::{tests::*, *};
use std::path::{Path, PathBuf};
fn bytes(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/extract")
            .join(name),
    )
    .unwrap()
}
fn numbered(root: &Path, b: &[u8], size: usize) -> Vec<PathBuf> {
    b.chunks(size)
        .enumerate()
        .map(|(i, chunk)| {
            let p = root.join(format!("分卷.zip.{:03}", i + 1));
            std::fs::write(&p, chunk).unwrap();
            p
        })
        .collect()
}
#[test]
fn numbered_parts_browse_from_any_volume_and_extract_across_headers_and_payload() {
    for fixture in [
        "stored.zip",
        "deflate.zip",
        "descriptor.zip",
        "descriptor64.zip",
    ] {
        let source = Target::new();
        let output = Target::new();
        let original = archive(fixture);
        let parts = numbered(&source.0, &bytes(fixture), 31);
        for part in [&parts[0], &parts[parts.len() / 2], parts.last().unwrap()] {
            let a = crate::Archive::open(part, &Cancellation::default())
                .unwrap_or_else(|e| panic!("{fixture}: {e}"));
            assert_eq!(a.rows, original.rows);
            assert_eq!(a.output_folder(), "分卷");
        }
        let a = crate::Archive::open(&parts[0], &Cancellation::default()).unwrap();
        let done = execute(&a, output.options(), &Control::default(), |_| {});
        assert_eq!(done.status, Status::Succeeded, "{fixture}: {}", done.error);
        assert_eq!(done.committed_files, 1);
        std::fs::write(&parts[1], b"changed").unwrap();
        assert_eq!(
            a.source.validate(),
            Err(xdzip_codec_api::ScanError::SourceChanged)
        );
    }
}
#[test]
fn missing_or_ambiguous_parts_are_rejected_without_partial_rows() {
    let source = Target::new();
    let parts = numbered(&source.0, &bytes("stored.zip"), 31);
    std::fs::remove_file(&parts[1]).unwrap();
    assert!(matches!(
        crate::Archive::open(&parts[0], &Cancellation::default()),
        Err(xdzip_codec_api::ScanError::MissingVolume)
    ));
    let mut footer = bytes("stored.zip").split_off(bytes("stored.zip").len() - 22);
    footer[4..6].copy_from_slice(&1u16.to_le_bytes());
    let final_only = source.0.join("final-only.zip");
    std::fs::write(&final_only, footer).unwrap();
    assert!(
        matches!(
            crate::Archive::open(&final_only, &Cancellation::default()),
            Err(xdzip_codec_api::ScanError::MissingVolume)
        ),
        "a known final disk must report missing preceding volumes"
    );
}
#[cfg(unix)]
#[test]
fn even_a_single_numbered_volume_rejects_link_replacement() {
    let source = Target::new();
    let file = source.0.join("owned.zip.001");
    std::fs::write(&file, bytes("stored.zip")).unwrap();
    let a = crate::Archive::open(&file, &Cancellation::default()).unwrap();
    let original = source.0.join("original.zip");
    std::fs::rename(&file, &original).unwrap();
    std::os::unix::fs::symlink(&original, &file).unwrap();
    assert_eq!(
        a.source.validate(),
        Err(xdzip_codec_api::ScanError::SourceChanged)
    );
    assert!(crate::Archive::open(&file, &Cancellation::default()).is_err());
}
#[test]
fn pkzip_disk_relative_directory_is_normalized_without_reassembling_payload() {
    for fixture in [
        "stored.zip",
        "deflate.zip",
        "descriptor.zip",
        "descriptor64.zip",
        "tree.zip",
    ] {
        let source = Target::new();
        let output = Target::new();
        let mut b = bytes(fixture);
        let end = b.windows(4).rposition(|s| s == b"PK\x05\x06").unwrap();
        let cd = u32::from_le_bytes(b[end + 16..end + 20].try_into().unwrap()) as usize;
        let size = if fixture == "tree.zip" { 4096 } else { 53 };
        let mut record = cd;
        while b.get(record..record + 4) == Some(b"PK\x01\x02") {
            let offset =
                u32::from_le_bytes(b[record + 42..record + 46].try_into().unwrap()) as usize;
            b[record + 34..record + 36].copy_from_slice(&((offset / size) as u16).to_le_bytes());
            b[record + 42..record + 46].copy_from_slice(&((offset % size) as u32).to_le_bytes());
            record += 46
                + [28, 30, 32]
                    .into_iter()
                    .map(|n| {
                        usize::from(u16::from_le_bytes(
                            b[record + n..record + n + 2].try_into().unwrap(),
                        ))
                    })
                    .sum::<usize>();
        }
        let last = (b.len() - 1) / size;
        b[end + 4..end + 6].copy_from_slice(&(last as u16).to_le_bytes());
        b[end + 6..end + 8].copy_from_slice(&((cd / size) as u16).to_le_bytes());
        b[end + 16..end + 20].copy_from_slice(&((cd % size) as u32).to_le_bytes());
        let mut parts = Vec::new();
        for (i, chunk) in b.chunks(size).enumerate() {
            let p = source.0.join(if i == last {
                "分卷.zip".into()
            } else {
                format!("分卷.z{:02}", i + 1)
            });
            std::fs::write(&p, chunk).unwrap();
            parts.push(p);
        }
        for part in [&parts[0], parts.last().unwrap()] {
            let a = crate::Archive::open(part, &Cancellation::default())
                .unwrap_or_else(|e| panic!("{fixture}: {e}"));
            assert_eq!(a.len(), archive(fixture).len());
        }
        let a = crate::Archive::open(&parts[0], &Cancellation::default()).unwrap();
        let done = execute(&a, output.options(), &Control::default(), |_| {});
        assert_eq!(done.status, Status::Succeeded, "{fixture}: {}", done.error);
        std::fs::remove_file(parts.last().unwrap()).unwrap();
        assert!(matches!(
            crate::Archive::open(&parts[0], &Cancellation::default()),
            Err(xdzip_codec_api::ScanError::MissingVolume)
        ));
    }
}
#[test]
fn temporary_open_is_exactly_one_verified_ordinary_file_and_survives_job_completion() {
    let cache = Target::new();
    let a = archive("tree.zip");
    let entry = a
        .rows
        .iter()
        .find(|r| r.display_name.ends_with("说明.txt"))
        .unwrap();
    let job = a
        .open_entry_job(entry.id.get(), &cache.0, &Cancellation::default())
        .unwrap();
    let path = PathBuf::from(&job.open_path);
    assert_eq!(path.file_name().unwrap(), "说明.txt");
    assert!(path.starts_with(&cache.0));
    assert!(!path.exists());
    let mut done = None;
    run(job, 42, &Control::default(), &mut |s| done = Some(s));
    let done = done.unwrap();
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(done.committed_files, 1);
    assert!(path.is_file());
    assert_eq!(std::fs::metadata(&path).unwrap().len(), entry.size);
    let second = a
        .open_entry_job(entry.id.get(), &cache.0, &Cancellation::default())
        .unwrap();
    assert_ne!(second.open_path, done.open_path);
    assert!(
        a.open_entry_job(u64::MAX, &cache.0, &Cancellation::default())
            .is_err()
    );
    let directory = a.rows.iter().find(|r| r.directory).unwrap();
    assert!(
        a.open_entry_job(directory.id.get(), &cache.0, &Cancellation::default())
            .is_err()
    );
}
#[test]
fn encrypted_split_temporary_open_retries_password_and_cancel_never_publishes() {
    let source = Target::new();
    let cache = Target::new();
    let data = super::password_tests::archive_bytes(
        &[(
            "目录/文档 % & space.txt",
            b"verified contents",
            Some("owned-test-secret"),
            false,
        )],
        8,
    );
    let parts = numbered(&source.0, &data, 17);
    let a = crate::Archive::open(parts.last().unwrap(), &Cancellation::default()).unwrap();
    let job = a
        .open_entry_job(1, &cache.0, &Cancellation::default())
        .unwrap();
    let path = PathBuf::from(&job.open_path);
    let control = Control::default();
    let mut prompts = 0;
    let mut last = None;
    run(job, 1, &control, &mut |s| {
        if let Some(p) = &s.password_prompt {
            prompts += 1;
            assert!(
                control.respond_password(
                    p.id,
                    Password::new(
                        if prompts == 1 {
                            "wrong"
                        } else {
                            "owned-test-secret"
                        }
                        .into()
                    )
                    .unwrap()
                )
            );
        }
        last = Some(s);
    });
    assert_eq!(prompts, 2);
    assert_eq!(last.unwrap().status, Status::Succeeded);
    assert_eq!(std::fs::read(path).unwrap(), b"verified contents");
    let job = a
        .open_entry_job(1, &cache.0, &Cancellation::default())
        .unwrap();
    let path = PathBuf::from(&job.open_path);
    let control = Control::default();
    let mut last = None;
    run(job, 2, &control, &mut |s| {
        if s.password_prompt.is_some() {
            control.cancel();
        }
        last = Some(s);
    });
    assert_eq!(last.unwrap().status, Status::Cancelled);
    assert!(!path.exists());
    let links = archive("unix_bundle.zip");
    let entry = links.rows.iter().find(|r| r.link).unwrap();
    assert!(
        links
            .open_entry_job(entry.id.get(), &cache.0, &Cancellation::default())
            .is_err()
    );
}
