// crates/xdzip-runtime/src/updating/moving.rs

//! Frozen archive-only moves. Names never confer filesystem authority.
use super::*;
use std::collections::BTreeSet;

/// Move selected direct children of an immutable archive directory.
#[derive(Clone, Debug)]
pub struct MoveOptions {
    /// Physical or synthetic directory IDs, never paths.
    pub selected: Vec<String>,
    /// Source directory ID; empty denotes the root.
    pub directory: String,
    /// Destination directory ID; empty denotes the root.
    pub target: String,
    /// Ordinary-file name conflict behavior.
    pub conflict: ConflictPolicy,
}
impl MoveOptions {
    pub(crate) fn bounded(&self) -> bool {
        !self.selected.is_empty()
            && self.selected.len() <= 100_000
            && self.directory.len() <= 21
            && self.target.len() <= 21
            && self
                .selected
                .iter()
                .all(|id| !id.is_empty() && id.len() <= 21)
    }
    pub(crate) fn bytes(&self) -> usize {
        self.selected
            .iter()
            .map(|s| s.capacity() + 24)
            .sum::<usize>()
            + self.directory.capacity()
            + self.target.capacity()
    }
}
pub(super) struct Moves {
    roots: BTreeMap<String, bool>,
    source_prefix: String,
}
impl Moves {
    pub(super) fn bytes(&self) -> usize {
        self.roots.keys().map(|s| s.capacity() + 64).sum::<usize>() + self.source_prefix.capacity()
    }
}
impl Archive {
    pub(crate) fn move_job(
        &self,
        options: MoveOptions,
        cancel: &Cancellation,
    ) -> Result<Job, &'static str> {
        if !options.bounded() {
            return Err("invalid_options");
        }
        if !self.can_update() {
            return Err("unsupported_feature");
        }
        let source = std::str::from_utf8(
            self.directory_prefix(&options.directory)
                .map_err(|_| "invalid_selection")?,
        )
        .map_err(|_| "unsafe_name")?;
        let target = std::str::from_utf8(
            self.directory_prefix(&options.target)
                .map_err(|_| "invalid_selection")?,
        )
        .map_err(|_| "unsafe_name")?;
        if source == target {
            return Err("move_same_directory");
        }
        let target_key = key(target.trim_end_matches('/'));
        let mut roots = BTreeMap::new();
        let mut bytes = 0usize;
        for id in &options.selected {
            cancel.check().map_err(|_| "cancelled")?;
            let (name, directory) = if id.starts_with('d') {
                (
                    self.directory_prefix(id).map_err(|_| "invalid_selection")?,
                    true,
                )
            } else {
                let n = id.parse::<u64>().map_err(|_| "invalid_selection")?;
                let row = n
                    .checked_sub(1)
                    .and_then(|n| usize::try_from(n).ok())
                    .and_then(|n| self.rows.get(n))
                    .filter(|r| r.id.get() == n)
                    .ok_or("invalid_selection")?;
                (row.raw_name.as_slice(), row.directory)
            };
            let name = std::str::from_utf8(name)
                .map_err(|_| "unsafe_name")?
                .trim_end_matches('/');
            PortablePath::parse(name).ok_or("unsafe_name")?;
            let relative = name
                .strip_prefix(source)
                .filter(|n| !n.is_empty() && !n.contains('/'))
                .ok_or("invalid_selection")?;
            if relative.is_empty() {
                return Err("invalid_selection");
            }
            let canonical = key(name);
            if directory && (target_key == canonical || target_key.starts_with(&(canonical + "/")))
            {
                return Err("move_selected_directory");
            }
            if !roots.contains_key(name) {
                bytes += name.len();
                if bytes > 4 * 1024 * 1024 {
                    return Err("resource_limit");
                }
                roots.insert(name.to_owned(), directory);
            }
        }
        Ok(Job {
            source: self.source.fork().map_err(|_| "source_changed")?,
            rows: Arc::clone(&self.rows),
            format: self.format,
            options: Options {
                inputs: vec![],
                directory: options.directory,
                conflict: options.conflict,
            },
            prefix: target.to_owned(),
            session: self.key.session.get(),
            moving: Some(Moves {
                roots,
                source_prefix: source.to_owned(),
            }),
        })
    }
}
pub(super) fn plan(
    job: &Job,
    planner: &mut Planner<'_>,
    additions: &mut Additions<'_>,
) -> Result<(), &'static str> {
    let moves = job.moving.as_ref().ok_or("invalid_options")?;
    // Removed source names must not consume the destination namespace budget.
    // Skipped source paths are disjoint from the destination and are rechecked
    // in the final output namespace before staging any rewritten bytes.
    planner.names.retain(|_, (name, _, _)| {
        !moves.roots.contains_key(name)
            && !name
                .match_indices('/')
                .any(|(at, _)| moves.roots.get(&name[..at]) == Some(&true))
    });
    planner.namespace_bytes = planner
        .names
        .iter()
        .map(|(key, (name, _, _))| key.len() + name.len())
        .sum();
    let mut merged = Vec::new();
    for (index, row) in job.rows.iter().enumerate() {
        planner.control.cancel.check().map_err(|_| "cancelled")?;
        let name = std::str::from_utf8(&row.raw_name)
            .map_err(|_| "unsafe_name")?
            .trim_end_matches('/');
        let root = if moves.roots.contains_key(name) {
            Some(name)
        } else {
            name.match_indices('/')
                .map(|(at, _)| &name[..at])
                .find(|p| moves.roots.get(*p) == Some(&true))
        };
        let Some(root) = root else { continue };
        let leaf = root.rsplit('/').next().ok_or("unsafe_name")?;
        let requested = format!("{}{leaf}{}", job.prefix, &name[root.len()..]);
        let Some(resolved) = planner.resolve(requested, row.directory, row.size)? else {
            continue;
        };
        if row.directory && resolved.directory_record {
            planner.keep[index] = false;
            merged.push((index, name));
        } else {
            planner.renames[index] = Some(format!(
                "{}{}",
                resolved.name,
                if row.directory { "/" } else { "" }
            ));
        }
    }
    // A skipped child remains at its source. Preserve its explicit ancestors,
    // including those whose destination was only an implicit directory.
    let mut retained_parents = BTreeSet::new();
    for (index, row) in job.rows.iter().enumerate() {
        if planner.keep[index] && planner.renames[index].is_none() {
            let name = std::str::from_utf8(&row.raw_name).map_err(|_| "unsafe_name")?;
            for (at, _) in name.match_indices('/') {
                retained_parents.insert(&name[..at]);
            }
        }
    }
    for (index, name) in merged {
        if retained_parents.contains(name) {
            planner.keep[index] = true;
        }
    }
    for (index, row) in job.rows.iter().enumerate() {
        if row.directory
            && planner.renames[index].is_some()
            && retained_parents.contains(
                std::str::from_utf8(&row.raw_name)
                    .map_err(|_| "unsafe_name")?
                    .trim_end_matches('/'),
            )
        {
            planner.renames[index] = None;
        }
    }
    if !moves.source_prefix.is_empty()
        && !job.rows.iter().enumerate().any(|(index, row)| {
            planner.keep[index]
                && planner.renames[index]
                    .as_ref()
                    .map_or(row.raw_name.as_slice(), |name| name.as_bytes())
                    .starts_with(moves.source_prefix.as_bytes())
        })
    {
        additions.entries.push(EncodeEntry {
            name: moves.source_prefix.trim_end_matches('/').to_owned(),
            directory: true,
            size: 0,
            modified_unix: 0,
        });
    }
    Ok(())
}
