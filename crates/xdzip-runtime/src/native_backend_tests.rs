// crates/xdzip-runtime/src/native_backend_tests.rs

//! Real official-engine tasks. Opt in after building the locked native worker.
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
#[ignore = "requires built official native codec worker"]
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
                Err(ScanError::WrongPassword)
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
#[ignore = "requires built official native codec worker"]
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
#[ignore = "requires built official native codec worker"]
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
#[ignore = "requires built official native codec worker"]
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
#[ignore = "requires built official native codec worker and 514 MiB temporary output"]
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
#[ignore = "requires built official native codec worker"]
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
