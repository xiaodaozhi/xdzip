//! First read-only session service. Called only on a bounded background worker.
mod index;
pub mod listing;
pub mod navigation;
pub mod query;
pub mod service;
mod source;
use std::fs::File;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
use xdzip_codec_api::{Cancellation, Entry, MetadataCodec, ScanError, ScanLimits};
use xdzip_codec_zip::ZipCodec;
use xdzip_domain::identity::{QueryGeneration, QueryId, SessionId, SnapshotRevision};
use xdzip_domain::query::{PageRequest, QueryKey};

static NEXT_SESSION: AtomicU64 = AtomicU64::new(1);

/// Immutable metadata snapshot backed by an owned read-only source handle.
pub struct Archive {
    source: source::Source,
    index: index::DirectoryIndex,
    cached_query: Option<query::QuerySnapshot>,
    next_query: u64,
    rows: Vec<Entry>,
    key: QueryKey,
    directories: Option<Vec<navigation::Directory>>,
}
impl Archive {
    /// Open a user-selected regular file; complete the scan before publishing any rows.
    /// Length/mtime checks are best-effort mutation detection, not an atomic source snapshot.
    pub fn open(path: &Path, control: &Cancellation) -> Result<Self, ScanError> {
        control.check()?;
        if !path.is_file() {
            return Err(ScanError::Io);
        }
        let mut source = File::open(path).map_err(|_| ScanError::Io)?;
        let before = source.metadata().map_err(|_| ScanError::Io)?;
        if !before.is_file() {
            return Err(ScanError::Io);
        }
        let version = source::SourceVersion::read(&before)?;
        let scanned = ZipCodec.scan(&mut source, control, ScanLimits::default());
        control.check()?;
        let source = source::Source::new(source, path.to_owned(), version);
        source.validate()?;
        let rows = scanned?;
        let next = NEXT_SESSION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| ScanError::Limit)?;
        let key = QueryKey {
            session: SessionId::try_from(next).map_err(|_| ScanError::Limit)?,
            query: QueryId::try_from(next).map_err(|_| ScanError::Limit)?,
            generation: QueryGeneration::try_from(1).map_err(|_| ScanError::Limit)?,
            revision: SnapshotRevision::try_from(1).map_err(|_| ScanError::Limit)?,
        };
        let directories = match navigation::build(&rows, control) {
            Ok(nodes) => Some(nodes),
            Err(ScanError::Limit) => None,
            Err(error) => return Err(error),
        };
        control.check()?;
        let index = index::DirectoryIndex::build(&rows, directories.as_deref(), control)?;
        Ok(Self {
            directories,
            source,
            index,
            cached_query: None,
            next_query: 0,
            rows,
            key,
        })
    }
    /// Complete navigation nodes, or None if the separate navigation budget was exceeded.
    pub fn directories(&self) -> Option<&[navigation::Directory]> {
        self.directories.as_deref()
    }
    /// Exact physical count, including duplicate names and explicit directories.
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    /// Whether this is a valid empty archive.
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    /// Immutable identity associated with every page of this scan.
    pub fn key(&self) -> QueryKey {
        self.key
    }
    /// Inspect metadata for selected physical IDs. This does not open any output file.
    pub fn preflight(
        &self,
        selected: &[xdzip_domain::identity::EntryId],
        control: &Cancellation,
    ) -> Result<xdzip_extract_plan::Report, ScanError> {
        self.source.validate()?;
        let result = xdzip_extract_plan::inspect(&self.rows, selected, control)?;
        self.source.validate()?;
        Ok(result)
    }
    /// Filter an immutable snapshot by an explicit safe directory and literal name text.
    /// Directory scope includes descendants; it never resolves a host filesystem path.
    pub fn browse(
        &self,
        directory: &str,
        text: &str,
        offset: u64,
        limit: u64,
        control: &Cancellation,
    ) -> Result<(usize, Vec<&Entry>), ScanError> {
        self.browse_sorted(directory, text, offset, limit, 0, control)
    }
    /// Sort the full matching snapshot before slicing a page. Codes: 0 physical,
    /// 1/2 name, 3/4 original size, 5/6 packed size, 7/8 method, 9/10 type, 11/12 local modification date; odd ascending.
    /// Names use Unicode scalar lexical order; ties retain ascending physical identity.
    pub fn browse_sorted(
        &self,
        directory: &str,
        text: &str,
        offset: u64,
        limit: u64,
        sort: u8,
        control: &Cancellation,
    ) -> Result<(usize, Vec<&Entry>), ScanError> {
        control.check()?;
        let request = PageRequest::new(self.key, offset, limit).map_err(|_| ScanError::Limit)?;
        if directory.len() > 4096 || text.len() > 1024 || sort > 12 {
            return Err(ScanError::Limit);
        }
        let mut rows = self.matching_physical(directory, text, control)?;
        let count = rows.len();
        if sort != 0 {
            // Sorting is bounded by the admitted 100,000 records. Check cancellation on
            // both sides; changing comparator behavior during cancellation is invalid.
            control.check()?;
            if sort == 9 {
                rows.sort_by_cached_key(|entry| (entry_type_key(entry), entry.id.get()));
            } else if sort == 10 {
                rows.sort_by_cached_key(|entry| {
                    let (file, category) = entry_type_key(entry);
                    (file, std::cmp::Reverse(category), entry.id.get())
                });
            } else {
                rows.sort_unstable_by(|a, b| {
                    let order = match sort {
                        1 | 2 => a.display_name.cmp(&b.display_name),
                        3 | 4 => a.size.cmp(&b.size),
                        5 | 6 => a.packed_size.cmp(&b.packed_size),
                        7 | 8 => a.method.cmp(&b.method),
                        _ => a.modified_dos.cmp(&b.modified_dos),
                    };
                    let order = if sort.is_multiple_of(2) {
                        order.reverse()
                    } else {
                        order
                    };
                    order.then_with(|| a.id.get().cmp(&b.id.get()))
                });
            }
            control.check()?;
        }
        if offset > count as u64 {
            return Err(ScanError::Limit);
        }
        Ok((
            count,
            rows.into_iter()
                .skip(request.offset() as usize)
                .take(usize::from(request.limit()))
                .collect(),
        ))
    }
    fn matching_physical(
        &self,
        directory: &str,
        text: &str,
        control: &Cancellation,
    ) -> Result<Vec<&Entry>, ScanError> {
        let prefix = self.directory_prefix(directory)?;
        let needle = text.to_lowercase();
        let mut rows = Vec::new();
        for entry in &self.rows {
            control.check()?;
            if entry.raw_name.starts_with(prefix)
                && (prefix.is_empty() || entry.raw_name != prefix)
                && (needle.is_empty() || entry.display_name.to_lowercase().contains(&needle))
            {
                rows.push(entry);
            }
        }
        Ok(rows)
    }

    fn directory_prefix(&self, directory: &str) -> Result<&[u8], ScanError> {
        let prefix = if directory.is_empty() {
            &[][..]
        } else if let Some(id) = directory.strip_prefix('d') {
            let id = id.parse::<u64>().map_err(|_| ScanError::Invalid)?;
            self.directories
                .as_ref()
                .and_then(|nodes| {
                    id.checked_sub(1)
                        .and_then(|at| usize::try_from(at).ok())
                        .and_then(|at| nodes.get(at))
                        .filter(|node| node.id == id)
                })
                .ok_or(ScanError::Invalid)?
                .path
                .as_bytes()
        } else {
            let id = directory.parse::<u64>().map_err(|_| ScanError::Invalid)?;
            let entry = self
                .rows
                .get(
                    id.checked_sub(1)
                        .and_then(|at| usize::try_from(at).ok())
                        .ok_or(ScanError::Invalid)?,
                )
                .filter(|entry| entry.id.get() == id)
                .filter(|entry| entry.directory && !entry.warning && entry.raw_name.ends_with(b"/"))
                .ok_or(ScanError::Invalid)?;
            entry.raw_name.as_slice()
        };
        Ok(prefix)
    }
    /// Return at most 256 references without copying the complete index.
    pub fn page(&self, offset: u64, limit: u64) -> Result<&[Entry], ScanError> {
        let request = PageRequest::new(self.key, offset, limit).map_err(|_| ScanError::Limit)?;
        let start = usize::try_from(request.offset()).map_err(|_| ScanError::Limit)?;
        if start > self.rows.len() {
            return Err(ScanError::Limit);
        }
        let end = start
            .saturating_add(usize::from(request.limit()))
            .min(self.rows.len());
        Ok(&self.rows[start..end])
    }
}

