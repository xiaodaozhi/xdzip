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
                .all(|id| !id.is_empty() && id.len() <= if id.starts_with('q') { 96 } else { 21 })
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
    selection: Selection,
}
impl Moves {
    pub(super) fn bytes(&self) -> usize {
        self.selection.bytes()
    }
}
impl Archive {
    pub(crate) fn move_job(
        &self,
        mut options: MoveOptions,
        cancel: &Cancellation,
    ) -> Result<Job, &'static str> {
        options.selected = self.resolve_query_selectors(&options.selected, cancel)?;
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
        let roots = self.selection_roots(&options.selected, source, cancel)?;
        if roots.iter().any(|(name, directory)| {
            let canonical = key(name);
            *directory && (target_key == canonical || target_key.starts_with(&(canonical + "/")))
        }) {
            return Err("move_selected_directory");
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
            operation: Operation::Move(Moves {
                selection: Selection {
                    roots: roots
                        .into_iter()
                        .map(|(name, dir)| (name.to_owned(), dir))
                        .collect(),
                    source_prefix: source.to_owned(),
                },
            }),
        })
    }
}
pub(super) fn plan(
    job: &Job,
    moves: &Moves,
    planner: &mut Planner<'_>,
    additions: &mut Additions<'_>,
) -> Result<(), &'static str> {
    // Removed source names must not consume the destination namespace budget.
    // Skipped source paths are disjoint from the destination and are rechecked
    // in the final output namespace before staging any rewritten bytes.
    planner
        .names
        .retain(|_, (name, _, _)| moves.selection.root_for(name).is_none());
    planner.namespace_bytes = planner
        .names
        .iter()
        .map(|(key, (name, _, _))| key.len() + name.len())
        .sum();
    let mut merged = Vec::new();
    for (index, row) in job.rows.iter().enumerate() {
        planner.control.cancel.check().map_err(|_| "cancelled")?;
        let name = std::str::from_utf8(row.name_bytes())
            .map_err(|_| "unsafe_name")?
            .trim_end_matches('/');
        let Some(root) = moves.selection.root_for(name) else {
            continue;
        };
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
            let name = std::str::from_utf8(row.name_bytes()).map_err(|_| "unsafe_name")?;
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
                std::str::from_utf8(row.name_bytes())
                    .map_err(|_| "unsafe_name")?
                    .trim_end_matches('/'),
            )
        {
            planner.renames[index] = None;
        }
    }
    moves
        .selection
        .preserve_source_directory(planner, additions);
    Ok(())
}
