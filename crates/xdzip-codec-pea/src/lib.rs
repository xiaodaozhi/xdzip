//! Bounded read-only PEA 1.x parsing.
//!
//! The admitted profile is deliberately small: one volume, no encryption,
//! no object/stream/volume checks and PCOMPRESS0 stored objects. The public
//! PEA specification is the wire-format authority; the LGPL reference
//! implementation is never linked or copied into the product.

use std::io::{ErrorKind, SeekFrom};

use xdzip_codec_api::encode::{
    CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    RewriteCodec, RewriteInput, RewriteOutput,
};
use xdzip_codec_api::names::{self, NameEncoding};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
};
use xdzip_domain::identity::EntryId;

const MAGIC: u8 = 0xea;
const VERSION: u8 = 1;
const MAX_REVISION: u8 = 6;
const HEADER_SIZE: usize = 10;
const STREAM_HEADER_SIZE: usize = 10;
const POD: [u8; 4] = *b"POD\0";
const EOA: [u8; 4] = *b"EOA\0";
const NOALGO: u8 = 0;
const PCOMPRESS0: u8 = 0;
const MAX_INPUT: u64 = 512 * 1024 * 1024;
const MAX_PAYLOAD: u64 = 512 * 1024 * 1024;

/// Read-only PEA codec for the admitted unencrypted stored profile.
pub struct PeaCodec;

#[derive(Clone)]
struct Record {
    entry: Entry,
    offset: Option<usize>,
}

struct Parsed {
    rows: Vec<Entry>,
    records: Vec<Record>,
}

impl PeaCodec {
    /// Confirm the PEA header and the admitted stream profile.
    pub fn candidate(
        source: &mut dyn ReadSource,
        control: &Cancellation,
    ) -> Result<bool, ScanError> {
        control.check()?;
        let Some(lengths) = source.volume_lengths() else {
            return Self::probe(source, control);
        };
        if lengths.len() != 1 {
            return Ok(false);
        }
        Self::probe(source, control)
    }

    fn probe(source: &mut dyn ReadSource, control: &Cancellation) -> Result<bool, ScanError> {
        source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
        let mut header = [0u8; HEADER_SIZE + STREAM_HEADER_SIZE];
        let result = match source.read_exact(&mut header) {
            Ok(()) => Ok(header[0] == MAGIC
                && header[1] == VERSION
                && header[2] <= MAX_REVISION
                && header[3] == NOALGO
                && header[4] == NOALGO
                && header[7] == 1
                && header[12..16] == POD[..]),
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => Ok(false),
            Err(error) => Err(ScanError::from_source_error(&error)),
        };
        source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
        control.check()?;
        result
    }
}

impl MetadataCodec for PeaCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        let bytes = read_source(source, control, MAX_INPUT)?;
        parse(&bytes, control, limits).map(|parsed| parsed.rows)
    }
}

impl PayloadCodec for PeaCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let bytes = read_source(source, control, MAX_INPUT).map_err(decode_error)?;
        let parsed = parse(&bytes, control, ScanLimits::default()).map_err(decode_error)?;
        if parsed.rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(Payload {
            bytes,
            records: parsed.records,
            failure: None,
        }))
    }
}

