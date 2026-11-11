use crate::{Error, package::Package};
use std::{
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};
use xdzip_plugin_protocol::Envelope;

pub(crate) struct Peer {
    child: Child,
    incoming: Option<Receiver<Result<Envelope, xdzip_plugin_protocol::Error>>>,
    outgoing: Option<SyncSender<Envelope>>,
    written: Receiver<bool>,
    workers: Vec<JoinHandle<()>>,
    pub isolation: String,
}
impl Peer {
    pub fn spawn(package: &Package) -> Result<Self, Error> {
        let mut command = crate::sandbox::command(&package.executable)?;
        command.env_clear().current_dir("/");
        crate::sandbox::configure(&mut command)?;
        Self::start(command, crate::sandbox::policy())
    }
    #[cfg(test)]
    pub fn trusted(command: Command) -> Result<Self, Error> {
        Self::start(command, "trusted-conformance")
    }
    fn start(mut command: Command, isolation: &str) -> Result<Self, Error> {
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| Error::Sandbox)?;
        let mut input = child.stdout.take().ok_or(Error::Io)?;
        let mut output = child.stdin.take().ok_or(Error::Io)?;
        let (tx, rx) = mpsc::sync_channel(4);
        let reader = thread::spawn(move || {
            loop {
                let frame = Envelope::read(&mut input);
                let failed = frame.is_err();
                if tx.send(frame).is_err() || failed {
                    break;
                }
            }
        });
        let (send, receive) = mpsc::sync_channel::<Envelope>(1);
        let (done, written) = mpsc::sync_channel(1);
        let writer = thread::spawn(move || {
            while let Ok(frame) = receive.recv() {
                let ok = frame.write(&mut output).is_ok();
                if done.send(ok).is_err() || !ok {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            incoming: Some(rx),
            outgoing: Some(send),
            written,
            workers: vec![reader, writer],
            isolation: isolation.into(),
        })
    }
    pub fn send(&mut self, frame: Envelope) -> Result<(), Error> {
        self.outgoing
            .as_ref()
            .ok_or(Error::Protocol)?
            .try_send(frame)
            .map_err(|_| Error::Limit)?;
        if self
            .written
            .recv_timeout(Duration::from_secs(2))
            .map_err(|_| Error::Limit)?
        {
            Ok(())
        } else {
            Err(Error::Protocol)
        }
    }
    pub fn receive(&mut self, deadline: Instant, cancel: &AtomicBool) -> Result<Envelope, Error> {
        loop {
            if cancel.load(Ordering::Acquire) {
                return Err(Error::Cancelled);
            }
            self.check(deadline)?;
            match self
                .incoming
                .as_ref()
                .ok_or(Error::Protocol)?
                .recv_timeout(Duration::from_millis(20))
            {
                Ok(value) => return value.map_err(Into::into),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(_) => return Err(Error::Protocol),
            }
        }
    }
    fn check(&mut self, deadline: Instant) -> Result<(), Error> {
        if Instant::now() >= deadline {
            return Err(Error::Limit);
        }
        if self.child.try_wait()?.is_none()
            && let Err(error) = crate::sandbox::check_memory(self.child.id(), &self.isolation)
            && self.child.try_wait()?.is_none()
        {
            return Err(error);
        }
        Ok(())
    }
    pub fn finish(&mut self) -> Result<(), Error> {
        self.outgoing.take();
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            self.check(deadline)?;
            if let Some(status) = self.child.try_wait()? {
                if !status.success() {
                    return Err(Error::Protocol);
                }
                // EOF is required after Shutdown. A second terminal or any late data fails.
                return match self
                    .incoming
                    .as_ref()
                    .ok_or(Error::Protocol)?
                    .recv_timeout(deadline.saturating_duration_since(Instant::now()))
                {
                    Ok(Err(xdzip_plugin_protocol::Error::Io)) => Ok(()),
                    _ => Err(Error::Protocol),
                };
            }
            thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Peer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.incoming.take();
        self.outgoing.take();
        // Kernel policies forbid descendants, so killing the peer closes every pipe end.
        for worker in self.workers.drain(..) {
            let _ = worker.join();
        }
    }
}
