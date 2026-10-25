//! 7z admission on the worker's shared borrowed I/O, without filesystem access.
use super::{Source, decode_rows_with_password, ffi, scan_wire};
use std::io::{self, Read, Seek, SeekFrom, Write};
use xdzip_codec_7z::SevenZipCodec;
use xdzip_codec_api::encode::{
    CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeOutput, EncodeProgress,
    EncodeSettings, RewriteCodec, RewriteInput,
};
use xdzip_codec_api::{Cancellation, MetadataCodec, ScanLimits};

const LIMIT: u64 = 20 << 30;
fn stage_fault(error: EncodeError) -> io::Error {
    match error {
        EncodeError::Limit => io::Error::other(xdzip_codec_api::ScanError::Limit),
        EncodeError::Cancelled => io::Error::other(xdzip_codec_api::ScanError::Cancelled),
        _ => io::Error::other("staging_failed"),
    }
}

fn encode_wire(error: EncodeError) -> u8 {
    match error {
        EncodeError::Source => 1,
        EncodeError::Output => 8,
        EncodeError::Invalid => 2,
        EncodeError::Unsupported => 3,
        EncodeError::Limit => 4,
        EncodeError::Cancelled => 5,
    }
}

struct Input {
    entries: Vec<EncodeEntry>,
    physical: Vec<u32>,
}
impl EncodeInput for Input {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        let entry = self.entries.get(index).ok_or(EncodeError::Invalid)?;
        if entry.directory {
            return Err(EncodeError::Invalid);
        }
        Ok(Box::new(InputReader {
            index: *self.physical.get(index).ok_or(EncodeError::Invalid)?,
        }))
    }
}
struct InputReader {
    index: u32,
}
impl Read for InputReader {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        let take = bytes.len().min(64 * 1024);
        let result = ffi::read_input_bytes(self.index, &mut bytes[..take]);
        if result.error != 0 {
            return Err(io::Error::other(super::wire_scan(result.error)));
        }
        usize::try_from(result.value)
            .ok()
            .filter(|count| *count <= take)
            .ok_or_else(|| io::Error::other("invalid_input_count"))
    }
}
#[derive(Default)]
struct Stage {
    position: u64,
    length: u64,
    written: u64,
}
#[derive(Default)]
struct Reporter {
    name: Vec<u8>,
    total: u64,
    declared: u64,
    error: u8,
}
impl Reporter {
    fn report(&mut self, progress: EncodeProgress<'_>, control: &Cancellation) {
        if self.error != 0 {
            return;
        }
        if progress.input_bytes < self.total || progress.input_bytes > self.declared {
            self.error = 2;
        } else if progress.name.as_bytes() != self.name
            || progress.input_bytes - self.total >= 64 * 1024
            || progress.input_bytes == self.declared
        {
            self.error =
                ffi::emit_creation_progress(progress.input_bytes, progress.name.as_bytes());
            self.total = progress.input_bytes;
            self.name.clear();
            self.name.extend_from_slice(progress.name.as_bytes());
        }
        if self.error != 0 {
            control.cancel();
        }
    }
    fn finish(self, result: Result<(), EncodeError>) -> Result<(), EncodeError> {
        match self.error {
            0 => result,
            4 => Err(EncodeError::Limit),
            5 => Err(EncodeError::Cancelled),
            2 => Err(EncodeError::Invalid),
            _ => Err(EncodeError::Output),
        }
    }
}
impl Stage {
    fn patch(&mut self, offset: u64, bytes: &[u8]) -> Result<(), EncodeError> {
        let end = offset
            .checked_add(bytes.len() as u64)
            .ok_or(EncodeError::Limit)?;
        let written = self
            .written
            .checked_add(bytes.len() as u64)
            .ok_or(EncodeError::Limit)?;
        if end > LIMIT || written > LIMIT {
            return Err(EncodeError::Limit);
        }
        for (index, block) in bytes.chunks(64 * 1024).enumerate() {
            let at = offset + (index * 64 * 1024) as u64;
            let error = ffi::write_archive_at(at, block);
            if error != 0 {
                return Err(match error {
                    4 => EncodeError::Limit,
                    5 => EncodeError::Cancelled,
                    _ => EncodeError::Output,
                });
            }
            self.written += block.len() as u64;
            self.length = self.length.max(at + block.len() as u64);
        }
        Ok(())
    }
}
impl Write for Stage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.patch(self.position, bytes).map_err(stage_fault)?;
        self.position += bytes.len() as u64;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Seek for Stage {
    fn seek(&mut self, seek: SeekFrom) -> io::Result<u64> {
        let at = match seek {
            SeekFrom::Start(at) => i128::from(at),
            SeekFrom::Current(delta) => i128::from(self.position) + i128::from(delta),
            SeekFrom::End(delta) => i128::from(self.length) + i128::from(delta),
        };
        self.position = u64::try_from(at)
            .ok()
            .filter(|at| *at <= LIMIT)
            .ok_or_else(|| stage_fault(EncodeError::Limit))?;
        Ok(self.position)
    }
}
impl EncodeOutput for Stage {
    fn write_at(&mut self, offset: u64, bytes: &[u8]) -> Result<(), EncodeError> {
        self.patch(offset, bytes)
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    operation: u8,
    length: u64,
    selected: u64,
    entries: u32,
    name_bytes: u32,
    volumes: &[u64],
    password: Option<&str>,
    records: &[ffi::CreationRow],
    options: &ffi::CreationOptions,
) -> Result<(), u8> {
    if !matches!(operation, 0..=3) {
        return Err(3);
    }
    if entries > 100_000
        || name_bytes > 4096
        || volumes.len() > 1024
        || !volumes.is_empty()
            && volumes.iter().try_fold(0u64, |n, v| n.checked_add(*v)) != Some(length)
        || password.is_some_and(|p| p.len() > 4096)
    {
        return Err(4);
    }
    let control = Cancellation::default();
    let mut source = Source {
        length,
        position: 0,
        volumes: volumes.to_vec(),
        volume_names: Vec::new(),
    };
    let codec = SevenZipCodec;
    if operation == 2 {
        return create(
            &mut source,
            length != 0,
            records,
            options,
            password,
            &control,
        )
        .map_err(encode_wire);
    }
    if !records.is_empty() {
        return Err(2);
    }
    let rows = codec
        .scan_with_password(
            &mut source,
            &control,
            ScanLimits {
                entries: entries as usize,
                name_bytes: name_bytes as usize,
                ..Default::default()
            },
            password,
        )
        .map_err(scan_wire)?;
    if operation == 0 {
        for (index, row) in rows.iter().enumerate() {
            let flags = u8::from(row.directory)
                | (u8::from(row.encrypted) << 1)
                | (u8::from(row.link) << 2)
                | (u8::from(row.warning) << 3);
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
    decode_rows_with_password(
        operation,
        selected,
        &codec,
        &mut source,
        &rows,
        &control,
        password,
    )
}

fn create(
    source: &mut Source,
    original: bool,
    records: &[ffi::CreationRow],
    options: &ffi::CreationOptions,
    password: Option<&str>,
    control: &Cancellation,
) -> Result<(), EncodeError> {
    if records.len() > 100_000 {
        return Err(EncodeError::Limit);
    }
    let settings = EncodeSettings {
        method: if options.copy { "copy" } else { "lzma2" },
        level: options.level,
        solid: options.solid,
        encrypt_headers: options.encrypt_headers,
        password,
    };
    let mut input = Input {
        entries: Vec::new(),
        physical: Vec::new(),
    };
    let mut total_names = 0usize;
    for (index, row) in records.iter().enumerate() {
        total_names = total_names
            .checked_add(row.name.len())
            .ok_or(EncodeError::Limit)?;
        if row.name.len() > 4096 || total_names > 16 << 20 {
            return Err(EncodeError::Limit);
        }
        input.entries.push(EncodeEntry {
            name: row.name.clone(),
            directory: row.directory,
            size: row.size,
            modified_unix: row.modified_unix,
        });
        input.physical.push(index as u32);
    }
    let mut output = Stage::default();
    let mut reporter = Reporter {
        declared: input
            .entries
            .iter()
            .try_fold(0u64, |sum, row| sum.checked_add(row.size))
            .ok_or(EncodeError::Limit)?,
        ..Default::default()
    };
    let codec = SevenZipCodec;
    if !original {
        if records.iter().any(|row| row.old_index != u64::MAX) {
            return Err(EncodeError::Invalid);
        }
        let result = codec.encode(&input, &mut output, settings, control, &mut |progress| {
            reporter.report(progress, control)
        });
        return reporter.finish(result);
    }
    if source.volumes.len() > 1 {
        return Err(EncodeError::Unsupported);
    }
    let rows = codec
        .scan(source, control, ScanLimits::default())
        .map_err(|error| match error {
            xdzip_codec_api::ScanError::Limit => EncodeError::Limit,
            xdzip_codec_api::ScanError::Cancelled => EncodeError::Cancelled,
            xdzip_codec_api::ScanError::Io | xdzip_codec_api::ScanError::SourceChanged => {
                EncodeError::Source
            }
            xdzip_codec_api::ScanError::Invalid => EncodeError::Invalid,
            _ => EncodeError::Unsupported,
        })?;
    let mut keep = vec![false; rows.len()];
    let mut renames = vec![None; rows.len()];
    let mut additions = Input {
        entries: Vec::new(),
        physical: Vec::new(),
    };
    let mut last_old = None;
    for (index, record) in records.iter().enumerate() {
        if record.old_index == u64::MAX {
            additions.entries.push(input.entries[index].clone());
            additions.physical.push(index as u32);
            continue;
        }
        let old = usize::try_from(record.old_index).map_err(|_| EncodeError::Invalid)?;
        let row = rows.get(old).ok_or(EncodeError::Invalid)?;
        if !additions.entries.is_empty()
            || last_old.is_some_and(|previous| old <= previous)
            || row.directory != record.directory
            || row.size != record.size
        {
            return Err(EncodeError::Invalid);
        }
        last_old = Some(old);
        keep[old] = true;
        renames[old] = Some(record.name.clone());
    }
    let result = codec.rewrite(
        source,
        RewriteInput {
            keep: &keep,
            renames: &renames,
            additions: &additions,
            settings,
        },
        &mut output,
        control,
        &mut |progress| reporter.report(progress, control),
    );
    reporter.finish(result)
}
