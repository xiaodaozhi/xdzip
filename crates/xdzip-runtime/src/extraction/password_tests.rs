//! Authenticated task regressions use only generated, owned temporary archives.
use super::tests::{Target, execute};
use super::*;
use std::io::{Cursor, Read};
use xdzip_codec_api::encode::{
    CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeSettings,
};
use xdzip_codec_zip::ZipCodec;

struct Input {
    entry: Vec<EncodeEntry>,
    data: Vec<u8>,
}
impl EncodeInput for Input {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entry
    }
    fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(Cursor::new(&self.data)))
    }
}
fn read32(bytes: &[u8]) -> usize {
    u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize
}
/// Assemble complete independently encoded single-record streams. Only central offsets
/// and explicitly requested Unix types are changed, never ciphertext or authentication.
fn archive_bytes(records: &[(&str, &[u8], Option<&str>, bool)], method: u16) -> Vec<u8> {
    let mut output = Vec::new();
    let mut directory = Vec::new();
    for (name, data, password, link) in records {
        let input = Input {
            entry: vec![EncodeEntry {
                name: (*name).into(),
                directory: false,
                size: data.len() as u64,
                modified_unix: 0,
            }],
            data: data.to_vec(),
        };
        let mut part = Vec::new();
        ZipCodec
            .encode(
                &input,
                &mut part,
                EncodeSettings {
                    method: if method == 0 { "store" } else { "deflate" },
                    level: 6,
                    password: *password,
                },
                &Cancellation::default(),
                &mut |_| {},
            )
            .unwrap();
        let end = part.len() - 22;
        assert_eq!(&part[end..end + 4], b"PK\x05\x06");
        let at = read32(&part[end + 16..]);
        let size = read32(&part[end + 12..]);
        let mut central = part[at..at + size].to_vec();
        assert_eq!(&central[..4], b"PK\x01\x02");
        central[42..46].copy_from_slice(&(output.len() as u32).to_le_bytes());
        central[5] = 3;
        let mode = if *link { 0o120777u32 } else { 0o100755u32 };
        central[38..42].copy_from_slice(&(mode << 16).to_le_bytes());
        output.extend_from_slice(&part[..at]);
        directory.extend_from_slice(&central);
    }
    let at = output.len() as u32;
    let size = directory.len() as u32;
    output.extend_from_slice(&directory);
    let mut end = [0; 22];
    end[..4].copy_from_slice(b"PK\x05\x06");
    end[8..10].copy_from_slice(&(records.len() as u16).to_le_bytes());
    end[10..12].copy_from_slice(&(records.len() as u16).to_le_bytes());
    end[12..16].copy_from_slice(&size.to_le_bytes());
    end[16..20].copy_from_slice(&at.to_le_bytes());
    output.extend_from_slice(&end);
    output
}
fn generated(source: &Target, bytes: &[u8]) -> Archive {
    let path = source.0.join("generated.zip");
    std::fs::write(&path, bytes).unwrap();
    Archive::open(&path, &Cancellation::default()).unwrap()
}
fn secret(value: &str) -> Password {
    Password::new(value.into()).unwrap()
}
fn verifier_rejects(archive: &Archive, index: usize, password: &str) -> bool {
    struct RejectOutput;
    impl xdzip_codec_api::payload::DecodedSink for RejectOutput {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), xdzip_codec_api::payload::DecodeError> {
            Err(xdzip_codec_api::payload::DecodeError::Output)
        }
    }
    let cancel = Cancellation::default();
    let mut source = archive.source.fork().unwrap();
    let mut decoder = crate::formats::codec(archive.format)
        .open_payload(source.file(), &archive.rows, &cancel)
        .unwrap();
    decoder.decode_with_password(
        &archive.rows[index],
        &mut RejectOutput,
        &cancel,
        Some(password),
    ) == Err(xdzip_codec_api::payload::DecodeError::WrongPassword)
}
fn wrong_password(archive: &Archive, index: usize) -> String {
    // AES's two-byte verifier can collide for a wrong password. Choose a candidate
    // known to fail that verifier, so random fixture salts cannot make this flaky.
    (0..1024)
        .map(|n| format!("incorrect-{n}"))
        .find(|password| verifier_rejects(archive, index, password))
        .expect("a rejected verifier candidate")
}

