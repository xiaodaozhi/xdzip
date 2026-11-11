//! Headless physical metadata and exact member selection, bound to one source snapshot.
use crate::Archive;
use std::collections::{BTreeMap, BTreeSet};
use xdzip_codec_api::{Cancellation, ScanError};
use xdzip_domain::path::PortablePath;

/// Owned physical metadata. A path is a selector, never output authority.
#[derive(Clone, Debug)]
pub struct PhysicalEntry {
    /// Physical identity, including duplicate names.
    pub id: u64,
    /// Effective decoded spelling, absent for unresolved or unsafe names.
    pub path: Option<String>,
    /// Escaped text safe to print in a terminal.
    pub display_name: String,
    /// Declared uncompressed size, distinct from unknown.
    pub size: Option<u64>,
    /// Declared compressed size.
    pub packed_size: u64,
    /// Container-specific method identifier.
    pub method: u16,
    /// Explicit directory record.
    pub directory: bool,
    /// Declared link record, with output admission decided by the task planner.
    pub link: bool,
    /// The format can decode this record under its supported profile.
    pub payload_supported: bool,
    /// Content encryption declaration.
    pub encrypted: bool,
    /// Suspicious name or unsupported record.
    pub warning: bool,
}

fn path(entry: &xdzip_codec_api::Entry) -> Option<String> {
    xdzip_domain::path::components(entry.name_bytes(), entry.directory)?;
    String::from_utf8(entry.name_bytes().to_vec()).ok()
}

impl Archive {
    /// Return at most 128 physical records after validating the frozen source.
    pub fn catalog(
        &self,
        offset: u64,
        limit: u64,
        control: &Cancellation,
    ) -> Result<Vec<PhysicalEntry>, ScanError> {
        if !(1..=128).contains(&limit) {
            return Err(ScanError::Limit);
        }
        control.check()?;
        self.source.validate()?;
        let owned;
        let page = if let Some(disk) = &self.disk {
            if offset > self.len() as u64 {
                return Err(ScanError::Limit);
            }
            owned = (offset..(offset + limit).min(self.len() as u64))
                .map(|at| disk.get(at + 1))
                .collect::<Result<Vec<_>, _>>()?;
            &owned[..]
        } else {
            self.page(offset, limit)?
        };
        let result = page
            .iter()
            .map(|entry| PhysicalEntry {
                id: entry.id.get(),
                path: path(entry),
                display_name: entry.display_name.clone(),
                size: entry.size_known.then_some(entry.size),
                packed_size: entry.packed_size,
                method: entry.method,
                directory: entry.directory,
                link: entry.link,
                payload_supported: entry.payload_supported(),
                encrypted: entry.encrypted,
                warning: entry.warning,
            })
            .collect();
        control.check()?;
        self.source.validate()?;
        Ok(result)
    }

