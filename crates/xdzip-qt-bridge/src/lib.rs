//! Product cxx boundary: unique command and blocking event handles, no GUI archive calls.
mod update;

use xdzip_domain::identity::EntryId;
use xdzip_runtime::service;

// Generated ABI glue is the only unsafe boundary; handwritten service code remains safe Rust.
#[allow(unsafe_code)]
#[cxx::bridge(namespace = "xdzip::reader")]
mod ffi {
    enum Admission {
        Accepted,
        Busy,
        Closed,
        Invalid,
        Exhausted,
    }
    struct Submission {
        status: Admission,
        request_id: u64,
    }
    enum CommandKind {
        Open,
        Browse,
        Preflight,
        CloseSession,
        Snapshot,
        Extract,
        CheckTarget,
        Create,
        InspectInputs,
        OpenEntry,
        Check,
    }
    struct MetadataLimits {
        entries: u64,
        directories: u64,
        directory_bytes: u64,
        rows_per_batch: u64,
    }
    struct Command {
        kind: CommandKind,
        session: u64,
        path: String,
        directory: String,
        text: String,
        offset: u64,
        query: u64,
        generation: u64,
        revision: u64,
        sort: u8,
        selected: Vec<u64>,
        selectors: Vec<String>,
        inputs: Vec<String>,
        method: u16,
        level: u8,
        volume_bytes: u64,
        encrypt: bool,
        password: String,
        all: bool,
        create_folder: bool,
        conflict: u8,
        allow_high_ratio: bool,
    }
    struct Row {
        id: String,
        navigation_id: String,
        physical: bool,
        name: String,
        size: u64,
        packed_size: u64,
        method: u16,
        modified_dos: u32,
        directory: bool,
        encrypted: bool,
        warning: bool,
    }
    struct Directory {
        id: String,
        parent: String,
        name: String,
        path: String,
        count: u64,
    }
    struct InputInfo {
        known: bool,
        directory: bool,
        bytes: u64,
    }
    struct PreflightReport {
        entries: u64,
        declared_bytes: String,
        findings: Vec<String>,
        finding_count: u64,
        unicode_review: bool,
    }
    struct TaskOutcome {
        name: String,
        size: u64,
        status: String,
        error: String,
    }
    struct TaskSnapshot {
        kind: String,
        source: String,
        format: String,
        creating: bool,
        total_known: bool,
        id: u64,
        session: u64,
        status: String,
        target: String,
        open_path: String,
        current: String,
        files: u64,
        declared_bytes: u64,
        decoded_bytes: u64,
        read_bytes: u64,
        verified_files: u64,
        verified_bytes: u64,
        written_bytes: u64,
        committed_bytes: u64,
        committed_files: u64,
        skipped_files: u64,
        created_directories: u64,
        elapsed_ms: u64,
        error: String,
        cleanup_warning: bool,
        outcome_uncertain: bool,
        leftovers: u64,
        details: Vec<TaskOutcome>,
        omitted_details: u64,
        external_reader: bool,
        output_parts: u64,
        prompt_id: u64,
        prompt_name: String,
        existing_bytes: u64,
        incoming_bytes: u64,
        existing_modified: String,
        incoming_modified_dos: u32,
        overwrite_allowed: bool,
        password_prompt_id: u64,
        password_prompt_name: String,
        password_retry: bool,
    }
    enum EventKind {
        Page,
        Preflight,
        Snapshot,
        Finished,
        ShutdownReady,
        RuntimeFault,
        End,
        TaskAccepted,
        TaskUpdate,
        TargetChecked,
        Inputs,
    }
    struct Event {
        kind: EventKind,
        sequence: u64,
        request_id: u64,
        session: u64,
        offset: u64,
        total: u64,
        archive_total: u64,
        format: String,
        folder_name: String,
        query: u64,
        generation: u64,
        revision: u64,
        rows: Vec<Row>,
        inputs: Vec<InputInfo>,
        directories: Vec<Directory>,
        replace_directories: bool,
        navigation_limited: bool,
        report: PreflightReport,
        error: String,
        task: TaskSnapshot,
    }
    extern "Rust" {
        type Runtime;
        type CommandClient;
        type EventReceiver;
        fn metadata_limits() -> MetadataLimits;
        fn sum_declared_sizes(values: &[u64]) -> String;
        fn new_runtime() -> Result<Box<Runtime>>;
        fn take_client(self: &mut Runtime) -> Result<Box<CommandClient>>;
        fn take_receiver(self: &mut Runtime) -> Result<Box<EventReceiver>>;
        fn submit(self: &CommandClient, command: Command) -> Submission;
        fn cancel(self: &CommandClient, request_id: u64) -> Submission;
        fn cancel_task(self: &CommandClient, task: u64) -> bool;
        fn decide_task(self: &CommandClient, task: u64, prompt: u64, policy: u8, all: bool)
        -> bool;
        fn respond_password(self: &CommandClient, task: u64, prompt: u64, password: String)
        -> bool;
        fn shutdown(self: &CommandClient) -> u64;
        fn next_event(self: &mut EventReceiver) -> Event;
    }
}
/// Read-only resource admission shared with the Qt receiver; no filesystem work.
pub fn metadata_limits() -> ffi::MetadataLimits {
    ffi::MetadataLimits {
        entries: xdzip_codec_api::MAX_ENTRIES as u64,
        directories: xdzip_runtime::navigation::MAX_DIRECTORIES as u64,
        directory_bytes: xdzip_runtime::navigation::MAX_DIRECTORY_BYTES as u64,
        rows_per_batch: 128,
    }
}
/// Exact bounded metadata aggregation, without I/O or retaining the borrowed slice.
pub fn sum_declared_sizes(values: &[u64]) -> String {
    if values.len() > xdzip_codec_api::MAX_ENTRIES {
        return String::new();
    }
    values
        .iter()
        .map(|&value| u128::from(value))
        .sum::<u128>()
        .to_string()
}
/// Temporary owner. Each independent handle may be transferred once.
pub struct Runtime {
    client: Option<CommandClient>,
    receiver: Option<EventReceiver>,
}
/// Exclusive GUI command handle. Short admission/control calls only.
pub struct CommandClient(service::Client);
/// Exclusive receiver-thread handle. Receive may block and joins the worker at shutdown.
pub struct EventReceiver(service::Receiver);
/// Start a runtime; thread creation failure returns a fixed error without source paths.
pub fn new_runtime() -> Result<Box<Runtime>, &'static str> {
    let (client, receiver) = service::start().map_err(|_| "runtime_unavailable")?;
    Ok(Box::new(Runtime {
        client: Some(CommandClient(client)),
        receiver: Some(EventReceiver(receiver)),
    }))
}
impl Runtime {
    /// Transfer the unique producer.
    pub fn take_client(&mut self) -> Result<Box<CommandClient>, &'static str> {
        self.client
            .take()
            .map(Box::new)
            .ok_or("client_already_taken")
    }
    /// Transfer the unique blocking receiver.
    pub fn take_receiver(&mut self) -> Result<Box<EventReceiver>, &'static str> {
        self.receiver
            .take()
            .map(Box::new)
            .ok_or("receiver_already_taken")
    }
}
fn admission(result: Result<u64, service::Rejection>) -> ffi::Submission {
    match result {
        Ok(request_id) => ffi::Submission {
            status: ffi::Admission::Accepted,
            request_id,
        },
        Err(error) => ffi::Submission {
            request_id: 0,
            status: match error {
                service::Rejection::Busy => ffi::Admission::Busy,
                service::Rejection::Closed => ffi::Admission::Closed,
                service::Rejection::Invalid => ffi::Admission::Invalid,
                service::Rejection::Exhausted => ffi::Admission::Exhausted,
            },
        },
    }
}
impl CommandClient {
    /// Validate bounded foreign DTOs before admitting any work. Rejection consumes no ID.
    pub fn submit(&self, command: ffi::Command) -> ffi::Submission {
        // Move the ABI-owned secret into zeroizing storage before any rejection.
        // It never becomes a progress field, debug string or retained event.
        let empty_password = command.password.is_empty();
        let candidate = xdzip_runtime::creation::Password::new(command.password);
        let password = if command.encrypt {
            match candidate {
                Ok(value) => Some(value),
                Err(_) => return admission(Err(Rejection::Invalid)),
            }
        } else {
            if !empty_password {
                return admission(Err(Rejection::Invalid));
            }
            None
        };
        use service::{Command, Rejection};
        if command.inputs.len() > xdzip_codec_api::MAX_ENTRIES
            || command
                .inputs
                .iter()
                .any(|p| p.len() > 4096 || p.contains('\0'))
            || command.inputs.iter().map(String::len).sum::<usize>() > 4 * 1024 * 1024
            || command.path.len() > 4096
            || command.directory.len() > 21
            || command.text.len() > 1024
            || command.selected.len() > 128
            || command.sort > 12
            || command.selectors.len() > xdzip_codec_api::MAX_ENTRIES
            || command.selectors.iter().any(|s| s.len() > 21)
            || command
                .selectors
                .iter()
                .map(|s| s.len() + std::mem::size_of::<String>())
                .sum::<usize>()
                > 4 * 1024 * 1024
        {
            return admission(Err(Rejection::Invalid));
        }
        let value = match command.kind {
            ffi::CommandKind::Check => Command::Check {
                session: command.session,
            },
            ffi::CommandKind::InspectInputs => Command::InspectInputs(command.inputs),
            ffi::CommandKind::Create => {
                let Some(conflict) =
                    xdzip_runtime::extraction::ConflictPolicy::from_code(command.conflict)
                else {
                    return admission(Err(Rejection::Invalid));
                };
                Command::Create(xdzip_runtime::creation::Options {
                    inputs: command.inputs,
                    target: command.path,
                    name: command.text,
                    method: command.method,
                    level: command.level,
                    conflict,
                    volume_bytes: command.volume_bytes,
                    password,
                })
            }
            ffi::CommandKind::Open => Command::Open(command.path),
            ffi::CommandKind::Browse => Command::Browse {
                session: command.session,
                directory: command.directory,
                text: command.text,
                offset: command.offset,
                query: command.query,
                generation: command.generation,
                revision: command.revision,
                sort: command.sort,
            },
            ffi::CommandKind::Preflight => {
                let Some(selected) = command
                    .selected
                    .into_iter()
                    .map(|n| EntryId::try_from(n).ok())
                    .collect::<Option<Vec<_>>>()
                else {
                    return admission(Err(Rejection::Invalid));
                };
                Command::Preflight {
                    session: command.session,
                    selected,
                }
            }
            ffi::CommandKind::OpenEntry => Command::OpenEntry {
                session: command.session,
                entry: command.offset,
                cache: command.path,
            },
            ffi::CommandKind::Extract => {
                let Some(conflict) =
                    xdzip_runtime::extraction::ConflictPolicy::from_code(command.conflict)
                else {
                    return admission(Err(Rejection::Invalid));
                };
                Command::Extract {
                    session: command.session,
                    options: xdzip_runtime::extraction::Options {
                        target: command.path,
                        all: command.all,
                        selected: command.selectors,
                        directory: command.directory,
                        create_folder: command.create_folder,
                        conflict,
                        allow_high_ratio: command.allow_high_ratio,
                    },
                }
            }
            ffi::CommandKind::CheckTarget => Command::CheckTarget {
                session: command.session,
                path: command.path,
            },
            ffi::CommandKind::CloseSession => Command::CloseSession(command.session),
            ffi::CommandKind::Snapshot => Command::Snapshot,
            _ => return admission(Err(Rejection::Invalid)),
        };
        admission(self.0.submit(value))
    }
    /// Independent control acknowledgement; does not pretend cancellation has finished.
    pub fn cancel(&self, request_id: u64) -> ffi::Submission {
        admission(self.0.cancel(request_id))
    }
    /// Cancel a task without waiting for the data lane or filesystem operations.
    pub fn cancel_task(&self, task: u64) -> bool {
        self.0.cancel_task(task).is_ok()
    }
    /// Resolve only a current conflict, with explicit overwrite policy.
    pub fn decide_task(&self, task: u64, prompt: u64, policy: u8, all: bool) -> bool {
        let Some(policy) = xdzip_runtime::extraction::ConflictPolicy::from_code(policy) else {
            return false;
        };
        self.0.decide_task(task, prompt, policy, all).is_ok()
    }
    /// Move a foreign-owned secret into zeroizing storage before any rejection.
    /// This uses the task's bounded control slot, never the command/event queues.
    pub fn respond_password(&self, task: u64, prompt: u64, password: String) -> bool {
        let Ok(password) = xdzip_runtime::extraction::Password::new(password) else {
            return false;
        };
        self.0.respond_password(task, prompt, password).is_ok()
    }
    /// Idempotently stop admission and request cooperative cancellation.
    pub fn shutdown(&self) -> u64 {
        self.0.shutdown()
    }
}
impl EventReceiver {
    /// Block without polling, return an owned typed event or explicit end-of-stream.
    pub fn next_event(&mut self) -> ffi::Event {
        let mut event = ffi::Event {
            kind: ffi::EventKind::End,
            sequence: 0,
            request_id: 0,
            session: 0,
            offset: 0,
            total: 0,
            archive_total: 0,
            format: String::new(),
            folder_name: String::new(),
            query: 0,
            generation: 0,
            revision: 0,
            rows: Vec::new(),
            inputs: Vec::new(),
            directories: Vec::new(),
            replace_directories: false,
            navigation_limited: false,
            error: String::new(),
            task: empty_task(),
            report: ffi::PreflightReport {
                entries: 0,
                declared_bytes: String::new(),
                findings: Vec::new(),
                finding_count: 0,
                unicode_review: false,
            },
        };
        let Some(value) = self.0.receive() else {
            return event;
        };
        event.sequence = value.sequence;
        event.request_id = value.request;
        match value.payload {
            service::Payload::Page {
                session,
                offset,
                total,
                archive_total,
                format,
                folder_name,
                key,
                rows,
                navigation,
                navigation_limited,
            } => {
                event.kind = ffi::EventKind::Page;
                event.session = session;
                event.offset = offset;
                event.total = total;
                event.archive_total = archive_total;
                event.format = format.label().into();
                event.folder_name = folder_name;
                event.query = key.query.get();
                event.generation = key.generation.get();
                event.revision = key.revision.get();
                event.replace_directories = navigation.is_some();
                event.navigation_limited = navigation_limited;
                event.directories = navigation
                    .unwrap_or_default()
                    .into_iter()
                    .map(|node| ffi::Directory {
                        id: format!("d{}", node.id),
                        parent: if node.parent == 0 {
                            String::new()
                        } else {
                            format!("d{}", node.parent)
                        },
                        name: node.name,
                        path: node.path,
                        count: node.count,
                    })
                    .collect();
                event.rows = rows
                    .into_iter()
                    .map(|row| ffi::Row {
                        id: if row.id == 0 {
                            format!("d{}", row.navigation_id)
                        } else {
                            row.id.to_string()
                        },
                        navigation_id: if row.navigation_id == 0 {
                            String::new()
                        } else {
                            format!("d{}", row.navigation_id)
                        },
                        physical: row.id != 0,
                        name: row.name,
                        size: row.size,
                        packed_size: row.packed_size,
                        method: row.method,
                        modified_dos: row.modified_dos,
                        directory: row.directory,
                        encrypted: row.encrypted,
                        warning: row.warning,
                    })
                    .collect();
            }
            service::Payload::Preflight(report) => {
                event.kind = ffi::EventKind::Preflight;
                event.report = ffi::PreflightReport {
                    entries: report.entries as u64,
                    declared_bytes: report
                        .declared_bytes
                        .map(|n| n.to_string())
                        .unwrap_or_else(|| "unknown".into()),
                    findings: report
                        .findings
                        .into_iter()
                        .map(|f| format!("{}:{}", f.entry.get(), f.reason))
                        .collect(),
                    finding_count: report.finding_count as u64,
                    unicode_review: report.unicode_review,
                };
            }
            service::Payload::Inputs(rows) => {
                event.kind = ffi::EventKind::Inputs;
                event.inputs = rows
                    .into_iter()
                    .map(|r| ffi::InputInfo {
                        known: r.known,
                        directory: r.directory,
                        bytes: r.bytes,
                    })
                    .collect();
            }
            service::Payload::TaskAccepted(id) => {
                event.kind = ffi::EventKind::TaskAccepted;
                event.task.id = id;
            }
            service::Payload::TargetChecked(leftovers) => {
                event.kind = ffi::EventKind::TargetChecked;
                event.total = leftovers;
            }
            service::Payload::Task(task) => {
                event.kind = ffi::EventKind::TaskUpdate;
                event.task = task_snapshot(*task);
            }
            service::Payload::Snapshot(session) => {
                event.kind = ffi::EventKind::Snapshot;
                event.session = session;
            }
            service::Payload::Finished(error) => {
                event.kind = ffi::EventKind::Finished;
                event.error = error.into();
            }
            service::Payload::ShutdownReady => event.kind = ffi::EventKind::ShutdownReady,
            service::Payload::RuntimeFault => {
                event.kind = ffi::EventKind::RuntimeFault;
                event.error = "runtime_unavailable".into();
            }
        }
        event
    }
}

