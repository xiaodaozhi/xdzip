//! Actual host tasks for Unix compress and its existing TAR composition.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-compress-{}-{}",
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
        .join("../../tests/fixtures/compress")
        .join(name)
}
fn pattern(count: usize) -> Vec<u8> {
    let mut state = 0x6d2b79f5u32;
    (0..count)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state >> 24) as u8
        })
        .collect()
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
fn independent_compress_profiles_use_real_browse_check_and_extract() {
    let cancel = Cancellation::default();
    let phrase = b"abracadabra abracadabra\n".repeat(6);
    let mut natural = phrase.repeat(4000);
    natural.extend(pattern(250000));
    for (name, expected) in [
        ("phrase.Z", phrase.clone()),
        ("empty.Z", vec![]),
        ("nonblock.Z", b"ABABABA".to_vec()),
        ("clear.Z", b"ABABCDCD".to_vec()),
        ("width-9.Z", pattern((2 << 9) + 2048)),
        ("width-16.Z", pattern((2 << 16) + 2048)),
        ("natural-clear.Z", natural),
        ("files.taz", phrase),
        ("zeros.Z", vec![0; 4096]),
        ("empty-tar.Z", vec![0; 1024]),
    ] {
        let scratch = Scratch::new();
        let archive = Archive::open(&fixture(name), &cancel).unwrap();
        let tar = name == "files.taz";
        assert_eq!(
            archive.format,
            if tar {
                xdzip_codec_api::Format::TarCompress
            } else {
                xdzip_codec_api::Format::Compress
            },
            "{name}"
        );
        assert_eq!(
            archive.update_allowed,
            archive.format == xdzip_codec_api::Format::TarCompress
        );
        assert_eq!(archive.rows.len(), 1);
        assert_eq!(archive.rows[0].method, 257);
        assert_eq!(archive.rows[0].packed_size_known(), !tar);
        let mut result = None;
        checking::run(
            archive.check_job(&cancel).unwrap(),
            1,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        let result = result.unwrap();
        assert_eq!(result.status, Status::Succeeded, "{name}: {}", result.error);
        assert_eq!(result.verified_bytes, expected.len() as u64);
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
            std::fs::read(
                scratch
                    .0
                    .join(std::str::from_utf8(archive.rows[0].name_bytes()).unwrap())
            )
            .unwrap(),
            expected
        );
    }
}
#[test]
fn zero_only_same_filter_hints_and_real_tar_headers_keep_distinct_views() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    for (name, format) in [
        ("ordinary.Z", xdzip_codec_api::Format::Compress),
        ("empty.tar.Z", xdzip_codec_api::Format::TarCompress),
        ("empty.taz", xdzip_codec_api::Format::TarCompress),
        ("unknown.bin", xdzip_codec_api::Format::TarCompress),
        ("different.gz", xdzip_codec_api::Format::TarCompress),
    ] {
        let source = scratch.0.join(name);
        std::fs::copy(fixture("empty-tar.Z"), &source).unwrap();
        let archive = Archive::open(&source, &cancel).unwrap();
        assert_eq!(archive.format, format, "{name}");
        assert_eq!(
            archive.rows.len(),
            usize::from(format == xdzip_codec_api::Format::Compress)
        );
        let mut result = None;
        checking::run(
            archive.check_job(&cancel).unwrap(),
            1,
            &Control::default(),
            &mut |s| result = Some(s),
        );
        assert_eq!(result.unwrap().status, Status::Succeeded);
    }
    let source = scratch.0.join("actual.Z");
    std::fs::copy(fixture("files.taz"), &source).unwrap();
    assert_eq!(
        Archive::open(&source, &cancel).unwrap().format,
        xdzip_codec_api::Format::TarCompress
    );
    let bad = scratch.0.join("damaged.Z");
    std::fs::copy(fixture("bad-header.Z"), &bad).unwrap();
    assert!(Archive::open(&bad, &cancel).is_err());
}
#[test]
fn malformed_and_unsupported_short_headers_fail_without_publishing() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let source = scratch.0.join("bad.Z");
    for bytes in [
        b"\x1f\x9d".as_slice(),
        b"\x1f\x9d\x90\xff\x01",
        b"\x1f\x9d\xb0",
        b"\x1f\x9d\x90\x41",
    ] {
        std::fs::write(&source, bytes).unwrap();
        assert!(Archive::open(&source, &cancel).is_err(), "{bytes:?}");
    }
}
#[test]
fn cancelled_and_changed_sources_keep_existing_output_unchanged() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let source = scratch.0.join("owned.Z");
    std::fs::copy(fixture("phrase.Z"), &source).unwrap();
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
    bytes.push(0);
    std::fs::write(&source, bytes).unwrap();
    let mut result = None;
    extraction::run(job, 2, &Control::default(), &mut |s| result = Some(s));
    let result = result.unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert_eq!(std::fs::read(target.join("owned")).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
}