    /// Resolve exact member paths and physical IDs without touching any target.
    /// Duplicate names require IDs. Directory descendants are expanded by the
    /// existing extraction planner, with its unresolved-name and collision rules.
    pub fn resolve_members(
        &self,
        members: &[String],
        entry_ids: &[u64],
        control: &Cancellation,
    ) -> Result<Vec<String>, &'static str> {
        if members.len().saturating_add(entry_ids.len()) > xdzip_codec_api::MAX_ENTRIES
            || members.iter().map(String::len).sum::<usize>() > 4 * 1024 * 1024
        {
            return Err("resource_limit");
        }
        self.source.validate().map_err(|_| "source_changed")?;
        let mut by_path: BTreeMap<String, Vec<u64>> = BTreeMap::new();
        let requested: BTreeSet<&str> = members
            .iter()
            .map(|name| name.strip_suffix('/').unwrap_or(name))
            .collect();
        let mut visit = |entry: &xdzip_codec_api::Entry| -> Result<(), ScanError> {
            control.check()?;
            if let Some(name) = path(entry)
                && requested.contains(name.trim_end_matches('/'))
            {
                let ids = by_path
                    .entry(name.trim_end_matches('/').to_owned())
                    .or_default();
                if ids.len() < 2 {
                    ids.push(entry.id.get());
                }
            }
            Ok(())
        };
        if !members.is_empty() {
            if let Some(disk) = &self.disk {
                disk.each(control, &mut visit)
                    .map_err(|_| "source_changed")?;
            } else {
                for entry in self.rows.iter() {
                    visit(entry).map_err(|_| "cancelled")?;
                }
            }
        }
        let mut selected = BTreeSet::new();
        for member in members {
            let name = member.strip_suffix('/').unwrap_or(member);
            if PortablePath::parse(name).is_none() {
                return Err("invalid_member_path");
            }
            if let Some(ids) = by_path.get(name) {
                if ids.len() != 1 {
                    return Err("ambiguous_member");
                }
                selected.insert(ids[0].to_string());
            } else {
                selected.insert(self.resolve_directory(name, control).map_err(|e| {
                    if e == "directory_not_found" {
                        "member_not_found"
                    } else {
                        e
                    }
                })?);
            }
        }
        for &id in entry_ids {
            let found = self.entry(id).is_ok();
            if !found {
                return Err("invalid_entry_id");
            }
            selected.insert(id.to_string());
        }
        control.check().map_err(|_| "cancelled")?;
        self.source.validate().map_err(|_| "source_changed")?;
        Ok(selected.into_iter().collect())
    }

    /// Resolve an existing safe archive directory; an empty path denotes root.
    pub fn resolve_directory(
        &self,
        path: &str,
        control: &Cancellation,
    ) -> Result<String, &'static str> {
        control.check().map_err(|_| "cancelled")?;
        self.source.validate().map_err(|_| "source_changed")?;
        if path.is_empty() {
            return Ok(String::new());
        }
        let name = path.strip_suffix('/').unwrap_or(path);
        if PortablePath::parse(name).is_none() {
            return Err("invalid_member_path");
        }
        let prefix = format!("{name}/");
        let nodes = self.directories.as_ref().ok_or("navigation_limit")?;
        nodes
            .iter()
            .find(|node| node.path == prefix)
            .map(|node| format!("d{}", node.id))
            .ok_or("directory_not_found")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn archive() -> Archive {
        Archive::open(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/zip/normal.zip"),
            &Cancellation::default(),
        )
        .unwrap()
    }
    #[test]
    fn physical_pages_and_exact_selectors_preserve_duplicates_and_unicode() {
        let archive = archive();
        let control = Cancellation::default();
        let page = archive.catalog(0, 128, &control).unwrap();
        assert_eq!(page.len(), 5);
        assert_eq!(page[1].path.as_deref(), Some("旅行/说明.txt"));
        assert_eq!(page[2].path, page[3].path);
        assert_ne!(page[2].id, page[3].id);
        assert_eq!(
            archive.resolve_members(&["notes.txt".into()], &[], &control),
            Err("ambiguous_member")
        );
        assert_eq!(archive.resolve_members(&[], &[3], &control).unwrap(), ["3"]);
        assert_eq!(
            archive
                .resolve_members(&["旅行/说明.txt".into()], &[], &control)
                .unwrap(),
            ["2"]
        );
        assert_eq!(
            archive.resolve_members(&["missing".into()], &[], &control),
            Err("member_not_found")
        );
        assert_eq!(
            archive.resolve_members(&["../escape".into()], &[], &control),
            Err("invalid_member_path")
        );
        assert_eq!(
            archive.resolve_members(&[], &[0, 6], &control),
            Err("invalid_entry_id")
        );
        assert!(
            archive
                .resolve_directory("旅行", &control)
                .unwrap()
                .starts_with('d')
        );
        assert!(archive.catalog(0, 129, &control).is_err());
        control.cancel();
        assert!(matches!(
            archive.catalog(0, 128, &control),
            Err(ScanError::Cancelled)
        ));
    }
}
