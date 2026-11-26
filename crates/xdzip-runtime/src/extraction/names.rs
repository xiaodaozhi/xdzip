use super::{Snapshot, plan};
use std::collections::{BTreeMap, BTreeSet};
use xdzip_codec_api::Cancellation;
use xdzip_domain::path::{DESTINATION_PUNCTUATION, PortablePath};
use xdzip_io::{ExistingFile, FileIdentity, Grant, OutputError};

/// Keeps archive paths distinct until the destination accepts or rejects a name.
pub(super) struct Names {
    originals: BTreeSet<String>,
    directories: BTreeMap<String, (PortablePath, FileIdentity)>,
}

impl Names {
    pub(super) fn new(job: &plan::Job, cancel: &Cancellation) -> Result<Self, &'static str> {
        let mut originals = BTreeSet::new();
        let mut bytes = 0usize;
        for item in &job.planned {
            cancel.check().map_err(|_| "cancelled")?;
            let mut path = String::new();
            for part in item.path.parts() {
                push_part(&mut path, part);
                let key = plan::key(&path);
                if !originals.contains(&key) {
                    bytes += key.len();
                    if bytes > 8 * 1024 * 1024 {
                        return Err("resource_limit");
                    }
                    originals.insert(key);
                }
            }
        }
        Ok(Self {
            originals,
            directories: BTreeMap::new(),
        })
    }

    pub(super) fn resolve(
        &mut self,
        target: &Grant,
        original: &PortablePath,
        directory: bool,
        cancel: &Cancellation,
        stats: &mut Snapshot,
    ) -> Result<PortablePath, &'static str> {
        let mut source = String::new();
        let mut output_parent = String::new();
        let mut parent_identity = None;
        for (at, leaf) in original.parts().iter().enumerate() {
            cancel.check().map_err(|_| "cancelled")?;
            push_part(&mut source, leaf);
            let last = at + 1 == original.parts().len();
            if last && !directory {
                return child_path(&output_parent, leaf);
            }
            if let Some((resolved, identity)) = self.directories.get(&source) {
                if last {
                    target
                        .check_claimed_directory(resolved, *identity)
                        .map_err(super::execute::output)?;
                    return Ok(resolved.clone());
                }
                output_parent = resolved.display();
                parent_identity = Some(*identity);
                continue;
            }
            let candidate = child_path(&output_parent, leaf)?;
            let claimed = if win32_rejects(&candidate) {
                Err(OutputError::NameRejected)
            } else {
                target.claim_directory(&candidate, parent_identity)
            };
            let free = |path: &PortablePath| match target.claim_directory(path, parent_identity) {
                Ok(identity) => Ok(Some(identity)),
                Err(OutputError::Conflict) => Ok(None),
                Err(error) => Err(error),
            };
            let (path, identity) = match claimed {
                Ok(identity) => (candidate, identity),
                Err(OutputError::NameRejected) => {
                    self.sanitized(&source, &candidate, true, cancel, free)?
                }
                Err(OutputError::Conflict) => {
                    if target
                        .leaf_metadata(&candidate)
                        .map_err(super::execute::output)?
                        .is_some_and(|kind| !kind.is_dir() && !kind.is_file())
                    {
                        return Err("unsafe_target");
                    }
                    match self.available(&source, &candidate, true, 1, cancel, free) {
                        Err("unsafe_name") if punctuation(&candidate) => {
                            self.sanitized(&source, &candidate, true, cancel, free)?
                        }
                        result => result?,
                    }
                }
                Err(error) => return Err(super::execute::output(error)),
            };
            output_parent = path.display();
            if source != output_parent {
                stats.record_mapping(&source, &output_parent, 0);
            }
            self.directories.insert(source.clone(), (path, identity));
            parent_identity = Some(identity);
        }
        PortablePath::parse_extraction(&output_parent).ok_or("unsafe_name")
    }

    pub(super) fn parent_identity(&self, path: &PortablePath) -> Option<FileIdentity> {
        let name = path.display();
        let (parent, _) = name.rsplit_once('/')?;
        self.directories.get(parent).map(|(_, identity)| *identity)
    }

    pub(super) fn fresh_file(
        &self,
        target: &Grant,
        original: &PortablePath,
        candidate: &PortablePath,
        cancel: &Cancellation,
    ) -> Result<PortablePath, &'static str> {
        let available = self.available(&original.display(), candidate, false, 1, cancel, |path| {
            Ok(target.leaf_metadata(path)?.is_none().then_some(()))
        });
        match available {
            Err("unsafe_name") if punctuation(candidate) => {
                self.replacement_file(target, original, candidate, cancel)
            }
            result => result.map(|(path, ())| path),
        }
    }

    /// Select an ordinary leaf. A forced replacement begins as a no-overwrite candidate.
    pub(super) fn file_slot(
        &self,
        target: &Grant,
        original: &PortablePath,
        candidate: PortablePath,
        parent: Option<FileIdentity>,
        cancel: &Cancellation,
    ) -> Result<(PortablePath, Option<ExistingFile>), &'static str> {
        let kind = if win32_rejects(&candidate) {
            Err(OutputError::NameRejected)
        } else {
            target.leaf_metadata(&candidate)
        };
        let existing = match kind {
            Ok(Some(kind)) if kind.is_dir() => {
                return Ok((self.fresh_file(target, original, &candidate, cancel)?, None));
            }
            Ok(Some(_)) => target.existing_with_parent(&candidate, parent),
            Ok(None) => Ok(None),
            Err(error) => Err(error),
        };
        match existing {
            Err(OutputError::NameRejected) => Ok((
                self.replacement_file(target, original, &candidate, cancel)?,
                None,
            )),
            result => Ok((candidate, result.map_err(super::execute::output)?)),
        }
    }

    pub(super) fn replacement_file(
        &self,
        target: &Grant,
        original: &PortablePath,
        candidate: &PortablePath,
        cancel: &Cancellation,
    ) -> Result<PortablePath, &'static str> {
        self.sanitized(&original.display(), candidate, false, cancel, |path| {
            Ok(target.leaf_metadata(path)?.is_none().then_some(()))
        })
        .map(|(path, ())| path)
    }

    fn sanitized<T>(
        &self,
        source: &str,
        path: &PortablePath,
        directory: bool,
        cancel: &Cancellation,
        try_name: impl FnMut(&PortablePath) -> Result<Option<T>, OutputError>,
    ) -> Result<(PortablePath, T), &'static str> {
        let (leaf, parent) = path.parts().split_last().expect("nonempty extraction path");
        let safe_leaf = leaf.replace(DESTINATION_PUNCTUATION, "_");
        let safe_path = child_path(&parent.join("/"), &safe_leaf)?;
        self.available(source, &safe_path, directory, 0, cancel, try_name)
    }

    fn available<T>(
        &self,
        source: &str,
        path: &PortablePath,
        directory: bool,
        first: usize,
        cancel: &Cancellation,
        mut try_name: impl FnMut(&PortablePath) -> Result<Option<T>, OutputError>,
    ) -> Result<(PortablePath, T), &'static str> {
        let original_parent = source.rsplit_once('/').map_or("", |(parent, _)| parent);
        let parts = path.parts();
        let output_parent = parts[..parts.len() - 1].join("/");
        let leaf = parts.last().expect("nonempty portable path");
        let available = 4096usize
            .saturating_sub(output_parent.len() + usize::from(!output_parent.is_empty()))
            .min(255);
        for index in first..=10_000 {
            cancel.check().map_err(|_| "cancelled")?;
            let name = if index == 0 {
                leaf.clone()
            } else {
                numbered(leaf, directory, available, index)?
            };
            let mut reserved = original_parent.to_owned();
            push_part(&mut reserved, &name);
            // Folding only keeps a generated suffix from stealing a later archive name.
            if self.originals.contains(&plan::key(&reserved)) {
                continue;
            }
            let path = child_path(&output_parent, &name)?;
            if let Some(value) = try_name(&path).map_err(super::execute::output)? {
                return Ok((path, value));
            }
        }
        Err("rename_limit")
    }
}

