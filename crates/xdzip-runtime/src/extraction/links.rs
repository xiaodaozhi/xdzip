//! ZIP symbolic links are data until every target resolves inside the selected output graph.
use super::plan::Planned;
use crate::Archive;
use std::collections::{BTreeMap, VecDeque};
use xdzip_codec_api::{
    Cancellation,
    payload::{DecodeError, DecodedSink},
};
use xdzip_domain::path::PortablePath;

type Nodes<T> = BTreeMap<String, (bool, Option<T>)>;
const MAX_TARGET_BYTES: usize = 4 * 1024 * 1024;

pub(super) struct LinkTarget {
    original: String,
    referent: String,
    followed: usize,
}

impl LinkTarget {
    pub(super) fn bytes(&self) -> usize {
        self.original.capacity() + self.referent.capacity()
    }
}

struct Target(Vec<u8>);
impl DecodedSink for Target {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self.0.len().saturating_add(bytes.len()) > 4096 {
            return Err(DecodeError::Limit);
        }
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

/// Validate a ZIP link target without treating `.` and `..` as output names.
/// They are resolved in filesystem order against the frozen archive graph below.
pub(super) fn validate_target(target: &str) -> Result<(), &'static str> {
    if target.is_empty() || target.len() > 4096 || target.starts_with('/') || target.contains('\\')
    {
        return Err(if target.len() > 4096 {
            "resource_limit"
        } else {
            "unsupported_link"
        });
    }
    let parts: Vec<_> = target.split('/').collect();
    if parts.len() > 64 {
        return Err("resource_limit");
    }
    for part in parts {
        if matches!(part, "." | "..") {
            continue;
        }
        if PortablePath::parse_extraction(part).is_none() {
            return Err("unsupported_link");
        }
    }
    Ok(())
}

pub(super) fn prepare(
    archive: &Archive,
    planned: &mut [Planned],
    cancel: &Cancellation,
) -> Result<(), &'static str> {
    if !planned.iter().any(|p| archive.rows[p.index].link) {
        return Ok(());
    }
    let mut source = archive.source.fork().map_err(|_| "source_changed")?;
    let mut decoder = crate::formats::codec(archive.format)
        .open_payload(source.file(), &archive.rows, cancel)
        .map_err(|e| match e {
            DecodeError::Cancelled => "cancelled",
            _ => "integrity_error",
        })?;
    prepare_decoded(&archive.rows, planned, cancel, |entry| {
        let mut sink = Target(Vec::new());
        let decoded = decoder.decode(entry, &mut sink, cancel);
        archive.source.validate().map_err(|_| "source_changed")?;
        decoded.map_err(|e| match e {
            DecodeError::Cancelled => "cancelled",
            DecodeError::Limit => "resource_limit",
            _ => "integrity_error",
        })?;
        Ok(sink.0)
    })
}

/// Run an authenticated payload callback before validating the complete selected link graph.
pub(super) fn prepare_decoded(
    rows: &[xdzip_codec_api::Entry],
    planned: &mut [Planned],
    cancel: &Cancellation,
    mut decode: impl FnMut(&xdzip_codec_api::Entry) -> Result<Vec<u8>, &'static str>,
) -> Result<(), &'static str> {
    let mut total = 0usize;
    for p in planned.iter_mut().filter(|p| rows[p.index].link) {
        cancel.check().map_err(|_| "cancelled")?;
        let entry = &rows[p.index];
        if entry.directory || entry.size == 0 || entry.size > 4096 {
            return Err("unsupported_link");
        }
        let bytes = decode(entry)?;
        total = total
            .checked_add(bytes.len())
            .filter(|n| *n <= MAX_TARGET_BYTES)
            .ok_or("resource_limit")?;
        let target = String::from_utf8(bytes).map_err(|_| "unsupported_link")?;
        // Standard macOS bundles use parent-relative targets. Resolve them with
        // every link prefix in filesystem order before admitting any output.
        validate_target(&target)?;
        p.link_target = Some(LinkTarget {
            original: target,
            referent: String::new(),
            followed: 0,
        });
    }
    // Exact spelling is required: aliases/duplicate records cannot silently retarget a link.
    let mut nodes: Nodes<String> = BTreeMap::new();
    let mut explicit = std::collections::BTreeSet::new();
    for p in planned.iter() {
        let name = p.path.display();
        if !explicit.insert(name.clone()) {
            return Err("ambiguous_name");
        }
        for at in 1..p.path.parts().len() {
            nodes
                .entry(p.path.parts()[..at].join("/"))
                .or_insert((true, None));
        }
        nodes.insert(
            name,
            (
                rows[p.index].directory,
                p.link_target.as_ref().map(|t| t.original.clone()),
            ),
        );
    }
    let mut referent_bytes = 0usize;
    for p in planned {
        if let Some(link) = &mut p.link_target {
            (link.referent, link.followed) = resolve(&p.path, &nodes, cancel)?;
            referent_bytes += link.referent.len();
            if referent_bytes > MAX_TARGET_BYTES {
                return Err("resource_limit");
            }
        }
    }
    Ok(())
}

