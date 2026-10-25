// crates/xdzip-runtime/src/native_backend_tests.rs

//! Real isolated-worker tasks. Opt in after building the locked worker.
use crate::{Archive, checking, creation, extraction, updating};
use extraction::{ConflictPolicy, Control, Snapshot, Status};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
use xdzip_codec_api::{
    Cancellation, Format, ScanError,
    payload::{DecodeError, DecodedSink},
};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "xdzip-native-tasks-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(root.join("input/empty")).unwrap();
        std::fs::create_dir(root.join("output")).unwrap();
        std::fs::write(root.join("input/first.txt"), b"first known bytes").unwrap();
        std::fs::write(root.join("input/second.txt"), "Unicode 内容".repeat(500)).unwrap();
        Self(root)
    }
    fn create(
        &self,
        name: &str,
        method: u16,
        solid: bool,
        password: bool,
        headers: bool,
        volume: u64,
    ) -> Snapshot {
        let options = creation::Options {
            inputs: vec![self.0.join("input").to_string_lossy().into()],
            target: self.0.join("output").to_string_lossy().into(),
            name: name.into(),
            method,
            level: 6,
            conflict: ConflictPolicy::Ask,
            volume_bytes: volume,
            solid,
            encrypt_headers: headers,
            password: password.then(|| creation::Password::new("password".into()).unwrap()),
        };
        assert!(options.bounded());
        task(false, |control, publish| {
            creation::run(creation::Job { options }, 1, control, publish)
        })
    }
    fn extract(&self, archive: &Archive, label: &str, mixed: bool) -> Snapshot {
        let target = self.0.join(label);
        std::fs::create_dir(&target).unwrap();
        let job = archive
            .extraction_job(
                extraction::Options {
                    target: target.to_string_lossy().into(),
                    all: true,
                    selected: vec![],
                    directory: String::new(),
                    create_folder: false,
                    conflict: ConflictPolicy::Ask,
                    allow_high_ratio: true,
                },
                &Default::default(),
            )
            .unwrap();
        task(mixed, |control, publish| {
            extraction::run(job, 2, control, publish)
        })
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn task(mixed: bool, run: impl FnOnce(&Control, &mut dyn FnMut(Snapshot))) -> Snapshot {
    let control = Control::default();
    let mut terminal = None;
    let mut count = 0;
    run(&control, &mut |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            let secret = if mixed && prompt.name.ends_with("d.txt") {
                "password2"
            } else {
                "password"
            };
            assert!(
                control
                    .respond_password(prompt.id, extraction::Password::new(secret.into()).unwrap())
            );
        }
        if snapshot.status.terminal() {
            terminal = Some(snapshot);
            count += 1;
        }
    });
    assert_eq!(count, 1);
    let result = terminal.unwrap();
    assert_eq!(
        result.status,
        Status::Succeeded,
        "{} kind={:?} format={} current={} verified={} read={} decoded={}",
        result.error,
        result.kind,
        result.format,
        result.current,
        result.verified_files,
        result.read_bytes,
        result.decoded_bytes
    );
    result
}
fn check(archive: &Archive, mixed: bool) -> Snapshot {
    let job = archive.check_job(&Default::default()).unwrap();
    let result = task(mixed, |control, publish| {
        checking::run(job, 3, control, publish)
    });
    assert_eq!(
        (
            result.written_bytes,
            result.committed_bytes,
            result.committed_files
        ),
        (0, 0, 0)
    );
    result
}
fn known_contents(archive: &Archive) -> BTreeMap<String, Vec<u8>> {
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }
    }
    let control = Cancellation::default();
    let mut source = archive.source.fork().unwrap();
    let mut reader = crate::formats::codec(archive.format)
        .open_payload_with_password(
            &mut source,
            &archive.rows,
            &control,
            archive.password.as_ref().map(extraction::Password::expose),
        )
        .unwrap();
    let mut result = BTreeMap::new();
    for row in archive.rows.iter().filter(|row| !row.directory) {
        let mut sink = Sink(Vec::new());
        reader
            .decode_with_password(
                row,
                &mut sink,
                &control,
                archive.password.as_ref().map(extraction::Password::expose),
            )
            .unwrap();
        result.insert(
            String::from_utf8(row.name_bytes().to_vec()).unwrap(),
            sink.0,
        );
    }
    result
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/native-archives")
        .join(name)
}
#[test]
#[ignore = "requires built isolated codec worker"]
fn seven_zip_create_open_check_extract_header_password_and_split() {
    for (method, solid, encrypted, headers, volume) in [
        (0, false, false, false, 0),
        (32, false, false, false, 0),
        (32, true, false, false, 0),
        (32, true, true, false, 0),
        (32, true, true, true, 0),
        (32, true, true, true, 64),
    ] {
        let f = Fixture::new();
        let created = f.create("archive.7z", method, solid, encrypted, headers, volume);
        let path = if volume == 0 {
            f.0.join("output/archive.7z")
        } else {
            assert!(created.output_parts > 1);
            f.0.join("output/archive.7z.002")
        };
        let archive = if headers {
            assert!(matches!(
                Archive::open(&path, &Default::default()),
                Err(ScanError::PasswordRequired)
            ));
            assert!(matches!(
                Archive::open_with_password(
                    &path,
                    &Default::default(),
                    Some(extraction::Password::new("wrong".into()).unwrap())
                ),
                Err(ScanError::PasswordOrIntegrity)
            ));
            Archive::open_with_password(
                &path,
                &Default::default(),
                Some(extraction::Password::new("password".into()).unwrap()),
            )
            .unwrap()
        } else {
            Archive::open(&path, &Default::default()).unwrap()
        };
        assert_eq!(archive.format(), Format::SevenZip);
        assert_eq!(archive.can_update(), !encrypted && volume == 0);
        assert_eq!(check(&archive, false).verified_files, 2);
        assert_eq!(f.extract(&archive, "decoded", false).committed_files, 2);
        assert_eq!(
            std::fs::read(f.0.join("decoded/input/first.txt")).unwrap(),
            b"first known bytes"
        );
        assert_eq!(
            std::fs::read(f.0.join("decoded/input/second.txt")).unwrap(),
            "Unicode 内容".repeat(500).as_bytes()
        );
        assert!(f.0.join("decoded/input/empty").is_dir());
    }
}
#[test]
#[ignore = "requires built isolated codec worker"]
fn seven_zip_solid_add_replace_move_and_cancel_preserve_transactions() {
    let f = Fixture::new();
    f.create("archive.7z", 32, true, false, false, 0);
    let path = f.0.join("output/archive.7z");
    let original = std::fs::read(&path).unwrap();
    std::fs::write(f.0.join("new.txt"), b"new file").unwrap();
    let archive = Archive::open(&path, &Default::default()).unwrap();
    let options = updating::Options {
        inputs: vec![f.0.join("new.txt").to_string_lossy().into()],
        directory: String::new(),
        conflict: ConflictPolicy::Overwrite,
    };
    let cancelled = Control::default();
    cancelled.cancel();
    let mut result = None;
    updating::run(
        archive
            .update_job(options.clone(), &Default::default())
            .unwrap(),
        4,
        &cancelled,
        &mut |s| {
            if s.status.terminal() {
                result = Some(s);
            }
        },
    );
    assert_eq!(result.unwrap().status, Status::Cancelled);
    assert_eq!(std::fs::read(&path).unwrap(), original);
    task(false, |control, publish| {
        updating::run(
            archive
                .update_job(options.clone(), &Default::default())
                .unwrap(),
            5,
            control,
            publish,
        )
    });
    std::fs::write(f.0.join("new.txt"), b"replacement bytes").unwrap();
    let archive = Archive::open(&path, &Default::default()).unwrap();
    task(false, |control, publish| {
        updating::run(
            archive.update_job(options, &Default::default()).unwrap(),
            6,
            control,
            publish,
        )
    });
    let archive = Archive::open(&path, &Default::default()).unwrap();
    let selected = archive
        .rows
        .iter()
        .find(|r| r.name_bytes() == b"new.txt")
        .unwrap()
        .id
        .get()
        .to_string();
    let target = archive
        .rows
        .iter()
        .find(|r| r.name_bytes() == b"input/")
        .unwrap()
        .id
        .get()
        .to_string();
    let options = updating::MoveOptions {
        selected: vec![selected],
        directory: String::new(),
        target,
        conflict: ConflictPolicy::Ask,
    };
    task(false, |control, publish| {
        updating::run(
            archive.move_job(options, &Default::default()).unwrap(),
            7,
            control,
            publish,
        )
    });
    let archive = Archive::open(&path, &Default::default()).unwrap();
    let contents = known_contents(&archive);
    assert_eq!(contents["input/new.txt"], b"replacement bytes");
    assert_eq!(contents["input/first.txt"], b"first known bytes");
    assert!(!contents.contains_key("new.txt"));
    check(&archive, false);
}
#[test]
#[ignore = "requires built isolated codec worker"]
fn rar4_rar5_encrypted_headers_and_mixed_content_are_read_only() {
    for (name, headers, mixed) in [
        ("test_read_format_rar4_encrypted.rar", false, true),
        ("test_read_format_rar4_encrypted_filenames.rar", true, false),
        ("test_read_format_rar5_encrypted.rar", false, true),
        ("test_read_format_rar5_encrypted_filenames.rar", true, false),
        ("test_read_format_rar5_solid_encrypted.rar", false, false),
        (
            "test_read_format_rar5_solid_encrypted_filenames.rar",
            true,
            false,
        ),
    ] {
        let f = Fixture::new();
        let path = fixture(name);
        let archive = if headers {
            assert!(
                matches!(
                    Archive::open(&path, &Default::default()),
                    Err(ScanError::PasswordRequired)
                ),
                "{name}"
            );
            Archive::open_with_password(
                &path,
                &Default::default(),
                Some(extraction::Password::new("password".into()).unwrap()),
            )
            .unwrap()
        } else {
            Archive::open(&path, &Default::default()).unwrap()
        };
        assert_eq!(archive.format(), Format::Rar);
        assert!(!archive.can_update());
        assert_eq!(check(&archive, mixed).verified_files, 4, "{name}");
        assert_eq!(
            f.extract(&archive, "decoded", mixed).committed_files,
            4,
            "{name}"
        );
        for name in ["a.txt", "b.txt", "c.txt", "d.txt"] {
            assert_eq!(
                std::fs::read(f.0.join("decoded").join(name)).unwrap(),
                format!("This is from {name}").as_bytes()
            );
        }
    }
}
#[test]
#[ignore = "requires built isolated codec worker"]
fn rar5_solid_volumes_open_from_every_part_and_check_without_output() {
    for index in 1..=4 {
        let path = fixture(&format!(
            "test_read_format_rar5_multiarchive_solid.part{index:02}.rar"
        ));
        let archive = Archive::open(&path, &Default::default()).unwrap();
        assert_eq!(archive.format(), Format::Rar);
        assert!(archive.source.is_split());
        assert!(!archive.can_update());
        let result = check(&archive, false);
        assert!(result.verified_files > 0 && result.verified_bytes > 0);
        assert!(result.decoded_bytes >= result.verified_bytes);
    }
}

