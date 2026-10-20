//! Strict Store/Deflate streams. The host retains every output and commit capability.
use crate::{ZipCodec, admission, aes};
use flate2::{Decompress, FlushDecompress, Status};
use rawzip::{ReaderAt, ZipArchive, ZipArchiveEntryWayfinder};
use std::cell::Cell;
use std::io::{self, Read, Seek, SeekFrom};
use std::rc::Rc;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, ReadSource, ScanError};

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
        let count = self.source.read(&mut bytes[..bound])?;
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
struct Record {
    finder: ZipArchiveEntryWayfinder,
    flags: u16,
    crc: u32,
    local_offset: u64,
    central_wide_sizes: bool,
    aes: Result<Option<aes::Profile>, DecodeError>,
}
struct ZipPayload<R> {
    archive: ZipArchive<R>,
    records: Vec<Record>,
    headers: Vec<u8>,
    output: Vec<u8>,
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
        let input = Input {
            source,
            control,
            remaining: Rc::clone(&remaining),
        };
        let mut buffer = vec![0; 256 * 1024];
        let archive = ZipArchive::from_seekable(input, &mut buffer).map_err(|_| error(control))?;
        admission::single_volume(&archive).map_err(|_| DecodeError::Integrity)?;
        if archive.entries_hint() != snapshot.len() as u64 {
            return Err(DecodeError::Integrity);
        }
        let mut entries = archive.entries(&mut buffer);
        let mut records = Vec::with_capacity(snapshot.len());
        while let Some(header) = entries.next_entry().map_err(|_| error(control))? {
            check(control)?;
            let expected = snapshot.get(records.len()).ok_or(DecodeError::Integrity)?;
            if expected.id.get() != records.len() as u64 + 1
                || header.file_path().as_bytes() != expected.raw_name
                || header.is_dir() != expected.directory
                || header.uncompressed_size_hint() != expected.size
                || header.compressed_size_hint() != expected.packed_size
                || header.compression_method().as_u16() != expected.method
                || header.flags().is_encrypted() != expected.encrypted
                || (header.flags().is_encrypted()
                    && header.compression_method().as_u16() == 99
                    && aes::supported(header.extra_fields(), header.crc32()))
                    != expected.password_supported
            {
                return Err(DecodeError::Integrity);
            }
            admission::entry_disk(&archive, &header).map_err(|_| DecodeError::Integrity)?;
            let mut fixed = [0; 46];
            archive
                .get_ref()
                .read_exact_at(&mut fixed, header.central_directory_offset())
                .map_err(read_error)?;
            records.push(Record {
                finder: header.wayfinder(),
                flags: header.flags().bits(),
                crc: header.crc32(),
                local_offset: header.local_header_offset(),
                central_wide_sizes: fixed[20..24] == [255; 4] || fixed[24..28] == [255; 4],
                aes: aes::profile(header.extra_fields()),
            });
        }
        if records.len() != snapshot.len() {
            return Err(DecodeError::Integrity);
        }
        remaining.set(INPUT_LIMIT);
        Ok(Box::new(ZipPayload {
            archive,
            records,
            headers: vec![0; BLOCK],
            output: vec![0; BLOCK],
        }))
    }
}

