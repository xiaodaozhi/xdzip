use super::plan::Job;
use super::{ConflictPolicy, Control, MAX_FILE_BYTES, MAX_TASK_BYTES, Prompt, Snapshot, Status};
use std::io::Write;
use std::path::Path;
use std::time::{Duration, Instant};
use xdzip_codec_api::payload::{DecodeError, DecodedSink};

use xdzip_domain::path::PortablePath;
use xdzip_io::{Grant, OutputError};

fn output(error: OutputError) -> &'static str {
    match error {
        OutputError::Io => "output_io",
        OutputError::NoSpace => "disk_full",
        OutputError::UnsafeTarget => "unsafe_target",
        OutputError::TargetChanged => "target_changed",
        OutputError::Conflict => "output_conflict",
        OutputError::Unsupported => "unsupported_filesystem",
        OutputError::Limit => "resource_limit",
    }
}
fn decoding(error: DecodeError) -> &'static str {
    match error {
        DecodeError::SourceIo => "source_io",
        DecodeError::Integrity => "integrity_error",
        DecodeError::Unsupported => "unsupported_feature",
        DecodeError::Limit => "resource_limit",
        DecodeError::Cancelled => "cancelled",
        DecodeError::Output => "output_io",
    }
}
pub(crate) fn initial(job: &Job, id: u64) -> Snapshot {
    Snapshot {
        creating: false,
        total_known: true,
        id,
        session: job.session,
        status: Status::Queued,
        target: job.options.target.clone(),
        current: String::new(),
        files: job
            .planned
            .iter()
            .filter(|p| !job.rows[p.index].directory)
            .count() as u64,
        declared_bytes: job
            .planned
            .iter()
            .filter(|p| !job.rows[p.index].directory)
            .map(|p| job.rows[p.index].size)
            .sum(),
        decoded_bytes: 0,
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
        details: Default::default(),
        omitted_details: 0,
        external_reader: false,
        output_parts: 0,
    }
}
fn emit(stats: &mut Snapshot, start: Instant, publish: &mut dyn FnMut(Snapshot)) {
    stats.elapsed_ms = start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    publish(stats.clone());
}
/// Finish cleanup and publish exactly one terminal snapshot even after a decode/output failure.
pub(crate) fn run(job: Job, id: u64, control: &Control, publish: &mut dyn FnMut(Snapshot)) {
    let start = Instant::now();
    let mut stats = initial(&job, id);
    stats.status = Status::Preparing;
    emit(&mut stats, start, publish);
    let mut workspace = None;
    let mut grant = None;
    let mut pending_file = None;
    let result = (|| -> Result<(), &'static str> {
        control.cancel.check().map_err(|_| "cancelled")?;
        job.source.validate().map_err(|_| "source_changed")?;
        let source_id = job.source.identity().map_err(|_| "source_changed")?;
        let root = Grant::open(Path::new(&job.options.target), source_id).map_err(output)?;
        grant = Some(root.clone());
        stats.leftovers = root.leftover_count().map_err(output)? as u64;
        let target = if job.options.create_folder {
            let name = PortablePath::parse(job.source.name()).ok_or("unsafe_folder_name")?;
            if name.parts().len() != 1 {
                return Err("unsafe_folder_name");
            }
            root.subdirectory(&name).map_err(output)?
        } else {
            root
        };
        stats.target = target.display_path().to_string_lossy().into_owned();
        workspace = Some(target.workspace(id).map_err(output)?);
        let workspace = workspace.as_mut().expect("workspace just created");
        let mut source = job.source.fork().map_err(|_| "source_changed")?;
        let opened = crate::formats::codec(job.format).open_payload(
            source.file(),
            &job.rows,
            &control.cancel,
        );
        job.source.validate().map_err(|_| "source_changed")?;
        let mut decoder = opened.map_err(decoding)?;
        let has_links = job.planned.iter().any(|p| p.link_target.is_some());
        if has_links {
            // Renaming/skipping a dependency would change the frozen link graph. Refuse
            // occupied file/link leaves before writing, rather than producing a broken bundle.
            for p in job.planned.iter().filter(|p| !job.rows[p.index].directory) {
                if target.existing(&p.path).map_err(output)?.is_some() {
                    return Err("output_conflict");
                }
            }
        }
        let mut policy = job.options.conflict;
        let mut prompt_id = 0u64;
        for planned in &job.planned {
            control.cancel.check().map_err(|_| "cancelled")?;
            job.source.validate().map_err(|_| "source_changed")?;
            let entry = &job.rows[planned.index];
            stats.current = entry.display_name.clone();
            stats.status = Status::Running;
            stats.prompt = None;
            if entry.directory {
                // Validate even empty directory local headers/CRC before creating a visible directory.
                let mut sink = Meter {
                    writer: std::io::sink(),
                    stats: &mut stats,
                    control,
                    file_bytes: 0,
                    error: None,
                    start,
                    last: Instant::now(),
                    publish,
                };
                decoder
                    .decode(entry, &mut sink, &control.cancel)
                    .map_err(decoding)?;
                job.source.validate().map_err(|_| "source_changed")?;
                target.ensure_directory(&planned.path).map_err(output)?;
                continue;
            }
            if let Some(link_target) = &planned.link_target {
                pending_file = Some(entry.size);
                // Payload was fully decoded and CRC checked before output admission. Source
                // identity is rechecked above; no untrusted link is ever used as a write path.
                control.begin_commit()?;
                let committed = workspace
                    .commit_symlink(
                        &planned.path,
                        &PortablePath::parse(link_target).ok_or("unsupported_link")?,
                    )
                    .map_err(output);
                let cancelled = control.end_commit(false);
                if let Ok(receipt) = &committed {
                    stats.committed_files += 1;
                    stats.decoded_bytes += entry.size;
                    stats.written_bytes += entry.size;
                    stats.committed_bytes += entry.size;
                    stats.cleanup_warning |= receipt.cleanup_warning;
                    stats.record(&planned.path.display(), entry.size, "completed", "");
                    pending_file = None;
                }
                committed?;
                cancelled?;
                emit(&mut stats, start, publish);
                continue;
            }
            pending_file = Some(entry.size);
            let mut path = planned.path.clone();
            let mut overwrite = None;
            if let Some(existing) = target.existing(&path).map_err(output)? {
                if has_links {
                    return Err("output_conflict");
                }
                let mut decision = policy;
                if decision == ConflictPolicy::Ask {
                    prompt_id = prompt_id.checked_add(1).ok_or("resource_limit")?;
                    control.arm(prompt_id);
                    stats.status = Status::Waiting;
                    stats.prompt = Some(Prompt {
                        id: prompt_id,
                        name: entry.display_name.clone(),
                        existing_bytes: existing.version.length,
                        incoming_bytes: entry.size,
                        existing_modified: existing
                            .version
                            .modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .ok()
                            .map(|d| d.as_secs()),
                        incoming_modified_dos: entry.modified_dos,
                        overwrite_allowed: !existing.source_protected,
                    });
                    emit(&mut stats, start, publish);
                    let (answer, all) = control.wait()?;
                    decision = answer;
                    if all {
                        policy = answer;
                    }
                    stats.prompt = None;
                    stats.status = Status::Running;
                }
                match decision {
                    ConflictPolicy::Skip => {
                        stats.skipped_files += 1;
                        stats.record(&entry.display_name, entry.size, "skipped", "");
                        pending_file = None;
                        emit(&mut stats, start, publish);
                        continue;
                    }
                    ConflictPolicy::Rename => path = fresh_name(&target, &path, &control.cancel)?,
                    ConflictPolicy::Overwrite if existing.source_protected => {
                        return Err("protected_source");
                    }
                    ConflictPolicy::Overwrite => overwrite = Some(existing),
                    ConflictPolicy::Ask => return Err("invalid_decision"),
                }
            }
            stats.current = path.display();
            emit(&mut stats, start, publish);
            let mut stage = workspace.stage().map_err(output)?;
            let mut meter = Meter {
                writer: &mut stage,
                stats: &mut stats,
                control,
                file_bytes: 0,
                error: None,
                start,
                last: Instant::now(),
                publish,
            };
            let decoded = decoder.decode(entry, &mut meter, &control.cancel);
            let actual = meter.file_bytes;
            let write_error = meter.error;
            // Source mutation takes precedence over a derivative CRC/header failure.
            job.source.validate().map_err(|_| "source_changed")?;
            decoded.map_err(|e| write_error.unwrap_or_else(|| decoding(e)))?;
            if actual != entry.size {
                return Err("integrity_error");
            }
            if entry.unix_mode != 0 {
                stage.restore_mode(entry.unix_mode).map_err(output)?;
            }
            control.begin_commit()?;
            stats.status = Status::Committing;
            emit(&mut stats, start, publish);
            let committed = job
                .source
                .validate()
                .map_err(|_| "source_changed")
                .and_then(|_| {
                    workspace
                        .commit(stage, &path, overwrite.as_ref())
                        .map_err(output)
                });
            let cancelled = control.end_commit(false);
            if let Ok(receipt) = &committed {
                stats.committed_files += 1;
                stats.committed_bytes += actual;
                stats.cleanup_warning |= receipt.cleanup_warning;
                stats.record(&path.display(), actual, "completed", "");
                pending_file = None;
            }
            committed?;
            cancelled?;
        }
        control.complete()
    })();
    stats.prompt = None;
    stats.status = if control.cancel.check().is_err() {
        Status::Cancelling
    } else {
        Status::Cleaning
    };
    emit(&mut stats, start, publish);
    if let Some(root) = grant {
        stats.created_directories = root.created_directories();
    }
    if let Some(workspace) = workspace {
        stats.cleanup_warning |= workspace.cleanup().is_err();
    }
    stats.status = match result {
        Ok(()) => Status::Succeeded,
        Err("cancelled") => Status::Cancelled,
        Err(_) => Status::Failed,
    };
    stats.error = result.err().unwrap_or_default().into();
    if let Some(size) = pending_file {
        let name = stats.current.clone();
        stats.record(
            &name,
            size,
            "unfinished",
            result.err().unwrap_or("internal_error"),
        );
    }
    control.mark_terminal();
    emit(&mut stats, start, publish);
}
fn fresh_name(
    target: &Grant,
    path: &PortablePath,
    cancel: &xdzip_codec_api::Cancellation,
) -> Result<PortablePath, &'static str> {
    let leaf = path.parts().last().expect("nonempty portable path");
    let (stem, extension) = leaf
        .rsplit_once('.')
        .filter(|(stem, _)| !stem.is_empty())
        .map(|(stem, extension)| (stem, format!(".{extension}")))
        .unwrap_or((leaf.as_str(), String::new()));
    for suffix in 1..=10_000 {
        cancel.check().map_err(|_| "cancelled")?;
        let mut parts = path.parts().to_vec();
        *parts.last_mut().expect("nonempty") = format!("{stem} ({suffix}){extension}");
        let candidate = PortablePath::parse(&parts.join("/")).ok_or("unsafe_name")?;
        if target.existing(&candidate).map_err(output)?.is_none() {
            return Ok(candidate);
        }
    }
    Err("rename_limit")
}
#[cfg(test)]
thread_local! { static TEST_WRITE_LIMIT: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) }; }
/// Meter before write and account partial writes. A quota cannot be raised by a declaration.
struct Meter<'a, W: Write> {
    writer: W,
    stats: &'a mut Snapshot,
    control: &'a Control,
    file_bytes: u64,
    error: Option<&'static str>,
    start: Instant,
    last: Instant,
    publish: &'a mut dyn FnMut(Snapshot),
}
impl<W: Write> DecodedSink for Meter<'_, W> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self.control.cancel.check().is_err() {
            return Err(DecodeError::Cancelled);
        }
        let n = bytes.len() as u64;
        if self
            .file_bytes
            .checked_add(n)
            .is_none_or(|n| n > MAX_FILE_BYTES)
        {
            self.error = Some("file_quota");
            return Err(DecodeError::Output);
        }
        if self
            .stats
            .decoded_bytes
            .checked_add(n)
            .is_none_or(|n| n > MAX_TASK_BYTES)
        {
            self.error = Some("task_quota");
            return Err(DecodeError::Output);
        }
        self.file_bytes += n;
        self.stats.decoded_bytes += n;
        let mut rest = bytes;
        while !rest.is_empty() {
            if self.control.cancel.check().is_err() {
                return Err(DecodeError::Cancelled);
            }
            #[cfg(test)]
            let remaining = TEST_WRITE_LIMIT.with(|limit| limit.get());
            #[cfg(test)]
            if remaining == Some(0) {
                self.error = Some("disk_full");
                return Err(DecodeError::Output);
            }
            #[cfg(test)]
            let request = &rest[..remaining.map_or(rest.len(), |n| rest.len().min(n as usize))];
            #[cfg(not(test))]
            let request = rest;
            match self.writer.write(request) {
                Ok(0) => {
                    self.error = Some("output_io");
                    return Err(DecodeError::Output);
                }
                Ok(n) => {
                    #[cfg(test)]
                    TEST_WRITE_LIMIT.with(|limit| {
                        if let Some(remaining) = limit.get() {
                            limit.set(Some(remaining - n as u64));
                        }
                    });
                    self.stats.written_bytes += n as u64;
                    rest = &rest[n..];
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => {
                    self.error = Some(if e.kind() == std::io::ErrorKind::StorageFull {
                        "disk_full"
                    } else {
                        "output_io"
                    });
                    return Err(DecodeError::Output);
                }
            }
        }
        if self.last.elapsed() >= Duration::from_millis(100) {
            emit(self.stats, self.start, self.publish);
            self.last = Instant::now();
        }
        Ok(())
    }
}
#[cfg(test)]
pub(super) fn inject_disk_full(bytes: Option<u64>) {
    TEST_WRITE_LIMIT.with(|limit| limit.set(bytes));
}

