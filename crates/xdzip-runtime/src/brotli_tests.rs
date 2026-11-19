//! Exact admission and actual host tasks for headerless Brotli streams and TAR composition.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-brotli-{}-{}",
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
        .join("../../tests/fixtures/brotli")
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
        conflict: ConflictPolicy::Overwrite,
        allow_high_ratio: false,
    }
}
#[test]
fn independent_streams_are_admitted_by_content_before_any_format_or_rows_are_published() {
    let cancel = Cancellation::default();
    let mut count = 0;
    for path in std::fs::read_dir(fixture("")).unwrap() {
        let path = path.unwrap().path();
        if !path.extension().is_some_and(|v| v == "br") {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy();
        if name == "bad-header.br" {
            continue;
        }
        let archive = Archive::open(&path, &cancel).unwrap_or_else(|e| panic!("{name}: {e}"));
        let tar = name.ends_with(".tar.br");
        assert_eq!(
            archive.format,
            if tar {
                xdzip_codec_api::Format::TarBrotli
            } else {
                xdzip_codec_api::Format::Brotli
            },
            "{name}"
        );
        assert_eq!(
            archive.update_allowed,
            archive.format == xdzip_codec_api::Format::TarBrotli
        );
        assert!(
            archive
                .rows
                .iter()
                .all(|r| r.method == 258 && r.payload_supported())
        );
        assert!(archive.rows.iter().all(|r| r.packed_size_known() != tar));
        let mut result = None;
        checking::run(
            archive.check_job(&cancel).unwrap(),
            1,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        count += 1;
    }
    assert_eq!(count, 83);
}
#[test]
fn actual_selected_export_and_preview_preserve_read_only_source_and_payload_identity() {
    let cancel = Cancellation::default();
    for (name, member, expected) in [
        (
            "q11-text.br",
            "q11-text",
            b"abracadabra abracadabra\n".repeat(6),
        ),
        (
            "files.tar.br",
            "hello.txt",
            b"abracadabra abracadabra\n".repeat(6),
        ),
        ("empty.br", "empty", vec![]),
        ("raw.br", "raw", b"Z".to_vec()),
    ] {
        let scratch = Scratch::new();
        let source = scratch.0.join(if name == "q11-text.br" {
            "owned.txt.br"
        } else {
            name
        });
        std::fs::copy(fixture(name), &source).unwrap();
        let archive = Archive::open(&source, &cancel).unwrap();
        let member = if name == "q11-text.br" {
            "owned.txt"
        } else {
            member
        };
        let row = archive
            .rows
            .iter()
            .find(|r| r.name_bytes() == member.as_bytes())
            .unwrap();
        if name == "q11-text.br" {
            assert_eq!(
                archive
                    .preview(row.id.get(), None, &cancel)
                    .unwrap()
                    .bytes
                    .as_slice(),
                expected
            );
        }
        let target = scratch.0.join("output");
        let mut selected = options(&target);
        selected.all = false;
        selected.selected = vec![row.id.get().to_string()];
        let mut result = None;
        extraction::run(
            archive.extraction_job(selected, &cancel).unwrap(),
            2,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(result.committed_files, 1);
        assert_eq!(std::fs::read(target.join(member)).unwrap(), expected);
    }
}
#[test]
fn zero_only_views_cross_suffixes_and_damaged_tar_keep_their_declared_boundaries() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    for (name, format) in [
        ("ordinary.br", xdzip_codec_api::Format::Brotli),
        ("empty.tar.br", xdzip_codec_api::Format::TarBrotli),
        ("unknown.zip", xdzip_codec_api::Format::TarBrotli),
        ("different.gz", xdzip_codec_api::Format::TarBrotli),
    ] {
        let path = scratch.0.join(name);
        std::fs::copy(fixture("empty-tar.br"), &path).unwrap();
        let archive = Archive::open(&path, &cancel).unwrap();
        assert_eq!(archive.format, format);
    }
    let path = scratch.0.join("source.zip");
    std::fs::copy(fixture("q11-text.br"), &path).unwrap();
    let archive = Archive::open(&path, &cancel).unwrap();
    assert_eq!(archive.format, xdzip_codec_api::Format::Brotli);
    assert_eq!(archive.rows[0].name_bytes(), b"content");
    std::fs::copy(fixture("files.tar.br"), &path).unwrap();
    assert_eq!(
        Archive::open(&path, &cancel).unwrap().format,
        xdzip_codec_api::Format::TarBrotli
    );
    assert!(matches!(
        Archive::open(&fixture("bad-header.br"), &cancel),
        Err(ScanError::Invalid)
    ));
    let archive = Archive::open(&fixture("raw-tar-marker.br"), &cancel).unwrap();
    assert_eq!(archive.format, xdzip_codec_api::Format::Brotli);
    assert_eq!(archive.rows[0].size, 512);
}
#[test]
fn cancellation_truncation_and_changed_sources_preserve_existing_output() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let source = scratch.0.join("owned.br");
    std::fs::copy(fixture("q11-text.br"), &source).unwrap();
    let archive = Archive::open(&source, &cancel).unwrap();
    let target = scratch.0.join("output");
    let opts = options(&target);
    std::fs::write(target.join("owned"), b"keep").unwrap();
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
    let mut bytes = std::fs::read(&source).unwrap();
    bytes.pop();
    std::fs::write(&source, bytes).unwrap();
    let mut result = None;
    extraction::run(job, 2, &Control::default(), &mut |s| result = Some(s));
    let result = result.unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert!(Archive::open(&source, &cancel).is_err());
    assert_eq!(std::fs::read(target.join("owned")).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
}
