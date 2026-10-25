// crates/xdzip-codec-zip/src/writer.rs

//! Streaming ZIP records, independently implemented from APPNOTE 6.3.10 and WinZip AE-2.
use crate::{ZipCodec, aes::AesWriter, record::Record};
use std::io::{self, Read, Seek, Write};
use xdzip_codec_api::{
    Cancellation, MAX_ENTRIES, MetadataCodec, ReadSource, ScanLimits,
    encode::{
        CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeOutput, EncodeProgress,
        EncodeSettings,
    },
    payload::{DecodeError, DecodedSink, PayloadCodec},
};
use xdzip_compression::{Crc32, DeflateWriter};

pub(crate) const WIDE: u64 = u32::MAX as u64;
fn out(result: io::Result<()>) -> Result<(), EncodeError> {
    result.map_err(|_| EncodeError::Output)
}
pub(crate) fn check(cancel: &Cancellation) -> Result<(), EncodeError> {
    cancel.check().map_err(|_| EncodeError::Cancelled)
}
pub(crate) fn scan_error(error: xdzip_codec_api::ScanError) -> EncodeError {
    use xdzip_codec_api::ScanError;
    match error {
        ScanError::Cancelled => EncodeError::Cancelled,
        ScanError::Limit => EncodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => EncodeError::Source,
        ScanError::Unsupported => EncodeError::Unsupported,
        _ => EncodeError::Invalid,
    }
}
pub(crate) fn settings_valid(settings: EncodeSettings<'_>) -> Result<(), EncodeError> {
    if !matches!(settings.method, "store" | "deflate")
        || ![1, 6, 9].contains(&settings.level)
        || settings.solid
        || settings.encrypt_headers
    {
        return Err(EncodeError::Unsupported);
    }
    Ok(())
}

struct Count<'a> {
    output: &'a mut dyn Write,
    position: u64,
}
impl Write for Count<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let count = self.output.write(bytes)?;
        self.position = self
            .position
            .checked_add(count as u64)
            .ok_or_else(|| io::Error::other("ZIP output overflow"))?;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

