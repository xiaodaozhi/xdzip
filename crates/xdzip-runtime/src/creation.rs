//! Whole-archive creation: input capabilities, streaming codec, validation and one commit.
use crate::extraction::{ConflictPolicy, Control, Prompt, Snapshot, Status};
use std::cell::Cell;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{Instant, UNIX_EPOCH};
use xdzip_codec_api::{
    Cancellation, MetadataCodec, ScanLimits,
    encode::{CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeOutput, EncodeSettings},
    payload::{DecodeError, DecodedSink, PayloadCodec},
};
use xdzip_codec_zip::ZipCodec;
use xdzip_domain::path::PortablePath;
use xdzip_io::{ExistingFile, Grant, InputTree, OutputError, Stage};

/// Explicit creation intent. Directory paths are host selections, not codec authority.
#[derive(Clone, Debug)]
pub struct Options {
    /// Selected absolute files/directories, with bounded aggregate path bytes.
    pub inputs: Vec<String>,
    /// Existing, explicitly selected output directory.
    pub target: String,
    /// Portable basename ending in .zip, never a subpath.
    pub name: String,
    /// ZIP method: 0 Store or 8 Deflate.
    pub method: u16,
    /// Deflate level: 1 quick, 6 balanced, 9 smaller.
    pub level: u8,
    /// Ask, rename, overwrite or decline the whole archive.
    pub conflict: ConflictPolicy,
}
impl Options {
    pub(crate) fn bounded(&self) -> bool {
        !self.inputs.is_empty()
            && self.inputs.len() <= 100_000
            && self.inputs.iter().all(|s| {
                !s.is_empty() && s.len() <= 4096 && !s.contains('\0') && Path::new(s).is_absolute()
            })
            && self.inputs.iter().map(String::len).sum::<usize>() <= 4 * 1024 * 1024
            && self.target.len() <= 4096
            && self.target.len() + self.name.len() < 4096
            && Path::new(&self.target).is_absolute()
            && !self.target.contains('\0')
            && self.name.len() <= 255
            && self.name.to_ascii_lowercase().ends_with(".zip")
            && PortablePath::parse(&self.name).is_some_and(|p| p.parts().len() == 1)
            && [0, 8].contains(&self.method)
            && [1, 6, 9].contains(&self.level)
    }
    pub(crate) fn bytes(&self) -> usize {
        self.inputs
            .iter()
            .map(|s| s.capacity() + std::mem::size_of::<String>())
            .sum::<usize>()
            + self.name.capacity()
            + self.target.capacity()
    }
}
pub(crate) struct Job {
    pub options: Options,
}
impl Job {
    pub fn retained_bytes(&self) -> usize {
        self.options.bytes()
    }
}
pub(crate) fn initial(job: &Job, id: u64) -> Snapshot {
    Snapshot {
        creating: true,
        total_known: false,
        id,
        session: 0,
        status: Status::Queued,
        target: Path::new(&job.options.target)
            .join(&job.options.name)
            .to_string_lossy()
            .into_owned(),
        current: String::new(),
        files: 0,
        declared_bytes: 0,
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
    }
}
struct Sources<'a> {
    tree: &'a InputTree,
    entries: Vec<EncodeEntry>,
    error: Cell<Option<&'static str>>,
}
struct CheckedRead<'a> {
    reader: xdzip_io::InputReader,
    error: &'a Cell<Option<&'static str>>,
}
impl Read for CheckedRead<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf).inspect_err(|e| {
            self.error.set(Some(if e.to_string() == "source_changed" {
                "source_changed"
            } else {
                "source_io"
            }))
        })
    }
}
impl EncodeInput for Sources<'_> {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        let reader = self.tree.open(index).map_err(|e| {
            self.error.set(Some(e.code()));
            EncodeError::Source
        })?;
        Ok(Box::new(CheckedRead {
            reader,
            error: &self.error,
        }))
    }
}
struct Output<'a> {
    stage: &'a mut Stage,
    bytes: &'a Cell<u64>,
    error: &'a Cell<Option<OutputError>>,
    cancel: &'a Cancellation,
}
impl Write for Output<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.cancel.check().is_err() {
            return Err(io::Error::other("cancelled"));
        }
        if self.bytes.get().saturating_add(buf.len() as u64) > 24 * 1024 * 1024 * 1024 {
            self.error.set(Some(OutputError::Limit));
            return Err(io::Error::other("output_limit"));
        }
        match self.stage.write(buf) {
            Ok(n) => {
                self.bytes.set(self.bytes.get() + n as u64);
                Ok(n)
            }
            Err(e) => {
                self.error
                    .set(Some(if e.kind() == io::ErrorKind::StorageFull {
                        OutputError::NoSpace
                    } else {
                        OutputError::Io
                    }));
                Err(e)
            }
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        self.stage.flush().inspect_err(|e| {
            self.error
                .set(Some(if e.kind() == io::ErrorKind::StorageFull {
                    OutputError::NoSpace
                } else {
                    OutputError::Io
                }))
        })
    }
}
struct VerifySink<'a> {
    bytes: u64,
    limit: u64,
    cancel: &'a Cancellation,
}
impl DecodedSink for VerifySink<'_> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.cancel.check().map_err(|_| DecodeError::Cancelled)?;
        self.bytes = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or(DecodeError::Limit)?;
        if self.bytes > self.limit {
            return Err(DecodeError::Limit);
        }
        Ok(())
    }
}
fn emit(stats: &mut Snapshot, start: Instant, publish: &mut dyn FnMut(Snapshot)) {
    stats.elapsed_ms = start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    publish(stats.clone());
}
fn fresh_name(
    root: &Grant,
    name: &str,
    cancel: &Cancellation,
) -> Result<PortablePath, &'static str> {
    let stem = name.get(..name.len() - 4).ok_or("unsafe_name")?;
    for suffix in 2..=10_000 {
        cancel.check().map_err(|_| "cancelled")?;
        let path = PortablePath::parse(&format!("{stem} ({suffix}).zip")).ok_or("unsafe_name")?;
        if root.existing(&path).map_err(output_code)?.is_none() {
            return Ok(path);
        }
    }
    Err("resource_limit")
}
fn output_code(e: OutputError) -> &'static str {
    match e {
        OutputError::Io => "output_io",
        OutputError::NoSpace => "disk_full",
        OutputError::UnsafeTarget => "unsafe_target",
        OutputError::TargetChanged => "target_changed",
        OutputError::Conflict => "output_conflict",
        OutputError::Unsupported => "unsupported_filesystem",
        OutputError::Limit => "resource_limit",
    }
}
pub(crate) fn run(job: Job, id: u64, control: &Control, publish: &mut dyn FnMut(Snapshot)) {
    let start = Instant::now();
    let mut stats = initial(&job, id);
    let mut workspace = None;
    let result = (|| -> Result<(), &'static str> {
        control.cancel.check().map_err(|_| "cancelled")?;
        stats.status = Status::Preparing;
        emit(&mut stats, start, publish);
        let root = Grant::for_creation(Path::new(&job.options.target)).map_err(output_code)?;
        stats.leftovers = root.leftover_count().map_err(output_code)? as u64;
        let output_path = Path::new(&job.options.target).join(&job.options.name);
        if job
            .options
            .inputs
            .iter()
            .any(|p| Path::new(p) == output_path)
        {
            return Err("unsafe_input");
        }
        workspace = Some(root.workspace(id).map_err(output_code)?);
        let w = workspace.as_mut().expect("workspace created");
        let (excluded, identity) = w.input_exclusion();
        let paths: Vec<_> = job.options.inputs.iter().map(PathBuf::from).collect();
        let stopped = || control.cancel.check().is_err();
        let tree = InputTree::scan(
            &paths,
            vec![(output_path, None), (excluded, Some(identity))],
            &stopped,
        )
        .map_err(|e| e.code())?;
        stats.files = tree.entries().iter().filter(|e| !e.directory).count() as u64;
        stats.declared_bytes = tree.total_bytes();
        stats.total_known = true;
        let entries = tree
            .entries()
            .iter()
            .map(|e| EncodeEntry {
                name: e.name.clone(),
                directory: e.directory,
                size: e.size,
                modified_unix: e
                    .modified
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs().min(i64::MAX as u64) as i64)
                    .unwrap_or(0),
            })
            .collect();
        let sources = Sources {
            tree: &tree,
            entries,
            error: Cell::new(None),
        };
        let mut stage = w.stage().map_err(output_code)?;
        let written = Cell::new(0);
        let output_error = Cell::new(None);
        let mut last = Instant::now();
        stats.status = Status::Running;
        emit(&mut stats, start, publish);
        ZipCodec
            .encode(
                &sources,
                &mut Output {
                    stage: &mut stage,
                    bytes: &written,
                    error: &output_error,
                    cancel: &control.cancel,
                } as &mut dyn EncodeOutput,
                EncodeSettings {
                    method: if job.options.method == 0 {
                        "store"
                    } else {
                        "deflate"
                    },
                    level: job.options.level,
                },
                &control.cancel,
                &mut |p| {
                    stats.current = p.name.into();
                    stats.decoded_bytes = p.input_bytes;
                    stats.written_bytes = written.get();
                    if last.elapsed().as_millis() >= 100 {
                        emit(&mut stats, start, publish);
                        last = Instant::now();
                    }
                },
            )
            .map_err(|e| match e {
                EncodeError::Cancelled => "cancelled",
                EncodeError::Source => sources.error.get().unwrap_or("source_changed"),
                EncodeError::Output => output_error
                    .get()
                    .map(output_code)
                    .unwrap_or(if stopped() { "cancelled" } else { "output_io" }),
                EncodeError::Invalid => "integrity_error",
                EncodeError::Unsupported => "unsupported_feature",
            })?;
        stats.written_bytes = written.get();
        stats.decoded_bytes = stats.declared_bytes;
        tree.validate(&stopped).map_err(|e| e.code())?;
        stats.status = Status::Verifying;
        emit(&mut stats, start, publish);
        let mut reader = stage.verification_reader().map_err(output_code)?;
        let rows = ZipCodec
            .scan(&mut reader, &control.cancel, ScanLimits::default())
            .map_err(|e| {
                if control.cancel.check().is_err() {
                    "cancelled"
                } else {
                    let _ = e;
                    "integrity_error"
                }
            })?;
        if rows.len() != tree.entries().len() {
            return Err("integrity_error");
        }
        {
            let mut decoder = ZipCodec
                .open_payload(&mut reader, &rows, &control.cancel)
                .map_err(|_| "integrity_error")?;
            for (row, input) in rows.iter().zip(tree.entries()) {
                control.cancel.check().map_err(|_| "cancelled")?;
                let expected = if input.directory {
                    format!("{}/", input.name)
                } else {
                    input.name.clone()
                };
                if row.raw_name != expected.as_bytes()
                    || row.directory != input.directory
                    || row.size != input.size
                {
                    return Err("integrity_error");
                }
                stats.current = input.name.clone();
                if last.elapsed().as_millis() >= 100 {
                    emit(&mut stats, start, publish);
                    last = Instant::now();
                }
                decoder
                    .decode(
                        row,
                        &mut VerifySink {
                            bytes: 0,
                            limit: input.size,
                            cancel: &control.cancel,
                        },
                        &control.cancel,
                    )
                    .map_err(|_| {
                        if stopped() {
                            "cancelled"
                        } else {
                            "integrity_error"
                        }
                    })?;
            }
        }
        drop(reader);
        tree.validate(&stopped).map_err(|e| e.code())?;
        let mut path = PortablePath::parse(&job.options.name).ok_or("unsafe_name")?;
        let mut overwrite: Option<ExistingFile> = None;
        if let Some(existing) = root.existing(&path).map_err(output_code)? {
            if tree.contains_identity(existing.version.identity) {
                return Err("unsafe_input");
            }
            let policy = if job.options.conflict == ConflictPolicy::Ask {
                control.arm(1);
                stats.status = Status::Waiting;
                stats.prompt = Some(Prompt {
                    id: 1,
                    name: path.display(),
                    existing_bytes: existing.version.length,
                    incoming_bytes: stats.written_bytes,
                    existing_modified: existing
                        .version
                        .modified
                        .duration_since(UNIX_EPOCH)
                        .ok()
                        .map(|d| d.as_secs()),
                    incoming_modified_dos: 0,
                    overwrite_allowed: true,
                });
                emit(&mut stats, start, publish);
                control.wait()?.0
            } else {
                job.options.conflict
            };
            stats.prompt = None;
            match policy {
                ConflictPolicy::Skip => return Err("cancelled"),
                ConflictPolicy::Rename => {
                    path = fresh_name(&root, &job.options.name, &control.cancel)?
                }
                ConflictPolicy::Overwrite => overwrite = Some(existing),
                ConflictPolicy::Ask => return Err("invalid_decision"),
            }
        }
        tree.validate(&stopped).map_err(|e| e.code())?;
        stats.target = root
            .display_path()
            .join(path.display())
            .to_string_lossy()
            .into_owned();
        control.begin_commit()?;
        stats.status = Status::Committing;
        emit(&mut stats, start, publish);
        // Whole-archive publication is the single commit point. A concurrent late
        // cancel cannot turn a successfully published archive into cancellation.
        let receipt = w
            .commit(stage, &path, overwrite.as_ref())
            .map_err(output_code)?;
        control.mark_terminal();
        stats.cleanup_warning |= receipt.cleanup_warning;
        stats.committed_files = stats.files;
        stats.committed_bytes = stats.declared_bytes;
        stats.created_directories = tree.entries().iter().filter(|e| e.directory).count() as u64;
        Ok(())
    })();
    stats.prompt = None;
    stats.status = Status::Cleaning;
    emit(&mut stats, start, publish);
    if let Some(w) = workspace {
        stats.cleanup_warning |= w.cleanup().is_err();
    }
    stats.status = match result {
        Ok(()) => Status::Succeeded,
        Err("cancelled") => Status::Cancelled,
        Err(_) => Status::Failed,
    };
    stats.error = result.err().unwrap_or_default().into();
    control.mark_terminal();
    emit(&mut stats, start, publish);
}
#[cfg(test)]
mod tests;