#[test]
fn extraction_password_policy_is_distinct_and_control_responses_are_single_use() {
    for value in ["", " \t\n\0密码 ", &"x".repeat(MAX_PASSWORD_BYTES)] {
        let password = secret(value);
        assert_eq!(password.expose(), value);
        assert_eq!(format!("{password:?}"), "[REDACTED]");
    }
    assert!(Password::new("x".repeat(MAX_PASSWORD_BYTES + 1)).is_err());
    let mut excessive_capacity = String::with_capacity(4 * MAX_PASSWORD_BYTES + 1);
    excessive_capacity.push_str("bounded contents");
    assert!(Password::new(excessive_capacity).is_err());
    let control = Control::default();
    assert!(!control.respond_password(1, secret("ignored")));
    control.arm(1);
    assert!(!control.respond_password(1, secret("ignored")));
    control.arm_password(2);
    assert!(!control.decide(2, ConflictPolicy::Overwrite, false));
    assert!(!control.respond_password(1, secret("ignored")));
    assert!(control.respond_password(2, secret("accepted")));
    assert!(!control.respond_password(2, secret("duplicate")));
    assert_eq!(control.wait_password().unwrap().expose(), "accepted");
    assert!(!control.respond_password(2, secret("expired")));
    control.arm_password(3);
    control.cancel();
    assert!(!control.respond_password(3, secret("cancelled")));
    assert_eq!(control.wait_password().unwrap_err(), "cancelled");
    control.mark_terminal();
    assert!(!control.respond_password(3, secret("terminal")));
}

#[test]
fn store_deflate_mixed_and_empty_files_authenticate_with_one_task_cache() {
    for method in [0, 8] {
        let source = Target::new();
        let target = Target::new();
        let a = generated(
            &source,
            &archive_bytes(
                &[
                    ("普通.txt", b"plain", None, false),
                    ("密码.txt", "完整内容".as_bytes(), Some(" 密码 "), false),
                    ("empty", b"", Some(" 密码 "), false),
                ],
                method,
            ),
        );
        let control = Control::default();
        let mut prompts = 0;
        let done = execute(&a, target.options(), &control, |snapshot| {
            assert!(!format!("{snapshot:?}").contains(" 密码 "));
            if let Some(prompt) = &snapshot.password_prompt {
                prompts += 1;
                assert!(!prompt.retry);
                assert!(control.respond_password(prompt.id, secret(" 密码 ")));
            }
        });
        assert_eq!(done.status, Status::Succeeded, "{}", done.error);
        assert_eq!(prompts, 1);
        assert_eq!(done.committed_files, 3);
        assert_eq!(done.decoded_bytes, done.declared_bytes);
        assert_eq!(
            std::fs::read(target.0.join("密码.txt")).unwrap(),
            "完整内容".as_bytes()
        );
        assert_eq!(std::fs::read(target.0.join("empty")).unwrap(), b"");
        assert!(done.password_prompt.is_none());
        assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 3);
    }
}

