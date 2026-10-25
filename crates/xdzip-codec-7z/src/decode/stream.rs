use super::*;
use crate::ByteRange;
use crate::crypto::AesReader;
use std::io::Read;
use xdzip_compression::{Bzip2Reader, DeflateReader, Lzma2Reader, LzmaReader, Ppmd7Reader};

pub(crate) type SharedSource<'a> = Rc<RefCell<Box<dyn HeaderSource + 'a>>>;
type SharedState = Rc<RefCell<State>>;
type Input<'a> = Box<dyn Read + 'a>;

/// Lazy single-consumer DAG. Its handles own upstream readers; the observer
/// table finishes every node, including zero-output nodes, without discarding.
pub(crate) struct FolderStream<'a> {
    source: SharedSource<'a>,
    control: &'a Cancellation,
    state: SharedState,
    root: Handle<'a>,
    nodes: Vec<Handle<'a>>,
    leaves: Vec<LeafHandle<'a>>,
    expected: u64,
    position: u64,
    crc: Crc32,
    declared_crc: Option<u32>,
    finished: bool,
}
impl<'a> FolderStream<'a> {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn new(
        source: SharedSource<'a>,
        streams: &Streams,
        index: usize,
        control: &'a Cancellation,
        password: Option<&str>,
        state: SharedState,
        metadata: bool,
    ) -> Result<Self, DecodeError> {
        check(control)?;
        let limits = state
            .try_borrow()
            .map_err(|_| DecodeError::Integrity)?
            .limits;
        let folder = streams.folders.get(index).ok_or(DecodeError::Integrity)?;
        let methods = plan(folder, streams, limits, 0, metadata)?;
        if folder.encrypted() && password.is_none() {
            return Err(DecodeError::PasswordRequired);
        }
        let length = {
            let source = source.try_borrow().map_err(|_| DecodeError::Integrity)?;
            if let Err(error) = source.check() {
                state
                    .try_borrow_mut()
                    .map_err(|_| DecodeError::Integrity)?
                    .source_error = Some(error);
                return Err(decode_error(error));
            }
            source.length()
        };
        for input in &folder.packed_inputs {
            let range = &streams.packs[input.pack].range;
            if range
                .offset
                .checked_add(range.length)
                .is_none_or(|n| n > length)
            {
                return Err(DecodeError::Integrity);
            }
        }
        let mut leaves = Vec::new();
        leaves
            .try_reserve_exact(folder.packed_inputs.len())
            .map_err(|_| DecodeError::Limit)?;
        for input in &folder.packed_inputs {
            let pack = &streams.packs[input.pack];
            leaves.push(LeafHandle(Rc::new(RefCell::new(Leaf {
                source: source.clone(),
                state: state.clone(),
                control,
                range: pack.range.clone(),
                expected_crc: pack.crc,
                position: 0,
                crc: Crc32::new(),
                failed: false,
            }))));
        }
        let mut outputs = Vec::new();
        outputs
            .try_reserve_exact(folder.output_sizes.len())
            .map_err(|_| DecodeError::Limit)?;
        outputs.resize_with(folder.output_sizes.len(), || None);
        let mut nodes = Vec::new();
        nodes
            .try_reserve_exact(methods.len())
            .map_err(|_| DecodeError::Limit)?;
        for &index in &folder.topological_order {
            check(control)?;
            let planned = &methods[index];
            let mut inputs: Vec<Input<'a>> = Vec::new();
            let mut lengths = Vec::new();
            inputs
                .try_reserve_exact(planned.inputs.len())
                .map_err(|_| DecodeError::Limit)?;
            lengths
                .try_reserve_exact(planned.inputs.len())
                .map_err(|_| DecodeError::Limit)?;
            for slot in &planned.inputs {
                match *slot {
                    Slot::Bound(output) => {
                        let handle: &Handle<'a> = outputs
                            .get(output)
                            .and_then(Option::as_ref)
                            .ok_or(DecodeError::Integrity)?;
                        inputs.push(Box::new(handle.clone()));
                        lengths.push(folder.output_sizes[output]);
                    }
                    Slot::Packed(input) => {
                        inputs.push(Box::new(leaves[input].clone()));
                        lengths.push(streams.packs[folder.packed_inputs[input].pack].range.length);
                    }
                }
            }
            let expected = folder.output_sizes[planned.output];
            let decoder = algorithm(
                planned, inputs, &lengths, expected, control, password, &state,
            )?;
            let node = Handle(Rc::new(RefCell::new(Node {
                decoder,
                inputs: lengths,
                expected,
                position: 0,
                ended: false,
                failed: false,
                terminal: planned.output == folder.terminal_output,
                state: state.clone(),
                control,
            })));
            outputs[planned.output] = Some(node.clone());
            nodes.push(node);
        }
        let root = outputs
            .get_mut(folder.terminal_output)
            .and_then(Option::take)
            .ok_or(DecodeError::Integrity)?;
        Ok(Self {
            source,
            control,
            state,
            root,
            nodes,
            leaves,
            expected: folder.unpacked_size(),
            position: 0,
            crc: Crc32::new(),
            declared_crc: folder.crc,
            finished: false,
        })
    }
    pub(crate) fn position(&self) -> u64 {
        self.position
    }
    pub(crate) fn read(&mut self, out: &mut [u8]) -> Result<usize, DecodeError> {
        self.check()?;
        if out.is_empty() {
            return Ok(0);
        }
        if self.finished {
            return Ok(0);
        }
        let n = self.root.read(out).map_err(algorithm_error)?;
        self.position = self
            .position
            .checked_add(n as u64)
            .filter(|n| *n <= self.expected)
            .ok_or(DecodeError::Integrity)?;
        if self.declared_crc.is_some() {
            self.crc.update(&out[..n]);
        }
        if n == 0 {
            self.finish()?;
        }
        self.check()?;
        Ok(n)
    }
    pub(crate) fn finish(&mut self) -> Result<(), DecodeError> {
        self.check()?;
        if self.finished {
            return Ok(());
        }
        if self.position != self.expected {
            return Err(DecodeError::Integrity);
        }
        for node in self.nodes.iter().rev() {
            self.check()?;
            node.finish()?;
        }
        for leaf in &self.leaves {
            self.check()?;
            leaf.finish()?;
        }
        if self
            .declared_crc
            .is_some_and(|crc| crc != self.crc.checksum())
        {
            return Err(DecodeError::Integrity);
        }
        self.finished = true;
        self.check()
    }
    fn check(&self) -> Result<(), DecodeError> {
        check(self.control)?;
        let result = self
            .source
            .try_borrow()
            .map_err(|_| DecodeError::Integrity)?
            .check();
        result.map_err(|error| {
            if let Ok(mut state) = self.state.try_borrow_mut() {
                state.source_error = Some(error);
            }
            decode_error(error)
        })
    }
}

