//! Product paths for owned reference streams, without extension-based trust.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Status};
use std::path::PathBuf;
use xdzip_codec_api::Format;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-stream-{}-{}",
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
        .join("../../tests/fixtures/streams")
        .join(name)
}
fn options(path: &Path) -> Options {
    Options {
        target: path.to_string_lossy().into_owned(),
        all: true,
        selected: vec![],
        directory: String::new(),
        create_folder: false,
        shell_layout: 0,
        conflict: ConflictPolicy::Ask,
        allow_high_ratio: false,
    }
}
fn verify(name: &str, format: Format, output_name: &str, expected: &[u8]) {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let archive = Archive::open(&fixture(name), &cancel).unwrap();
    assert_eq!(archive.format, format);
    assert!(!archive.update_allowed);
    assert_eq!(archive.rows.len(), 1);
    assert_eq!(archive.rows[0].name_bytes(), output_name.as_bytes());
    assert_eq!(archive.rows[0].size, expected.len() as u64);
    assert!(archive.rows[0].size_known);
    let mut checked = None;
    crate::checking::run(
        archive.check_job(&cancel).unwrap(),
        1,
        &Control::default(),
        &mut |s| checked = Some(s),
    );
    let checked = checked.unwrap();
    assert_eq!(
        checked.status,
        Status::Succeeded,
        "{name}: {}",
        checked.error
    );
    assert_eq!(checked.verified_files, 1);
    assert_eq!(checked.verified_bytes, expected.len() as u64);
    let mut result = None;
    crate::extraction::run(
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
        std::fs::read(scratch.0.join(output_name)).unwrap(),
        expected
    );
}
#[test]
fn ordinary_streams_browse_check_and_extract_real_content() {
    let phrase = b"abracadabra abracadabra\n".repeat(6);
    for (name, format, output_name, expected) in [
        ("phrase.bz2", Format::Bzip2, "phrase", phrase.clone()),
        ("empty.bz2", Format::Bzip2, "empty", vec![]),
        ("multiple.bz2", Format::Bzip2, "multiple", phrase.clone()),
        ("phrase.lzma", Format::Lzma, "phrase", phrase.clone()),
        (
            "known-size.lzma",
            Format::Lzma,
            "known-size",
            phrase.clone(),
        ),
        ("empty.lzma", Format::Lzma, "empty", vec![]),
        ("phrase.lz", Format::Lzip, "phrase", phrase.clone()),
        ("empty.lz", Format::Lzip, "empty", vec![]),
        ("multiple.lz", Format::Lzip, "multiple", phrase.repeat(2)),
        ("phrase-none.xz", Format::Xz, "phrase-none", phrase.clone()),
        (
            "phrase-crc32.xz",
            Format::Xz,
            "phrase-crc32",
            phrase.clone(),
        ),
        (
            "phrase-crc64.xz",
            Format::Xz,
            "phrase-crc64",
            phrase.clone(),
        ),
        (
            "phrase-sha256.xz",
            Format::Xz,
            "phrase-sha256",
            phrase.clone(),
        ),
        ("empty.xz", Format::Xz, "empty", vec![]),
        ("phrase.lz4", Format::Lz4, "phrase", phrase.clone()),
        (
            "phrase-block-checksum.lz4",
            Format::Lz4,
            "phrase-block-checksum",
            phrase.clone(),
        ),
        (
            "phrase-no-checksum.lz4",
            Format::Lz4,
            "phrase-no-checksum",
            phrase.clone(),
        ),
        ("empty.lz4", Format::Lz4, "empty", vec![]),
        ("phrase.zst", Format::Zstd, "phrase", phrase.clone()),
        (
            "phrase-no-checksum.zst",
            Format::Zstd,
            "phrase-no-checksum",
            phrase.clone(),
        ),
        (
            "phrase-unknown-size.zst",
            Format::Zstd,
            "phrase-unknown-size",
            phrase.clone(),
        ),
        ("empty.zst", Format::Zstd, "empty", vec![]),
        ("multiple.zst", Format::Zstd, "multiple", phrase.repeat(2)),
        (
            "leading-skip.zst",
            Format::Zstd,
            "leading-skip",
            phrase.clone(),
        ),
        ("multiple.lz4", Format::Lz4, "multiple", phrase.repeat(2)),
        (
            "leading-skip.lz4",
            Format::Lz4,
            "leading-skip",
            phrase.clone(),
        ),
        ("multiple.xz", Format::Xz, "multiple", phrase.repeat(2)),
        ("multiblock.xz", Format::Xz, "multiblock", phrase.repeat(4)),
    ] {
        verify(name, format, output_name, &expected);
    }
}
#[test]
fn zipx_methods_check_and_extract_without_granting_modification() {
    for method in ["deflate64", "bzip2", "lzma", "xz"] {
        for descriptor in [false, true] {
            let name = format!(
                "zipx-{method}{}.zipx",
                if descriptor { "-descriptor" } else { "" }
            );
            verify(
                &name,
                Format::Zip,
                "hello.txt",
                &b"abracadabra abracadabra\n".repeat(6),
            );
        }
    }
}
#[test]
fn tar_compositions_reuse_the_container_and_stay_read_only() {
    for (name, format) in [
        ("files.tbz2", Format::TarBzip2),
        ("files.tar.lzma", Format::TarLzma),
        ("files.tlz", Format::TarLzip),
        ("files.txz", Format::TarXz),
        ("files.tar.lz4", Format::TarLz4),
        ("files.tar.zst", Format::TarZstd),
    ] {
        verify(
            name,
            format,
            "hello.txt",
            &b"abracadabra abracadabra\n".repeat(6),
        );
    }
}
#[test]
fn weak_lzma_discovery_cannot_supersede_valid_tar_or_zip_sfx() {
    use std::io::{Cursor, Read};
    use xdzip_codec_stream::controlled::{ControlledStream, Filter};
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    let mut compressed = Cursor::new(std::fs::read(fixture("files.tar.lzma")).unwrap());
    let mut stream =
        ControlledStream::new(&mut compressed, &cancel, Filter::Lzma, 1 << 20, 1 << 20).unwrap();
    let mut tar = Vec::new();
    stream.read_to_end(&mut tar).unwrap();
    tar[..100].fill(0);
    tar[0] = b'a';
    tar[148..156].fill(b' ');
    let sum: u64 = tar[..512].iter().map(|b| u64::from(*b)).sum();
    tar[148..156].copy_from_slice(format!("{sum:06o}\0 ").as_bytes());
    let path = scratch.0.join("misleading.lzma");
    std::fs::write(&path, &tar).unwrap();
    assert_eq!(Archive::open(&path, &cancel).unwrap().format, Format::Tar);
    let mut sfx = vec![0; 512];
    sfx[..2].copy_from_slice(b"MZ");
    sfx.extend(
        std::fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip"),
        )
        .unwrap(),
    );
    std::fs::write(&path, &sfx).unwrap();
    assert_eq!(Archive::open(&path, &cancel).unwrap().format, Format::Zip);
}