// Keep directory identity separate from the extension category and translated labels.
// Only the category is reversed for descending type order; directories remain first.
fn entry_type_key(entry: &Entry) -> (bool, String) {
    (!entry.directory, entry_type(entry))
}

fn entry_type(entry: &Entry) -> String {
    if entry.directory {
        return String::new();
    }
    let name = entry.display_name.rsplit('/').next().unwrap_or_default();
    let extension = name.rsplit_once('.').map_or("", |(_, extension)| extension);
    if extension.eq_ignore_ascii_case("jpg") {
        "JPEG".to_owned()
    } else {
        extension.to_uppercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sorting_spans_pages_and_virtual_directory_keeps_physical_ids() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/pages.zip");
        let control = Cancellation::default();
        let archive = Archive::open(&path, &control).unwrap();
        let nodes = archive.directories().unwrap();
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].count, 270);
        let directory = format!("d{}", nodes[0].id);
        let (total, first) = archive
            .browse_sorted(&directory, "", 0, 128, 2, &control)
            .unwrap();
        let (_, second) = archive
            .browse_sorted(&directory, "", 128, 128, 2, &control)
            .unwrap();
        assert_eq!(total, 270);
        assert_eq!(first[0].id.get(), 270);
        assert_eq!(first[127].id.get(), 143);
        assert_eq!(second[0].id.get(), 142);
        assert_eq!(
            archive
                .browse_sorted(&directory, "269", 0, 128, 2, &control)
                .unwrap()
                .0,
            1
        );
        assert!(
            archive
                .browse_sorted("d999", "", 0, 128, 0, &control)
                .is_err()
        );
        assert!(archive.browse_sorted("", "", 0, 128, 13, &control).is_err());
        control.cancel();
        assert!(matches!(
            archive.browse_sorted("", "", 0, 128, 2, &control),
            Err(ScanError::Cancelled)
        ));
    }
    #[test]
    fn type_and_local_date_sort_both_directions_keep_identity_ties() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip");
        let control = Cancellation::default();
        let mut archive = Archive::open(&path, &control).unwrap();
        archive.rows[2].display_name = "same.ZIP".into();
        archive.rows[3].display_name = "same.jpg".into();
        archive.rows[2].modified_dos = 200;
        archive.rows[3].modified_dos = 100;
        for (order, first) in [(9, 4), (10, 3), (11, 4), (12, 3)] {
            let (_, rows) = archive
                .browse_sorted("", "same", 0, 128, order, &control)
                .unwrap();
            assert_eq!(rows[0].id.get(), first);
            assert_eq!(rows.len(), 2);
        }
        archive.rows[2].modified_dos = 100;
        for order in [11, 12] {
            let (_, rows) = archive
                .browse_sorted("", "same", 0, 128, order, &control)
                .unwrap();
            assert_eq!(rows.iter().map(|r| r.id.get()).collect::<Vec<_>>(), [3, 4]);
        }
    }
    #[test]
    fn numeric_sort_is_exact_and_duplicate_names_keep_stable_physical_ties() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip");
        let control = Cancellation::default();
        let mut archive = Archive::open(&path, &control).unwrap();
        // Exercise values beyond JavaScript precision without trusting display strings.
        archive.rows[2].size = (1 << 53) + 1;
        archive.rows[3].size = 1 << 53;
        archive.rows[2].packed_size = u64::MAX;
        archive.rows[3].packed_size = u64::MAX - 1;
        for sort in [1, 2] {
            let (_, rows) = archive
                .browse_sorted("", "notes", 0, 128, sort, &control)
                .unwrap();
            assert_eq!(
                rows.iter().map(|row| row.id.get()).collect::<Vec<_>>(),
                [3, 4]
            );
        }
        for sort in [3, 5] {
            assert_eq!(
                archive
                    .browse_sorted("", "notes", 0, 128, sort, &control)
                    .unwrap()
                    .1[0]
                    .id
                    .get(),
                4
            );
        }
        for sort in [4, 6] {
            assert_eq!(
                archive
                    .browse_sorted("", "notes", 0, 128, sort, &control)
                    .unwrap()
                    .1[0]
                    .id
                    .get(),
                3
            );
        }
        assert_eq!(
            archive
                .browse_sorted("", "", 0, 128, 8, &control)
                .unwrap()
                .1[0]
                .method,
            8
        );
    }
    #[test]
    fn directory_queries_keep_physical_identity_and_enforce_limits() {
        let path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/normal.zip");
        let control = Cancellation::default();
        let archive = Archive::open(&path, &control).unwrap();
        let (total, rows) = archive.browse("1", "", 0, 128, &control).unwrap();
        assert_eq!(total, 1);
        assert_eq!(rows[0].id.get(), 2);
        let (total, rows) = archive.browse("", "NOTES", 1, 1, &control).unwrap();
        assert_eq!(total, 2);
        assert_eq!(rows[0].id.get(), 4);
        assert_eq!(
            archive.browse("", "missing", 0, 128, &control).unwrap().0,
            0
        );
        assert!(archive.browse("2", "", 0, 128, &control).is_err());
        assert!(archive.browse("../", "", 0, 128, &control).is_err());
        assert!(
            archive
                .browse("", &"x".repeat(1025), 0, 128, &control)
                .is_err()
        );
        assert!(archive.browse("", "", u64::MAX, 128, &control).is_err());
        control.cancel();
        assert!(matches!(
            archive.browse("", "", 0, 128, &control),
            Err(ScanError::Cancelled)
        ));
    }
    #[test]
    fn pages_preserve_identity_and_bounds_without_reopening_or_writing() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/zip/pages.zip");
        let before = std::fs::read(&path).unwrap();
        let archive = Archive::open(&path, &Cancellation::default()).unwrap();
        assert_eq!(archive.len(), 270);
        assert_eq!(archive.page(0, 128).unwrap().len(), 128);
        assert_eq!(archive.page(256, 128).unwrap().len(), 14);
        assert_eq!(archive.page(128, 128).unwrap()[0].id.get(), 129);
        assert!(archive.page(0, 257).is_err());
        assert!(archive.page(u64::MAX, 1).is_err());
        assert_ne!(
            archive.key().session,
            Archive::open(&path, &Cancellation::default())
                .unwrap()
                .key()
                .session
        );
        let report = archive
            .preflight(
                &[archive.page(0, 1).unwrap()[0].id],
                &Cancellation::default(),
            )
            .unwrap();
        assert_eq!(report.entries, 1);
        assert_eq!(report.finding_count, 0);
        assert_eq!(before, std::fs::read(&path).unwrap());
    }
}
