//! Product command/event service. GUI admission is bounded; all archive work is worker-owned.
use crate::Archive;
mod tasks;
use std::collections::{BTreeMap, VecDeque};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, TryLockError};
use std::thread::{self, JoinHandle};
use xdzip_codec_api::Cancellation;
use xdzip_domain::identity::{EntryId, QueryGeneration, QueryId, SnapshotRevision};
use xdzip_domain::query::QueryKey;

/// Maximum accepted normal requests not yet delivered to the receiver as finished.
pub const MAX_OUTSTANDING: usize = 128;
/// Maximum owned page/report string bytes retained in the Rust event queue.
pub const MAX_EVENT_BYTES: usize = 8 * 1024 * 1024;

/// Owned bounded commands. Source paths are user-selected inputs, never output authority.
pub enum Command {
    /// Replace the single development-stage session and return its first page.
    Open(String),
    /// Query an immutable session; session mismatch fails before reading metadata.
    Browse {
        /// Session identity checked before acting.
        session: u64,
        /// Physical directory EntryId or virtual d-prefixed node ID; empty means all records.
        directory: String,
        /// Bounded literal name search.
        text: String,
        /// Offset within the immutable result.
        offset: u64,
        /// Query ID of an immutable continuation; zero starts a new query.
        query: u64,
        /// UI intent generation; zero asks the internal client to allocate one.
        generation: u64,
        /// Immutable scan version for a continuation.
        revision: u64,
        /// Full-result sort code: 0 physical; pairs 1..12 select name/size/packed/method/type/date.
        sort: u8,
    },
    /// Inspect selected physical records without writing.
    Preflight {
        /// Session identity checked before acting.
        session: u64,
        /// Bounded physical record selection.
        selected: Vec<EntryId>,
    },
    /// Release the session only if its identity still matches.
    CloseSession(u64),
    /// Freeze a source-bound plan and queue a real heavy task.
    Extract {
        /// Immutable source session.
        session: u64,
        /// Explicit selected output intent.
        options: crate::extraction::Options,
    },
    /// Validate a selected target and private staging primitives on the metadata worker.
    CheckTarget {
        /// Current input session, used to protect the source archive.
        session: u64,
        /// Explicit selected existing directory.
        path: String,
    },
    /// Queue a whole-archive creation independent of an open archive session.
    Create(crate::creation::Options),
    /// Read nonrecursive selection sizes for display, never creation authority.
    InspectInputs(Vec<String>),
    /// Return the current session identity without filesystem work.
    Snapshot,
}
impl Command {
    fn bytes(&self) -> usize {
        match self {
            Self::Open(path) | Self::CheckTarget { path, .. } => path.len(),
            Self::Browse {
                directory, text, ..
            } => directory.len() + text.len(),
            Self::Preflight { selected, .. } => selected.len() * 8,
            Self::Extract { options, .. } => options.bytes(),
            Self::Create(options) => options.bytes(),
            Self::InspectInputs(paths) => paths
                .iter()
                .map(|p| p.len() + std::mem::size_of::<String>())
                .sum(),
            _ => 0,
        }
    }
    fn bounded(&self) -> bool {
        match self {
            Self::Open(path) => !path.is_empty() && path.len() <= 4096 && !path.contains('\0'),
            Self::CheckTarget { path, .. } => {
                !path.is_empty()
                    && path.len() <= 4096
                    && !path.contains('\0')
                    && Path::new(path).is_absolute()
            }
            Self::Browse {
                directory,
                text,
                sort,
                ..
            } => directory.len() <= 21 && text.len() <= 1024 && *sort <= 12,
            Self::Preflight { selected, .. } => !selected.is_empty() && selected.len() <= 128,
            Self::Extract { options, .. } => options.bounded(),
            Self::Create(options) => options.bounded(),
            Self::InspectInputs(paths) => {
                !paths.is_empty()
                    && paths.len() <= 256
                    && paths.iter().all(|p| {
                        !p.is_empty()
                            && p.len() <= 4096
                            && !p.contains('\0')
                            && Path::new(p).is_absolute()
                    })
            }
            _ => true,
        }
    }
}
/// A display row has no raw filename or output capability.
#[derive(Debug)]
pub struct Row {
    /// Physical identity, retained through view filtering.
    pub id: u64,
    /// Nonzero only for an implicit directory; id is then zero, never an EntryId.
    pub navigation_id: u64,
    /// Escaped display text.
    pub name: String,
    /// Declared original bytes.
    pub size: u64,
    /// Declared compressed bytes.
    pub packed_size: u64,
    /// ZIP method, not an assertion of decode support.
    pub method: u16,
    /// ZIP local DOS date/time without an inferred timezone.
    pub modified_dos: u32,
    /// Explicit directory marker.
    pub directory: bool,
    /// Encryption metadata flag.
    pub encrypted: bool,
    /// Metadata requiring review.
    pub warning: bool,
}
/// Advisory selection metadata. Creation always takes a new frozen input tree.
#[derive(Debug)]
pub struct InputInfo {
    /// An ordinary file size is available; directory totals are deliberately unknown.
    pub known: bool,
    /// Selected leaf is a directory; no descendants were scanned.
    pub directory: bool,
    /// Current ordinary-file length, valid only when known.
    pub bytes: u64,
}
/// The sole payload event for one accepted request. Finished follows it exactly once.
#[derive(Debug)]
pub enum Payload {
    /// Metadata in the same order as the bounded selected input paths.
    Inputs(Vec<InputInfo>),
    /// Immutable bounded page with session and offset identity.
    Page {
        /// Session identity checked before acting.
        session: u64,
        /// Complete query result identity.
        key: QueryKey,
        /// Offset within the immutable result.
        offset: u64,
        /// Listing rows matching this query, including implicit navigation directories.
        total: u64,
        /// Complete physical record count, independent of the current directory.
        archive_total: u64,
        /// Container determined from source bytes.
        format: xdzip_codec_api::Format,
        /// Host-derived same-folder name, revalidated by the output grant.
        folder_name: String,
        /// At most 128 owned display rows.
        rows: Vec<Row>,
        /// Complete bounded navigation, sent only when opening a new session.
        navigation: Option<Vec<crate::navigation::Directory>>,
        /// The independent navigation budget was exceeded; flat browsing remains available.
        navigation_limited: bool,
    },
    /// Bounded metadata report.
    Preflight(xdzip_extract_plan::Report),
    /// Current session, or zero when no session is open.
    Snapshot(u64),
    /// Request terminal; an empty error code means success.
    Finished(&'static str),
    /// All accepted requests drained, archive dropped, worker joined by receiver.
    ShutdownReady,
    /// One accepted heavy task identity, returned before its request terminal.
    TaskAccepted(u64),
    /// Target staging check succeeded; number of untouched possible recovery workspaces.
    TargetChecked(u64),
    /// Latest coalesced actual task state, independent of query request IDs.
    Task(crate::extraction::Snapshot),
    /// Worker panicked; the service is abandoned and pending requests fail.
    RuntimeFault,
}
impl Payload {
    /// Count retained string bytes for end-to-end mailbox admission (containers separately bounded).
    pub fn bytes(&self) -> usize {
        match self {
            Self::Page {
                rows,
                navigation,
                folder_name,
                ..
            } => {
                folder_name.len()
                    + rows.iter().map(|row| row.name.len()).sum::<usize>()
                    + navigation
                        .iter()
                        .flatten()
                        .map(|node| node.name.len() + node.path.len())
                        .sum::<usize>()
            }
            Self::Preflight(report) => report.findings.len() * 64,
            Self::Inputs(rows) => rows.len() * std::mem::size_of::<InputInfo>(),
            _ => 0,
        }
    }
}
/// Receiver-delivered sequence is monotonic across payload, terminal and control events.
#[derive(Debug)]
pub struct Event {
    /// Delivery sequence, never reused within a service.
    pub sequence: u64,
    /// Accepted request identity; ShutdownReady uses the idempotent shutdown request ID.
    pub request: u64,
    /// Owned event data.
    pub payload: Payload,
}
/// Immediate command rejection. No ID is consumed when rejected.
#[derive(Debug, PartialEq, Eq)]
pub enum Rejection {
    /// A bounded queue is full or admission mutex is briefly contended.
    Busy,
    /// Shutdown or receiver loss already closed admission.
    Closed,
    /// Command payload exceeds the public bound.
    Invalid,
    /// Request sequence has no remaining representable identity.
    Exhausted,
}
struct Pending {
    id: u64,
    command: Command,
    control: Arc<Cancellation>,
}
struct State {
    commands: VecDeque<Pending>,
    command_bytes: usize,
    tasks: tasks::Queue,
    task_turn: bool,
    events: VecDeque<(u64, Payload)>,
    controls: VecDeque<(u64, Payload)>,
    outstanding: BTreeMap<u64, Arc<Cancellation>>,
    last_id: u64,
    sequence: u64,
    bytes: usize,
    shutdown: Option<u64>,
    receiver_alive: bool,
    worker_done: bool,
    shutdown_delivered: bool,
    failed: bool,
    fault_pending: bool,
    control_turn: bool,
}
impl Default for State {
    fn default() -> Self {
        Self {
            commands: VecDeque::with_capacity(MAX_OUTSTANDING),
            command_bytes: 0,
            tasks: tasks::Queue::default(),
            task_turn: false,
            events: VecDeque::with_capacity(256),
            controls: VecDeque::with_capacity(MAX_OUTSTANDING),
            outstanding: BTreeMap::new(),
            last_id: 0,
            sequence: 0,
            bytes: 0,
            shutdown: None,
            receiver_alive: true,
            worker_done: false,
            shutdown_delivered: false,
            failed: false,
            fault_pending: false,
            control_turn: true,
        }
    }
}
struct Shared {
    state: Mutex<State>,
    changed: Condvar,
    admission_closed: AtomicBool,
}
impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock().expect("runtime mutex poisoned")
    }
    fn stop(&self, disconnect: bool) -> u64 {
        // This control lane never waits for data capacity. Critical sections contain no I/O,
        // callbacks or archive work. At most 128 atomic cancellation stores occur here.
        let mut state = self.lock();
        let id = match state.shutdown {
            Some(id) => id,
            None => {
                // Ordinary admission reserves the last ID for this mandatory control signal.
                state.last_id += 1;
                let id = state.last_id;
                state.shutdown = Some(id);
                id
            }
        };
        // A closed service must not appear transiently busy while workers finish.
        // Publish closure while holding the same lock as the authoritative state.
        self.admission_closed.store(true, Ordering::Release);
        for control in state.outstanding.values() {
            control.cancel();
        }
        state.tasks.cancel_all();
        if disconnect {
            state.receiver_alive = false;
        }
        self.changed.notify_all();
        id
    }
}
/// Exclusive producer handle. Send, but foreign callers must not concurrently call it.
pub struct Client(Arc<Shared>);
/// Exclusive receiver. Blocking receive/join belongs on the receiver thread, never the GUI.
pub struct Receiver {
    shared: Arc<Shared>,
    worker: Option<JoinHandle<()>>,
    task_worker: Option<JoinHandle<()>>,
}

