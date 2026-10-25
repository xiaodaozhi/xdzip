//! Thin worker boundary. All image format logic lives in xdzip-codec-optical.
mod cab;
use std::io::{self, Read, Seek, SeekFrom};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
};
use xdzip_codec_optical::OpticalCodec;

// CXX validates layouts and borrows all slices synchronously. The C++ callbacks
// catch every exception and return stable errors. They retain no Rust pointers,
// and only use the worker's already authorized pipe protocol, never pathnames.
#[allow(unsafe_code)]
#[cxx::bridge(namespace = "xdzip::readonly_codec")]
mod ffi {
    struct Transfer {
        value: u64,
        error: u8,
    }
    unsafe extern "C++" {
        include!("host.hpp");
        fn read_bytes(offset: u64, buffer: &mut [u8]) -> Transfer;
        fn emit_row(
            index: u64,
            flags: u8,
            size: u64,
            packed: u64,
            dos: u32,
            method: u16,
            name: &[u8],
        ) -> u8;
        fn emit_data(bytes: &[u8]) -> u8;
        fn emit_precharged_data(bytes: &[u8]) -> u8;
        fn emit_work(bytes: u64) -> u8;
        fn emit_auxiliary(bytes: u64) -> u8;
        fn complete_entry(index: u64) -> Transfer;
    }
    extern "Rust" {
        fn run_optical(
            operation: u8,
            format: u8,
            length: u64,
            selected: u64,
            entries: u32,
            name_bytes: u32,
        ) -> u8;
        fn run_cab(
            operation: u8,
            length: u64,
            selected: u64,
            entries: u32,
            name_bytes: u32,
            volumes: &[u64],
            volume_names: &[u8],
        ) -> u8;
    }
}

