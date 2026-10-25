//! Bounded 7z method plans shared by metadata collection and payload streams.
mod stream;
use crate::crypto::{AesKeyCache, AesProperties, KdfBudget};
use crate::{Coder, Folder, HeaderDecoder, HeaderPurpose, HeaderSource, MetadataSink, Streams};
use std::cell::RefCell;
use std::io;
use std::rc::Rc;
pub(crate) use stream::{FolderStream, SharedSource};
use xdzip_codec_api::payload::DecodeError;
use xdzip_codec_api::{Cancellation, ScanError};
use xdzip_compression::{
    Crc32, bzip2_memory_required, lzma_memory_required, lzma2_memory_required,
    ppmd7_memory_required,
};
use zeroize::Zeroizing;
const CHUNK: usize = 8192;
const METADATA_RESERVE: u64 = 64 * 1024 * 1024;
const GRAPH_OVERHEAD: u64 = 1024 * 1024;
/// Explicit task resource policy, independent of archive-declared lengths.
#[derive(Clone, Copy, Debug)]
pub struct DecodeLimits {
    /// Sum of dictionaries, models, node buffers and retained metadata.
    pub memory_bytes: u64,
    /// Cumulative physical packed input, including graph restarts.
    pub input_bytes: u64,
    /// Cumulative terminal output, matching host WORK.
    pub work_bytes: u64,
    /// Independent cumulative output from all intermediate and terminal nodes.
    pub node_work_bytes: u64,
    /// Cumulative AES derivation rounds.
    pub kdf_rounds: u64,
    /// Cumulative AES derivation hash-input bytes.
    pub kdf_bytes: u64,
}
impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            memory_bytes: 1 << 30,
            input_bytes: 20 << 30,
            work_bytes: 20 << 30,
            node_work_bytes: 64 * ((20 << 30) + METADATA_RESERVE),
            kdf_rounds: 1 << 26,
            kdf_bytes: 1 << 30,
        }
    }
}
struct State {
    limits: DecodeLimits,
    kdf: KdfBudget,
    keys: AesKeyCache,
    input: u64,
    work: u64,
    node_work: u64,
    work_reserved: u64,
    node_reserved: u64,
    source_error: Option<ScanError>,
    payload: bool,
}
/// One task's method admission and cumulative work, shared across graph restarts.
pub struct MethodGraphDecoder {
    pub(crate) limits: DecodeLimits,
    state: Rc<RefCell<State>>,
    failed: bool,
}
impl Default for MethodGraphDecoder {
    fn default() -> Self {
        Self::new(DecodeLimits::default())
    }
}
impl MethodGraphDecoder {
    pub(crate) fn begin_payload(&mut self) -> Result<(), DecodeError> {
        let mut state = self
            .state
            .try_borrow_mut()
            .map_err(|_| DecodeError::Integrity)?;
        if state.payload || state.work_reserved != 0 || state.node_reserved != 0 {
            return Err(DecodeError::Integrity);
        }
        // Metadata has a separate 64MiB admission. This phase transition starts
        // payload I/O/terminal counters once; graph restarts refund no budget.
        // Node work and KDF admission remain cumulative across both phases.
        state.input = 0;
        state.work = 0;
        state.payload = true;
        Ok(())
    }
    /// Start a bounded task; graph restarts retain the same input/work/KDF state.
    pub fn new(limits: DecodeLimits) -> Self {
        Self {
            limits,
            state: Rc::new(RefCell::new(State {
                limits,
                kdf: KdfBudget::new(limits.kdf_rounds, limits.kdf_bytes),
                keys: AesKeyCache::default(),
                input: 0,
                work: 0,
                node_work: 0,
                work_reserved: 0,
                node_reserved: 0,
                source_error: None,
                payload: false,
            })),
            failed: false,
        }
    }
    /// Admitted physical packed input and actual terminal output.
    pub fn counters(&self) -> (u64, u64) {
        self.state
            .try_borrow()
            .map_or((u64::MAX, u64::MAX), |s| (s.input, s.work))
    }
    /// Actual output from all nodes, separately admitted from host WORK.
    pub fn node_work(&self) -> u64 {
        self.state.try_borrow().map_or(u64::MAX, |s| s.node_work)
    }
    /// Remaining task-wide password derivation allowance.
    pub fn remaining_kdf(&self) -> (u64, u64) {
        self.state
            .try_borrow()
            .map_or((0, 0), |s| s.kdf.remaining())
    }
    pub(crate) fn stream<'a>(
        &mut self,
        source: SharedSource<'a>,
        streams: &Streams,
        index: usize,
        control: &'a Cancellation,
        password: Option<&str>,
    ) -> Result<FolderStream<'a>, DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let result = FolderStream::new(
            source,
            streams,
            index,
            control,
            password,
            self.state.clone(),
            false,
        );
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn folder(
        &mut self,
        source: &mut dyn HeaderSource,
        streams: &Streams,
        index: usize,
        retained: u64,
        control: &Cancellation,
        password: Option<&str>,
        charge: &mut dyn FnMut(u64) -> Result<(), DecodeError>,
    ) -> Result<Zeroizing<Vec<u8>>, DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let result = (|| {
            let folder = streams.folders.get(index).ok_or(DecodeError::Integrity)?;
            plan(folder, streams, self.limits, retained, true)?;
            let source: SharedSource<'_> = Rc::new(RefCell::new(Box::new(Borrowed(source))));
            let mut graph = FolderStream::new(
                source,
                streams,
                index,
                control,
                password,
                self.state.clone(),
                true,
            )?;
            let mut output = Zeroizing::new(Vec::new());
            let size = usize::try_from(folder.unpacked_size()).map_err(|_| DecodeError::Limit)?;
            output
                .try_reserve_exact(size)
                .map_err(|_| DecodeError::Limit)?;
            let mut bytes = Zeroizing::new([0; CHUNK]);
            loop {
                check(control)?;
                let n = graph.read(&mut bytes[..])?;
                if n == 0 {
                    break;
                }
                charge(n as u64)?;
                output.extend_from_slice(&bytes[..n]);
            }
            graph.finish()?;
            verify(folder, &output, control)?;
            Ok(output)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
struct Borrowed<'a>(&'a mut dyn HeaderSource);
impl HeaderSource for Borrowed<'_> {
    fn length(&self) -> u64 {
        self.0.length()
    }
    fn check(&self) -> Result<(), ScanError> {
        self.0.check()
    }
    fn read_exact_at(&mut self, at: u64, out: &mut [u8]) -> Result<(), ScanError> {
        self.0.read_exact_at(at, out)
    }
}
impl HeaderDecoder for MethodGraphDecoder {
    fn decode_metadata(
        &mut self,
        _purpose: HeaderPurpose,
        source: &mut dyn HeaderSource,
        streams: &Streams,
        sink: &mut dyn MetadataSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), ScanError> {
        if self.failed {
            return Err(ScanError::Invalid);
        }
        let result = (|| {
            for folder in 0..streams.folders.len() {
                let output = self
                    .folder(source, streams, folder, 0, control, password, &mut |_| {
                        Ok(())
                    })
                    .map_err(|error| {
                        self.state
                            .try_borrow_mut()
                            .ok()
                            .and_then(|mut s| s.source_error.take())
                            .unwrap_or_else(|| scan_error(error))
                    })?;
                for block in output.chunks(CHUNK) {
                    control.check()?;
                    sink.write_folder(folder, block)?;
                }
            }
            control.check()
        })();
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}
enum Method {
    Copy,
    Lzma,
    Lzma2(u8),
    Deflate,
    Deflate64,
    Bzip2,
    Ppmd,
    Aes(AesProperties),
    Filter,
    Bcj2,
}
#[derive(Clone, Copy)]
enum Slot {
    Bound(usize),
    Packed(usize),
}
struct Planned<'a> {
    method: Method,
    id: &'a [u8],
    properties: &'a [u8],
    inputs: Vec<Slot>,
    output: usize,
}
fn method(coder: &Coder) -> Result<(Method, u64), DecodeError> {
    if coder.outputs != 1 {
        return Err(DecodeError::Unsupported);
    }
    let result = match coder.method.as_bytes() {
        [0] if coder.properties.is_empty() => (Method::Copy, 0),
        [3, 1, 1] => (
            Method::Lzma,
            lzma_memory_required(&coder.properties).map_err(algorithm_error)?,
        ),
        [0x21] => {
            let [property] = coder.properties.as_slice() else {
                return Err(DecodeError::Integrity);
            };
            (
                Method::Lzma2(*property),
                lzma2_memory_required(*property).map_err(algorithm_error)?,
            )
        }
        [4, 1, 8] if coder.properties.is_empty() => (Method::Deflate, 128 * 1024),
        [4, 1, 9] if coder.properties.is_empty() => (Method::Deflate64, 128 * 1024),
        [4, 2, 2] if coder.properties.is_empty() => (
            Method::Bzip2,
            bzip2_memory_required(9).map_err(algorithm_error)?,
        ),
        [3, 4, 1] => (
            Method::Ppmd,
            ppmd7_memory_required(&coder.properties).map_err(algorithm_error)?,
        ),
        [6, 0xf1, 7, 1] => (
            Method::Aes(AesProperties::parse(&coder.properties)?),
            64 * 1024,
        ),
        [3, 3, 1, 0x1b] if coder.properties.is_empty() => (Method::Bcj2, 64 * 1024),
        [0] | [4, 1, 8] | [4, 1, 9] | [4, 2, 2] | [3, 3, 1, 0x1b] => {
            return Err(DecodeError::Integrity);
        }
        id => {
            crate::filters::ByteFilter::new(id, &coder.properties)?;
            (Method::Filter, 32 * 1024)
        }
    };
    if coder.inputs
        != if matches!(result.0, Method::Bcj2) {
            4
        } else {
            1
        }
    {
        return Err(DecodeError::Unsupported);
    }
    Ok(result)
}
fn plan<'a>(
    folder: &'a Folder,
    streams: &Streams,
    limits: DecodeLimits,
    retained: u64,
    materialized: bool,
) -> Result<Vec<Planned<'a>>, DecodeError> {
    let count = folder.coders.len();
    if count == 0 || count > 64 || folder.topological_order.len() != count {
        return Err(DecodeError::Integrity);
    }
    let mut memory = METADATA_RESERVE
        .checked_add(retained)
        .and_then(|n| n.checked_add(GRAPH_OVERHEAD))
        .and_then(|n| n.checked_add(AesKeyCache::MEMORY_BYTES))
        .ok_or(DecodeError::Limit)?;
    let mut methods = Vec::new();
    methods
        .try_reserve_exact(count)
        .map_err(|_| DecodeError::Limit)?;
    let (mut input_total, mut output_total) = (0usize, 0usize);
    let mut input_owner = Vec::new();
    let mut output_owner = Vec::new();
    for (node, coder) in folder.coders.iter().enumerate() {
        let (method, workspace) = method(coder)?;
        input_total = input_total
            .checked_add(coder.inputs)
            .filter(|n| *n <= 256)
            .ok_or(DecodeError::Integrity)?;
        output_total = output_total
            .checked_add(coder.outputs)
            .filter(|n| *n <= 256)
            .ok_or(DecodeError::Integrity)?;
        memory = memory
            .checked_add(workspace)
            .and_then(|n| n.checked_add(coder.properties.len() as u64))
            .ok_or(DecodeError::Limit)?;
        input_owner.extend(std::iter::repeat_n(node, coder.inputs));
        output_owner.extend(std::iter::repeat_n(node, coder.outputs));
        methods.push(Planned {
            method,
            id: coder.method.as_bytes(),
            properties: &coder.properties,
            inputs: Vec::new(),
            output: output_total - 1,
        });
    }
    if folder.output_sizes.len() != output_total
        || folder.terminal_output >= output_total
        || folder.binds.len() != output_total - 1
        || folder.packed_inputs.len() != input_total - folder.binds.len()
    {
        return Err(DecodeError::Integrity);
    }
    let mut used_input = [false; 256];
    let mut used_output = [false; 256];
    for bind in &folder.binds {
        if bind.input >= input_total
            || bind.output >= output_total
            || used_input[bind.input]
            || used_output[bind.output]
        {
            return Err(DecodeError::Integrity);
        }
        used_input[bind.input] = true;
        used_output[bind.output] = true;
    }
    if used_output[folder.terminal_output] {
        return Err(DecodeError::Integrity);
    }
    for (index, packed) in folder.packed_inputs.iter().enumerate() {
        if packed.input >= input_total
            || used_input[packed.input]
            || folder.packed_inputs[..index]
                .iter()
                .any(|p| p.pack == packed.pack)
        {
            return Err(DecodeError::Integrity);
        }
        used_input[packed.input] = true;
        streams
            .packs
            .get(packed.pack)
            .ok_or(DecodeError::Integrity)?;
    }
    if used_input[..input_total].iter().any(|used| !*used) {
        return Err(DecodeError::Integrity);
    }
    let mut visited = [false; 64];
    for &node in &folder.topological_order {
        if node >= count
            || visited[node]
            || folder
                .binds
                .iter()
                .any(|b| input_owner[b.input] == node && !visited[output_owner[b.output]])
        {
            return Err(DecodeError::Integrity);
        }
        visited[node] = true;
    }
    if materialized {
        if folder.unpacked_size() > crate::source::METADATA_LIMIT {
            return Err(DecodeError::Limit);
        }
        memory = memory
            .checked_add(folder.unpacked_size())
            .ok_or(DecodeError::Limit)?;
    }
    if memory > limits.memory_bytes {
        return Err(DecodeError::Limit);
    }
    for (input, &owner) in input_owner.iter().enumerate() {
        let slot = if let Some(bind) = folder.binds.iter().find(|b| b.input == input) {
            Slot::Bound(bind.output)
        } else {
            Slot::Packed(
                folder
                    .packed_inputs
                    .iter()
                    .position(|p| p.input == input)
                    .ok_or(DecodeError::Integrity)?,
            )
        };
        methods[owner].inputs.push(slot);
    }
    for node in &methods {
        let lengths = node
            .inputs
            .iter()
            .map(|slot| match *slot {
                Slot::Bound(output) => folder.output_sizes[output],
                Slot::Packed(input) => streams.packs[folder.packed_inputs[input].pack].range.length,
            })
            .collect::<Vec<_>>();
        let output = folder.output_sizes[node.output];
        let input = lengths[0];
        if matches!(node.method, Method::Copy | Method::Filter) && input != output {
            return Err(DecodeError::Integrity);
        }
        if matches!(node.method, Method::Aes(_))
            && (!input.is_multiple_of(16)
                || input < output.checked_add(15).ok_or(DecodeError::Limit)? / 16 * 16)
        {
            return Err(DecodeError::Integrity);
        }
        if matches!(node.method, Method::Bcj2)
            && (!lengths[1].is_multiple_of(4)
                || !lengths[2].is_multiple_of(4)
                || lengths[3] < 5
                || lengths[0]
                    .checked_add(lengths[1])
                    .and_then(|n| n.checked_add(lengths[2]))
                    != Some(output))
        {
            return Err(DecodeError::Integrity);
        }
    }
    Ok(methods)
}

