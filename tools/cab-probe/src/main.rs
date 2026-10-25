//! Non-shipping CAB probe. Only explicitly named files become source volumes.
use serde_json::{Value, json};
use std::io::{self, Read, Seek, SeekFrom};
use xdzip_codec_api::encode::{
    CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeSettings, RewriteCodec,
    RewriteInput,
};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
use xdzip_codec_api::{Cancellation, MetadataCodec, ReadSource, ScanError, ScanLimits};
use xdzip_codec_cab::CabCodec;

const MAX_STORED: usize = 8 * 1024 * 1024;
const MAX_OUTPUT: usize = 16 * 1024 * 1024;
const MAX_SOURCE: u64 = 256 * 1024 * 1024;
const MAX_READS: u64 = 512 * 1024 * 1024;
const MAX_REPORT: usize = 64 * 1024 * 1024;

struct Volume {
    file: std::fs::File,
    base: u64,
    length: u64,
}
struct FixtureSource {
    volumes: Vec<Volume>,
    lengths: Vec<u64>,
    names: Vec<Vec<u8>>,
    length: u64,
    position: u64,
    bytes_read: u64,
    maximum_read_offset: u64,
}
impl FixtureSource {
    fn open(paths: &[std::path::PathBuf]) -> Result<Self, Box<dyn std::error::Error>> {
        if paths.is_empty() || paths.len() > 1024 {
            return Err("volume_count_limit".into());
        }
        let mut source = Self {
            volumes: Vec::new(),
            lengths: Vec::new(),
            names: Vec::new(),
            length: 0,
            position: 0,
            bytes_read: 0,
            maximum_read_offset: 0,
        };
        for path in paths {
            if std::fs::symlink_metadata(path)?.file_type().is_symlink() {
                return Err("linked_fixture_source".into());
            }
            let file = std::fs::File::open(path)?;
            let metadata = file.metadata()?;
            let end = source
                .length
                .checked_add(metadata.len())
                .ok_or("source_size_limit")?;
            if !metadata.is_file() || end > MAX_SOURCE {
                return Err("source_size_limit".into());
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or("volume_filename")?;
            if name.len() > 4096 {
                return Err("volume_name_limit".into());
            }
            source.lengths.push(metadata.len());
            source.names.push(name.as_bytes().to_vec());
            source.volumes.push(Volume {
                file,
                base: source.length,
                length: metadata.len(),
            });
            source.length = end;
        }
        Ok(source)
    }
}
impl Read for FixtureSource {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() || self.position >= self.length {
            return Ok(0);
        }
        let volume = self
            .volumes
            .iter_mut()
            .find(|volume| self.position < volume.base + volume.length)
            .ok_or_else(|| io::Error::other("source_volume_bounds"))?;
        if volume.file.metadata()?.len() != volume.length {
            return Err(io::Error::other("changed_fixture_source"));
        }
        let local = self.position - volume.base;
        let count = bytes
            .len()
            .min(64 * 1024)
            .min((volume.length - local) as usize);
        if self
            .bytes_read
            .checked_add(count as u64)
            .is_none_or(|size| size > MAX_READS)
        {
            return Err(io::Error::other("probe_input_work_limit"));
        }
        volume.file.seek(SeekFrom::Start(local))?;
        let count = volume.file.read(&mut bytes[..count])?;
        self.position += count as u64;
        self.bytes_read += count as u64;
        self.maximum_read_offset = self.maximum_read_offset.max(self.position);
        Ok(count)
    }
}
impl Seek for FixtureSource {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let value = match position {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
            SeekFrom::End(n) => i128::from(self.length) + i128::from(n),
        };
        self.position = u64::try_from(value).map_err(|_| io::Error::other("invalid_seek"))?;
        Ok(self.position)
    }
}
impl ReadSource for FixtureSource {
    fn volume_lengths(&self) -> Option<&[u64]> {
        Some(&self.lengths)
    }
    fn volume_names(&self) -> Option<&[Vec<u8>]> {
        Some(&self.names)
    }
}

