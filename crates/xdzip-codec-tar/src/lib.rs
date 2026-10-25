//! Bounded TAR scanning and GZIP composition. Codecs never receive filesystem paths.
mod rewrite;
mod writer;
use flate2::read::MultiGzDecoder;
use std::io::{self, Read, SeekFrom};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
};
use xdzip_domain::identity::EntryId;

/// TAR container, optionally behind exactly one GZIP filter.
pub struct TarCodec {
    /// Decode/encode GZIP before/after the TAR container.
    pub gzip: bool,
}
const EXTENSION_LIMIT: u64 = 64 * 1024;
const STREAM_LIMIT: u64 = 24 * 1024 * 1024 * 1024;

struct Stream<'a> {
    reader: Box<dyn Read + 'a>,
    control: &'a Cancellation,
    remaining: u64,
    position: u64,
    failure: Option<ScanError>,
}
struct SourceGuard<'a> {
    source: &'a mut dyn ReadSource,
    control: &'a Cancellation,
    remaining: u64,
}
impl Read for SourceGuard<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        self.control.check().map_err(io::Error::other)?;
        if self.remaining == 0 {
            return Err(io::Error::other(ScanError::Limit));
        }
        let take = output
            .len()
            .min(self.remaining.min(usize::MAX as u64) as usize);
        let n = self
            .source
            .read(&mut output[..take])
            .map_err(|_| io::Error::other(ScanError::Io))?;
        self.remaining -= n as u64;
        Ok(n)
    }
}
impl Read for Stream<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let fault = self
            .control
            .check()
            .err()
            .or_else(|| (self.remaining == 0).then_some(ScanError::Limit));
        if let Some(error) = fault {
            self.failure = Some(error);
            return Err(io::Error::other(error));
        }
        let length = bytes
            .len()
            .min(self.remaining.min(usize::MAX as u64) as usize);
        let n = self.reader.read(&mut bytes[..length]).inspect_err(|e| {
            self.failure = Some(
                e.get_ref()
                    .and_then(|e| e.downcast_ref::<ScanError>())
                    .copied()
                    .unwrap_or(ScanError::Invalid),
            );
        })?;
        self.remaining -= n as u64;
        self.position += n as u64;
        Ok(n)
    }
}
impl TarCodec {
    fn stream<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        limit: u64,
    ) -> Result<Stream<'a>, ScanError> {
        source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
        let guarded = SourceGuard {
            source,
            control,
            remaining: limit,
        };
        let reader: Box<dyn Read + 'a> = if self.gzip {
            Box::new(MultiGzDecoder::new(guarded))
        } else {
            Box::new(guarded)
        };
        Ok(Stream {
            reader,
            control,
            remaining: limit,
            position: 0,
            failure: None,
        })
    }
    fn inspect(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<(Vec<Entry>, Vec<u64>), ScanError> {
        let mut stream = self.stream(source, control, limits.read_bytes)?;
        let result = scan(&mut stream, self.gzip, limits);
        stream.failure.map_or(result, Err)
    }
}
fn skip(stream: &mut Stream<'_>, mut count: u64) -> Result<(), ScanError> {
    let mut bytes = [0; 64 * 1024];
    while count != 0 {
        let take = count.min(bytes.len() as u64) as usize;
        stream
            .read_exact(&mut bytes[..take])
            .map_err(|_| ScanError::Invalid)?;
        count -= take as u64;
    }
    Ok(())
}
fn padded(size: u64) -> Result<u64, ScanError> {
    size.checked_add(511)
        .map(|n| n & !511)
        .ok_or(ScanError::Limit)
}
// UTC civil conversion keeps the existing display wire shape without guessing a
// timezone for ZIP timestamps. Values outside its representable years stay unknown.
fn display_time(seconds: u64) -> u32 {
    if seconds > 4_354_819_199 {
        return 0;
    }
    let days = (seconds / 86400) as i64 + 719468;
    let era = days / 146097;
    let doe = days - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let month_part = (5 * doy + 2) / 153;
    let day = doy - (153 * month_part + 2) / 5 + 1;
    let month = month_part + if month_part < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    if !(1980..=2107).contains(&year) {
        return 0;
    }
    (((year - 1980) as u32) << 25)
        | ((month as u32) << 21)
        | ((day as u32) << 16)
        | (((seconds % 86400 / 3600) as u32) << 11)
        | (((seconds % 3600 / 60) as u32) << 5)
        | ((seconds % 60 / 2) as u32)
}
fn validate_global_comments(mut data: &[u8]) -> Result<(), ScanError> {
    while !data.is_empty() {
        let space = data
            .iter()
            .position(|b| *b == b' ')
            .ok_or(ScanError::Invalid)?;
        if space == 0 || !data[..space].iter().all(u8::is_ascii_digit) {
            return Err(ScanError::Invalid);
        }
        let length = std::str::from_utf8(&data[..space])
            .map_err(|_| ScanError::Invalid)?
            .parse::<usize>()
            .map_err(|_| ScanError::Invalid)?;
        let record = data.get(..length).ok_or(ScanError::Invalid)?;
        let value = record.get(space + 1..).ok_or(ScanError::Invalid)?;
        if value.last() != Some(&b'\n') {
            return Err(ScanError::Invalid);
        }
        let equals = value
            .iter()
            .position(|b| *b == b'=')
            .ok_or(ScanError::Invalid)?;
        if &value[..equals] != b"comment" {
            return Err(ScanError::Unsupported);
        }
        data = &data[length..];
    }
    Ok(())
}
fn scan(
    stream: &mut Stream<'_>,
    gzip: bool,
    limits: ScanLimits,
) -> Result<(Vec<Entry>, Vec<u64>), ScanError> {
    let mut rows = Vec::new();
    let mut offsets = Vec::new();
    let mut names = 0usize;
    let mut legacy = Vec::new();
    let mut next_utf8 = false;
    let mut next_binary = None;
    let mut next_name: Option<Vec<u8>> = None;
    let mut next_size = None;
    let mut next_time = None;
    let mut next_special = false;
    let mut extensions = 0usize;
    let mut local_extensions = 0usize;
    loop {
        let mut block = [0; 512];
        stream
            .read_exact(&mut block)
            .map_err(|_| ScanError::Invalid)?;
        if block.iter().all(|b| *b == 0) {
            // Require both TAR end blocks and drain the GZIP trailer. Reject concatenated
            // hidden TAR records/nonzero garbage rather than silently showing a partial index.
            if local_extensions != 0 {
                return Err(ScanError::Invalid);
            }
            stream
                .read_exact(&mut block)
                .map_err(|_| ScanError::Invalid)?;
            if block.iter().any(|b| *b != 0) {
                return Err(ScanError::Invalid);
            }
            let mut tail = [0; 64 * 1024];
            loop {
                let n = stream.read(&mut tail).map_err(|_| ScanError::Invalid)?;
                if n == 0 {
                    xdzip_codec_api::names::resolve(
                        &mut rows,
                        &legacy,
                        false,
                        stream.control,
                        limits,
                    )?;
                    return Ok((rows, offsets));
                }
                if tail[..n].iter().any(|b| *b != 0) {
                    return Err(ScanError::Invalid);
                }
            }
        }
        let header = tar::Header::from_byte_slice(&block);
        let sum: u32 = block
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if (148..156).contains(&i) {
                    32
                } else {
                    u32::from(*b)
                }
            })
            .sum();
        if header.cksum().map_err(|_| ScanError::Invalid)? != sum {
            return Err(ScanError::Invalid);
        }
        let kind = header.entry_type();
        let size = header.entry_size().map_err(|_| ScanError::Invalid)?;
        if kind.is_gnu_longname()
            || kind.is_pax_local_extensions()
            || kind.is_pax_global_extensions()
            || kind.is_gnu_longlink()
        {
            extensions += 1;
            if extensions > 4 || size > EXTENSION_LIMIT {
                return Err(ScanError::Limit);
            }
            // Long link targets change payload interpretation and remain unsupported.
            if kind.is_gnu_longlink() {
                return Err(ScanError::Unsupported);
            }
            let mut data = vec![0; size as usize];
            stream
                .read_exact(&mut data)
                .map_err(|_| ScanError::Invalid)?;
            skip(stream, padded(size)? - size)?;
            if kind.is_pax_global_extensions() {
                // Git archives carry their commit in a global comment. Only this
                // inert annotation is admitted; never ignore global path, size,
                // link, sparse or unknown semantics. No global state is retained.
                validate_global_comments(&data)?;
                continue;
            }
            local_extensions += 1;
            if kind.is_gnu_longname() {
                if next_name.is_some() {
                    return Err(ScanError::Invalid);
                }
                if data.last() == Some(&0) {
                    data.pop();
                }
                next_name = Some(data);
            } else {
                for item in tar::PaxExtensions::new(&data) {
                    let item = item.map_err(|_| ScanError::Invalid)?;
                    match item.key_bytes() {
                        b"path" => {
                            if next_name.is_some() {
                                return Err(ScanError::Invalid);
                            }
                            next_name = Some(item.value_bytes().to_vec());
                            next_utf8 = true;
                        }
                        b"hdrcharset" => {
                            if next_binary.is_some() {
                                return Err(ScanError::Invalid);
                            }
                            next_binary = Some(match item.value_bytes() {
                                b"BINARY" => true,
                                b"ISO-IR 10646 2000 UTF-8" => false,
                                _ => return Err(ScanError::Unsupported),
                            });
                        }
                        b"mtime" => {
                            next_time = item
                                .value()
                                .ok()
                                .and_then(|v| v.split('.').next())
                                .and_then(|v| v.parse::<u64>().ok());
                        }
                        b"size" => {
                            if next_size.is_some() {
                                return Err(ScanError::Invalid);
                            }
                            next_size = Some(
                                item.value()
                                    .map_err(|_| ScanError::Invalid)?
                                    .parse::<u64>()
                                    .map_err(|_| ScanError::Invalid)?,
                            );
                        }
                        key if key.starts_with(b"GNU.sparse") || key == b"SCHILY.filetype" => {
                            next_special = true
                        }
                        _ => {}
                    }
                }
            }
            continue;
        }
        let unlabelled = !next_utf8 || next_binary.unwrap_or(false);
        let mut name = next_name
            .take()
            .unwrap_or_else(|| header.path_bytes().into_owned());
        if name.len() > limits.name_bytes {
            return Err(ScanError::Limit);
        }
        let size = next_size.take().unwrap_or(size);
        let modified = next_time
            .take()
            .or_else(|| header.mtime().ok())
            .map(display_time)
            .unwrap_or(0);
        let directory = kind.is_dir();
        // Common system-tar roots are structural markers, not output directories.
        let mut prefix = 0;
        while name[prefix..].starts_with(b"./") {
            prefix += 2;
        }
        name.drain(..prefix);
        if directory && (name.is_empty() || name == b".") && size == 0 {
            extensions = 0;
            local_extensions = 0;
            next_special = false;
            next_utf8 = false;
            next_binary = None;
            continue;
        }
        if directory && !name.ends_with(b"/") {
            name.push(b'/');
        }
        names = names.checked_add(name.len()).ok_or(ScanError::Limit)?;
        if rows.len() >= limits.entries
            || name.len() > limits.name_bytes
            || names > limits.total_name_bytes
        {
            return Err(ScanError::Limit);
        }
        let warning = next_special || !(kind.is_file() || directory) || (directory && size != 0);
        legacy.push(unlabelled);
        offsets.push(stream.position);
        rows.push(Entry {
            id: EntryId::try_from(rows.len() as u64 + 1).map_err(|_| ScanError::Limit)?,
            raw_name: name,
            decoded_name: None,
            name_encoding: xdzip_codec_api::names::NameEncoding::Utf8,
            display_name: String::new(),
            directory,
            link: kind.is_symlink() || kind.is_hard_link(),
            unix_mode: 0,
            size,
            packed_size: if gzip { 0 } else { size },
            method: if gzip { 8 } else { 0 },
            modified_dos: modified,
            encrypted: false,
            password_supported: false,
            warning,
            format: if gzip { Format::TarGzip } else { Format::Tar },
        });
        skip(stream, padded(size)?)?;
        extensions = 0;
        local_extensions = 0;
        next_special = false;
        next_utf8 = false;
        next_binary = None;
    }
}
impl MetadataCodec for TarCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        self.inspect(source, control, limits).map(|(rows, _)| rows)
    }
}
struct Reader<'a> {
    stream: Stream<'a>,
    rows: Vec<Entry>,
    offsets: Vec<u64>,
}
fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Io => DecodeError::SourceIo,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}
impl PayloadCodec for TarCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let limits = ScanLimits {
            read_bytes: STREAM_LIMIT,
            ..ScanLimits::default()
        };
        let (rows, offsets) = self
            .inspect(source, control, limits)
            .map_err(decode_error)?;
        if rows.len() != snapshot.len() || rows.iter().zip(snapshot).any(|(a, b)| a != b) {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(Reader {
            stream: self
                .stream(source, control, STREAM_LIMIT)
                .map_err(decode_error)?,
            rows,
            offsets,
        }))
    }
}
impl PayloadReader for Reader<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        let at = usize::try_from(entry.id.get() - 1).map_err(|_| DecodeError::Integrity)?;
        if self.rows.get(at) != Some(entry) {
            return Err(DecodeError::Integrity);
        }
        if entry.warning {
            return Err(DecodeError::Unsupported);
        }
        let distance = self.offsets[at]
            .checked_sub(self.stream.position)
            .ok_or(DecodeError::Integrity)?;
        let result = skip(&mut self.stream, distance);
        if let Some(e) = self.stream.failure {
            return Err(decode_error(e));
        }
        result.map_err(decode_error)?;
        let mut remaining = entry.size;
        let mut bytes = [0; 64 * 1024];
        while remaining != 0 {
            control.check().map_err(decode_error)?;
            let count = remaining.min(bytes.len() as u64) as usize;
            if self.stream.read_exact(&mut bytes[..count]).is_err() {
                return Err(decode_error(
                    self.stream.failure.unwrap_or(ScanError::Invalid),
                ));
            }
            sink.write_chunk(&bytes[..count])?;
            remaining -= count as u64;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use xdzip_codec_api::encode::{
        CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeSettings,
    };
    struct Inputs {
        entries: Vec<EncodeEntry>,
        data: Vec<Vec<u8>>,
    }
    impl EncodeInput for Inputs {
        fn entries(&self) -> &[EncodeEntry] {
            &self.entries
        }
        fn open(&self, at: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            Ok(Box::new(Cursor::new(&self.data[at])))
        }
    }
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }
    }
    fn input() -> Inputs {
        Inputs {
            entries: vec![
                EncodeEntry {
                    name: "资料".into(),
                    directory: true,
                    size: 0,
                    modified_unix: 1_700_000_000,
                },
                EncodeEntry {
                    name: format!("资料/{}/数据.txt", "深层".repeat(45)),
                    directory: false,
                    size: 7,
                    modified_unix: 1_700_000_000,
                },
                EncodeEntry {
                    name: "empty".into(),
                    directory: false,
                    size: 0,
                    modified_unix: 0,
                },
            ],
            data: vec![vec![], b"payload".to_vec(), vec![]],
        }
    }
    fn encoded(gzip: bool) -> Vec<u8> {
        let mut bytes = Vec::new();
        TarCodec { gzip }
            .encode(
                &input(),
                &mut bytes,
                EncodeSettings {
                    solid: false,
                    encrypt_headers: false,
                    method: if gzip { "deflate" } else { "store" },
                    level: 6,
                    password: None,
                },
                &Cancellation::default(),
                &mut |_| {},
            )
            .unwrap();
        bytes
    }
    fn scan_bytes(bytes: Vec<u8>, gzip: bool) -> Result<Vec<Entry>, ScanError> {
        TarCodec { gzip }.scan(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
    }
    fn global_header(data: &[u8]) -> Vec<u8> {
        let mut header = tar::Header::new_ustar();
        header.set_path("pax_global_header").unwrap();
        header.set_entry_type(tar::EntryType::XGlobalHeader);
        header.set_size(data.len() as u64);
        header.set_cksum();
        let mut bytes = header.as_bytes().to_vec();
        bytes.extend(data);
        bytes.resize(512 + padded(data.len() as u64).unwrap() as usize, 0);
        bytes
    }
    fn pax_record(key: &str, value: &str) -> Vec<u8> {
        let mut length = key.len() + value.len() + 4;
        loop {
            let record = format!("{length} {key}={value}\n");
            if record.len() == length {
                return record.into_bytes();
            }
            length = record.len();
        }
    }
    #[test]
    fn raw_filename_encoding_keeps_utf8_and_resolves_legacy_bytes() {
        for name in ["中文.txt".as_bytes(), b"\xD6\xD0\xCE\xC4.txt"] {
            let mut header = tar::Header::new_ustar();
            header.set_entry_type(tar::EntryType::Regular);
            header.set_size(0);
            header.set_mode(0o644);
            header.as_mut_bytes()[..name.len()].copy_from_slice(name);
            header.set_cksum();
            let mut builder = tar::Builder::new(Vec::new());
            builder.append(&header, io::empty()).unwrap();
            let bytes = builder.into_inner().unwrap();
            let rows = scan_bytes(bytes, false).unwrap();
            assert_eq!(rows[0].raw_name, name);
            assert!(!rows[0].warning);
            assert_eq!(rows[0].display_name, "中文.txt");
            assert_eq!(rows[0].name_bytes(), "中文.txt".as_bytes());
        }
    }
    #[test]
    fn git_global_comments_preserve_local_paths_and_payload_offsets() {
        let mut comment = pax_record("comment", "0123456789abcdef0123456789abcdef01234567");
        comment.extend(pax_record("comment", "annotation\nwith a newline"));
        let mut plain = global_header(&comment);
        plain.extend(encoded(false));
        for gzip in [false, true] {
            let bytes = if gzip {
                let mut encoder = flate2::write::GzEncoder::new(Vec::new(), Default::default());
                encoder.write_all(&plain).unwrap();
                encoder.finish().unwrap()
            } else {
                plain.clone()
            };
            let codec = TarCodec { gzip };
            let cancel = Cancellation::default();
            let mut source = Cursor::new(bytes);
            let rows = codec
                .scan(&mut source, &cancel, ScanLimits::default())
                .unwrap();
            assert_eq!(rows.len(), 3);
            assert_eq!(rows[1].raw_name, input().entries[1].name.as_bytes());
            assert!(rows.iter().all(|row| !row.warning));
            let mut payload = codec.open_payload(&mut source, &rows, &cancel).unwrap();
            let mut sink = Sink(vec![]);
            payload.decode(&rows[1], &mut sink, &cancel).unwrap();
            assert_eq!(sink.0, b"payload");
        }
        let mut empty = global_header(&comment);
        empty.extend([0; 1024]);
        assert!(scan_bytes(empty, false).unwrap().is_empty());
    }
    #[test]
    fn global_overrides_malformed_records_and_extension_floods_are_rejected() {
        for key in [
            "path",
            "size",
            "mtime",
            "linkpath",
            "GNU.sparse.map",
            "SCHILY.filetype",
            "unknown",
        ] {
            let mut bytes = global_header(&pax_record(key, "value"));
            bytes.extend(encoded(false));
            assert_eq!(
                scan_bytes(bytes, false).unwrap_err(),
                ScanError::Unsupported
            );
        }
        for record in [
            b"0 comment=x\n".as_slice(),
            b"999 comment=x\n",
            b"\n12 path=x\n",
            b"13 comment=x",
            b"+13 comment=x\n",
        ] {
            let mut bytes = global_header(record);
            bytes.extend(encoded(false));
            assert_eq!(scan_bytes(bytes, false).unwrap_err(), ScanError::Invalid);
        }
        let mut bytes = global_header(&pax_record("comment", "git")).repeat(5);
        bytes.extend(encoded(false));
        assert_eq!(scan_bytes(bytes, false).unwrap_err(), ScanError::Limit);
        let bytes = global_header(&vec![b'x'; EXTENSION_LIMIT as usize + 1]);
        assert_eq!(scan_bytes(bytes, false).unwrap_err(), ScanError::Limit);
    }
    #[test]
    fn host_read_failures_keep_their_stable_io_class() {
        struct Broken;
        impl ReadSource for Broken {}
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("owned fault"))
            }
        }
        impl std::io::Seek for Broken {
            fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
                Ok(0)
            }
        }
        for gzip in [false, true] {
            assert_eq!(
                TarCodec { gzip }
                    .scan(&mut Broken, &Cancellation::default(), ScanLimits::default())
                    .unwrap_err(),
                ScanError::Io
            );
        }
    }
    #[test]
    fn utc_display_dates_keep_seconds_and_boundaries() {
        assert_eq!(display_time(0), 0);
        assert_eq!(display_time(315532800), (1 << 21) | (1 << 16));
        assert_eq!(
            display_time(1700000000),
            (43 << 25) | (11 << 21) | (14 << 16) | (22 << 11) | (13 << 5) | 10
        );
    }
    #[test]
    fn unicode_long_names_empty_records_and_sequential_selected_payloads() {
        for gzip in [false, true] {
            let mut source = Cursor::new(encoded(gzip));
            let codec = TarCodec { gzip };
            let cancel = Cancellation::default();
            let rows = codec
                .scan(&mut source, &cancel, ScanLimits::default())
                .unwrap();
            assert_eq!(rows.len(), 3);
            assert!(!rows[1].warning);
            assert_eq!(rows[1].raw_name, input().entries[1].name.as_bytes());
            let mut payload = codec.open_payload(&mut source, &rows, &cancel).unwrap();
            let mut sink = Sink(vec![]);
            payload.decode(&rows[1], &mut sink, &cancel).unwrap();
            assert_eq!(sink.0, b"payload");
            payload.decode(&rows[2], &mut sink, &cancel).unwrap();
            assert_eq!(
                payload.decode(&rows[0], &mut sink, &cancel),
                Err(DecodeError::Integrity)
            );
        }
    }
    #[test]
    fn truncated_headers_missing_end_blocks_gzip_crc_and_trailing_junk_fail() {
        for gzip in [false, true] {
            let bytes = encoded(gzip);
            for length in [0, 100, bytes.len() / 2, bytes.len() - 1] {
                assert!(scan_bytes(bytes[..length].to_vec(), gzip).is_err());
            }
        }
        let mut gzip = encoded(true);
        let at = gzip.len() - 8;
        gzip[at] ^= 1;
        assert!(scan_bytes(gzip, true).is_err());
        let mut tar = encoded(false);
        tar.push(1);
        assert!(scan_bytes(tar, false).is_err());
        let mut tar = encoded(false);
        tar[0] ^= 1;
        assert!(scan_bytes(tar, false).is_err());
    }
    #[test]
    fn cancellation_byte_name_entry_and_extension_budgets_are_enforced() {
        let bytes = encoded(false);
        let cancel = Cancellation::default();
        cancel.cancel();
        assert_eq!(
            TarCodec { gzip: false }
                .scan(&mut Cursor::new(&bytes), &cancel, ScanLimits::default())
                .unwrap_err(),
            ScanError::Cancelled
        );
        for limits in [
            ScanLimits {
                read_bytes: 1024,
                ..Default::default()
            },
            ScanLimits {
                entries: 1,
                ..Default::default()
            },
            ScanLimits {
                name_bytes: 2,
                ..Default::default()
            },
        ] {
            assert_eq!(
                TarCodec { gzip: false }
                    .scan(&mut Cursor::new(&bytes), &Cancellation::default(), limits)
                    .unwrap_err(),
                ScanError::Limit
            );
        }
        let mut header = tar::Header::new_gnu();
        header.set_path("long").unwrap();
        header.set_entry_type(tar::EntryType::GNULongName);
        header.set_size(EXTENSION_LIMIT + 1);
        header.set_cksum();
        assert_eq!(
            scan_bytes(header.as_bytes().to_vec(), false).unwrap_err(),
            ScanError::Limit
        );
    }
    #[test]
    fn links_devices_and_traversal_are_not_payload_capabilities() {
        for kind in [
            tar::EntryType::Symlink,
            tar::EntryType::Link,
            tar::EntryType::Fifo,
            tar::EntryType::Char,
            tar::EntryType::Block,
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_path("item").unwrap();
            header.set_size(0);
            header.set_entry_type(kind);
            header.set_cksum();
            let mut bytes = header.as_bytes().to_vec();
            bytes.resize(1536, 0);
            let mut source = Cursor::new(bytes);
            let cancel = Cancellation::default();
            let codec = TarCodec { gzip: false };
            let rows = codec
                .scan(&mut source, &cancel, ScanLimits::default())
                .unwrap();
            assert!(rows[0].warning);
            let mut reader = codec.open_payload(&mut source, &rows, &cancel).unwrap();
            assert_eq!(
                reader.decode(&rows[0], &mut Sink(vec![]), &cancel),
                Err(DecodeError::Unsupported)
            );
        }
        let mut unsafe_header = tar::Header::new_gnu();
        unsafe_header.set_path("safe").unwrap();
        unsafe_header.set_size(0);
        unsafe_header.set_cksum();
        let mut bytes = unsafe_header.as_bytes().to_vec();
        bytes.resize(1536, 0);
        bytes[..100].fill(0);
        bytes[..9].copy_from_slice(b"../escape");
        let header = tar::Header::from_byte_slice(&bytes[..512]).clone();
        let mut header = header;
        header.set_cksum();
        bytes[..512].copy_from_slice(header.as_bytes());
        assert!(scan_bytes(bytes, false).unwrap()[0].warning);
        let mut builder = tar::Builder::new(Vec::new());
        builder
            .append_pax_extensions([("path", b"../escape".as_slice())])
            .unwrap();
        builder
            .append_data(&mut unsafe_header, "safe", io::empty())
            .unwrap();
        assert!(scan_bytes(builder.into_inner().unwrap(), false).unwrap()[0].warning);
    }
    #[test]
    fn gzip_members_compose_one_tar_stream_and_non_tar_gzip_is_rejected() {
        let plain = encoded(false);
        let mut joined = Vec::new();
        for half in plain.chunks(plain.len() / 2) {
            let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            e.write_all(half).unwrap();
            joined.extend(e.finish().unwrap());
        }
        assert_eq!(scan_bytes(joined, true).unwrap().len(), 3);
        let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        e.write_all(b"not a TAR").unwrap();
        assert!(scan_bytes(e.finish().unwrap(), true).is_err());
    }
    #[test]
    fn empty_gzip_members_cannot_evade_input_budget_and_special_extensions_fail() {
        let empty = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default())
            .finish()
            .unwrap();
        let mut members = empty.repeat(1000);
        members.extend(encoded(true));
        let error = TarCodec { gzip: true }
            .scan(
                &mut Cursor::new(members),
                &Cancellation::default(),
                ScanLimits {
                    read_bytes: 1024,
                    ..Default::default()
                },
            )
            .unwrap_err();
        assert_eq!(error, ScanError::Limit);
        let mut header = tar::Header::new_gnu();
        header.set_path("extension").unwrap();
        header.set_size(0);
        header.set_entry_type(tar::EntryType::GNULongLink);
        header.set_cksum();
        assert_eq!(
            scan_bytes(header.as_bytes().to_vec(), false).unwrap_err(),
            ScanError::Unsupported
        );
    }
    #[test]
    fn source_length_mismatch_and_non_tar_capabilities_never_succeed() {
        let mut inputs = input();
        inputs.entries[1].size += 1;
        assert!(
            TarCodec { gzip: false }
                .encode(
                    &inputs,
                    &mut Vec::new(),
                    EncodeSettings {
                        solid: false,
                        encrypt_headers: false,
                        method: "store",
                        level: 6,
                        password: None
                    },
                    &Cancellation::default(),
                    &mut |_| {}
                )
                .is_err()
        );
        assert!(
            TarCodec { gzip: true }
                .encode(
                    &input(),
                    &mut Vec::new(),
                    EncodeSettings {
                        solid: false,
                        encrypt_headers: false,
                        method: "deflate",
                        level: 6,
                        password: Some("secret")
                    },
                    &Cancellation::default(),
                    &mut |_| {}
                )
                .is_err()
        );
    }
}

#[cfg(test)]
mod names_tests;