#[test]
fn wrong_password_retries_use_fresh_ids_and_never_replace_destination_before_authentication() {
    let source = Target::new();
    let target = Target::new();
    let a = generated(
        &source,
        &archive_bytes(&[("file", b"authenticated", Some("right"), false)], 8),
    );
    std::fs::write(target.0.join("file"), b"original").unwrap();
    let control = Control::default();
    let wrong = wrong_password(&a, 0);
    let mut ids = Vec::new();
    let mut passwords = 0;
    let done = execute(&a, target.options(), &control, |snapshot| {
        if let Some(prompt) = &snapshot.prompt {
            ids.push(prompt.id);
            assert!(control.decide(prompt.id, ConflictPolicy::Overwrite, false));
        }
        if let Some(prompt) = &snapshot.password_prompt {
            ids.push(prompt.id);
            assert_eq!(std::fs::read(target.0.join("file")).unwrap(), b"original");
            for entry in std::fs::read_dir(&target.0).unwrap().flatten() {
                if entry.file_type().unwrap().is_dir() {
                    assert!(
                        !std::fs::read_dir(entry.path())
                            .unwrap()
                            .flatten()
                            .any(|entry| entry
                                .path()
                                .extension()
                                .is_some_and(|extension| extension == "part"))
                    );
                }
            }
            assert_eq!(prompt.retry, passwords > 0);
            assert!(!control.respond_password(prompt.id + 1, secret("stale")));
            assert!(control.respond_password(
                prompt.id,
                secret(if passwords == 0 { &wrong } else { "right" })
            ));
            assert!(!control.respond_password(prompt.id, secret("duplicate")));
            passwords += 1;
        }
    });
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(ids, vec![1, 2, 3]);
    assert_eq!(passwords, 2);
    assert_eq!(
        std::fs::read(target.0.join("file")).unwrap(),
        b"authenticated"
    );
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 1);
}

#[test]
fn different_entry_passwords_reprompt_and_wrong_passwords_have_a_fixed_attempt_limit() {
    let source = Target::new();
    let target = Target::new();
    let a = (0..1024)
        .find_map(|_| {
            let a = generated(
                &source,
                &archive_bytes(
                    &[
                        ("one", b"one", Some("first"), false),
                        ("two", b"two", Some("second"), false),
                    ],
                    0,
                ),
            );
            verifier_rejects(&a, 1, "first").then_some(a)
        })
        .expect("a second entry which rejects the previous verifier");
    let control = Control::default();
    let mut prompts = Vec::new();
    let done = execute(&a, target.options(), &control, |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            prompts.push((prompt.name.clone(), prompt.retry));
            control.respond_password(
                prompt.id,
                secret(if prompt.name == "one" {
                    "first"
                } else {
                    "second"
                }),
            );
        }
    });
    assert_eq!(done.status, Status::Succeeded);
    assert_eq!(prompts, vec![("one".into(), false), ("two".into(), true)]);
    let target = Target::new();
    let control = Control::default();
    let wrong = wrong_password(&a, 0);
    let mut prompts = 0;
    let done = execute(&a, target.options(), &control, |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            prompts += 1;
            control.respond_password(prompt.id, secret(&wrong));
        }
    });
    assert_eq!(prompts, 10);
    assert_eq!(done.error, "wrong_password");
    assert_eq!(done.committed_files, 0);
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
}

#[test]
fn external_empty_and_control_character_passwords_are_not_rejected_as_creation_options() {
    for password in ["", "\0\t\n foreign password "] {
        let source = Target::new();
        let target = Target::new();
        let a = generated(
            &source,
            &archive_bytes(&[("file", b"external contents", Some(password), false)], 8),
        );
        let control = Control::default();
        let done = execute(&a, target.options(), &control, |snapshot| {
            if let Some(prompt) = &snapshot.password_prompt {
                assert!(!prompt.retry);
                control.respond_password(prompt.id, secret(password));
            }
        });
        assert_eq!(done.status, Status::Succeeded, "{}", done.error);
        assert_eq!(
            std::fs::read(target.0.join("file")).unwrap(),
            b"external contents"
        );
    }
}

#[test]
fn source_mutation_while_password_prompt_is_open_prevents_publication() {
    let source = Target::new();
    let target = Target::new();
    let a = generated(
        &source,
        &archive_bytes(&[("file", b"contents", Some("right"), false)], 8),
    );
    let control = Control::default();
    let done = execute(&a, target.options(), &control, |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            std::fs::write(source.0.join("generated.zip"), b"changed").unwrap();
            control.respond_password(prompt.id, secret("right"));
        }
    });
    assert_eq!(done.error, "source_changed");
    assert_eq!(done.committed_files, 0);
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
}

