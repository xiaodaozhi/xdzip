//! Current-directory listings keep virtual navigation separate from physical records.
use crate::{Archive, entry_type, navigation::Directory};
use xdzip_codec_api::{Cancellation, Entry, ScanError};
use xdzip_domain::query::PageRequest;

/// An implicit directory has navigation identity, never a synthetic physical EntryId.
#[derive(Clone, Copy)]
pub enum ListingRow<'a> {
    /// Real archive record, retaining its physical identity.
    Physical(&'a Entry),
    /// Safe implicit directory used for navigation only.
    Directory(&'a Directory),
}
impl ListingRow<'_> {
    /// Full escaped archive-relative path for navigation and search display.
    pub fn path(&self) -> &str {
        match self {
            Self::Physical(entry) => &entry.display_name,
            Self::Directory(node) => &node.path,
        }
    }
    fn stable_key(&self) -> (u8, u64) {
        match self {
            Self::Physical(entry) => (0, entry.id.get()),
            Self::Directory(node) => (1, node.id),
        }
    }
    fn kind(&self) -> String {
        match self {
            Self::Physical(entry) => entry_type(entry),
            Self::Directory(_) => String::new(),
        }
    }
    fn number(&self, sort: u8) -> u64 {
        match self {
            Self::Directory(_) => 0,
            Self::Physical(entry) => match sort {
                3 | 4 => entry.size,
                5 | 6 => entry.packed_size,
                7 | 8 => u64::from(entry.method),
                _ => u64::from(entry.modified_dos),
            },
        }
    }
}

impl Archive {
    /// Direct children for ordinary browsing; recursive physical matches for search.
    /// The admitted snapshot and navigation budgets bound both collection and sorting.
    pub fn listing_sorted(
        &self,
        directory: &str,
        text: &str,
        offset: u64,
        limit: u64,
        sort: u8,
        control: &Cancellation,
    ) -> Result<(usize, Vec<ListingRow<'_>>), ScanError> {
        if !text.is_empty() {
            let (total, entries) =
                self.browse_sorted(directory, text, offset, limit, sort, control)?;
            return Ok((
                total,
                entries.into_iter().map(ListingRow::Physical).collect(),
            ));
        }
        control.check()?;
        let request = PageRequest::new(self.key, offset, limit).map_err(|_| ScanError::Limit)?;
        if directory.len() > 4096 || sort > 12 {
            return Err(ScanError::Limit);
        }
        let prefix = self.directory_prefix(directory)?;
        let mut rows = Vec::new();
        for entry in &self.rows {
            control.check()?;
            let Some(tail) = entry.raw_name.strip_prefix(prefix) else {
                continue;
            };
            let tail = tail.strip_suffix(b"/").unwrap_or(tail);
            if !tail.is_empty() && !tail.contains(&b'/') {
                rows.push(ListingRow::Physical(entry));
            }
        }
        if let Some(nodes) = &self.directories {
            for node in nodes {
                control.check()?;
                let Some(tail) = node.path.as_bytes().strip_prefix(prefix) else {
                    continue;
                };
                let tail = tail.strip_suffix(b"/").unwrap_or(tail);
                if tail.is_empty() || tail.contains(&b'/') {
                    continue;
                }
                // Explicit records (including duplicates) retain their physical identities.
                if !rows.iter().any(|row| {
                    matches!(row, ListingRow::Physical(e)
                    if e.directory && !e.warning && e.raw_name == node.path.as_bytes())
                }) {
                    rows.push(ListingRow::Directory(node));
                }
            }
        }
        if sort == 9 {
            rows.sort_by_cached_key(|row| (row.kind(), row.stable_key()));
        } else if sort == 10 {
            rows.sort_by_cached_key(|row| (std::cmp::Reverse(row.kind()), row.stable_key()));
        } else if sort != 0 {
            rows.sort_unstable_by(|a, b| {
                let order = if sort <= 2 {
                    a.path().cmp(b.path())
                } else {
                    a.number(sort).cmp(&b.number(sort))
                };
                let order = if sort.is_multiple_of(2) {
                    order.reverse()
                } else {
                    order
                };
                order.then_with(|| a.stable_key().cmp(&b.stable_key()))
            });
        }
        control.check()?;
        let total = rows.len();
        if total > 10000 || offset > total as u64 {
            return Err(ScanError::Limit);
        }
        Ok((
            total,
            rows.into_iter()
                .skip(request.offset() as usize)
                .take(usize::from(request.limit()))
                .collect(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn direct_children_implicit_parents_duplicates_and_recursive_search() {
        let control = Cancellation::default();
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip");
        let mut archive = Archive::open(&path, &control).unwrap();
        archive.rows = [
            "root.txt",
            "app/Contents/file.txt",
            "app/Contents/sub/other.md",
            "app/Contents/empty/",
            "explicit/",
            "explicit/inside.txt",
            "root.txt",
        ]
        .iter()
        .enumerate()
        .map(|(i, name)| Entry {
            id: (i as u64 + 1).try_into().unwrap(),
            raw_name: name.as_bytes().to_vec(),
            display_name: (*name).into(),
            directory: name.ends_with('/'),
            size: 1,
            packed_size: 1,
            method: 0,
            modified_dos: 0,
            encrypted: false,
            warning: false,
        })
        .collect();
        archive.directories = Some(crate::navigation::build(&archive.rows, &control).unwrap());
        let (total, root) = archive.listing_sorted("", "", 0, 128, 0, &control).unwrap();
        assert_eq!(total, 4);
        assert_eq!(
            root.iter().map(ListingRow::path).collect::<Vec<_>>(),
            ["root.txt", "explicit/", "root.txt", "app/"]
        );
        let app = archive
            .directories
            .as_ref()
            .unwrap()
            .iter()
            .find(|d| d.path == "app/")
            .unwrap();
        let (_, children) = archive
            .listing_sorted(&format!("d{}", app.id), "", 0, 128, 0, &control)
            .unwrap();
        assert_eq!(children.len(), 1);
        assert!(matches!(children[0], ListingRow::Directory(_)));
        assert_eq!(children[0].path(), "app/Contents/");
        let contents = archive
            .directories
            .as_ref()
            .unwrap()
            .iter()
            .find(|d| d.path == "app/Contents/")
            .unwrap();
        let (total, children) = archive
            .listing_sorted(&format!("d{}", contents.id), "", 0, 128, 1, &control)
            .unwrap();
        assert_eq!(total, 3);
        assert!(
            children
                .iter()
                .any(|row| row.path() == "app/Contents/file.txt")
        );
        assert!(!children.iter().any(|row| row.path().ends_with("other.md")));
        let (_, search) = archive
            .listing_sorted("", "inside", 0, 128, 1, &control)
            .unwrap();
        assert!(matches!(search.as_slice(), [ListingRow::Physical(entry)] if entry.id.get() == 6));
        control.cancel();
        assert!(matches!(
            archive.listing_sorted("", "", 0, 128, 0, &control),
            Err(ScanError::Cancelled)
        ));
    }
}