impl CreationCodec for PeaCodec {
    fn encode(
        &self,
        _input: &dyn EncodeInput,
        _output: &mut dyn EncodeOutput,
        _settings: EncodeSettings<'_>,
        _cancel: &Cancellation,
        _progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}

impl RewriteCodec for PeaCodec {
    fn rewrite(
        &self,
        _original: &mut dyn ReadSource,
        _input: RewriteInput<'_>,
        _output: &mut dyn RewriteOutput,
        _cancel: &Cancellation,
        _progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}

struct Payload {
    bytes: Vec<u8>,
    records: Vec<Record>,
    failure: Option<DecodeError>,
}

impl PayloadReader for Payload {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = (|| {
            control.check().map_err(decode_error)?;
            let index = entry
                .id
                .get()
                .checked_sub(1)
                .and_then(|value| usize::try_from(value).ok())
                .ok_or(DecodeError::Integrity)?;
            let record = self.records.get(index).ok_or(DecodeError::Integrity)?;
            if &record.entry != entry {
                return Err(DecodeError::Integrity);
            }
            if entry.directory || !entry.payload_supported() {
                return Err(DecodeError::Unsupported);
            }
            let offset = record.offset.ok_or(DecodeError::Integrity)?;
            let end = offset
                .checked_add(usize::try_from(entry.size).map_err(|_| DecodeError::Limit)?)
                .ok_or(DecodeError::Limit)?;
            let data = self.bytes.get(offset..end).ok_or(DecodeError::Integrity)?;
            control.check().map_err(decode_error)?;
            sink.write_chunk(data)?;
            Ok(())
        })();
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
}

fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}

fn read_source(
    source: &mut dyn ReadSource,
    control: &Cancellation,
    limit: u64,
) -> Result<Vec<u8>, ScanError> {
    if let Some(lengths) = source.volume_lengths()
        && lengths.len() != 1
    {
        return Err(ScanError::Unsupported);
    }
    let length = source.seek(SeekFrom::End(0)).map_err(|_| ScanError::Io)?;
    if length < (HEADER_SIZE + STREAM_HEADER_SIZE + 6) as u64 {
        return Err(ScanError::Invalid);
    }
    if length > limit {
        return Err(ScanError::Limit);
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let capacity = usize::try_from(length).map_err(|_| ScanError::Limit)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| ScanError::Limit)?;
    bytes.resize(capacity, 0);
    let mut offset = 0usize;
    while offset != capacity {
        control.check()?;
        let read = match source.read(&mut bytes[offset..]) {
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::Interrupted && error.get_ref().is_none() => {
                continue;
            }
            Err(error) => return Err(ScanError::from_source_error(&error)),
        };
        if read == 0 {
            return Err(ScanError::Invalid);
        }
        offset = offset.checked_add(read).ok_or(ScanError::Limit)?;
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    Ok(bytes)
}

fn parse(bytes: &[u8], control: &Cancellation, limits: ScanLimits) -> Result<Parsed, ScanError> {
    if bytes.len() < HEADER_SIZE + STREAM_HEADER_SIZE
        || bytes[0] != MAGIC
        || bytes[1] != VERSION
        || bytes[2] > MAX_REVISION
        || bytes[3] != NOALGO
        || bytes[4] != NOALGO
        || bytes[7] != 1
        || bytes[10..16] != [0, 0, POD[0], POD[1], POD[2], POD[3]]
        || bytes[16] != PCOMPRESS0
        || bytes[17] != NOALGO
        || bytes[18] != NOALGO
        || bytes[19] != NOALGO
    {
        return Err(ScanError::Unsupported);
    }
    let mut cursor = HEADER_SIZE + STREAM_HEADER_SIZE;
    let mut records = Vec::new();
    records
        .try_reserve(limits.entries)
        .map_err(|_| ScanError::Limit)?;
    let mut total_names = 0usize;
    let mut total_display_names = 0usize;
    let mut total_payload = 0u64;
    loop {
        control.check()?;
        let name_len = usize::from(read_u16(bytes, cursor)?);
        cursor = cursor.checked_add(2).ok_or(ScanError::Limit)?;
        if name_len == 0 {
            if bytes.get(cursor..cursor + 4) != Some(&EOA) {
                return Err(ScanError::Unsupported);
            }
            cursor = cursor.checked_add(4).ok_or(ScanError::Limit)?;
            if cursor != bytes.len() {
                return Err(ScanError::Invalid);
            }
            break;
        }
        if name_len > limits.name_bytes {
            return Err(ScanError::Limit);
        }
        let name_end = cursor.checked_add(name_len).ok_or(ScanError::Limit)?;
        let raw_name = bytes
            .get(cursor..name_end)
            .ok_or(ScanError::Invalid)?
            .to_vec();
        cursor = name_end;
        total_names = total_names
            .checked_add(raw_name.len())
            .ok_or(ScanError::Limit)?;
        if total_names > limits.total_name_bytes {
            return Err(ScanError::Limit);
        }
        let name = std::str::from_utf8(&raw_name)
            .map_err(|_| ScanError::Unsupported)?
            .to_owned();
        if name.contains('\0') || name.is_empty() {
            return Err(ScanError::Invalid);
        }
        let directory = name.ends_with('/') || name.ends_with('\\');
        let modified_offset = cursor;
        let metadata_end = cursor.checked_add(8).ok_or(ScanError::Limit)?;
        let _modified = read_u32(bytes, modified_offset)?;
        let _attributes = read_u32(bytes, modified_offset + 4)?;
        cursor = metadata_end;
        let offset = if directory {
            None
        } else {
            let size = read_u64(bytes, cursor)?;
            cursor = cursor.checked_add(8).ok_or(ScanError::Limit)?;
            total_payload = total_payload.checked_add(size).ok_or(ScanError::Limit)?;
            if total_payload > MAX_PAYLOAD {
                return Err(ScanError::Limit);
            }
            let data_offset = cursor;
            cursor = cursor
                .checked_add(usize::try_from(size).map_err(|_| ScanError::Limit)?)
                .ok_or(ScanError::Limit)?;
            bytes.get(data_offset..cursor).ok_or(ScanError::Invalid)?;
            Some((data_offset, size))
        };
        if records.len() >= limits.entries {
            return Err(ScanError::Limit);
        }
        let id = EntryId::try_from(records.len() as u64 + 1).map_err(|_| ScanError::Limit)?;
        let warning = names::suspicious(&name, directory);
        let display_name = names::display(&name);
        total_display_names = total_display_names
            .checked_add(display_name.len())
            .ok_or(ScanError::Limit)?;
        if total_display_names > limits.display_name_bytes {
            return Err(ScanError::Limit);
        }
        let (size, offset) = offset.map_or((0, None), |(offset, size)| (size, Some(offset)));
        records.push(Record {
            entry: Entry {
                details: Default::default(),
                format: Format::Pea,
                id,
                raw_name,
                decoded_name: None,
                name_encoding: NameEncoding::Utf8,
                display_name,
                directory,
                link: false,
                unix_mode: 0,
                size,
                size_known: true,
                packed_size: size,
                method: 0,
                modified_dos: 0,
                encrypted: false,
                password_supported: false,
                warning,
            },
            offset,
        });
    }
    if records.is_empty() {
        return Err(ScanError::Invalid);
    }
    let rows = records.iter().map(|record| record.entry.clone()).collect();
    Ok(Parsed { rows, records })
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, ScanError> {
    let data = bytes.get(offset..offset + 2).ok_or(ScanError::Invalid)?;
    Ok(u16::from_le_bytes([data[0], data[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ScanError> {
    let data = bytes.get(offset..offset + 4).ok_or(ScanError::Invalid)?;
    Ok(u32::from_le_bytes([data[0], data[1], data[2], data[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, ScanError> {
    let data = bytes.get(offset..offset + 8).ok_or(ScanError::Invalid)?;
    Ok(u64::from_le_bytes([
        data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> Vec<u8> {
        let mut bytes = vec![MAGIC, VERSION, MAX_REVISION, 0, 0, 0, 0, 1, 0x21, 0];
        bytes.extend_from_slice(&[0, 0, POD[0], POD[1], POD[2], POD[3], 0, 0, 0, 0]);
        push_file(&mut bytes, b"hello.txt", b"hello");
        push_dir(&mut bytes, b"folder/");
        push_file(&mut bytes, "utf8-文件".as_bytes(), b"payload");
        bytes.extend_from_slice(&[0, 0, EOA[0], EOA[1], EOA[2], EOA[3]]);
        bytes
    }

    fn push_file(bytes: &mut Vec<u8>, name: &[u8], data: &[u8]) {
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&(data.len() as u64).to_le_bytes());
        bytes.extend_from_slice(data);
    }

    fn push_dir(bytes: &mut Vec<u8>, name: &[u8]) {
        bytes.extend_from_slice(&(name.len() as u16).to_le_bytes());
        bytes.extend_from_slice(name);
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }

    #[test]
    fn scans_store_only_profile_and_preserves_duplicate_physical_rows() {
        let rows = PeaCodec
            .scan(
                &mut Cursor::new(fixture()),
                &Cancellation::default(),
                ScanLimits::default(),
            )
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].raw_name, b"hello.txt");
        assert_eq!(rows[0].size, 5);
        assert!(!rows[0].warning);
        assert!(rows[1].directory);
        assert_eq!(rows[2].display_name, "utf8-文件");
    }

    #[test]
    fn rejects_compression_checks_encryption_and_trailing_bytes() {
        let mut compressed = fixture();
        compressed[16] = 1;
        assert_eq!(
            PeaCodec.scan(
                &mut Cursor::new(compressed),
                &Cancellation::default(),
                ScanLimits::default()
            ),
            Err(ScanError::Unsupported)
        );
        let mut trailing = fixture();
        trailing.push(0);
        assert_eq!(
            PeaCodec.scan(
                &mut Cursor::new(trailing),
                &Cancellation::default(),
                ScanLimits::default()
            ),
            Err(ScanError::Invalid)
        );
    }

    #[test]
    fn payload_snapshot_and_copy_are_bounded() {
        let bytes = fixture();
        let rows = PeaCodec
            .scan(
                &mut Cursor::new(bytes.clone()),
                &Cancellation::default(),
                ScanLimits::default(),
            )
            .unwrap();
        let mut source = Cursor::new(bytes);
        let cancellation = Cancellation::default();
        let mut reader = PeaCodec
            .open_payload(&mut source, &rows, &cancellation)
            .unwrap();
        let mut output = Vec::new();
        struct Sink<'a>(&'a mut Vec<u8>);
        impl DecodedSink for Sink<'_> {
            fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
                self.0.extend_from_slice(bytes);
                Ok(())
            }
        }
        reader
            .decode(&rows[0], &mut Sink(&mut output), &Cancellation::default())
            .unwrap();
        assert_eq!(output, b"hello");
        assert_eq!(
            reader.decode(&rows[1], &mut Sink(&mut output), &Cancellation::default()),
            Err(DecodeError::Unsupported)
        );
    }
}