fn verify(folder: &Folder, output: &[u8], control: &Cancellation) -> Result<(), DecodeError> {
    if output.len() as u64 != folder.unpacked_size() {
        return Err(DecodeError::Integrity);
    }
    if let Some(expected) = folder.crc
        && expected != crc_bytes(output, control)?
    {
        return Err(DecodeError::Integrity);
    }
    for stream in &folder.substreams {
        check(control)?;
        let start = usize::try_from(stream.offset).map_err(|_| DecodeError::Integrity)?;
        let end = stream
            .offset
            .checked_add(stream.length)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or(DecodeError::Integrity)?;
        let data = output.get(start..end).ok_or(DecodeError::Integrity)?;
        if let Some(expected) = stream.crc
            && expected != crc_bytes(data, control)?
        {
            return Err(DecodeError::Integrity);
        }
    }
    check(control)
}
fn crc_bytes(bytes: &[u8], control: &Cancellation) -> Result<u32, DecodeError> {
    let mut crc = Crc32::new();
    for block in bytes.chunks(CHUNK) {
        check(control)?;
        crc.update(block);
    }
    check(control)?;
    Ok(crc.checksum())
}
fn check(control: &Cancellation) -> Result<(), DecodeError> {
    control.check().map_err(decode_error)
}
pub(crate) fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Unsupported => DecodeError::Unsupported,
        ScanError::PasswordRequired => DecodeError::PasswordRequired,
        ScanError::WrongPassword => DecodeError::WrongPassword,
        ScanError::PasswordOrIntegrity => DecodeError::PasswordOrIntegrity,
        _ => DecodeError::Integrity,
    }
}
pub(crate) fn scan_error(error: DecodeError) -> ScanError {
    match error {
        DecodeError::SourceIo => ScanError::Io,
        DecodeError::Limit => ScanError::Limit,
        DecodeError::Cancelled => ScanError::Cancelled,
        DecodeError::Unsupported => ScanError::Unsupported,
        DecodeError::PasswordRequired => ScanError::PasswordRequired,
        DecodeError::WrongPassword => ScanError::WrongPassword,
        DecodeError::PasswordOrIntegrity => ScanError::PasswordOrIntegrity,
        _ => ScanError::Invalid,
    }
}
pub(crate) fn algorithm_error(error: io::Error) -> DecodeError {
    error
        .get_ref()
        .and_then(|e| e.downcast_ref::<DecodeError>())
        .copied()
        .or_else(|| {
            error
                .get_ref()
                .and_then(|e| e.downcast_ref::<ScanError>())
                .copied()
                .map(decode_error)
        })
        .unwrap_or(if error.kind() == io::ErrorKind::OutOfMemory {
            DecodeError::Limit
        } else {
            DecodeError::Integrity
        })
}
