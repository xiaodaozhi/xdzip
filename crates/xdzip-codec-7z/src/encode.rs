//! Host-owned streaming 7z staging writer. No path or publication authority.
use crate::{
    SevenZipCodec,
    crypto::{AesKeyCache, AesProperties, AesWriter, KdfBudget},
};
use std::io::{self, Read, Write};
use xdzip_codec_api::encode::{
    CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeOutput, EncodeProgress,
    EncodeSettings,
};
use xdzip_codec_api::payload::DecodeError;
use xdzip_codec_api::{Cancellation, MAX_ENTRIES, ScanError};
use xdzip_compression::{Crc32, Lzma2Writer, lzma2_writer_memory_required};
#[path = "encode/wire.rs"]
mod wire;
pub(crate) const TASK_LIMIT: u64 = 20 << 30;
pub(crate) const FILE_LIMIT: u64 = 8 << 30;
pub(crate) const MEMORY_LIMIT: u64 = 1 << 30;
pub(crate) const METADATA_MEMORY: u64 = 192 << 20;
const META_LIMIT: usize = 64 << 20;
const NAME_LIMIT: usize = 4096;
const NAMES_LIMIT: usize = 16 << 20;
const CHUNK: usize = 8192;
#[derive(Clone, Copy)]
pub(crate) struct FileProperties {
    pub(crate) attributes: Option<u32>,
    pub(crate) creation_time: Option<u64>,
    pub(crate) access_time: Option<u64>,
    pub(crate) modified_time: Option<u64>,
}
impl FileProperties {
    pub(crate) fn new(entry: &EncodeEntry) -> Self {
        Self {
            attributes: Some(if entry.directory { 16 } else { 32 }),
            creation_time: None,
            access_time: None,
            modified_time: u64::try_from(
                (i128::from(entry.modified_unix) + 11_644_473_600) * 10_000_000,
            )
            .ok(),
        }
    }
}
#[derive(Debug)]
struct Failure(EncodeError);
impl std::fmt::Display for Failure {
    fn fmt(&self, output: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(output, "encode_{:?}", self.0)
    }
}
impl std::error::Error for Failure {}
fn fault(error: EncodeError) -> io::Error {
    io::Error::other(Failure(error))
}
pub(crate) fn scan_error(error: ScanError) -> EncodeError {
    match error {
        ScanError::Cancelled => EncodeError::Cancelled,
        ScanError::Limit => EncodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => EncodeError::Source,
        ScanError::Unsupported
        | ScanError::PasswordRequired
        | ScanError::WrongPassword
        | ScanError::PasswordOrIntegrity
        | ScanError::MissingVolume => EncodeError::Unsupported,
        ScanError::Invalid => EncodeError::Invalid,
    }
}
pub(crate) fn decode_error(error: DecodeError) -> EncodeError {
    match error {
        DecodeError::Cancelled => EncodeError::Cancelled,
        DecodeError::Limit => EncodeError::Limit,
        DecodeError::SourceIo => EncodeError::Source,
        DecodeError::Output => EncodeError::Output,
        DecodeError::Unsupported
        | DecodeError::PasswordRequired
        | DecodeError::WrongPassword
        | DecodeError::PasswordOrIntegrity => EncodeError::Unsupported,
        DecodeError::Integrity => EncodeError::Invalid,
    }
}
fn io_error(error: io::Error, fallback: EncodeError) -> EncodeError {
    if let Some(payload) = error.get_ref() {
        if let Some(error) = payload.downcast_ref::<Failure>() {
            return error.0;
        }
        if let Some(error) = payload.downcast_ref::<ScanError>() {
            return scan_error(*error);
        }
        if let Some(error) = payload.downcast_ref::<DecodeError>() {
            return decode_error(*error);
        }
    }
    if error.kind() == io::ErrorKind::OutOfMemory {
        EncodeError::Limit
    } else {
        fallback
    }
}
fn check(control: &Cancellation) -> Result<(), EncodeError> {
    control.check().map_err(scan_error)
}
fn retry(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none()
}
fn write_all(writer: &mut dyn Write, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        match writer.write(bytes) {
            Err(error) if retry(&error) => continue,
            Err(error) => return Err(error),
            Ok(0) => return Err(fault(EncodeError::Output)),
            Ok(count) if count <= bytes.len() => bytes = &bytes[count..],
            Ok(_) => return Err(fault(EncodeError::Output)),
        }
    }
    Ok(())
}
pub(crate) fn source_read(
    source: &mut dyn Read,
    bytes: &mut [u8],
    control: &Cancellation,
) -> Result<usize, EncodeError> {
    loop {
        check(control)?;
        match source.read(bytes) {
            Err(error) if retry(&error) => continue,
            Err(error) => return Err(io_error(error, EncodeError::Source)),
            Ok(count) if count <= bytes.len() => return Ok(count),
            Ok(_) => return Err(EncodeError::Source),
        }
    }
}
pub(crate) fn admit(
    entries: &[EncodeEntry],
    settings: EncodeSettings<'_>,
    retained: u64,
) -> Result<u64, EncodeError> {
    if !matches!(
        (settings.method, settings.level),
        ("copy", 0) | ("lzma2", 1 | 6 | 9)
    ) || settings.encrypt_headers && settings.password.is_none()
    {
        return Err(EncodeError::Unsupported);
    }
    if settings
        .password
        .is_some_and(|password| password.len() > 4096)
    {
        return Err(EncodeError::Limit);
    }
    if entries.len() > MAX_ENTRIES {
        return Err(EncodeError::Limit);
    }
    let mut total = 0u64;
    let mut names = 0usize;
    let mut owned = (entries.len() as u64)
        .checked_mul(512)
        .ok_or(EncodeError::Limit)?;
    for entry in entries {
        if entry.name.len() > NAME_LIMIT {
            return Err(EncodeError::Limit);
        }
        if entry.directory && entry.size != 0
            || entry.name.is_empty()
            || xdzip_domain::path::components(entry.name.as_bytes(), false).is_none()
            || entry.name.ends_with('/')
            || entry.name.contains('\0')
        {
            return Err(EncodeError::Invalid);
        }
        if entry.name.len() > NAME_LIMIT {
            return Err(EncodeError::Limit);
        }
        names = names
            .checked_add(entry.name.len())
            .ok_or(EncodeError::Limit)?;
        owned = owned
            .checked_add(entry.name.capacity() as u64)
            .ok_or(EncodeError::Limit)?;
        if names > NAMES_LIMIT || entry.size > FILE_LIMIT {
            return Err(EncodeError::Limit);
        }
        total = total.checked_add(entry.size).ok_or(EncodeError::Limit)?;
    }
    if total > TASK_LIMIT {
        return Err(EncodeError::Limit);
    }
    let encoder = if settings.method == "lzma2" {
        lzma2_writer_memory_required(settings.level)
            .map_err(|e| io_error(e, EncodeError::Unsupported))?
    } else {
        0
    };
    let memory = retained
        .checked_add(METADATA_MEMORY + AesKeyCache::MEMORY_BYTES)
        .and_then(|v| v.checked_add(owned))
        .and_then(|v| v.checked_add(encoder))
        .ok_or(EncodeError::Limit)?;
    if memory > MEMORY_LIMIT {
        return Err(EncodeError::Limit);
    }
    Ok(total)
}
struct Stage<'a> {
    output: &'a mut dyn EncodeOutput,
    control: &'a Cancellation,
    offset: u64,
    written: u64,
}
impl Stage<'_> {
    fn patch(&mut self, offset: u64, bytes: &[u8]) -> Result<(), EncodeError> {
        check(self.control)?;
        let end = offset
            .checked_add(bytes.len() as u64)
            .ok_or(EncodeError::Limit)?;
        let written = self
            .written
            .checked_add(bytes.len() as u64)
            .ok_or(EncodeError::Limit)?;
        if end > TASK_LIMIT || written > TASK_LIMIT {
            return Err(EncodeError::Limit);
        }
        self.output.write_at(offset, bytes)?;
        self.written = written;
        check(self.control)
    }
    fn append(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        for bytes in bytes.chunks(CHUNK) {
            self.patch(self.offset, bytes)?;
            self.offset += bytes.len() as u64;
        }
        Ok(())
    }
}
struct Pack<'a, 'b> {
    stage: &'a mut Stage<'b>,
    count: u64,
    crc: Crc32,
    buffer: [u8; CHUNK],
    filled: usize,
}
impl Pack<'_, '_> {
    fn complete(&mut self) -> io::Result<()> {
        check(self.stage.control).map_err(fault)?;
        if self.filled != 0 {
            self.stage
                .append(&self.buffer[..self.filled])
                .map_err(fault)?;
            self.filled = 0;
        }
        Ok(())
    }
}
impl Write for Pack<'_, '_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        check(self.stage.control).map_err(fault)?;
        let bytes = &bytes[..bytes.len().min(CHUNK)];
        let mut copied = 0;
        while copied != bytes.len() {
            let count = (bytes.len() - copied).min(CHUNK - self.filled);
            self.buffer[self.filled..self.filled + count]
                .copy_from_slice(&bytes[copied..copied + count]);
            self.filled += count;
            copied += count;
            if self.filled == CHUNK {
                self.complete()?;
            }
        }
        self.count = self
            .count
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| fault(EncodeError::Limit))?;
        self.crc.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.complete()
    }
}
struct WrittenFolder {
    first: usize,
    count: usize,
    packed: u64,
    packed_crc: u32,
    unpacked: u64,
    crc: u32,
    intermediate: u64,
    aes: Option<Vec<u8>>,
}
struct InputState<'a> {
    input: &'a dyn EncodeInput,
    control: &'a Cancellation,
    progress: &'a mut dyn FnMut(EncodeProgress<'_>),
    total: u64,
    crcs: Vec<u32>,
}
impl InputState<'_> {
    fn file(
        &mut self,
        index: usize,
        sink: &mut dyn Write,
        folder_crc: &mut Crc32,
    ) -> Result<(), EncodeError> {
        check(self.control)?;
        let entry = &self.input.entries()[index];
        let mut source = self.input.open(index)?;
        let mut remaining = entry.size;
        let mut crc = Crc32::new();
        let mut buffer = [0; CHUNK];
        while remaining != 0 {
            let width = remaining.min(CHUNK as u64) as usize;
            let count = source_read(&mut source, &mut buffer[..width], self.control)?;
            if count == 0 {
                return Err(EncodeError::Source);
            }
            self.total = self
                .total
                .checked_add(count as u64)
                .ok_or(EncodeError::Limit)?;
            if self.total > TASK_LIMIT {
                return Err(EncodeError::Limit);
            }
            (self.progress)(EncodeProgress {
                name: &entry.name,
                input_bytes: self.total,
            });
            check(self.control)?;
            write_all(sink, &buffer[..count]).map_err(|e| io_error(e, EncodeError::Output))?;
            crc.update(&buffer[..count]);
            folder_crc.update(&buffer[..count]);
            remaining -= count as u64;
        }
        if source_read(&mut source, &mut buffer[..1], self.control)? != 0 {
            return Err(EncodeError::Source);
        }
        self.crcs[index] = crc.checksum();
        check(self.control)
    }
    fn files(
        &mut self,
        indices: &[usize],
        sink: &mut dyn Write,
        crc: &mut Crc32,
    ) -> Result<(), EncodeError> {
        for &index in indices {
            self.file(index, sink, crc)?;
        }
        Ok(())
    }
}
fn folder(
    stage: &mut Stage<'_>,
    state: &mut InputState<'_>,
    indices: &[usize],
    first: usize,
    settings: EncodeSettings<'_>,
    kdf: &mut KdfBudget,
    cache: &mut AesKeyCache,
) -> Result<WrittenFolder, EncodeError> {
    let aes = settings
        .password
        .map(|_| AesProperties::fresh().map_err(decode_error))
        .transpose()?;
    let pack = Pack {
        stage,
        count: 0,
        crc: Crc32::new(),
        buffer: [0; CHUNK],
        filled: 0,
    };
    let mut crc = Crc32::new();
    let (mut pack, intermediate) = if let Some(properties) = &aes {
        let key = cache
            .key(properties, settings.password, state.control, kdf)
            .map_err(decode_error)?;
        let writer =
            AesWriter::with_key(pack, properties, key, state.control).map_err(decode_error)?;
        let writer = if settings.method == "lzma2" {
            let mut writer = Lzma2Writer::new(writer, settings.level, MEMORY_LIMIT)
                .map_err(|e| io_error(e, EncodeError::Limit))?;
            state.files(indices, &mut writer, &mut crc)?;
            writer
                .finish()
                .map_err(|e| io_error(e, EncodeError::Output))?
        } else {
            let mut writer = writer;
            state.files(indices, &mut writer, &mut crc)?;
            writer
        };
        let intermediate = writer.input_bytes();
        (
            writer
                .finish()
                .map_err(|e| io_error(e, EncodeError::Output))?,
            intermediate,
        )
    } else if settings.method == "lzma2" {
        let mut writer = Lzma2Writer::new(pack, settings.level, MEMORY_LIMIT)
            .map_err(|e| io_error(e, EncodeError::Limit))?;
        state.files(indices, &mut writer, &mut crc)?;
        let pack = writer
            .finish()
            .map_err(|e| io_error(e, EncodeError::Output))?;
        let packed = pack.count;
        (pack, packed)
    } else {
        let mut pack = pack;
        state.files(indices, &mut pack, &mut crc)?;
        let size = pack.count;
        (pack, size)
    };
    pack.complete()
        .map_err(|e| io_error(e, EncodeError::Output))?;
    let unpacked = indices.iter().try_fold(0u64, |sum, &index| {
        sum.checked_add(state.input.entries()[index].size)
            .ok_or(EncodeError::Limit)
    })?;
    Ok(WrittenFolder {
        first,
        count: indices.len(),
        packed: pack.count,
        packed_crc: pack.crc.checksum(),
        unpacked,
        crc: crc.checksum(),
        intermediate,
        aes: aes.map(|p| p.encode()),
    })
}
pub(crate) fn encode_internal(
    input: &dyn EncodeInput,
    output: &mut dyn EncodeOutput,
    settings: EncodeSettings<'_>,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
    properties: Option<&[FileProperties]>,
    retained: u64,
) -> Result<(), EncodeError> {
    admit(input.entries(), settings, retained)?;
    check(cancel)?;
    if properties.is_some_and(|properties| properties.len() != input.entries().len()) {
        return Err(EncodeError::Invalid);
    }
    let mut stage = Stage {
        output,
        control: cancel,
        offset: 0,
        written: 0,
    };
    stage.append(&[0; 32])?;
    let mut nonempty = Vec::new();
    nonempty
        .try_reserve_exact(input.entries().len())
        .map_err(|_| EncodeError::Limit)?;
    let mut crcs = Vec::new();
    crcs.try_reserve_exact(input.entries().len())
        .map_err(|_| EncodeError::Limit)?;
    crcs.resize(input.entries().len(), 0);
    let mut state = InputState {
        input,
        control: cancel,
        progress,
        total: 0,
        crcs,
    };
    for (index, entry) in input.entries().iter().enumerate() {
        check(cancel)?;
        if !entry.directory {
            if entry.size == 0 {
                state.file(index, &mut io::sink(), &mut Crc32::new())?;
            } else {
                nonempty.push(index);
            }
        }
    }
    let mut kdf = KdfBudget::new(1 << 26, 1 << 30);
    let mut cache = AesKeyCache::default();
    let count = if settings.solid && !nonempty.is_empty() {
        1
    } else {
        nonempty.len()
    };
    let mut folders = Vec::new();
    folders
        .try_reserve_exact(count)
        .map_err(|_| EncodeError::Limit)?;
    if settings.solid && !nonempty.is_empty() {
        folders.push(folder(
            &mut stage, &mut state, &nonempty, 0, settings, &mut kdf, &mut cache,
        )?);
    } else {
        for (first, index) in nonempty.iter().enumerate() {
            folders.push(folder(
                &mut stage,
                &mut state,
                std::slice::from_ref(index),
                first,
                settings,
                &mut kdf,
                &mut cache,
            )?);
        }
    }
    let header = wire::header(
        input.entries(),
        properties,
        &folders,
        &nonempty,
        &state.crcs,
        settings.method == "lzma2",
        cancel,
    )?;
    let next = if settings.encrypt_headers {
        let properties = AesProperties::fresh().map_err(decode_error)?;
        let offset = stage.offset.checked_sub(32).ok_or(EncodeError::Invalid)?;
        let pack = Pack {
            stage: &mut stage,
            count: 0,
            crc: Crc32::new(),
            buffer: [0; CHUNK],
            filled: 0,
        };
        let key = cache
            .key(&properties, settings.password, cancel, &mut kdf)
            .map_err(decode_error)?;
        let mut writer =
            AesWriter::with_key(pack, &properties, key, cancel).map_err(decode_error)?;
        write_all(&mut writer, &header).map_err(|e| io_error(e, EncodeError::Output))?;
        let mut pack = writer
            .finish()
            .map_err(|e| io_error(e, EncodeError::Output))?;
        pack.complete()
            .map_err(|e| io_error(e, EncodeError::Output))?;
        wire::encoded_header(
            offset,
            pack.count,
            pack.crc.checksum(),
            header.len() as u64,
            wire::crc(&header, cancel)?,
            &properties.encode(),
        )?
    } else {
        header
    };
    check(cancel)?;
    let offset = stage.offset.checked_sub(32).ok_or(EncodeError::Invalid)?;
    stage.append(&next)?;
    let signature = wire::signature(offset, next.len() as u64, wire::crc(&next, cancel)?);
    stage.patch(0, &signature)?;
    check(cancel)?;
    loop {
        check(cancel)?;
        match stage.output.flush() {
            Err(error) if retry(&error) => continue,
            Err(error) => return Err(io_error(error, EncodeError::Output)),
            Ok(()) => break,
        }
    }
    check(cancel)
}
impl CreationCodec for SevenZipCodec {
    fn encode(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        encode_internal(input, output, settings, cancel, progress, None, 0)
    }
}
#[cfg(test)]
#[path = "encode_tests.rs"]
pub(crate) mod tests;
