//! Diagnostic-only cxx boundary for toolchain and transport admission tests.
//! This is not the product's archive command protocol or plugin ABI.
mod transport;

// cxx generates ABI glue containing unsafe internals. Handwritten
// transport code stays under the workspace's unsafe_code=deny policy.
#[allow(unsafe_code)]
#[cxx::bridge(namespace = "xdzip::probe")]
mod ffi {
    /// Typed admission result; only Accepted carries a nonzero request_id.
    enum Admission {
        Accepted,
        Busy,
        Closed,
        TooLarge,
        Exhausted,
    }
    /// Immediate, nonblocking diagnostic submission outcome.
    struct Submission {
        status: Admission,
        request_id: u64,
    }
    /// End-of-stream is explicit; an accepted empty string is still a message.
    struct Event {
        end: bool,
        request_id: u64,
        payload: String,
    }
    extern "Rust" {
        type Probe;
        type CommandClient;
        type EventReceiver;
        fn new_probe(capacity: u32) -> Result<Box<Probe>>;
        fn take_client(self: &mut Probe) -> Result<Box<CommandClient>>;
        fn take_receiver(self: &mut Probe) -> Result<Box<EventReceiver>>;
        fn submit(self: &CommandClient, text: &str) -> Submission;
        fn shutdown(self: &CommandClient);
        fn next_event(self: &mut EventReceiver) -> Event;
    }
}

/// Temporary owner from which each unique handle can be taken exactly once.
pub struct Probe {
    client: Option<CommandClient>,
    receiver: Option<EventReceiver>,
}

/// Exclusive producer wrapper; safe to move, never concurrently call from C++.
pub struct CommandClient(transport::Client);
/// Exclusive receiver wrapper; blocking calls belong on the receiving thread.
pub struct EventReceiver(transport::Receiver);

/// Construct a bounded diagnostic pair (1–128 slots), without starting a thread.
/// Invalid capacities become cxx exceptions containing no input data or paths.
pub fn new_probe(capacity: u32) -> Result<Box<Probe>, &'static str> {
    if !(1..=128).contains(&capacity) {
        return Err("probe capacity must be between 1 and 128");
    }
    let (client, receiver) = transport::pair(capacity as usize);
    Ok(Box::new(Probe {
        client: Some(CommandClient(client)),
        receiver: Some(EventReceiver(receiver)),
    }))
}

impl Probe {
    /// Transfer producer ownership; repeated transfer returns a cxx exception.
    pub fn take_client(&mut self) -> Result<Box<CommandClient>, &'static str> {
        self.client
            .take()
            .map(Box::new)
            .ok_or("command client already taken")
    }
    /// Transfer receiver ownership; repeated transfer returns a cxx exception.
    pub fn take_receiver(&mut self) -> Result<Box<EventReceiver>, &'static str> {
        self.receiver
            .take()
            .map(Box::new)
            .ok_or("event receiver already taken")
    }
}

impl CommandClient {
    /// Return typed admission without queue-space waits. At most 4096 bytes copy.
    pub fn submit(&self, text: &str) -> ffi::Submission {
        use ffi::Admission;
        match self.0.submit(text) {
            Ok(request_id) => ffi::Submission {
                status: Admission::Accepted,
                request_id,
            },
            Err(error) => ffi::Submission {
                status: match error {
                    transport::Rejection::Busy => Admission::Busy,
                    transport::Rejection::Closed => Admission::Closed,
                    transport::Rejection::TooLarge => Admission::TooLarge,
                    transport::Rejection::Exhausted => Admission::Exhausted,
                },
                request_id: 0,
            },
        }
    }
    /// Stop admission independently of queue capacity, retaining accepted events.
    pub fn shutdown(&self) {
        self.0.shutdown();
    }
}

impl EventReceiver {
    /// Drain an owned message or block until data/shutdown. Never run on GUI thread.
    pub fn next_event(&mut self) -> ffi::Event {
        match self.0.receive() {
            Some(message) => ffi::Event {
                end: false,
                request_id: message.id,
                payload: message.payload,
            },
            None => ffi::Event {
                end: true,
                request_id: 0,
                payload: String::new(),
            },
        }
    }
}