struct CancelAfterRead<'a> {
    input: &'a mut FixtureSource,
    control: &'a Cancellation,
}
impl Read for CancelAfterRead<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let count = self.input.read(bytes)?;
        self.control.cancel();
        Ok(count)
    }
}
impl Seek for CancelAfterRead<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.input.seek(position)
    }
}
impl ReadSource for CancelAfterRead<'_> {
    fn volume_lengths(&self) -> Option<&[u64]> {
        self.input.volume_lengths()
    }
    fn volume_names(&self) -> Option<&[Vec<u8>]> {
        self.input.volume_names()
    }
}

struct BytesSink(Vec<u8>, u64);
impl DecodedSink for BytesSink {
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.1 = self.1.checked_add(bytes).ok_or(DecodeError::Limit)?;
        if self.1 > MAX_READS {
            return Err(DecodeError::Limit);
        }
        Ok(())
    }
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self
            .0
            .len()
            .checked_add(bytes.len())
            .is_none_or(|size| size > MAX_STORED)
        {
            return Err(DecodeError::Limit);
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}
struct RejectSink;
impl DecodedSink for RejectSink {
    fn charge_skipped(&mut self, _: u64) -> Result<(), DecodeError> {
        Err(DecodeError::Limit)
    }
    fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
        Err(DecodeError::Limit)
    }
}

struct CancelSink<'a>(&'a Cancellation);
impl DecodedSink for CancelSink<'_> {
    fn charge_skipped(&mut self, _: u64) -> Result<(), DecodeError> {
        self.0.cancel();
        Ok(())
    }
    fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
        self.0.cancel();
        Ok(())
    }
}

struct EmptyInput;
impl EncodeInput for EmptyInput {
    fn entries(&self) -> &[EncodeEntry] {
        &[]
    }
    fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Err(EncodeError::Source)
    }
}

struct SourceIo<'a>(&'a mut FixtureSource);
impl Read for SourceIo<'_> {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("probe_injected_source_io"))
    }
}
impl Seek for SourceIo<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.0.seek(position)
    }
}
impl ReadSource for SourceIo<'_> {
    fn volume_lengths(&self) -> Option<&[u64]> {
        self.0.volume_lengths()
    }
    fn volume_names(&self) -> Option<&[Vec<u8>]> {
        self.0.volume_names()
    }
}

struct PayloadIo<'a> {
    source: &'a mut FixtureSource,
    failed: &'a std::cell::Cell<bool>,
}
impl Read for PayloadIo<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.failed.get() {
            Err(io::Error::other("probe_payload_source_io"))
        } else {
            self.source.read(bytes)
        }
    }
}
impl Seek for PayloadIo<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.source.seek(position)
    }
}
impl ReadSource for PayloadIo<'_> {
    fn volume_lengths(&self) -> Option<&[u64]> {
        self.source.volume_lengths()
    }
    fn volume_names(&self) -> Option<&[Vec<u8>]> {
        self.source.volume_names()
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        result.push(DIGITS[(byte >> 4) as usize] as char);
        result.push(DIGITS[(byte & 15) as usize] as char);
    }
    result
}

