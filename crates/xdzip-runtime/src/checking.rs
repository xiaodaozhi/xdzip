// crates/xdzip-runtime/src/checking.rs

//! Whole-archive verification has source authority only, never a destination or stage.
use crate::Archive;
use crate::extraction::auxiliary::AuxiliaryBudget;
use crate::extraction::{Control, MAX_FILE_BYTES, MAX_TASK_BYTES, Snapshot, Status, TaskKind};
use crate::source::Source;
use crate::task_interaction::{Interaction, decoding, emit};
use std::cell::Cell;
use std::io::{self, Read, Seek, SeekFrom};
use std::sync::Arc;
use std::time::{Duration, Instant};
use xdzip_codec_api::payload::{DecodeError, DecodedSink};
use xdzip_codec_api::{Cancellation, Entry, Format};

pub(crate) struct Job {
    source: Source,
    password: Option<crate::extraction::Password>,
    rows: Arc<Vec<Entry>>,
    format: Format,
    session: u64,
}
impl Job {
    pub fn retained_bytes(&self) -> usize {
        self.source.retained_bytes()
            + self.password.as_ref().map_or(0, |p| p.expose().len())
            + self
                .rows
                .iter()
                .map(|entry| std::mem::size_of::<Entry>() + entry.name_capacity())
                .sum::<usize>()
    }
}
impl Archive {
    /// Freeze the existing input identity; checking does not resolve output paths or selectors.
    pub(crate) fn check_job(&self, cancel: &Cancellation) -> Result<Job, &'static str> {
        cancel.check().map_err(|_| "cancelled")?;
        Ok(Job {
            source: self.source.fork().map_err(|_| "source_changed")?,
            password: self.password.clone(),
            rows: Arc::clone(&self.rows),
            format: self.format,
            session: self.key.session.get(),
        })
    }
}
pub(crate) fn initial(job: &Job, id: u64) -> Snapshot {
    Snapshot {
        kind: TaskKind::Check,
        source: job.source.display_path().to_string_lossy().into_owned(),
        format: job.format.label(),
        creating: false,
        // TAR/GZIP scanning checks the entire stream before reporting verified records.
        total_known: !matches!(job.format, Format::Tar | Format::TarGzip)
            && job.rows.iter().all(|row| row.directory || row.size_known),
        id,
        session: job.session,
        status: Status::Queued,
        target: String::new(),
        open_path: String::new(),
        current: String::new(),
        files: job.rows.iter().filter(|row| !row.directory).count() as u64,
        declared_bytes: job
            .rows
            .iter()
            .filter(|row| !row.directory && row.size_known)
            .fold(0u64, |n, row| n.saturating_add(row.size)),
        decoded_bytes: 0,
        read_bytes: 0,
        verified_files: 0,
        verified_bytes: 0,
        written_bytes: 0,
        committed_bytes: 0,
        committed_files: 0,
        skipped_files: 0,
        created_directories: 0,
        elapsed_ms: 0,
        error: String::new(),
        cleanup_warning: false,
        outcome_uncertain: false,
        leftovers: 0,
        prompt: None,
        password_prompt: None,
        details: Default::default(),
        omitted_details: 0,
        external_reader: false,
        output_parts: 0,
    }
}
struct Counted<'a> {
    source: &'a mut Source,
    bytes: &'a Cell<u64>,
    progress: &'a mut dyn FnMut(u64),
}
impl Read for Counted<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let n = self.source.read(bytes)?;
        let next = self
            .bytes
            .get()
            .checked_add(n as u64)
            .ok_or_else(|| io::Error::other("resource_limit"))?;
        self.bytes.set(next);
        (self.progress)(next);
        Ok(n)
    }
}
impl xdzip_codec_api::ReadSource for Counted<'_> {
    fn volume_lengths(&self) -> Option<&[u64]> {
        xdzip_codec_api::ReadSource::volume_lengths(self.source)
    }
    fn volume_names(&self) -> Option<&[Vec<u8>]> {
        xdzip_codec_api::ReadSource::volume_names(self.source)
    }
}
impl Seek for Counted<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.source.seek(position)
    }
}
struct Discard<'a> {
    stats: &'a mut Snapshot,
    read: &'a Cell<u64>,
    file_bytes: u64,
    declared: u64,
    precharged: &'a mut u64,
    output_bytes: &'a mut u64,
    auxiliary: &'a mut AuxiliaryBudget,
    control: &'a Cancellation,
    start: Instant,
    last: &'a mut Instant,
    publish: &'a mut dyn FnMut(Snapshot),
}
impl DecodedSink for Discard<'_> {
    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.control.check().map_err(|_| DecodeError::Cancelled)?;
        self.auxiliary.charge(bytes)
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.control.check().map_err(|_| DecodeError::Cancelled)?;
        let next = self
            .stats
            .decoded_bytes
            .checked_add(bytes)
            .filter(|n| *n <= MAX_TASK_BYTES)
            .ok_or(DecodeError::Limit)?;
        let credit = self
            .precharged
            .checked_add(bytes)
            .filter(|n| *n <= MAX_TASK_BYTES)
            .ok_or(DecodeError::Limit)?;
        self.stats.decoded_bytes = next;
        *self.precharged = credit;
        Ok(())
    }
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.write(bytes, false)
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.write(bytes, true)
    }
}
impl Discard<'_> {
    fn write(&mut self, bytes: &[u8], precharged: bool) -> Result<(), DecodeError> {
        self.control.check().map_err(|_| DecodeError::Cancelled)?;
        let count = bytes.len() as u64;
        if precharged && *self.precharged < count {
            return Err(DecodeError::Integrity);
        }
        let file_bytes = self
            .file_bytes
            .checked_add(count)
            .ok_or(DecodeError::Limit)?;
        if file_bytes > MAX_FILE_BYTES {
            return Err(DecodeError::Limit);
        }
        if file_bytes > self.declared {
            return Err(DecodeError::Integrity);
        }
        let output = self
            .output_bytes
            .checked_add(count)
            .filter(|n| *n <= MAX_TASK_BYTES)
            .ok_or(DecodeError::Limit)?;
        let decoded = if precharged {
            self.stats.decoded_bytes
        } else {
            self.stats
                .decoded_bytes
                .checked_add(count)
                .filter(|n| *n <= MAX_TASK_BYTES)
                .ok_or(DecodeError::Limit)?
        };
        self.file_bytes = file_bytes;
        *self.output_bytes = output;
        self.stats.decoded_bytes = decoded;
        if precharged {
            *self.precharged -= count;
        }
        self.stats.read_bytes = self.read.get();
        if self.last.elapsed() >= Duration::from_millis(100) {
            emit(self.stats, self.start, self.publish);
            *self.last = Instant::now();
        }
        Ok(())
    }
}
pub(crate) fn run(job: Job, id: u64, control: &Control, publish: &mut dyn FnMut(Snapshot)) {
    let start = Instant::now();
    let mut stats = initial(&job, id);
    stats.status = Status::Preparing;
    emit(&mut stats, start, publish);
    let read = Cell::new(0u64);
    let result = (|| -> Result<(), &'static str> {
        control.cancel.check().map_err(|_| "cancelled")?;
        let mut declared = 0u64;
        for entry in job.rows.iter() {
            control.cancel.check().map_err(|_| "cancelled")?;
            if !entry.size_known {
                return Err("unsupported_feature");
            }
            if entry.size > MAX_FILE_BYTES {
                return Err("file_quota");
            }
            if !entry.directory {
                declared = declared
                    .checked_add(entry.size)
                    .filter(|n| *n <= MAX_TASK_BYTES)
                    .ok_or("task_quota")?;
            }
        }
        stats.declared_bytes = declared;
        job.source.validate().map_err(|_| "source_changed")?;
        let mut source = job.source.fork().map_err(|_| "source_changed")?;
        stats.status = Status::Running;
        emit(&mut stats, start, publish);
        let checked = if !matches!(job.format, Format::Tar | Format::TarGzip) {
            let mut ignore_progress = |_: u64| {};
            let mut input = Counted {
                source: &mut source,
                bytes: &read,
                progress: &mut ignore_progress,
            };
            let mut decoder = crate::formats::codec(job.format)
                .open_payload_with_password(
                    &mut input,
                    &job.rows,
                    &control.cancel,
                    job.password
                        .as_ref()
                        .map(crate::extraction::Password::expose),
                )
                .map_err(decoding)?;
            let mut interaction = Interaction::with_password(job.password.clone());
            let mut last = Instant::now();
            let mut precharged = 0;
            let mut output_bytes = 0;
            let mut auxiliary = AuxiliaryBudget::default();
            for entry in job.rows.iter() {
                control.cancel.check().map_err(|_| "cancelled")?;
                stats.current = entry.display_name.clone();
                emit(&mut stats, start, publish);
                interaction.decode(
                    entry,
                    control,
                    &mut stats,
                    start,
                    publish,
                    |password, stats, publish| {
                        let mut sink = Discard {
                            stats,
                            read: &read,
                            file_bytes: 0,
                            declared: entry.size,
                            precharged: &mut precharged,
                            output_bytes: &mut output_bytes,
                            auxiliary: &mut auxiliary,
                            control: &control.cancel,
                            start,
                            last: &mut last,
                            publish,
                        };
                        let result = decoder
                            .decode_with_password(entry, &mut sink, &control.cancel, password)
                            .map_err(decoding);
                        sink.stats.read_bytes = read.get();
                        result
                    },
                )?;
                stats.read_bytes = read.get();
                if !entry.directory {
                    stats.verified_files += 1;
                    stats.verified_bytes += entry.size;
                }
                stats.record(&entry.display_name, entry.size, "completed", "");
            }
            Ok(())
        } else {
            // A full scan reads payloads, both TAR end blocks and the GZIP trailer.
            // A second payload decoder pass would read the content twice and would not
            // itself consume the final GZIP trailer after the last record.
            let mut last = Instant::now();
            let mut progress = |bytes| {
                stats.read_bytes = bytes;
                if last.elapsed() >= Duration::from_millis(100) {
                    emit(&mut stats, start, publish);
                    last = Instant::now();
                }
            };
            let mut input = Counted {
                source: &mut source,
                bytes: &read,
                progress: &mut progress,
            };
            let scanned = crate::formats::codec(job.format)
                .scan(
                    &mut input,
                    &control.cancel,
                    crate::formats::limits(job.format),
                )
                .map_err(|e| match e {
                    xdzip_codec_api::ScanError::Cancelled => "cancelled",
                    xdzip_codec_api::ScanError::Limit => "resource_limit",
                    xdzip_codec_api::ScanError::Io => "source_io",
                    xdzip_codec_api::ScanError::Unsupported => "unsupported_feature",
                    _ => "integrity_error",
                })?;
            if scanned.as_slice() != job.rows.as_slice() {
                return Err("integrity_error");
            }
            for entry in scanned {
                if entry.warning {
                    stats.current = entry.display_name;
                    return Err("unsupported_entry");
                }
                if !entry.directory {
                    stats.verified_files += 1;
                    stats.verified_bytes += entry.size;
                }
                stats.record(&entry.display_name, entry.size, "completed", "");
            }
            stats.decoded_bytes = stats.verified_bytes;
            Ok(())
        };
        // Check both the reader's cached handle and every originally frozen path/volume.
        source.validate().map_err(|_| "source_changed")?;
        job.source.validate().map_err(|_| "source_changed")?;
        checked?;
        control.complete()?;
        Ok(())
    })();
    let result = if result != Err("cancelled") && job.source.validate().is_err() {
        Err("source_changed")
    } else {
        result
    };
    stats.read_bytes = read.get();
    stats.password_prompt = None;
    stats.status = match result {
        Ok(()) => {
            stats.total_known = true;
            Status::Succeeded
        }
        Err("cancelled") => Status::Cancelled,
        Err(_) => Status::Failed,
    };
    stats.error = result.err().unwrap_or_default().into();
    if result.is_err() {
        let current = stats.current.clone();
        stats.record(
            &current,
            0,
            "unfinished",
            result.err().unwrap_or("internal_error"),
        );
    }
    control.mark_terminal();
    emit(&mut stats, start, publish);
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod precharged_tests {
    use super::*;

    fn snapshot() -> Snapshot {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/extract/stored.zip");
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        initial(&archive.check_job(&Cancellation::default()).unwrap(), 1)
    }

    #[test]
    fn auxiliary_check_work_crosses_records_without_changing_verified_or_decoded_bytes() {
        use crate::extraction::auxiliary::MAX_AUXILIARY_BYTES;
        let mut stats = snapshot();
        let read = Cell::new(0);
        let control = Cancellation::default();
        let mut credit = 0;
        let mut output_bytes = 0;
        let mut auxiliary = AuxiliaryBudget::default();
        auxiliary.charge(MAX_AUXILIARY_BYTES - 2).unwrap();
        let mut last = Instant::now();
        let mut publish = |_| {};
        for _ in 0..2 {
            let mut sink = Discard {
                stats: &mut stats,
                read: &read,
                file_bytes: 0,
                declared: 87,
                precharged: &mut credit,
                output_bytes: &mut output_bytes,
                auxiliary: &mut auxiliary,
                control: &control,
                start: Instant::now(),
                last: &mut last,
                publish: &mut publish,
            };
            sink.charge_auxiliary(1).unwrap();
            assert_eq!(
                sink.write_precharged_chunk(b"x"),
                Err(DecodeError::Integrity)
            );
        }
        let mut sink = Discard {
            stats: &mut stats,
            read: &read,
            file_bytes: 0,
            declared: 87,
            precharged: &mut credit,
            output_bytes: &mut output_bytes,
            auxiliary: &mut auxiliary,
            control: &control,
            start: Instant::now(),
            last: &mut last,
            publish: &mut publish,
        };
        assert_eq!(sink.charge_auxiliary(1), Err(DecodeError::Limit));
        assert_eq!(sink.auxiliary.used(), MAX_AUXILIARY_BYTES);
        assert_eq!(
            (
                sink.stats.decoded_bytes,
                sink.stats.verified_bytes,
                sink.stats.written_bytes,
                *sink.precharged,
                *sink.output_bytes
            ),
            (0, 0, 0, 0, 0)
        );
        control.cancel();
        assert_eq!(sink.charge_auxiliary(1), Err(DecodeError::Cancelled));
    }

    #[test]
    fn precharged_discard_credit_crosses_records_without_double_work_or_replay() {
        let mut stats = snapshot();
        let read = Cell::new(0);
        let control = Cancellation::default();
        let mut credit = 0;
        let mut output_bytes = 0;
        let mut auxiliary = AuxiliaryBudget::default();
        let mut last = Instant::now();
        let mut publish = |_| {};
        for bytes in [b"ab", b"cd"] {
            let mut sink = Discard {
                stats: &mut stats,
                read: &read,
                file_bytes: 0,
                declared: 2,
                precharged: &mut credit,
                output_bytes: &mut output_bytes,
                auxiliary: &mut auxiliary,
                control: &control,
                start: Instant::now(),
                last: &mut last,
                publish: &mut publish,
            };
            if sink.stats.decoded_bytes == 0 {
                sink.charge_skipped(4).unwrap();
            }
            sink.write_precharged_chunk(bytes).unwrap();
        }
        assert_eq!((stats.decoded_bytes, credit, output_bytes), (4, 0, 4));
        let mut sink = Discard {
            stats: &mut stats,
            read: &read,
            file_bytes: 0,
            declared: 2,
            precharged: &mut credit,
            output_bytes: &mut output_bytes,
            auxiliary: &mut auxiliary,
            control: &control,
            start: Instant::now(),
            last: &mut last,
            publish: &mut publish,
        };
        assert_eq!(
            sink.write_precharged_chunk(b"x"),
            Err(DecodeError::Integrity)
        );
        assert_eq!(sink.file_bytes, 0);
        assert_eq!(
            (sink.stats.written_bytes, sink.stats.committed_bytes),
            (0, 0)
        );
    }

    #[test]
    fn precharged_discard_preserves_work_declared_file_output_and_cancel_limits() {
        let mut stats = snapshot();
        let read = Cell::new(0);
        let control = Cancellation::default();
        let mut credit = 0;
        let mut output_bytes = 0;
        let mut auxiliary = AuxiliaryBudget::default();
        let mut last = Instant::now();
        let mut publish = |_| {};
        let mut sink = Discard {
            stats: &mut stats,
            read: &read,
            file_bytes: 0,
            declared: 1,
            precharged: &mut credit,
            output_bytes: &mut output_bytes,
            auxiliary: &mut auxiliary,
            control: &control,
            start: Instant::now(),
            last: &mut last,
            publish: &mut publish,
        };
        sink.charge_skipped(MAX_TASK_BYTES).unwrap();
        assert_eq!(sink.charge_skipped(1), Err(DecodeError::Limit));
        assert_eq!(*sink.precharged, MAX_TASK_BYTES);
        assert_eq!(
            sink.write_precharged_chunk(b"ab"),
            Err(DecodeError::Integrity)
        );
        sink.declared = u64::MAX;
        sink.file_bytes = MAX_FILE_BYTES;
        assert_eq!(sink.write_precharged_chunk(b"x"), Err(DecodeError::Limit));
        sink.file_bytes = 0;
        *sink.output_bytes = MAX_TASK_BYTES;
        assert_eq!(sink.write_precharged_chunk(b"x"), Err(DecodeError::Limit));
        assert_eq!(*sink.precharged, MAX_TASK_BYTES);
        *sink.output_bytes = 0;
        sink.write_precharged_chunk(b"x").unwrap();
        assert_eq!(sink.stats.decoded_bytes, MAX_TASK_BYTES);
        assert_eq!(sink.write_chunk(b"x"), Err(DecodeError::Limit));
        control.cancel();
        assert_eq!(
            sink.write_precharged_chunk(b"x"),
            Err(DecodeError::Cancelled)
        );
        assert_eq!(*sink.precharged, MAX_TASK_BYTES - 1);
    }
}