#[test]
#[ignore = "requires built isolated codec worker and 514 MiB temporary output"]
fn seven_zip_streaming_creation_exceeds_former_output_cache() {
    let f = Fixture::new();
    let size = 513 * 1024 * 1024u64;
    let input = std::fs::File::create(f.0.join("input/large-zero.bin")).unwrap();
    input.set_len(size).unwrap();
    drop(input);
    let result = f.create("large.7z", 0, false, false, false, 0);
    assert!(result.written_bytes > 512 * 1024 * 1024);
    assert_eq!(result.verified_files, 3);
    let path = f.0.join("output/large.7z");
    assert!(std::fs::metadata(&path).unwrap().len() > size);
    let archive = Archive::open(&path, &Default::default()).unwrap();
    let result = check(&archive, false);
    assert_eq!(result.verified_files, 3);
    assert!(result.verified_bytes >= size);
}

#[test]
#[ignore = "requires built isolated codec worker"]
fn rar7_store_roundtrip_and_dictionary_admission_publish_no_failed_output() {
    let archive = Archive::open(&fixture("rar7-store-small.rar"), &Default::default()).unwrap();
    assert_eq!(archive.rows[0].method, 70);
    assert_eq!(
        known_contents(&archive)["sample.txt"],
        b"RAR newest algorithm verification sample"
    );
    check(&archive, false);
    let f = Fixture::new();
    let target = f.0.join("limited-output");
    std::fs::create_dir(&target).unwrap();
    match Archive::open(&fixture("rar7-dictionary-limit.rar"), &Default::default()) {
        Err(ScanError::Limit) => (),
        Ok(archive) => {
            let job = archive
                .extraction_job(
                    extraction::Options {
                        target: target.to_string_lossy().into(),
                        all: true,
                        selected: vec![],
                        directory: String::new(),
                        create_folder: false,
                        conflict: ConflictPolicy::Ask,
                        allow_high_ratio: true,
                    },
                    &Default::default(),
                )
                .unwrap();
            let mut result = None;
            extraction::run(job, 8, &Default::default(), &mut |snapshot| {
                if snapshot.status.terminal() {
                    result = Some(snapshot);
                }
            });
            let result = result.unwrap();
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.error, "resource_limit");
            assert_eq!((result.committed_bytes, result.committed_files), (0, 0));
        }
        Err(error) => panic!("unexpected RAR7 admission result: {error}"),
    }
    assert_eq!(std::fs::read_dir(target).unwrap().count(), 0);
}

