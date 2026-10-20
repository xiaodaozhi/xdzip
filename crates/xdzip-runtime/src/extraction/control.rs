use super::{ConflictPolicy, Password};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Condvar, Mutex};
use xdzip_codec_api::Cancellation;

/// Independent task control. Cancellation never waits for filesystem work.
pub struct Control {
    pub(crate) cancel: Cancellation,
    gate: AtomicU8,
    answer: Mutex<Answer>,
    changed: Condvar,
}
#[derive(Default)]
struct Answer {
    prompt: u64,
    decision: Option<(ConflictPolicy, bool)>,
    password_prompt: bool,
    password: Option<Password>,
}
impl Default for Control {
    fn default() -> Self {
        Self {
            cancel: Cancellation::default(),
            gate: AtomicU8::new(0),
            answer: Mutex::new(Answer::default()),
            changed: Condvar::new(),
        }
    }
}
impl Control {
    /// Request cancellation. False means the task outcome is already fixed.
    pub fn cancel(&self) -> bool {
        loop {
            let old = self.gate.load(Ordering::Acquire);
            let new = match old {
                0 => 1,
                2 => 3,
                1 | 3 => break,
                _ => return false,
            };
            if self
                .gate
                .compare_exchange(old, new, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                break;
            }
        }
        self.cancel.cancel();
        // Synchronize wakeup with the waiter's check-to-wait interval; no I/O under this lock.
        let mut answer = self.answer.lock().expect("task control poisoned");
        answer.password = None;
        self.changed.notify_all();
        true
    }
    /// Respond only once to the latest prompt. Ask is not a valid resolution.
    pub fn decide(&self, prompt: u64, policy: ConflictPolicy, apply_all: bool) -> bool {
        if policy == ConflictPolicy::Ask || self.cancel.check().is_err() {
            return false;
        }
        let mut answer = self.answer.lock().expect("task control poisoned");
        if prompt == 0
            || answer.prompt != prompt
            || answer.password_prompt
            || answer.decision.is_some()
            || self.cancel.check().is_err()
            || self.gate.load(Ordering::Acquire) == 4
        {
            return false;
        }
        answer.decision = Some((policy, apply_all));
        self.changed.notify_all();
        true
    }
    pub(crate) fn arm(&self, id: u64) {
        let mut a = self.answer.lock().expect("task control poisoned");
        a.prompt = id;
        a.decision = None;
        a.password_prompt = false;
        a.password = None;
    }
    /// Submit only to an armed password prompt; rejected secrets are dropped and zeroed.
    pub fn respond_password(&self, prompt: u64, password: Password) -> bool {
        let mut answer = self.answer.lock().expect("task control poisoned");
        if prompt == 0
            || answer.prompt != prompt
            || !answer.password_prompt
            || answer.password.is_some()
            || self.cancel.check().is_err()
            || self.gate.load(Ordering::Acquire) == 4
        {
            return false;
        }
        answer.password = Some(password);
        self.changed.notify_all();
        true
    }
    pub(crate) fn arm_password(&self, id: u64) {
        let mut answer = self.answer.lock().expect("task control poisoned");
        answer.prompt = id;
        answer.decision = None;
        answer.password_prompt = true;
        answer.password = None;
    }
    pub(crate) fn wait_password(&self) -> Result<Password, &'static str> {
        let mut answer = self.answer.lock().expect("task control poisoned");
        loop {
            if self.cancel.check().is_err() {
                answer.prompt = 0;
                answer.password = None;
                return Err("cancelled");
            }
            if let Some(password) = answer.password.take() {
                answer.prompt = 0;
                return Ok(password);
            }
            answer = self.changed.wait(answer).expect("task control poisoned");
        }
    }
    pub(crate) fn wait(&self) -> Result<(ConflictPolicy, bool), &'static str> {
        let mut a = self.answer.lock().expect("task control poisoned");
        loop {
            if self.cancel.check().is_err() {
                a.prompt = 0;
                return Err("cancelled");
            }
            if let Some(result) = a.decision.take() {
                a.prompt = 0;
                return Ok(result);
            }
            a = self.changed.wait(a).expect("task control poisoned");
        }
    }
    pub(crate) fn begin_commit(&self) -> Result<(), &'static str> {
        self.gate
            .compare_exchange(0, 2, Ordering::AcqRel, Ordering::Acquire)
            .map(|_| ())
            .map_err(|_| "cancelled")
    }
    pub(crate) fn end_commit(&self, last: bool) -> Result<(), &'static str> {
        match self.gate.compare_exchange(
            2,
            if last { 4 } else { 0 },
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(()),
            Err(3) => {
                self.gate.store(1, Ordering::Release);
                Err("cancelled")
            }
            Err(_) => Err("cancelled"),
        }
    }
    pub(crate) fn mark_terminal(&self) {
        // Failure and cancellation also close control admission after cleanup.
        // Otherwise a late cancel could replace an undelivered terminal snapshot.
        self.gate.store(4, Ordering::Release);
        let mut answer = self.answer.lock().expect("task control poisoned");
        answer.prompt = 0;
        answer.password = None;
        answer.decision = None;
    }
    pub(crate) fn complete(&self) -> Result<(), &'static str> {
        match self
            .gate
            .compare_exchange(0, 4, Ordering::AcqRel, Ordering::Acquire)
        {
            Ok(_) | Err(4) => Ok(()),
            _ => Err("cancelled"),
        }
    }
}