#[test]
fn authenticated_damage_is_not_retried_or_published_and_cancel_cleans_waiting_stage() {
    let source = Target::new();
    let target = Target::new();
    let mut bytes = archive_bytes(
        &[("file", b"private authenticated bytes", Some("right"), false)],
        0,
    );
    let central = read32(&bytes[bytes.len() - 6..]);
    let packed = read32(&bytes[central + 20..]);
    let name = u16::from_le_bytes(bytes[26..28].try_into().unwrap()) as usize;
    let extra = u16::from_le_bytes(bytes[28..30].try_into().unwrap()) as usize;
    bytes[30 + name + extra + packed - 1] ^= 1;
    let a = generated(&source, &bytes);
    std::fs::write(target.0.join("file"), b"old").unwrap();
    let mut options = target.options();
    options.conflict = ConflictPolicy::Overwrite;
    let control = Control::default();
    let mut prompts = 0;
    let done = execute(&a, options, &control, |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            prompts += 1;
            control.respond_password(prompt.id, secret("right"));
        }
    });
    assert_eq!(prompts, 1);
    assert_eq!(done.error, "integrity_error");
    assert_eq!(done.committed_files, 0);
    assert_eq!(std::fs::read(target.0.join("file")).unwrap(), b"old");
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 1);
    let target = Target::new();
    let control = Control::default();
    let done = execute(&a, target.options(), &control, |snapshot| {
        if snapshot.password_prompt.is_some() {
            control.cancel();
        }
    });
    assert_eq!(done.status, Status::Cancelled);
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
    assert!(done.password_prompt.is_none());
}

#[cfg(unix)]
#[test]
fn encrypted_link_graph_is_authenticated_before_any_output_and_restores_executable_mode() {
    use std::os::unix::fs::PermissionsExt;
    for invalid in [false, true] {
        let source = Target::new();
        let target = Target::new();
        let a = generated(
            &source,
            &archive_bytes(
                &[
                    (
                        "Demo.app/program",
                        b"#!/bin/sh\nexit 0\n",
                        Some("pw"),
                        false,
                    ),
                    (
                        "Demo.app/alias",
                        if invalid { b"../outside" } else { b"program" },
                        Some("pw"),
                        true,
                    ),
                ],
                8,
            ),
        );
        let control = Control::default();
        let mut prompts = 0;
        let done = execute(&a, target.options(), &control, |snapshot| {
            if let Some(prompt) = &snapshot.password_prompt {
                prompts += 1;
                assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
                control.respond_password(prompt.id, secret("pw"));
            }
        });
        assert_eq!(prompts, 1);
        if invalid {
            assert_eq!(done.error, "unsupported_link");
            assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
        } else {
            assert_eq!(done.status, Status::Succeeded, "{}", done.error);
            assert_eq!(
                std::fs::read_link(target.0.join("Demo.app/alias")).unwrap(),
                std::path::Path::new("program")
            );
            assert_eq!(
                std::fs::metadata(target.0.join("Demo.app/program"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o755
            );
            assert_eq!(done.decoded_bytes, done.declared_bytes);
            assert_eq!(done.committed_files, 2);
        }
    }
}

#[cfg(unix)]
#[test]
fn source_change_at_encrypted_link_commit_keeps_authenticated_files_and_omits_the_link() {
    let source = Target::new();
    let target = Target::new();
    let a = generated(
        &source,
        &archive_bytes(
            &[
                ("program", b"executable", Some("pw"), false),
                ("alias", b"program", Some("pw"), true),
            ],
            0,
        ),
    );
    let control = Control::default();
    let done = execute(&a, target.options(), &control, |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            control.respond_password(prompt.id, secret("pw"));
        }
        if snapshot.status == Status::Committing && snapshot.current == "alias" {
            std::fs::write(source.0.join("generated.zip"), b"changed").unwrap();
        }
    });
    assert_eq!(done.error, "source_changed");
    assert_eq!(done.committed_files, 1);
    assert_eq!(
        std::fs::read(target.0.join("program")).unwrap(),
        b"executable"
    );
    assert!(std::fs::symlink_metadata(target.0.join("alias")).is_err());
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 1);
}