#[test]
fn identified_compressed_tar_damage_never_downgrades_to_an_ordinary_file() {
    for name in [
        "bad-header.tbz2",
        "bad-header.tar.lzma",
        "bad-header.tlz",
        "bad-header.txz",
        "bad-header.tar.lz4",
        "bad-header.tar.zst",
    ] {
        assert!(
            matches!(
                Archive::open(&fixture(name), &Cancellation::default()),
                Err(ScanError::Invalid)
            ),
            "{name}"
        );
    }
}

#[test]
fn changed_stream_payload_cannot_publish_a_file_even_if_source_timestamps_are_restored() {
    for (name, at) in [
        ("phrase.bz2", 10),
        ("known-size.lzma", 5),
        ("phrase-crc64.xz", 10),
        ("phrase.lz4", 6),
        ("phrase.zst", 8),
        (
            "phrase.lz",
            std::fs::read(fixture("phrase.lz")).unwrap().len() - 20,
        ),
    ] {
        let source = Scratch::new();
        let target = Scratch::new();
        let path = source.0.join(name);
        let mut bytes = std::fs::read(fixture(name)).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        let modified = std::fs::metadata(&path).unwrap().modified().unwrap();
        let cancel = Cancellation::default();
        let archive = Archive::open(&path, &cancel).unwrap();
        bytes[at] ^= 1;
        std::fs::write(&path, &bytes).unwrap();
        std::fs::File::options()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(modified)
            .unwrap();
        let job = archive.extraction_job(options(&target.0), &cancel).unwrap();
        let mut done = None;
        crate::extraction::run(job, 1, &Control::default(), &mut |s| done = Some(s));
        let done = done.unwrap();
        assert_eq!(done.status, Status::Failed);
        assert_eq!(done.committed_files, 0);
        assert!(
            !target
                .0
                .join(if name == "known-size.lzma" {
                    "known-size"
                } else {
                    "phrase"
                })
                .exists()
        );
    }
}