impl<R: ReaderAt> PayloadReader for ZipPayload<R> {
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
        let index = usize::try_from(expected.id.get() - 1).map_err(|_| DecodeError::Integrity)?;
        let record = self.records.get(index).ok_or(DecodeError::Integrity)?;
        let aes = if expected.encrypted && expected.method == 99 {
            Some(record.aes?.ok_or(DecodeError::Integrity)?)
        } else if expected.encrypted || expected.method == 99 {
            return Err(DecodeError::Unsupported);
        } else {
            None
        };
        // Only UTF-8, data descriptor and Deflate-level flags are admitted.
        let admitted_flags = if aes.is_some() { 0x080f } else { 0x080e };
        if record.flags & !admitted_flags != 0 || (expected.directory && expected.size != 0) {
            return Err(DecodeError::Unsupported);
        }
        let entry = self
            .archive
            .get_entry(record.finder)
            .map_err(|_| error(control))?;
        let header = entry
            .local_header(&mut self.headers)
            .map_err(|_| error(control))?;
        if header.file_path().as_bytes() != expected.raw_name
            || header.flags().bits() != record.flags
            || header.compression_method().as_u16() != expected.method
            || (aes.is_some()
                && aes::profile(header.extra_fields()).map_err(|_| DecodeError::Integrity)? != aes)
        {
            return Err(DecodeError::Integrity);
        }
        let descriptor = record.flags & 8 != 0;
        if (!descriptor
            && (header.crc32() != record.crc
                || header.compressed_size_hint() != expected.packed_size
                || header.uncompressed_size_hint() != expected.size))
            || (descriptor
                && ((header.crc32() != 0 && header.crc32() != record.crc)
                    || (header.compressed_size_hint() != 0
                        && header.compressed_size_hint() != expected.packed_size)
                    || (header.uncompressed_size_hint() != 0
                        && header.uncompressed_size_hint() != expected.size)))
        {
            return Err(DecodeError::Integrity);
        }
        let (start, end) = entry.compressed_data_range();
        if start > end
            || end > self.archive.directory_offset()
            || end - start != expected.packed_size
        {
            return Err(DecodeError::Integrity);
        }
        if descriptor {
            // Streaming writers can force ZIP64 locally while retaining 32-bit
            // central sizes. rawzip's descriptor width uses central sentinels;
            // determine the width from the local size sentinels as well.
            let mut fixed = [0; 30];
            self.archive
                .get_ref()
                .read_exact_at(&mut fixed, record.local_offset)
                .map_err(read_error)?;
            let wide = fixed[18..22] == [255; 4]
                || fixed[22..26] == [255; 4]
                || record.central_wide_sizes
                || expected.size >= u64::from(u32::MAX)
                || expected.packed_size >= u64::from(u32::MAX);
            verify_descriptor(
                self.archive.get_ref(),
                end,
                self.archive.directory_offset(),
                wide,
                record.crc,
                expected,
            )?;
        }
        if let Some(aes) = aes {
            // WinZip AE-2 does not use CRC and requires its stored value to be zero.
            // The private crypto framing never hides an invalid original checksum.
            if aes.version == 2 && record.crc != 0 {
                return Err(DecodeError::Integrity);
            }
            let packed = expected
                .packed_size
                .checked_sub(28)
                .ok_or(DecodeError::Integrity)?;
            if aes.method == 0 && expected.size != packed {
                return Err(DecodeError::Integrity);
            }
            let password = password.ok_or(DecodeError::PasswordRequired)?;
            let view = aes::CryptoView::new(self.archive.get_ref(), start, expected.packed_size)?;
            let mut crypto = zip::ZipArchive::new(view).map_err(zip_error)?;
            let mut plain = crypto
                .by_index_decrypt(0, password.as_bytes())
                .map_err(zip_error)?;
            let mut payload: Box<dyn Read + '_> = if aes.method == 0 {
                Box::new(&mut plain)
            } else {
                Box::new(StrictDeflate::new(&mut plain, packed))
            };
            read_to_sink(
                &mut payload,
                &mut self.output,
                expected.size,
                (aes.version == 1).then_some(record.crc),
                sink,
                control,
            )?;
            drop(payload);
            // A zero-length Store entry still needs a real read for its HMAC.
            // For Deflate, do not rely on its StreamEnd to authenticate ciphertext.
            let mut last = [0u8; 1];
            if plain.read(&mut last).map_err(read_error)? != 0 {
                return Err(DecodeError::Integrity);
            }
            return check(control);
        }
        let compressed = entry.reader();
        if expected.method == 0 && expected.size != expected.packed_size {
            return Err(DecodeError::Integrity);
        }
        let payload: Box<dyn Read + '_> = if expected.method == 0 {
            Box::new(compressed)
        } else {
            Box::new(StrictDeflate::new(compressed, expected.packed_size))
        };
        let mut verified = entry.verifying_reader(payload);
        read_to_sink(
            &mut verified,
            &mut self.output,
            expected.size,
            None,
            sink,
            control,
        )
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
    let mut checksum = rawzip::Crc32::new();
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

fn zip_error(error: zip::result::ZipError) -> DecodeError {
    match error {
        zip::result::ZipError::InvalidPassword => DecodeError::WrongPassword,
        zip::result::ZipError::Io(error) => read_error(error),
        _ => DecodeError::Integrity,
    }
}