#[test]
fn password_control_bypasses_a_full_command_queue_and_shutdown_wakes_waiting_tasks() {
    use crate::service::{self, Command, Payload, Rejection};
    use std::time::{Duration, Instant};
    for shutdown_while_waiting in [false, true] {
        let source = Target::new();
        let target = Target::new();
        let path = source.0.join("generated.zip");
        std::fs::write(
            &path,
            archive_bytes(&[("file", b"content", Some("right"), false)], 8),
        )
        .unwrap();
        let (client, mut receiver) = service::start().unwrap();
        let submit = |make: &mut dyn FnMut() -> Command| {
            let end = Instant::now() + Duration::from_secs(5);
            loop {
                match client.submit(make()) {
                    Ok(id) => break id,
                    Err(Rejection::Busy) if Instant::now() < end => std::thread::yield_now(),
                    other => panic!("unexpected admission: {other:?}"),
                }
            }
        };
        let open = submit(&mut || Command::Open(path.to_string_lossy().into_owned()));
        let mut session = 0;
        loop {
            let event = receiver.receive().unwrap();
            if let Payload::Page { session: id, .. } = event.payload {
                session = id;
            }
            if event.request == open && matches!(event.payload, Payload::Finished("")) {
                break;
            }
        }
        submit(&mut || Command::Extract {
            session,
            options: target.options(),
        });
        let (task, prompt) = loop {
            if let Payload::Task(snapshot) = receiver.receive().unwrap().payload
                && let Some(prompt) = snapshot.password_prompt
            {
                break (snapshot.id, prompt.id);
            }
        };
        assert!(
            client
                .respond_password(task + 1, prompt, secret("right"))
                .is_err()
        );
        assert!(
            client
                .respond_password(task, prompt + 1, secret("right"))
                .is_err()
        );
        // Requests retain their admission slots until their terminal is received.
        for _ in 0..service::MAX_OUTSTANDING {
            submit(&mut || Command::Snapshot);
        }
        assert_eq!(client.submit(Command::Snapshot), Err(Rejection::Busy));
        if !shutdown_while_waiting {
            assert!(
                client
                    .respond_password(task, prompt, secret("right"))
                    .is_ok()
            );
            assert!(
                client
                    .respond_password(task, prompt, secret("duplicate"))
                    .is_err()
            );
        }
        // Drain in another thread, with a bounded completion deadline so a lost wakeup
        // is reported as a failure instead of leaving the suite blocked indefinitely.
        let (tx, rx) = std::sync::mpsc::channel();
        let consumer = std::thread::spawn(move || {
            while let Some(event) = receiver.receive() {
                tx.send(event).unwrap();
            }
        });
        let mut terminals = 0;
        if shutdown_while_waiting {
            client.shutdown();
        } else {
            loop {
                let event = rx.recv_timeout(Duration::from_secs(5)).unwrap();
                if let Payload::Task(snapshot) = event.payload
                    && snapshot.id == task
                    && snapshot.status.terminal()
                {
                    assert_eq!(snapshot.status, Status::Succeeded, "{}", snapshot.error);
                    terminals += 1;
                    break;
                }
            }
            client.shutdown();
        }
        loop {
            let event = rx.recv_timeout(Duration::from_secs(5)).unwrap();
            if let Payload::Task(snapshot) = &event.payload
                && snapshot.id == task
                && snapshot.status.terminal()
            {
                assert_eq!(snapshot.status, Status::Cancelled);
                assert!(snapshot.password_prompt.is_none());
                terminals += 1;
            }
            if matches!(event.payload, Payload::ShutdownReady) {
                break;
            }
        }
        consumer.join().unwrap();
        assert_eq!(terminals, 1);
        assert!(
            client
                .respond_password(task, prompt, secret("terminal"))
                .is_err()
        );
        if shutdown_while_waiting {
            assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 0);
        } else {
            assert_eq!(std::fs::read(target.0.join("file")).unwrap(), b"content");
        }
    }
}
