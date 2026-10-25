// crates/xdzip-codec-native/src/lib.rs

//! Official native engines behind a bounded worker and host-owned streams.
mod protocol;
#[cfg(test)]
mod tests;
use protocol::{Fault, Fields, Worker};
use std::io::{Cursor, Read, SeekFrom};
use xdzip_codec_api::encode::{
    CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeOutput, EncodeProgress,
    EncodeSettings, RewriteCodec, RewriteInput, RewriteOutput,
};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
};
use xdzip_domain::identity::EntryId;
use zeroize::Zeroizing;

/// Native worker budget, enforced as committed memory on Windows and address
/// space on Unix. macOS excludes its pre-existing dyld virtual reservation.
pub const NATIVE_MEMORY_LIMIT: u64 = 1024 * 1024 * 1024;
/// Official 7-Zip (7z only) or UnRAR (RAR only) adapter.
pub struct NativeCodec {
    /// Selects an exact engine; other formats fail closed.
    pub format: Format,
}
struct RewriteProperties<'a> {
    source: &'a mut dyn ReadSource,
    old_indices: &'a [Option<usize>],
}
fn scan_error(error: Fault) -> ScanError {
    match error {
        Fault::Io | Fault::Output => ScanError::Io,
        Fault::Unsupported => ScanError::Unsupported,
        Fault::Limit => ScanError::Limit,
        Fault::Cancelled => ScanError::Cancelled,
        Fault::PasswordRequired => ScanError::PasswordRequired,
        Fault::WrongPassword => ScanError::WrongPassword,
        Fault::Invalid => ScanError::Invalid,
        Fault::MissingVolume => ScanError::MissingVolume,
    }
}
fn decode_error(error: Fault) -> DecodeError {
    match error {
        Fault::Io => DecodeError::SourceIo,
        Fault::Unsupported => DecodeError::Unsupported,
        Fault::Limit => DecodeError::Limit,
        Fault::Cancelled => DecodeError::Cancelled,
        Fault::PasswordRequired => DecodeError::PasswordRequired,
        Fault::WrongPassword => DecodeError::WrongPassword,
        Fault::Output => DecodeError::Output,
        Fault::Invalid => DecodeError::Integrity,
        Fault::MissingVolume => DecodeError::SourceIo,
    }
}
fn encode_error(error: Fault) -> EncodeError {
    match error {
        Fault::Io | Fault::MissingVolume => EncodeError::Source,
        Fault::Output => EncodeError::Output,
        Fault::Unsupported => EncodeError::Unsupported,
        Fault::Limit => EncodeError::Limit,
        Fault::Cancelled => EncodeError::Cancelled,
        _ => EncodeError::Invalid,
    }
}
impl NativeCodec {
    fn request(
        &self,
        operation: u8,
        length: u64,
        password: Option<&str>,
        selected: u64,
        limits: ScanLimits,
        volumes: Option<&[u64]>,
    ) -> Result<Vec<u8>, Fault> {
        let format = match self.format {
            Format::SevenZip => 7,
            Format::Rar => 4,
            _ => return Err(Fault::Unsupported),
        };
        let has_password = password.is_some();
        let password = password.unwrap_or("");
        if password.len() > 4096 {
            return Err(Fault::Limit);
        }
        let mut request = vec![operation, format];
        request.extend_from_slice(&NATIVE_MEMORY_LIMIT.to_le_bytes());
        request.extend_from_slice(
            &(limits.entries.min(xdzip_codec_api::MAX_ENTRIES) as u32).to_le_bytes(),
        );
        request.extend_from_slice(&(limits.name_bytes.min(4096) as u32).to_le_bytes());
        request.extend_from_slice(&length.to_le_bytes());
        request.push(u8::from(has_password));
        protocol::put_string(&mut request, password);
        request.extend_from_slice(&selected.to_le_bytes());
        let volumes = volumes.unwrap_or(&[]);
        if volumes.len() > 1024
            || (!volumes.is_empty()
                && volumes
                    .iter()
                    .try_fold(0u64, |sum, size| sum.checked_add(*size))
                    != Some(length))
        {
            return Err(Fault::Invalid);
        }
        request.extend_from_slice(&(volumes.len() as u32).to_le_bytes());
        for length in volumes {
            request.extend_from_slice(&length.to_le_bytes());
        }
        Ok(request)
    }
    fn scan_internal(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
        password: Option<&str>,
    ) -> Result<Vec<Entry>, Fault> {
        control.check().map_err(|_| Fault::Cancelled)?;
        let length = source.seek(SeekFrom::End(0)).map_err(|_| Fault::Io)?;
        let request = Zeroizing::new(self.request(
            0,
            length,
            password,
            0,
            limits,
            source.volume_lengths(),
        )?);
        let mut worker = Worker::start(&request)?;
        let mut remaining = limits.read_bytes;
        let mut rows = Vec::new();
        let mut total_names = 0usize;
        let mut total_display = 0usize;
        loop {
            let frame = worker.next(control)?;
            let mut fields = Fields(&frame);
            match fields.byte()? {
                protocol::READ => answer_read(&mut worker, source, &mut fields, &mut remaining)?,
                protocol::ROW => {
                    let index = fields.u64()?;
                    let flags = fields.byte()?;
                    let size = fields.u64()?;
                    let packed_size = fields.u64()?;
                    let modified_dos = fields.u32()?;
                    let method = u16::try_from(fields.u32()?).map_err(|_| Fault::Invalid)?;
                    let name = fields.string(limits.name_bytes)?;
                    fields.end()?;
                    if index != rows.len() as u64 || rows.len() >= limits.entries {
                        return Err(Fault::Limit);
                    }
                    total_names = total_names.checked_add(name.len()).ok_or(Fault::Limit)?;
                    let display_name = xdzip_codec_api::names::display(&name);
                    let raw_name = name.into_bytes();
                    total_display = total_display
                        .checked_add(display_name.len())
                        .ok_or(Fault::Limit)?;
                    if total_names > limits.total_name_bytes
                        || total_display > limits.display_name_bytes
                    {
                        return Err(Fault::Limit);
                    }
                    rows.push(Entry {
                        format: self.format,
                        id: EntryId::try_from(index + 1).map_err(|_| Fault::Limit)?,
                        raw_name,
                        decoded_name: None,
                        name_encoding: xdzip_codec_api::names::NameEncoding::Utf8,
                        display_name,
                        directory: flags & 1 != 0,
                        link: flags & 4 != 0,
                        unix_mode: 0,
                        size,
                        packed_size,
                        method,
                        modified_dos,
                        encrypted: flags & 2 != 0,
                        password_supported: flags & 2 != 0,
                        warning: flags & 8 != 0,
                    });
                    worker.send(protocol::ACK, &[])?;
                }
                protocol::ERROR => return Err(Fault::from_wire(fields.byte()?)),
                protocol::DONE => {
                    fields.end()?;
                    worker.finish(control)?;
                    return Ok(rows);
                }
                _ => return Err(Fault::Invalid),
            }
        }
    }
}
fn answer_read(
    worker: &mut Worker,
    source: &mut dyn ReadSource,
    fields: &mut Fields<'_>,
    remaining: &mut u64,
) -> Result<(), Fault> {
    let offset = fields.u64()?;
    let count = fields.u32()? as usize;
    fields.end()?;
    if count > protocol::CHUNK || count as u64 > *remaining {
        return Err(Fault::Limit);
    }
    source
        .seek(SeekFrom::Start(offset))
        .map_err(|_| Fault::Io)?;
    let mut bytes = vec![0; count];
    let count = source.read(&mut bytes).map_err(|_| Fault::Io)?;
    bytes.truncate(count);
    *remaining -= count as u64;
    worker.send(protocol::BYTES, &bytes)
}
impl MetadataCodec for NativeCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        self.scan_internal(source, control, limits, None)
            .map_err(scan_error)
    }
    fn scan_with_password(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
        password: Option<&str>,
    ) -> Result<Vec<Entry>, ScanError> {
        self.scan_internal(source, control, limits, password)
            .map_err(scan_error)
    }
}
struct NativeReader<'a> {
    codec: NativeCodec,
    source: &'a mut dyn ReadSource,
    snapshot: Vec<Entry>,
    password: Option<Zeroizing<String>>,
    read_remaining: u64,
    worker: Option<Worker>,
    last_index: Option<u64>,
    worker_password: Zeroizing<String>,
    worker_has_password: bool,
}
impl PayloadCodec for NativeCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        self.open_payload_with_password(source, snapshot, control, None)
    }
    fn open_payload_with_password<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
        password: Option<&str>,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let rows = self
            .scan_internal(source, control, ScanLimits::default(), password)
            .map_err(decode_error)?;
        if rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(NativeReader {
            codec: Self {
                format: self.format,
            },
            source,
            snapshot: rows,
            password: password.map(|secret| Zeroizing::new(secret.to_owned())),
            read_remaining: 20 * 1024 * 1024 * 1024,
            worker: None,
            last_index: None,
            worker_password: Zeroizing::new(String::new()),
            worker_has_password: false,
        }))
    }
}
impl PayloadReader for NativeReader<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.decode_with_password(entry, sink, control, None)
    }
    fn decode_with_password(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), DecodeError> {
        let index = entry
            .id
            .get()
            .checked_sub(1)
            .ok_or(DecodeError::Integrity)?;
        if self.snapshot.get(index as usize) != Some(entry) {
            return Err(DecodeError::Integrity);
        }
        if entry.link || entry.warning {
            return Err(DecodeError::Unsupported);
        }
        let password_option = password.or(self.password.as_ref().map(|secret| secret.as_str()));
        let password = password_option.unwrap_or("");
        let mut worker = if self.last_index.is_some_and(|last| last < index)
            && self.worker_password.as_str() == password
            && self.worker_has_password == password_option.is_some()
        {
            if let Some(mut worker) = self.worker.take() {
                worker
                    .send(protocol::NEXT, &index.to_le_bytes())
                    .map_err(decode_error)?;
                worker
            } else {
                return Err(DecodeError::Integrity);
            }
        } else {
            self.worker = None;
            let length = self
                .source
                .seek(SeekFrom::End(0))
                .map_err(|_| DecodeError::SourceIo)?;
            let request = Zeroizing::new(
                self.codec
                    .request(
                        3,
                        length,
                        password_option,
                        index,
                        ScanLimits::default(),
                        self.source.volume_lengths(),
                    )
                    .map_err(decode_error)?,
            );
            self.worker_password = Zeroizing::new(password.to_owned());
            self.worker_has_password = password_option.is_some();
            Worker::start(&request).map_err(decode_error)?
        };
        self.last_index = None;
        let mut written = 0u64;
        loop {
            let frame = worker.next(control).map_err(decode_error)?;
            let mut fields = Fields(&frame);
            match fields.byte().map_err(decode_error)? {
                protocol::READ => answer_read(
                    &mut worker,
                    self.source,
                    &mut fields,
                    &mut self.read_remaining,
                )
                .map_err(decode_error)?,
                protocol::DATA => {
                    let bytes = fields.0;
                    if bytes.len() > protocol::CHUNK {
                        return Err(DecodeError::Limit);
                    }
                    written = written
                        .checked_add(bytes.len() as u64)
                        .ok_or(DecodeError::Limit)?;
                    if written > entry.size {
                        return Err(DecodeError::Integrity);
                    }
                    sink.write_chunk(bytes)?;
                    worker.send(protocol::ACK, &[]).map_err(decode_error)?;
                }
                protocol::WORK => {
                    let count = fields.u64().map_err(decode_error)?;
                    fields.end().map_err(decode_error)?;
                    sink.charge_skipped(count)?;
                    worker.send(protocol::ACK, &[]).map_err(decode_error)?;
                }
                protocol::ERROR => {
                    return Err(decode_error(Fault::from_wire(
                        fields.byte().map_err(decode_error)?,
                    )));
                }
                protocol::ENTRY_DONE => {
                    if fields.u64().map_err(decode_error)? != index || written != entry.size {
                        return Err(DecodeError::Integrity);
                    }
                    fields.end().map_err(decode_error)?;
                    self.worker = Some(worker);
                    self.last_index = Some(index);
                    return Ok(());
                }
                protocol::DONE => {
                    fields.end().map_err(decode_error)?;
                    worker.finish(control).map_err(decode_error)?;
                    return if written == entry.size {
                        Ok(())
                    } else {
                        Err(DecodeError::Integrity)
                    };
                }
                _ => return Err(DecodeError::Integrity),
            }
        }
    }
}
impl NativeCodec {
    fn encode_internal(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
        properties: Option<RewriteProperties<'_>>,
    ) -> Result<(), EncodeError> {
        let (mut original, old_indices) = properties.map_or((None, &[][..]), |properties| {
            (Some(properties.source), properties.old_indices)
        });
        if self.format != Format::SevenZip
            || !matches!(settings.method, "copy" | "lzma2")
            || settings.level > 9
            || (settings.encrypt_headers && settings.password.is_none())
        {
            return Err(EncodeError::Unsupported);
        }
        let entries = input.entries();
        if entries.len() > xdzip_codec_api::MAX_ENTRIES {
            return Err(EncodeError::Invalid);
        }
        if !old_indices.is_empty() && old_indices.len() != entries.len() {
            return Err(EncodeError::Invalid);
        }
        let length = original
            .as_mut()
            .map(|source| {
                source
                    .seek(SeekFrom::End(0))
                    .map_err(|_| EncodeError::Source)
            })
            .transpose()?
            .unwrap_or(0);
        let volumes = original
            .as_ref()
            .and_then(|source| source.volume_lengths().map(<[u64]>::to_vec));
        let mut request = Zeroizing::new(
            self.request(
                2,
                length,
                settings.password,
                0,
                ScanLimits::default(),
                volumes.as_deref(),
            )
            .map_err(encode_error)?,
        );
        if original.is_some() {
            request[2..10].copy_from_slice(&(NATIVE_MEMORY_LIMIT / 2).to_le_bytes());
        }
        request.extend_from_slice(&[
            settings.level,
            u8::from(settings.solid),
            u8::from(settings.encrypt_headers),
            u8::from(settings.method == "copy"),
        ]);
        request.extend_from_slice(&(entries.len() as u32).to_le_bytes());
        let mut worker = Worker::start(&request).map_err(encode_error)?;
        // Entries are separate acknowledged frames, bounding metadata before the native parser allocates.
        for (index, entry) in entries.iter().enumerate() {
            if entry.name.len() > 4096 {
                return Err(EncodeError::Invalid);
            }
            let mut record = vec![u8::from(entry.directory)];
            record.extend_from_slice(&entry.size.to_le_bytes());
            record.extend_from_slice(&entry.modified_unix.to_le_bytes());
            record.extend_from_slice(
                &old_indices
                    .get(index)
                    .and_then(|index| *index)
                    .map_or(u64::MAX, |index| index as u64)
                    .to_le_bytes(),
            );
            protocol::put_string(&mut record, &entry.name);
            worker.send(protocol::ROW, &record).map_err(encode_error)?;
        }
        let mut streams: std::collections::BTreeMap<usize, Box<dyn Read + '_>> =
            std::collections::BTreeMap::new();
        let mut consumed = vec![0u64; entries.len()];
        let mut total = 0u64;
        let mut read_remaining = 20 * 1024 * 1024 * 1024;
        loop {
            let frame = worker.next(cancel).map_err(encode_error)?;
            let mut fields = Fields(&frame);
            match fields.byte().map_err(encode_error)? {
                protocol::READ => {
                    let source = original.as_mut().ok_or(EncodeError::Invalid)?;
                    answer_read(&mut worker, *source, &mut fields, &mut read_remaining)
                        .map_err(encode_error)?;
                }
                protocol::INPUT => {
                    let index = fields.u32().map_err(encode_error)? as usize;
                    let count = fields.u32().map_err(encode_error)? as usize;
                    fields.end().map_err(encode_error)?;
                    let entry = entries.get(index).ok_or(EncodeError::Invalid)?;
                    if count > protocol::CHUNK || entry.directory {
                        return Err(EncodeError::Invalid);
                    }
                    let remaining = entry
                        .size
                        .checked_sub(consumed[index])
                        .ok_or(EncodeError::Invalid)?;
                    if remaining == 0 {
                        worker.send(protocol::BYTES, &[]).map_err(encode_error)?;
                        continue;
                    }
                    if let std::collections::btree_map::Entry::Vacant(slot) = streams.entry(index) {
                        slot.insert(input.open(index)?);
                    }
                    let stream = streams.get_mut(&index).ok_or(EncodeError::Source)?;
                    let mut bytes =
                        vec![0; count.min(usize::try_from(remaining).unwrap_or(usize::MAX))];
                    let count = stream.read(&mut bytes).map_err(|_| EncodeError::Source)?;
                    bytes.truncate(count);
                    if count == 0 && remaining != 0 {
                        return Err(EncodeError::Source);
                    }
                    consumed[index] += count as u64;
                    if consumed[index] == entry.size {
                        streams.remove(&index);
                    }
                    total = total
                        .checked_add(count as u64)
                        .ok_or(EncodeError::Invalid)?;
                    progress(EncodeProgress {
                        name: &entry.name,
                        input_bytes: total,
                    });
                    worker.send(protocol::BYTES, &bytes).map_err(encode_error)?;
                }
                protocol::DATA => {
                    if fields.0.len() > protocol::CHUNK {
                        return Err(EncodeError::Invalid);
                    }
                    output
                        .write_all(fields.0)
                        .map_err(|_| EncodeError::Output)?;
                    worker.send(protocol::ACK, &[]).map_err(encode_error)?;
                }
                protocol::WRITE_AT => {
                    let offset = fields.u64().map_err(encode_error)?;
                    if fields.0.len() > protocol::CHUNK {
                        return Err(EncodeError::Invalid);
                    }
                    output.write_at(offset, fields.0)?;
                    worker.send(protocol::ACK, &[]).map_err(encode_error)?;
                }
                protocol::PROGRESS => (),
                protocol::ERROR => {
                    return Err(encode_error(Fault::from_wire(
                        fields.byte().map_err(encode_error)?,
                    )));
                }
                protocol::DONE => {
                    fields.end().map_err(encode_error)?;
                    worker.finish(cancel).map_err(encode_error)?;
                    if entries
                        .iter()
                        .zip(consumed)
                        .any(|(entry, read)| read != entry.size)
                    {
                        return Err(EncodeError::Source);
                    }
                    return Ok(());
                }
                _ => return Err(EncodeError::Invalid),
            }
        }
    }
}
impl CreationCodec for NativeCodec {
    fn encode(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        self.encode_internal(input, output, settings, cancel, progress, None)
    }
}
struct BufferedInput {
    entries: Vec<EncodeEntry>,
    bytes: Vec<Vec<u8>>,
}
impl EncodeInput for BufferedInput {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(Cursor::new(
            self.bytes
                .get(index)
                .ok_or(EncodeError::Invalid)?
                .as_slice(),
        )))
    }
}
struct BufferSink<'a> {
    bytes: &'a mut Vec<u8>,
    remaining: &'a mut u64,
    work_remaining: &'a mut u64,
}
impl DecodedSink for BufferSink<'_> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.charge_skipped(bytes.len() as u64)?;
        *self.remaining = self
            .remaining
            .checked_sub(bytes.len() as u64)
            .ok_or(DecodeError::Limit)?;
        self.bytes
            .try_reserve(bytes.len())
            .map_err(|_| DecodeError::Limit)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn charge_skipped(&mut self, count: u64) -> Result<(), DecodeError> {
        *self.work_remaining = self
            .work_remaining
            .checked_sub(count)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
}
impl RewriteCodec for NativeCodec {
    fn rewrite(
        &self,
        original: &mut dyn ReadSource,
        input: RewriteInput<'_>,
        output: &mut dyn RewriteOutput,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        if self.format != Format::SevenZip {
            return Err(EncodeError::Unsupported);
        }
        let rows = self
            .scan_internal(original, cancel, ScanLimits::default(), None)
            .map_err(encode_error)?;
        if rows.len() != input.keep.len()
            || (!input.renames.is_empty() && rows.len() != input.renames.len())
            || rows
                .iter()
                .any(|row| row.encrypted || row.link || row.warning)
            || original
                .volume_lengths()
                .is_some_and(|volumes| volumes.len() > 1)
        {
            return Err(EncodeError::Unsupported);
        }
        // Reserve half the process budget for the native encoder. No temporary pathname or
        // filesystem publication capability is introduced into the codec.
        let mut remaining = NATIVE_MEMORY_LIMIT / 2;
        let mut work_remaining = 20 * 1024 * 1024 * 1024;
        let mut buffered = BufferedInput {
            entries: Vec::new(),
            bytes: Vec::new(),
        };
        let mut old_indices = Vec::new();
        let mut reader = self
            .open_payload(original, &rows, cancel)
            .map_err(|_| EncodeError::Invalid)?;
        for (index, row) in rows
            .iter()
            .enumerate()
            .filter(|(index, _)| input.keep[*index])
        {
            if row.size > remaining {
                return Err(EncodeError::Limit);
            }
            let mut bytes = Vec::new();
            if !row.directory {
                bytes
                    .try_reserve_exact(usize::try_from(row.size).map_err(|_| EncodeError::Limit)?)
                    .map_err(|_| EncodeError::Limit)?;
                reader
                    .decode(
                        row,
                        &mut BufferSink {
                            bytes: &mut bytes,
                            remaining: &mut remaining,
                            work_remaining: &mut work_remaining,
                        },
                        cancel,
                    )
                    .map_err(|error| match error {
                        DecodeError::Cancelled => EncodeError::Cancelled,
                        DecodeError::Limit => EncodeError::Limit,
                        DecodeError::Unsupported => EncodeError::Unsupported,
                        _ => EncodeError::Invalid,
                    })?;
            }
            let name = if input.renames.is_empty() {
                None
            } else {
                input.renames[index].as_ref()
            };
            buffered.entries.push(EncodeEntry {
                name: name
                    .map_or_else(
                        || String::from_utf8_lossy(row.name_bytes()).into_owned(),
                        Clone::clone,
                    )
                    .trim_end_matches('/')
                    .to_owned(),
                directory: row.directory,
                size: row.size,
                modified_unix: 0,
            });
            buffered.bytes.push(bytes);
            old_indices.push(Some(index));
        }
        drop(reader);
        for (index, entry) in input.additions.entries().iter().enumerate() {
            if entry.size > remaining {
                return Err(EncodeError::Limit);
            }
            let mut bytes = Vec::new();
            if !entry.directory {
                bytes
                    .try_reserve_exact(usize::try_from(entry.size).map_err(|_| EncodeError::Limit)?)
                    .map_err(|_| EncodeError::Limit)?;
                let mut stream = input.additions.open(index)?;
                let mut block = [0; protocol::CHUNK];
                loop {
                    cancel.check().map_err(|_| EncodeError::Cancelled)?;
                    let count = stream.read(&mut block).map_err(|_| EncodeError::Source)?;
                    if count == 0 {
                        break;
                    }
                    if bytes.len() as u64 + count as u64 > entry.size {
                        return Err(EncodeError::Source);
                    }
                    BufferSink {
                        bytes: &mut bytes,
                        remaining: &mut remaining,
                        work_remaining: &mut work_remaining,
                    }
                    .write_chunk(&block[..count])
                    .map_err(|_| EncodeError::Limit)?;
                }
                if bytes.len() as u64 != entry.size {
                    return Err(EncodeError::Source);
                }
            }
            buffered.entries.push(entry.clone());
            buffered.bytes.push(bytes);
            old_indices.push(None);
        }
        self.encode_internal(
            &buffered,
            &mut { output },
            input.settings,
            cancel,
            progress,
            Some(RewriteProperties {
                source: original,
                old_indices: &old_indices,
            }),
        )
    }
}