fn readonly_fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/cab-iso")
        .join(name)
}
fn readonly_options(archive: &Archive, target: &Path) -> extraction::Options {
    extraction::Options {
        target: target.to_string_lossy().into(),
        all: false,
        selected: archive
            .rows
            .iter()
            .filter(|entry| !entry.link && !entry.warning)
            .map(|entry| entry.id.get().to_string())
            .collect(),
        directory: String::new(),
        create_folder: false,
        conflict: ConflictPolicy::Ask,
        allow_high_ratio: true,
    }
}
fn assert_readonly(archive: &Archive, f: &Fixture) {
    assert!(!archive.can_update());
    assert_ne!(archive.edit_restriction & 4, 0);
    assert!(matches!(
        archive.update_job(
            updating::Options {
                inputs: vec![f.0.join("input/first.txt").to_string_lossy().into()],
                directory: String::new(),
                conflict: ConflictPolicy::Ask
            },
            &Default::default()
        ),
        Err("unsupported_feature")
    ));
}
fn readonly_transfer(archive: &Archive, f: &Fixture, label: &str, name: &str, bytes: &[u8]) {
    let target = f.0.join(format!("extract-{label}"));
    std::fs::create_dir(&target).unwrap();
    let options = readonly_options(archive, &target);
    let job = archive
        .extraction_job(options, &Default::default())
        .unwrap();
    task(false, |control, publish| {
        extraction::run(job, 2, control, publish)
    });
    assert_eq!(std::fs::read(target.join(name)).unwrap(), bytes);
    let cache = f.0.join(format!("cache-{label}"));
    std::fs::create_dir(&cache).unwrap();
    let entry = archive
        .rows
        .iter()
        .find(|entry| entry.name_bytes() == name.as_bytes())
        .unwrap();
    let job = archive
        .open_entry_job(entry.id.get(), &cache, &Default::default())
        .unwrap();
    let result = task(false, |control, publish| {
        extraction::run(job, 3, control, publish)
    });
    assert_eq!(std::fs::read(&result.open_path).unwrap(), bytes);
    let options = readonly_options(archive, &cache);
    let manifest = archive
        .export_manifest(&options, &Default::default())
        .unwrap();
    assert!(manifest.iter().any(|entry| entry.name == name));
    let job = archive.export_job(options, &Default::default()).unwrap();
    let result = task(false, |control, publish| {
        extraction::run(job, 4, control, publish)
    });
    assert_eq!(
        std::fs::read(Path::new(&result.target).join(name)).unwrap(),
        bytes
    );
}

