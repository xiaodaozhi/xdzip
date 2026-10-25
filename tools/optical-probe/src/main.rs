//! Non-shipping optical interoperability probe. Filesystem publication stays in m3-probe.
use serde_json::{Value, json};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::Path;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
use xdzip_codec_api::{Cancellation, Format, MetadataCodec, ReadSource, ScanError, ScanLimits};
use xdzip_codec_optical::OpticalCodec;

const MAX_STORED: usize = 8 * 1024 * 1024;

struct Segment {
    offset: u64,
    bytes: Vec<u8>,
}

struct VirtualSource {
    length: u64,
    position: u64,
    segments: Vec<Segment>,
    bytes_read: u64,
    maximum_read_offset: u64,
}

impl VirtualSource {
    fn from_path(path: &Path, virtual_mode: bool) -> Result<Self, Box<dyn std::error::Error>> {
        let mut segments = Vec::new();
        let length;
        if virtual_mode {
            let manifest: Value = serde_json::from_slice(&std::fs::read(path)?)?;
            length = manifest["length"]
                .as_u64()
                .ok_or("invalid_virtual_length")?;
            let records = manifest["segments"]
                .as_array()
                .ok_or("invalid_virtual_segments")?;
            if records.len() > 16 {
                return Err("virtual_segment_limit".into());
            }
            let mut stored = 0usize;
            for record in records {
                let offset = record["offset"].as_u64().ok_or("invalid_virtual_offset")?;
                let name = record["file"].as_str().ok_or("invalid_virtual_file")?;
                if Path::new(name).components().count() != 1 || name.contains(['/', '\\']) {
                    return Err("invalid_virtual_file".into());
                }
                let bytes = std::fs::read(path.parent().ok_or("virtual_parent")?.join(name))?;
                stored = stored
                    .checked_add(bytes.len())
                    .ok_or("virtual_size_limit")?;
                if stored > MAX_STORED
                    || offset
                        .checked_add(bytes.len() as u64)
                        .is_none_or(|end| end > length)
                {
                    return Err("virtual_size_limit".into());
                }
                segments.push(Segment { offset, bytes });
            }
            segments.sort_by_key(|segment| segment.offset);
            if segments
                .windows(2)
                .any(|pair| pair[0].offset + pair[0].bytes.len() as u64 > pair[1].offset)
            {
                return Err("overlapping_virtual_segments".into());
            }
        } else {
            let bytes = std::fs::read(path)?;
            if bytes.len() > MAX_STORED {
                return Err("fixture_size_limit".into());
            }
            length = bytes.len() as u64;
            segments.push(Segment { offset: 0, bytes });
        }
        Ok(Self {
            length,
            position: 0,
            segments,
            bytes_read: 0,
            maximum_read_offset: 0,
        })
    }
}

impl Read for VirtualSource {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let count = bytes
            .len()
            .min(self.length.saturating_sub(self.position) as usize);
        let end = self.position + count as u64;
        bytes[..count].fill(0);
        for segment in &self.segments {
            let start = self.position.max(segment.offset);
            let stop = end.min(segment.offset + segment.bytes.len() as u64);
            if start < stop {
                bytes[(start - self.position) as usize..(stop - self.position) as usize]
                    .copy_from_slice(
                        &segment.bytes
                            [(start - segment.offset) as usize..(stop - segment.offset) as usize],
                    );
            }
        }
        self.position = end;
        self.bytes_read += count as u64;
        self.maximum_read_offset = self.maximum_read_offset.max(end);
        Ok(count)
    }
}

impl Seek for VirtualSource {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let position = match position {
            SeekFrom::Start(value) => i128::from(value),
            SeekFrom::Current(value) => i128::from(self.position) + i128::from(value),
            SeekFrom::End(value) => i128::from(self.length) + i128::from(value),
        };
        self.position = u64::try_from(position).map_err(|_| io::Error::other("invalid_seek"))?;
        Ok(self.position)
    }
}
impl ReadSource for VirtualSource {}

struct CancelAfterRead<'a> {
    input: &'a mut VirtualSource,
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
impl ReadSource for CancelAfterRead<'_> {}

struct BytesSink(Vec<u8>);
impl DecodedSink for BytesSink {
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
    fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
        Err(DecodeError::Limit)
    }
}

struct CancelSink<'a>(&'a Cancellation);
impl DecodedSink for CancelSink<'_> {
    fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
        self.0.cancel();
        Ok(())
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
    let mut args: Vec<_> = std::env::args().skip(1).collect();
    let guards = args.iter().any(|arg| arg == "--guards");
    let virtual_mode = args.iter().any(|arg| arg == "--virtual");
    args.retain(|arg| arg != "--guards" && arg != "--virtual");
    if args.len() != 2 {
        return Err("provide_fixture_iso_or_udf_and_optional_guards_virtual".into());
    }
    let format = match args[1].as_str() {
        "iso" => Format::Iso,
        "udf" => Format::Udf,
        _ => return Err("invalid_format".into()),
    };
    let codec = OpticalCodec { format };
    let mut source = VirtualSource::from_path(Path::new(&args[0]), virtual_mode)?;
    let control = Cancellation::default();
    let entries = codec.scan(&mut source, &control, ScanLimits::default())?;
    let mut rows = Vec::new();
    {
        let mut reader = codec.open_payload(&mut source, &entries, &control)?;
        for entry in &entries {
            let mut row = json!({"id":entry.id.get(), "name_hex":hex(entry.name_bytes()),
                "raw_name_hex":hex(&entry.raw_name), "decoded_name":entry.decoded_name,
                "name_encoding":format!("{:?}", entry.name_encoding),
                "directory":entry.directory, "link":entry.link, "warning":entry.warning,
                "size":entry.size, "format":entry.format.label()});
            if !entry.directory {
                let mut sink = BytesSink(Vec::new());
                match reader.decode(entry, &mut sink, &control) {
                    Ok(()) => {
                        row["hex"] = json!(hex(&sink.0));
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
                reader.decode(entry, &mut BytesSink(Vec::new()), &decode_cancel)
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
                "decode_cancelled_after_output",
                reader.decode(entry, &mut CancelSink(&during_decode), &during_decode)
                    == Err(DecodeError::Cancelled),
            )?;
            drop(reader);
            let mut reader = codec.open_payload(&mut source, &entries, &control)?;
            let mut changed = entry.clone();
            changed.size ^= 1;
            check(
                "changed_decode_entry",
                reader.decode(&changed, &mut BytesSink(Vec::new()), &control)
                    == Err(DecodeError::Integrity),
            )?;
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
    }
    Ok(
        json!({"passed":true,"format":format.label(),"source_length":source.length,
        "bytes_read":source.bytes_read,"maximum_read_offset":source.maximum_read_offset,
        "rows":rows,"guards":checks}),
    )
}

fn main() {
    match run() {
        Ok(result) => println!("{result}"),
        Err(error) => {
            eprintln!("{}", json!({"passed":false,"error":error.to_string()}));
            std::process::exit(1);
        }
    }
}