fn run() -> Result<Value, Box<dyn std::error::Error>> {
    let mut paths = Vec::new();
    let mut guards = false;
    let mut args = std::env::args_os().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--guards" {
            guards = true;
        } else if argument == "--volume" {
            paths.push(std::path::PathBuf::from(
                args.next().ok_or("missing_volume_path")?,
            ));
        } else if argument.to_string_lossy().starts_with("--") {
            return Err("unknown_probe_option".into());
        } else {
            paths.push(std::path::PathBuf::from(argument));
        }
    }
    let codec = CabCodec;
    let mut source = FixtureSource::open(&paths)?;
    let control = Cancellation::default();
    let entries = codec.scan(
        &mut source,
        &control,
        ScanLimits {
            entries: 4096,
            total_name_bytes: 1024 * 1024,
            display_name_bytes: 1024 * 1024,
            ..ScanLimits::default()
        },
    )?;
    let mut rows = Vec::new();
    let mut output_bytes = 0usize;
    {
        let mut reader = codec.open_payload(&mut source, &entries, &control)?;
        for entry in &entries {
            if !entry.directory && entry.size > MAX_STORED as u64 {
                return Err("probe_file_output_limit".into());
            }
            if !entry.directory {
                output_bytes = output_bytes
                    .checked_add(usize::try_from(entry.size)?)
                    .ok_or("probe_output_limit")?;
                if output_bytes > MAX_OUTPUT {
                    return Err("probe_output_limit".into());
                }
            }
            let mut row = json!({"id":entry.id.get(), "name_hex":hex(entry.name_bytes()),
                "raw_name_hex":hex(&entry.raw_name), "decoded_name":entry.decoded_name,
                "name_encoding":format!("{:?}", entry.name_encoding),
                "directory":entry.directory, "link":entry.link, "warning":entry.warning,
                "size":entry.size, "format":entry.format.label()});
            if !entry.directory {
                let mut sink = BytesSink(Vec::new(), 0);
                match reader.decode(entry, &mut sink, &control) {
                    Ok(()) => {
                        row["hex"] = json!(hex(&sink.0));
                        row["actual_skipped_bytes"] = json!(sink.1);
                    }
                    Err(error) if entry.link || entry.warning => {
                        row["decode_error"] = json!(error.to_string());
                    }
                    Err(error) => return Err(error.into()),
                }
            }
            rows.push(row);
        }
    }
    let mut checks = Vec::new();
    if guards {
        let mut check = |name: &str, passed: bool| -> Result<(), Box<dyn std::error::Error>> {
            checks.push(json!({"case":name, "passed":passed}));
            if passed {
                Ok(())
            } else {
                Err(format!("guard_failed:{name}").into())
            }
        };
        let cancelled = Cancellation::default();
        cancelled.cancel();
        check(
            "scan_pre_cancelled",
            codec.scan(&mut source, &cancelled, ScanLimits::default()) == Err(ScanError::Cancelled),
        )?;
        check(
            "scan_source_io",
            codec.scan(&mut SourceIo(&mut source), &control, ScanLimits::default())
                == Err(ScanError::Io),
        )?;
        let during = Cancellation::default();
        let mut interrupt = CancelAfterRead {
            input: &mut source,
            control: &during,
        };
        check(
            "scan_cancelled_after_read",
            codec.scan(&mut interrupt, &during, ScanLimits::default()) == Err(ScanError::Cancelled),
        )?;
        for name in [
            "entries",
            "name_bytes",
            "total_name_bytes",
            "display_name_bytes",
            "read_bytes",
        ] {
            let mut limits = ScanLimits::default();
            match name {
                "entries" => limits.entries = 0,
                "name_bytes" => limits.name_bytes = 0,
                "total_name_bytes" => limits.total_name_bytes = 0,
                "display_name_bytes" => limits.display_name_bytes = 0,
                _ => limits.read_bytes = 0,
            }
            check(
                name,
                codec.scan(&mut source, &control, limits) == Err(ScanError::Limit),
            )?;
        }
        let mut changed = entries.clone();
        changed.first_mut().ok_or("guard_requires_records")?.size ^= 1;
        check(
            "changed_snapshot",
            matches!(
                codec.open_payload(&mut source, &changed, &control),
                Err(DecodeError::Integrity)
            ),
        )?;
        if let Some(entry) = entries
            .iter()
            .find(|entry| !entry.directory && !entry.link && !entry.warning && entry.size > 0)
        {
            let reader = codec.open_payload(&mut source, &entries, &cancelled);
            check(
                "payload_open_pre_cancelled",
                reader
                    .as_ref()
                    .is_err_and(|error| *error == DecodeError::Cancelled),
            )?;
            // Drop the Result to release the exclusive source borrow on every branch.
            drop(reader);
            let decode_cancel = Cancellation::default();
            let mut reader = codec.open_payload(&mut source, &entries, &decode_cancel)?;
            decode_cancel.cancel();
            check(
                "decode_pre_cancelled",
                reader.decode(entry, &mut BytesSink(Vec::new(), 0), &decode_cancel)
                    == Err(DecodeError::Cancelled),
            )?;
            drop(reader);
            let mut reader = codec.open_payload(&mut source, &entries, &control)?;
            check(
                "sink_resource_limit",
                reader.decode(entry, &mut RejectSink, &control) == Err(DecodeError::Limit),
            )?;
            drop(reader);
            let during_decode = Cancellation::default();
            let mut reader = codec.open_payload(&mut source, &entries, &during_decode)?;
            check(
                "decode_cancelled_after_output_or_skip",
                reader.decode(entry, &mut CancelSink(&during_decode), &during_decode)
                    == Err(DecodeError::Cancelled),
            )?;
            drop(reader);
            let mut reader = codec.open_payload(&mut source, &entries, &control)?;
            let mut changed = entry.clone();
            changed.size ^= 1;
            check(
                "changed_decode_entry",
                reader.decode(&changed, &mut BytesSink(Vec::new(), 0), &control)
                    == Err(DecodeError::Integrity),
            )?;
            drop(reader);
            let injected = std::cell::Cell::new(false);
            let mut failing = PayloadIo {
                source: &mut source,
                failed: &injected,
            };
            let mut reader = codec.open_payload(&mut failing, &entries, &control)?;
            injected.set(true);
            check(
                "decode_source_io",
                reader.decode(entry, &mut BytesSink(Vec::new(), 0), &control)
                    == Err(DecodeError::SourceIo),
            )?;
            drop(reader);
            if let Some(later) = entries
                .iter()
                .rev()
                .find(|row| !row.directory && row.size > 0 && row.id != entry.id)
            {
                let mut reader = codec.open_payload(&mut source, &entries, &control)?;
                check(
                    "solid_selection_budget",
                    reader.decode(later, &mut RejectSink, &control) == Err(DecodeError::Limit),
                )?;
                drop(reader);
                let cancel = Cancellation::default();
                let mut reader = codec.open_payload(&mut source, &entries, &cancel)?;
                check(
                    "solid_selection_cancel",
                    reader.decode(later, &mut CancelSink(&cancel), &cancel)
                        == Err(DecodeError::Cancelled),
                )?;
            }
        }
        let special: Vec<_> = rows
            .iter()
            .filter(|row| row["link"] == true || row["warning"] == true)
            .collect();
        if !special.is_empty() {
            check(
                "special_records_rejected",
                special.iter().all(|row| {
                    row["directory"] == true || row["decode_error"] == "unsupported_feature"
                }),
            )?;
        }
        let settings = EncodeSettings {
            method: "copy",
            level: 0,
            solid: false,
            encrypt_headers: false,
            password: None,
        };
        let mut encoded = Vec::new();
        check(
            "read_only_creation",
            codec.encode(&EmptyInput, &mut encoded, settings, &control, &mut |_| {})
                == Err(EncodeError::Unsupported)
                && encoded.is_empty(),
        )?;
        let mut rewritten = io::Cursor::new(Vec::new());
        check(
            "read_only_rewrite",
            codec.rewrite(
                &mut source,
                RewriteInput {
                    keep: &[],
                    renames: &[],
                    additions: &EmptyInput,
                    settings,
                },
                &mut rewritten,
                &control,
                &mut |_| {},
            ) == Err(EncodeError::Unsupported)
                && rewritten.get_ref().is_empty(),
        )?;
    }
    Ok(
        json!({"passed":true,"format":"CAB","source_length":source.length,
        "source_kind":"physical_files","volume_lengths":source.lengths,"volume_names_hex":source.names.iter().map(|name|hex(name)).collect::<Vec<_>>(),
        "bytes_read":source.bytes_read,"maximum_read_offset":source.maximum_read_offset,
        "rows":rows,"guards":checks}),
    )
}

fn main() {
    match run() {
        Ok(result) => {
            let text = result.to_string();
            if text.len() > MAX_REPORT {
                eprintln!("{}", json!({"passed":false,"error":"probe_report_limit"}));
                std::process::exit(1);
            }
            println!("{text}");
        }
        Err(error) => {
            eprintln!("{}", json!({"passed":false,"error":error.to_string()}));
            std::process::exit(1);
        }
    }
}