#[test]
#[ignore = "requires built isolated codec worker"]
fn cabinet_store_mszip_lzx_quantum_tasks_and_trailing_signature_are_read_only() {
    let f = Fixture::new();
    for (name, count) in [
        ("named.cab", 3),
        ("normal_2files_1folder.cab", 2),
        ("mszip_lzx_qtm.cab", 3),
    ] {
        let archive = Archive::open(&readonly_fixture(name), &Default::default()).unwrap();
        assert_eq!(archive.format(), Format::Cab);
        assert_eq!(archive.len(), count);
        assert_readonly(&archive, &f);
        let contents = known_contents(&archive);
        assert_eq!(check(&archive, false).verified_files, count as u64);
        if name == "named.cab" {
            assert_eq!(contents["hello.txt"], b"xdzip read-only archive fixture\n");
            assert_eq!(
                contents["nested/中文.txt"],
                "Unicode payload: 中文\n".as_bytes()
            );
            assert_eq!(contents["empty.txt"], b"");
            readonly_transfer(&archive, &f, "cab", "hello.txt", &contents["hello.txt"]);
            let mut signed = std::fs::read(readonly_fixture(name)).unwrap();
            signed.extend_from_slice(b"post-cabinet signature data");
            let path = f.0.join("signed.data");
            std::fs::write(&path, signed).unwrap();
            let signed = Archive::open(&path, &Default::default()).unwrap();
            assert_eq!(known_contents(&signed), contents);
        }
    }
    for name in ["new.cab", "new.iso", "new.udf"] {
        assert_eq!(crate::formats::creation_format(name), None);
    }
}

