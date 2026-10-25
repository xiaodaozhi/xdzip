//! Opt-in real Rust-CAB-worker tasks. These test the host chain, not manual GUI use.
//! Use an owner-only TEMP/TMP as in the existing native-runtime test environment.
use crate::extraction::{self, ConflictPolicy, Control, Snapshot, Status, TaskKind};
use crate::{Archive, service};
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use xdzip_codec_api::{Cancellation, Format};

const BLOCK: u32 = 32768;
const PREFIX: u32 = 2 * BLOCK - 13;
const SELECTED: u32 = BLOCK + 270;
const SMALL_TOTAL: u32 = 4 * BLOCK;
const LARGE: u32 = 128 * 1024 * 1024;
const DEADLINE: Duration = Duration::from_secs(45);
const JOIN_DEADLINE: Duration = Duration::from_secs(10);
const OLD: &[u8] = b"old destination must survive";
static NEXT: AtomicU64 = AtomicU64::new(1);

struct Fixture {
    temporary: PathBuf,
    root: PathBuf,
    source: PathBuf,
    target: PathBuf,
    large: bool,
}

fn private_directory(path: &Path) {
    std::fs::create_dir(path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
}

impl Fixture {
    fn new(large: bool, damaged_tail: bool) -> Self {
        assert_eq!(
            std::env::var("XDZIP_CAB_BACKEND").as_deref(),
            Ok("rust"),
            "opt in only with the newly built fixed Rust CAB worker"
        );
        let worker = std::env::var_os("XDZIP_NATIVE_CODEC")
            .expect("set XDZIP_NATIVE_CODEC to the newly built Rust CAB worker");
        assert!(Path::new(&worker).is_file(), "built worker is missing");
        let temporary = std::fs::canonicalize(std::env::temp_dir()).unwrap();
        let root = temporary.join(format!(
            "xdzip-cab-host-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        private_directory(&root);
        let target = root.join("target");
        private_directory(&target);
        let source = root.join("owned.cab");
        write_store_cab(&source, large, damaged_tail);
        Self {
            temporary,
            root,
            source,
            target,
            large,
        }
    }

    fn leaf(&self) -> &'static str {
        if self.large {
            "large.bin"
        } else {
            "selected.bin"
        }
    }

    fn selected_bytes(&self) -> u64 {
        u64::from(if self.large { LARGE } else { SELECTED })
    }

    fn open(&self) -> Archive {
        let archive = bounded(|control| Archive::open(&self.source, &control.cancel)).unwrap();
        assert_eq!(archive.format(), Format::Cab);
        archive
    }

    fn options(&self, archive: &Archive, names: &[&str]) -> extraction::Options {
        extraction::Options {
            target: self.target.to_string_lossy().into(),
            all: false,
            selected: names
                .iter()
                .map(|name| {
                    archive
                        .rows
                        .iter()
                        .find(|row| row.name_bytes() == name.as_bytes())
                        .expect("owned CAB record")
                        .id
                        .get()
                        .to_string()
                })
                .collect(),
            directory: String::new(),
            create_folder: false,
            conflict: ConflictPolicy::Overwrite,
            allow_high_ratio: false,
        }
    }

    fn change_source(&self) {
        // A length change is deterministic even when the filesystem's mtime is coarse.
        std::fs::OpenOptions::new()
            .append(true)
            .open(&self.source)
            .unwrap()
            .write_all(b"changed")
            .unwrap();
    }

    fn old_target(&self) {
        std::fs::write(self.target.join(self.leaf()), OLD).unwrap();
    }

    fn assert_old_and_clean(&self) {
        assert_eq!(std::fs::read(self.target.join(self.leaf())).unwrap(), OLD);
        assert_eq!(std::fs::read_dir(&self.target).unwrap().count(), 1);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        // Only this freshly created child of the verified temporary root is owned.
        if self.root.parent() == Some(self.temporary.as_path())
            && self
                .root
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("xdzip-cab-host-"))
        {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}

fn parity(bytes: &[u8]) -> u32 {
    let (words, tail) = bytes.as_chunks::<4>();
    let full = words
        .iter()
        .fold(0, |sum, word| sum ^ u32::from_le_bytes(*word));
    full ^ tail
        .iter()
        .fold(0u32, |tail, byte| (tail << 8) | u32::from(*byte))
}

fn write_store_cab(path: &Path, large: bool, damaged_tail: bool) {
    let total = if large { LARGE } else { SMALL_TOTAL };
    let records = if large {
        vec![("large.bin", 0, LARGE)]
    } else {
        vec![
            ("prefix.bin", 0, PREFIX),
            ("selected.bin", PREFIX, SELECTED),
        ]
    };
    write_store_records(path, &records, total, large, damaged_tail);
}

fn write_store_records(
    path: &Path,
    records: &[(&str, u32, u32)],
    total: u32,
    large: bool,
    damaged_tail: bool,
) {
    let until = Instant::now() + DEADLINE;
    let mut files = Vec::new();
    for (name, offset, size) in records {
        files.extend_from_slice(&size.to_le_bytes());
        files.extend_from_slice(&offset.to_le_bytes());
        files.extend_from_slice(&0u16.to_le_bytes()); // One Store folder.
        files.extend_from_slice(&0x5821u16.to_le_bytes());
        files.extend_from_slice(&0u16.to_le_bytes());
        files.extend_from_slice(&0x20u16.to_le_bytes());
        files.extend_from_slice(name.as_bytes());
        files.push(0);
    }
    let data_at = 44 + files.len() as u32;
    let blocks = total.div_ceil(BLOCK);
    let cabinet_bytes = data_at + total + blocks * 8;
    let mut header = b"MSCF".to_vec();
    for field in [0, cabinet_bytes, 0, 44, 0] {
        header.extend_from_slice(&field.to_le_bytes());
    }
    header.extend_from_slice(&[3, 1]);
    for field in [1, records.len() as u16, 0, 0x5844, 0] {
        header.extend_from_slice(&field.to_le_bytes());
    }
    assert_eq!(header.len(), 36);
    header.extend_from_slice(&data_at.to_le_bytes());
    header.extend_from_slice(&(blocks as u16).to_le_bytes());
    header.extend_from_slice(&0u16.to_le_bytes());
    let mut output = BufWriter::new(std::fs::File::create(path).unwrap());
    output.write_all(&header).unwrap();
    output.write_all(&files).unwrap();
    let mut bytes = [0u8; BLOCK as usize];
    for frame in 0..blocks {
        assert!(Instant::now() < until, "bounded owned CAB generation");
        let start = frame * BLOCK;
        let count = (total - start).min(BLOCK) as usize;
        for (offset, byte) in bytes[..count].iter_mut().enumerate() {
            let position = start + offset as u32;
            *byte = if large {
                b'Q'
            } else if position < PREFIX {
                b'A'
            } else if position < PREFIX + SELECTED {
                b'B'
            } else {
                b'C'
            };
        }
        let mut sizes = [0u8; 4];
        sizes[..2].copy_from_slice(&(count as u16).to_le_bytes());
        sizes[2..].copy_from_slice(&(count as u16).to_le_bytes());
        let checksum = parity(&sizes) ^ parity(&bytes[..count]);
        assert_ne!(checksum, 0, "owned frame actually declares a checksum");
        output.write_all(&checksum.to_le_bytes()).unwrap();
        output.write_all(&sizes).unwrap();
        if damaged_tail && frame + 1 == blocks {
            // Damage only the unselected remainder of the selected file's last
            // complete frame, keeping its original nonzero checksum.
            bytes[count - 1] ^= 0x80;
        }
        output.write_all(&bytes[..count]).unwrap();
    }
    output.flush().unwrap();
    assert!(Instant::now() < until, "owned CAB generation deadline");
    assert_eq!(
        output.get_ref().metadata().unwrap().len(),
        u64::from(cabinet_bytes)
    );
}

/// Cancellation also bounds scanner/worker waits. Even a panicking assertion
/// wakes and joins the watchdog rather than leaving a sleeper behind.
fn bounded<T>(run: impl FnOnce(&Control) -> T) -> T {
    let control = Control::default();
    let (stop, wait) = mpsc::sync_channel(1);
    std::thread::scope(|scope| {
        let deadline_control = &control;
        let watchdog = scope.spawn(move || {
            if wait.recv_timeout(DEADLINE).is_err() {
                deadline_control.cancel();
                true
            } else {
                false
            }
        });
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&control)));
        let _ = stop.send(());
        let timed_out = watchdog.join().unwrap();
        match result {
            Ok(value) => {
                assert!(
                    !timed_out,
                    "CAB operation exceeded its cancellation deadline"
                );
                value
            }
            Err(panic) => std::panic::resume_unwind(panic),
        }
    })
}

