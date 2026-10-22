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
    pub(crate) fn fresh_prompt(&mut self) -> Result<u64, &'static str> {
        self.next_prompt = self.next_prompt.checked_add(1).ok_or("resource_limit")?;
        Ok(self.next_prompt)
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
                    // Do not retain both a rejected cached secret and the next pending response.
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
                }
                other => return other,
            }
        }
    }
}