#[test]
#[ignore = "requires built isolated codec worker"]
fn cabinet_checksum_failure_publishes_no_selected_output() {
    let f = Fixture::new();
    let mut bytes = std::fs::read(readonly_fixture("named.cab")).unwrap();
    let block = u32::from_le_bytes(bytes[36..40].try_into().unwrap()) as usize;
    bytes[block..block + 4].copy_from_slice(&0xdead_beefu32.to_le_bytes());
    let path = f.0.join("bad-checksum.cab");
    std::fs::write(&path, bytes).unwrap();
    let archive = Archive::open(&path, &Default::default()).unwrap();
    let hello = archive
        .rows
        .iter()
        .find(|entry| entry.name_bytes() == b"hello.txt")
        .unwrap();
    let target = f.0.join("bad-output");
    std::fs::create_dir(&target).unwrap();
    let mut options = readonly_options(&archive, &target);
    options.selected = vec![hello.id.get().to_string()];
    let job = archive
        .extraction_job(options, &Default::default())
        .unwrap();
    let mut terminal = None;
    extraction::run(job, 9, &Default::default(), &mut |snapshot| {
        if snapshot.status.terminal() {
            terminal = Some(snapshot);
        }
    });
    let terminal = terminal.unwrap();
    assert_eq!(terminal.status, Status::Failed);
    assert_eq!(terminal.error, "integrity_error");
    assert_eq!((terminal.committed_files, terminal.committed_bytes), (0, 0));
    assert_eq!(std::fs::read_dir(target).unwrap().count(), 0);
}

