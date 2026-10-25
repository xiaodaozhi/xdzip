// crates/xdzip-runtime/src/task_interaction.rs

use crate::extraction::{Control, Password, PasswordPrompt, Snapshot, Status};
use std::time::Instant;
use xdzip_codec_api::payload::DecodeError;

pub(crate) fn decoding(error: DecodeError) -> &'static str {
    match error {
        DecodeError::SourceIo => "source_io",
        DecodeError::Integrity => "integrity_error",
        DecodeError::Unsupported => "unsupported_feature",
        DecodeError::Limit => "resource_limit",
        DecodeError::Cancelled => "cancelled",
        DecodeError::Output => "output_io",
        DecodeError::PasswordRequired => "password_required",
        DecodeError::WrongPassword => "wrong_password",
    }
}
pub(crate) fn emit(stats: &mut Snapshot, start: Instant, publish: &mut dyn FnMut(Snapshot)) {
    stats.elapsed_ms = start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    publish(stats.clone());
}
#[derive(Default)]
pub(crate) struct Interaction {
    next_prompt: u64,
    password: Option<Password>,
}
impl Interaction {
    pub(crate) fn with_password(password: Option<Password>) -> Self {
        Self {
            next_prompt: 0,
            password,
        }
    }
    pub(crate) fn fresh_prompt(&mut self) -> Result<u64, &'static str> {
        self.next_prompt = self.next_prompt.checked_add(1).ok_or("resource_limit")?;
        Ok(self.next_prompt)
    }
    fn request_password(
        &mut self,
        entry: &xdzip_codec_api::Entry,
        control: &Control,
        stats: &mut Snapshot,
        start: Instant,
        publish: &mut dyn FnMut(Snapshot),
        retry: bool,
    ) -> Result<(), &'static str> {
        self.password = None;
        let id = self.fresh_prompt()?;
        control.arm_password(id);
        let previous = stats.status;
        stats.status = Status::WaitingPassword;
        stats.password_prompt = Some(PasswordPrompt {
            id,
            name: entry.display_name.clone(),
            retry,
        });
        emit(stats, start, publish);
        self.password = Some(control.wait_password()?);
        stats.password_prompt = None;
        stats.status = previous;
        emit(stats, start, publish);
        Ok(())
    }
    pub(crate) fn decode<T>(
        &mut self,
        entry: &xdzip_codec_api::Entry,
        control: &Control,
        stats: &mut Snapshot,
        start: Instant,
        publish: &mut dyn FnMut(Snapshot),
        mut attempt: impl FnMut(
            Option<&str>,
            &mut Snapshot,
            &mut dyn FnMut(Snapshot),
        ) -> Result<T, &'static str>,
    ) -> Result<T, &'static str> {
        if entry.format == xdzip_codec_api::Format::Rar && entry.method < 50 && entry.encrypted {
            // Old RAR has no reliable verifier. Do not try a prior record's secret and then misclassify a CRC failure as a proven wrong password.
            if !entry.password_supported {
                return Err("unsupported_feature");
            }
            self.request_password(entry, control, stats, start, publish, false)?;
        }
        let mut failures = 0;
        loop {
            control.cancel.check().map_err(|_| "cancelled")?;
            let result = attempt(self.password.as_ref().map(Password::expose), stats, publish);
            match result {
                Err(error @ ("password_required" | "wrong_password")) => {
                    if !entry.password_supported {
                        return Err("unsupported_feature");
                    }
                    let retry = error == "wrong_password";
                    if retry {
                        failures += 1;
                        if failures >= 10 {
                            return Err("wrong_password");
                        }
                    }
                    self.request_password(entry, control, stats, start, publish, retry)?;
                }
                other => return other,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn legacy_rar_requests_each_secret_before_decoding_and_preserves_integrity_errors() {
        let archive = crate::Archive::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/extract/stored.zip"),
            &Default::default(),
        )
        .unwrap();
        let mut stats =
            crate::checking::initial(&archive.check_job(&Default::default()).unwrap(), 1);
        let control = Control::default();
        let mut interaction = Interaction::with_password(Some(
            Password::new("unrelated cached secret".into()).unwrap(),
        ));
        let mut entry = archive.rows[0].clone();
        entry.format = xdzip_codec_api::Format::Rar;
        entry.method = 29;
        entry.encrypted = true;
        entry.password_supported = true;
        let mut prompts = 0;
        for secret in ["first record", "second record"] {
            let result = interaction.decode(
                &entry,
                &control,
                &mut stats,
                Instant::now(),
                &mut |snapshot| {
                    if let Some(prompt) = snapshot.password_prompt {
                        prompts += 1;
                        assert!(!prompt.retry);
                        assert!(
                            control
                                .respond_password(prompt.id, Password::new(secret.into()).unwrap())
                        );
                    }
                },
                |password, _, _| {
                    assert_eq!(password, Some(secret));
                    Err::<(), _>("integrity_error")
                },
            );
            assert_eq!(result, Err("integrity_error"));
        }
        assert_eq!(prompts, 2);
    }
}