#[cfg(test)]
mod meter_tests {
    use super::*;
    #[test]
    fn quota_is_checked_before_write_and_partial_disk_full_is_counted() {
        struct Full(bool);
        impl Write for Full {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                if self.0 {
                    Err(std::io::ErrorKind::StorageFull.into())
                } else {
                    self.0 = true;
                    Ok(b.len().min(2))
                }
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let archive = ArchiveForTest::snapshot();
        let mut stats = archive;
        let control = Control::default();
        let mut publish = |_| {};
        let mut meter = Meter {
            writer: Full(false),
            stats: &mut stats,
            control: &control,
            file_bytes: MAX_FILE_BYTES,
            error: None,
            start: Instant::now(),
            last: Instant::now(),
            publish: &mut publish,
        };
        assert_eq!(meter.write_chunk(b"data"), Err(DecodeError::Output));
        assert_eq!(meter.stats.decoded_bytes, 0);
        assert_eq!(meter.stats.written_bytes, 0);
        meter.file_bytes = 0;
        assert_eq!(meter.write_chunk(b"data"), Err(DecodeError::Output));
        assert_eq!(meter.error, Some("disk_full"));
        assert_eq!(meter.stats.decoded_bytes, 4);
        assert_eq!(meter.stats.written_bytes, 2);
    }
    struct ArchiveForTest;
    impl ArchiveForTest {
        fn snapshot() -> Snapshot {
            let archive = crate::Archive::open(
                &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/extract/stored.zip"),
                &xdzip_codec_api::Cancellation::default(),
            )
            .unwrap();
            let job = archive
                .extraction_job(
                    super::super::Options {
                        target: std::env::temp_dir().to_string_lossy().into(),
                        all: true,
                        selected: vec![],
                        directory: String::new(),
                        create_folder: false,
                        conflict: ConflictPolicy::Ask,
                        allow_high_ratio: false,
                    },
                    &xdzip_codec_api::Cancellation::default(),
                )
                .unwrap();
            initial(&job, 1)
        }
    }
}
