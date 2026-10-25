//! Bounded method-graph execution. Container descriptions do not grant codecs.
use crate::crypto::{AesProperties, AesReader, KdfBudget};
use crate::{Coder, Folder, HeaderDecoder, HeaderPurpose, HeaderSource, MetadataSink, Streams};
use std::io::{self, Read};
use xdzip_codec_api::payload::DecodeError;
use xdzip_codec_api::{Cancellation, ScanError};
use xdzip_compression::{
    Bzip2Reader, Crc32, DeflateReader, Lzma2Reader, LzmaReader, bzip2_memory_required,
    lzma_memory_required, lzma2_memory_required,
};
use zeroize::Zeroizing;

const CHUNK: usize = 8192;
const METADATA_RESERVE: u64 = 64 * 1024 * 1024;
const GRAPH_OVERHEAD: u64 = 1024 * 1024;

/// Explicit candidate resource policy, independent of archive-declared sizes.
#[derive(Clone, Copy, Debug)]
pub struct DecodeLimits {
    /// Aggregate dictionaries, models, node output buffers and retained caches.
    pub memory_bytes: u64,
    /// Maximum complete terminal folder retained for verification.
    pub folder_bytes: u64,
    /// Aggregate verified terminal output retained by one payload session.
    pub cache_bytes: u64,
    /// Cumulative physical packed input, including metadata graph inputs.
    pub input_bytes: u64,
    /// Cumulative terminal output, matching host WORK accounting.
    pub work_bytes: u64,
    /// Independent cumulative output from all nodes, bounding intermediate work.
    pub node_work_bytes: u64,
    /// Cumulative admitted AES password-derivation rounds.
    pub kdf_rounds: u64,
    /// Cumulative admitted AES password-derivation hash-input bytes.
    pub kdf_bytes: u64,
}
impl Default for DecodeLimits {
    fn default() -> Self {
        Self {
            memory_bytes: 1024 * 1024 * 1024,
            folder_bytes: 256 * 1024 * 1024,
            cache_bytes: 256 * 1024 * 1024,
            input_bytes: 20 * 1024 * 1024 * 1024,
            work_bytes: 20 * 1024 * 1024 * 1024,
            node_work_bytes: 64 * 20 * 1024 * 1024 * 1024,
            kdf_rounds: 1 << 26,
            kdf_bytes: 1024 * 1024 * 1024,
        }
    }
}