fn task(
    run: impl FnOnce(&Control, &mut dyn FnMut(Snapshot)),
    mut hook: impl FnMut(&Snapshot, &Control),
) -> Snapshot {
    bounded(|control| {
        let mut done = None;
        let mut count = 0;
        run(control, &mut |snapshot| {
            hook(&snapshot, control);
            if snapshot.status.terminal() {
                count += 1;
                done = Some(snapshot);
            }
        });
        assert_eq!(count, 1, "exactly one terminal CAB task state");
        let done = done.unwrap();
        assert!(
            !control.cancel(),
            "terminal outcome rejects late cancellation"
        );
        done
    })
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

struct Service {
    client: service::Client,
    events: mpsc::Receiver<service::Event>,
    listener: Option<JoinHandle<()>>,
}

impl Service {
    fn new() -> Self {
        let (client, mut receiver) = service::start().unwrap();
        let (send, events) = mpsc::channel();
        let listener = std::thread::spawn(move || {
            while let Some(event) = receiver.receive() {
                if send.send(event).is_err() {
                    break;
                }
            }
        });
        Self {
            client,
            events,
            listener: Some(listener),
        }
    }

    fn submit(&self, command: impl Fn() -> service::Command) -> u64 {
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            match self.client.submit(command()) {
                Ok(id) => return id,
                Err(service::Rejection::Busy) if Instant::now() < until => std::thread::yield_now(),
                _ => panic!("CAB service admission failed"),
            }
        }
    }

    fn event(&self, until: Instant) -> service::Event {
        self.events
            .recv_timeout(until.saturating_duration_since(Instant::now()))
            .expect("CAB service event deadline")
    }

    fn check(&self, path: &Path) -> Snapshot {
        let until = Instant::now() + DEADLINE;
        let open = self.submit(|| service::Command::Open(path.to_string_lossy().into()));
        let mut session = 0;
        loop {
            let event = self.event(until);
            match event.payload {
                service::Payload::Page {
                    session: id,
                    format,
                    ..
                } => {
                    assert_eq!(format, Format::Cab);
                    session = id;
                }
                service::Payload::Finished(error) if event.request == open => {
                    assert!(error.is_empty(), "CAB service open: {error}");
                    break;
                }
                _ => {}
            }
        }
        assert!(session > 0);
        let request = self.submit(|| service::Command::Check { session });
        let until = Instant::now() + DEADLINE;
        let mut accepted = 0;
        let mut finished = false;
        let mut done = None;
        while done.is_none() || accepted == 0 || !finished {
            let event = self.event(until);
            match event.payload {
                service::Payload::TaskAccepted(id) if event.request == request => accepted = id,
                service::Payload::Finished(error) if event.request == request => {
                    assert!(error.is_empty(), "CAB check admission: {error}");
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
        done
    }

    fn close(&mut self) -> bool {
        if self.listener.is_none() {
            return true;
        }
        self.client.shutdown();
        let until = Instant::now() + JOIN_DEADLINE;
        while let Ok(event) = self
            .events
            .recv_timeout(until.saturating_duration_since(Instant::now()))
        {
            if matches!(event.payload, service::Payload::ShutdownReady) {
                // ShutdownReady is emitted only after both real service workers
                // have joined. The forwarding listener then receives EOF.
                return self.listener.take().unwrap().join().is_ok();
            }
        }
        false
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

fn stage_has_payload(target: &Path, expected: u8) -> bool {
    let until = Instant::now() + Duration::from_secs(2);
    for directory in std::fs::read_dir(target).unwrap().take(4).flatten() {
        assert!(Instant::now() < until, "stage inspection deadline");
        if !directory
            .file_name()
            .to_string_lossy()
            .starts_with(".xdzip-work-")
        {
            continue;
        }
        for entry in std::fs::read_dir(directory.path())
            .unwrap()
            .take(8)
            .flatten()
        {
            assert!(Instant::now() < until, "stage inspection deadline");
            if entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "part")
            {
                let mut byte = [0];
                if let Ok(mut file) = std::fs::File::open(entry.path())
                    && file.read(&mut byte).ok() == Some(1)
                {
                    assert_eq!(byte, [expected]);
                    return true;
                }
            }
        }
    }
    false
}

#[test]
#[ignore = "requires the newly built fixed Rust CAB worker"]
fn cab_service_open_check_validates_all_records_and_frame_tail_without_output() {
    for damaged in [false, true] {
        let fixture = Fixture::new(false, damaged);
        let before = std::fs::read(&fixture.source).unwrap();
        let mut service = Service::new();
        let done = service.check(&fixture.source);
        assert!(service.close(), "bounded joined CAB service shutdown");
        assert_eq!(done.format, Format::Cab.label());
        assert_eq!(done.files, 2);
        assert!(done.read_bytes >= u64::from(SMALL_TOTAL));
        no_output(&done);
        if damaged {
            assert_eq!(done.status, Status::Failed);
            assert_eq!(done.error, "integrity_error");
            assert_eq!(
                (done.verified_files, done.verified_bytes),
                (1, u64::from(PREFIX))
            );
        } else {
            assert_eq!(done.status, Status::Succeeded, "{}", done.error);
            assert_eq!(
                (done.verified_files, done.verified_bytes),
                (2, u64::from(PREFIX + SELECTED))
            );
            assert_eq!(done.decoded_bytes, u64::from(SMALL_TOTAL));
        }
        assert_eq!(std::fs::read(&fixture.source).unwrap(), before);
        assert_eq!(std::fs::read_dir(&fixture.target).unwrap().count(), 0);
        assert_eq!(std::fs::read_dir(&fixture.root).unwrap().count(), 2);
    }
}

#[test]
#[ignore = "requires the newly built fixed Rust CAB worker"]
fn cab_same_folder_selection_charges_history_and_whole_selected_frame_tail() {
    for names in [&["selected.bin"][..], &["prefix.bin", "selected.bin"][..]] {
        let fixture = Fixture::new(false, false);
        let archive = fixture.open();
        let job = archive
            .extraction_job(fixture.options(&archive, names), &Cancellation::default())
            .unwrap();
        let done = task(
            |control, publish| extraction::run(job, 21, control, publish),
            |_, _| {},
        );
        assert_eq!(done.status, Status::Succeeded, "{}", done.error);
        let selected = u64::from(SELECTED)
            + if names.len() == 2 {
                u64::from(PREFIX)
            } else {
                0
            };
        assert_eq!(
            (done.written_bytes, done.committed_bytes),
            (selected, selected)
        );
        assert_eq!(
            (done.files, done.committed_files),
            (names.len() as u64, names.len() as u64)
        );
        // Prefix history and the selected file's unread final-frame remainder
        // consume the actual-work meter, while only selected bytes are written.
        assert_eq!(done.decoded_bytes, u64::from(SMALL_TOTAL));
        assert!(done.decoded_bytes > selected && done.decoded_bytes < extraction::MAX_TASK_BYTES);
        assert_eq!(
            std::fs::read(fixture.target.join("selected.bin")).unwrap(),
            vec![b'B'; SELECTED as usize]
        );
        if names.len() == 2 {
            assert_eq!(
                std::fs::read(fixture.target.join("prefix.bin")).unwrap(),
                vec![b'A'; PREFIX as usize]
            );
        }
        assert_eq!(
            std::fs::read_dir(&fixture.target).unwrap().count(),
            names.len()
        );
        assert!(!done.cleanup_warning && !done.outcome_uncertain);
    }
    let fixture = Fixture::new(false, true);
    let archive = fixture.open();
    fixture.old_target();
    let job = archive
        .extraction_job(
            fixture.options(&archive, &[fixture.leaf()]),
            &Cancellation::default(),
        )
        .unwrap();
    let done = task(
        |control, publish| extraction::run(job, 22, control, publish),
        |_, _| {},
    );
    assert_eq!(done.status, Status::Failed);
    assert_eq!(done.error, "integrity_error");
    assert!(done.written_bytes > 0 && done.written_bytes < u64::from(SELECTED));
    assert_eq!((done.committed_files, done.committed_bytes), (0, 0));
    assert!(!done.cleanup_warning && !done.outcome_uncertain);
    fixture.assert_old_and_clean();

    // An earlier independent selection need not traverse unrelated later frames.
    let job = archive
        .extraction_job(
            fixture.options(&archive, &["prefix.bin"]),
            &Cancellation::default(),
        )
        .unwrap();
    let done = task(
        |control, publish| extraction::run(job, 23, control, publish),
        |_, _| {},
    );
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(done.decoded_bytes, u64::from(2 * BLOCK));
    assert_eq!(
        std::fs::read(fixture.target.join("prefix.bin")).unwrap(),
        vec![b'A'; PREFIX as usize]
    );
}

#[test]
#[ignore = "requires the newly built fixed Rust CAB worker and precharged protocol"]
fn cab_one_frame_with_128_files_charges_work_once_in_check_and_extraction() {
    let fixture = Fixture::new(false, false);
    let names: Vec<_> = (0..128)
        .map(|index| format!("file-{index:03}.bin"))
        .collect();
    let records: Vec<_> = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index as u32 * 256, 256))
        .collect();
    write_store_records(&fixture.source, &records, BLOCK, false, false);
    let mut service = Service::new();
    let checked = service.check(&fixture.source);
    assert!(service.close(), "bounded joined CAB service shutdown");
    assert_eq!(checked.status, Status::Succeeded, "{}", checked.error);
    assert_eq!(
        (
            checked.verified_files,
            checked.verified_bytes,
            checked.decoded_bytes
        ),
        (128, u64::from(BLOCK), u64::from(BLOCK))
    );
    no_output(&checked);
    let archive = fixture.open();
    let selected: Vec<_> = names.iter().map(String::as_str).collect();
    let job = archive
        .extraction_job(
            fixture.options(&archive, &selected),
            &Cancellation::default(),
        )
        .unwrap();
    let done = task(
        |control, publish| extraction::run(job, 27, control, publish),
        |_, _| {},
    );
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(
        (done.decoded_bytes, done.written_bytes, done.committed_bytes),
        (u64::from(BLOCK), u64::from(BLOCK), u64::from(BLOCK))
    );
    assert_eq!(done.committed_files, 128);
    assert_eq!(std::fs::read_dir(&fixture.target).unwrap().count(), 128);
    for name in names {
        assert_eq!(
            std::fs::read(fixture.target.join(name)).unwrap(),
            vec![b'A'; 256]
        );
    }
    assert!(!done.cleanup_warning && !done.outcome_uncertain);
}

#[test]
#[ignore = "requires the newly built fixed Rust CAB worker"]
fn cab_frozen_and_running_source_changes_fail_without_publishing_staged_bytes() {
    for running in [false, true] {
        let fixture = Fixture::new(running, false);
        let archive = fixture.open();
        fixture.old_target();
        let job = archive
            .extraction_job(
                fixture.options(&archive, &[fixture.leaf()]),
                &Cancellation::default(),
            )
            .unwrap();
        if !running {
            fixture.change_source();
        }
        let mut changed = !running;
        let done = task(
            |control, publish| extraction::run(job, 24, control, publish),
            |snapshot, _| {
                if running
                    && !changed
                    && snapshot.status == Status::Running
                    && snapshot.written_bytes > 0
                    && snapshot.written_bytes < fixture.selected_bytes()
                {
                    assert!(stage_has_payload(&fixture.target, b'Q'));
                    fixture.change_source();
                    changed = true;
                }
            },
        );
        assert!(changed, "source changed after actual partial staging");
        assert_eq!(done.status, Status::Failed);
        assert_eq!(done.error, "source_changed");
        if running {
            assert!(done.written_bytes > 0);
        } else {
            assert_eq!(
                (done.read_bytes, done.decoded_bytes, done.written_bytes),
                (0, 0, 0)
            );
        }
        assert_eq!((done.committed_files, done.committed_bytes), (0, 0));
        assert!(!done.cleanup_warning && !done.outcome_uncertain);
        fixture.assert_old_and_clean();
    }
}

#[test]
#[ignore = "requires the newly built fixed Rust CAB worker"]
fn cab_committing_source_change_keeps_old_destination_after_complete_decode() {
    let fixture = Fixture::new(false, false);
    let archive = fixture.open();
    fixture.old_target();
    let job = archive
        .extraction_job(
            fixture.options(&archive, &[fixture.leaf()]),
            &Cancellation::default(),
        )
        .unwrap();
    let mut changed = false;
    let done = task(
        |control, publish| extraction::run(job, 25, control, publish),
        |snapshot, _| {
            if !changed && snapshot.status == Status::Committing {
                assert_eq!(snapshot.written_bytes, u64::from(SELECTED));
                assert_eq!(snapshot.decoded_bytes, u64::from(SMALL_TOTAL));
                assert_eq!(snapshot.committed_files, 0);
                assert!(stage_has_payload(&fixture.target, b'B'));
                fixture.change_source();
                changed = true;
            }
        },
    );
    assert!(
        changed,
        "mutation occurs at the synchronous host publication boundary"
    );
    assert_eq!(done.status, Status::Failed);
    assert_eq!(done.error, "source_changed");
    assert_eq!((done.committed_files, done.committed_bytes), (0, 0));
    assert!(!done.cleanup_warning && !done.outcome_uncertain);
    fixture.assert_old_and_clean();
}

#[test]
#[ignore = "requires the newly built fixed Rust CAB worker"]
fn cab_running_stage_cancellation_preserves_old_file_and_joins_cleanup() {
    let fixture = Fixture::new(true, false);
    let archive = fixture.open();
    fixture.old_target();
    let job = archive
        .extraction_job(
            fixture.options(&archive, &[fixture.leaf()]),
            &Cancellation::default(),
        )
        .unwrap();
    let mut observed = None;
    let done = task(
        |control, publish| extraction::run(job, 26, control, publish),
        |snapshot, control| {
            if observed.is_none()
                && snapshot.status == Status::Running
                && snapshot.written_bytes > 0
                && snapshot.written_bytes < u64::from(LARGE)
            {
                // A fresh handle reads actual stage data; Windows directory-entry
                // length caching cannot substitute for observing a real write.
                assert!(stage_has_payload(&fixture.target, b'Q'));
                assert_eq!(snapshot.committed_files, 0);
                observed = Some(snapshot.written_bytes);
                assert!(control.cancel());
            }
        },
    );
    let observed = observed.expect("cancel only after a real Running partial write");
    assert!(observed > 0 && observed < u64::from(LARGE));
    assert_eq!(done.status, Status::Cancelled, "{}", done.error);
    assert!(done.written_bytes >= observed && done.written_bytes < u64::from(LARGE));
    assert!(done.decoded_bytes > 0);
    assert_eq!((done.committed_files, done.committed_bytes), (0, 0));
    assert!(!done.cleanup_warning && !done.outcome_uncertain);
    fixture.assert_old_and_clean();
    // Returning through run dropped/terminated the native worker and joined its
    // pipe reader; bounded() also joined the cancellation watchdog above.
}