/// Resolve link prefixes as well as complete targets (Framework/Versions/Current/A).
fn resolve<T: AsRef<str>>(
    path: &PortablePath,
    nodes: &Nodes<T>,
    cancel: &Cancellation,
) -> Result<(String, usize), &'static str> {
    walk(
        Vec::new(),
        path.parts().iter().cloned().collect(),
        0,
        nodes,
        cancel,
    )
}

fn walk<T: AsRef<str>>(
    mut resolved: Vec<String>,
    mut pending: VecDeque<String>,
    mut followed: usize,
    nodes: &Nodes<T>,
    cancel: &Cancellation,
) -> Result<(String, usize), &'static str> {
    loop {
        cancel.check().map_err(|_| "cancelled")?;
        let count = resolved.len() + pending.len();
        if count > 64
            || resolved
                .iter()
                .chain(pending.iter())
                .map(String::len)
                .sum::<usize>()
                + count
                > 4096
        {
            return Err("resource_limit");
        }
        let Some(component) = pending.pop_front() else {
            break;
        };
        match component.as_str() {
            "." => {}
            ".." => {
                resolved.pop().ok_or("unsupported_link")?;
            }
            _ => {
                let name = resolved
                    .iter()
                    .map(String::as_str)
                    .chain([component.as_str()])
                    .collect::<Vec<_>>()
                    .join("/");
                let (directory, target) = nodes.get(&name).ok_or("unsupported_link")?;
                if let Some(target) = target {
                    let target = target.as_ref();
                    followed += 1;
                    if followed > 64 {
                        return Err("unsupported_link");
                    }
                    validate_target(target)?;
                    for part in target.split('/').rev() {
                        pending.push_front(part.to_owned());
                    }
                } else {
                    if !pending.is_empty() && !directory {
                        return Err("unsupported_link");
                    }
                    resolved.push(component);
                }
            }
        }
    }
    Ok((resolved.join("/"), followed))
}

impl LinkTarget {
    /// Every link encountered while resolving this link has a smaller count.
    pub(super) fn order(&self) -> usize {
        self.followed
    }
}

/// Only objects published by this task can satisfy a link in the output graph.
#[derive(Default)]
pub(super) struct Output {
    assigned: BTreeMap<String, String>,
    nodes: Nodes<String>,
    path_bytes: usize,
    target_bytes: usize,
}

