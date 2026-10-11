//! Product cxx boundary: unique command and blocking event handles, no GUI archive calls.
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
    }
    struct Command {
        kind: CommandKind,
        session: u64,
        path: String,
        directory: String,
        text: String,
        offset: u64,
        selected: Vec<u64>,
    }
    struct Row {
        id: String,
        name: String,
        size: u64,
        packed_size: u64,
        method: u16,
        directory: bool,
        encrypted: bool,
        warning: bool,
    }
    struct PreflightReport {
        entries: u64,
        declared_bytes: String,
        findings: Vec<String>,
        finding_count: u64,
        unicode_review: bool,
    }
    enum EventKind {
        Page,
        Preflight,
        Snapshot,
        Finished,
        ShutdownReady,
        RuntimeFault,
        End,
    }
    struct Event {
        kind: EventKind,
        sequence: u64,
        request_id: u64,
        session: u64,
        offset: u64,
        total: u64,
        rows: Vec<Row>,
        report: PreflightReport,
        error: String,
    }
    extern "Rust" {
        type Runtime;
        type CommandClient;
        type EventReceiver;
        fn new_runtime() -> Result<Box<Runtime>>;
        fn take_client(self: &mut Runtime) -> Result<Box<CommandClient>>;
        fn take_receiver(self: &mut Runtime) -> Result<Box<EventReceiver>>;
        fn submit(self: &CommandClient, command: Command) -> Submission;
        fn cancel(self: &CommandClient, request_id: u64) -> Submission;
        fn shutdown(self: &CommandClient) -> u64;
        fn next_event(self: &mut EventReceiver) -> Event;
    }
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
        use service::{Command, Rejection};
        if command.path.len() > 4096
            || command.directory.len() > 20
            || command.text.len() > 1024
            || command.selected.len() > 128
        {
            return admission(Err(Rejection::Invalid));
        }
        let value = match command.kind {
            ffi::CommandKind::Open => Command::Open(command.path),
            ffi::CommandKind::Browse => Command::Browse {
                session: command.session,
                directory: command.directory,
                text: command.text,
                offset: command.offset,
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
            rows: Vec::new(),
            error: String::new(),
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
                rows,
            } => {
                event.kind = ffi::EventKind::Page;
                event.session = session;
                event.offset = offset;
                event.total = total;
                event.rows = rows
                    .into_iter()
                    .map(|row| ffi::Row {
                        id: row.id.to_string(),
                        name: row.name,
                        size: row.size,
                        packed_size: row.packed_size,
                        method: row.method,
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unique_handles_foreign_bounds_and_shutdown() {
        let mut runtime = new_runtime().unwrap();
        let client = runtime.take_client().unwrap();
        assert!(runtime.take_client().is_err());
        let mut receiver = runtime.take_receiver().unwrap();
        assert!(runtime.take_receiver().is_err());
        drop(runtime);
        let invalid = ffi::Command {
            kind: ffi::CommandKind::Open,
            session: 0,
            path: "x".repeat(4097),
            directory: String::new(),
            text: String::new(),
            offset: 0,
            selected: Vec::new(),
        };
        assert!(client.submit(invalid).status == ffi::Admission::Invalid);
        let shutdown = client.shutdown();
        assert_eq!(shutdown, client.shutdown());
        let event = receiver.next_event();
        assert!(event.kind == ffi::EventKind::ShutdownReady);
        assert_eq!(event.request_id, shutdown);
        assert!(receiver.next_event().kind == ffi::EventKind::End);
    }
}
