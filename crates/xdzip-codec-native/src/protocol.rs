// crates/xdzip-codec-native/src/protocol.rs

//! Bounded duplex worker transport. No archive or output pathname crosses this boundary.
use std::io::{Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;
use xdzip_codec_api::Cancellation;

pub const MAX_FRAME: usize = 1024 * 1024;
pub const CHUNK: usize = 64 * 1024;
pub const START: u8 = 1;
pub const READ: u8 = 10;
pub const INPUT: u8 = 11;
pub const DATA: u8 = 12;
pub const ROW: u8 = 13;
pub const ERROR: u8 = 14;
pub const DONE: u8 = 15;
pub const PROGRESS: u8 = 16;
pub const WORK: u8 = 17;
pub const ENTRY_DONE: u8 = 18;
pub const WRITE_AT: u8 = 19;
pub const BYTES: u8 = 20;
pub const ACK: u8 = 21;
pub const NEXT: u8 = 22;
pub const PRECHARGED_DATA: u8 = 23;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Fault {
    Io,
    // Host-only frozen-source failure; never an additional wire identifier.
    SourceChanged,
    Invalid,
    Unsupported,
    Limit,
    Cancelled,
    PasswordRequired,
    WrongPassword,
    PasswordOrIntegrity,
    Output,
    MissingVolume,
}
impl Fault {
    pub fn from_wire(code: u8) -> Self {
        match code {
            1 => Self::Io,
            3 => Self::Unsupported,
            4 => Self::Limit,
            5 => Self::Cancelled,
            6 => Self::PasswordRequired,
            7 => Self::WrongPassword,
            8 => Self::Output,
            9 => Self::MissingVolume,
            10 => Self::PasswordOrIntegrity,
            _ => Self::Invalid,
        }
    }
}

pub struct Worker {
    child: Child,
    input: ChildStdin,
    receiver: Option<Receiver<Result<Vec<u8>, Fault>>>,
    reader: Option<std::thread::JoinHandle<()>>,
}
impl Worker {
    pub fn start(request: &[u8]) -> Result<Self, Fault> {
        let path = if let Some(path) = std::env::var_os("XDZIP_NATIVE_CODEC_WORKER")
            .or_else(|| std::env::var_os("XDZIP_NATIVE_CODEC"))
        {
            std::path::PathBuf::from(path)
        } else {
            let mut path = std::env::current_exe().map_err(|_| Fault::Io)?;
            path.set_file_name(if cfg!(windows) {
                "xdzip-native-codec.exe"
            } else {
                "xdzip-native-codec"
            });
            path
        };
        let mut command = Command::new(path);
        // The decoder only needs its private pipes and OS loader settings, not
        // application credentials, proxy configuration or developer search paths.
        command.env_clear();
        #[cfg(windows)]
        for name in ["SystemRoot", "WINDIR"] {
            if let Some(value) = std::env::var_os(name) {
                command.env(name, value);
            }
        }
        command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(test)]
        command.stderr(Stdio::inherit());
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(0x0800_0000); // CREATE_NO_WINDOW, never show a console for archive work.
        }
        let mut child = command.spawn().map_err(|_| Fault::Unsupported)?;
        let input = child.stdin.take().ok_or(Fault::Io)?;
        let mut output = child.stdout.take().ok_or(Fault::Io)?;
        let (sender, receiver) = mpsc::sync_channel(2);
        let reader = std::thread::spawn(move || {
            loop {
                let frame = read_frame(&mut output);
                let stop = frame.is_err();
                if sender.send(frame).is_err() || stop {
                    break;
                }
            }
        });
        let mut worker = Self {
            child,
            input,
            receiver: Some(receiver),
            reader: Some(reader),
        };
        worker.send(START, request)?;
        Ok(worker)
    }
    pub fn send(&mut self, kind: u8, bytes: &[u8]) -> Result<(), Fault> {
        if bytes.len() >= MAX_FRAME {
            return Err(Fault::Limit);
        }
        self.input
            .write_all(&((bytes.len() + 1) as u32).to_le_bytes())
            .map_err(|_| Fault::Io)?;
        self.input.write_all(&[kind]).map_err(|_| Fault::Io)?;
        self.input.write_all(bytes).map_err(|_| Fault::Io)?;
        self.input.flush().map_err(|_| Fault::Io)
    }
    pub fn next(&mut self, control: &Cancellation) -> Result<Vec<u8>, Fault> {
        loop {
            control.check().map_err(|_| Fault::Cancelled)?;
            match self
                .receiver
                .as_ref()
                .ok_or(Fault::Io)?
                .recv_timeout(Duration::from_millis(50))
            {
                Ok(frame) => return frame,
                Err(mpsc::RecvTimeoutError::Timeout) => (),
                Err(_) => return Err(Fault::Io),
            }
        }
    }
    pub fn finish(&mut self, control: &Cancellation) -> Result<(), Fault> {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            control.check().map_err(|_| Fault::Cancelled)?;
            if let Some(status) = self.child.try_wait().map_err(|_| Fault::Io)? {
                if !status.success() {
                    return Err(Fault::Io);
                }
                if self
                    .receiver
                    .as_ref()
                    .is_some_and(|receiver| matches!(receiver.try_recv(), Ok(Ok(_))))
                {
                    return Err(Fault::Invalid);
                }
                return Ok(());
            }
            if std::time::Instant::now() >= deadline {
                return Err(Fault::Io);
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        // Every error and cancellation terminates an in-flight native parser before returning.
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Disconnect the bounded channel before joining: the reader may be blocked
        // sending a final frame. The killed child closes stdout and unblocks reads.
        self.receiver.take();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}
fn read_frame(reader: &mut dyn Read) -> Result<Vec<u8>, Fault> {
    let mut length = [0; 4];
    reader.read_exact(&mut length).map_err(|_| Fault::Io)?;
    let length = u32::from_le_bytes(length) as usize;
    if length == 0 || length > MAX_FRAME {
        return Err(Fault::Limit);
    }
    let mut bytes = vec![0; length];
    reader.read_exact(&mut bytes).map_err(|_| Fault::Io)?;
    Ok(bytes)
}

pub struct Fields<'a>(pub &'a [u8]);
impl<'a> Fields<'a> {
    pub fn take(&mut self, count: usize) -> Result<&'a [u8], Fault> {
        if self.0.len() < count {
            return Err(Fault::Invalid);
        }
        let (value, tail) = self.0.split_at(count);
        self.0 = tail;
        Ok(value)
    }
    pub fn byte(&mut self) -> Result<u8, Fault> {
        Ok(self.take(1)?[0])
    }
    pub fn u32(&mut self) -> Result<u32, Fault> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| Fault::Invalid)?,
        ))
    }
    pub fn u64(&mut self) -> Result<u64, Fault> {
        Ok(u64::from_le_bytes(
            self.take(8)?.try_into().map_err(|_| Fault::Invalid)?,
        ))
    }
    #[cfg(test)]
    pub fn string(&mut self, limit: usize) -> Result<String, Fault> {
        String::from_utf8(self.bytes(limit)?).map_err(|_| Fault::Invalid)
    }
    pub fn bytes(&mut self, limit: usize) -> Result<Vec<u8>, Fault> {
        let length = self.u32()? as usize;
        if length > limit {
            return Err(Fault::Limit);
        }
        Ok(self.take(length)?.to_vec())
    }
    pub fn end(&self) -> Result<(), Fault> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(Fault::Invalid)
        }
    }
}
pub fn put_string(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(&(value.len() as u32).to_le_bytes());
    output.extend_from_slice(value.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn framing_rejects_zero_and_oversized_before_allocation() {
        assert_eq!(read_frame(&mut &0u32.to_le_bytes()[..]), Err(Fault::Limit));
        assert_eq!(
            read_frame(&mut &((MAX_FRAME + 1) as u32).to_le_bytes()[..]),
            Err(Fault::Limit)
        );
    }
    #[test]
    fn fields_reject_truncation_trailing_data_and_invalid_utf8() {
        assert_eq!(Fields(&[1]).u64(), Err(Fault::Invalid));
        assert_eq!(Fields(&[1]).end(), Err(Fault::Invalid));
        assert_eq!(Fields(&[1, 0, 0, 0, 255]).string(4), Err(Fault::Invalid));
        assert_eq!(Fields(&[5, 0, 0, 0]).string(4), Err(Fault::Limit));
    }
}