/// A failed encode leaves an incomplete staging stream. Only the host may verify and publish it.
pub(crate) struct ArchiveWriter<'a> {
    output: Count<'a>,
    records: Vec<Record>,
}
impl Write for ArchiveWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.output.write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}
impl<'a> ArchiveWriter<'a> {
    pub(crate) fn new(output: &'a mut dyn Write) -> Self {
        Self {
            output: Count {
                output,
                position: 0,
            },
            records: Vec::new(),
        }
    }
    pub(crate) fn position(&self) -> u64 {
        self.output.position
    }
    pub(crate) fn push(&mut self, record: Record) -> Result<(), EncodeError> {
        if self.records.len() >= MAX_ENTRIES {
            return Err(EncodeError::Limit);
        }
        self.records.push(record);
        Ok(())
    }
    pub(crate) fn append(
        &mut self,
        input: &dyn EncodeInput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        settings_valid(settings)?;
        if input.entries().len().saturating_add(self.records.len()) > MAX_ENTRIES {
            return Err(EncodeError::Limit);
        }
        let mut total = 0u64;
        let mut names = 0usize;
        for (index, entry) in input.entries().iter().enumerate() {
            check(cancel)?;
            let name = if entry.directory {
                format!("{}/", entry.name.trim_end_matches('/'))
            } else {
                entry.name.clone()
            };
            names = names.checked_add(name.len()).ok_or(EncodeError::Limit)?;
            if name.is_empty()
                || name.len() > 4096
                || names > 16 * 1024 * 1024
                || (entry.directory && entry.size != 0)
            {
                return Err(EncodeError::Invalid);
            }
            let method: u16 = if entry.directory || settings.method == "store" {
                0
            } else {
                8
            };
            let encrypted = !entry.directory && settings.password.is_some();
            // A deliberately generous bound covers this encoder's RFC1951 block overhead.
            // Deciding before the local header avoids seeking even for incompressible files.
            let bound = if method == 0 {
                entry.size
            } else {
                entry
                    .size
                    .checked_add(entry.size / 8)
                    .and_then(|n| n.checked_add(1024))
                    .ok_or(EncodeError::Limit)?
            }
            .checked_add(if encrypted { 28 } else { 0 })
            .ok_or(EncodeError::Limit)?;
            let wide = entry.size >= WIDE || bound >= WIDE;
            let mut extra = timestamp_extra(entry.modified_unix)?;
            if wide {
                extra.extend_from_slice(&1u16.to_le_bytes());
                extra.extend_from_slice(&16u16.to_le_bytes());
                extra.extend_from_slice(&entry.size.to_le_bytes());
                extra.extend_from_slice(&0u64.to_le_bytes());
            }
            if encrypted {
                extra.extend_from_slice(&[0x01, 0x99, 7, 0, 2, 0, b'A', b'E', 3]);
                extra.extend_from_slice(&method.to_le_bytes());
            }
            let mut record = Record {
                name: name.into_bytes(),
                extra,
                comment: Vec::new(),
                flags: 0x0800
                    | if entry.directory { 0 } else { 8 }
                    | if encrypted { 1 } else { 0 }
                    | if method == 8 {
                        match settings.level {
                            1 => 4,
                            9 => 2,
                            _ => 0,
                        }
                    } else {
                        0
                    },
                method: if encrypted { 99 } else { method },
                version_made_by: 0x033f,
                version_needed: if wide { 45 } else { 20 },
                internal_attributes: 0,
                crc: 0,
                external_attributes: if entry.directory {
                    (0o040755 << 16) | 0x10
                } else {
                    0o100644 << 16
                },
                modified_dos: dos_time(entry.modified_unix),
                size: entry.size,
                packed_size: 0,
                local_offset: self.position(),
                central_offset: 0,
                central_wide_sizes: wide,
            };
            self.local(&record, wide)?;
            if !entry.directory {
                let start = self.position();
                let mut reader = input.open(index)?;
                let crc = if let Some(password) = settings.password {
                    let mut aes = AesWriter::new(&mut self.output, password)
                        .map_err(|_| EncodeError::Output)?;
                    let crc = compress(
                        &mut reader,
                        &mut aes,
                        method,
                        settings.level,
                        entry,
                        &mut total,
                        cancel,
                        progress,
                    )?;
                    aes.finish().map_err(|_| EncodeError::Output)?;
                    crc
                } else {
                    compress(
                        &mut reader,
                        &mut self.output,
                        method,
                        settings.level,
                        entry,
                        &mut total,
                        cancel,
                        progress,
                    )?
                };
                record.packed_size = self
                    .position()
                    .checked_sub(start)
                    .ok_or(EncodeError::Invalid)?;
                record.crc = if encrypted { 0 } else { crc };
                if !wide && record.packed_size >= WIDE {
                    return Err(EncodeError::Limit);
                }
                out(self.write_all(&0x08074b50u32.to_le_bytes()))?;
                out(self.write_all(&record.crc.to_le_bytes()))?;
                if wide {
                    out(self.write_all(&record.packed_size.to_le_bytes()))?;
                    out(self.write_all(&record.size.to_le_bytes()))?;
                } else {
                    out(self.write_all(&(record.packed_size as u32).to_le_bytes()))?;
                    out(self.write_all(&(record.size as u32).to_le_bytes()))?;
                }
            }
            self.push(record)?;
        }
        Ok(())
    }
    fn local(&mut self, record: &Record, wide: bool) -> Result<(), EncodeError> {
        let mut header = Vec::with_capacity(30);
        header.extend_from_slice(&0x04034b50u32.to_le_bytes());
        header.extend_from_slice(&record.version_needed.to_le_bytes());
        header.extend_from_slice(&record.flags.to_le_bytes());
        header.extend_from_slice(&record.method.to_le_bytes());
        header.extend_from_slice(&record.modified_dos.to_le_bytes());
        header.extend_from_slice(&0u32.to_le_bytes());
        header.extend_from_slice(&(if wide { u32::MAX } else { 0 }).to_le_bytes());
        header.extend_from_slice(&(if wide { u32::MAX } else { 0 }).to_le_bytes());
        header.extend_from_slice(&(record.name.len() as u16).to_le_bytes());
        header.extend_from_slice(&(record.extra.len() as u16).to_le_bytes());
        out(self.write_all(&header))?;
        out(self.write_all(&record.name))?;
        out(self.write_all(&record.extra))
    }
    pub(crate) fn finish(
        mut self,
        comment: &[u8],
        cancel: &Cancellation,
    ) -> Result<(), EncodeError> {
        if comment.len() > u16::MAX as usize {
            return Err(EncodeError::Limit);
        }
        check(cancel)?;
        let central = self.position();
        let records = std::mem::take(&mut self.records);
        let mut has_wide = false;
        for record in &records {
            check(cancel)?;
            has_wide |= self.central(record)?;
        }
        let size = self
            .position()
            .checked_sub(central)
            .ok_or(EncodeError::Invalid)?;
        if has_wide || records.len() >= u16::MAX as usize || central >= WIDE || size >= WIDE {
            let location = self.position();
            let mut end = Vec::with_capacity(76);
            end.extend_from_slice(&0x06064b50u32.to_le_bytes());
            end.extend_from_slice(&44u64.to_le_bytes());
            end.extend_from_slice(&0x032du16.to_le_bytes());
            end.extend_from_slice(&45u16.to_le_bytes());
            end.extend_from_slice(&[0; 8]);
            end.extend_from_slice(&(records.len() as u64).to_le_bytes());
            end.extend_from_slice(&(records.len() as u64).to_le_bytes());
            end.extend_from_slice(&size.to_le_bytes());
            end.extend_from_slice(&central.to_le_bytes());
            end.extend_from_slice(&0x07064b50u32.to_le_bytes());
            end.extend_from_slice(&0u32.to_le_bytes());
            end.extend_from_slice(&location.to_le_bytes());
            end.extend_from_slice(&1u32.to_le_bytes());
            out(self.write_all(&end))?;
        }
        let mut end = Vec::with_capacity(22);
        end.extend_from_slice(&0x06054b50u32.to_le_bytes());
        end.extend_from_slice(&[0; 4]);
        let count = records.len().min(u16::MAX as usize) as u16;
        end.extend_from_slice(&count.to_le_bytes());
        end.extend_from_slice(&count.to_le_bytes());
        end.extend_from_slice(&(size.min(WIDE) as u32).to_le_bytes());
        end.extend_from_slice(&(central.min(WIDE) as u32).to_le_bytes());
        end.extend_from_slice(&(comment.len() as u16).to_le_bytes());
        out(self.write_all(&end))?;
        out(self.write_all(comment))
    }
    fn central(&mut self, record: &Record) -> Result<bool, EncodeError> {
        let wide_sizes =
            record.central_wide_sizes || record.size >= WIDE || record.packed_size >= WIDE;
        let wide_offset = record.local_offset >= WIDE;
        let mut extra = filter_extra(&record.extra, &[1])?;
        if wide_sizes || wide_offset {
            extra.extend_from_slice(&1u16.to_le_bytes());
            extra.extend_from_slice(
                &(if wide_sizes { 16u16 } else { 0 } + if wide_offset { 8 } else { 0 })
                    .to_le_bytes(),
            );
            if wide_sizes {
                extra.extend_from_slice(&record.size.to_le_bytes());
                extra.extend_from_slice(&record.packed_size.to_le_bytes());
            }
            if wide_offset {
                extra.extend_from_slice(&record.local_offset.to_le_bytes());
            }
        }
        if record.name.len() > u16::MAX as usize
            || extra.len() > u16::MAX as usize
            || record.comment.len() > u16::MAX as usize
        {
            return Err(EncodeError::Limit);
        }
        let mut header = Vec::with_capacity(46);
        header.extend_from_slice(&0x02014b50u32.to_le_bytes());
        header.extend_from_slice(&record.version_made_by.to_le_bytes());
        header.extend_from_slice(
            &(if wide_sizes || wide_offset {
                record.version_needed.max(45)
            } else {
                record.version_needed
            })
            .to_le_bytes(),
        );
        header.extend_from_slice(&record.flags.to_le_bytes());
        header.extend_from_slice(&record.method.to_le_bytes());
        header.extend_from_slice(&record.modified_dos.to_le_bytes());
        header.extend_from_slice(&record.crc.to_le_bytes());
        header.extend_from_slice(
            &(if wide_sizes {
                u32::MAX
            } else {
                record.packed_size as u32
            })
            .to_le_bytes(),
        );
        header.extend_from_slice(
            &(if wide_sizes {
                u32::MAX
            } else {
                record.size as u32
            })
            .to_le_bytes(),
        );
        header.extend_from_slice(&(record.name.len() as u16).to_le_bytes());
        header.extend_from_slice(&(extra.len() as u16).to_le_bytes());
        header.extend_from_slice(&(record.comment.len() as u16).to_le_bytes());
        header.extend_from_slice(&0u16.to_le_bytes());
        header.extend_from_slice(&record.internal_attributes.to_le_bytes());
        header.extend_from_slice(&record.external_attributes.to_le_bytes());
        header.extend_from_slice(
            &(if wide_offset {
                u32::MAX
            } else {
                record.local_offset as u32
            })
            .to_le_bytes(),
        );
        out(self.write_all(&header))?;
        out(self.write_all(&record.name))?;
        out(self.write_all(&extra))?;
        out(self.write_all(&record.comment))?;
        Ok(wide_sizes || wide_offset)
    }
}

