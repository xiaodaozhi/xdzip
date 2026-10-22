// crates/xdzip-runtime/src/checking/tests.rs

use super::*;
use crate::extraction::Password;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use xdzip_codec_api::encode::{
    CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeSettings,
};

static NEXT: AtomicU64 = AtomicU64::new(1);
struct Input {
    entries: Vec<EncodeEntry>,
    bytes: Vec<u8>,
}
impl EncodeInput for Input {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(io::Cursor::new(&self.bytes)))
    }
}
struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-check-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn archive(&self, bytes: &[u8]) -> Archive {
        let path = self.0.join("input.bin");
        std::fs::write(&path, bytes).unwrap();
        Archive::open(&path, &Cancellation::default()).unwrap()
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn execute(archive: &Archive, control: &Control, mut hook: impl FnMut(&Snapshot)) -> Snapshot {
    let job = archive.check_job(&Cancellation::default()).unwrap();
    let mut terminal = None;
    let mut terminals = 0;
    run(job, 1, control, &mut |s| {
        hook(&s);
        assert_eq!(s.kind, TaskKind::Check);
        assert!(!s.creating && s.target.is_empty() && s.open_path.is_empty());
        assert_eq!(
            (
                s.written_bytes,
                s.committed_bytes,
                s.committed_files,
                s.created_directories
            ),
            (0, 0, 0, 0)
        );
        if s.status.terminal() {
            terminals += 1;
            terminal = Some(s);
        }
    });
    assert_eq!(terminals, 1);
    assert!(!control.respond_password(1, Password::new(String::new()).unwrap()));
    terminal.unwrap()
}
fn fixture(name: &str) -> Archive {
    Archive::open(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(name),
        &Cancellation::default(),
    )
    .unwrap()
}
#[test]
fn whole_zip_checks_empty_duplicate_directory_descriptor_and_link_records_without_output() {
    for name in [
        "extract/stored.zip",
        "extract/deflate.zip",
        "extract/descriptor64.zip",
        "zip/empty.zip",
        "zip/normal.zip",
        "extract/aliases.zip",
        "extract/unix_bundle.zip",
    ] {
        let archive = fixture(name);
        let done = execute(&archive, &Control::default(), |_| {});
        assert_eq!(done.status, Status::Succeeded, "{name}: {}", done.error);
        assert_eq!(
            done.verified_files,
            archive.rows.iter().filter(|r| !r.directory).count() as u64
        );
        assert_eq!(
            done.verified_bytes,
            archive
                .rows
                .iter()
                .filter(|r| !r.directory)
                .map(|r| r.size)
                .sum()
        );
        assert!(done.read_bytes > 0);
    }
}
#[test]
fn corrupt_payloads_stop_at_first_failure_and_never_count_failed_files() {
    for name in [
        "crc_bad.zip",
        "partial_crc.zip",
        "header_bad.zip",
        "descriptor_bad.zip",
        "descriptor64_bad.zip",
        "no_end.zip",
        "trailing.zip",
        "link_crc.zip",
    ] {
        let archive = fixture(&format!("extract/{name}"));
        let done = execute(&archive, &Control::default(), |_| {});
        assert_eq!(done.status, Status::Failed, "{name}");
        assert_eq!(done.error, "integrity_error", "{name}");
        assert!(done.verified_files < done.files);
        assert!(!done.details.is_empty());
    }
}
#[test]
fn mixed_passwords_retry_with_fresh_tokens_and_authenticate_empty_link_payloads() {
    let temp = Temp::new();
    let bytes = crate::extraction::password_tests::archive_bytes(
        &[
            ("plain", b"data", None, false),
            ("secret", b"private", Some("one"), false),
            ("link", b"secret", Some("two"), true),
            ("empty", b"", Some(""), false),
        ],
        8,
    );
    let archive = temp.archive(&bytes);
    let control = Control::default();
    let mut tokens = Vec::new();
    let done = execute(&archive, &control, |s| {
        if let Some(prompt) = &s.password_prompt {
            assert!(!tokens.contains(&prompt.id));
            tokens.push(prompt.id);
            let password = match prompt.name.as_str() {
                "secret" if !prompt.retry => "incorrect",
                "secret" => "one",
                "link" => "two",
                "empty" => "",
                _ => panic!("unexpected prompt"),
            };
            assert!(control.respond_password(prompt.id, Password::new(password.into()).unwrap()));
            assert!(!control.respond_password(prompt.id, Password::new(password.into()).unwrap()));
        }
    });
    assert_eq!(done.status, Status::Succeeded);
    assert_eq!(done.verified_files, 4);
    assert!(tokens.len() >= 4);
    assert_eq!(std::fs::read(temp.0.join("input.bin")).unwrap(), bytes);
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 1);
}
#[test]
fn password_wait_cancellation_and_retry_exhaustion_are_terminal() {
    let temp = Temp::new();
    let bytes = crate::extraction::password_tests::archive_bytes(
        &[("secret", b"private", Some("one"), false)],
        0,
    );
    let archive = temp.archive(&bytes);
    let control = Control::default();
    let done = execute(&archive, &control, |s| {
        if s.password_prompt.is_some() {
            control.cancel();
        }
    });
    assert_eq!(done.status, Status::Cancelled);
    assert_eq!(done.verified_files, 0);
    let control = Control::default();
    let mut prompts = 0;
    let done = execute(&archive, &control, |s| {
        if let Some(prompt) = &s.password_prompt {
            prompts += 1;
            assert!(
                control.respond_password(prompt.id, Password::new("incorrect".into()).unwrap())
            );
        }
    });
    assert_eq!(done.error, "wrong_password");
    assert_eq!(prompts, 10);
    assert_eq!(done.verified_files, 0);
}