struct Source {
    length: u64,
    position: u64,
    volumes: Vec<u64>,
    volume_names: Vec<Vec<u8>>,
}
impl Read for Source {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.position >= self.length || output.is_empty() {
            return Ok(0);
        }
        let count = (self.length - self.position).min(output.len().min(64 * 1024) as u64) as usize;
        let result = ffi::read_bytes(self.position, &mut output[..count]);
        if result.error != 0 {
            return Err(io::Error::other(wire_scan(result.error)));
        }
        let read =
            usize::try_from(result.value).map_err(|_| io::Error::other(ScanError::Invalid))?;
        if read > count {
            return Err(io::Error::other(ScanError::Invalid));
        }
        self.position += read as u64;
        Ok(read)
    }
}
impl Seek for Source {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        let at = match position {
            SeekFrom::Start(at) => i128::from(at),
            SeekFrom::Current(delta) => i128::from(self.position) + i128::from(delta),
            SeekFrom::End(delta) => i128::from(self.length) + i128::from(delta),
        };
        let at = u64::try_from(at).map_err(|_| io::Error::other(ScanError::Invalid))?;
        if at > self.length {
            return Err(io::Error::other(ScanError::Invalid));
        }
        self.position = at;
        Ok(at)
    }
}
impl ReadSource for Source {
    fn volume_lengths(&self) -> Option<&[u64]> {
        (!self.volumes.is_empty()).then_some(self.volumes.as_slice())
    }
    fn volume_names(&self) -> Option<&[Vec<u8>]> {
        (!self.volumes.is_empty()).then_some(self.volume_names.as_slice())
    }
}
fn wire_scan(code: u8) -> ScanError {
    match code {
        1 => ScanError::Io,
        3 => ScanError::Unsupported,
        4 => ScanError::Limit,
        5 => ScanError::Cancelled,
        9 => ScanError::MissingVolume,
        _ => ScanError::Invalid,
    }
}
fn scan_wire(error: ScanError) -> u8 {
    match error {
        ScanError::Io | ScanError::SourceChanged => 1,
        ScanError::Unsupported => 3,
        ScanError::Limit => 4,
        ScanError::Cancelled => 5,
        ScanError::MissingVolume => 9,
        _ => 2,
    }
}
fn decode_wire(error: DecodeError) -> u8 {
    match error {
        DecodeError::SourceIo => 1,
        DecodeError::Unsupported => 3,
        DecodeError::Limit => 4,
        DecodeError::Cancelled => 5,
        DecodeError::Output => 8,
        _ => 2,
    }
}
struct Output;
impl DecodedSink for Output {
    fn charge_auxiliary(&mut self, mut bytes: u64) -> Result<(), DecodeError> {
        while bytes != 0 {
            let take = bytes.min(64 * 1024);
            match ffi::emit_auxiliary(take) {
                0 => (),
                4 => return Err(DecodeError::Limit),
                5 => return Err(DecodeError::Cancelled),
                _ => return Err(DecodeError::Output),
            }
            bytes -= take;
        }
        Ok(())
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        for chunk in bytes.chunks(64 * 1024) {
            match ffi::emit_precharged_data(chunk) {
                0 => (),
                4 => return Err(DecodeError::Limit),
                5 => return Err(DecodeError::Cancelled),
                _ => return Err(DecodeError::Output),
            }
        }
        Ok(())
    }
    fn charge_skipped(&mut self, mut bytes: u64) -> Result<(), DecodeError> {
        while bytes != 0 {
            let take = bytes.min(64 * 1024);
            match ffi::emit_work(take) {
                0 => (),
                4 => return Err(DecodeError::Limit),
                5 => return Err(DecodeError::Cancelled),
                _ => return Err(DecodeError::Output),
            }
            bytes -= take;
        }
        Ok(())
    }
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        for chunk in bytes.chunks(64 * 1024) {
            match ffi::emit_data(chunk) {
                0 => (),
                4 => return Err(DecodeError::Limit),
                5 => return Err(DecodeError::Cancelled),
                _ => return Err(DecodeError::Output),
            }
        }
        Ok(())
    }
}
fn run(
    operation: u8,
    format: u8,
    length: u64,
    selected: u64,
    entries: u32,
    name_bytes: u32,
) -> Result<(), u8> {
    if !matches!(operation, 0 | 1 | 3) {
        return Err(3);
    }
    let format = match format {
        9 => Format::Iso,
        10 => Format::Udf,
        _ => return Err(3),
    };
    let codec = OpticalCodec { format };
    let control = Cancellation::default();
    let mut source = Source {
        length,
        position: 0,
        volumes: Vec::new(),
        volume_names: Vec::new(),
    };
    let limits = ScanLimits {
        entries: entries as usize,
        name_bytes: name_bytes as usize,
        ..Default::default()
    };
    let rows = codec
        .scan(&mut source, &control, limits)
        .map_err(scan_wire)?;
    if operation == 0 {
        for (index, row) in rows.iter().enumerate() {
            // Preserve the parser's namespace declaration through the wire.
            // Successfully decoded legacy bytes can themselves be valid UTF-8,
            // and must still participate in the host's whole-archive decision.
            let raw = format == Format::Iso
                && matches!(
                    row.name_encoding,
                    xdzip_codec_api::names::NameEncoding::UnmarkedUtf8
                        | xdzip_codec_api::names::NameEncoding::Legacy(_)
                        | xdzip_codec_api::names::NameEncoding::Unresolved
                );
            let flags = u8::from(row.directory)
                | (u8::from(row.link) << 2)
                | (u8::from(row.warning) << 3)
                | if raw { 0x10 } else { 0 };
            let error = ffi::emit_row(
                index as u64,
                flags,
                row.size,
                row.packed_size,
                row.modified_dos,
                row.method,
                &row.raw_name,
            );
            if error != 0 {
                return Err(error);
            }
        }
        return Ok(());
    }
    decode_rows(operation, selected, &codec, &mut source, &rows, &control)
}
fn decode_rows(
    operation: u8,
    selected: u64,
    codec: &dyn PayloadCodec,
    source: &mut Source,
    rows: &[Entry],
    control: &Cancellation,
) -> Result<(), u8> {
    let mut reader = codec
        .open_payload(source, rows, control)
        .map_err(decode_wire)?;
    let mut selected = selected;
    loop {
        let index = usize::try_from(selected).map_err(|_| 2)?;
        let row = rows.get(index).ok_or(2)?;
        reader
            .decode(row, &mut Output, control)
            .map_err(decode_wire)?;
        if operation != 3 {
            return Ok(());
        }
        let next = ffi::complete_entry(selected);
        if next.error != 0 {
            return Err(next.error);
        }
        if next.value <= selected || next.value >= rows.len() as u64 {
            return Err(2);
        }
        selected = next.value;
    }
}
fn run_optical(
    operation: u8,
    format: u8,
    length: u64,
    selected: u64,
    entries: u32,
    name_bytes: u32,
) -> u8 {
    // No unwind crosses the generated CXX boundary. OOM/abort still kills only
    // this isolated worker; the host rejects its incomplete staged output.
    match std::panic::catch_unwind(|| run(operation, format, length, selected, entries, name_bytes))
    {
        Ok(Ok(())) => 0,
        Ok(Err(code)) => code,
        Err(_) => 2,
    }
}
// Scalar controls mirror the existing bounded worker request on this boundary.
#[allow(clippy::too_many_arguments)]
fn run_cab(
    operation: u8,
    length: u64,
    selected: u64,
    entries: u32,
    name_bytes: u32,
    volumes: &[u64],
    volume_names: &[u8],
) -> u8 {
    // The slice arguments are borrowed only during this synchronous call.
    // As for optical images, a panic cannot cross the CXX boundary.
    match std::panic::catch_unwind(|| {
        cab::run(
            operation,
            length,
            selected,
            entries,
            name_bytes,
            volumes,
            volume_names,
        )
    }) {
        Ok(Ok(())) => 0,
        Ok(Err(code)) => code,
        Err(_) => 2,
    }
}
