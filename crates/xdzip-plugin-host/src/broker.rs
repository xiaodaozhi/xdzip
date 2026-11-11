use crate::{Error, Report, package::hex, process::Peer};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Seek, SeekFrom, Write},
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
use xdzip_domain::path::PortablePath;
use xdzip_io::{FileVersion, Grant, Stage, Workspace, open_regular_readonly};
use xdzip_plugin_protocol::{Envelope, Kind, MAJOR, MAX_BLOCK, Value, accept};
const MAX_SOURCE: u64 = 64 * 1024 * 1024;
const MAX_OUTPUT: u64 = 4 * 1024 * 1024;
const MAX_ENTRIES: usize = 1000;
fn number(n: u64) -> Value {
    Value::UInt(n)
}
fn binary(b: &[u8]) -> Value {
    Value::Bytes(b.to_vec())
}
fn nonce() -> Result<Vec<u8>, Error> {
    let mut value = vec![0; 16];
    getrandom::fill(&mut value).map_err(|_| Error::Io)?;
    Ok(value)
}

struct Source {
    file: File,
    path: PathBuf,
    version: FileVersion,
    token: Vec<u8>,
    read: u64,
}
impl Source {
    fn validate(&self) -> Result<(), Error> {
        if FileVersion::read(&self.file)? != self.version
            || FileVersion::read(&open_regular_readonly(&self.path)?)? != self.version
        {
            return Err(Error::Io);
        }
        Ok(())
    }
    fn read(&mut self, p: &Value) -> Result<Value, Error> {
        p.schema(&[1, 2, 3])?;
        if p.field(1)?.bytes()? != self.token {
            return Err(Error::Protocol);
        }
        let offset = p.field(2)?.uint()?;
        let size = p.field(3)?.uint()?;
        if size == 0
            || size > MAX_BLOCK as u64
            || offset
                .checked_add(size)
                .is_none_or(|end| end > self.version.length)
        {
            return Err(Error::Limit);
        }
        self.read = self
            .read
            .checked_add(size)
            .filter(|n| *n <= MAX_SOURCE)
            .ok_or(Error::Limit)?;
        self.validate()?;
        let mut bytes = vec![0; size as usize];
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read_exact(&mut bytes)?;
        self.validate()?;
        Ok(Value::map([
            (1, binary(&self.token)),
            (2, number(offset)),
            (3, Value::Bytes(bytes)),
        ]))
    }
}
struct Entry {
    token: Vec<u8>,
    path: PortablePath,
    size: u64,
    directory: bool,
}
// Rust drops struct fields in declaration order. Close/remove the stage before
// Workspace removes its transaction directory, including on a late bad terminal.
struct PendingOutput {
    stage: Stage,
    workspace: Workspace,
    path: PortablePath,
}
struct Channel<'a> {
    peer: &'a mut Peer,
    request: u64,
    operation: u64,
    sent: u64,
    next: u64,
    messages: usize,
    cancel: &'a AtomicBool,
}
impl Channel<'_> {
    fn send(&mut self, kind: Kind, payload: Value) -> Result<(), Error> {
        let frame = Envelope {
            kind,
            request: self.request,
            operation: self.operation,
            sequence: self.sent,
            payload,
        };
        self.sent = self.sent.checked_add(1).ok_or(Error::Limit)?;
        self.peer.send(frame)
    }
    fn receive(&mut self, deadline: Instant) -> Result<Envelope, Error> {
        let frame = match self.peer.receive(deadline, self.cancel) {
            Err(Error::Cancelled) => {
                // Cancel revokes the current sink immediately. Only control
                // responses are admitted during the bounded cooperative grace.
                self.send(Kind::Cancel, Value::map([]))?;
                let grace = Instant::now() + Duration::from_secs(2);
                let never = AtomicBool::new(false);
                while let Ok(frame) = self.peer.receive(grace, &never) {
                    accept(&frame, &mut self.next, self.request, self.operation)?;
                    if frame.kind == Kind::OperationFinished {
                        break;
                    }
                    if frame.kind != Kind::CancelAck {
                        break;
                    }
                }
                return Err(Error::Cancelled);
            }
            result => result?,
        };
        accept(&frame, &mut self.next, self.request, self.operation)?;
        self.messages += 1;
        if self.messages > 8192 {
            return Err(Error::Limit);
        }
        Ok(frame)
    }
    fn call(&mut self, kind: Kind, payload: Value) -> Result<Instant, Error> {
        self.request += 1;
        self.send(kind, payload)?;
        Ok(Instant::now() + Duration::from_secs(5))
    }
    fn terminal(&mut self, deadline: Instant) -> Result<Value, Error> {
        let frame = self.receive(deadline)?;
        if frame.kind != Kind::OperationFinished || frame.payload.field(1)?.uint()? != 0 {
            return Err(Error::Protocol);
        }
        Ok(frame.payload)
    }
    fn reply_read(&mut self, source: &mut Source, frame: &Envelope) -> Result<(), Error> {
        if frame.kind != Kind::ReadAt {
            return Err(Error::Protocol);
        }
        let reply = source.read(&frame.payload)?;
        self.send(Kind::ReadData, reply)
    }
}

