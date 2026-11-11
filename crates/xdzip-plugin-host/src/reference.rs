//! Owned XDP1 read-only protocol example. Contains no host filesystem access
//! during ordinary codec operation; all source bytes arrive through ReadAt.
use sha2::{Digest, Sha256};
use std::io::{stdin, stdout};
use xdzip_plugin_protocol::{Envelope, Error, Kind, MAX_BLOCK, Value, accept};
fn n(value: u64) -> Value {
    Value::UInt(value)
}
fn b(value: &[u8]) -> Value {
    Value::Bytes(value.to_vec())
}
struct Peer {
    sent: u64,
    next: u64,
    request: u64,
    operation: u64,
    source: Vec<u8>,
    length: u64,
    session: Vec<u8>,
}
impl Peer {
    fn send(&mut self, kind: Kind, payload: Value) -> Result<(), Error> {
        let frame = Envelope {
            kind,
            request: self.request,
            operation: self.operation,
            sequence: self.sent,
            payload,
        };
        self.sent += 1;
        frame.write(&mut stdout().lock())
    }
    fn receive(&mut self) -> Result<Envelope, Error> {
        let frame = Envelope::read(&mut stdin().lock())?;
        accept(&frame, &mut self.next, self.request, self.operation)?;
        if frame.kind == Kind::Cancel {
            self.send(Kind::CancelAck, Value::map([]))?;
            self.send(Kind::OperationFinished, Value::map([(1, n(1))]))?;
            std::process::exit(0);
        }
        Ok(frame)
    }
    fn read(&mut self, offset: u64, length: usize) -> Result<Vec<u8>, Error> {
        if offset
            .checked_add(length as u64)
            .is_none_or(|end| end > self.length)
            || length == 0
            || length > MAX_BLOCK
        {
            return Err(Error::Limit);
        }
        self.send(
            Kind::ReadAt,
            Value::map([(1, b(&self.source)), (2, n(offset)), (3, n(length as u64))]),
        )?;
        let response = self.receive()?;
        response.payload.schema(&[1, 2, 3])?;
        if response.kind != Kind::ReadData
            || response.payload.field(1)?.bytes()? != self.source
            || response.payload.field(2)?.uint()? != offset
            || response.payload.field(3)?.bytes()?.len() != length
        {
            return Err(Error::Invalid);
        }
        Ok(response.payload.field(3)?.bytes()?.to_vec())
    }
    fn done(&mut self) -> Result<(), Error> {
        self.send(Kind::OperationFinished, Value::map([(1, n(0))]))
    }
}
struct Record {
    token: Vec<u8>,
    path: Vec<u8>,
    size: u32,
    offset: u64,
}
fn run() -> Result<(), Error> {
    let mut peer = Peer {
        sent: 1,
        next: 1,
        request: 0,
        operation: 0,
        source: vec![],
        length: 0,
        session: b"reference-session"[..16].to_vec(),
    };
    peer.send(
        Kind::Hello,
        Value::map([
            (1, n(1)),
            (2, n(1)),
            (3, Value::Text("org.xdzip.sdk.reference".into())),
            (4, Value::Text("xdp1".into())),
        ]),
    )?;
    let welcome = peer.receive()?;
    if welcome.kind != Kind::Welcome || welcome.payload.field(1)?.uint()? != 1 {
        return Err(Error::Invalid);
    }
    let mut records: Vec<Record> = vec![];
    let mut state = 0;
    loop {
        let frame = Envelope::read(&mut stdin().lock())?;
        if frame.kind == Kind::Shutdown {
            accept(&frame, &mut peer.next, peer.request, peer.operation)?;
            if state != 4 {
                return Err(Error::Invalid);
            }
            return Ok(());
        }
        if frame.request != peer.request + 1
            || frame.operation == 0
            || (peer.operation != 0 && frame.operation != peer.operation)
        {
            return Err(Error::Invalid);
        }
        peer.request = frame.request;
        peer.operation = frame.operation;
        accept(&frame, &mut peer.next, peer.request, peer.operation)?;
        match frame.kind {
            Kind::Probe if state == 0 => {
                frame.payload.schema(&[1, 2])?;
                peer.source = frame.payload.field(1)?.bytes()?.to_vec();
                peer.length = frame.payload.field(2)?.uint()?;
                if peer.read(0, 4)? != b"XDP1" {
                    return Err(Error::Invalid);
                }
                peer.send(
                    Kind::OperationFinished,
                    Value::map([(1, n(0)), (2, Value::Text("xdp1".into()))]),
                )?;
                state = 1;
            }
            Kind::Open if state == 1 => {
                frame.payload.schema(&[1])?;
                if frame.payload.field(1)?.bytes()? != peer.source {
                    return Err(Error::Invalid);
                }
                peer.send(
                    Kind::OperationFinished,
                    Value::map([(1, n(0)), (2, b(&peer.session))]),
                )?;
                state = 2;
            }
            Kind::Scan if state == 2 => {
                frame.payload.schema(&[1])?;
                if frame.payload.field(1)?.bytes()? != peer.session {
                    return Err(Error::Invalid);
                }
                let count = u32::from_le_bytes(peer.read(4, 4)?.try_into().unwrap());
                if count > 1000 {
                    return Err(Error::Limit);
                }
                let mut offset = 8u64;
                for index in 0..count {
                    let header = peer.read(offset, 6)?;
                    let path_len = u16::from_le_bytes(header[..2].try_into().unwrap()) as usize;
                    let size = u32::from_le_bytes(header[2..].try_into().unwrap());
                    if path_len == 0 || path_len > 4096 {
                        return Err(Error::Limit);
                    }
                    let path = peer.read(offset + 6, path_len)?;
                    offset = offset
                        .checked_add(6 + path_len as u64)
                        .ok_or(Error::Limit)?;
                    let record = Record {
                        token: index.to_le_bytes().to_vec(),
                        path,
                        size,
                        offset,
                    };
                    offset = offset
                        .checked_add(u64::from(size))
                        .filter(|end| *end <= peer.length)
                        .ok_or(Error::Invalid)?;
                    peer.send(
                        Kind::EntryBatch,
                        Value::map([
                            (1, b(&peer.session)),
                            (
                                2,
                                Value::Array(vec![Value::map([
                                    (1, b(&record.token)),
                                    (2, b(&record.path)),
                                    (3, n(0)),
                                    (4, n(u64::from(size))),
                                ])]),
                            ),
                        ]),
                    )?;
                    records.push(record);
                }
                if offset != peer.length {
                    return Err(Error::Invalid);
                }
                peer.send(Kind::ScanFinished, Value::map([(1, n(u64::from(count)))]))?;
                peer.done()?;
                state = 3;
            }
            Kind::ReadEntry if state == 3 => {
                frame.payload.schema(&[1, 2, 3, 4, 5, 6])?;
                if frame.payload.field(1)?.bytes()? != peer.session
                    || frame.payload.field(4)?.uint()? != 1
                {
                    return Err(Error::Invalid);
                }
                let token = frame.payload.field(2)?.bytes()?;
                let record = records
                    .iter()
                    .find(|r| r.token == token)
                    .ok_or(Error::Invalid)?;
                let sink = frame.payload.field(3)?.bytes()?.to_vec();
                let mut credit = frame.payload.field(6)?.bytes()?.to_vec();
                let mut read = 0;
                let mut block = 0;
                let mut hash = Sha256::new();
                while read < record.size as usize {
                    let count = (record.size as usize - read).min(MAX_BLOCK);
                    let bytes = peer.read(record.offset + read as u64, count)?;
                    hash.update(&bytes);
                    peer.send(
                        Kind::WriteData,
                        Value::map([
                            (1, b(&sink)),
                            (2, n(block)),
                            (3, Value::Bytes(bytes)),
                            (4, b(&credit)),
                        ]),
                    )?;
                    let ack = peer.receive()?;
                    ack.payload.schema(&[1, 2, 3])?;
                    block += 1;
                    if ack.kind != Kind::WriteAck
                        || ack.payload.field(1)?.bytes()? != sink
                        || ack.payload.field(2)?.uint()? != block
                    {
                        return Err(Error::Invalid);
                    }
                    credit = ack.payload.field(3)?.bytes()?.to_vec();
                    read += count;
                }
                peer.send(
                    Kind::FinishOutput,
                    Value::map([(1, b(&sink)), (2, n(read as u64)), (3, b(&hash.finalize()))]),
                )?;
                peer.done()?;
            }
            Kind::Close if state == 3 => {
                frame.payload.schema(&[1])?;
                if frame.payload.field(1)?.bytes()? != peer.session {
                    return Err(Error::Invalid);
                }
                peer.done()?;
                state = 4;
            }
            _ => return Err(Error::Invalid),
        }
    }
}
#[cfg(unix)]
#[allow(unsafe_code)]
fn fork_allowed() -> bool {
    // SAFETY: the trusted audit child performs only async-signal-safe _exit.
    // The parent waits only for this owned PID. This deliberately bypasses the
    // std::process helper pipe so its denial cannot mask an allowed raw fork.
    unsafe {
        let pid = libc::fork();
        if pid < 0 {
            return false;
        }
        if pid == 0 {
            libc::_exit(0);
        }
        let mut status = 0;
        while libc::waitpid(pid, &mut status, 0) < 0 {
            if std::io::Error::last_os_error().raw_os_error() != Some(libc::EINTR) {
                break;
            }
        }
        true
    }
}
#[cfg(not(unix))]
fn fork_allowed() -> bool {
    false
}
fn audit(args: &[String]) -> bool {
    use std::{
        io::Read,
        net::{Ipv4Addr, SocketAddrV4, TcpStream},
        process::{Command, Stdio},
        time::Duration,
    };
    if args.len() != 3 {
        return false;
    }
    let read = std::fs::File::open(&args[0])
        .and_then(|mut f| {
            let mut b = [0; 1];
            f.read_exact(&mut b)
        })
        .is_ok();
    let write = std::fs::write(&args[1], b"forbidden").is_ok();
    let network = args[2].parse::<u16>().ok().is_some_and(|port| {
        TcpStream::connect_timeout(
            &SocketAddrV4::new(Ipv4Addr::LOCALHOST, port).into(),
            Duration::from_millis(200),
        )
        .is_ok()
    });
    let descendant = std::env::current_exe().ok().is_some_and(|exe| {
        Command::new(exe)
            .arg("--audit-child")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .is_ok_and(|s| s.success())
    });
    let fork = fork_allowed();
    println!(
        "{{\"read_denied\":{},\"write_denied\":{},\"network_denied\":{},\"descendant_denied\":{},\"fork_denied\":{}}}",
        !read, !write, !network, !descendant, !fork
    );
    !read && !write && !network && !descendant && !fork
}
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.first().is_some_and(|s| s == "--audit-child") {
        return;
    }
    if args.first().is_some_and(|s| s == "--sandbox-audit") {
        std::process::exit(if audit(&args[1..]) { 0 } else { 1 });
    }
    if run().is_err() {
        std::process::exit(1);
    }
}
