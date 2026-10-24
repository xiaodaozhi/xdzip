// crates/xdzip-runtime/src/updating.rs

//! Whole-archive edit with frozen source, input tree, full verification and one commit.
use crate::extraction::{ConflictPolicy, Control, Prompt, Snapshot, Status, TaskKind};
use crate::{Archive, creation, formats::codec, source::Source};
use std::{
    collections::BTreeMap,
    io::Read,
    path::PathBuf,
    sync::Arc,
    time::{Instant, UNIX_EPOCH},
};
use unicode_normalization::UnicodeNormalization;
use xdzip_codec_api::{
    Cancellation, Entry,
    encode::*,
    payload::{DecodeError, DecodedSink},
};
use xdzip_domain::path::PortablePath;
use xdzip_io::{ArchiveUpdate, InputTree};

mod moving;
pub use moving::MoveOptions;

/// Explicit host edit intent; the runtime resolves the opaque directory identity.
#[derive(Clone, Debug)]
pub struct Options {
    /// Absolute local input paths, reauthorized through a frozen no-follow tree.
    pub inputs: Vec<String>,
    /// Current archive directory ID; empty denotes the archive root.
    pub directory: String,
    /// Conflict policy within the archive, never authorization to overwrite host inputs.
    pub conflict: ConflictPolicy,
}
impl Options {
    pub(crate) fn bounded(&self) -> bool {
        !self.inputs.is_empty()
            && self.inputs.len() <= 100_000
            && self.directory.len() <= 21
            && self.inputs.iter().all(|p| {
                !p.contains('\0') && p.len() <= 4096 && std::path::Path::new(p).is_absolute()
            })
            && self.inputs.iter().map(String::len).sum::<usize>() <= 4 * 1024 * 1024
    }
    pub(crate) fn bytes(&self) -> usize {
        self.inputs.iter().map(|p| p.capacity() + 24).sum::<usize>() + self.directory.capacity()
    }
}
pub(crate) struct Job {
    source: Source,
    rows: Arc<Vec<Entry>>,
    format: xdzip_codec_api::Format,
    options: Options,
    prefix: String,
    session: u64,
    moving: Option<moving::Moves>,
}
impl Job {
    pub fn retained_bytes(&self) -> usize {
        self.options.bytes()
            + self.source.retained_bytes()
            + self.rows.len() * 256
            + self.moving.as_ref().map_or(0, moving::Moves::bytes)
    }
}
fn key(name: &str) -> String {
    name.nfd().flat_map(char::to_lowercase).collect()
}
impl Archive {
    pub(crate) fn can_update(&self) -> bool {
        self.update_allowed
    }
    pub(crate) fn update_job(
        &self,
        options: Options,
        cancel: &Cancellation,
    ) -> Result<Job, &'static str> {
        if !options.bounded() {
            return Err("invalid_options");
        }
        if !self.can_update() {
            return Err("unsupported_feature");
        }
        let prefix = std::str::from_utf8(
            self.directory_prefix(&options.directory)
                .map_err(|_| "invalid_selection")?,
        )
        .map_err(|_| "unsafe_name")?
        .to_string();
        cancel.check().map_err(|_| "cancelled")?;
        Ok(Job {
            source: self.source.fork().map_err(|_| "source_changed")?,
            rows: Arc::clone(&self.rows),
            format: self.format,
            options,
            prefix,
            session: self.key.session.get(),
            moving: None,
        })
    }
}
pub(crate) fn initial(job: &Job, id: u64) -> Snapshot {
    let path = job.source.display_path();
    let mut snapshot = creation::initial(
        &creation::Job {
            options: creation::Options {
                inputs: vec![],
                target: path.parent().unwrap_or(path).to_string_lossy().into(),
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into(),
                method: 8,
                level: 6,
                conflict: ConflictPolicy::Ask,
                volume_bytes: 0,
                password: None,
            },
        },
        id,
    );
    snapshot.kind = if job.moving.is_some() {
        TaskKind::Move
    } else {
        TaskKind::Update
    };
    snapshot.source = path.to_string_lossy().into();
    snapshot.session = job.session;
    snapshot.format = job.format.label();
    snapshot
}
struct Additions<'a> {
    tree: Option<&'a InputTree>,
    entries: Vec<EncodeEntry>,
    indices: Vec<usize>,
}
impl EncodeInput for Additions<'_> {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(
            self.tree
                .ok_or(EncodeError::Source)?
                .open(self.indices[index])
                .map_err(|_| EncodeError::Source)?,
        ))
    }
}
struct Verify {
    bytes: u64,
    limit: u64,
}
impl DecodedSink for Verify {
    fn write_chunk(&mut self, data: &[u8]) -> Result<(), DecodeError> {
        self.bytes = self
            .bytes
            .checked_add(data.len() as u64)
            .ok_or(DecodeError::Limit)?;
        if self.bytes > self.limit {
            return Err(DecodeError::Limit);
        }
        Ok(())
    }
}
fn output(e: xdzip_io::OutputError) -> &'static str {
    match e {
        xdzip_io::OutputError::NoSpace => "disk_full",
        xdzip_io::OutputError::UnsafeTarget => "unsafe_target",
        xdzip_io::OutputError::TargetChanged => "source_changed",
        xdzip_io::OutputError::Limit => "resource_limit",
        xdzip_io::OutputError::Unsupported => "unsupported_filesystem",
        _ => "output_io",
    }
}
fn emit(s: &mut Snapshot, start: Instant, publish: &mut dyn FnMut(Snapshot)) {
    s.elapsed_ms = start.elapsed().as_millis() as u64;
    publish(s.clone());
}
// The namespace includes implicit directories, so a file cannot replace an
// ancestor or hide descendants. Canonical aliases keep the original spelling.
fn insert_name(
    map: &mut BTreeMap<String, (String, bool, Option<usize>)>,
    budget: &mut usize,
    name: &str,
    directory: bool,
    index: Option<usize>,
) -> Result<(), &'static str> {
    let parts: Vec<_> = name.split('/').collect();
    for end in 1..=parts.len() {
        let spelling = parts[..end].join("/");
        let is_dir = end < parts.len() || directory;
        let k = key(&spelling);
        if let Some((old, kind, record)) = map.get_mut(&k) {
            if *kind != is_dir
                || old != &spelling
                || (end == parts.len() && record.is_some() && index.is_some())
            {
                return Err("name_collision");
            }
            if end == parts.len() && index.is_some() {
                *record = index;
            }
        } else {
            *budget = budget
                .checked_add(k.len() + spelling.len())
                .ok_or("resource_limit")?;
            if map.len() >= 100_000 || *budget > 16 * 1024 * 1024 {
                return Err("resource_limit");
            }
            map.insert(
                k,
                (
                    spelling,
                    is_dir,
                    if end == parts.len() { index } else { None },
                ),
            );
        }
    }
    Ok(())
}
struct Resolved {
    name: String,
    directory_exists: bool,
    directory_record: bool,
}
struct Planner<'a> {
    rows: &'a [Entry],
    names: BTreeMap<String, (String, bool, Option<usize>)>,
    namespace_bytes: usize,
    keep: Vec<bool>,
    renames: Vec<Option<String>>,
    policy: ConflictPolicy,
    prompt: u64,
    control: &'a Control,
    snapshot: &'a mut Snapshot,
    start: Instant,
    publish: &'a mut dyn FnMut(Snapshot),
}
impl Planner<'_> {
    fn resolve(
        &mut self,
        mut name: String,
        directory: bool,
        size: u64,
    ) -> Result<Option<Resolved>, &'static str> {
        // Normalize directory spellings against already admitted ancestors.
        let parts: Vec<_> = name.split('/').map(str::to_string).collect();
        let mut actual = String::new();
        for part in &parts[..parts.len() - 1] {
            if !actual.is_empty() {
                actual.push('/');
            }
            actual.push_str(part);
            if let Some((spelling, dir, _)) = self.names.get(&key(&actual)) {
                if !dir {
                    return Err("name_collision");
                }
                actual = spelling.clone();
            }
        }
        name = if actual.is_empty() {
            parts.last().unwrap().clone()
        } else {
            format!("{actual}/{}", parts.last().unwrap())
        };
        if let Some((old, existing_directory, original)) = self.names.get(&key(&name)).cloned() {
            if existing_directory != directory {
                return Err("name_collision");
            }
            if directory {
                return Ok(Some(Resolved {
                    name: old,
                    directory_exists: true,
                    directory_record: original.is_some(),
                }));
            }
            let mut decision = self.policy;
            if decision == ConflictPolicy::Ask {
                self.prompt += 1;
                self.control.arm(self.prompt);
                self.snapshot.status = Status::Waiting;
                self.snapshot.prompt = Some(Prompt {
                    id: self.prompt,
                    name: name.clone(),
                    existing_bytes: original.map_or(0, |i| self.rows[i].size),
                    incoming_bytes: size,
                    existing_modified: None,
                    incoming_modified_dos: 0,
                    overwrite_allowed: original.is_some(),
                });
                emit(self.snapshot, self.start, self.publish);
                let (answer, all) = self.control.wait()?;
                decision = answer;
                if all {
                    self.policy = answer;
                }
                self.snapshot.prompt = None;
            }
            match decision {
                ConflictPolicy::Skip => {
                    self.snapshot.skipped_files += 1;
                    return Ok(None);
                }
                ConflictPolicy::Overwrite => {
                    let i = original.ok_or("name_collision")?;
                    self.keep[i] = false;
                    name = old;
                }
                ConflictPolicy::Rename => {
                    let stem = name.clone();
                    let mut fresh = None;
                    for number in 2..=10_000 {
                        let (parent, leaf) = stem
                            .rsplit_once('/')
                            .map_or((String::new(), stem.as_str()), |(p, n)| {
                                (format!("{p}/"), n)
                            });
                        let (base, ext) = leaf
                            .rsplit_once('.')
                            .filter(|(a, _)| !a.ends_with('/') && !a.is_empty())
                            .map_or((leaf, String::new()), |(a, b)| (a, format!(".{b}")));
                        let candidate = format!("{parent}{base} ({number}){ext}");
                        if !self.names.contains_key(&key(&candidate)) {
                            fresh = Some(candidate);
                            break;
                        }
                    }
                    name = fresh.ok_or("resource_limit")?;
                }
                ConflictPolicy::Ask => return Err("invalid_decision"),
            }
        }
        PortablePath::parse(&name).ok_or("unsafe_name")?;
        if !self.names.contains_key(&key(&name)) {
            insert_name(
                &mut self.names,
                &mut self.namespace_bytes,
                &name,
                directory,
                None,
            )?;
        }

        Ok(Some(Resolved {
            name,
            directory_exists: false,
            directory_record: false,
        }))
    }
}

