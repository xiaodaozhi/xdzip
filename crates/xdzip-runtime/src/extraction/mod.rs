//! Safe ZIP task contracts. Plans are frozen before granting any output authority.
pub(crate) mod auxiliary;
mod control;
mod execute;
mod links;
#[cfg(test)]
mod opening_tests;
#[cfg(test)]
pub(crate) mod password_tests;
mod plan;
#[cfg(test)]
mod tests;
use crate::Archive;
pub use control::Control;
pub(crate) use execute::{initial, run};
pub(crate) use plan::Job;
use std::sync::Arc;
use xdzip_codec_api::Cancellation;

/// Default hard limits meter actual bytes independently of archive declarations.
pub const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// Per-task decompressed byte ceiling, including failed or repeated decoding.
pub const MAX_TASK_BYTES: u64 = 20 * 1024 * 1024 * 1024;
/// Maximum accepted tasks, including undelivered terminal snapshots.
pub const MAX_TASKS: usize = 32;
/// UTF-8 bytes accepted from a password response, independently of creation policy.
pub const MAX_PASSWORD_BYTES: usize = 1024;

/// One task-local extraction secret. It is never cloned into events or commands.
#[derive(Clone)]
pub struct Password(zeroize::Zeroizing<String>);
impl Password {
    /// Preserve all UTF-8 bytes, including empty values, whitespace and controls.
    pub fn new(value: String) -> Result<Self, &'static str> {
        let value = zeroize::Zeroizing::new(value);
        if value.len() > MAX_PASSWORD_BYTES || value.capacity() > 4 * MAX_PASSWORD_BYTES {
            return Err("invalid_password");
        }
        // Copy only the bounded contents, rather than retaining an oversized caller capacity.
        Ok(Self(zeroize::Zeroizing::new(value.as_str().to_owned())))
    }
    /// Borrow only for the synchronous codec call.
    pub fn expose(&self) -> &str {
        &self.0
    }
}
impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED]")
    }
}