#[test]
fn aes_authentication_damage_is_not_retried_as_a_password_error() {
    let temp = Temp::new();
    let mut bytes = crate::extraction::password_tests::archive_bytes(
        &[("secret", b"private", Some("one"), false)],
        0,
    );
    let central = bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
    let packed = u32::from_le_bytes(bytes[central + 20..central + 24].try_into().unwrap()) as usize;
    let name = u16::from_le_bytes(bytes[26..28].try_into().unwrap()) as usize;
    let extra = u16::from_le_bytes(bytes[28..30].try_into().unwrap()) as usize;
    bytes[30 + name + extra + packed - 1] ^= 1;
    let archive = temp.archive(&bytes);
    let control = Control::default();
    let mut prompts = 0;
    let done = execute(&archive, &control, |s| {
        if let Some(prompt) = &s.password_prompt {
            prompts += 1;
            assert!(control.respond_password(prompt.id, Password::new("one".into()).unwrap()));
        }
    });
    assert_eq!(prompts, 1);
    assert_eq!(done.status, Status::Failed);
    assert_eq!(done.error, "integrity_error");
    assert_eq!(done.verified_files, 0);
}
#[test]
fn tar_and_gzip_check_one_complete_stream_without_writing() {
    let input = Input {
        entries: vec![EncodeEntry {
            name: "folder/data".into(),
            directory: false,
            size: 5,
            modified_unix: 0,
        }],
        bytes: b"hello".to_vec(),
    };
    for gzip in [false, true] {
        let temp = Temp::new();
        let mut bytes = Vec::new();
        xdzip_codec_tar::TarCodec { gzip }
            .encode(
                &input,
                &mut bytes,
                EncodeSettings {
                    method: if gzip { "deflate" } else { "store" },
                    level: 6,
                    password: None,
                },
                &Cancellation::default(),
                &mut |_| {},
            )
            .unwrap();
        let archive = temp.archive(&bytes);
        let done = execute(&archive, &Control::default(), |_| {});
        assert_eq!(done.status, Status::Succeeded);
        assert_eq!(done.verified_files, 1);
        assert_eq!(done.verified_bytes, 5);
        assert_eq!(done.read_bytes, bytes.len() as u64);
        assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 1);
        assert_eq!(std::fs::read(temp.0.join("input.bin")).unwrap(), bytes);
    }
}
#[test]
fn queued_source_changes_cancellation_and_declared_quotas_cannot_pass() {
    let temp = Temp::new();
    let bytes = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/extract/stored.zip"),
    )
    .unwrap();
    let archive = temp.archive(&bytes);
    let job = archive.check_job(&Cancellation::default()).unwrap();
    let mut changed = bytes.clone();
    changed.push(0);
    std::fs::write(temp.0.join("input.bin"), changed).unwrap();
    let mut done = None;
    run(job, 7, &Control::default(), &mut |s| {
        if s.status.terminal() {
            done = Some(s);
        }
    });
    assert_eq!(done.unwrap().error, "source_changed");
    let mut archive = temp.archive(&bytes);
    Arc::make_mut(&mut archive.rows)[0].size = u64::MAX;
    let done = execute(&archive, &Control::default(), |_| {});
    assert_eq!(done.error, "file_quota");
    assert_eq!(done.read_bytes, 0);
    let control = Control::default();
    control.cancel();
    let done = execute(&archive, &control, |_| {});
    assert_eq!(done.status, Status::Cancelled);
    assert_eq!(done.read_bytes, 0);
}

