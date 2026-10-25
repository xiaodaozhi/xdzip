//! Explicit owned official-author fixtures through the actual runtime/worker path.
use super::tests::{Target, execute};
use super::*;
use crate::Archive;
use std::path::PathBuf;

const CORRECT: &str = "owned-graph-fixture-password";
fn fixture() -> (Archive, Vec<u8>) {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../xdzip-codec-7z/tests/fixtures/password");
    let expected = (0..16391)
        .map(|i| ((i * 17 + i / 251) & 255) as u8)
        .collect::<Vec<_>>();
    let archive = Archive::open(&root.join("copy-aes.7z"), &Default::default()).unwrap();
    (archive, expected)
}
fn options(archive: &Archive, target: &Target) -> Options {
    let mut options = target.options();
    options.all = false;
    options.selected = vec![
        archive
            .rows
            .iter()
            .find(|row| row.name_bytes() == b"first.bin")
            .unwrap()
            .id
            .get()
            .to_string(),
    ];
    options.conflict = ConflictPolicy::Overwrite;
    options.allow_high_ratio = true;
    options
}
fn original_is_unpublished(target: &Target) {
    assert_eq!(
        std::fs::read(target.0.join("first.bin")).unwrap(),
        b"original-target"
    );
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
}
#[test]
fn ambiguous_sevenzip_retry_discards_failed_stage_and_publishes_only_verified_bytes() {
    let (archive, expected) = fixture();
    let target = Target::new();
    std::fs::write(target.0.join("first.bin"), b"original-target").unwrap();
    let control = Control::default();
    let mut prompts = 0;
    let done = execute(&archive, options(&archive, &target), &control, |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            original_is_unpublished(&target);
            assert_eq!(prompt.retry, prompts != 0);
            assert!(
                control.respond_password(
                    prompt.id,
                    Password::new(
                        if prompts == 0 {
                            "incorrect-fixture-secret"
                        } else {
                            CORRECT
                        }
                        .into()
                    )
                    .unwrap()
                )
            );
            prompts += 1;
        }
    });
    assert_eq!(done.status, Status::Succeeded, "{}", done.error);
    assert_eq!(prompts, 2);
    assert_eq!(done.committed_files, 1);
    assert_eq!(std::fs::read(target.0.join("first.bin")).unwrap(), expected);
    assert!(done.decoded_bytes >= expected.len() as u64 * 2);
    assert!(done.written_bytes >= expected.len() as u64 * 2);
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 1);
}
#[test]
fn ambiguous_sevenzip_tenth_failure_keeps_its_error_and_never_replaces_the_target() {
    let (archive, expected) = fixture();
    let target = Target::new();
    std::fs::write(target.0.join("first.bin"), b"original-target").unwrap();
    let control = Control::default();
    let mut prompts = 0;
    let done = execute(&archive, options(&archive, &target), &control, |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            original_is_unpublished(&target);
            assert_eq!(prompt.retry, prompts != 0);
            assert!(control.respond_password(
                prompt.id,
                Password::new("incorrect-fixture-secret".into()).unwrap()
            ));
            prompts += 1;
        }
    });
    assert_eq!(done.status, Status::Failed);
    assert_eq!(done.error, "password_or_integrity");
    assert_eq!(prompts, 10);
    assert_eq!(done.committed_files, 0);
    assert!(done.decoded_bytes >= expected.len() as u64 * 10);
    assert!(done.written_bytes >= expected.len() as u64 * 10);
    original_is_unpublished(&target);
    assert_eq!(std::fs::read_dir(&target.0).unwrap().count(), 1);
}
#[test]
fn ambiguous_sevenzip_prompt_can_cancel_without_publishing_failed_plaintext() {
    let (archive, _) = fixture();
    let target = Target::new();
    std::fs::write(target.0.join("first.bin"), b"original-target").unwrap();
    let control = Control::default();
    let mut prompts = 0;
    let done = execute(&archive, options(&archive, &target), &control, |snapshot| {
        if let Some(prompt) = &snapshot.password_prompt {
            original_is_unpublished(&target);
            prompts += 1;
            if prompt.retry {
                control.cancel();
            } else {
                assert!(control.respond_password(
                    prompt.id,
                    Password::new("incorrect-fixture-secret".into()).unwrap()
                ));
            }
        }
    });
    assert_eq!(done.status, Status::Cancelled);
    assert_eq!(prompts, 2);
    assert_eq!(done.committed_files, 0);
    original_is_unpublished(&target);
}