/// Start one product worker and return independently owned command and receive handles.
pub fn start() -> std::io::Result<(Client, Receiver)> {
    let mut archive = None;
    spawn_with_shared(move |command, control, shared| {
        let result = execute(&mut archive, command, control, shared);
        if matches!(result, Err("source_changed")) {
            archive = None;
        }
        result
    })
}
#[cfg(test)]
fn spawn<F>(mut handler: F) -> std::io::Result<(Client, Receiver)>
where
    F: FnMut(Command, &Cancellation) -> Result<Payload, &'static str> + Send + 'static,
{
    spawn_with_shared(move |command, control, _| handler(command, control))
}
fn spawn_with_shared<F>(mut handler: F) -> std::io::Result<(Client, Receiver)>
where
    F: FnMut(Command, &Cancellation, &Arc<Shared>) -> Result<Payload, &'static str>
        + Send
        + 'static,
{
    let shared = Arc::new(Shared {
        state: Mutex::new(State::default()),
        changed: Condvar::new(),
        admission_closed: AtomicBool::new(false),
    });
    let task_shared = Arc::clone(&shared);
    let task_worker = thread::Builder::new()
        .name("xdzip-extract".into())
        .spawn(move || tasks::worker(task_shared))?;
    let worker_shared = Arc::clone(&shared);
    let worker = thread::Builder::new()
        .name("xdzip-runtime".into())
        .spawn(move || {
            loop {
                let pending = {
                    let mut state = worker_shared.lock();
                    loop {
                        if let Some(pending) = state.commands.pop_front() {
                            state.command_bytes -= pending.command.bytes();
                            break Some(pending);
                        }
                        if state.shutdown.is_some() || !state.receiver_alive {
                            break None;
                        }
                        state = worker_shared
                            .changed
                            .wait(state)
                            .expect("runtime mutex poisoned");
                    }
                };
                let Some(pending) = pending else { break };
                let result = if pending.control.check().is_err() {
                    Err("cancelled")
                } else {
                    match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        handler(pending.command, &pending.control, &worker_shared)
                    })) {
                        Ok(result) => result,
                        Err(_) => {
                            worker_shared.stop(false);
                            let mut state = worker_shared.lock();
                            state.failed = true;
                            state.fault_pending = true;
                            worker_shared.changed.notify_all();
                            Err("runtime_unavailable")
                        }
                    }
                };
                let (mut payload, mut error) = match result {
                    Ok(payload) => (Some(payload), ""),
                    Err(error) => (None, error),
                };
                let bytes = payload.as_ref().map_or(0, Payload::bytes);
                if bytes > MAX_EVENT_BYTES {
                    payload = None;
                    error = "resource_limit";
                }
                let mut state = worker_shared.lock();
                while payload.is_some()
                    && state.bytes + bytes > MAX_EVENT_BYTES
                    && state.receiver_alive
                    && pending.control.check().is_ok()
                {
                    state = worker_shared
                        .changed
                        .wait(state)
                        .expect("runtime mutex poisoned");
                }
                if !state.receiver_alive {
                    break;
                }
                if pending.control.check().is_err() {
                    payload = None;
                    error = "cancelled";
                }
                if state.failed {
                    payload = None;
                    error = "runtime_unavailable";
                }
                if let Some(payload) = payload {
                    state.bytes += bytes;
                    state.events.push_back((pending.id, payload));
                }
                // Each outstanding request reserves two event slots; terminal cannot be crowded out.
                state
                    .events
                    .push_back((pending.id, Payload::Finished(error)));
                worker_shared.changed.notify_all();
            }
            // Drop session/source and handler captures before acknowledging worker completion.
            drop(handler);
            let mut state = worker_shared.lock();
            state.worker_done = true;
            worker_shared.changed.notify_all();
        });
    let worker = match worker {
        Ok(worker) => worker,
        Err(error) => {
            shared.stop(false);
            return Err(error);
        }
    };
    Ok((
        Client(Arc::clone(&shared)),
        Receiver {
            shared,
            worker: Some(worker),
            task_worker: Some(task_worker),
        },
    ))
}
impl Client {
    /// Nonblocking normal admission: at most 128 accepted requests, bounded strings/IDs.
    pub fn submit(&self, command: Command) -> Result<u64, Rejection> {
        if !command.bounded() {
            return Err(Rejection::Invalid);
        }
        let bytes = command.bytes();
        if self.0.admission_closed.load(Ordering::Acquire) {
            return Err(Rejection::Closed);
        }
        let mut state = match self.0.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) => {
                return Err(if self.0.admission_closed.load(Ordering::Acquire) {
                    Rejection::Closed
                } else {
                    Rejection::Busy
                });
            }
            Err(TryLockError::Poisoned(_)) => panic!("runtime mutex poisoned"),
        };
        if state.shutdown.is_some() || !state.receiver_alive {
            return Err(Rejection::Closed);
        }
        if state.outstanding.len() >= MAX_OUTSTANDING
            || state.command_bytes + bytes > MAX_EVENT_BYTES
        {
            return Err(Rejection::Busy);
        }
        if state.last_id >= u64::MAX - 1 {
            return Err(Rejection::Exhausted);
        }
        state.last_id += 1;
        let id = state.last_id;
        let control = Arc::new(Cancellation::default());
        state.outstanding.insert(id, Arc::clone(&control));
        state.command_bytes += bytes;
        state.commands.push_back(Pending {
            id,
            command,
            control,
        });
        self.0.changed.notify_all();
        Ok(id)
    }
    /// Independent cancellation lane. Acknowledgement is a separate terminal event;
    /// the target may already have completed and still retains its original terminal outcome.
    pub fn cancel(&self, target: u64) -> Result<u64, Rejection> {
        let mut state = self.0.lock();
        if state.shutdown.is_some() || !state.receiver_alive {
            return Err(Rejection::Closed);
        }
        if state.controls.len() >= MAX_OUTSTANDING {
            return Err(Rejection::Busy);
        }
        if state.last_id >= u64::MAX - 1 {
            return Err(Rejection::Exhausted);
        }
        state.last_id += 1;
        let id = state.last_id;
        let error = if let Some(control) = state.outstanding.get(&target) {
            control.cancel();
            ""
        } else {
            "stale_request"
        };
        state.controls.push_back((id, Payload::Finished(error)));
        self.0.changed.notify_all();
        Ok(id)
    }
    /// Idempotent out-of-band shutdown; repeated calls return the same control request ID.
    pub fn shutdown(&self) -> u64 {
        self.0.stop(false)
    }
}
impl Receiver {
    /// Blocking event receive, including the shutdown join barrier. No idle polling.
    pub fn receive(&mut self) -> Option<Event> {
        let mut state = self.shared.lock();
        loop {
            let item = if state.task_turn && !state.tasks.pending.is_empty() {
                state.task_turn = false;
                state.tasks.take_event().map(|p| (0, p))
            } else if state.fault_pending {
                state.fault_pending = false;
                Some((0, Payload::RuntimeFault))
            } else if state.control_turn && !state.controls.is_empty() || state.events.is_empty() {
                state.control_turn = false;
                state.controls.pop_front()
            } else {
                state.control_turn = true;
                state.events.pop_front()
            };
            let item = item.or_else(|| state.tasks.take_event().map(|p| (0, p)));
            if let Some((request, payload)) = item {
                state.task_turn = !matches!(payload, Payload::Task(_));
                state.bytes -= payload.bytes();
                if matches!(payload, Payload::Finished(_)) {
                    state.outstanding.remove(&request);
                }
                state.sequence = state
                    .sequence
                    .checked_add(1)
                    .expect("event sequence exhausted");
                let event = Event {
                    sequence: state.sequence,
                    request,
                    payload,
                };
                self.shared.changed.notify_all();
                return Some(event);
            }
            if state.worker_done && state.tasks.done {
                if state.shutdown_delivered {
                    return None;
                }
                state.shutdown_delivered = true;
                state.sequence = state
                    .sequence
                    .checked_add(1)
                    .expect("event sequence exhausted");
                let event = Event {
                    sequence: state.sequence,
                    request: state.shutdown.unwrap_or(0),
                    payload: Payload::ShutdownReady,
                };
                drop(state);
                if let Some(worker) = self.worker.take() {
                    worker.join().expect("runtime worker panicked");
                }
                if let Some(worker) = self.task_worker.take() {
                    worker.join().expect("task worker panicked");
                }
                return Some(event);
            }
            state = self
                .shared
                .changed
                .wait(state)
                .expect("runtime mutex poisoned");
        }
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.shutdown();
    }
}
impl Drop for Receiver {
    fn drop(&mut self) {
        // Abandonment cancels and wakes publication waits. Dropping JoinHandle detaches;
        // graceful ShutdownReady is the explicit joined path. Never join in a GUI destructor.
        self.shared.stop(true);
    }
}
fn execute(
    archive: &mut Option<Archive>,
    command: Command,
    control: &Cancellation,
    shared: &Arc<Shared>,
) -> Result<Payload, &'static str> {
    let fail = |error: xdzip_codec_api::ScanError| match error {
        xdzip_codec_api::ScanError::Io => "source_io",
        xdzip_codec_api::ScanError::Invalid => "invalid_archive",
        xdzip_codec_api::ScanError::Unsupported => "unsupported_feature",
        xdzip_codec_api::ScanError::Limit => "resource_limit",
        xdzip_codec_api::ScanError::Cancelled => "cancelled",
        xdzip_codec_api::ScanError::SourceChanged => "source_changed",
    };
    let (session, directory, text, offset, sort, query, generation, revision, replace_navigation) =
        match command {
            Command::InspectInputs(paths) => {
                let mut rows = Vec::with_capacity(paths.len());
                for path in paths {
                    control.check().map_err(|_| "cancelled")?;
                    // Keep links visible. These lengths are display-only and cannot
                    // authorize encoding, traversal, quota admission or publication.
                    let metadata = std::fs::symlink_metadata(path).ok();
                    rows.push(InputInfo {
                        known: metadata.as_ref().is_some_and(|m| m.is_file()),
                        directory: metadata.as_ref().is_some_and(|m| m.is_dir()),
                        bytes: metadata.filter(|m| m.is_file()).map_or(0, |m| m.len()),
                    });
                }
                return Ok(Payload::Inputs(rows));
            }
            Command::CheckTarget { session, path } => {
                let grant = if session == 0 {
                    xdzip_io::Grant::for_creation(Path::new(&path))
                } else {
                    let archive = archive
                        .as_ref()
                        .filter(|a| a.key().session.get() == session)
                        .ok_or("stale_session")?;
                    archive.source.validate().map_err(|_| "source_changed")?;
                    let id = archive.source.identity().map_err(|_| "source_changed")?;
                    xdzip_io::Grant::open(Path::new(&path), id)
                }
                .map_err(|e| match e {
                    xdzip_io::OutputError::UnsafeTarget => "unsafe_target",
                    _ => "output_io",
                })?;
                let leftovers = grant.leftover_count().map_err(|_| "output_io")?;
                let workspace = grant.workspace(0).map_err(|_| "output_io")?;
                workspace.cleanup().map_err(|_| "cleanup_error")?;
                return Ok(Payload::TargetChecked(leftovers as u64));
            }
            Command::Extract { session, options } => {
                let archive = archive
                    .as_ref()
                    .filter(|a| a.key().session.get() == session)
                    .ok_or("stale_session")?;
                let job = archive.extraction_job(options, control)?;
                return shared
                    .enqueue_task(tasks::Job::Extract(job))
                    .map(Payload::TaskAccepted);
            }
            Command::Create(options) => {
                return shared
                    .enqueue_task(tasks::Job::Create(crate::creation::Job { options }))
                    .map(Payload::TaskAccepted);
            }
            Command::Snapshot => {
                return Ok(Payload::Snapshot(
                    archive.as_ref().map_or(0, |a| a.key().session.get()),
                ));
            }
            Command::Open(path) => {
                *archive = None;
                let opened = Archive::open(Path::new(&path), control).map_err(fail)?;
                let session = opened.key().session.get();
                *archive = Some(opened);
                (session, String::new(), String::new(), 0, 0, 0, 1, 0, true)
            }
            Command::Browse {
                session,
                directory,
                text,
                offset,
                sort,
                query,
                generation,
                revision,
            } => (
                session, directory, text, offset, sort, query, generation, revision, false,
            ),
            Command::Preflight { session, selected } => {
                let archive = archive
                    .as_ref()
                    .filter(|a| a.key().session.get() == session)
                    .ok_or("stale_session")?;
                return archive
                    .preflight(&selected, control)
                    .map(Payload::Preflight)
                    .map_err(fail);
            }
            Command::CloseSession(session) => {
                if archive
                    .as_ref()
                    .is_none_or(|a| a.key().session.get() != session)
                {
                    return Err("stale_session");
                }
                *archive = None;
                return Ok(Payload::Snapshot(0));
            }
        };
    let archive = archive
        .as_mut()
        .filter(|a| a.key().session.get() == session)
        .ok_or("stale_session")?;
    let archive_total = archive.len() as u64;
    let format = archive.format();
    let folder_name = archive.output_folder().to_owned();
    let navigation = replace_navigation.then(|| archive.directories().unwrap_or(&[]).to_vec());
    let navigation_limited = archive.directories().is_none();
    let generation = if generation == 0 {
        archive
            .cached_query
            .as_ref()
            .map_or(Some(1), |cache| cache.key.generation.get().checked_add(1))
            .ok_or("resource_limit")?
    } else {
        generation
    };
    let expected = if query == 0 {
        None
    } else {
        Some(QueryKey {
            session: archive.key().session,
            query: QueryId::try_from(query).map_err(|_| "stale_query")?,
            generation: QueryGeneration::try_from(generation).map_err(|_| "stale_query")?,
            revision: SnapshotRevision::try_from(revision).map_err(|_| "stale_query")?,
        })
    };
    let page = archive
        .query_page(
            crate::query::QueryRequest {
                directory: &directory,
                text: &text,
                sort,
                offset,
                expected,
                generation,
            },
            control,
        )
        .map_err(|error| match error {
            crate::query::QueryError::Scan(error) => fail(error),
            crate::query::QueryError::Stale => "stale_query",
        })?;
    Ok(Payload::Page {
        session,
        key: page.key,
        offset,
        total: page.total as u64,
        archive_total,
        format,
        folder_name,
        navigation,
        navigation_limited,
        rows: page
            .rows
            .into_iter()
            .map(|listing| match listing {
                crate::listing::ListingRow::Physical(entry) => Row {
                    id: entry.id.get(),
                    navigation_id: 0,
                    name: entry.display_name.clone(),
                    size: entry.size,
                    packed_size: entry.packed_size,
                    method: entry.method,
                    modified_dos: entry.modified_dos,
                    directory: entry.directory,
                    encrypted: entry.encrypted,
                    warning: entry.warning,
                },
                crate::listing::ListingRow::Directory(node) => Row {
                    id: 0,
                    navigation_id: node.id,
                    name: node.path.clone(),
                    size: 0,
                    packed_size: 0,
                    method: 0,
                    modified_dos: 0,
                    directory: true,
                    encrypted: false,
                    warning: false,
                },
            })
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    fn submit(client: &Client, make: impl Fn() -> Command) -> u64 {
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            match client.submit(make()) {
                Ok(id) => return id,
                Err(Rejection::Busy) if Instant::now() < deadline => thread::yield_now(),
                other => panic!("unexpected admission: {other:?}"),
            }
        }
    }
    fn drain(mut receiver: Receiver) -> Vec<Event> {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut events = Vec::new();
            while let Some(event) = receiver.receive() {
                events.push(event);
            }
            tx.send(events).unwrap();
        });
        rx.recv_timeout(Duration::from_secs(5))
            .expect("shutdown/receiver must not deadlock")
    }
    #[test]
    fn advisory_input_sizes_are_ordered_nonrecursive_and_bounded() {
        let root = std::env::temp_dir().join(format!("xdzip-input-preview-{}", std::process::id()));
        std::fs::create_dir(&root).unwrap();
        struct Cleanup(std::path::PathBuf);
        impl Drop for Cleanup {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let _cleanup = Cleanup(root.clone());
        std::fs::write(root.join("file"), b"seven!!").unwrap();
        std::fs::write(root.join("empty"), b"").unwrap();
        std::fs::create_dir(root.join("directory")).unwrap();
        std::fs::write(root.join("directory/descendant"), b"not included").unwrap();
        let paths = ["file", "directory", "empty", "missing"]
            .map(|name| root.join(name).to_string_lossy().into_owned())
            .to_vec();
        #[cfg(unix)]
        let paths = {
            std::os::unix::fs::symlink(root.join("file"), root.join("link")).unwrap();
            let mut paths = paths;
            paths.push(root.join("link").to_string_lossy().into_owned());
            paths
        };
        let (client, mut receiver) = start().unwrap();
        assert_eq!(
            client.submit(Command::InspectInputs(vec![paths[0].clone(); 257])),
            Err(Rejection::Invalid)
        );
        assert_eq!(
            client.submit(Command::InspectInputs(vec!["relative".into()])),
            Err(Rejection::Invalid)
        );
        let id = submit(&client, || Command::InspectInputs(paths.clone()));
        let event = receiver.receive().unwrap();
        assert_eq!(event.request, id);
        let Payload::Inputs(rows) = event.payload else {
            panic!("expected selection metadata")
        };
        assert_eq!(rows.len(), paths.len());
        assert!(rows[0].known && !rows[0].directory);
        assert_eq!(rows[0].bytes, 7);
        assert!(!rows[1].known && rows[1].directory);
        assert_eq!(rows[1].bytes, 0);
        assert!(rows[2].known && rows[2].bytes == 0);
        for row in &rows[3..] {
            assert!(!row.known && !row.directory && row.bytes == 0);
        }
        let terminal = receiver.receive().unwrap();
        assert_eq!(terminal.request, id);
        assert!(matches!(terminal.payload, Payload::Finished("")));
        client.shutdown();
        drain(receiver);
        assert_eq!(std::fs::read(root.join("file")).unwrap(), b"seven!!");
        assert_eq!(
            std::fs::read(root.join("directory/descendant")).unwrap(),
            b"not included"
        );
    }
    #[test]
    fn full_admission_cancel_shutdown_and_exactly_once_terminals() {
        let (release, gate) = mpsc::channel();
        let mut first = true;
        let (client, receiver) = spawn(move |_, _| {
            if first {
                first = false;
                gate.recv().unwrap();
            }
            Ok(Payload::Snapshot(0))
        })
        .unwrap();
        let ids: Vec<_> = (0..MAX_OUTSTANDING)
            .map(|_| submit(&client, || Command::Snapshot))
            .collect();
        assert_eq!(client.submit(Command::Snapshot), Err(Rejection::Busy));
        let cancel = client.cancel(ids[0]).unwrap();
        let shutdown = client.shutdown();
        assert_eq!(client.shutdown(), shutdown);
        assert_eq!(client.submit(Command::Snapshot), Err(Rejection::Closed));
        let held = client.0.lock();
        assert_eq!(client.submit(Command::Snapshot), Err(Rejection::Closed));
        drop(held);
        release.send(()).unwrap();
        let events = drain(receiver);
        for id in ids.into_iter().chain([cancel]) {
            assert_eq!(
                events
                    .iter()
                    .filter(|e| e.request == id && matches!(e.payload, Payload::Finished(_)))
                    .count(),
                1
            );
        }
        assert!(matches!(
            events.last().unwrap().payload,
            Payload::ShutdownReady
        ));
        assert_eq!(events.last().unwrap().request, shutdown);
        assert!(
            events
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        );
    }
    #[test]
    fn byte_backpressure_does_not_block_control_or_shutdown() {
        let (client, receiver) = spawn(|_, _| {
            Ok(Payload::Page {
                key: QueryKey {
                    session: 1.try_into().unwrap(),
                    query: 1.try_into().unwrap(),
                    generation: 1.try_into().unwrap(),
                    revision: 1.try_into().unwrap(),
                },
                navigation: None,
                navigation_limited: false,
                session: 1,
                offset: 0,
                total: 1,
                archive_total: 1,
                format: xdzip_codec_api::Format::Zip,
                folder_name: "archive".into(),
                rows: vec![Row {
                    id: 1,
                    navigation_id: 0,
                    name: "x".repeat(MAX_EVENT_BYTES / 2 - "archive".len()),
                    size: 0,
                    packed_size: 0,
                    method: 0,
                    modified_dos: 0,
                    directory: false,
                    encrypted: false,
                    warning: false,
                }],
            })
        })
        .unwrap();
        let ids: Vec<_> = (0..3)
            .map(|_| submit(&client, || Command::Snapshot))
            .collect();
        let mut state = client.0.lock();
        let deadline = Instant::now() + Duration::from_secs(3);
        while state.bytes != MAX_EVENT_BYTES {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero());
            state = client.0.changed.wait_timeout(state, remaining).unwrap().0;
        }
        drop(state);
        client.cancel(ids[2]).unwrap();
        client.shutdown();
        let events = drain(receiver);
        assert!(events.iter().any(|event| event.request == ids[2]
            && matches!(event.payload, Payload::Finished("cancelled"))));
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e.payload, Payload::ShutdownReady))
                .count(),
            1
        );
    }
    #[test]
    fn receiver_loss_and_idle_shutdown_wake_without_polling() {
        let (client, receiver) = start().unwrap();
        drop(receiver);
        assert_eq!(client.submit(Command::Snapshot), Err(Rejection::Closed));
        let (client, receiver) = start().unwrap();
        drop(client);
        let events = drain(receiver);
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].payload, Payload::ShutdownReady));
    }
    #[test]
    fn contention_rejection_and_identifier_exhaustion_preserve_shutdown_slot() {
        let (client, receiver) = start().unwrap();
        {
            let mut state = client.0.lock();
            assert_eq!(client.submit(Command::Snapshot), Err(Rejection::Busy));
            state.last_id = u64::MAX - 1;
        }
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            let result = client.submit(Command::Snapshot);
            if result == Err(Rejection::Busy) && Instant::now() < deadline {
                thread::yield_now();
                continue;
            }
            assert_eq!(result, Err(Rejection::Exhausted));
            break;
        }
        assert_eq!(client.shutdown(), u64::MAX);
        assert_eq!(drain(receiver).last().unwrap().request, u64::MAX);
    }
    #[test]
    fn worker_fault_abandons_service_and_completes_pending_requests() {
        let (client, receiver) =
            spawn(|_, _| -> Result<Payload, &'static str> { panic!("injected worker fault") })
                .unwrap();
        let id = submit(&client, || Command::Snapshot);
        let events = drain(receiver);
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e.payload, Payload::RuntimeFault))
                .count(),
            1
        );
        assert!(
            events.iter().any(|e| e.request == id
                && matches!(e.payload, Payload::Finished("runtime_unavailable")))
        );
        assert!(matches!(
            events.last().unwrap().payload,
            Payload::ShutdownReady
        ));
    }
    #[test]
    fn real_commands_preserve_session_and_payload_before_terminal() {
        let (client, mut receiver) = start().unwrap();
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/zip/normal.zip")
            .to_string_lossy()
            .into_owned();
        let id = submit(&client, || Command::Open(path.clone()));
        let page = receiver.receive().unwrap();
        assert_eq!(page.request, id);
        let session = match page.payload {
            Payload::Page {
                session,
                rows,
                navigation,
                navigation_limited,
                ..
            } => {
                assert_eq!(rows.len(), 4);
                assert!(!navigation_limited);
                assert_eq!(navigation.unwrap().len(), 1);
                session
            }
            _ => panic!("expected page"),
        };
        assert!(matches!(
            receiver.receive().unwrap().payload,
            Payload::Finished("")
        ));
        submit(&client, || Command::Browse {
            query: 0,
            generation: 0,
            revision: 0,
            session,
            directory: "d1".into(),
            text: String::new(),
            offset: 0,
            sort: 2,
        });
        match receiver.receive().unwrap().payload {
            Payload::Page {
                navigation,
                total,
                rows,
                ..
            } => {
                assert!(navigation.is_none());
                assert_eq!(total, 1);
                assert_eq!(rows[0].id, 2);
            }
            _ => panic!("expected bounded directory query"),
        }
        assert!(matches!(
            receiver.receive().unwrap().payload,
            Payload::Finished("")
        ));
        assert_eq!(
            client.submit(Command::Browse {
                query: 0,
                generation: 0,
                revision: 0,
                session,
                directory: String::new(),
                text: String::new(),
                offset: 0,
                sort: 13
            }),
            Err(Rejection::Invalid)
        );
        let stale = submit(&client, || Command::Browse {
            query: 0,
            generation: 0,
            revision: 0,
            sort: 0,
            session: session + 1,
            directory: String::new(),
            text: String::new(),
            offset: 0,
        });
        let error = receiver.receive().unwrap();
        assert_eq!(error.request, stale);
        assert!(matches!(error.payload, Payload::Finished("stale_session")));
        submit(&client, || Command::CloseSession(session));
        assert!(matches!(
            receiver.receive().unwrap().payload,
            Payload::Snapshot(0)
        ));
        assert!(matches!(
            receiver.receive().unwrap().payload,
            Payload::Finished("")
        ));
        client.shutdown();
        drain(receiver);
    }
}