pub(crate) fn filter_extra(extra: &[u8], remove: &[u16]) -> Result<Vec<u8>, EncodeError> {
    let mut result = Vec::with_capacity(extra.len());
    let mut remaining = extra;
    while !remaining.is_empty() {
        if remaining.len() < 4 {
            return Err(EncodeError::Invalid);
        }
        let tag = u16::from_le_bytes([remaining[0], remaining[1]]);
        let size = usize::from(u16::from_le_bytes([remaining[2], remaining[3]]));
        let field = remaining.get(..4 + size).ok_or(EncodeError::Invalid)?;
        if !remove.contains(&tag) {
            result.extend_from_slice(field);
        }
        remaining = &remaining[4 + size..];
    }
    Ok(result)
}

#[allow(clippy::too_many_arguments)]
fn compress(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    method: u16,
    level: u8,
    entry: &EncodeEntry,
    total: &mut u64,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
) -> Result<u32, EncodeError> {
    if method == 0 {
        copying(reader, writer, entry, total, cancel, progress)
    } else {
        let mut deflate = DeflateWriter::new(writer, level).map_err(|_| EncodeError::Output)?;
        let crc = copying(reader, &mut deflate, entry, total, cancel, progress)?;
        deflate.finish().map_err(|_| EncodeError::Output)?;
        Ok(crc)
    }
}
fn copying(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    entry: &EncodeEntry,
    total: &mut u64,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
) -> Result<u32, EncodeError> {
    let mut buffer = [0; 64 * 1024];
    let mut count = 0u64;
    let mut crc = Crc32::new();
    loop {
        check(cancel)?;
        let n = reader.read(&mut buffer).map_err(|_| EncodeError::Source)?;
        if n == 0 {
            break;
        }
        count = count.checked_add(n as u64).ok_or(EncodeError::Limit)?;
        if count > entry.size {
            return Err(EncodeError::Source);
        }
        out(writer.write_all(&buffer[..n]))?;
        crc.update(&buffer[..n]);
        *total = total.checked_add(n as u64).ok_or(EncodeError::Limit)?;
        progress(EncodeProgress {
            name: &entry.name,
            input_bytes: *total,
        });
    }
    if count != entry.size {
        return Err(EncodeError::Source);
    }
    Ok(crc.checksum())
}

