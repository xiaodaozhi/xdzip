use super::{Snapshot, plan};
use std::collections::{BTreeMap, BTreeSet};
use xdzip_codec_api::Cancellation;
use xdzip_domain::path::PortablePath;
use xdzip_io::{FileIdentity, Grant, OutputError};

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
            let (path, identity) = match target.claim_directory(&candidate, parent_identity) {
                Ok(identity) => (candidate, identity),
                Err(OutputError::Conflict) => {
                    if target
                        .leaf_metadata(&candidate)
                        .map_err(super::execute::output)?
                        .is_some_and(|kind| !kind.is_dir() && !kind.is_file())
                    {
                        return Err("unsafe_target");
                    }
                    self.suffixed(&source, &candidate, true, cancel, |path| {
                        match target.claim_directory(path, parent_identity) {
                            Ok(identity) => Ok(Some(identity)),
                            Err(OutputError::Conflict) => Ok(None),
                            Err(error) => Err(error),
                        }
                    })?
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
        PortablePath::parse(&output_parent).ok_or("unsafe_name")
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
        self.suffixed(&original.display(), candidate, false, cancel, |path| {
            Ok(target.leaf_metadata(path)?.is_none().then_some(()))
        })
        .map(|(path, ())| path)
    }

    fn suffixed<T>(
        &self,
        source: &str,
        path: &PortablePath,
        directory: bool,
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
        for index in 1..=10_000 {
            cancel.check().map_err(|_| "cancelled")?;
            let suffix = format!(" ({index})");
            if available <= suffix.len() {
                return Err("rename_limit");
            }
            let (stem, extension) = leaf
                .rfind('.')
                .filter(|dot| !directory && *dot > 0 && leaf.len() - dot + suffix.len() < available)
                .map_or((leaf.as_str(), ""), |dot| leaf.split_at(dot));
            let end = stem
                .floor_char_boundary((available - suffix.len() - extension.len()).min(stem.len()));
            let name = format!("{}{suffix}{extension}", &stem[..end]);
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

fn push_part(path: &mut String, component: &str) {
    if !path.is_empty() {
        path.push('/');
    }
    path.push_str(component);
}

fn child_path(parent: &str, leaf: &str) -> Result<PortablePath, &'static str> {
    let mut path = parent.to_owned();
    push_part(&mut path, leaf);
    PortablePath::parse(&path).ok_or("unsafe_name")
}