#[derive(Clone)]
struct Handle<'a>(Rc<RefCell<Node<'a>>>);
impl Handle<'_> {
    fn finish(&self) -> Result<(), DecodeError> {
        let mut node = self
            .0
            .try_borrow_mut()
            .map_err(|_| DecodeError::Integrity)?;
        if node.position != node.expected {
            return Err(DecodeError::Integrity);
        }
        let mut probe = [0];
        if node.read(&mut probe).map_err(algorithm_error)? != 0 {
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
}
impl Read for Handle<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.0
            .try_borrow_mut()
            .map_err(|_| io::Error::other(DecodeError::Integrity))?
            .read(out)
            .map_err(stable)
    }
}
struct Node<'a> {
    decoder: Algorithm<'a>,
    inputs: Vec<u64>,
    expected: u64,
    position: u64,
    ended: bool,
    failed: bool,
    terminal: bool,
    state: SharedState,
    control: &'a Cancellation,
}
impl Read for Node<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(io::Error::other(DecodeError::Integrity));
        }
        let result = (|| {
            check(self.control)?;
            if self.ended {
                return Ok(0);
            }
            let left = self.expected - self.position;
            let allowance = {
                let state = self
                    .state
                    .try_borrow()
                    .map_err(|_| DecodeError::Integrity)?;
                let mut left = state
                    .limits
                    .node_work_bytes
                    .checked_sub(state.node_work)
                    .and_then(|n| n.checked_sub(state.node_reserved))
                    .ok_or(DecodeError::Limit)?;
                if self.terminal {
                    left = left.min(
                        state
                            .limits
                            .work_bytes
                            .checked_sub(state.work)
                            .and_then(|n| n.checked_sub(state.work_reserved))
                            .ok_or(DecodeError::Limit)?,
                    );
                }
                left
            };
            if left != 0 && allowance == 0 {
                return Err(DecodeError::Limit);
            }
            let width = if left == 0 {
                1
            } else {
                out.len()
                    .min(CHUNK)
                    .min(left.min(allowance).min(CHUNK as u64) as usize)
            };
            // Hold budget credit, not a RefCell borrow, across recursive reads.
            // Upstream nodes cannot spend the allowance for this node's output.
            let reserved = if left == 0 { 0 } else { width as u64 };
            {
                let mut state = self
                    .state
                    .try_borrow_mut()
                    .map_err(|_| DecodeError::Integrity)?;
                state.node_reserved = state
                    .node_reserved
                    .checked_add(reserved)
                    .ok_or(DecodeError::Limit)?;
                if self.terminal {
                    state.work_reserved = state
                        .work_reserved
                        .checked_add(reserved)
                        .ok_or(DecodeError::Limit)?;
                }
            }
            let result = self.decoder.read(&mut out[..width]);
            {
                let mut state = self
                    .state
                    .try_borrow_mut()
                    .map_err(|_| DecodeError::Integrity)?;
                state.node_reserved = state
                    .node_reserved
                    .checked_sub(reserved)
                    .ok_or(DecodeError::Integrity)?;
                if self.terminal {
                    state.work_reserved = state
                        .work_reserved
                        .checked_sub(reserved)
                        .ok_or(DecodeError::Integrity)?;
                }
            }
            let n = result.map_err(algorithm_error)?;
            if n > width || n as u64 > left {
                return Err(DecodeError::Integrity);
            }
            if n == 0 {
                if self.position != self.expected {
                    return Err(DecodeError::Integrity);
                }
                self.decoder.finish(&self.inputs, self.expected)?;
                self.ended = true;
                return Ok(0);
            }
            self.position += n as u64;
            let mut state = self
                .state
                .try_borrow_mut()
                .map_err(|_| DecodeError::Integrity)?;
            state.node_work = state
                .node_work
                .checked_add(n as u64)
                .filter(|n| *n <= state.limits.node_work_bytes)
                .ok_or(DecodeError::Limit)?;
            if self.terminal {
                state.work = state
                    .work
                    .checked_add(n as u64)
                    .filter(|n| *n <= state.limits.work_bytes)
                    .ok_or(DecodeError::Limit)?;
            }
            check(self.control)?;
            Ok(n)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(io::Error::other)
    }
}

