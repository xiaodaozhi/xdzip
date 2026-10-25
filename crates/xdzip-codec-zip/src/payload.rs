// crates/xdzip-codec-zip/src/payload.rs

//! Strict ZIP payload ranges from APPNOTE 6.3.10 sections 4.3.7-4.3.9/4.5.3.
//! The host retains every output and commit capability.
use crate::{
    ZipCodec, aes,
    record::{self, ExtraFields},
};
use std::cell::Cell;
use std::io::{self, Read, Seek, SeekFrom};
use std::rc::Rc;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, ReadSource, ScanError, ScanLimits};
use xdzip_compression::{Crc32, DeflateReader};

const BLOCK: usize = 128 * 1024;
const INPUT_LIMIT: u64 = 20 * 1024 * 1024 * 1024;
struct Input<'a> {
    source: &'a mut dyn ReadSource,
    control: &'a Cancellation,
    remaining: Rc<Cell<u64>>,
}
impl Read for Input<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.control.check().map_err(io::Error::other)?;
        let remaining = self.remaining.get();
        if remaining == 0 && !bytes.is_empty() {
            return Err(io::Error::other(ScanError::Limit));
        }
        let bound = bytes.len().min(remaining.min(usize::MAX as u64) as usize);
        let count = loop {
            self.control.check().map_err(io::Error::other)?;
            match self.source.read(&mut bytes[..bound]) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => break result?,
            }
        };
        self.remaining.set(remaining - count as u64);
        Ok(count)
    }
}
impl Seek for Input<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.control.check().map_err(io::Error::other)?;
        self.source.seek(position)
    }
}
impl ReadSource for Input<'_> {}
struct PayloadRecord {
    central: record::Record,
    effective_name: Vec<u8>,
    boundary: u64,
}
struct ZipPayload<'a> {
    input: Input<'a>,
    records: Vec<PayloadRecord>,
    output: Vec<u8>,
}
fn matches_snapshot(expected: &Entry, header: &record::Record) -> bool {
    header.name == expected.raw_name
        && crate::names::matches(expected, header.utf8(), &header.extra)
        && header.is_dir() == expected.directory
        && header.size == expected.size
        && header.packed_size == expected.packed_size
        && header.method == expected.method
        && header.encrypted() == expected.encrypted
        && (header.encrypted()
            && header.method == 99
            && aes::supported(ExtraFields::new(&header.extra), header.crc))
            == expected.password_supported
}
impl PayloadCodec for ZipCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        check(control)?;
        if snapshot.len() > xdzip_codec_api::MAX_ENTRIES {
            return Err(DecodeError::Limit);
        }
        let remaining = Rc::new(Cell::new(64 * 1024 * 1024));
        let mut input = Input {
            source,
            control,
            remaining: Rc::clone(&remaining),
        };
        let directory = record::read_directory(&mut input, control, ScanLimits::default())
            .map_err(scan_error)?;
        if directory.records.len() != snapshot.len() {
            return Err(DecodeError::Integrity);
        }
        let mut offsets: Vec<_> = directory
            .records
            .iter()
            .map(|record| record.local_offset)
            .collect();
        offsets.sort_unstable();
        check(control)?;
        if offsets
            .iter()
            .any(|offset| *offset < directory.base_offset || *offset >= directory.central_offset)
            || offsets.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(DecodeError::Integrity);
        }
        let mut records = Vec::with_capacity(snapshot.len());
        let mut names = 0usize;
        for (index, central) in directory.records.into_iter().enumerate() {
            check(control)?;
            let expected = &snapshot[index];
            if expected.id.get() != index as u64 + 1 || !matches_snapshot(expected, &central) {
                return Err(DecodeError::Integrity);
            }
            names = names
                .checked_add(expected.name_bytes().len())
                .ok_or(DecodeError::Limit)?;
            if expected.name_bytes().len() > 4096 || names > 16 * 1024 * 1024 {
                return Err(DecodeError::Limit);
            }
            let physical = offsets
                .binary_search(&central.local_offset)
                .map_err(|_| DecodeError::Integrity)?;
            let boundary = offsets
                .get(physical + 1)
                .copied()
                .unwrap_or(directory.central_offset);
            records.push(PayloadRecord {
                boundary,
                central,
                effective_name: expected.name_bytes().to_vec(),
            });
        }
        remaining.set(INPUT_LIMIT);
        Ok(Box::new(ZipPayload {
            input,
            records,
            output: vec![0; BLOCK],
        }))
    }
}
impl PayloadReader for ZipPayload<'_> {
    fn decode(
        &mut self,
        expected: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.decode_with_password(expected, sink, control, None)
    }
    fn decode_with_password(
        &mut self,
        expected: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), DecodeError> {
        check(control)?;
        if expected.warning || !matches!(expected.method, 0 | 8 | 99) {
            return Err(DecodeError::Unsupported);
        }
        let index = expected
            .id
            .get()
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .ok_or(DecodeError::Integrity)?;
        let record = self.records.get(index).ok_or(DecodeError::Integrity)?;
        let central = &record.central;
        if expected.name_bytes() != record.effective_name || !matches_snapshot(expected, central) {
            return Err(DecodeError::Integrity);
        }
        let profile = if expected.encrypted && expected.method == 99 {
            Some(aes::profile(ExtraFields::new(&central.extra))?.ok_or(DecodeError::Integrity)?)
        } else if expected.encrypted || expected.method == 99 {
            return Err(DecodeError::Unsupported);
        } else {
            None
        };
        let admitted_flags = if profile.is_some() { 0x080f } else { 0x080e };
        if central.flags & !admitted_flags != 0 || (expected.directory && expected.size != 0) {
            return Err(DecodeError::Unsupported);
        }
        let local = local_record(
            &mut self.input,
            central,
            expected.name_bytes(),
            record.boundary,
            control,
        )?;
        self.input
            .seek(SeekFrom::Start(local.data_start))
            .map_err(read_error)?;
        let mut range = (&mut self.input).take(local.data_end - local.data_start);
        if let Some(profile) = profile {
            if profile.version == 2 && central.crc != 0 {
                return Err(DecodeError::Integrity);
            }
            let packed = central
                .packed_size
                .checked_sub(28)
                .ok_or(DecodeError::Integrity)?;
            if profile.method == 0 && expected.size != packed {
                return Err(DecodeError::Integrity);
            }
            let password = password.ok_or(DecodeError::PasswordRequired)?;
            let mut plain = aes::AesReader::new(&mut range, central.packed_size, password)?;
            {
                let mut payload: Box<dyn Read + '_> = if profile.method == 0 {
                    Box::new(&mut plain)
                } else {
                    Box::new(DeflateReader::new(&mut plain, packed))
                };
                read_to_sink(
                    &mut payload,
                    &mut self.output,
                    expected.size,
                    (profile.version == 1).then_some(central.crc),
                    sink,
                    control,
                )?;
            }
            plain.finish_auth().map_err(read_error)?;
            if range.limit() != 0 {
                return Err(DecodeError::Integrity);
            }
            return check(control);
        }
        if central.method == 0 && central.size != central.packed_size {
            return Err(DecodeError::Integrity);
        }
        {
            let mut payload: Box<dyn Read + '_> = if central.method == 0 {
                Box::new(&mut range)
            } else {
                Box::new(DeflateReader::new(&mut range, central.packed_size))
            };
            read_to_sink(
                &mut payload,
                &mut self.output,
                expected.size,
                Some(central.crc),
                sink,
                control,
            )?;
        }
        if range.limit() != 0 {
            return Err(DecodeError::Integrity);
        }
        check(control)
    }
}