/// Existing regular-file policy. Links and source archives remain protected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConflictPolicy {
    /// Ask with a fresh prompt identity and an inspected existing-file version.
    Ask,
    /// Leave the existing destination intact.
    Skip,
    /// Publish to a fresh portable suffixed name.
    Rename,
    /// Replace the inspected ordinary file by a whole-file rename.
    Overwrite,
}
impl ConflictPolicy {
    /// Stable UI codes; unknown values cannot become an overwrite decision.
    pub fn from_code(value: u8) -> Option<Self> {
        Some(match value {
            0 => Self::Ask,
            1 => Self::Skip,
            2 => Self::Rename,
            3 => Self::Overwrite,
            _ => return None,
        })
    }
}
/// Explicit host-selected extraction intent. Archive names never supply the target root.
#[derive(Clone, Debug)]
pub struct Options {
    /// Absolute existing destination directory selected in the host UI.
    pub target: String,
    /// Include every physical entry, independent of the current view.
    pub all: bool,
    /// Physical decimal IDs or d-prefixed navigation IDs, at most 100,000.
    pub selected: Vec<String>,
    /// Safe directory ID whose prefix is removed for selected-file output.
    pub directory: String,
    /// Derive one archive-named child grant under the selected directory.
    pub create_folder: bool,
    /// Initial ordinary-file conflict policy.
    pub conflict: ConflictPolicy,
    /// Permit declared ratios above 1000:1, retaining all hard byte limits.
    pub allow_high_ratio: bool,
}
impl Options {
    pub(crate) fn bounded(&self) -> bool {
        !self.target.is_empty()
            && self.target.len() <= 4096
            && !self.target.contains('\0')
            && std::path::Path::new(&self.target).is_absolute()
            && self.directory.len() <= 21
            && self.selected.len() <= xdzip_codec_api::MAX_ENTRIES
            && self
                .selected
                .iter()
                .all(|id| !id.is_empty() && id.len() <= 21)
            && (self.all || !self.selected.is_empty())
    }
    pub(crate) fn bytes(&self) -> usize {
        self.target.capacity()
            + self.directory.capacity()
            + self.selected.capacity() * std::mem::size_of::<String>()
            + self.selected.iter().map(String::capacity).sum::<usize>()
    }
}
/// Execution state; terminal states are published only after staging cleanup.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    /// Accepted and waiting for the sole heavy worker.
    Queued,
    /// Acquiring target capabilities and format locators.
    Preparing,
    /// Decoding a verified admitted file.
    Running,
    /// Re-reading a complete private archive before whole-file publication.
    Verifying,
    /// Awaiting an explicit conflict response.
    Waiting,
    /// Awaiting a one-use password response through the independent control lane.
    WaitingPassword,
    /// Publishing one complete file.
    Committing,
    /// Cancellation was requested; it has not yet finished.
    Cancelling,
    /// Closing and removing only known private staging objects.
    Cleaning,
    /// Every admitted file completed or was explicitly skipped.
    Succeeded,
    /// A stable error stopped execution; earlier committed files remain.
    Failed,
    /// Cancellation completed; earlier committed files remain.
    Cancelled,
}
impl Status {
    /// Stable bridge label, independent of translated UI text.
    pub fn code(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Preparing => "preparing",
            Self::Running => "running",
            Self::Verifying => "verifying",
            Self::Waiting => "waiting",
            Self::WaitingPassword => "waiting_password",
            Self::Committing => "committing",
            Self::Cancelling => "cancelling",
            Self::Cleaning => "cleaning",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
    /// Whether no more task work or prompts will be produced.
    pub fn terminal(self) -> bool {
        matches!(self, Self::Succeeded | Self::Failed | Self::Cancelled)
    }
}
/// Versioned conflict prompt. It grants no filesystem access to the GUI.
#[derive(Clone, Debug)]
pub struct Prompt {
    /// Fresh per-task response identity, never reused.
    pub id: u64,
    /// Safe escaped relative name.
    pub name: String,
    /// Existing ordinary-file bytes.
    pub existing_bytes: u64,
    /// Incoming declared bytes, not yet verified.
    pub incoming_bytes: u64,
    /// Existing filesystem modification time in Unix seconds, if representable.
    pub existing_modified: Option<u64>,
    /// Incoming ZIP local DOS date/time; no timezone is inferred.
    pub incoming_modified_dos: u32,
    /// False when replacing would overwrite the selected archive.
    pub overwrite_allowed: bool,
}
/// A password request contains display metadata only, never a secret or verifier.
#[derive(Clone, Debug)]
pub struct PasswordPrompt {
    /// Fresh per-task response identity, shared with conflict prompt numbering.
    pub id: u64,
    /// Escaped relative name requiring this password.
    pub name: String,
    /// A previous password failed the entry's password verifier.
    pub retry: bool,
}
/// Actual completed file observation, independent of UI strings and icons.
#[derive(Clone, Debug)]
pub struct Outcome {
    /// Escaped relative name, capped at 1024 UTF-8 bytes for display.
    pub name: String,
    /// Actual file's admitted byte size.
    pub size: u64,
    /// Stable status: completed, skipped or unfinished.
    pub status: &'static str,
    /// Fixed error code, never arbitrary OS text.
    pub error: &'static str,
}
/// Operation identity shared by task scheduling and display snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskKind {
    /// Move archive entries through a verified whole-container rewrite.
    Move,
    /// Publish a prepared private item to a native file-promise destination.
    ExportCopy,
    /// Verified private copies offered to an external copy operation.
    Export,
    /// Publish selected archive entries.
    Extract,
    /// Publish a new complete archive.
    Create,
    /// Atomically replace an explicitly selected archive after full validation.
    Update,
    /// Read and validate an archive without filesystem output.
    Check,
}
impl TaskKind {
    /// Stable bridge label, independent of translated text.
    pub fn code(self) -> &'static str {
        match self {
            Self::ExportCopy => "export-copy",
            Self::Export => "export",
            Self::Extract => "extract",
            Self::Create => "create",
            Self::Update => "update",
            Self::Move => "move",
            Self::Check => "check",
        }
    }
}
/// Bounded replaceable display snapshot. Counters describe actual work, not a timer simulation.
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// Operation identity. Checking never grants output authority.
    pub kind: TaskKind,
    /// Original check input for display, never a destination or output authority.
    pub source: String,
    /// Validated check format, independent of a user-controlled extension.
    pub format: &'static str,
    /// True for whole-archive creation; false for per-file extraction.
    pub creating: bool,
    /// False while a creation input tree is still being scanned.
    pub total_known: bool,
    /// Opaque task identity across session changes.
    pub id: u64,
    /// Original archive session; source failures cannot invalidate a different session.
    pub session: u64,
    /// Current execution status.
    pub status: Status,
    /// Display-only output location.
    pub target: String,
    /// Local file produced by an explicit temporary-open intent, never an archive URL.
    pub open_path: String,
    /// Escaped current relative filename.
    pub current: String,
    /// Declared admitted file count, excluding directories.
    pub files: u64,
    /// Declared admitted bytes, not a write authority.
    pub declared_bytes: u64,
    /// Actual decoded bytes accepted by the host meter.
    pub decoded_bytes: u64,
    /// Actual source reads during a read-only check, including header reads and retries.
    pub read_bytes: u64,
    /// Non-directory files whose required checks completed, including empty files.
    pub verified_files: u64,
    /// Declared payload bytes in fully verified non-directory files, never written bytes.
    pub verified_bytes: u64,
    /// Actual bytes successfully written into private stages.
    pub written_bytes: u64,
    /// Actual bytes in successfully published complete files.
    pub committed_bytes: u64,
    /// Published regular files.
    pub committed_files: u64,
    /// Explicitly skipped existing regular files.
    pub skipped_files: u64,
    /// Directories actually created; cancellation does not roll them back.
    pub created_directories: u64,
    /// Monotonic wall-clock elapsed milliseconds.
    pub elapsed_ms: u64,
    /// Stable error code, without OS text or arbitrary archive strings.
    pub error: String,
    /// Cleanup could not remove all known staging objects.
    pub cleanup_warning: bool,
    /// A runtime panic prevents proving exact final publication counts.
    pub outcome_uncertain: bool,
    /// Possible previous private workspaces found at the target; never auto-deleted.
    pub leftovers: u64,
    /// Present only for the latest outstanding conflict.
    pub prompt: Option<Prompt>,
    /// Present only while waiting for the latest one-use password response.
    pub password_prompt: Option<PasswordPrompt>,
    /// Recent actual file outcomes, at most 128 records, shared by progress snapshots.
    pub details: std::sync::Arc<Vec<Outcome>>,
    /// Number of older records omitted from this bounded display history.
    pub omitted_details: u64,
    /// True if the result needs an external reader: numbered split ZIP.
    pub external_reader: bool,
    /// Number of published archive parts; zero for extraction or failed creation.
    pub output_parts: u64,
}
impl Snapshot {
    pub(crate) fn record(
        &mut self,
        name: &str,
        size: u64,
        status: &'static str,
        error: &'static str,
    ) {
        let records = std::sync::Arc::make_mut(&mut self.details);
        if records.len() == 128 {
            records.remove(0);
            self.omitted_details += 1;
        }
        let mut limited = String::new();
        for ch in name.chars() {
            if limited.len() + ch.len_utf8() > 1024 {
                break;
            }
            limited.push(ch);
        }
        records.push(Outcome {
            name: limited,
            size,
            status,
            error,
        });
    }
}
/// Relative descriptor for an external copy, never a host pathname.
#[derive(Debug)]
pub struct ExportEntry {
    /// Validated relative path under the private payload directory.
    pub name: String,
    /// Directory, including implicit parent directories.
    pub directory: bool,
    /// Declared bytes; the export task verifies content before exposing it.
    pub size: u64,
}
impl Archive {
    pub(crate) fn export_manifest(
        &self,
        options: &Options,
        cancel: &Cancellation,
    ) -> Result<Vec<ExportEntry>, &'static str> {
        self.source.validate().map_err(|_| "source_changed")?;
        let planned = plan::prepare(self, options, cancel)?;
        let mut entries = std::collections::BTreeMap::<String, ExportEntry>::new();
        let mut bytes = 0usize;
        for item in planned {
            let row = &self.rows[item.index];
            if row.link {
                return Err("unsupported_feature");
            }
            for end in 1..=item.path.parts().len() {
                let name = item.path.parts()[..end].join("/");
                let directory = end < item.path.parts().len() || row.directory;
                if let Some(old) = entries.get(&name) {
                    if old.directory != directory || !directory {
                        return Err("name_collision");
                    }
                } else {
                    bytes += name.len() + 32;
                    if bytes > 4 * 1024 * 1024 || entries.len() >= 100_000 {
                        return Err("resource_limit");
                    }
                    entries.insert(
                        name.clone(),
                        ExportEntry {
                            name,
                            directory,
                            size: if directory { 0 } else { row.size },
                        },
                    );
                }
            }
        }
        if entries.is_empty() {
            return Err("invalid_selection");
        }
        Ok(entries.into_values().collect())
    }
    pub(crate) fn export_job(
        &self,
        options: Options,
        cancel: &Cancellation,
    ) -> Result<Job, &'static str> {
        self.export_manifest(&options, cancel)?;
        let cache = std::path::PathBuf::from(&options.target);
        let mut job = self.extraction_job(options, cancel)?;
        let ids = self.source.identities().map_err(|_| "source_changed")?;
        let grant = xdzip_io::Grant::temporary(&cache, &ids).map_err(|e| match e {
            xdzip_io::OutputError::Limit => "resource_limit",
            xdzip_io::OutputError::UnsafeTarget => "unsafe_target",
            _ => "output_io",
        })?;
        job.options.target = grant.display_path().to_string_lossy().into_owned();
        job.temporary = Some(grant);
        Ok(job)
    }

    /// Freeze selectors and input identity without creating a destination object.
    pub(crate) fn extraction_job(
        &self,
        options: Options,
        cancel: &Cancellation,
    ) -> Result<Job, &'static str> {
        self.source.validate().map_err(|_| "source_changed")?;
        let source = self.source.fork().map_err(|_| "source_changed")?;
        let planned = plan::prepare(self, &options, cancel)?;
        let links_pending = planned
            .iter()
            .any(|p| self.rows[p.index].link && self.rows[p.index].encrypted);
        Ok(Job {
            source,
            password: self.password.clone(),
            format: self.format,
            rows: Arc::clone(&self.rows),
            planned,
            options,
            session: self.key.session.get(),
            links_pending,
            temporary: None,
            open_path: String::new(),
        })
    }
    pub(crate) fn open_entry_job(
        &self,
        entry: u64,
        cache: &std::path::Path,
        cancel: &Cancellation,
    ) -> Result<Job, &'static str> {
        let index = entry
            .checked_sub(1)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or("invalid_selection")?;
        self.rows
            .get(index)
            .filter(|r| {
                r.id.get() == entry
                    && !r.directory
                    && !r.link
                    && (r.unix_mode & 0o170000 == 0 || r.unix_mode & 0o170000 == 0o100000)
            })
            .ok_or("unsupported_feature")?;
        let options = Options {
            target: cache.to_string_lossy().into_owned(),
            all: false,
            selected: vec![entry.to_string()],
            directory: String::new(),
            create_folder: false,
            conflict: ConflictPolicy::Ask,
            allow_high_ratio: false,
        };
        let mut job = self.extraction_job(options, cancel)?;
        if job.planned.len() != 1 {
            return Err("invalid_selection");
        }
        let filename = job.planned[0]
            .path
            .parts()
            .last()
            .ok_or("invalid_selection")?
            .clone();
        let path = xdzip_domain::path::PortablePath::parse(&filename).ok_or("invalid_selection")?;
        // Validate original names before flattening. Random private destinations
        // isolate duplicate archive names and separate open requests.
        let ids = self.source.identities().map_err(|_| "source_changed")?;
        let grant = xdzip_io::Grant::temporary(cache, &ids).map_err(|e| match e {
            xdzip_io::OutputError::Limit => "resource_limit",
            xdzip_io::OutputError::UnsafeTarget => "unsafe_target",
            _ => "output_io",
        })?;
        cancel.check().map_err(|_| "cancelled")?;
        job.options.target = grant.display_path().to_string_lossy().into_owned();
        job.open_path = grant
            .display_path()
            .join(&filename)
            .to_string_lossy()
            .into_owned();
        if job.options.target.len() > 4096 || job.open_path.len() > 4096 {
            return Err("invalid_options");
        }
        job.planned[0].path = path;
        job.temporary = Some(grant);
        Ok(job)
    }
}
