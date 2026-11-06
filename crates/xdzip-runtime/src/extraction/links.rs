//! ZIP symbolic links are data until every target resolves inside the selected output graph.
use super::plan::Planned;
use crate::Archive;
use std::collections::BTreeMap;
use xdzip_codec_api::{
    Cancellation,
    payload::{DecodeError, DecodedSink},
};
use xdzip_domain::path::PortablePath;

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
/// They are resolved against the link's parent below, then the resulting path
/// is checked against the frozen archive graph.
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
        if PortablePath::parse(part).is_none() {
            return Err("unsupported_link");
        }
    }
    Ok(())
}

fn normalize_target(base: &[String], target: &str) -> Result<Vec<String>, &'static str> {
    validate_target(target)?;
    let mut parts = base.to_vec();
    for component in target.split('/') {
        match component {
            "." => {}
            ".." => {
                parts.pop().ok_or("unsupported_link")?;
            }
            component => parts.push(component.to_owned()),
        }
        if parts.len() > 64 || parts.iter().map(String::len).sum::<usize>() + parts.len() > 4096 {
            return Err("resource_limit");
        }
    }
    Ok(parts)
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
            .filter(|n| *n <= 4 * 1024 * 1024)
            .ok_or("resource_limit")?;
        let target = String::from_utf8(bytes).map_err(|_| "unsupported_link")?;
        // Standard macOS bundles use parent-relative targets. They are safe only
        // after normalization against the link parent and graph resolution below.
        validate_target(&target)?;
        p.link_target = Some(target);
    }
    // Exact spelling is required: aliases/duplicate records cannot silently retarget a link.
    let mut nodes: BTreeMap<String, (bool, Option<&str>)> = BTreeMap::new();
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
        nodes.insert(name, (rows[p.index].directory, p.link_target.as_deref()));
    }
    for p in planned.iter().filter(|p| p.link_target.is_some()) {
        resolve(&p.path, &nodes, cancel)?;
    }
    Ok(())
}

/// Resolve link prefixes as well as complete targets (Framework/Versions/Current/A).
fn resolve(
    path: &PortablePath,
    nodes: &BTreeMap<String, (bool, Option<&str>)>,
    cancel: &Cancellation,
) -> Result<(), &'static str> {
    let mut parts = path.parts().to_vec();
    let mut followed = 0;
    loop {
        cancel.check().map_err(|_| "cancelled")?;
        let mut replaced = false;
        for at in 0..parts.len() {
            let name = parts[..=at].join("/");
            let (directory, target) = nodes.get(&name).ok_or("unsupported_link")?;
            if let Some(target) = target {
                followed += 1;
                if followed > 64 {
                    return Err("unsupported_link");
                }
                let mut next = normalize_target(&parts[..at], target)?;
                next.extend_from_slice(&parts[at + 1..]);
                if next.len() > 64
                    || next.iter().map(String::len).sum::<usize>() + next.len() > 4096
                {
                    return Err("resource_limit");
                }
                parts = next;
                replaced = true;
                break;
            }
            if at + 1 < parts.len() && !directory {
                return Err("unsupported_link");
            }
        }
        if !replaced {
            return Ok(());
        }
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
}