/// Validated physical local-header/payload/descriptor span, reusable by rewrite.
pub(crate) struct LocalRecord {
    pub data_start: u64,
    pub data_end: u64,
    pub end: u64,
    pub extra: Vec<u8>,
}
fn n16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn n32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("bounded ZIP field"))
}
fn n64(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().expect("bounded ZIP field"))
}
fn read_at(
    source: &mut dyn ReadSource,
    at: u64,
    bytes: &mut [u8],
    boundary: u64,
) -> Result<(), DecodeError> {
    if at
        .checked_add(bytes.len() as u64)
        .is_none_or(|end| end > boundary)
    {
        return Err(DecodeError::Integrity);
    }
    source.seek(SeekFrom::Start(at)).map_err(read_error)?;
    source.read_exact(bytes).map_err(read_error)
}
pub(crate) fn local_record(
    source: &mut dyn ReadSource,
    central: &record::Record,
    effective_name: &[u8],
    boundary: u64,
    control: &Cancellation,
) -> Result<LocalRecord, DecodeError> {
    check(control)?;
    let mut fixed = [0; 30];
    read_at(source, central.local_offset, &mut fixed, boundary)?;
    if &fixed[..4] != b"PK\x03\x04"
        || n16(&fixed, 6) != central.flags
        || n16(&fixed, 8) != central.method
        || usize::from(n16(&fixed, 26)) != central.name.len()
    {
        return Err(DecodeError::Integrity);
    }
    let extra_len = usize::from(n16(&fixed, 28));
    let header_end = central
        .local_offset
        .checked_add(30)
        .and_then(|at| at.checked_add(central.name.len() as u64))
        .and_then(|at| at.checked_add(extra_len as u64))
        .ok_or(DecodeError::Integrity)?;
    if header_end > boundary {
        return Err(DecodeError::Integrity);
    }
    let mut variable = vec![0; central.name.len() + extra_len];
    read_at(source, central.local_offset + 30, &mut variable, boundary)?;
    if variable[..central.name.len()] != central.name {
        return Err(DecodeError::Integrity);
    }
    let extra = variable.split_off(central.name.len());
    let mut packed = u64::from(n32(&fixed, 18));
    let mut unpacked = u64::from(n32(&fixed, 22));
    let local_wide = packed == u64::from(u32::MAX) || unpacked == u64::from(u32::MAX);
    let mut zip64 = None;
    for field in ExtraFields::new(&extra) {
        let (id, bytes) = field.map_err(scan_error)?;
        if id == 1 && zip64.replace(bytes).is_some() {
            return Err(DecodeError::Integrity);
        }
    }
    if local_wide {
        let values = zip64.ok_or(DecodeError::Integrity)?;
        let mut at = 0;
        if unpacked == u64::from(u32::MAX) {
            if values.len() < at + 8 {
                return Err(DecodeError::Integrity);
            }
            unpacked = n64(values, at);
            at += 8;
        }
        if packed == u64::from(u32::MAX) {
            if values.len() < at + 8 {
                return Err(DecodeError::Integrity);
            }
            packed = n64(values, at);
        }
    }
    let local_profile =
        aes::profile(ExtraFields::new(&extra)).map_err(|_| DecodeError::Integrity)?;
    let central_profile =
        aes::profile(ExtraFields::new(&central.extra)).map_err(|_| DecodeError::Integrity)?;
    if central.encrypted() && central.method == 99 && local_profile != central_profile {
        return Err(DecodeError::Integrity);
    }
    if let Some(name) = crate::names::unicode_path(&central.name, central.utf8(), &extra)
        .map_err(|_| DecodeError::Integrity)?
        && name.as_bytes() != effective_name
    {
        return Err(DecodeError::Integrity);
    }
    let descriptor = central.flags & 8 != 0;
    let crc = n32(&fixed, 14);
    if (!descriptor
        && (crc != central.crc || packed != central.packed_size || unpacked != central.size))
        || (descriptor
            && ((crc != 0 && crc != central.crc)
                || (packed != 0 && packed != central.packed_size)
                || (unpacked != 0 && unpacked != central.size)))
    {
        return Err(DecodeError::Integrity);
    }
    let data_end = header_end
        .checked_add(central.packed_size)
        .ok_or(DecodeError::Integrity)?;
    if data_end > boundary {
        return Err(DecodeError::Integrity);
    }
    let end = if descriptor {
        let wide = local_wide
            || central.central_wide_sizes
            || central.size >= u64::from(u32::MAX)
            || central.packed_size >= u64::from(u32::MAX);
        verify_descriptor(source, data_end, boundary, wide, central)?
    } else {
        data_end
    };
    check(control)?;
    Ok(LocalRecord {
        data_start: header_end,
        data_end,
        end,
        extra,
    })
}
fn verify_descriptor(
    source: &mut dyn ReadSource,
    at: u64,
    boundary: u64,
    wide: bool,
    central: &record::Record,
) -> Result<u64, DecodeError> {
    let size = if wide { 20 } else { 12 };
    let mut bytes = [0u8; 24];
    read_at(source, at, &mut bytes[..size], boundary)?;
    let matches = |bytes: &[u8]| {
        let (packed, unpacked) = if wide {
            (n64(bytes, 4), n64(bytes, 12))
        } else {
            (u64::from(n32(bytes, 4)), u64::from(n32(bytes, 8)))
        };
        n32(bytes, 0) == central.crc && packed == central.packed_size && unpacked == central.size
    };
    // CRC can equal the optional signature, so the unsigned form has priority.
    if matches(&bytes[..size]) {
        return Ok(at + size as u64);
    }
    if bytes[..4] != *b"PK\x07\x08" {
        return Err(DecodeError::Integrity);
    }
    read_at(source, at, &mut bytes[..size + 4], boundary)?;
    if matches(&bytes[4..size + 4]) {
        Ok(at + size as u64 + 4)
    } else {
        Err(DecodeError::Integrity)
    }
}
fn read_to_sink(
    input: &mut dyn Read,
    buffer: &mut [u8],
    size: u64,
    crc: Option<u32>,
    sink: &mut dyn DecodedSink,
    control: &Cancellation,
) -> Result<(), DecodeError> {
    let mut checksum = Crc32::new();
    let mut actual = 0u64;
    loop {
        check(control)?;
        let count = input.read(buffer).map_err(read_error)?;
        if count == 0 {
            break;
        }
        actual = actual.checked_add(count as u64).ok_or(DecodeError::Limit)?;
        if actual > size {
            return Err(DecodeError::Integrity);
        }
        if crc.is_some() {
            checksum.update(&buffer[..count]);
        }
        sink.write_chunk(&buffer[..count])?;
    }
    if actual != size || crc.is_some_and(|expected| checksum.checksum() != expected) {
        return Err(DecodeError::Integrity);
    }
    check(control)
}
fn check(control: &Cancellation) -> Result<(), DecodeError> {
    control.check().map_err(|_| DecodeError::Cancelled)
}
fn scan_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        _ => DecodeError::Integrity,
    }
}
pub(super) fn read_error(error: io::Error) -> DecodeError {
    if let Some(value) = error
        .get_ref()
        .and_then(|error| error.downcast_ref::<ScanError>())
    {
        return scan_error(*value);
    }
    if matches!(
        error.kind(),
        io::ErrorKind::InvalidData | io::ErrorKind::UnexpectedEof
    ) {
        DecodeError::Integrity
    } else {
        DecodeError::SourceIo
    }
}

