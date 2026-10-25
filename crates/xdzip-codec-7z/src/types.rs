use xdzip_codec_api::{Cancellation, Entry, ScanError};

/// Exact binary method identifier. Its byte spelling is never normalized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MethodId(Vec<u8>);
impl MethodId {
    pub(crate) fn new(bytes: Vec<u8>) -> Result<Self, ScanError> {
        if bytes.is_empty() || bytes.len() > 8 || (bytes.len() == 8 && bytes[0] & 0x80 != 0) {
            return Err(ScanError::Invalid);
        }
        Ok(Self(bytes))
    }
    /// The serialized method identifier, up to eight bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}
/// Absolute range within the already-authorized concatenated source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ByteRange {
    /// Absolute starting byte.
    pub offset: u64,
    /// Exact byte count.
    pub length: u64,
}
/// One compressed input stream and its optional content checksum.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackStream {
    /// Admitted source range.
    pub range: ByteRange,
    /// IEEE CRC-32, when declared.
    pub crc: Option<u32>,
}
/// A coder description, independent of its implementation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Coder {
    /// Full binary method identifier.
    pub method: MethodId,
    /// Exact method properties.
    pub properties: Vec<u8>,
    /// Number of input ports.
    pub inputs: usize,
    /// Number of output ports.
    pub outputs: usize,
}
/// A connection between the folder's global output and input port spaces.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bind {
    /// Consumer input port.
    pub input: usize,
    /// Producer output port.
    pub output: usize,
}
/// One unbound folder input and the packed stream that supplies it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackedInput {
    /// Global input port, preserving serialized packed order.
    pub input: usize,
    /// Index into the owning Streams.packs array.
    pub pack: usize,
}
/// One ordered partition of the terminal folder output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Substream {
    /// Offset in terminal uncompressed output.
    pub offset: u64,
    /// Exact uncompressed length.
    pub length: u64,
    /// Explicit or inherited CRC-32.
    pub crc: Option<u32>,
}
/// Structurally validated coder graph. Parsing does not admit an algorithm.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Folder {
    /// Coders in serialized order.
    pub coders: Vec<Coder>,
    /// Unique input/output bindings.
    pub binds: Vec<Bind>,
    /// Ordered source inputs.
    pub packed_inputs: Vec<PackedInput>,
    /// Size of each global output port.
    pub output_sizes: Vec<u64>,
    /// The unique unbound output port.
    pub terminal_output: usize,
    /// Whole terminal-output CRC-32, when present.
    pub crc: Option<u32>,
    /// Ordered logical streams.
    pub substreams: Vec<Substream>,
    /// Producer-before-consumer coder order.
    pub topological_order: Vec<usize>,
}
impl Folder {
    /// Declared length of the terminal output.
    pub fn unpacked_size(&self) -> u64 {
        self.output_sizes[self.terminal_output]
    }
    /// Whether the graph declares the established 7z AES method.
    pub fn encrypted(&self) -> bool {
        self.coders
            .iter()
            .any(|c| c.method.as_bytes() == [0x06, 0xf1, 0x07, 0x01])
    }
}
/// Packed streams and their complete folder graphs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Streams {
    /// Physical packed streams.
    pub packs: Vec<PackStream>,
    /// Logical coder graphs.
    pub folders: Vec<Folder>,
}
/// Private location of a file's logical data stream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StreamRef {
    /// Folder index.
    pub folder: usize,
    /// Substream index within that folder.
    pub stream: usize,
}
/// Original per-file properties not representable in the public Entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FileRecord {
    /// None for an EmptyStream record, including directories and empty files.
    pub data: Option<StreamRef>,
    /// EmptyFile bit, indexed only among EmptyStream records.
    pub empty_file: bool,
    /// Anti-item deletion marker; never output permission.
    pub anti: bool,
    /// Unmodified attributes, distinguishing undefined from zero.
    pub attributes: Option<u32>,
    /// Raw Windows creation timestamp.
    pub creation_time: Option<u64>,
    /// Raw Windows access timestamp.
    pub access_time: Option<u64>,
    /// Raw Windows modification timestamp.
    pub modified_time: Option<u64>,
}
/// Complete immutable-scan result, in original FilesInfo record order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Index {
    /// Public metadata; method remains zero for 7z.
    pub rows: Vec<Entry>,
    /// Physical identity and retained metadata, one per row.
    pub files: Vec<FileRecord>,
    /// Main compressed-data geometry.
    pub streams: Streams,
}
/// Why a graph must be decoded during metadata discovery.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HeaderPurpose {
    /// EncodedHeader containing another Header.
    EncodedHeader,
    /// AdditionalStreams holding external properties.
    AdditionalProperties,
}
/// Metered random access used by an injected metadata graph decoder.
pub trait HeaderSource {
    /// Frozen total logical source length.
    fn length(&self) -> u64;
    /// Cancellation checkpoint.
    fn check(&self) -> Result<(), ScanError>;
    /// Exact bounded read; all actual source bytes consume the scan budget.
    fn read_exact_at(&mut self, offset: u64, output: &mut [u8]) -> Result<(), ScanError>;
}
/// Parser-owned bounded terminal-output collector; it exposes no filesystem.
pub trait MetadataSink {
    /// Append bytes to one folder's terminal output in offset order.
    fn write_folder(&mut self, folder: usize, bytes: &[u8]) -> Result<(), ScanError>;
}
/// Injection point for compressed or encrypted metadata graphs.
pub trait HeaderDecoder {
    /// Finish every packed reader/coder, validate input CRCs, then return success.
    /// The parser independently checks output lengths and folder/substream CRCs.
    fn decode_metadata(
        &mut self,
        purpose: HeaderPurpose,
        source: &mut dyn HeaderSource,
        streams: &Streams,
        sink: &mut dyn MetadataSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), ScanError>;
}
