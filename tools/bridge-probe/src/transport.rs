//! Bounded diagnostic transport. No archive command or filesystem access lives here.
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex, MutexGuard, TryLockError};

/// Maximum copied diagnostic text size; validated before allocating a queue item.
pub const MAX_PAYLOAD_BYTES: usize = 4096;

/// A rejected submission never consumes an identifier or queue slot.
#[derive(Debug, PartialEq, Eq)]
pub enum Rejection {
    /// Producer lock contention or no remaining queue capacity.
    Busy,
    /// Shutdown or receiver disconnection has already linearized.
    Closed,
    /// The UTF-8 representation exceeds the diagnostic payload budget.
    TooLarge,
    /// The monotonically increasing identifier space was exhausted.
    Exhausted,
}

/// An owned message; it remains valid after either transport handle is destroyed.
#[derive(Debug, PartialEq, Eq)]
pub struct Message {
    /// Nonzero identifier allocated only for accepted submissions.
    pub id: u64,
    /// Independent copy of the producer's bounded input.
    pub payload: String,
}

#[derive(Default)]
struct State {
    queue: VecDeque<Message>,
    closed: bool,
    last_id: u64,
}

struct Shared {
    state: Mutex<State>,
    ready: Condvar,
    capacity: usize,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, State> {
        // No callbacks, fallible indexing or user code run under this mutex.
        // Poisoning is an internal fault, not a recoverable transport state.
        self.state
            .lock()
            .expect("diagnostic transport mutex poisoned")
    }

    fn close(&self, discard: bool) {
        let mut state = self.lock();
        state.closed = true;
        if discard {
            state.queue.clear();
        }
        self.ready.notify_all();
    }
}

/// Exclusive producer. Move between threads if needed, but do not share calls.
/// Dropping it closes admission and wakes the receiver without discarding messages.
pub struct Client(Arc<Shared>);

/// Exclusive blocking receiver. Dropping it disconnects and discards queued probes.
/// It must run on a worker thread, never the Qt GUI thread.
pub struct Receiver(Arc<Shared>);

/// Create a bounded pair. Capacity is fixed for this lifetime and must be nonzero.
pub fn pair(capacity: usize) -> (Client, Receiver) {
    assert!(capacity > 0, "internal capacity must be nonzero");
    let shared = Arc::new(Shared {
        state: Mutex::new(State::default()),
        ready: Condvar::new(),
        capacity,
    });
    (Client(Arc::clone(&shared)), Receiver(shared))
}

impl Client {
    /// Attempt a bounded copy without waiting for queue space or mutex ownership.
    /// UTF-8 bytes (not characters) count toward the allocation budget.
    pub fn submit(&self, text: &str) -> Result<u64, Rejection> {
        if text.len() > MAX_PAYLOAD_BYTES {
            return Err(Rejection::TooLarge);
        }
        let mut state = match self.0.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::WouldBlock) => return Err(Rejection::Busy),
            Err(TryLockError::Poisoned(_)) => panic!("diagnostic transport mutex poisoned"),
        };
        if state.closed {
            return Err(Rejection::Closed);
        }
        if state.queue.len() == self.0.capacity {
            return Err(Rejection::Busy);
        }
        let id = state.last_id.checked_add(1).ok_or(Rejection::Exhausted)?;
        state.queue.push_back(Message {
            id,
            payload: text.to_owned(),
        });
        state.last_id = id;
        self.0.ready.notify_one();
        Ok(id)
    }

    /// Stop admission and wake a blocked receiver, even if the data queue is full.
    /// Accepted messages are drained before end-of-stream. Repeated calls are safe.
    pub fn shutdown(&self) {
        self.0.close(false);
    }
}

impl Receiver {
    /// Wait until an owned message or end-of-stream is available.
    /// Spurious wakeups are checked under the same mutex as close/admission.
    pub fn receive(&mut self) -> Option<Message> {
        let mut state = self.0.lock();
        loop {
            if let Some(message) = state.queue.pop_front() {
                return Some(message);
            }
            if state.closed {
                return None;
            }
            state = self
                .0
                .ready
                .wait(state)
                .expect("diagnostic transport mutex poisoned");
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
        self.0.close(true);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn full_queue_does_not_block_shutdown_or_lose_accepted_messages() {
        let (client, mut receiver) = pair(2);
        assert_eq!(client.submit("first"), Ok(1));
        assert_eq!(client.submit("second"), Ok(2));
        assert_eq!(client.submit("rejected"), Err(Rejection::Busy));
        client.shutdown();
        client.shutdown();
        assert_eq!(client.submit("late"), Err(Rejection::Closed));
        assert_eq!(receiver.receive().unwrap().payload, "first");
        assert_eq!(receiver.receive().unwrap().payload, "second");
        assert!(receiver.receive().is_none());
        assert!(receiver.receive().is_none());
    }

    #[test]
    fn mutex_contention_rejects_instead_of_waiting() {
        let (client, _receiver) = pair(2);
        let _guard = client.0.lock();
        assert_eq!(client.submit("never waits"), Err(Rejection::Busy));
    }

    #[test]
    fn payload_is_owned_and_budget_counts_utf8_bytes() {
        let (client, mut receiver) = pair(2);
        let mut original = "中文 🗜️ e\u{301}\0tail".to_owned();
        client.submit(&original).unwrap();
        original.clear();
        assert_eq!(
            receiver.receive().unwrap().payload,
            "中文 🗜️ e\u{301}\0tail"
        );
        assert_eq!(client.submit(&"a".repeat(MAX_PAYLOAD_BYTES)), Ok(2));
        assert_eq!(
            client.submit(&"中".repeat(MAX_PAYLOAD_BYTES / 3 + 1)),
            Err(Rejection::TooLarge)
        );
    }

    #[test]
    fn dropping_receiver_closes_admission() {
        let (client, receiver) = pair(1);
        client.submit("queued").unwrap();
        drop(receiver);
        assert_eq!(client.submit("late"), Err(Rejection::Closed));
    }

    #[test]
    fn close_wakes_a_receiver_without_polling() {
        let (client, mut receiver) = pair(1);
        let (started_tx, started_rx) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let worker = thread::spawn(move || {
            started_tx.send(()).unwrap();
            done_tx.send(receiver.receive()).unwrap();
        });
        started_rx.recv_timeout(Duration::from_secs(2)).unwrap();
        drop(client);
        assert!(
            done_rx
                .recv_timeout(Duration::from_secs(2))
                .unwrap()
                .is_none()
        );
        worker.join().unwrap();
    }

    #[test]
    fn identifier_exhaustion_never_wraps_or_enqueues() {
        let (client, mut receiver) = pair(1);
        client.0.lock().last_id = u64::MAX;
        assert_eq!(client.submit("overflow"), Err(Rejection::Exhausted));
        client.shutdown();
        assert!(receiver.receive().is_none());
    }
}