fn numbered(
    leaf: &str,
    directory: bool,
    available: usize,
    index: usize,
) -> Result<String, &'static str> {
    let suffix = format!(" ({index})");
    if available <= suffix.len() {
        return Err("rename_limit");
    }
    let (stem, extension) = leaf
        .rfind('.')
        .filter(|dot| !directory && *dot > 0 && leaf.len() - dot + suffix.len() < available)
        .map_or((leaf, ""), |dot| leaf.split_at(dot));
    let end =
        stem.floor_char_boundary((available - suffix.len() - extension.len()).min(stem.len()));
    Ok(format!("{}{suffix}{extension}", &stem[..end]))
}

pub(super) fn punctuation(path: &PortablePath) -> bool {
    path.parts()
        .last()
        .is_some_and(|leaf| leaf.contains(DESTINATION_PUNCTUATION))
}

fn win32_rejects(path: &PortablePath) -> bool {
    cfg!(windows) && punctuation(path)
}

fn push_part(path: &mut String, component: &str) {
    if !path.is_empty() {
        path.push('/');
    }
    path.push_str(component);
}

fn child_path(parent: &str, leaf: &str) -> Result<PortablePath, &'static str> {
    let mut path = parent.to_owned();
    push_part(&mut path, leaf);
    PortablePath::parse_extraction(&path).ok_or("unsafe_name")
}
