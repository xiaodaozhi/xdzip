//! Archive-local entry edits. Selection identities and names grant no host path authority.
use super::*;

/// A same-directory rename or a deletion of the entire selection.
#[derive(Clone, Debug)]
pub enum EntryEdit {
    /// Rename one file or directory using an archive member name.
    Rename(String),
    /// Delete selected entries, including descendants of selected directories.
    Delete,
}

/// Frozen archive-directory selection submitted to the edit transaction.
#[derive(Clone, Debug)]
pub struct EntryEditOptions {
    /// Physical or synthetic IDs, or bounded selectors for the current directory query.
    pub selected: Vec<String>,
    /// Source directory ID, with the empty string denoting the archive root.
    pub directory: String,
    /// The requested entry edit.
    pub action: EntryEdit,
}
impl EntryEditOptions {
    pub(crate) fn bytes(&self) -> usize {
        self.selected
            .iter()
            .map(|id| id.capacity() + 24)
            .sum::<usize>()
            + self.directory.capacity()
            + match &self.action {
                EntryEdit::Rename(name) => name.capacity(),
                EntryEdit::Delete => 0,
            }
    }

    pub(crate) fn bounded(&self) -> bool {
        !self.selected.is_empty()
            && self.selected.len() <= xdzip_codec_api::MAX_ENTRIES
            && self.directory.len() <= 21
            && self
                .selected
                .iter()
                .all(|id| !id.is_empty() && id.len() <= if id.starts_with('q') { 96 } else { 21 })
            && self.bytes() <= 4 * 1024 * 1024
            && match &self.action {
                EntryEdit::Rename(name) => self.selected.len() == 1 && name.len() <= 255,
                EntryEdit::Delete => true,
            }
    }
}

/// Check syntax only. The worker still checks the destination and source identity.
pub fn valid_entry_name(name: &str) -> bool {
    !name.contains('/') && archive_components(name.as_bytes(), false).is_some()
}

pub(super) struct Selection {
    pub(super) roots: BTreeMap<String, bool>,
    pub(super) source_prefix: String,
}
impl Selection {
    pub(super) fn bytes(&self) -> usize {
        self.roots.keys().map(|s| s.capacity() + 64).sum::<usize>() + self.source_prefix.capacity()
    }

    pub(super) fn root_for<'a>(&self, name: &'a str) -> Option<&'a str> {
        if self.roots.contains_key(name) {
            return Some(name);
        }
        name.match_indices('/')
            .map(|(at, _)| &name[..at])
            .find(|parent| self.roots.get(*parent) == Some(&true))
    }

    pub(super) fn preserve_source_directory(
        &self,
        planner: &Planner<'_>,
        additions: &mut Additions<'_>,
    ) {
        let prefix = &self.source_prefix;
        if !prefix.is_empty()
            && !planner.rows.iter().enumerate().any(|(index, row)| {
                planner.keep[index]
                    && planner.renames[index]
                        .as_ref()
                        .map_or(row.name_bytes(), |name| name.as_bytes())
                        .starts_with(prefix.as_bytes())
            })
        {
            additions.entries.push(EncodeEntry {
                name: prefix.trim_end_matches('/').to_owned(),
                directory: true,
                size: 0,
                modified_unix: 0,
            });
        }
    }
}

pub(super) struct Edit {
    selection: Selection,
    // Full archive-relative destination for a rename, already validated on admission.
    destination: Option<String>,
}

impl Archive {
    pub(super) fn selection_roots<'a>(
        &'a self,
        selected: &[String],
        source: &str,
        cancel: &Cancellation,
    ) -> Result<BTreeMap<&'a str, bool>, &'static str> {
        let mut roots = BTreeMap::new();
        let mut bytes = 0usize;
        for id in selected {
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
                (row.name_bytes(), row.directory)
            };
            let name = std::str::from_utf8(name)
                .map_err(|_| "unsafe_name")?
                .trim_end_matches('/');
            archive_components(name.as_bytes(), false).ok_or("unsafe_name")?;
            name.strip_prefix(source)
                .filter(|n| !n.is_empty() && !n.contains('/'))
                .ok_or("invalid_selection")?;
            if let std::collections::btree_map::Entry::Vacant(entry) = roots.entry(name) {
                bytes += name.len();
                if bytes > 4 * 1024 * 1024 {
                    return Err("resource_limit");
                }
                entry.insert(directory);
            }
        }
        Ok(roots)
    }

    pub(crate) fn edit_entries_job(
        &self,
        options: EntryEditOptions,
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
        self.validate_directory_selectors(&options.selected, &options.directory)?;
        let selected = self.resolve_query_selectors(&options.selected, cancel)?;
        let roots = self.selection_roots(&selected, source, cancel)?;
        if roots.is_empty() {
            return Err("invalid_selection");
        }
        let destination = match options.action {
            EntryEdit::Rename(name) => {
                if selected.len() != 1 || roots.len() != 1 {
                    return Err("invalid_selection");
                }
                let full = format!("{source}{name}");
                if !valid_entry_name(&name) || archive_components(full.as_bytes(), false).is_none()
                {
                    return Err("entry_name_invalid");
                }
                if roots.contains_key(full.as_str()) {
                    return Err("entry_name_unchanged");
                }
                Some(full)
            }
            EntryEdit::Delete => None,
        };
        Ok(Job {
            source: self.source.fork().map_err(|_| "source_changed")?,
            rows: Arc::clone(&self.rows),
            format: self.format,
            options: Options {
                inputs: vec![],
                directory: options.directory,
                conflict: ConflictPolicy::Ask,
            },
            prefix: String::new(),
            session: self.key.session.get(),
            operation: Operation::Edit(Edit {
                selection: Selection {
                    roots: roots
                        .into_iter()
                        .map(|(name, dir)| (name.to_owned(), dir))
                        .collect(),
                    source_prefix: source.to_owned(),
                },
                destination,
            }),
        })
    }
}
impl Edit {
    pub(super) fn bytes(&self) -> usize {
        self.selection.bytes() + self.destination.as_ref().map_or(0, String::capacity)
    }

    pub(super) fn kind(&self) -> TaskKind {
        if self.destination.is_some() {
            TaskKind::Rename
        } else {
            TaskKind::Delete
        }
    }

    pub(super) fn plan(
        &self,
        planner: &mut Planner<'_>,
        additions: &mut Additions<'_>,
    ) -> Result<(), &'static str> {
        if self
            .destination
            .as_ref()
            .is_some_and(|name| planner.names.contains_key(name))
        {
            return Err("entry_name_exists");
        }
        for (index, row) in planner.rows.iter().enumerate() {
            planner.control.cancel.check().map_err(|_| "cancelled")?;
            let name = std::str::from_utf8(row.name_bytes())
                .map_err(|_| "unsafe_name")?
                .trim_end_matches('/');
            let Some(root) = self.selection.root_for(name) else {
                continue;
            };
            if let Some(destination) = &self.destination {
                let path = format!("{destination}{}", &name[root.len()..]);
                archive_components(path.as_bytes(), false).ok_or("entry_name_invalid")?;
                planner.renames[index] = Some(if row.directory {
                    format!("{path}/")
                } else {
                    path
                });
            } else {
                planner.keep[index] = false;
            }
        }
        if self.destination.is_none() {
            self.selection.preserve_source_directory(planner, additions);
        }
        Ok(())
    }
}