// ZIP carries a two-second DOS timestamp without a zone. New entries consistently use UTC.
fn dos_time(unix: i64) -> u32 {
    let unix = unix.clamp(315_532_800, 4_354_819_199) as u64;
    let mut days = unix / 86_400;
    let mut year = 1970u32;
    loop {
        let length = if leap(year) { 366 } else { 365 };
        if days < length {
            break;
        }
        days -= length;
        year += 1;
    }
    let months = [
        31u64,
        if leap(year) { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u32;
    for length in months {
        if days < length {
            break;
        }
        days -= length;
        month += 1;
    }
    let seconds = unix % 86_400;
    let date = ((year - 1980) << 9) | (month << 5) | (days as u32 + 1);
    let time = ((seconds / 3600) as u32) << 11
        | ((seconds / 60 % 60) as u32) << 5
        | (seconds % 60 / 2) as u32;
    date << 16 | time
}
fn timestamp_extra(unix: i64) -> Result<Vec<u8>, EncodeError> {
    // Info-ZIP's published extended timestamp format has a signed 32-bit UTC mtime.
    // Flags=1 omits access/creation times entirely in both local and central headers.
    // Limit it to nonnegative values: independent readers disagree about signed pre-1970 times.
    // https://libzip.org/specifications/extrafld.txt (format document, not implementation source).
    if let Ok(seconds) = i32::try_from(unix)
        && seconds >= 0
    {
        let mut extra = vec![0x55, 0x54, 5, 0, 1];
        extra.extend_from_slice(&seconds.to_le_bytes());
        return Ok(extra);
    }
    // NTFS stores 100 ns intervals from 1601; unlike DOS it preserves pre-1980 dates.
    let ticks = i128::from(unix)
        .checked_add(11_644_473_600)
        .and_then(|seconds| seconds.checked_mul(10_000_000))
        .and_then(|ticks| u64::try_from(ticks).ok())
        .ok_or(EncodeError::Unsupported)?;
    let mut extra = Vec::with_capacity(36);
    extra.extend_from_slice(&0x000au16.to_le_bytes());
    extra.extend_from_slice(&32u16.to_le_bytes());
    extra.extend_from_slice(&0u32.to_le_bytes());
    extra.extend_from_slice(&1u16.to_le_bytes());
    extra.extend_from_slice(&24u16.to_le_bytes());
    // Common ZIP assigns no meaning to atime/ctime and recommends zero for both.
    // This compatibility convention is not an APPNOTE claim that zero means absent.
    // https://commonzip.org/spec/#lastmodifiedtimestamp
    extra.extend_from_slice(&ticks.to_le_bytes());
    extra.extend_from_slice(&[0; 16]);
    Ok(extra)
}
fn leap(year: u32) -> bool {
    year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400))
}

