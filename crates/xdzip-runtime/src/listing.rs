//! Current-directory listings keep virtual navigation separate from physical records.
use crate::{Archive, entry_type_key, navigation::Directory};
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
    fn type_key(&self) -> (bool, String) {
        match self {
            Self::Physical(entry) => entry_type_key(entry),
            Self::Directory(_) => (false, String::new()),
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
        control.check()?;
        let request = PageRequest::new(self.key, offset, limit).map_err(|_| ScanError::Limit)?;
        let rows = self.listing_rows(directory, text, sort, control)?;
        let total = rows.len();
        if offset > total as u64 {
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

    pub(crate) fn listing_rows(
        &self,
        directory: &str,
        text: &str,
        sort: u8,
        control: &Cancellation,
    ) -> Result<Vec<ListingRow<'_>>, ScanError> {
        if directory.len() > 21 || text.len() > 1024 || sort > 12 {
            return Err(ScanError::Limit);
        }
        let mut rows = if !text.is_empty() {
            self.matching_physical(directory, text, control)?
                .into_iter()
                .map(ListingRow::Physical)
                .collect::<Vec<_>>()
        } else {
            let prefix = self.directory_prefix(directory)?;
            self.index
                .children
                .get(prefix)
                .into_iter()
                .flatten()
                .map(|row| row.resolve(self))
                .collect()
        };
        if sort == 9 {
            rows.sort_by_cached_key(|row| (row.type_key(), row.stable_key()));
        } else if sort == 10 {
            rows.sort_by_cached_key(|row| {
                let (file, category) = row.type_key();
                (file, std::cmp::Reverse(category), row.stable_key())
            });
        } else if sort != 0 {
            rows.sort_unstable_by(|a, b| {
                if matches!(sort, 3 | 4) {
                    let known = |row: &ListingRow<'_>| match row {
                        ListingRow::Physical(entry) => entry.size_known,
                        ListingRow::Directory(_) => true,
                    };
                    if known(a) != known(b) {
                        return known(b).cmp(&known(a));
                    }
                }
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
        if rows.len() > xdzip_codec_api::MAX_ENTRIES {
            return Err(ScanError::Limit);
        }
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn type_sort_keeps_directories_first_in_both_directions_and_search() {
        let control = Cancellation::default();
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/type-order.zip");
        let archive = Archive::open(&path, &control).unwrap();
        for query in ["", "item"] {
            let directories = if query.is_empty() {
                vec!["item-alpha/", "item-folder.ZIP/", "item-implicit/"]
            } else {
                vec!["item-alpha/", "item-folder.ZIP/"]
            };
            let mut extensionless = vec![
                "item-README",
                "item-LICENSE",
                "item-duplicate",
                "item-duplicate",
            ];
            if !query.is_empty() {
                extensionless.push("item-implicit/item-child");
            }
            for order in [9, 10] {
                let mut expected = directories.clone();
                if order == 9 {
                    expected.extend(extensionless.iter().copied());
                    expected.extend(["item-image.JPG", "item-image.jpeg", "item-last.ZIP"]);
                } else {
                    expected.extend(["item-last.ZIP", "item-image.JPG", "item-image.jpeg"]);
                    expected.extend(extensionless.iter().copied());
                }
                let (total, rows) = archive
                    .listing_sorted("", query, 0, 128, order, &control)
                    .unwrap();
                assert_eq!(total, 10);
                assert_eq!(
                    rows.iter().map(ListingRow::path).collect::<Vec<_>>(),
                    expected
                );
                // Equal categories retain physical identity order in both directions.
                let duplicate_ids = rows
                    .iter()
                    .filter_map(|row| match row {
                        ListingRow::Physical(entry) if entry.display_name == "item-duplicate" => {
                            Some(entry.id.get())
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                assert_eq!(duplicate_ids, [5, 6]);
                // Group the complete result before splitting transport batches.
                let mut batched = Vec::new();
                for offset in [0, 3, 6, 9] {
                    let (_, page) = archive
                        .listing_sorted("", query, offset, 3, order, &control)
                        .unwrap();
                    batched.extend(page.iter().map(|row| row.path().to_owned()));
                }
                assert_eq!(batched, expected);
            }
        }
    }

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
            format: xdzip_codec_api::Format::Zip,
            id: (i as u64 + 1).try_into().unwrap(),
            raw_name: name.as_bytes().to_vec(),
            decoded_name: None,
            name_encoding: xdzip_codec_api::names::NameEncoding::Utf8,
            display_name: (*name).into(),
            directory: name.ends_with('/'),
            link: false,
            unix_mode: 0,
            size: 1,
            size_known: true,
            packed_size: 1,
            method: 0,
            modified_dos: 0,
            encrypted: false,
            password_supported: false,
            warning: false,
        })
        .collect::<Vec<_>>()
        .into();
        archive.directories = Some(crate::navigation::build(&archive.rows, &control).unwrap());
        archive.index = crate::index::DirectoryIndex::build(
            &archive.rows,
            archive.directories.as_deref(),
            &control,
        )
        .unwrap();
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
