//! Bounded development probe. All file access belongs to this host, not the codec.
use std::io::{self, Cursor, Read, Seek, SeekFrom, Write};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
use xdzip_codec_api::{Cancellation, MetadataCodec, ReadSource, ScanLimits};
use xdzip_codec_rar::RarCodec;
use zeroize::Zeroizing;

struct Source {
    bytes: Cursor<Vec<u8>>,
    lengths: Vec<u64>,
}
impl Read for Source {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        self.bytes.read(b)
    }
}
impl Seek for Source {
    fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(p)
    }
}
impl ReadSource for Source {
    fn volume_lengths(&self) -> Option<&[u64]> {
        Some(&self.lengths)
    }
}
struct Output {
    bytes: Option<Vec<u8>>,
    hash: xdzip_crypto::SecretSha256,
    written: u64,
    auxiliary: u64,
}
impl DecodedSink for Output {
    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.auxiliary = self
            .auxiliary
            .checked_add(bytes)
            .filter(|n| *n <= 20 * 1024 * 1024 * 1024)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
    fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
        let next = self
            .written
            .checked_add(b.len() as u64)
            .ok_or(DecodeError::Limit)?;
        let maximum = if self.bytes.is_some() {
            8 * 1024 * 1024
        } else {
            8 * 1024 * 1024 * 1024
        };
        if next > maximum {
            return Err(DecodeError::Limit);
        }
        if let Some(bytes) = &mut self.bytes {
            bytes.extend(b);
        }
        self.hash.update(b);
        self.written = next;
        Ok(())
    }
}
fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    for b in bytes {
        write!(out, "{b:02x}").expect("String formatting");
    }
    out
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let operation = args.next().ok_or("operation")?;
    let selected = if operation == "decode" || operation == "digest" {
        Some(args.next().ok_or("selected")?.parse::<usize>()?)
    } else if operation == "metadata" {
        None
    } else {
        return Err("operation".into());
    };
    let mut source = Source {
        bytes: Cursor::new(Vec::new()),
        lengths: Vec::new(),
    };
    for path in args {
        let mut file = std::fs::File::open(path)?;
        let n = file.metadata()?.len();
        if n > 8 * 1024 * 1024 || source.bytes.get_ref().len() as u64 + n > 8 * 1024 * 1024 {
            return Err("fixture_limit".into());
        }
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        source.lengths.push(bytes.len() as u64);
        source.bytes.get_mut().extend(bytes);
    }
    let control = Cancellation::default();
    // Development-only public fixture password: kept out of argv and reports.
    let password = std::env::var("XDZIP_RAR_PROBE_PASSWORD")
        .ok()
        .map(Zeroizing::new);
    let password = password.as_ref().map(|p| p.as_str());
    let rows =
        RarCodec.scan_with_password(&mut source, &control, ScanLimits::default(), password)?;
    if let Some(selected) = selected {
        let entry = rows.get(selected).ok_or("selected")?;
        let mut payload =
            RarCodec.open_payload_with_password(&mut source, &rows, &control, password)?;
        let mut output = Output {
            bytes: (operation == "decode").then(Vec::new),
            hash: xdzip_crypto::SecretSha256::new(),
            written: 0,
            auxiliary: 0,
        };
        payload.decode_with_password(entry, &mut output, &control, password)?;
        if let Some(bytes) = output.bytes {
            io::stdout().lock().write_all(&bytes)?;
        } else {
            println!("{}\t{}", output.written, hex(&output.hash.finish()[..]));
        }
    } else {
        for row in rows {
            println!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                row.id.get(),
                row.method,
                row.size,
                row.packed_size,
                u8::from(row.encrypted),
                u8::from(row.directory),
                u8::from(row.link),
                hex(&row.raw_name),
                hex(row.name_bytes())
            );
        }
    }
    Ok(())
}
