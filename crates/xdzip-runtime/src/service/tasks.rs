//! Task lane shares only short state locks and the receiver wakeup with metadata commands.
use super::*;
use crate::extraction::{self, ConflictPolicy, Control, Snapshot, Status};

pub(super) enum Job {
    Extract(extraction::Job),
    Create(crate::creation::Job),
}
impl Job {
    fn retained_bytes(&self) -> usize {
        match self {
            Self::Extract(j) => j.retained_bytes(),
            Self::Create(j) => j.retained_bytes(),
        }
    }
    fn initial(&self, id: u64) -> Snapshot {
        match self {
            Self::Extract(j) => extraction::initial(j, id),
            Self::Create(j) => crate::creation::initial(j, id),
        }
    }
    fn run(self, id: u64, control: &Control, publish: &mut dyn FnMut(Snapshot)) {
        match self {
            Self::Extract(j) => extraction::run(j, id, control, publish),
            Self::Create(j) => crate::creation::run(j, id, control, publish),
        }
    }
}
#[derive(Default)]
pub(super) struct Queue {
    next_id: u64,
    retained: BTreeMap<u64, usize>,
    jobs: VecDeque<(u64, Job, Arc<Control>)>,
    controls: BTreeMap<u64, Arc<Control>>,
    latest: BTreeMap<u64, Snapshot>,
    pub(super) pending: BTreeMap<u64, Snapshot>,
    pub done: bool,
}
impl Queue {
    pub fn cancel_all(&self) {
        for c in self.controls.values() {
            c.cancel();
        }
    }
    pub fn take_event(&mut self) -> Option<Payload> {
        let (id, snapshot) = self.pending.pop_first()?;
        if snapshot.status.terminal() {
            self.controls.remove(&id);
            self.retained.remove(&id);
            self.latest.remove(&id);
        }
        Some(Payload::Task(snapshot))
    }
}
impl Shared {
    pub(super) fn enqueue_task(&self, job: Job) -> Result<u64, &'static str> {
        let bytes = job.retained_bytes();
        let mut state = self.lock();
        if state.shutdown.is_some() || !state.receiver_alive {
            return Err("runtime_closed");
        }
        if bytes > 64 * 1024 * 1024 {
            return Err("resource_limit");
        }
        if state.tasks.controls.len() >= extraction::MAX_TASKS
            || state.tasks.retained.values().sum::<usize>() + bytes > 64 * 1024 * 1024
        {
            return Err("task_busy");
        }
        let id = state.tasks.next_id.checked_add(1).ok_or("resource_limit")?;
        state.tasks.next_id = id;
        state.tasks.retained.insert(id, bytes);
        let control = Arc::new(Control::default());
        let snapshot = job.initial(id);
        state.tasks.controls.insert(id, Arc::clone(&control));
        state.tasks.latest.insert(id, snapshot.clone());
        state.tasks.pending.insert(id, snapshot);
        state.tasks.jobs.push_back((id, job, control));
        self.changed.notify_all();
        Ok(id)
    }
    fn publish_task(&self, mut snapshot: Snapshot) {
        let mut state = self.lock();
        if !state.receiver_alive {
            return;
        }
        if !snapshot.status.terminal()
            && snapshot.status != Status::Cleaning
            && state
                .tasks
                .controls
                .get(&snapshot.id)
                .is_some_and(|c| c.cancel.check().is_err())
        {
            snapshot.status = Status::Cancelling;
            snapshot.prompt = None;
        }
        // At most 32 tasks, bounded 128-record display history (1024-byte names).
        // Arc-backed records are shared by latest/pending snapshots; chunks do
        // not append events or history. No secret ever enters a snapshot.
        state.tasks.latest.insert(snapshot.id, snapshot.clone());
        state.tasks.pending.insert(snapshot.id, snapshot);
        self.changed.notify_all();
    }
}
pub(super) fn worker(shared: Arc<Shared>) {
    loop {
        let next = {
            let mut state = shared.lock();
            loop {
                if let Some(job) = state.tasks.jobs.pop_front() {
                    break Some(job);
                }
                if state.shutdown.is_some() || !state.receiver_alive {
                    break None;
                }
                state = shared.changed.wait(state).expect("runtime mutex poisoned");
            }
        };
        let Some((id, job, control)) = next else {
            break;
        };
        let mut fallback = job.initial(id);
        if std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut last_progress = std::time::Instant::now();
            job.run(id, &control, &mut |s| {
                if matches!(
                    s.status,
                    Status::Running | Status::Committing | Status::Verifying
                ) {
                    if last_progress.elapsed() < std::time::Duration::from_millis(100) {
                        return;
                    }
                    last_progress = std::time::Instant::now();
                }
                // Prompt, cancellation, cleanup and final outcome bypass progress throttling.
                shared.publish_task(s);
            });
        }))
        .is_err()
        {
            shared.stop(false);
            fallback = shared
                .lock()
                .tasks
                .latest
                .get(&id)
                .cloned()
                .unwrap_or(fallback);
            fallback.status = Status::Failed;
            fallback.outcome_uncertain = true;
            fallback.error = "task_abandoned".into();
            // Unwind drops private stages. A panic is not proof of successful cleanup.
            fallback.cleanup_warning = true;
            control.mark_terminal();
            shared.publish_task(fallback);
            let mut state = shared.lock();
            state.failed = true;
            state.fault_pending = true;
        }
    }
    let mut state = shared.lock();
    state.tasks.done = true;
    shared.changed.notify_all();
}
impl Client {
    /// Independent task cancellation; accepted tasks survive closing their source session.
    pub fn cancel_task(&self, id: u64) -> Result<(), Rejection> {
        let mut state = self.0.lock();
        if state
            .tasks
            .latest
            .get(&id)
            .is_some_and(|s| s.status.terminal())
        {
            return Ok(());
        }
        let Some(control) = state.tasks.controls.get(&id) else {
            return Err(Rejection::Invalid);
        };
        if control.cancel()
            && let Some(mut current) = state.tasks.latest.get(&id).cloned()
        {
            current.status = Status::Cancelling;
            current.prompt = None;
            state.tasks.latest.insert(id, current.clone());
            state.tasks.pending.insert(id, current);
            self.0.changed.notify_all();
        }
        Ok(())
    }
    /// Resolve the currently waiting prompt, rejecting stale or duplicate responses.
    pub fn decide_task(
        &self,
        id: u64,
        prompt: u64,
        policy: ConflictPolicy,
        all: bool,
    ) -> Result<(), Rejection> {
        let state = self.0.lock();
        let control = state.tasks.controls.get(&id).ok_or(Rejection::Invalid)?;
        if control.decide(prompt, policy, all) {
            Ok(())
        } else {
            Err(Rejection::Invalid)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};
    fn submit(client: &Client, mut command: impl FnMut() -> Command) -> u64 {
        let end = Instant::now() + Duration::from_secs(5);
        loop {
            match client.submit(command()) {
                Ok(id) => return id,
                Err(Rejection::Busy) if Instant::now() < end => thread::yield_now(),
                other => panic!("{other:?}"),
            }
        }
    }
    #[test]
    fn waiting_task_allows_browsing_and_session_close_then_shutdown_wakes_prompt() {
        let target =
            std::env::temp_dir().join(format!("xdzip-task-service-{}", std::process::id()));
        std::fs::create_dir(&target).unwrap();
        let input =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/extract/stored.zip");
        let a = Archive::open(&input, &Cancellation::default()).unwrap();
        std::fs::write(
            target.join(std::str::from_utf8(&a.rows[0].raw_name).unwrap()),
            b"keep",
        )
        .unwrap();
        let (client, mut receiver) = start().unwrap();
        let (tx, rx) = mpsc::channel();
        let recv = thread::spawn(move || {
            while let Some(e) = receiver.receive() {
                tx.send(e).unwrap();
            }
        });
        let open = submit(&client, || Command::Open(input.to_string_lossy().into()));
        let mut session = 0;
        loop {
            let e = rx.recv_timeout(Duration::from_secs(5)).unwrap();
            if let Payload::Page { session: s, .. } = e.payload {
                session = s;
            }
            if e.request == open && matches!(e.payload, Payload::Finished("")) {
                break;
            }
        }
        assert_ne!(session, 0);
        let options = extraction::Options {
            target: target.to_string_lossy().into(),
            all: true,
            selected: vec![],
            directory: String::new(),
            create_folder: false,
            conflict: ConflictPolicy::Ask,
            allow_high_ratio: false,
        };
        let extract = submit(&client, || Command::Extract {
            session,
            options: options.clone(),
        });
        let mut task = 0;
        let mut waiting = false;
        let mut accepted = false;
        while !waiting || !accepted {
            let e = rx.recv_timeout(Duration::from_secs(5)).unwrap();
            match e.payload {
                Payload::TaskAccepted(id) if e.request == extract => {
                    task = id;
                    accepted = true;
                }
                Payload::Task(s) if s.status == Status::Waiting => waiting = true,
                _ => {}
            }
        }
        let close = submit(&client, || Command::CloseSession(session));
        loop {
            let e = rx.recv_timeout(Duration::from_secs(5)).unwrap();
            if e.request == close && matches!(e.payload, Payload::Finished("")) {
                break;
            }
        }
        // No source session remains, but task cancellation is still independently addressable.
        assert!(client.cancel_task(task).is_ok());
        let shutdown = client.shutdown();
        let mut terminal = 0;
        let mut last = 0;
        loop {
            let e = rx.recv_timeout(Duration::from_secs(5)).unwrap();
            assert!(e.sequence > last);
            last = e.sequence;
            if let Payload::Task(ref s) = e.payload
                && s.id == task
                && s.status.terminal()
            {
                terminal += 1;
                assert_eq!(s.status, Status::Cancelled);
            }
            if matches!(e.payload, Payload::ShutdownReady) {
                assert_eq!(e.request, shutdown);
                break;
            }
        }
        recv.join().unwrap();
        assert_eq!(terminal, 1);
        assert_eq!(std::fs::read_dir(&target).unwrap().count(), 1);
        std::fs::remove_dir_all(target).unwrap();
    }
    #[test]
    fn task_queue_counts_undelivered_terminals_and_command_bytes_are_bounded() {
        let shared = Shared {
            state: Mutex::new(State::default()),
            changed: Condvar::new(),
            admission_closed: AtomicBool::new(false),
        };
        let input =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/extract/stored.zip");
        let a = Archive::open(&input, &Cancellation::default()).unwrap();
        let options = extraction::Options {
            target: std::env::temp_dir().to_string_lossy().into(),
            all: true,
            selected: vec![],
            directory: String::new(),
            create_folder: false,
            conflict: ConflictPolicy::Ask,
            allow_high_ratio: false,
        };
        for _ in 0..extraction::MAX_TASKS {
            shared
                .enqueue_task(Job::Extract(
                    a.extraction_job(options.clone(), &Cancellation::default())
                        .unwrap(),
                ))
                .unwrap();
        }
        assert_eq!(
            shared.enqueue_task(Job::Extract(
                a.extraction_job(options, &Cancellation::default()).unwrap()
            )),
            Err("task_busy")
        );
        let state = shared.lock();
        assert_eq!(state.tasks.pending.len(), extraction::MAX_TASKS);
        assert!(state.tasks.retained.values().sum::<usize>() <= 64 * 1024 * 1024);
    }
}