pub(crate) fn run(
    peer: &mut Peer,
    path: &Path,
    selected: Option<(usize, &Path)>,
    expected_id: &str,
    cancel: &AtomicBool,
) -> Result<Report, Error> {
    let file = open_regular_readonly(path)?;
    let version = FileVersion::read(&file)?;
    let mut source = Source {
        file,
        path: path.to_owned(),
        version,
        token: nonce()?,
        read: 0,
    };
    let mut channel = Channel {
        peer,
        request: 0,
        operation: 0,
        sent: 1,
        next: 1,
        messages: 0,
        cancel,
    };
    let hello = channel.receive(Instant::now() + Duration::from_secs(2))?;
    hello.payload.schema(&[1, 2, 3, 4])?;
    if hello.kind != Kind::Hello
        || hello.payload.field(1)?.uint()? > MAJOR
        || hello.payload.field(2)?.uint()? < MAJOR
        || hello.payload.field(3)?.text()? != expected_id
        || hello.payload.field(4)?.text()? != "xdp1"
    {
        return Err(Error::Protocol);
    }
    channel.send(
        Kind::Welcome,
        Value::map([
            (1, number(MAJOR)),
            (2, number(MAX_BLOCK as u64)),
            (3, number(1)),
            (4, number(MAX_SOURCE)),
            (5, number(MAX_OUTPUT)),
        ]),
    )?;
    let identity = nonce()?;
    channel.operation = u64::from_le_bytes(identity[..8].try_into().unwrap()).max(1);
    let deadline = channel.call(
        Kind::Probe,
        Value::map([
            (1, binary(&source.token)),
            (2, number(source.version.length)),
        ]),
    )?;
    loop {
        let frame = channel.receive(deadline)?;
        if frame.kind == Kind::OperationFinished {
            frame.payload.schema(&[1, 2])?;
            if frame.payload.field(1)?.uint()? != 0 || frame.payload.field(2)?.text()? != "xdp1" {
                return Err(Error::Protocol);
            }
            break;
        }
        channel.reply_read(&mut source, &frame)?;
    }
    let deadline = channel.call(Kind::Open, Value::map([(1, binary(&source.token))]))?;
    let opened = channel.terminal(deadline)?;
    opened.schema(&[1, 2])?;
    let session = opened.field(2)?.bytes()?.to_vec();
    if session.len() != 16 {
        return Err(Error::Protocol);
    }
    let deadline = channel.call(Kind::Scan, Value::map([(1, binary(&session))]))?;
    let mut entries = Vec::new();
    let mut tokens = BTreeSet::new();
    let mut metadata_bytes = 0;
    let mut finished = false;
    loop {
        let frame = channel.receive(deadline)?;
        match frame.kind {
            Kind::ReadAt if !finished => channel.reply_read(&mut source, &frame)?,
            Kind::EntryBatch if !finished => {
                frame.payload.schema(&[1, 2])?;
                if frame.payload.field(1)?.bytes()? != session {
                    return Err(Error::Protocol);
                }
                for value in frame.payload.field(2)?.array()? {
                    value.schema(&[1, 2, 3, 4])?;
                    let token = value.field(1)?.bytes()?;
                    let raw = value.field(2)?.bytes()?;
                    let directory = match value.field(3)?.uint()? {
                        0 => false,
                        1 => true,
                        _ => return Err(Error::Protocol),
                    };
                    metadata_bytes += raw.len() + token.len();
                    if token.is_empty() || token.len() > 32 || !tokens.insert(token.to_vec()) {
                        return Err(Error::Protocol);
                    }
                    if entries.len() >= MAX_ENTRIES || metadata_bytes > 256 * 1024 {
                        return Err(Error::Limit);
                    }
                    let text = std::str::from_utf8(raw).map_err(|_| Error::Protocol)?;
                    let path = PortablePath::parse(if directory {
                        text.strip_suffix('/').ok_or(Error::Protocol)?
                    } else {
                        text
                    })
                    .ok_or(Error::Protocol)?;
                    entries.push(Entry {
                        token: token.to_vec(),
                        path,
                        size: value.field(4)?.uint()?,
                        directory,
                    });
                }
            }
            Kind::ScanFinished if !finished => {
                frame.payload.schema(&[1])?;
                if frame.payload.field(1)?.uint()? != entries.len() as u64 {
                    return Err(Error::Protocol);
                }
                finished = true;
            }
            Kind::OperationFinished if finished => {
                frame.payload.schema(&[1])?;
                if frame.payload.field(1)?.uint()? != 0 {
                    return Err(Error::Protocol);
                }
                break;
            }
            _ => return Err(Error::Protocol),
        }
    }
    let mut output: Option<PendingOutput> = None;
    let mut output_size = 0u64;
    let mut digest = String::new();
    if let Some((index, target)) = selected {
        let entry = entries.get(index).ok_or(Error::Protocol)?;
        if entry.directory || entry.size > MAX_OUTPUT {
            return Err(Error::Limit);
        }
        let grant = Grant::open(target, source.version.identity)?;
        let mut workspace = grant.workspace(channel.operation)?;
        let mut stage = workspace.stage()?;
        let sink = nonce()?;
        let mut credit = nonce()?;
        let mut block = 0;
        let deadline = channel.call(
            Kind::ReadEntry,
            Value::map([
                (1, binary(&session)),
                (2, binary(&entry.token)),
                (3, binary(&sink)),
                (4, number(1)),
                (5, number(MAX_OUTPUT)),
                (6, binary(&credit)),
            ]),
        )?;
        let mut hash = Sha256::new();
        let mut sealed = false;
        loop {
            let frame = channel.receive(deadline)?;
            match frame.kind {
                Kind::ReadAt if !sealed => channel.reply_read(&mut source, &frame)?,
                Kind::WriteData if !sealed => {
                    frame.payload.schema(&[1, 2, 3, 4])?;
                    if frame.payload.field(1)?.bytes()? != sink
                        || frame.payload.field(2)?.uint()? != block
                        || frame.payload.field(4)?.bytes()? != credit
                    {
                        return Err(Error::Protocol);
                    }
                    let bytes = frame.payload.field(3)?.bytes()?;
                    if bytes.is_empty() {
                        return Err(Error::Protocol);
                    }
                    output_size = output_size
                        .checked_add(bytes.len() as u64)
                        .filter(|n| *n <= entry.size && *n <= MAX_OUTPUT)
                        .ok_or(Error::Limit)?;
                    stage.write_all(bytes)?;
                    hash.update(bytes);
                    block += 1;
                    credit = nonce()?;
                    channel.send(
                        Kind::WriteAck,
                        Value::map([(1, binary(&sink)), (2, number(block)), (3, binary(&credit))]),
                    )?;
                }
                Kind::FinishOutput if !sealed => {
                    frame.payload.schema(&[1, 2, 3])?;
                    let actual = hash.clone().finalize();
                    if frame.payload.field(1)?.bytes()? != sink
                        || frame.payload.field(2)?.uint()? != output_size
                        || output_size != entry.size
                        || frame.payload.field(3)?.bytes()? != actual.as_slice()
                    {
                        return Err(Error::Protocol);
                    }
                    digest = hex(&actual);
                    stage.finish()?;
                    sealed = true;
                }
                Kind::OperationFinished if sealed => {
                    frame.payload.schema(&[1])?;
                    if frame.payload.field(1)?.uint()? != 0 {
                        return Err(Error::Protocol);
                    }
                    break;
                }
                _ => return Err(Error::Protocol),
            }
        }
        output = Some(PendingOutput {
            stage,
            workspace,
            path: entry.path.clone(),
        });
    }
    let deadline = channel.call(Kind::Close, Value::map([(1, binary(&session))]))?;
    channel.terminal(deadline)?.schema(&[1])?;
    channel.send(Kind::Shutdown, Value::map([]))?;
    channel.peer.finish()?;
    source.validate()?;
    // Final cancellation check precedes publication. This single-entry trial
    // has no previously committed output; every earlier failure drops staging.
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return Err(Error::Cancelled);
    }
    let committed = output.is_some();
    let mut cleanup_warning = false;
    if let Some(PendingOutput {
        mut workspace,
        stage,
        path,
    }) = output
    {
        cleanup_warning = workspace.commit(stage, &path, None)?.cleanup_warning;
        cleanup_warning |= workspace.cleanup().is_err();
    }
    Ok(Report {
        protocol_major: MAJOR,
        isolation: channel.peer.isolation.clone(),
        entries: entries.len() as u64,
        source_bytes: source.read,
        output_bytes: output_size,
        output_sha256: digest,
        committed,
        cleanup_warning,
    })
}
