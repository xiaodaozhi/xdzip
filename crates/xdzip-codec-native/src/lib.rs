// crates/xdzip-codec-native/src/lib.rs

//! Isolated format codecs behind a bounded worker and host-owned streams.
mod encode_input;
mod protocol;
#[cfg(test)]
mod tests;
use protocol::{Fault, Fields, Worker};
#[cfg(test)]
use std::io::Cursor;
use std::io::{self, Read, SeekFrom};
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
/// Isolated adapter for independent Rust codecs and the remaining UnRAR engine.
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
        Fault::SourceChanged => ScanError::SourceChanged,
        Fault::Unsupported => ScanError::Unsupported,
        Fault::Limit => ScanError::Limit,
        Fault::Cancelled => ScanError::Cancelled,
        Fault::PasswordRequired => ScanError::PasswordRequired,
        Fault::WrongPassword => ScanError::WrongPassword,
        Fault::PasswordOrIntegrity => ScanError::PasswordOrIntegrity,
        Fault::Invalid => ScanError::Invalid,
        Fault::MissingVolume => ScanError::MissingVolume,
    }
}
fn decode_error(error: Fault) -> DecodeError {
    match error {
        Fault::Io | Fault::SourceChanged => DecodeError::SourceIo,
        Fault::Unsupported => DecodeError::Unsupported,
        Fault::Limit => DecodeError::Limit,
        Fault::Cancelled => DecodeError::Cancelled,
        Fault::PasswordRequired => DecodeError::PasswordRequired,
        Fault::WrongPassword => DecodeError::WrongPassword,
        Fault::PasswordOrIntegrity => DecodeError::PasswordOrIntegrity,
        Fault::Output => DecodeError::Output,
        Fault::Invalid => DecodeError::Integrity,
        Fault::MissingVolume => DecodeError::SourceIo,
    }
}
fn encode_error(error: Fault) -> EncodeError {
    match error {
        Fault::Io | Fault::SourceChanged | Fault::MissingVolume => EncodeError::Source,
        Fault::Output => EncodeError::Output,
        Fault::Unsupported
        | Fault::PasswordRequired
        | Fault::WrongPassword
        | Fault::PasswordOrIntegrity => EncodeError::Unsupported,
        Fault::Limit => EncodeError::Limit,
        Fault::Cancelled => EncodeError::Cancelled,
        _ => EncodeError::Invalid,
    }
}
fn source_fault(error: io::Error) -> Fault {
    match error
        .get_ref()
        .and_then(|error| error.downcast_ref::<ScanError>())
    {
        Some(ScanError::Io) => Fault::Io,
        Some(ScanError::SourceChanged) => Fault::SourceChanged,
        Some(ScanError::Invalid) => Fault::Invalid,
        Some(ScanError::Unsupported) => Fault::Unsupported,
        Some(ScanError::Limit) => Fault::Limit,
        Some(ScanError::Cancelled) => Fault::Cancelled,
        Some(ScanError::MissingVolume) => Fault::MissingVolume,
        Some(ScanError::PasswordRequired) => Fault::PasswordRequired,
        Some(ScanError::WrongPassword) => Fault::WrongPassword,
        Some(ScanError::PasswordOrIntegrity) => Fault::PasswordOrIntegrity,
        None if error.kind() == io::ErrorKind::OutOfMemory => Fault::Limit,
        None => Fault::Io,
    }
}
fn source_call<T>(
    control: &Cancellation,
    mut operation: impl FnMut() -> io::Result<T>,
) -> Result<T, Fault> {
    loop {
        control.check().map_err(|_| Fault::Cancelled)?;
        match operation() {
            Err(error)
                if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none() => {}
            result => return result.map_err(source_fault),
        }
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
        volume_info: (Option<&[u64]>, Option<&[Vec<u8>]>),
    ) -> Result<Vec<u8>, Fault> {
        let format = match self.format {
            Format::SevenZip => 7,
            Format::Rar => 4,
            Format::Cab => 8,
            Format::Iso => 9,
            Format::Udf => 10,
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
        // A plain File/Cursor authorizes only its one stream. Synthetic naming
        // lets single CABs use the codec API; a requested neighbour still fails
        // because no other name or stream has been authorized by the host.
        let single_length = [length];
        let single_name = [b"archive.cab".to_vec()];
        let volumes = volume_info.0.unwrap_or(if self.format == Format::Cab {
            &single_length
        } else {
            &[]
        });
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
        if self.format == Format::Cab {
            let names = volume_info
                .1
                .or_else(|| volume_info.0.is_none().then_some(single_name.as_slice()))
                .ok_or(Fault::Invalid)?;
            if names.is_empty() || names.len() != volumes.len() {
                return Err(Fault::Invalid);
            }
            request.extend_from_slice(&(names.len() as u32).to_le_bytes());
            for name in names {
                if name.is_empty()
                    || name.len() > 4096
                    || request.len() + 4 + name.len() >= protocol::MAX_FRAME
                {
                    return Err(Fault::Limit);
                }
                request.extend_from_slice(&(name.len() as u32).to_le_bytes());
                request.extend_from_slice(name);
            }
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
        let length = source_call(control, || source.seek(SeekFrom::End(0)))?;
        let request = Zeroizing::new(self.request(
            0,
            length,
            password,
            0,
            limits,
            (source.volume_lengths(), source.volume_names()),
        )?);
        let mut worker = Worker::start(&request)?;
        let mut remaining = limits.read_bytes;
        let mut rows = Vec::new();
        let mut total_names = 0usize;
        let mut total_display = 0usize;
        let mut legacy = Vec::new();
        loop {
            let frame = worker.next(control)?;
            let mut fields = Fields(&frame);
            match fields.byte()? {
                protocol::READ => {
                    answer_read(&mut worker, source, &mut fields, &mut remaining, control)?
                }
                protocol::ROW => {
                    let index = fields.u64()?;
                    let flags = fields.byte()?;
                    let size = fields.u64()?;
                    let packed_size = fields.u64()?;
                    let modified_dos = fields.u32()?;
                    let method = u16::try_from(fields.u32()?).map_err(|_| Fault::Invalid)?;
                    let raw_name = fields.bytes(limits.name_bytes)?;
                    fields.end()?;
                    let cab = self.format == Format::Cab;
                    let raw = flags & 0x10 != 0;
                    if (cab && !raw)
                        || (raw && !matches!(self.format, Format::Cab | Format::Iso))
                        || (!cab && flags & 0x20 != 0)
                    {
                        return Err(Fault::Invalid);
                    }
                    if index != rows.len() as u64 || rows.len() >= limits.entries {
                        return Err(Fault::Limit);
                    }
                    total_names = total_names
                        .checked_add(raw_name.len())
                        .ok_or(Fault::Limit)?;
                    let utf8 = std::str::from_utf8(&raw_name);
                    let declared_utf8 = !raw || flags & 0x20 != 0;
                    let name_encoding = if declared_utf8 && utf8.is_err() {
                        xdzip_codec_api::names::NameEncoding::Unresolved
                    } else {
                        xdzip_codec_api::names::NameEncoding::Utf8
                    };
                    if !raw && utf8.is_err() {
                        return Err(Fault::Invalid);
                    }
                    let display_name = utf8
                        .map(xdzip_codec_api::names::display)
                        .unwrap_or_else(|_| xdzip_codec_api::names::escaped_bytes(&raw_name));
                    legacy.push(raw && !declared_utf8);
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
                        name_encoding,
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
                    if matches!(self.format, Format::Cab | Format::Iso | Format::Udf) {
                        let resolved = if self.format == Format::Cab {
                            xdzip_codec_api::names::resolve_cab(&mut rows, &legacy, control, limits)
                        } else {
                            xdzip_codec_api::names::resolve(
                                &mut rows, &legacy, false, control, limits,
                            )
                        };
                        resolved.map_err(|error| match error {
                            ScanError::Cancelled => Fault::Cancelled,
                            ScanError::Limit => Fault::Limit,
                            _ => Fault::Invalid,
                        })?;
                    }
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
    control: &Cancellation,
) -> Result<(), Fault> {
    let offset = fields.u64()?;
    let count = fields.u32()? as usize;
    fields.end()?;
    let bytes = source_bytes(source, offset, count, remaining, control)?;
    worker.send(protocol::BYTES, &bytes)
}
fn source_bytes(
    source: &mut dyn ReadSource,
    offset: u64,
    count: usize,
    remaining: &mut u64,
    control: &Cancellation,
) -> Result<Vec<u8>, Fault> {
    control.check().map_err(|_| Fault::Cancelled)?;
    if count > protocol::CHUNK || count as u64 > *remaining {
        return Err(Fault::Limit);
    }
    source_call(control, || source.seek(SeekFrom::Start(offset)))?;
    let mut bytes = vec![0; count];
    let count = source_call(control, || source.read(&mut bytes))?;
    if count > bytes.len() {
        return Err(Fault::Io);
    }
    bytes.truncate(count);
    *remaining -= count as u64;
    control.check().map_err(|_| Fault::Cancelled)?;
    Ok(bytes)
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
const PRECHARGED_LIMIT: u64 = 20 * 1024 * 1024 * 1024;

/// Credit belongs to one live worker, rather than to a source or record ID.
#[derive(Default)]
struct WorkerCredit(u64);

impl WorkerCredit {
    fn reset(&mut self) {
        self.0 = 0;
    }

    fn work(
        &mut self,
        format: Format,
        sink: &mut dyn DecodedSink,
        count: u64,
    ) -> Result<(), DecodeError> {
        let next = if matches!(format, Format::Cab | Format::SevenZip) {
            self.0
                .checked_add(count)
                .filter(|n| *n <= PRECHARGED_LIMIT)
                .ok_or(DecodeError::Limit)?
        } else {
            self.0
        };
        // Rejected host admission never creates prepaid authority.
        sink.charge_skipped(count)?;
        self.0 = next;
        Ok(())
    }

    fn data(
        &mut self,
        format: Format,
        sink: &mut dyn DecodedSink,
        bytes: &[u8],
        precharged: bool,
        written: &mut u64,
        declared: u64,
    ) -> Result<(), DecodeError> {
        if bytes.len() > protocol::CHUNK {
            return Err(DecodeError::Limit);
        }
        let count = bytes.len() as u64;
        let next = written.checked_add(count).ok_or(DecodeError::Limit)?;
        if next > declared {
            return Err(DecodeError::Integrity);
        }
        if precharged {
            if !matches!(format, Format::Cab | Format::SevenZip) || count == 0 {
                return Err(DecodeError::Integrity);
            }
            self.0 = self.0.checked_sub(count).ok_or(DecodeError::Integrity)?;
            // Debit before calling the host: a partial/failed output must not
            // leave the same bytes available for another exempt write.
            sink.write_precharged_chunk(bytes)?;
        } else {
            sink.write_chunk(bytes)?;
        }
        *written = next;
        Ok(())
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
    credit: WorkerCredit,
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
            credit: WorkerCredit::default(),
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
        let continued = self.last_index.is_some_and(|last| last < index)
            && self.worker_password.as_str() == password
            && self.worker_has_password == password_option.is_some();
        self.last_index = None;
        let mut worker = if continued {
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
            self.credit.reset();
            let length = source_call(control, || self.source.seek(SeekFrom::End(0)))
                .map_err(decode_error)?;
            let request = Zeroizing::new(
                self.codec
                    .request(
                        3,
                        length,
                        password_option,
                        index,
                        ScanLimits::default(),
                        (self.source.volume_lengths(), self.source.volume_names()),
                    )
                    .map_err(decode_error)?,
            );
            self.worker_password = Zeroizing::new(password.to_owned());
            self.worker_has_password = password_option.is_some();
            Worker::start(&request).map_err(decode_error)?
        };
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
                    control,
                )
                .map_err(decode_error)?,
                kind @ (protocol::DATA | protocol::PRECHARGED_DATA) => {
                    self.credit.data(
                        self.codec.format,
                        sink,
                        fields.0,
                        kind == protocol::PRECHARGED_DATA,
                        &mut written,
                        entry.size,
                    )?;
                    worker.send(protocol::ACK, &[]).map_err(decode_error)?;
                }
                protocol::WORK => {
                    let count = fields.u64().map_err(decode_error)?;
                    fields.end().map_err(decode_error)?;
                    self.credit.work(self.codec.format, sink, count)?;
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
                    self.credit.reset();
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
            || settings.method == "lzma2" && !matches!(settings.level, 1 | 6 | 9)
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
        let declared_total = entries.iter().try_fold(0u64, |total, entry| {
            if entry.size > 8 << 30 || entry.directory && entry.size != 0 {
                return Err(EncodeError::Limit);
            }
            total
                .checked_add(entry.size)
                .filter(|total| *total <= 20 << 30)
                .ok_or(EncodeError::Limit)
        })?;
        let progress_names: std::collections::BTreeSet<&str> =
            entries.iter().map(|entry| entry.name.as_str()).collect();
        let mut progressed = 0u64;
        let length = original
            .as_mut()
            .map(|source| {
                source_call(cancel, || source.seek(SeekFrom::End(0))).map_err(encode_error)
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
                (volumes.as_deref(), None),
            )
            .map_err(encode_error)?,
        );
        request.extend_from_slice(&[
            if settings.method == "copy" {
                0
            } else {
                settings.level
            },
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
        let mut streams = encode_input::Inputs::new(input);
        let mut total = 0u64;
        let mut read_remaining = 20 * 1024 * 1024 * 1024;
        loop {
            let frame = worker.next(cancel).map_err(encode_error)?;
            let mut fields = Fields(&frame);
            match fields.byte().map_err(encode_error)? {
                protocol::READ => {
                    let source = original.as_mut().ok_or(EncodeError::Invalid)?;
                    answer_read(
                        &mut worker,
                        *source,
                        &mut fields,
                        &mut read_remaining,
                        cancel,
                    )
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
                    let bytes = streams.read(index, count, cancel)?;
                    let count = bytes.len();
                    total = total
                        .checked_add(count as u64)
                        .filter(|total| *total <= 20 << 30)
                        .ok_or(EncodeError::Invalid)?;
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
                protocol::PROGRESS => {
                    let total = fields.u64().map_err(encode_error)?;
                    let bytes = fields.bytes(4096).map_err(encode_error)?;
                    fields.end().map_err(encode_error)?;
                    let name = std::str::from_utf8(&bytes).map_err(|_| EncodeError::Invalid)?;
                    if total < progressed
                        || total > declared_total
                        || !progress_names.contains(name)
                    {
                        return Err(EncodeError::Invalid);
                    }
                    progressed = total;
                    progress(EncodeProgress {
                        name,
                        input_bytes: total,
                    });
                    worker.send(protocol::ACK, &[]).map_err(encode_error)?;
                }
                protocol::ERROR => {
                    return Err(encode_error(Fault::from_wire(
                        fields.byte().map_err(encode_error)?,
                    )));
                }
                protocol::DONE => {
                    fields.end().map_err(encode_error)?;
                    worker.finish(cancel).map_err(encode_error)?;
                    streams.finish(old_indices, cancel)?;
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
#[cfg(test)]
struct BufferedInput {
    entries: Vec<EncodeEntry>,
    bytes: Vec<Vec<u8>>,
}
#[cfg(test)]
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

struct RewriteDelivery<'a> {
    entries: Vec<EncodeEntry>,
    additions: &'a dyn EncodeInput,
    added_indices: Vec<Option<usize>>,
}
impl EncodeInput for RewriteDelivery<'_> {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        let addition = self
            .added_indices
            .get(index)
            .copied()
            .flatten()
            .ok_or(EncodeError::Invalid)?;
        self.additions.open(addition)
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
        // The isolated Rust worker owns the bounded reconstruction buffer.
        // The host sends metadata and authorized addition streams only.
        let mut remaining = NATIVE_MEMORY_LIMIT / 2;
        let mut delivery = RewriteDelivery {
            entries: Vec::new(),
            additions: input.additions,
            added_indices: Vec::new(),
        };
        let mut old_indices = Vec::new();
        for (index, row) in rows
            .iter()
            .enumerate()
            .filter(|(index, _)| input.keep[*index])
        {
            remaining = remaining.checked_sub(row.size).ok_or(EncodeError::Limit)?;
            let renamed = input.renames.get(index).and_then(Option::as_deref);
            let name = renamed.map_or_else(
                || std::str::from_utf8(row.name_bytes()).map_err(|_| EncodeError::Invalid),
                Ok,
            )?;
            delivery.entries.push(EncodeEntry {
                name: name.trim_end_matches('/').to_owned(),
                directory: row.directory,
                size: row.size,
                modified_unix: 0,
            });
            delivery.added_indices.push(None);
            old_indices.push(Some(index));
        }
        for (index, entry) in input.additions.entries().iter().enumerate() {
            remaining = remaining
                .checked_sub(entry.size)
                .ok_or(EncodeError::Limit)?;
            delivery.entries.push(entry.clone());
            delivery.added_indices.push(Some(index));
            old_indices.push(None);
        }
        self.encode_internal(
            &delivery,
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