pub(crate) fn run(mut job: Job, id: u64, control: &Control, publish: &mut dyn FnMut(Snapshot)) {
    let start = Instant::now();
    let mut s = initial(&job, id);
    let mut workspace = None;
    let result = (|| -> Result<(), &'static str> {
        s.status = Status::Preparing;
        emit(&mut s, start, publish);
        let stopped = || control.cancel.check().is_err();
        control.cancel.check().map_err(|_| "cancelled")?;
        job.source.validate().map_err(|_| "source_changed")?;
        let ids = job.source.identities().map_err(|_| "source_changed")?;
        if ids.len() != 1 {
            return Err("unsupported_feature");
        }
        let authority = ArchiveUpdate::open(job.source.display_path(), ids[0]).map_err(output)?;
        job.source.validate().map_err(|_| "source_changed")?;
        workspace = Some(authority.workspace(id).map_err(output)?);
        let w = workspace.as_mut().expect("created workspace");
        let tree = if job.moving.is_none() {
            let tree = InputTree::scan_for_archive(
                &job.options
                    .inputs
                    .iter()
                    .map(PathBuf::from)
                    .collect::<Vec<_>>(),
                vec![w.input_exclusion()],
                &stopped,
            )
            .map_err(|e| e.code())?;
            if tree.contains_identity(authority.version().identity) {
                return Err("protected_source");
            }
            Some(tree)
        } else {
            None
        };
        let mut planner = Planner {
            rows: &job.rows,
            names: BTreeMap::new(),
            namespace_bytes: 0,
            keep: vec![true; job.rows.len()],
            renames: vec![None; job.rows.len()],
            policy: job.options.conflict,
            prompt: 0,
            control,
            snapshot: &mut s,
            start,
            publish,
        };
        for (index, row) in job.rows.iter().enumerate() {
            let name = std::str::from_utf8(&row.raw_name)
                .map_err(|_| "unsafe_name")?
                .trim_end_matches('/');
            PortablePath::parse(name).ok_or("unsafe_name")?;
            insert_name(
                &mut planner.names,
                &mut planner.namespace_bytes,
                name,
                row.directory,
                Some(index),
            )?;
        }
        let mut additions = Additions {
            tree: tree.as_ref(),
            entries: vec![],
            indices: vec![],
        };
        if job.moving.is_some() {
            moving::plan(&job, &mut planner, &mut additions)?;
        } else if let Some(tree) = &tree {
            for (index, entry) in tree.entries().iter().enumerate() {
                control.cancel.check().map_err(|_| "cancelled")?;
                let Some(resolved) = planner.resolve(
                    format!("{}{}", job.prefix, entry.name),
                    entry.directory,
                    entry.size,
                )?
                else {
                    continue;
                };
                if entry.directory && resolved.directory_exists {
                    continue;
                }
                additions.entries.push(EncodeEntry {
                    name: resolved.name,
                    directory: entry.directory,
                    size: entry.size,
                    modified_unix: entry
                        .modified
                        .duration_since(UNIX_EPOCH)
                        .map_or(0, |d| d.as_secs() as i64),
                });
                additions.indices.push(index);
            }
        }
        let Planner { keep, renames, .. } = planner;
        let mut expected: Vec<(Vec<u8>, bool, u64)> = job
            .rows
            .iter()
            .zip(&keep)
            .filter(|(_, k)| **k)
            .map(|(r, _)| {
                (
                    renames[r.id.get() as usize - 1]
                        .as_ref()
                        .map_or_else(|| r.raw_name.clone(), |n| n.as_bytes().to_vec()),
                    r.directory,
                    r.size,
                )
            })
            .collect();
        expected.extend(additions.entries.iter().map(|e| {
            (
                format!("{}{}", e.name, if e.directory { "/" } else { "" }).into_bytes(),
                e.directory,
                e.size,
            )
        }));
        if expected.len() > 100_000
            || expected.iter().map(|e| e.0.len()).sum::<usize>() > 16 * 1024 * 1024
        {
            return Err("resource_limit");
        }
        let mut final_names = BTreeMap::new();
        let mut final_budget = 0usize;
        for (index, (name, directory, _)) in expected.iter().enumerate() {
            let name = std::str::from_utf8(name)
                .map_err(|_| "unsafe_name")?
                .trim_end_matches('/');
            insert_name(
                &mut final_names,
                &mut final_budget,
                name,
                *directory,
                Some(index),
            )?;
        }
        let mut total = 0u64;
        for (_, _, size) in &expected {
            if *size > 8 * 1024 * 1024 * 1024 {
                return Err("resource_limit");
            }
            total = total.checked_add(*size).ok_or("resource_limit")?;
        }
        if total > 20 * 1024 * 1024 * 1024 {
            return Err("resource_limit");
        }
        s.files = expected.iter().filter(|e| !e.1).count() as u64;
        s.declared_bytes = total;
        s.total_known = true;
        s.status = Status::Running;
        emit(&mut s, start, publish);
        let mut stage = w.stage().map_err(output)?;
        let written = std::cell::Cell::new(0);
        let output_error = std::cell::Cell::new(None);
        let mut out = creation::Output {
            stage: &mut stage,
            bytes: &written,
            error: &output_error,
            cancel: &control.cancel,
        };
        codec(job.format)
            .rewrite(
                &mut job.source,
                RewriteInput {
                    keep: &keep,
                    renames: &renames,
                    additions: &additions,
                    settings: EncodeSettings {
                        method: "deflate",
                        level: 6,
                        password: None,
                    },
                },
                &mut out,
                &control.cancel,
                &mut |p| {
                    s.current = p.name.into();
                    s.decoded_bytes = p.input_bytes;
                    emit(&mut s, start, publish);
                },
            )
            .map_err(|e| {
                if stopped() {
                    "cancelled"
                } else if let Some(failure) = output_error.get() {
                    output(failure)
                } else {
                    match e {
                        EncodeError::Source => "source_changed",
                        EncodeError::Output => {
                            output_error.get().map(output).unwrap_or("output_io")
                        }
                        EncodeError::Unsupported => "unsupported_feature",
                        _ => "integrity_error",
                    }
                }
            })?;
        s.written_bytes = written.get();
        s.status = Status::Verifying;
        emit(&mut s, start, publish);
        let mut reader = stage.verification_reader().map_err(output)?;
        let rows = codec(job.format)
            .scan(
                &mut reader,
                &control.cancel,
                crate::formats::limits(job.format),
            )
            .map_err(|_| "integrity_error")?;
        if rows.len() != expected.len() {
            return Err("integrity_error");
        }
        let mut decoder = codec(job.format)
            .open_payload(&mut reader, &rows, &control.cancel)
            .map_err(|_| "integrity_error")?;
        for (row, (name, directory, size)) in rows.iter().zip(&expected) {
            control.cancel.check().map_err(|_| "cancelled")?;
            if &row.raw_name != name || row.directory != *directory || row.size != *size {
                return Err("integrity_error");
            }
            if !directory {
                let mut sink = Verify {
                    bytes: 0,
                    limit: *size,
                };
                decoder
                    .decode(row, &mut sink, &control.cancel)
                    .map_err(|_| "integrity_error")?;
                if sink.bytes != *size {
                    return Err("integrity_error");
                }
                s.verified_files += 1;
                s.verified_bytes += sink.bytes;
            }
        }
        drop(decoder);
        drop(reader);
        if let Some(tree) = &tree {
            tree.validate(&stopped).map_err(|e| e.code())?;
        }
        job.source.validate().map_err(|_| "source_changed")?;
        control.begin_commit()?;
        s.status = Status::Committing;
        emit(&mut s, start, publish);
        let receipt = authority.commit(w, stage).map_err(output)?;
        control.mark_terminal();
        s.cleanup_warning |= receipt.cleanup_warning;
        s.committed_files = s.files;
        s.committed_bytes = total;
        s.output_parts = 1;
        Ok(())
    })();
    s.prompt = None;
    s.status = Status::Cleaning;
    emit(&mut s, start, publish);
    if let Some(w) = workspace {
        s.cleanup_warning |= w.cleanup().is_err();
    }
    s.status = if result.is_ok() {
        Status::Succeeded
    } else if control.cancel.check().is_err() {
        Status::Cancelled
    } else {
        Status::Failed
    };
    s.error = result.err().unwrap_or_default().into();
    control.mark_terminal();
    emit(&mut s, start, publish);
}

#[cfg(test)]
mod tests;