#[derive(Clone)]
struct LeafHandle<'a>(Rc<RefCell<Leaf<'a>>>);
impl LeafHandle<'_> {
    fn finish(&self) -> Result<(), DecodeError> {
        self.0
            .try_borrow_mut()
            .map_err(|_| DecodeError::Integrity)?
            .finish()
    }
}
impl Read for LeafHandle<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.0
            .try_borrow_mut()
            .map_err(|_| io::Error::other(DecodeError::Integrity))?
            .read(out)
            .map_err(stable)
    }
}
struct Leaf<'a> {
    source: SharedSource<'a>,
    state: SharedState,
    control: &'a Cancellation,
    range: ByteRange,
    expected_crc: Option<u32>,
    position: u64,
    crc: Crc32,
    failed: bool,
}
impl Leaf<'_> {
    fn finish(&mut self) -> Result<(), DecodeError> {
        check(self.control)?;
        if self.failed
            || self.position != self.range.length
            || self
                .expected_crc
                .is_some_and(|crc| crc != self.crc.checksum())
        {
            self.failed = true;
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
}
impl Read for Leaf<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(io::Error::other(DecodeError::Integrity));
        }
        let result = (|| {
            check(self.control)?;
            if self.position == self.range.length {
                self.finish()?;
                return Ok(0);
            }
            let remaining = {
                let state = self
                    .state
                    .try_borrow()
                    .map_err(|_| DecodeError::Integrity)?;
                state
                    .limits
                    .input_bytes
                    .checked_sub(state.input)
                    .ok_or(DecodeError::Limit)?
            };
            if remaining == 0 {
                return Err(DecodeError::Limit);
            }
            let n = out.len().min(CHUNK).min(
                (self.range.length - self.position)
                    .min(remaining)
                    .min(CHUNK as u64) as usize,
            );
            {
                let mut state = self
                    .state
                    .try_borrow_mut()
                    .map_err(|_| DecodeError::Integrity)?;
                state.input = state
                    .input
                    .checked_add(n as u64)
                    .ok_or(DecodeError::Limit)?;
            }
            let at = self
                .range
                .offset
                .checked_add(self.position)
                .ok_or(DecodeError::Integrity)?;
            let read = self
                .source
                .try_borrow_mut()
                .map_err(|_| DecodeError::Integrity)?
                .read_exact_at(at, &mut out[..n]);
            if let Err(error) = read {
                self.state
                    .try_borrow_mut()
                    .map_err(|_| DecodeError::Integrity)?
                    .source_error = Some(error);
                return Err(decode_error(error));
            }
            self.position += n as u64;
            if self.expected_crc.is_some() {
                self.crc.update(&out[..n]);
            }
            check(self.control)?;
            Ok(n)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(io::Error::other)
    }
}