#[test]
#[ignore = "requires built isolated codec worker"]
fn cabinet_arbitrary_volume_names_open_from_every_part_and_detect_later_changes() {
    let f = Fixture::new();
    let root = f.0.join("volumes");
    std::fs::create_dir(&root).unwrap();
    for part in 1..=5 {
        let original = format!("cabd_multi_basic_pt{part}.cab");
        let mut bytes = std::fs::read(readonly_fixture(&original)).unwrap();
        for index in 1..=5 {
            let before = format!("cabd_multi_basic_pt{index}.cab");
            let after = format!("piece_{index:014}.bin");
            assert_eq!(before.len(), after.len());
            for at in 0..=bytes.len() - before.len() {
                if bytes[at..at + before.len()] == *before.as_bytes() {
                    bytes[at..at + before.len()].copy_from_slice(after.as_bytes());
                }
            }
        }
        std::fs::write(root.join(format!("piece_{part:014}.bin")), bytes).unwrap();
    }
    let data = (1..=5)
        .map(|index| format!("This is the data from cabinet part {index}.\n"))
        .collect::<String>()
        .into_bytes();
    let expected = BTreeMap::from([
        ("test1.txt".into(), data[..76].to_vec()),
        ("test2.txt".into(), data[76..114].to_vec()),
        ("test3.txt".into(), data[114..].to_vec()),
    ]);
    for part in 1..=5 {
        let archive = Archive::open(
            &root.join(format!("piece_{part:014}.bin")),
            &Default::default(),
        )
        .unwrap();
        assert_eq!(archive.format(), Format::Cab);
        assert!(archive.source.is_split());
        assert_eq!(known_contents(&archive), expected);
        assert_readonly(&archive, &f);
        if part == 3 {
            check(&archive, false);
            readonly_transfer(&archive, &f, "cab-split", "test3.txt", &data[114..]);
        }
    }
    let archive =
        Archive::open(&root.join("piece_00000000000003.bin"), &Default::default()).unwrap();
    let final_part = root.join("piece_00000000000005.bin");
    std::fs::remove_file(&final_part).unwrap();
    assert_eq!(archive.source.validate(), Err(ScanError::SourceChanged));
    assert!(matches!(
        Archive::open(&root.join("piece_00000000000003.bin"), &Default::default()),
        Err(ScanError::MissingVolume)
    ));
}

#[test]
#[ignore = "requires built isolated codec worker"]
fn optical_images_preserve_names_readable_content_and_reject_link_output_and_udf_fallback() {
    use std::io::Read;
    let f = Fixture::new();
    for (name, format) in [
        ("iso9660", Format::Iso),
        ("joliet", Format::Iso),
        ("rock-ridge", Format::Iso),
        ("udf", Format::Udf),
        ("udf-regular", Format::Udf),
        ("hybrid", Format::Udf),
    ] {
        let mut bytes = Vec::new();
        flate2::read::GzDecoder::new(
            std::fs::File::open(readonly_fixture(&format!("{name}.iso.gz"))).unwrap(),
        )
        .read_to_end(&mut bytes)
        .unwrap();
        let path = f.0.join(format!("{name}.wrong-extension"));
        std::fs::write(&path, &bytes).unwrap();
        let archive = Archive::open(&path, &Default::default()).unwrap();
        assert_eq!(archive.format(), format);
        assert_readonly(&archive, &f);
        let hello = if name == "iso9660" {
            "HELLO.TXT"
        } else {
            "hello.txt"
        };
        let nested = if name == "iso9660" {
            "NESTED/DATA.TXT"
        } else {
            "nested/data.txt"
        };
        assert!(
            archive
                .rows
                .iter()
                .any(|entry| entry.name_bytes() == nested.as_bytes())
        );
        if name != "iso9660" {
            assert!(
                archive
                    .rows
                    .iter()
                    .any(|entry| entry.name_bytes() == "中文.txt".as_bytes())
            );
        }
        if format == Format::Udf {
            assert!(
                archive
                    .rows
                    .iter()
                    .any(|entry| entry.name_bytes() == b"udf-only.txt")
            );
        }
        let links: Vec<_> = archive.rows.iter().filter(|entry| entry.link).collect();
        if links.is_empty() {
            check(&archive, false);
        } else {
            let mut result = None;
            checking::run(
                archive.check_job(&Default::default()).unwrap(),
                7,
                &Default::default(),
                &mut |snapshot| {
                    if snapshot.status.terminal() {
                        result = Some(snapshot);
                    }
                },
            );
            let result = result.unwrap();
            assert_eq!(result.status, Status::Failed);
            assert_eq!(result.error, "unsupported_feature");
            for link in links {
                assert!(matches!(
                    archive.open_entry_job(link.id.get(), &f.0, &Default::default()),
                    Err("unsupported_feature")
                ));
            }
        }
        readonly_transfer(
            &archive,
            &f,
            name,
            hello,
            b"xdzip read-only archive fixture\n",
        );
        assert_eq!(
            std::fs::read(f.0.join(format!("extract-{name}")).join(nested)).unwrap(),
            "Unicode payload: 中文\n".as_bytes()
        );
        if name == "hybrid" {
            // Destroy UDF anchors/descriptors while preserving the complete ISO tree.
            for sector in bytes.chunks_mut(2048).skip(256) {
                sector.fill(0);
            }
            std::fs::write(&path, bytes).unwrap();
            assert_eq!(
                crate::formats::detect(&mut std::io::Cursor::new(std::fs::read(&path).unwrap())),
                Ok(Format::Udf)
            );
            assert!(Archive::open(&path, &Default::default()).is_err());
        }
    }
}