#[cfg(test)]
#[path = "payload_aes_tests.rs"]
mod aes_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use xdzip_codec_api::{MetadataCodec, ScanLimits};
    struct Memory(Vec<u8>);
    impl DecodedSink for Memory {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }
    }
    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/extract")
                .join(format!("{name}.zip")),
        )
        .unwrap()
    }
    fn set16(bytes: &mut [u8], at: usize, value: u16) {
        bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn set32(bytes: &mut [u8], at: usize, value: u32) {
        bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn owned_pair(duplicate_names: bool) -> (Vec<u8>, record::Directory) {
        let first: &[u8] = if duplicate_names {
            b"same.txt"
        } else {
            b"first.txt"
        };
        let second: &[u8] = if duplicate_names {
            b"same.txt"
        } else {
            b"second.txt"
        };
        let bytes = crate::legacy_fixture::zip(&[(first, 0, &[]), (second, 0, &[])]);
        let directory = record::read_directory(
            &mut Cursor::new(&bytes),
            &Default::default(),
            Default::default(),
        )
        .unwrap();
        (bytes, directory)
    }
    #[test]
    fn stored_crc_cannot_authorize_payload_overlapping_the_next_local_record() {
        let (mut bytes, directory) = owned_pair(false);
        let first = &directory.records[0];
        let start = first.local_offset as usize + 30 + first.name.len();
        let end = directory.central_offset as usize;
        let size = (end - start) as u32;
        let crc = crate::legacy_fixture::crc(&bytes[start..end]);
        for at in [
            first.local_offset as usize + 14,
            first.central_offset as usize + 16,
        ] {
            set32(&mut bytes, at, crc);
            set32(&mut bytes, at + 4, size);
            set32(&mut bytes, at + 8, size);
        }
        let control = Cancellation::default();
        let mut source = Cursor::new(bytes);
        let rows = ZipCodec
            .scan(&mut source, &control, Default::default())
            .unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].size, u64::from(size));
        let mut decoder = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
        let mut output = Memory(Vec::new());
        assert_eq!(
            decoder.decode(&rows[0], &mut output, &control),
            Err(DecodeError::Integrity)
        );
        assert!(output.0.is_empty());
    }
    #[test]
    fn descriptor_cannot_reuse_the_next_local_header_as_matching_fields() {
        let (mut bytes, directory) = owned_pair(false);
        let first = &directory.records[0];
        let second = &directory.records[1];
        // An owned CRC preimage generated independently with Python zlib. Its
        // checksum equals the local-header signature, allowing the next header
        // to look like a complete unsigned descriptor to an unbounded reader.
        let body = b"owned descriptor!\xFD\xE4\x84\x30";
        assert_eq!(body.len(), crate::legacy_fixture::CONTENT.len());
        assert_eq!(crate::legacy_fixture::crc(body), 0x04034b50);
        let start = first.local_offset as usize + 30 + first.name.len();
        bytes[start..start + body.len()].copy_from_slice(body);
        set16(&mut bytes, first.local_offset as usize + 6, 8);
        for at in [14, 18, 22] {
            set32(&mut bytes, first.local_offset as usize + at, 0);
        }
        set16(&mut bytes, first.central_offset as usize + 8, 8);
        set32(&mut bytes, first.central_offset as usize + 16, 0x04034b50);
        // Version and unsupported method 21 occupy exactly the two size fields
        // of that forged descriptor. Metadata may browse unsupported methods.
        set16(
            &mut bytes,
            second.local_offset as usize + 4,
            body.len() as u16,
        );
        set16(
            &mut bytes,
            second.local_offset as usize + 8,
            body.len() as u16,
        );
        set16(
            &mut bytes,
            second.central_offset as usize + 6,
            body.len() as u16,
        );
        set16(
            &mut bytes,
            second.central_offset as usize + 10,
            body.len() as u16,
        );
        let at = second.local_offset as usize;
        assert_eq!(
            (n32(&bytes, at), n32(&bytes, at + 4), n32(&bytes, at + 8)),
            (0x04034b50, body.len() as u32, body.len() as u32)
        );
        let control = Cancellation::default();
        let mut source = Cursor::new(bytes);
        let rows = ZipCodec
            .scan(&mut source, &control, Default::default())
            .unwrap();
        let mut decoder = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
        let mut output = Memory(Vec::new());
        assert_eq!(
            decoder.decode(&rows[0], &mut output, &control),
            Err(DecodeError::Integrity)
        );
        assert!(output.0.is_empty());
    }
    #[test]
    fn distinct_duplicate_names_decode_even_when_central_and_physical_orders_differ() {
        let (mut bytes, directory) = owned_pair(true);
        let mut expected = Vec::new();
        for (index, record) in directory.records.iter().enumerate() {
            let start = record.local_offset as usize + 30 + record.name.len();
            bytes[start] = b'A' + index as u8;
            let content = bytes[start..start + record.size as usize].to_vec();
            let crc = crate::legacy_fixture::crc(&content);
            set32(&mut bytes, record.local_offset as usize + 14, crc);
            set32(&mut bytes, record.central_offset as usize + 16, crc);
            expected.push(content);
        }
        let first = directory.records[0].central_offset as usize;
        let second = directory.records[1].central_offset as usize;
        let end = (directory.central_offset + directory.central_size) as usize;
        let first_record = bytes[first..second].to_vec();
        let second_record = bytes[second..end].to_vec();
        bytes[first..first + second_record.len()].copy_from_slice(&second_record);
        bytes[first + second_record.len()..end].copy_from_slice(&first_record);
        expected.reverse();
        let control = Cancellation::default();
        let mut source = Cursor::new(bytes);
        let rows = ZipCodec
            .scan(&mut source, &control, Default::default())
            .unwrap();
        assert_eq!(rows[0].raw_name, rows[1].raw_name);
        assert_ne!(rows[0].id, rows[1].id);
        let mut decoder = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
        for (entry, expected) in rows.iter().zip(expected) {
            let mut output = Memory(Vec::new());
            decoder.decode(entry, &mut output, &control).unwrap();
            assert_eq!(output.0, expected);
        }
    }
    #[test]
    fn duplicate_or_out_of_range_local_offsets_never_open_a_payload_reader() {
        for outside in [None, Some(0), Some(1)] {
            let (mut bytes, directory) = owned_pair(true);
            let control = Cancellation::default();
            let snapshot = ZipCodec
                .scan(&mut Cursor::new(&bytes), &control, Default::default())
                .unwrap();
            let offset = outside.map_or(directory.records[0].local_offset, |extra| {
                directory.central_offset + extra
            });
            set32(
                &mut bytes,
                directory.records[1].central_offset as usize + 42,
                offset as u32,
            );
            assert!(matches!(
                ZipCodec.open_payload(&mut Cursor::new(bytes), &snapshot, &control),
                Err(DecodeError::Integrity)
            ));
        }
    }
    #[test]
    fn actual_store_deflate_and_both_descriptor_widths_decode_the_same_bytes() {
        let expected = "xdzip 实际解压\n".as_bytes().repeat(100);
        for name in ["stored", "deflate", "descriptor", "descriptor64"] {
            let control = Cancellation::default();
            let mut source = Cursor::new(fixture(name));
            let rows = ZipCodec
                .scan(&mut source, &control, ScanLimits::default())
                .unwrap();
            let mut decoder = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
            let mut output = Memory(Vec::new());
            decoder
                .decode(&rows[0], &mut output, &control)
                .unwrap_or_else(|e| panic!("{name}: {e}"));
            assert_eq!(output.0, expected, "{name}");
        }
    }
    #[test]
    fn sfx_payload_offsets_are_absolute_after_directory_relocation() {
        let expected = "xdzip 实际解压\n".as_bytes().repeat(100);
        for name in ["stored", "deflate", "descriptor", "descriptor64"] {
            let mut bytes = b"MZ owned SFX test prefix\0".to_vec();
            bytes.extend_from_slice(&fixture(name));
            let control = Cancellation::default();
            let mut source = Cursor::new(bytes);
            let rows = ZipCodec
                .scan(&mut source, &control, ScanLimits::default())
                .unwrap();
            let mut decoder = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
            let mut output = Memory(Vec::new());
            decoder.decode(&rows[0], &mut output, &control).unwrap();
            assert_eq!(output.0, expected, "{name}");
        }
    }
    #[test]
    fn bad_crc_header_descriptor_no_end_and_trailing_data_never_verify() {
        for name in [
            "crc_bad",
            "header_bad",
            "descriptor_bad",
            "descriptor64_bad",
            "no_end",
            "trailing",
        ] {
            let control = Cancellation::default();
            let mut source = Cursor::new(fixture(name));
            let rows = ZipCodec
                .scan(&mut source, &control, ScanLimits::default())
                .unwrap();
            let mut decoder = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
            assert_eq!(
                decoder.decode(&rows[0], &mut Memory(Vec::new()), &control),
                Err(DecodeError::Integrity),
                "{name}"
            );
        }
    }
    #[test]
    fn changed_central_snapshot_and_cancellation_are_rejected() {
        let control = Cancellation::default();
        let mut source = Cursor::new(fixture("stored"));
        let mut rows = ZipCodec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        rows[0].size += 1;
        assert!(matches!(
            ZipCodec.open_payload(&mut source, &rows, &control),
            Err(DecodeError::Integrity)
        ));
        control.cancel();
        assert!(matches!(
            ZipCodec.open_payload(&mut source, &rows, &control),
            Err(DecodeError::Cancelled)
        ));
    }
}