/// Decoder shared by all header, external-property and payload graphs in a task.
/// It never opens paths, delegates to a process, or resets cumulative KDF work.
pub struct MethodGraphDecoder {
    pub(crate) limits: DecodeLimits,
    kdf: KdfBudget,
    input: u64,
    work: u64,
    node_work: u64,
    failed: bool,
    source_error: Option<ScanError>,
}
impl Default for MethodGraphDecoder {
    fn default() -> Self {
        Self::new(DecodeLimits::default())
    }
}
impl MethodGraphDecoder {
    /// Construct a task-scoped decoder with explicit bounded resource admission.
    pub fn new(limits: DecodeLimits) -> Self {
        Self {
            kdf: KdfBudget::new(limits.kdf_rounds, limits.kdf_bytes),
            limits,
            input: 0,
            work: 0,
            node_work: 0,
            failed: false,
            source_error: None,
        }
    }
    /// Admitted packed reads and actual terminal output.
    pub fn counters(&self) -> (u64, u64) {
        (self.input, self.work)
    }
    /// Actual output from all nodes, separate from host terminal WORK.
    pub fn node_work(&self) -> u64 {
        self.node_work
    }
    /// Remaining task-wide AES derivation admission.
    pub fn remaining_kdf(&self) -> (u64, u64) {
        self.kdf.remaining()
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn folder(
        &mut self,
        source: &mut dyn HeaderSource,
        streams: &Streams,
        folder_index: usize,
        retained: u64,
        control: &Cancellation,
        password: Option<&str>,
        charge: &mut dyn FnMut(u64) -> Result<(), DecodeError>,
    ) -> Result<Zeroizing<Vec<u8>>, DecodeError> {
        if self.failed {
            return Err(DecodeError::Integrity);
        }
        let result = self.execute(
            source,
            streams,
            folder_index,
            retained,
            control,
            password,
            charge,
        );
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    #[allow(clippy::too_many_arguments)]
    fn execute(
        &mut self,
        source: &mut dyn HeaderSource,
        streams: &Streams,
        folder_index: usize,
        retained: u64,
        control: &Cancellation,
        password: Option<&str>,
        charge: &mut dyn FnMut(u64) -> Result<(), DecodeError>,
    ) -> Result<Zeroizing<Vec<u8>>, DecodeError> {
        check(control)?;
        source.check().map_err(|error| {
            self.source_error = Some(error);
            decode_error(error)
        })?;
        let folder = streams
            .folders
            .get(folder_index)
            .ok_or(DecodeError::Integrity)?;
        let methods = plan(folder, streams, self.limits, retained)?;
        if folder.encrypted() && password.is_none() {
            return Err(DecodeError::PasswordRequired);
        }
        let mut packed_size = 0u64;
        for input in &folder.packed_inputs {
            let range = &streams.packs[input.pack].range;
            if range
                .offset
                .checked_add(range.length)
                .is_none_or(|n| n > source.length())
            {
                return Err(DecodeError::Integrity);
            }
            packed_size = packed_size
                .checked_add(range.length)
                .ok_or(DecodeError::Limit)?;
        }
        if self
            .input
            .checked_add(packed_size)
            .is_none_or(|n| n > self.limits.input_bytes)
        {
            return Err(DecodeError::Limit);
        }
        let mut packed = Vec::new();
        packed
            .try_reserve_exact(folder.packed_inputs.len())
            .map_err(|_| DecodeError::Limit)?;
        for slot in &folder.packed_inputs {
            check(control)?;
            let pack = &streams.packs[slot.pack];
            let mut bytes = Zeroizing::new(Vec::new());
            let count = usize::try_from(pack.range.length).map_err(|_| DecodeError::Limit)?;
            bytes
                .try_reserve_exact(count)
                .map_err(|_| DecodeError::Limit)?;
            bytes.resize(count, 0);
            let mut reader = PackedReader {
                source,
                offset: pack.range.offset,
                length: pack.range.length,
                position: 0,
                crc: Crc32::new(),
                total: &mut self.input,
                limit: self.limits.input_bytes,
                control,
                error: &mut self.source_error,
            };
            for block in bytes.chunks_mut(CHUNK) {
                reader.read_exact(block).map_err(algorithm_error)?;
            }
            if reader.position != pack.range.length
                || pack.crc.is_some_and(|crc| crc != reader.crc.checksum())
            {
                return Err(DecodeError::Integrity);
            }
            packed.push(Some(bytes));
        }
        let mut outputs: Vec<Option<Zeroizing<Vec<u8>>>> = Vec::new();
        outputs
            .try_reserve_exact(folder.output_sizes.len())
            .map_err(|_| DecodeError::Limit)?;
        outputs.resize_with(folder.output_sizes.len(), || None);
        for &node in &folder.topological_order {
            check(control)?;
            let planned = &methods[node];
            let mut inputs = Vec::new();
            inputs
                .try_reserve_exact(planned.inputs.len())
                .map_err(|_| DecodeError::Limit)?;
            for slot in &planned.inputs {
                let bytes = match *slot {
                    Slot::Bound(output) => outputs.get_mut(output).and_then(Option::take),
                    Slot::Packed(input) => packed.get_mut(input).and_then(Option::take),
                }
                .ok_or(DecodeError::Integrity)?;
                inputs.push(bytes);
            }
            let mut meter = Meter {
                limits: self.limits,
                work: &mut self.work,
                node_work: &mut self.node_work,
                terminal: planned.output == folder.terminal_output,
                charge,
            };
            let output = run_node(
                planned,
                &inputs,
                folder.output_sizes[planned.output],
                self.limits,
                &mut self.kdf,
                &mut meter,
                control,
                password,
            )?;
            outputs[planned.output] = Some(output);
        }
        if packed.iter().any(Option::is_some) {
            return Err(DecodeError::Integrity);
        }
        let output = outputs
            .get_mut(folder.terminal_output)
            .and_then(Option::take)
            .ok_or(DecodeError::Integrity)?;
        if outputs.iter().any(Option::is_some) {
            return Err(DecodeError::Integrity);
        }
        verify(folder, &output, control)?;
        Ok(output)
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
                        self.source_error
                            .take()
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
    Bzip2,
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
        [4, 2, 2] if coder.properties.is_empty() => (
            Method::Bzip2,
            bzip2_memory_required(9).map_err(algorithm_error)?,
        ),
        [6, 0xf1, 7, 1] => (
            Method::Aes(AesProperties::parse(&coder.properties)?),
            64 * 1024,
        ),
        [3, 3, 1, 0x1b] if coder.properties.is_empty() => (Method::Bcj2, 64 * 1024),
        [0] | [4, 1, 8] | [4, 2, 2] | [3, 3, 1, 0x1b] => return Err(DecodeError::Integrity),
        id => {
            crate::filters::ByteFilter::new(id, &coder.properties)?;
            (Method::Filter, 1024)
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
) -> Result<Vec<Planned<'a>>, DecodeError> {
    let count = folder.coders.len();
    if count == 0 || count > 64 || folder.topological_order.len() != count {
        return Err(DecodeError::Integrity);
    }
    let mut memory = METADATA_RESERVE
        .checked_add(retained)
        .and_then(|n| n.checked_add(GRAPH_OVERHEAD))
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
        let length = streams
            .packs
            .get(packed.pack)
            .ok_or(DecodeError::Integrity)?
            .range
            .length;
        usize::try_from(length).map_err(|_| DecodeError::Limit)?;
        memory = memory.checked_add(length).ok_or(DecodeError::Limit)?;
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
    for &output in &folder.output_sizes {
        usize::try_from(output).map_err(|_| DecodeError::Limit)?;
        memory = memory.checked_add(output).ok_or(DecodeError::Limit)?;
    }
    if memory > limits.memory_bytes || folder.unpacked_size() > limits.folder_bytes {
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

#[allow(clippy::too_many_arguments)]
fn run_node(
    planned: &Planned<'_>,
    inputs: &[Zeroizing<Vec<u8>>],
    expected: u64,
    limits: DecodeLimits,
    kdf: &mut KdfBudget,
    meter: &mut Meter<'_>,
    control: &Cancellation,
    password: Option<&str>,
) -> Result<Zeroizing<Vec<u8>>, DecodeError> {
    let mut output = Zeroizing::new(Vec::new());
    output
        .try_reserve_exact(usize::try_from(expected).map_err(|_| DecodeError::Limit)?)
        .map_err(|_| DecodeError::Limit)?;
    if matches!(planned.method, Method::Bcj2) {
        let sources: [Stable<SliceInput<'_>>; 4] = std::array::from_fn(|i| {
            Stable(SliceInput {
                bytes: &inputs[i],
                position: 0,
                control,
            })
        });
        let lengths = std::array::from_fn(|i| inputs[i].len() as u64);
        let mut reader = crate::bcj2::Bcj2Reader::new(sources, lengths, expected, control)
            .map_err(algorithm_error)?;
        collect(&mut reader, &mut output, expected, meter, control)?;
        reader.finish().map_err(algorithm_error)?;
        if reader.consumed_bytes() != lengths || reader.decoded_bytes() != expected {
            return Err(DecodeError::Integrity);
        }
        return Ok(output);
    }
    let mut input = SliceInput {
        bytes: &inputs[0],
        position: 0,
        control,
    };
    let packed = inputs[0].len() as u64;
    let mut stable = Stable(&mut input);
    match &planned.method {
        Method::Copy => collect(&mut stable, &mut output, expected, meter, control)?,
        Method::Filter => {
            collect(&mut stable, &mut output, expected, meter, control)?;
            let mut filter = crate::filters::ByteFilter::new(planned.id, planned.properties)?;
            if filter.transform(&mut output, true, control)? != output.len() {
                return Err(DecodeError::Integrity);
            }
        }
        Method::Bcj2 => return Err(DecodeError::Integrity),
        Method::Lzma => {
            let mut reader = LzmaReader::new(
                &mut stable,
                packed,
                Some(expected),
                planned.properties,
                limits.memory_bytes,
            )
            .map_err(algorithm_error)?;
            collect(&mut reader, &mut output, expected, meter, control)?;
            reader.finish().map_err(algorithm_error)?;
            if reader.consumed_bytes() != packed || reader.decoded_bytes() != expected {
                return Err(DecodeError::Integrity);
            }
        }
        Method::Lzma2(property) => {
            let mut reader = Lzma2Reader::new(
                &mut stable,
                packed,
                expected,
                *property,
                limits.memory_bytes,
            )
            .map_err(algorithm_error)?;
            collect(&mut reader, &mut output, expected, meter, control)?;
            reader.finish().map_err(algorithm_error)?;
            if reader.consumed_bytes() != packed || reader.decoded_bytes() != expected {
                return Err(DecodeError::Integrity);
            }
        }
        Method::Deflate => {
            let mut reader = DeflateReader::new(&mut stable, packed);
            collect(&mut reader, &mut output, expected, meter, control)?;
            if reader.consumed_bytes() != packed {
                return Err(DecodeError::Integrity);
            }
        }
        Method::Bzip2 => {
            let mut reader = Bzip2Reader::new(&mut stable, packed, expected, limits.memory_bytes)
                .map_err(algorithm_error)?;
            collect(&mut reader, &mut output, expected, meter, control)?;
            reader.finish().map_err(algorithm_error)?;
            if reader.consumed_bytes() != packed || reader.decoded_bytes() != expected {
                return Err(DecodeError::Integrity);
            }
        }
        Method::Aes(properties) => {
            let mut reader = AesReader::new(
                &mut stable,
                packed,
                expected,
                properties,
                password,
                control,
                kdf,
            )?;
            collect(&mut reader, &mut output, expected, meter, control)?;
            reader.finish().map_err(algorithm_error)?;
            if reader.consumed_bytes() != packed || reader.decoded_bytes() != expected {
                return Err(DecodeError::Integrity);
            }
        }
    }
    Ok(output)
}
struct Meter<'a> {
    limits: DecodeLimits,
    work: &'a mut u64,
    node_work: &'a mut u64,
    terminal: bool,
    charge: &'a mut dyn FnMut(u64) -> Result<(), DecodeError>,
}
impl Meter<'_> {
    fn admit(&mut self, count: u64) -> Result<(), DecodeError> {
        *self.node_work = self
            .node_work
            .checked_add(count)
            .filter(|n| *n <= self.limits.node_work_bytes)
            .ok_or(DecodeError::Limit)?;
        if self.terminal {
            *self.work = self
                .work
                .checked_add(count)
                .filter(|n| *n <= self.limits.work_bytes)
                .ok_or(DecodeError::Limit)?;
            (self.charge)(count)?;
        }
        Ok(())
    }
}
fn collect(
    reader: &mut dyn Read,
    output: &mut Vec<u8>,
    expected: u64,
    meter: &mut Meter<'_>,
    control: &Cancellation,
) -> Result<(), DecodeError> {
    let mut block = Zeroizing::new([0u8; CHUNK]);
    loop {
        check(control)?;
        let count = reader.read(&mut block[..]).map_err(algorithm_error)?;
        if count > CHUNK {
            return Err(DecodeError::Integrity);
        }
        if count == 0 {
            break;
        }
        meter.admit(count as u64)?;
        if (output.len() as u64)
            .checked_add(count as u64)
            .is_none_or(|n| n > expected)
        {
            return Err(DecodeError::Integrity);
        }
        output.extend_from_slice(&block[..count]);
    }
    check(control)?;
    if output.len() as u64 != expected {
        return Err(DecodeError::Integrity);
    }
    Ok(())
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
struct Stable<R>(R);
impl<R: Read> Read for Stable<R> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.0.read(out).map_err(|error| {
            if error
                .get_ref()
                .is_some_and(|e| e.is::<DecodeError>() || e.is::<ScanError>())
            {
                io::Error::other(error.into_inner().expect("typed payload checked"))
            } else {
                error
            }
        })
    }
}
struct SliceInput<'a> {
    bytes: &'a [u8],
    position: usize,
    control: &'a Cancellation,
}
impl Read for SliceInput<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        check(self.control).map_err(io::Error::other)?;
        let count = out.len().min(self.bytes.len() - self.position).min(CHUNK);
        out[..count].copy_from_slice(&self.bytes[self.position..self.position + count]);
        self.position += count;
        Ok(count)
    }
}
struct PackedReader<'a> {
    source: &'a mut dyn HeaderSource,
    offset: u64,
    length: u64,
    position: u64,
    crc: Crc32,
    total: &'a mut u64,
    limit: u64,
    control: &'a Cancellation,
    error: &'a mut Option<ScanError>,
}
impl Read for PackedReader<'_> {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        check(self.control).map_err(io::Error::other)?;
        let count = out
            .len()
            .min(CHUNK)
            .min((self.length - self.position).min(CHUNK as u64) as usize);
        if count == 0 {
            return Ok(0);
        }
        *self.total = self
            .total
            .checked_add(count as u64)
            .filter(|n| *n <= self.limit)
            .ok_or_else(|| io::Error::other(DecodeError::Limit))?;
        let at = self
            .offset
            .checked_add(self.position)
            .ok_or_else(|| io::Error::other(DecodeError::Integrity))?;
        self.source
            .read_exact_at(at, &mut out[..count])
            .map_err(|error| {
                *self.error = Some(error);
                io::Error::other(error)
            })?;
        self.crc.update(&out[..count]);
        self.position += count as u64;
        Ok(count)
    }
}
