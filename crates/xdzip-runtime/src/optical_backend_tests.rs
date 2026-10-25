//! Opt-in real Rust-optical-worker tasks. These are host/worker tests, not GUI acceptance.
use crate::extraction::{self, ConflictPolicy, Control, Snapshot, Status, TaskKind};
use crate::{Archive, checking, service};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};
use xdzip_codec_api::{Cancellation, Format};

const BLOCK: usize = 2048;
const LARGE_BYTES: u32 = 256 * 1024 * 1024;
const HELLO: &[u8] = b"xdzip read-only archive fixture\n";
const OLD: &[u8] = b"old destination must survive";
static NEXT: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    root: PathBuf,
    source: PathBuf,
    target: PathBuf,
    leaf: &'static str,
}

impl Fixture {
    fn new(format: Format, large: bool, change_payload: bool) -> Self {
        let worker = std::env::var_os("XDZIP_NATIVE_CODEC")
            .expect("set XDZIP_NATIVE_CODEC to the newly built Rust optical worker");
        assert!(Path::new(&worker).is_file(), "built worker is missing");
        let root = std::env::temp_dir().join(format!(
            "xdzip-optical-host-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let target = root.join("target");
        std::fs::create_dir(&target).unwrap();
        let source = root.join("owned.data");
        let (name, leaf) = match format {
            Format::Iso => ("iso9660.iso.gz", "HELLO.TXT"),
            Format::Udf => ("udf-regular.iso.gz", "hello.txt"),
            _ => unreachable!(),
        };
        let input = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/cab-iso")
            .join(name);
        let mut bytes = Vec::new();
        flate2::read::GzDecoder::new(std::fs::File::open(input).unwrap())
            .read_to_end(&mut bytes)
            .unwrap();
        assert!(bytes.len() < 1024 * 1024);
        if change_payload {
            let at = bytes
                .windows(HELLO.len())
                .position(|part| part == HELLO)
                .unwrap();
            bytes[at] ^= 1;
        }
        let mut expanded_length = None;
        if large {
            match format {
                Format::Iso => {
                    let name = b"HELLO.TXT;1";
                    let at = bytes
                        .windows(name.len())
                        .position(|part| part == name)
                        .unwrap()
                        - 33;
                    assert!(usize::from(bytes[at]) >= 33 + name.len());
                    let extent = u32::from_le_bytes(bytes[at + 2..at + 6].try_into().unwrap());
                    bytes[at + 10..at + 14].copy_from_slice(&LARGE_BYTES.to_le_bytes());
                    bytes[at + 14..at + 18].copy_from_slice(&LARGE_BYTES.to_be_bytes());
                    let end = u64::from(extent) * BLOCK as u64 + u64::from(LARGE_BYTES);
                    let blocks = u32::try_from(end.div_ceil(BLOCK as u64)).unwrap();
                    bytes[16 * BLOCK + 80..16 * BLOCK + 84].copy_from_slice(&blocks.to_le_bytes());
                    bytes[16 * BLOCK + 84..16 * BLOCK + 88].copy_from_slice(&blocks.to_be_bytes());
                    expanded_length = Some(end);
                }
                Format::Udf => {
                    // ECMA-167 File Entry, short_ad type 2: logical zero bytes need
                    // no recorded source allocation. Recompute its descriptor CRC/tag.
                    let at = (0..bytes.len())
                        .step_by(BLOCK)
                        .find(|&at| {
                            u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap()) == 261
                                && bytes[at + 27] == 5
                                && u64::from_le_bytes(bytes[at + 56..at + 64].try_into().unwrap())
                                    == HELLO.len() as u64
                                && u32::from_le_bytes(bytes[at + 168..at + 172].try_into().unwrap())
                                    == 0
                                && u32::from_le_bytes(bytes[at + 172..at + 176].try_into().unwrap())
                                    == 8
                        })
                        .expect("owned UDF hello File Entry");
                    bytes[at + 56..at + 64].copy_from_slice(&u64::from(LARGE_BYTES).to_le_bytes());
                    bytes[at + 64..at + 72].fill(0);
                    bytes[at + 176..at + 180]
                        .copy_from_slice(&((2u32 << 30) | LARGE_BYTES).to_le_bytes());
                    bytes[at + 180..at + 184].fill(0);
                    let length =
                        u16::from_le_bytes(bytes[at + 10..at + 12].try_into().unwrap()) as usize;
                    let crc = crc16(&bytes[at + 16..at + 16 + length]);
                    bytes[at + 8..at + 10].copy_from_slice(&crc.to_le_bytes());
                    bytes[at + 4] = 0;
                    bytes[at + 4] = bytes[at..at + 16]
                        .iter()
                        .fold(0u8, |sum, byte| sum.wrapping_add(*byte));
                }
                _ => unreachable!(),
            }
        }
        std::fs::write(&source, bytes).unwrap();
        if let Some(length) = expanded_length {
            // Only the ISO test's local source grows logically. No large image is
            // retained in the repository and cancellation stops its output early.
            std::fs::OpenOptions::new()
                .write(true)
                .open(&source)
                .unwrap()
                .set_len(length)
                .unwrap();
        }
        Self {
            root,
            source,
            target,
            leaf,
        }
    }

    fn open(&self, format: Format) -> Archive {
        let archive = Archive::open(&self.source, &Cancellation::default()).unwrap();
        assert_eq!(archive.format(), format);
        archive
    }

    fn options(&self, archive: &Archive) -> extraction::Options {
        let selected = archive
            .rows
            .iter()
            .find(|row| row.name_bytes() == self.leaf.as_bytes())
            .unwrap();
        extraction::Options {
            target: self.target.to_string_lossy().into(),
            all: false,
            selected: vec![selected.id.get().to_string()],
            directory: String::new(),
            create_folder: false,
            conflict: ConflictPolicy::Overwrite,
            allow_high_ratio: false,
        }
    }

    fn change_source(&self) {
        // Length changes are deterministic even on filesystems with coarse mtimes.
        std::fs::OpenOptions::new()
            .append(true)
            .open(&self.source)
            .unwrap()
            .write_all(b"changed")
            .unwrap();
    }

    fn assert_old_and_clean(&self) {
        assert_eq!(std::fs::read(self.target.join(self.leaf)).unwrap(), OLD);
        assert_eq!(std::fs::read_dir(&self.target).unwrap().count(), 1);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn crc16(bytes: &[u8]) -> u16 {
    let mut crc = 0u16;
    for byte in bytes {
        crc ^= u16::from(*byte) << 8;
        for _ in 0..8 {
            crc = (crc << 1) ^ if crc & 0x8000 != 0 { 0x1021 } else { 0 };
        }
    }
    crc
}

fn no_output(snapshot: &Snapshot) {
    assert_eq!(snapshot.kind, TaskKind::Check);
    assert!(snapshot.target.is_empty() && snapshot.open_path.is_empty());
    assert_eq!(
        (
            snapshot.written_bytes,
            snapshot.committed_bytes,
            snapshot.committed_files,
            snapshot.created_directories
        ),
        (0, 0, 0, 0)
    );
    assert!(!snapshot.cleanup_warning && !snapshot.outcome_uncertain);
}

fn terminal(
    run: impl FnOnce(&mut dyn FnMut(Snapshot)),
    mut hook: impl FnMut(&Snapshot),
) -> Snapshot {
    let mut done = None;
    let mut count = 0;
    run(&mut |snapshot| {
        hook(&snapshot);
        if snapshot.status.terminal() {
            count += 1;
            done = Some(snapshot);
        }
    });
    assert_eq!(count, 1, "exactly one terminal task state");
    done.unwrap()
}

fn submit(client: &service::Client, command: impl Fn() -> service::Command) -> u64 {
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        match client.submit(command()) {
            Ok(id) => return id,
            Err(service::Rejection::Busy) if Instant::now() < until => std::thread::yield_now(),
            _ => panic!("service command admission failed"),
        }
    }
}

fn service_check(path: &Path, format: Format) -> Snapshot {
    let (client, mut receiver) = service::start().unwrap();
    let (send, receive) = mpsc::channel();
    let listener = std::thread::spawn(move || {
        while let Some(event) = receiver.receive() {
            if send.send(event).is_err() {
                break;
            }
        }
    });
    let open = submit(&client, || {
        service::Command::Open(path.to_string_lossy().into())
    });
    let mut session = 0;
    loop {
        let event = receive
            .recv_timeout(Duration::from_secs(20))
            .expect("bounded optical open");
        match event.payload {
            service::Payload::Page {
                session: id,
                format: actual,
                ..
            } => {
                session = id;
                assert_eq!(actual, format);
            }
            service::Payload::Finished(error) if event.request == open => {
                assert!(error.is_empty(), "optical open: {error}");
                break;
            }
            _ => {}
        }
    }
    assert!(session > 0);
    let request = submit(&client, || service::Command::Check { session });
    let mut accepted = 0;
    let mut finished = false;
    let mut done = None;
    while done.is_none() || accepted == 0 || !finished {
        let event = receive
            .recv_timeout(Duration::from_secs(20))
            .expect("bounded optical check");
        match event.payload {
            service::Payload::TaskAccepted(id) if event.request == request => accepted = id,
            service::Payload::Finished(error) if event.request == request => {
                assert!(error.is_empty(), "check admission: {error}");
                finished = true;
            }
            service::Payload::Task(snapshot) if event.request == request => {
                no_output(&snapshot);
                assert_eq!(snapshot.session, session);
                if snapshot.status.terminal() {
                    assert!(done.is_none());
                    done = Some(*snapshot);
                }
            }
            _ => {}
        }
    }
    let done = done.unwrap();
    assert_eq!(done.id, accepted);
    client.shutdown();
    loop {
        if matches!(
            receive
                .recv_timeout(Duration::from_secs(20))
                .expect("joined service shutdown")
                .payload,
            service::Payload::ShutdownReady
        ) {
            break;
        }
    }
    listener.join().unwrap();
    done
}

#[test]
#[ignore = "requires built Rust optical worker"]
fn optical_service_checks_readability_without_output_or_payload_authenticity_claim() {
    for format in [Format::Iso, Format::Udf] {
        for change_payload in [false, true] {
            let fixture = Fixture::new(format, false, change_payload);
            let before = std::fs::read(&fixture.source).unwrap();
            let done = service_check(&fixture.source, format);
            assert_eq!(done.status, Status::Succeeded, "{}", done.error);
            assert_eq!(done.format, format.label());
            assert_eq!(
                done.verified_files,
                if format == Format::Iso { 3 } else { 5 }
            );
            assert_eq!(
                done.verified_bytes,
                if format == Format::Iso { 56 } else { 112 }
            );
            assert_eq!(done.decoded_bytes, done.verified_bytes);
            assert!(done.read_bytes > 0);
            no_output(&done);
            assert_eq!(std::fs::read(&fixture.source).unwrap(), before);
            assert_eq!(std::fs::read_dir(&fixture.target).unwrap().count(), 0);
            assert_eq!(std::fs::read_dir(&fixture.root).unwrap().count(), 2);
            // The changed payload was present before opening. Neither ISO9660
            // nor these UDF ordinary files contain an authenticated content hash.
        }
    }
}

#[test]
#[ignore = "requires built Rust optical worker"]
fn optical_frozen_and_running_checks_reject_real_source_changes() {
    for format in [Format::Iso, Format::Udf] {
        for while_running in [false, true] {
            let fixture = Fixture::new(format, false, false);
            let archive = fixture.open(format);
            let job = archive.check_job(&Cancellation::default()).unwrap();
            if !while_running {
                fixture.change_source();
            }
            let mut changed = !while_running;
            let control = Control::default();
            let done = terminal(
                |publish| checking::run(job, 1, &control, publish),
                |snapshot| {
                    no_output(snapshot);
                    if while_running && !changed && snapshot.status == Status::Running {
                        fixture.change_source();
                        changed = true;
                    }
                },
            );
            assert!(changed);
            assert_eq!(done.status, Status::Failed);
            assert_eq!(done.error, "source_changed");
            if !while_running {
                assert_eq!(
                    done.read_bytes, 0,
                    "frozen changes fail before worker reads"
                );
            }
            no_output(&done);
            assert_eq!(std::fs::read_dir(&fixture.target).unwrap().count(), 0);
            assert!(!control.cancel(), "terminal task rejects late cancellation");
        }
    }
}

fn stage_has_payload(target: &Path) -> bool {
    let Ok(directories) = std::fs::read_dir(target) else {
        return false;
    };
    for directory in directories.flatten() {
        if !directory
            .file_name()
            .to_string_lossy()
            .starts_with(".xdzip-work-")
        {
            continue;
        }
        let Ok(entries) = std::fs::read_dir(directory.path()) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry
                .path()
                .extension()
                .is_some_and(|value| value == "part")
            {
                // Windows directory-entry lengths can remain cached until the
                // writer closes. Read actual staged data through a fresh handle.
                let mut byte = [0];
                if let Ok(mut file) = std::fs::File::open(entry.path())
                    && file.read(&mut byte).ok() == Some(1)
                {
                    return true;
                }
            }
        }
    }
    false
}

#[test]
#[ignore = "requires built Rust optical worker"]
fn optical_running_stream_cancellation_preserves_old_file_and_removes_private_stage() {
    for format in [Format::Iso, Format::Udf] {
        let fixture = Fixture::new(format, true, false);
        let archive = fixture.open(format);
        std::fs::write(fixture.target.join(fixture.leaf), OLD).unwrap();
        let job = archive
            .extraction_job(fixture.options(&archive), &Cancellation::default())
            .unwrap();
        let control = Control::default();
        let finished = AtomicBool::new(false);
        let observed_payload = AtomicBool::new(false);
        let (done, observed) = std::thread::scope(|scope| {
            let observer = scope.spawn(|| {
                let until = Instant::now() + Duration::from_secs(20);
                while !finished.load(Ordering::Acquire) && Instant::now() < until {
                    if stage_has_payload(&fixture.target) {
                        observed_payload.store(true, Ordering::Release);
                        return control.cancel();
                    }
                    std::thread::yield_now();
                }
                // A failed worker cannot strand the observer or an active task.
                control.cancel();
                false
            });
            let done = terminal(
                |publish| extraction::run(job, 2, &control, publish),
                |snapshot| {
                    if snapshot.status == Status::Running
                        && snapshot.written_bytes > 0
                        && snapshot.written_bytes < u64::from(LARGE_BYTES)
                    {
                        assert!(stage_has_payload(&fixture.target));
                        observed_payload.store(true, Ordering::Release);
                        control.cancel();
                    }
                },
            );
            finished.store(true, Ordering::Release);
            (done, observer.join().unwrap())
        });
        assert!(
            observed_payload.load(Ordering::Acquire),
            "observe actual staged payload before requesting cancellation"
        );
        assert!(observed || control.cancel.check().is_err());
        assert_eq!(done.status, Status::Cancelled, "{}", done.error);
        assert!(done.written_bytes > 0 && done.decoded_bytes > 0);
        assert!(done.written_bytes < u64::from(LARGE_BYTES));
        assert_eq!((done.committed_files, done.committed_bytes), (0, 0));
        assert!(!done.cleanup_warning && !done.outcome_uncertain);
        fixture.assert_old_and_clean();
        assert!(!control.cancel());
    }
}

#[test]
#[ignore = "requires built Rust optical worker"]
fn optical_source_change_at_publish_boundary_keeps_old_destination() {
    for format in [Format::Iso, Format::Udf] {
        let fixture = Fixture::new(format, false, false);
        let archive = fixture.open(format);
        std::fs::write(fixture.target.join(fixture.leaf), OLD).unwrap();
        let job = archive
            .extraction_job(fixture.options(&archive), &Cancellation::default())
            .unwrap();
        let control = Control::default();
        let mut changed = false;
        let done = terminal(
            |publish| extraction::run(job, 3, &control, publish),
            |snapshot| {
                if !changed && snapshot.status == Status::Committing {
                    assert_eq!(snapshot.written_bytes, HELLO.len() as u64);
                    assert_eq!(snapshot.committed_files, 0);
                    fixture.change_source();
                    changed = true;
                }
            },
        );
        assert!(
            changed,
            "source mutation occurs after the real worker decoded the file"
        );
        assert_eq!(done.status, Status::Failed);
        assert_eq!(done.error, "source_changed");
        assert_eq!((done.committed_files, done.committed_bytes), (0, 0));
        assert!(!done.cleanup_warning && !done.outcome_uncertain);
        fixture.assert_old_and_clean();
    }
}