enum Algorithm<'a> {
    Copy(Input<'a>),
    Deflate(Box<DeflateReader<Input<'a>>>),
    Lzma(Box<LzmaReader<Input<'a>>>),
    Lzma2(Box<Lzma2Reader<Input<'a>>>),
    Bzip2(Box<Bzip2Reader<Input<'a>>>),
    Ppmd(Box<Ppmd7Reader<Input<'a>>>),
    Aes(Box<AesReader<'a, Input<'a>>>),
    Filter(Box<FilterReader<'a>>),
    Bcj2(Box<crate::bcj2::Bcj2Reader<'a, Input<'a>>>),
}
impl Read for Algorithm<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Copy(r) => r.read(out),
            Self::Deflate(r) => r.read(out),
            Self::Lzma(r) => r.read(out),
            Self::Lzma2(r) => r.read(out),
            Self::Bzip2(r) => r.read(out),
            Self::Ppmd(r) => r.read(out),
            Self::Aes(r) => r.read(out),
            Self::Filter(r) => r.read(out),
            Self::Bcj2(r) => r.read(out),
        }
    }
}
impl Algorithm<'_> {
    fn finish(&mut self, inputs: &[u64], expected: u64) -> Result<(), DecodeError> {
        let valid = match self {
            Self::Copy(_) => true,
            Self::Deflate(r) => r.consumed_bytes() == inputs[0],
            Self::Lzma(r) => {
                r.finish().map_err(algorithm_error)?;
                r.consumed_bytes() == inputs[0] && r.decoded_bytes() == expected
            }
            Self::Lzma2(r) => {
                r.finish().map_err(algorithm_error)?;
                r.consumed_bytes() == inputs[0] && r.decoded_bytes() == expected
            }
            Self::Bzip2(r) => {
                r.finish().map_err(algorithm_error)?;
                r.consumed_bytes() == inputs[0] && r.decoded_bytes() == expected
            }
            Self::Ppmd(r) => {
                r.finish().map_err(algorithm_error)?;
                r.consumed_bytes() == inputs[0] && r.decoded_bytes() == expected
            }
            Self::Aes(r) => {
                r.finish().map_err(algorithm_error)?;
                r.consumed_bytes() == inputs[0] && r.decoded_bytes() == expected
            }
            Self::Filter(r) => {
                r.source_eof && r.start == r.ready && r.ready == r.end && r.consumed == inputs[0]
            }
            Self::Bcj2(r) => {
                r.finish().map_err(algorithm_error)?;
                r.consumed_bytes().as_slice() == inputs && r.decoded_bytes() == expected
            }
        };
        if valid {
            Ok(())
        } else {
            Err(DecodeError::Integrity)
        }
    }
}
#[allow(clippy::too_many_arguments)]
fn algorithm<'a>(
    planned: &Planned<'_>,
    mut inputs: Vec<Input<'a>>,
    lengths: &[u64],
    expected: u64,
    control: &'a Cancellation,
    password: Option<&str>,
    state: &SharedState,
) -> Result<Algorithm<'a>, DecodeError> {
    let memory = state
        .try_borrow()
        .map_err(|_| DecodeError::Integrity)?
        .limits
        .memory_bytes;
    if matches!(planned.method, Method::Bcj2) {
        let sources: [Input<'a>; 4] = inputs.try_into().map_err(|_| DecodeError::Integrity)?;
        let lengths: [u64; 4] = lengths.try_into().map_err(|_| DecodeError::Integrity)?;
        return Ok(Algorithm::Bcj2(Box::new(
            crate::bcj2::Bcj2Reader::new(sources, lengths, expected, control)
                .map_err(algorithm_error)?,
        )));
    }
    let input = inputs.pop().ok_or(DecodeError::Integrity)?;
    let packed = lengths[0];
    Ok(match &planned.method {
        Method::Copy => Algorithm::Copy(input),
        Method::Deflate => Algorithm::Deflate(Box::new(DeflateReader::new(input, packed))),
        Method::Deflate64 => {
            Algorithm::Deflate(Box::new(DeflateReader::new_deflate64(input, packed)))
        }
        Method::Lzma => Algorithm::Lzma(Box::new(
            LzmaReader::new(input, packed, Some(expected), planned.properties, memory)
                .map_err(algorithm_error)?,
        )),
        Method::Lzma2(p) => Algorithm::Lzma2(Box::new(
            Lzma2Reader::new(input, packed, expected, *p, memory).map_err(algorithm_error)?,
        )),
        Method::Bzip2 => Algorithm::Bzip2(Box::new(
            Bzip2Reader::new(input, packed, expected, memory).map_err(algorithm_error)?,
        )),
        Method::Ppmd => Algorithm::Ppmd(Box::new(
            Ppmd7Reader::new(input, packed, Some(expected), planned.properties, memory)
                .map_err(algorithm_error)?,
        )),
        Method::Aes(p) => {
            // Key lookup/derivation and with_key perform no input I/O, so this
            // short task-state borrow cannot recurse through upstream nodes.
            let mut task = state.try_borrow_mut().map_err(|_| DecodeError::Integrity)?;
            let State { keys, kdf, .. } = &mut *task;
            let key = keys.key(p, password, control, kdf)?;
            Algorithm::Aes(Box::new(AesReader::with_key(
                input, packed, expected, p, key, control,
            )?))
        }
        Method::Filter => Algorithm::Filter(Box::new(FilterReader {
            source: input,
            filter: crate::filters::ByteFilter::new(planned.id, planned.properties)?,
            bytes: Zeroizing::new([0; CHUNK + 32]),
            start: 0,
            ready: 0,
            end: 0,
            source_eof: false,
            consumed: 0,
            control,
        })),
        Method::Bcj2 => return Err(DecodeError::Integrity),
    })
}
struct FilterReader<'a> {
    source: Input<'a>,
    filter: crate::filters::ByteFilter,
    bytes: Zeroizing<[u8; CHUNK + 32]>,
    start: usize,
    ready: usize,
    end: usize,
    source_eof: bool,
    consumed: u64,
    control: &'a Cancellation,
}
impl Read for FilterReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        check(self.control).map_err(io::Error::other)?;
        while self.start == self.ready {
            if self.source_eof {
                return Ok(0);
            }
            let tail = self.end - self.ready;
            if tail > 32 {
                return Err(io::Error::other(DecodeError::Integrity));
            }
            self.bytes.copy_within(self.ready..self.end, 0);
            let n = self.source.read(&mut self.bytes[tail..tail + CHUNK])?;
            if n > CHUNK {
                return Err(io::Error::other(DecodeError::Integrity));
            }
            self.consumed = self
                .consumed
                .checked_add(n as u64)
                .ok_or_else(|| io::Error::other(DecodeError::Integrity))?;
            self.source_eof = n == 0;
            self.end = tail + n;
            self.ready = self
                .filter
                .transform(&mut self.bytes[..self.end], self.source_eof, self.control)
                .map_err(io::Error::other)?;
            self.start = 0;
            if self.ready > self.end || self.source_eof && self.ready != self.end {
                return Err(io::Error::other(DecodeError::Integrity));
            }
        }
        let n = out.len().min(CHUNK).min(self.ready - self.start);
        out[..n].copy_from_slice(&self.bytes[self.start..self.start + n]);
        self.start += n;
        Ok(n)
    }
}
fn stable(error: io::Error) -> io::Error {
    if error
        .get_ref()
        .is_some_and(|e| e.is::<DecodeError>() || e.is::<ScanError>())
    {
        io::Error::other(error.into_inner().expect("checked typed payload"))
    } else {
        error
    }
}
