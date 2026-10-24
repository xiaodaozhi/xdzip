//! Direct-child indexes retain identities, not copied row metadata.
use crate::{Archive, listing::ListingRow, navigation::Directory};
use std::collections::{BTreeMap, HashSet};
use xdzip_codec_api::{Cancellation, Entry, ScanError};

#[derive(Clone, Copy, Debug)]
pub(crate) enum RowRef {
    Physical(usize),
    Directory(usize),
}
impl RowRef {
    pub fn resolve(self, archive: &Archive) -> ListingRow<'_> {
        match self {
            Self::Physical(at) => ListingRow::Physical(&archive.rows[at]),
            Self::Directory(at) => ListingRow::Directory(
                &archive.directories.as_ref().expect("indexed navigation")[at],
            ),
        }
    }
    pub fn from_row(row: ListingRow<'_>) -> Self {
        match row {
            ListingRow::Physical(entry) => Self::Physical(entry.id.get() as usize - 1),
            ListingRow::Directory(node) => Self::Directory(node.id as usize - 1),
        }
    }
}
pub(crate) struct DirectoryIndex {
    pub children: BTreeMap<Vec<u8>, Vec<RowRef>>,
}
fn parent(name: &[u8]) -> Option<&[u8]> {
    let name = name.strip_suffix(b"/").unwrap_or(name);
    if name.is_empty() || name.ends_with(b"/") {
        return None;
    }
    Some(
        name.iter()
            .rposition(|byte| *byte == b'/')
            .map_or(&[][..], |at| &name[..=at]),
    )
}
impl DirectoryIndex {
    pub fn build(
        rows: &[Entry],
        directories: Option<&[Directory]>,
        control: &Cancellation,
    ) -> Result<Self, ScanError> {
        let mut children = BTreeMap::<Vec<u8>, Vec<RowRef>>::new();
        let mut explicit = HashSet::new();
        for (at, entry) in rows.iter().enumerate() {
            control.check()?;
            if let Some(prefix) = parent(entry.name_bytes()) {
                children
                    .entry(prefix.to_vec())
                    .or_default()
                    .push(RowRef::Physical(at));
            }
            if entry.directory && !entry.warning {
                explicit.insert(entry.name_bytes());
            }
        }
        for (at, node) in directories.into_iter().flatten().enumerate() {
            control.check()?;
            if !explicit.contains(node.path.as_bytes()) {
                children
                    .entry(
                        parent(node.path.as_bytes())
                            .expect("valid navigation")
                            .to_vec(),
                    )
                    .or_default()
                    .push(RowRef::Directory(at));
            }
        }
        Ok(Self { children })
    }
}