#[test]
fn service_check_keeps_password_and_task_identity_after_source_session_closes() {
    use crate::service::{self, Command, Payload, Rejection};
    use std::sync::mpsc;
    use std::time::Duration;
    let temp = Temp::new();
    let bytes = crate::extraction::password_tests::archive_bytes(
        &[("secret", b"private", Some("one"), false)],
        0,
    );
    std::fs::write(temp.0.join("input.zip"), bytes).unwrap();
    let (client, mut receiver) = service::start().unwrap();
    let (tx, rx) = mpsc::channel();
    let listener = std::thread::spawn(move || {
        while let Some(event) = receiver.receive() {
            if tx.send(event).is_err() {
                break;
            }
        }
    });
    fn submit(client: &service::Client, mut command: impl FnMut() -> Command) -> u64 {
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            match client.submit(command()) {
                Ok(id) => break id,
                Err(Rejection::Busy) if Instant::now() < until => std::thread::yield_now(),
                other => panic!("{other:?}"),
            }
        }
    }
    let input = temp.0.join("input.zip").to_string_lossy().into_owned();
    let open = submit(&client, || Command::Open(input.clone()));
    let mut session = 0;
    loop {
        let event = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        if let Payload::Page { session: id, .. } = event.payload {
            session = id;
        }
        if event.request == open && matches!(event.payload, Payload::Finished("")) {
            break;
        }
    }
    assert!(session > 0);
    let request = submit(&client, || Command::Check { session });
    let (mut task, mut prompt) = (0, 0);
    let mut accepted = false;
    while task == 0 || prompt == 0 || !accepted {
        let event = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        match event.payload {
            Payload::TaskAccepted(id) if event.request == request => {
                task = id;
                accepted = true;
            }
            Payload::Task(s) => {
                if let Some(p) = s.password_prompt {
                    task = s.id;
                    prompt = p.id;
                }
            }
            _ => {}
        }
    }
    let close = submit(&client, || Command::CloseSession(session));
    loop {
        let event = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        if event.request == close && matches!(event.payload, Payload::Finished("")) {
            break;
        }
    }
    assert!(
        client
            .respond_password(task, prompt + 1, Password::new("one".into()).unwrap())
            .is_err()
    );
    assert!(
        client
            .respond_password(task, prompt, Password::new("one".into()).unwrap())
            .is_ok()
    );
    loop {
        let event = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        if let Payload::Task(s) = event.payload
            && s.status.terminal()
        {
            assert_eq!(s.id, task);
            assert_eq!(s.kind, TaskKind::Check);
            assert_eq!(s.session, session);
            assert_eq!(s.status, Status::Succeeded);
            assert_eq!(s.verified_files, 1);
            break;
        }
    }
    client.shutdown();
    loop {
        if matches!(
            rx.recv_timeout(Duration::from_secs(5)).unwrap().payload,
            Payload::ShutdownReady
        ) {
            break;
        }
    }
    listener.join().unwrap();
    assert_eq!(std::fs::read_dir(&temp.0).unwrap().count(), 1);
}
