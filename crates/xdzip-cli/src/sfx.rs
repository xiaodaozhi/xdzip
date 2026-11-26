//! Thin standalone host. All extraction and output authority remain in the shared Rust runtime.
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use xdzip_codec_api::{Cancellation, Format};
use xdzip_runtime::extraction::{ConflictPolicy, Options, Snapshot, Status};
use xdzip_runtime::service::{self, Command, Payload};

#[cfg(windows)]
mod windows;

#[used]
static MARKER: &[u8] = xdzip_runtime::sfx::STUB_MARKER;

#[derive(Default)]
struct Control {
    cancel: Cancellation,
    client: Mutex<Option<Arc<service::Client>>>,
    task: AtomicU64,
    request: AtomicU64,
}
impl Control {
    fn stop(&self) {
        self.cancel.cancel();
        if let Some(client) = self
            .client
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            let _ = client.cancel(self.request.load(Ordering::Acquire));
            let _ = client.cancel_task(self.task.load(Ordering::Acquire));
        }
    }
}
fn extract(
    source: &Path,
    target: Option<&Path>,
    policy: ConflictPolicy,
    control: &Control,
    mut progress: impl FnMut(Snapshot),
) -> Result<Snapshot, &'static str> {
    xdzip_runtime::sfx::validate(source, &control.cancel)?;
    if target.is_some_and(|target| !target.is_absolute() || !target.is_dir()) {
        return Err("unsafe_target");
    }
    let (client, mut receiver) = service::start().map_err(|_| "runtime_closed")?;
    let client = Arc::new(client);
    *control.client.lock().unwrap_or_else(|e| e.into_inner()) = Some(client.clone());
    let result: Result<Snapshot, &'static str> = (|| {
        let request = client
            .submit_wait(Command::Open(source.to_str().ok_or("unsafe_name")?.into()))
            .map_err(|_| "runtime_closed")?;
        control.request.store(request, Ordering::Release);
        let mut session = 0;
        while let Some(event) = receiver.receive() {
            if control.cancel.check().is_err() {
                control.stop();
            }
            match event.payload {
                Payload::Page {
                    session: id,
                    format,
                    ..
                } if event.request == request => {
                    if format != Format::Zip {
                        return Err("unsupported_feature");
                    }
                    session = id;
                }
                Payload::Finished(error) if event.request == request => {
                    if !error.is_empty() {
                        return Err(error);
                    }
                    break;
                }
                Payload::RuntimeFault => return Err("runtime_fault"),
                _ => {}
            }
        }
        control.cancel.check().map_err(|_| "cancelled")?;
        if session == 0 {
            return Err("runtime_closed");
        }
        let command = if let Some(target) = target {
            Command::Extract {
                session,
                target_identity: Some(
                    xdzip_io::Grant::for_creation(target)
                        .map_err(|_| "unsafe_target")?
                        .identity(),
                ),
                options: Options {
                    target: target.to_str().ok_or("unsafe_name")?.into(),
                    all: true,
                    selected: vec![],
                    directory: String::new(),
                    create_folder: false,
                    shell_layout: 0,
                    conflict: policy,
                    allow_high_ratio: false,
                },
            }
        } else {
            Command::Check { session }
        };
        let request = client.submit_wait(command).map_err(|_| "runtime_closed")?;
        control.request.store(request, Ordering::Release);
        while let Some(event) = receiver.receive() {
            if control.cancel.check().is_err() {
                control.stop();
            }
            match event.payload {
                Payload::TaskAccepted(id) if event.request == request => {
                    control.task.store(id, Ordering::Release);
                    if control.cancel.check().is_err() {
                        control.stop();
                    }
                }
                Payload::Task(snapshot) => {
                    control.task.store(snapshot.id, Ordering::Release);
                    progress((*snapshot).clone());
                    if snapshot.status.terminal() {
                        return Ok(*snapshot);
                    }
                    if matches!(snapshot.status, Status::Waiting | Status::WaitingPassword) {
                        control.stop();
                        return Err("unsupported_feature");
                    }
                }
                Payload::Finished(error) if event.request == request && !error.is_empty() => {
                    return Err(error);
                }
                Payload::RuntimeFault => return Err("runtime_fault"),
                _ => {}
            }
        }
        Err("runtime_closed")
    })();
    client.shutdown();
    while let Some(event) = receiver.receive() {
        if matches!(event.payload, Payload::ShutdownReady) {
            break;
        }
    }
    *control.client.lock().unwrap_or_else(|e| e.into_inner()) = None;
    result
}

pub(super) fn run() -> i32 {
    std::hint::black_box(MARKER);
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return 1;
    }
    let source = match std::env::current_exe() {
        Ok(path) => path,
        Err(_) => return 1,
    };
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.is_empty() {
        #[cfg(windows)]
        return windows::show(&source);
        #[cfg(not(windows))]
        return 1;
    }
    let result: Result<Snapshot, String> = (|| -> Result<Snapshot, String> {
        if args.len() == 1 && args[0] == "--test" {
            return Ok(extract(
                &source,
                None,
                ConflictPolicy::Skip,
                &Control::default(),
                |_| {},
            )?);
        }
        let mut target = None;
        let mut policy = ConflictPolicy::Skip;
        let mut index = 0;
        while index < args.len() {
            let value = args.get(index + 1).ok_or("invalid_arguments")?;
            if args[index] == "--extract-to" && target.is_none() {
                target = Some(PathBuf::from(value));
            } else if args[index] == "--conflict" {
                policy = match value.to_str() {
                    Some("skip") => ConflictPolicy::Skip,
                    Some("rename") => ConflictPolicy::Rename,
                    Some("overwrite") => ConflictPolicy::Overwrite,
                    _ => return Err("invalid_arguments".into()),
                };
            } else {
                return Err("invalid_arguments".into());
            }
            index += 2;
        }
        Ok(extract(
            &source,
            Some(&target.ok_or("invalid_arguments")?),
            policy,
            &Arc::new(Control::default()),
            |_| {},
        )?)
    })();
    match result {
        Ok(snapshot) => {
            let succeeded = snapshot.status == Status::Succeeded;
            let status = if succeeded {
                "succeeded"
            } else if snapshot.status == Status::Cancelled {
                "cancelled"
            } else {
                "failed"
            };
            let error = if snapshot.status == Status::Cancelled {
                "cancelled"
            } else {
                &snapshot.error
            };
            println!(
                "{}",
                serde_json::json!({"status":status,"error_code":error,"committed_files":snapshot.committed_files,"skipped_files":snapshot.skipped_files,"cleanup_warning":snapshot.cleanup_warning,"leftovers":snapshot.leftovers,"outcome_uncertain":snapshot.outcome_uncertain})
            );
            i32::from(!succeeded)
        }
        Err(error) => {
            println!(
                "{}",
                serde_json::json!({"status":"failed","error_code":error})
            );
            1
        }
    }
}