fn empty_task() -> ffi::TaskSnapshot {
    ffi::TaskSnapshot {
        kind: String::new(),
        source: String::new(),
        format: String::new(),
        creating: false,
        total_known: false,
        id: 0,
        session: 0,
        status: String::new(),
        target: String::new(),
        open_path: String::new(),
        current: String::new(),
        files: 0,
        declared_bytes: 0,
        decoded_bytes: 0,
        read_bytes: 0,
        verified_files: 0,
        verified_bytes: 0,
        written_bytes: 0,
        committed_bytes: 0,
        committed_files: 0,
        skipped_files: 0,
        created_directories: 0,
        elapsed_ms: 0,
        error: String::new(),
        cleanup_warning: false,
        outcome_uncertain: false,
        leftovers: 0,
        details: Vec::new(),
        omitted_details: 0,
        external_reader: false,
        output_parts: 0,
        prompt_id: 0,
        prompt_name: String::new(),
        existing_bytes: 0,
        incoming_bytes: 0,
        existing_modified: String::new(),
        incoming_modified_dos: 0,
        overwrite_allowed: false,
        password_prompt_id: 0,
        password_prompt_name: String::new(),
        password_retry: false,
    }
}
fn task_snapshot(s: xdzip_runtime::extraction::Snapshot) -> ffi::TaskSnapshot {
    let prompt = s.prompt;
    let password = s.password_prompt;
    ffi::TaskSnapshot {
        kind: s.kind.code().into(),
        source: s.source,
        format: s.format.into(),
        creating: s.creating,
        total_known: s.total_known,
        id: s.id,
        session: s.session,
        status: s.status.code().into(),
        target: s.target,
        open_path: s.open_path,
        current: s.current,
        files: s.files,
        declared_bytes: s.declared_bytes,
        decoded_bytes: s.decoded_bytes,
        read_bytes: s.read_bytes,
        verified_files: s.verified_files,
        verified_bytes: s.verified_bytes,
        written_bytes: s.written_bytes,
        committed_bytes: s.committed_bytes,
        committed_files: s.committed_files,
        skipped_files: s.skipped_files,
        created_directories: s.created_directories,
        elapsed_ms: s.elapsed_ms,
        error: s.error,
        cleanup_warning: s.cleanup_warning,
        outcome_uncertain: s.outcome_uncertain,
        leftovers: s.leftovers,
        details: s
            .details
            .iter()
            .map(|d| ffi::TaskOutcome {
                name: d.name.clone(),
                size: d.size,
                status: d.status.into(),
                error: d.error.into(),
            })
            .collect(),
        omitted_details: s.omitted_details,
        external_reader: s.external_reader,
        output_parts: s.output_parts,
        prompt_id: prompt.as_ref().map_or(0, |p| p.id),
        prompt_name: prompt.as_ref().map_or(String::new(), |p| p.name.clone()),
        existing_bytes: prompt.as_ref().map_or(0, |p| p.existing_bytes),
        incoming_bytes: prompt.as_ref().map_or(0, |p| p.incoming_bytes),
        existing_modified: prompt
            .as_ref()
            .and_then(|p| p.existing_modified)
            .map_or(String::new(), |n| n.to_string()),
        incoming_modified_dos: prompt.as_ref().map_or(0, |p| p.incoming_modified_dos),
        overwrite_allowed: prompt.as_ref().is_some_and(|p| p.overwrite_allowed),
        password_prompt_id: password.as_ref().map_or(0, |p| p.id),
        password_prompt_name: password.as_ref().map_or(String::new(), |p| p.name.clone()),
        password_retry: password.as_ref().is_some_and(|p| p.retry),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_command_delivers_read_only_kind_and_verified_counters() {
        fn command(kind: ffi::CommandKind, session: u64, path: String) -> ffi::Command {
            ffi::Command {
                kind,
                session,
                path,
                directory: String::new(),
                text: String::new(),
                offset: 0,
                query: 0,
                generation: 0,
                revision: 0,
                sort: 0,
                selected: Vec::new(),
                selectors: Vec::new(),
                inputs: Vec::new(),
                method: 0,
                level: 0,
                volume_bytes: 0,
                encrypt: false,
                password: String::new(),
                all: false,
                create_folder: false,
                conflict: 0,
                allow_high_ratio: false,
            }
        }
        let mut runtime = new_runtime().unwrap();
        let client = runtime.take_client().unwrap();
        let mut receiver = runtime.take_receiver().unwrap();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/extract/stored.zip");
        let request = client.submit(command(
            ffi::CommandKind::Open,
            0,
            path.to_string_lossy().into_owned(),
        ));
        assert!(request.status == ffi::Admission::Accepted);
        let mut session = 0;
        loop {
            let event = receiver.next_event();
            if event.kind == ffi::EventKind::Page {
                session = event.session;
            }
            if event.kind == ffi::EventKind::Finished && event.request_id == request.request_id {
                assert!(event.error.is_empty());
                break;
            }
        }
        assert_ne!(session, 0);
        let request = client.submit(command(ffi::CommandKind::Check, session, String::new()));
        assert!(request.status == ffi::Admission::Accepted);
        let (mut accepted, mut finished, mut terminal) = (false, false, false);
        while !accepted || !finished || !terminal {
            let event = receiver.next_event();
            if event.kind == ffi::EventKind::TaskAccepted {
                accepted = true;
            }
            if event.kind == ffi::EventKind::Finished && event.request_id == request.request_id {
                assert!(event.error.is_empty());
                finished = true;
            }
            if event.kind == ffi::EventKind::TaskUpdate && event.task.status == "succeeded" {
                assert_eq!(event.task.kind, "check");
                assert!(
                    !event.task.creating
                        && event.task.target.is_empty()
                        && event.task.open_path.is_empty()
                );
                assert_eq!(event.task.verified_files, 1);
                assert!(event.task.verified_bytes > 0 && event.task.read_bytes > 0);
                assert_eq!(event.task.written_bytes, 0);
                assert_eq!(event.task.committed_files, 0);
                terminal = true;
            }
        }
        client.shutdown();
        while receiver.next_event().kind != ffi::EventKind::ShutdownReady {}
        assert!(receiver.next_event().kind == ffi::EventKind::End);
    }
    #[test]
    fn metadata_summary_is_exact_above_64_bits_and_bounded() {
        assert_eq!(sum_declared_sizes(&[]), "0");
        assert_eq!(
            sum_declared_sizes(&[u64::MAX, u64::MAX]),
            "36893488147419103230"
        );
        assert!(sum_declared_sizes(&vec![0; xdzip_codec_api::MAX_ENTRIES + 1]).is_empty());
    }
    #[test]
    fn unique_handles_foreign_bounds_and_shutdown() {
        let mut runtime = new_runtime().unwrap();
        let client = runtime.take_client().unwrap();
        assert!(runtime.take_client().is_err());
        let mut receiver = runtime.take_receiver().unwrap();
        assert!(runtime.take_receiver().is_err());
        drop(runtime);
        let invalid = ffi::Command {
            query: 0,
            generation: 0,
            revision: 0,
            kind: ffi::CommandKind::Open,
            session: 0,
            path: "x".repeat(4097),
            directory: String::new(),
            text: String::new(),
            offset: 0,
            sort: 0,
            selected: Vec::new(),
            selectors: Vec::new(),
            inputs: Vec::new(),
            method: 8,
            level: 6,
            volume_bytes: 0,
            encrypt: false,
            password: String::new(),
            all: false,
            create_folder: false,
            conflict: 0,
            allow_high_ratio: false,
        };
        assert!(client.submit(invalid).status == ffi::Admission::Invalid);
        assert!(!client.respond_password(0, 0, String::new()));
        assert!(!client.respond_password(1, 1, "x".repeat(1025)));
        let shutdown = client.shutdown();
        assert_eq!(shutdown, client.shutdown());
        assert!(!client.respond_password(1, 1, String::new()));
        let event = receiver.next_event();
        assert!(event.kind == ffi::EventKind::ShutdownReady);
        assert_eq!(event.request_id, shutdown);
        assert!(receiver.next_event().kind == ffi::EventKind::End);
    }
}