impl Output {
    pub(super) fn parents(
        &mut self,
        original: &PortablePath,
        path: &PortablePath,
    ) -> Result<(), &'static str> {
        for at in 1..path.parts().len() {
            self.record(
                original.parts()[..at].join("/"),
                path.parts()[..at].join("/"),
                true,
                None,
            )?;
        }
        Ok(())
    }

    pub(super) fn entry(
        &mut self,
        original: &PortablePath,
        path: &PortablePath,
        directory: bool,
        link: Option<String>,
    ) -> Result<(), &'static str> {
        self.parents(original, path)?;
        self.record(original.display(), path.display(), directory, link)
    }

    fn record(
        &mut self,
        original: String,
        path: String,
        directory: bool,
        link: Option<String>,
    ) -> Result<(), &'static str> {
        if let Some(previous) = self.assigned.get(&original) {
            if previous == &path && directory && self.nodes.get(&path).is_some_and(|node| node.0) {
                return Ok(());
            }
            return Err("ambiguous_name");
        }
        if self.nodes.contains_key(&path) {
            return Err("ambiguous_name");
        }
        self.path_bytes += original.len() + 2 * path.len();
        if self.path_bytes > 24 * 1024 * 1024 {
            return Err("resource_limit");
        }
        self.nodes.insert(path.clone(), (directory, link));
        self.assigned.insert(original, path);
        Ok(())
    }

    pub(super) fn target(
        &mut self,
        link: &LinkTarget,
        path: &PortablePath,
        cancel: &Cancellation,
    ) -> Result<String, &'static str> {
        let wanted = if link.referent.is_empty() {
            ""
        } else {
            self.assigned
                .get(&link.referent)
                .ok_or("unsupported_link")?
        };
        let parent = &path.parts()[..path.parts().len() - 1];
        let resolve = |target: &str| {
            validate_target(target)?;
            walk(
                parent.to_vec(),
                target.split('/').map(str::to_owned).collect(),
                1,
                &self.nodes,
                cancel,
            )
        };
        let target = match resolve(&link.original) {
            Ok((resolved, _)) if resolved == wanted => link.original.clone(),
            Err("cancelled") => return Err("cancelled"),
            _ => {
                let wanted_parts: Vec<_> = wanted.split('/').filter(|n| !n.is_empty()).collect();
                let shared = parent
                    .iter()
                    .zip(&wanted_parts)
                    .take_while(|(a, b)| a.as_str() == **b)
                    .count();
                let relative = std::iter::repeat_n("..", parent.len() - shared)
                    .chain(wanted_parts[shared..].iter().copied())
                    .collect::<Vec<_>>()
                    .join("/");
                let relative = if relative.is_empty() {
                    ".".into()
                } else {
                    relative
                };
                if resolve(&relative)?.0 != wanted {
                    return Err("unsupported_link");
                }
                relative
            }
        };
        self.target_bytes += target.len();
        if self.target_bytes > MAX_TARGET_BYTES {
            return Err("resource_limit");
        }
        Ok(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn resolution_bounds_chains_and_rejects_a_file_used_as_directory() {
        let cancel = Cancellation::default();
        let targets: Vec<_> = (1..=65).map(|n| format!("l{n}")).collect();
        let mut nodes = BTreeMap::new();
        nodes.insert("l64".into(), (false, None));
        for (n, target) in targets.iter().take(64).enumerate() {
            nodes.insert(format!("l{n}"), (false, Some(target.as_str())));
        }
        assert!(resolve(&PortablePath::parse("l0").unwrap(), &nodes, &cancel).is_ok());
        nodes.insert("l64".into(), (false, Some(targets[64].as_str())));
        nodes.insert("l65".into(), (false, None));
        assert_eq!(
            resolve(&PortablePath::parse("l0").unwrap(), &nodes, &cancel),
            Err("unsupported_link")
        );
        nodes.insert("link".into(), (false, Some("l65/child")));
        assert_eq!(
            resolve(&PortablePath::parse("link").unwrap(), &nodes, &cancel),
            Err("unsupported_link")
        );
        cancel.cancel();
        assert_eq!(
            resolve(&PortablePath::parse("l65").unwrap(), &nodes, &cancel),
            Err("cancelled")
        );
    }

    #[test]
    fn parent_relative_bundle_targets_are_normalized_inside_the_archive_graph() {
        let cancel = Cancellation::default();
        let mut nodes = BTreeMap::new();
        for directory in [
            "Demo.app",
            "Demo.app/Contents",
            "Demo.app/Contents/PlugIns",
            "Demo.app/Contents/PlugIns/quick",
            "Demo.app/Contents/Resources",
            "Demo.app/Contents/Resources/qml",
            "Demo.app/Contents/Resources/qml/QtQml",
        ] {
            nodes.insert(directory.into(), (true, None));
        }
        nodes.insert(
            "Demo.app/Contents/PlugIns/quick/plugin".into(),
            (false, None),
        );
        nodes.insert(
            "Demo.app/Contents/Resources/qml/QtQml/plugin".into(),
            (false, Some("../../../PlugIns/quick/plugin")),
        );
        let link = PortablePath::parse("Demo.app/Contents/Resources/qml/QtQml/plugin").unwrap();
        assert!(resolve(&link, &nodes, &cancel).is_ok());

        nodes.insert(
            "Demo.app/Contents/Resources/qml/QtQml/escape".into(),
            (false, Some("../../../../../../outside")),
        );
        let escape = PortablePath::parse("Demo.app/Contents/Resources/qml/QtQml/escape").unwrap();
        assert_eq!(resolve(&escape, &nodes, &cancel), Err("unsupported_link"));
    }

    #[test]
    fn parent_after_a_link_is_resolved_after_following_that_link() {
        let cancel = Cancellation::default();
        let mut nodes = BTreeMap::new();
        nodes.insert("dir".into(), (true, None));
        nodes.insert("dir/alias".into(), (false, Some("..")));
        nodes.insert("dir/link".into(), (false, Some("alias/../../outside")));
        nodes.insert("outside".into(), (false, None));
        assert_eq!(
            resolve(&PortablePath::parse("dir/link").unwrap(), &nodes, &cancel),
            Err("unsupported_link")
        );

        nodes.insert("dir/alias".into(), (false, Some("subdir")));
        nodes.insert("dir/subdir".into(), (true, None));
        nodes.insert("dir/link".into(), (false, Some("alias/../file")));
        nodes.insert("dir/file".into(), (false, None));
        assert_eq!(
            resolve(&PortablePath::parse("dir/link").unwrap(), &nodes, &cancel),
            Ok(("dir/file".into(), 2))
        );

        for target in ["file/../file", "missing/../file", "file/."] {
            nodes.insert("dir/link".into(), (false, Some(target)));
            assert_eq!(
                resolve(&PortablePath::parse("dir/link").unwrap(), &nodes, &cancel),
                Err("unsupported_link"),
                "{target}"
            );
        }
    }
}