#[test]
#[ignore = "requires built isolated codec worker"]
fn iso_extent_above_four_gib_reads_host_sparse_source_without_large_allocations() {
    use std::io::{self, Read, Seek, SeekFrom};
    struct Sparse {
        image: Vec<u8>,
        payload: Vec<u8>,
        extent: u64,
        length: u64,
        position: u64,
        read_bytes: u64,
        high_read: u64,
    }
    impl Read for Sparse {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let count = output
                .len()
                .min(self.length.saturating_sub(self.position) as usize);
            let output = &mut output[..count];
            output.fill(0);
            let end = self.position + count as u64;
            for (base, bytes) in [
                (0, self.image.as_slice()),
                (self.extent, self.payload.as_slice()),
            ] {
                let start = self.position.max(base);
                let stop = end.min(base + bytes.len() as u64);
                if stop > start {
                    output[(start - self.position) as usize..(stop - self.position) as usize]
                        .copy_from_slice(&bytes[(start - base) as usize..(stop - base) as usize]);
                }
            }
            self.read_bytes += count as u64;
            self.high_read = self.high_read.max(end);
            self.position = end;
            Ok(count)
        }
    }
    impl Seek for Sparse {
        fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
            let position = match from {
                SeekFrom::Start(at) => i128::from(at),
                SeekFrom::Current(at) => i128::from(self.position) + i128::from(at),
                SeekFrom::End(at) => i128::from(self.length) + i128::from(at),
            };
            self.position = u64::try_from(position).map_err(|_| io::ErrorKind::InvalidInput)?;
            Ok(self.position)
        }
    }
    impl xdzip_codec_api::ReadSource for Sparse {}
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }
    }
    fn both32(image: &mut [u8], at: usize, value: u32) {
        image[at..at + 4].copy_from_slice(&value.to_le_bytes());
        image[at + 4..at + 8].copy_from_slice(&value.to_be_bytes());
    }
    let mut image = Vec::new();
    flate2::read::GzDecoder::new(std::fs::File::open(readonly_fixture("iso9660.iso.gz")).unwrap())
        .read_to_end(&mut image)
        .unwrap();
    let descriptor = 16 * 2048;
    let root = u32::from_le_bytes(
        image[descriptor + 158..descriptor + 162]
            .try_into()
            .unwrap(),
    ) as usize
        * 2048;
    let lba = 0x20_0008u32;
    both32(&mut image, descriptor + 80, lba + 1);
    let mut at = root;
    let mut changed = false;
    while image[at] != 0 {
        let size = image[at] as usize;
        let name_len = image[at + 32] as usize;
        if &image[at + 33..at + 33 + name_len] == b"HELLO.TXT;1" {
            both32(&mut image, at + 2, lba);
            changed = true;
        }
        at += size;
    }
    assert!(changed);
    let expected = b"xdzip read-only archive fixture\n";
    let mut source = Sparse {
        image,
        payload: expected.to_vec(),
        extent: u64::from(lba) * 2048,
        length: u64::from(lba + 1) * 2048,
        position: 0,
        read_bytes: 0,
        high_read: 0,
    };
    let control = Cancellation::default();
    let codec = crate::formats::codec(Format::Iso);
    let rows = codec
        .scan(&mut source, &control, Default::default())
        .unwrap();
    let hello = rows
        .iter()
        .find(|entry| entry.name_bytes() == b"HELLO.TXT")
        .unwrap();
    let mut sink = Sink(Vec::new());
    codec
        .open_payload(&mut source, &rows, &control)
        .unwrap()
        .decode(hello, &mut sink, &control)
        .unwrap();
    assert_eq!(sink.0, expected);
    assert!(source.high_read > 4 * 1024 * 1024 * 1024);
    assert!(source.read_bytes < 2 * 1024 * 1024);
}