fn verify_descriptor<R: ReaderAt>(
    source: &R,
    at: u64,
    boundary: u64,
    wide: bool,
    crc: u32,
    entry: &Entry,
) -> Result<(), DecodeError> {
    let size = if wide { 20 } else { 12 };
    let mut bytes = [0u8; 24];
    if at.checked_add(size as u64).is_none_or(|end| end > boundary) {
        return Err(DecodeError::Integrity);
    }
    source
        .read_exact_at(&mut bytes[..size], at)
        .map_err(read_error)?;
    let matches = |bytes: &[u8]| {
        let actual_crc = u32::from_le_bytes(bytes[..4].try_into().expect("bounded descriptor"));
        let (packed, unpacked) = if wide {
            (
                u64::from_le_bytes(bytes[4..12].try_into().expect("wide descriptor")),
                u64::from_le_bytes(bytes[12..20].try_into().expect("wide descriptor")),
            )
        } else {
            (
                u64::from(u32::from_le_bytes(
                    bytes[4..8].try_into().expect("descriptor"),
                )),
                u64::from(u32::from_le_bytes(
                    bytes[8..12].try_into().expect("descriptor"),
                )),
            )
        };
        actual_crc == crc && packed == entry.packed_size && unpacked == entry.size
    };
    // A CRC can equal the optional signature. Try the valid unsigned form first.
    if matches(&bytes[..size]) {
        return Ok(());
    }
    if bytes[..4] != *b"PK\x07\x08"
        || at
            .checked_add(size as u64 + 4)
            .is_none_or(|end| end > boundary)
    {
        return Err(DecodeError::Integrity);
    }
    source
        .read_exact_at(&mut bytes[..size + 4], at)
        .map_err(read_error)?;
    if matches(&bytes[4..size + 4]) {
        Ok(())
    } else {
        Err(DecodeError::Integrity)
    }
}
fn check(control: &Cancellation) -> Result<(), DecodeError> {
    control.check().map_err(|_| DecodeError::Cancelled)
}
fn error(control: &Cancellation) -> DecodeError {
    if control.check().is_err() {
        DecodeError::Cancelled
    } else {
        DecodeError::Integrity
    }
}
fn read_error(error: io::Error) -> DecodeError {
    if let Some(value) = error.get_ref().and_then(|e| e.downcast_ref::<ScanError>()) {
        return match value {
            ScanError::Cancelled => DecodeError::Cancelled,
            ScanError::Limit => DecodeError::Limit,
            _ => DecodeError::SourceIo,
        };
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

/// Require StreamEnd and exact compressed-byte consumption. Read adapters that
/// accept truncated streams merely because size/CRC match are not sufficient.
struct StrictDeflate<R> {
    input: R,
    inflater: Decompress,
    buffer: Vec<u8>,
    position: usize,
    length: usize,
    packed: u64,
    ended: bool,
}
impl<R: Read> StrictDeflate<R> {
    fn new(input: R, packed: u64) -> Self {
        Self {
            input,
            inflater: Decompress::new(false),
            buffer: vec![0; BLOCK],
            position: 0,
            length: 0,
            packed,
            ended: false,
        }
    }
}
impl<R: Read> Read for StrictDeflate<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() || self.ended {
            return Ok(0);
        }
        loop {
            if self.position == self.length {
                self.length = self.input.read(&mut self.buffer)?;
                self.position = 0;
            }
            let before_in = self.inflater.total_in();
            let before_out = self.inflater.total_out();
            let status = self
                .inflater
                .decompress(
                    &self.buffer[self.position..self.length],
                    output,
                    FlushDecompress::None,
                )
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "deflate_structure"))?;
            let consumed = (self.inflater.total_in() - before_in) as usize;
            let produced = (self.inflater.total_out() - before_out) as usize;
            self.position += consumed;
            if status == Status::StreamEnd {
                if self.inflater.total_in() != self.packed {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "deflate_trailing_bytes",
                    ));
                }
                self.ended = true;
                return Ok(produced);
            }
            if produced != 0 {
                return Ok(produced);
            }
            if consumed == 0 || self.length == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "deflate_no_end",
                ));
            }
        }
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