impl CreationCodec for ZipCodec {
    fn encode(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        settings_valid(settings)?;
        let mut archive = ArchiveWriter::new(output);
        archive.append(input, settings, cancel, progress)?;
        archive.finish(&[], cancel)
    }
}

struct Source<R>(R);
impl<R: Read> Read for Source<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.0.read(bytes)
    }
}
impl<R: Seek> Seek for Source<R> {
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}
impl<R: Read + Seek> ReadSource for Source<R> {}
struct VerifySink {
    bytes: u64,
    expected: u64,
}
impl DecodedSink for VerifySink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or(DecodeError::Integrity)?;
        if self.bytes > self.expected {
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
}
impl ZipCodec {
    /// Authenticate and decode each record in a host-owned staging stream before publication.
    pub fn verify_encrypted<R: Read + Seek>(
        &self,
        reader: R,
        entries: &[EncodeEntry],
        password: &str,
        cancel: &Cancellation,
    ) -> Result<(), EncodeError> {
        let mut source = Source(reader);
        let snapshot = self
            .scan(&mut source, cancel, ScanLimits::default())
            .map_err(scan_error)?;
        if snapshot.len() != entries.len() {
            return Err(EncodeError::Invalid);
        }
        for (actual, expected) in snapshot.iter().zip(entries) {
            let name = if expected.directory {
                format!("{}/", expected.name.trim_end_matches('/'))
            } else {
                expected.name.clone()
            };
            if actual.name_bytes() != name.as_bytes()
                || actual.directory != expected.directory
                || actual.size != expected.size
                || actual.warning
                || actual.link
                || (expected.directory && actual.encrypted)
                || (!expected.directory && (!actual.encrypted || !actual.password_supported))
            {
                return Err(EncodeError::Invalid);
            }
        }
        let mut payload = self
            .open_payload(&mut source, &snapshot, cancel)
            .map_err(|error| decode_error(error, cancel))?;
        for entry in &snapshot {
            check(cancel)?;
            let mut sink = VerifySink {
                bytes: 0,
                expected: entry.size,
            };
            payload
                .decode_with_password(entry, &mut sink, cancel, Some(password))
                .map_err(|error| decode_error(error, cancel))?;
            if sink.bytes != entry.size {
                return Err(EncodeError::Invalid);
            }
        }
        Ok(())
    }
}
fn decode_error(error: DecodeError, cancel: &Cancellation) -> EncodeError {
    if check(cancel).is_err() || error == DecodeError::Cancelled {
        EncodeError::Cancelled
    } else if error == DecodeError::Limit {
        EncodeError::Limit
    } else {
        EncodeError::Invalid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    struct Inputs {
        entries: Vec<EncodeEntry>,
        bodies: Vec<Vec<u8>>,
    }
    impl EncodeInput for Inputs {
        fn entries(&self) -> &[EncodeEntry] {
            &self.entries
        }
        fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            Ok(Box::new(Cursor::new(&self.bodies[index])))
        }
    }
    fn settings<'a>(method: &'a str, level: u8, password: Option<&'a str>) -> EncodeSettings<'a> {
        EncodeSettings {
            method,
            level,
            password,
            solid: false,
            encrypt_headers: false,
        }
    }
    fn inputs() -> Inputs {
        Inputs {
            entries: vec![
                EncodeEntry {
                    name: "目录".into(),
                    directory: true,
                    size: 0,
                    modified_unix: 1_583_298_368,
                },
                EncodeEntry {
                    name: "目录/empty.txt".into(),
                    directory: false,
                    size: 0,
                    modified_unix: 0,
                },
                EncodeEntry {
                    name: "目录/data.txt".into(),
                    directory: false,
                    size: 200_000,
                    modified_unix: 1_583_298_368,
                },
            ],
            bodies: vec![
                vec![],
                vec![],
                (0..200_000).map(|n| (n % 251) as u8).collect(),
            ],
        }
    }
    #[test]
    fn streaming_creation_store_deflate_and_aes_are_independently_readable() {
        let input = inputs();
        for (method, level) in [("store", 1), ("deflate", 1), ("deflate", 6), ("deflate", 9)] {
            for password in [None, Some("密碼 with spaces")] {
                let mut output = Vec::new();
                let mut last = 0;
                ZipCodec
                    .encode(
                        &input,
                        &mut output,
                        settings(method, level, password),
                        &Cancellation::default(),
                        &mut |p| {
                            assert!(p.input_bytes >= last);
                            last = p.input_bytes;
                        },
                    )
                    .unwrap();
                assert_eq!(last, 200_000);
                let mut archive = zip::ZipArchive::new(Cursor::new(&output)).unwrap();
                assert_eq!(archive.len(), 3);
                for (index, expected) in input.entries.iter().enumerate() {
                    let mut file = if let Some(password) = password {
                        archive
                            .by_index_decrypt(index, password.as_bytes())
                            .unwrap()
                    } else {
                        archive.by_index(index).unwrap()
                    };
                    assert_eq!(file.is_dir(), expected.directory);
                    assert_eq!(file.encrypted(), password.is_some() && !expected.directory);
                    assert_eq!(
                        file.unix_mode().unwrap() & 0o777,
                        if expected.directory { 0o755 } else { 0o644 }
                    );
                    let mut bytes = Vec::new();
                    file.read_to_end(&mut bytes).unwrap();
                    assert_eq!(bytes, input.bodies[index]);
                }
                if let Some(password) = password {
                    ZipCodec
                        .verify_encrypted(
                            Cursor::new(&output),
                            &input.entries,
                            password,
                            &Cancellation::default(),
                        )
                        .unwrap();
                    assert_eq!(
                        ZipCodec.verify_encrypted(
                            Cursor::new(&output),
                            &input.entries,
                            "incorrect",
                            &Cancellation::default()
                        ),
                        Err(EncodeError::Invalid)
                    );
                }
            }
        }
    }
    #[test]
    fn aes_authentication_and_frozen_names_are_required() {
        let input = inputs();
        let mut output = Vec::new();
        ZipCodec
            .encode(
                &input,
                &mut output,
                settings("store", 6, Some("secret")),
                &Cancellation::default(),
                &mut |_| {},
            )
            .unwrap();
        let directory = crate::record::read_directory(
            &mut Cursor::new(&output),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap();
        let record = &directory.records[2];
        let payload = record.local_offset as usize + 30 + record.name.len() + record.extra.len();
        // Alter ciphertext while leaving the password verifier and directory intact.
        output[payload + 18] ^= 1;
        assert_eq!(
            ZipCodec.verify_encrypted(
                Cursor::new(&output),
                &input.entries,
                "secret",
                &Cancellation::default()
            ),
            Err(EncodeError::Invalid)
        );
        let mut expected = input.entries.clone();
        expected[2].name = "another.txt".into();
        assert_eq!(
            ZipCodec.verify_encrypted(
                Cursor::new(&output),
                &expected,
                "secret",
                &Cancellation::default()
            ),
            Err(EncodeError::Invalid)
        );
    }
    #[test]
    fn source_lengths_cancellation_and_output_failure_never_finish_archive() {
        let mut input = inputs();
        input.entries[2].size += 1;
        let mut output = Vec::new();
        assert_eq!(
            ZipCodec.encode(
                &input,
                &mut output,
                settings("deflate", 6, None),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Source)
        );
        assert!(!output.windows(4).any(|bytes| bytes == b"PK\x05\x06"));
        let cancel = Cancellation::default();
        output.clear();
        input.entries[2].size -= 1;
        assert_eq!(
            ZipCodec.encode(
                &input,
                &mut output,
                settings("store", 6, None),
                &cancel,
                &mut |_| cancel.cancel()
            ),
            Err(EncodeError::Cancelled)
        );
        struct Broken;
        impl Write for Broken {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("full"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        impl EncodeOutput for Broken {}
        assert_eq!(
            ZipCodec.encode(
                &input,
                &mut Broken,
                settings("store", 6, None),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Output)
        );
    }
    #[test]
    fn timestamps_are_utc_bounded_and_extended_when_dos_cannot_represent_them() {
        assert_eq!(dos_time(315_532_800), 0x0021_0000);
        assert_eq!(dos_time(0), 0x0021_0000);
        assert_eq!(dos_time(4_354_819_199), 0xff9f_bf7d);
        assert_eq!(dos_time(i64::MAX), 0xff9f_bf7d);
        assert_eq!(dos_time(951_827_697), 0x285d_645c); // 2000-02-29 12:34:57 UTC.
        for unix in [0, 1_583_298_368] {
            let bytes = timestamp_extra(unix).unwrap();
            assert_eq!(&bytes[..5], &[0x55, 0x54, 5, 0, 1]);
            assert_eq!(
                i32::from_le_bytes(bytes[5..9].try_into().unwrap()) as i64,
                unix
            );
        }
        for unix in [-1, 4_354_819_200] {
            let bytes = timestamp_extra(unix).unwrap();
            assert_eq!(bytes.len(), 36);
            let ticks = u64::from_le_bytes(bytes[12..20].try_into().unwrap());
            assert_eq!(
                i128::from(ticks) / 10_000_000 - 11_644_473_600,
                i128::from(unix)
            );
            assert_eq!(&bytes[20..36], &[0; 16]);
        }
        assert_eq!(timestamp_extra(3_000_000_000).unwrap().len(), 36);
        assert_eq!(timestamp_extra(i64::MIN), Err(EncodeError::Unsupported));
    }
    fn record() -> Record {
        Record {
            name: b"large.bin".to_vec(),
            extra: vec![],
            comment: vec![],
            flags: 0x800,
            method: 0,
            crc: 0,
            size: WIDE + 1,
            packed_size: WIDE + 1,
            local_offset: WIDE + 1024,
            central_offset: 0,
            modified_dos: 0x00210000,
            version_made_by: 0x033f,
            version_needed: 45,
            internal_attributes: 0,
            external_attributes: 0o100644 << 16,
            central_wide_sizes: true,
        }
    }
    #[test]
    fn zip64_records_encode_full_sizes_offsets_and_end_locator_without_truncation() {
        // A virtual output position exercises 64-bit serialization without allocating a 4 GiB fixture.
        let mut bytes = Vec::new();
        let mut writer = ArchiveWriter::new(&mut bytes);
        writer.output.position = WIDE + 4096;
        let central = writer.position();
        writer.push(record()).unwrap();
        writer.finish(b"comment", &Cancellation::default()).unwrap();
        assert_eq!(&bytes[..4], b"PK\x01\x02");
        assert_eq!(&bytes[20..28], &[255; 8]);
        assert_eq!(&bytes[42..46], &[255; 4]);
        let extra = &bytes[46 + b"large.bin".len()..];
        assert_eq!(&extra[..4], &[1, 0, 24, 0]);
        assert_eq!(
            u64::from_le_bytes(extra[4..12].try_into().unwrap()),
            WIDE + 1
        );
        assert_eq!(
            u64::from_le_bytes(extra[20..28].try_into().unwrap()),
            WIDE + 1024
        );
        let zip64 = bytes.windows(4).position(|v| v == b"PK\x06\x06").unwrap();
        assert_eq!(
            u64::from_le_bytes(bytes[zip64 + 48..zip64 + 56].try_into().unwrap()),
            central
        );
        let locator = zip64 + 56;
        assert_eq!(&bytes[locator..locator + 4], b"PK\x06\x07");
        assert_eq!(
            u64::from_le_bytes(bytes[locator + 8..locator + 16].try_into().unwrap()),
            central + zip64 as u64
        );
        let end = locator + 20;
        assert_eq!(&bytes[end + 16..end + 20], &[255; 4]);
    }
    #[test]
    fn large_declared_inputs_choose_zip64_before_streaming() {
        let input = Inputs {
            entries: vec![EncodeEntry {
                name: "large.bin".into(),
                directory: false,
                size: WIDE,
                modified_unix: 315_532_800,
            }],
            bodies: vec![vec![]],
        };
        let mut output = Vec::new();
        assert_eq!(
            ZipCodec.encode(
                &input,
                &mut output,
                settings("store", 6, None),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Source)
        );
        assert_eq!(&output[18..26], &[255; 8]);
        assert_eq!(&output[48..52], &[1, 0, 16, 0]);
        assert_eq!(u64::from_le_bytes(output[52..60].try_into().unwrap()), WIDE);
    }
}
