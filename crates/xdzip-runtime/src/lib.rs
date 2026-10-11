//! First read-only session service. Called only on a bounded background worker.
pub mod service;
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
    _source: File,
    rows: Vec<Entry>,
    key: QueryKey,
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
        let rows = ZipCodec.scan(&mut source, control, ScanLimits::default())?;
        control.check()?;
        let after = source.metadata().map_err(|_| ScanError::Io)?;
        if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
            return Err(ScanError::SourceChanged);
        }
        let next = NEXT_SESSION
            .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
            .map_err(|_| ScanError::Limit)?;
        let key = QueryKey {
            session: SessionId::try_from(next).map_err(|_| ScanError::Limit)?,
            query: QueryId::try_from(next).map_err(|_| ScanError::Limit)?,
            generation: QueryGeneration::try_from(1).map_err(|_| ScanError::Limit)?,
            revision: SnapshotRevision::try_from(1).map_err(|_| ScanError::Limit)?,
        };
        Ok(Self {
            _source: source,
            rows,
            key,
        })
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
        xdzip_extract_plan::inspect(&self.rows, selected, control)
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
        control.check()?;
        let request = PageRequest::new(self.key, offset, limit).map_err(|_| ScanError::Limit)?;
        if directory.len() > 4096 || text.len() > 1024 {
            return Err(ScanError::Limit);
        }
        let prefix = if directory.is_empty() {
            &[][..]
        } else {
            let id = directory.parse::<u64>().map_err(|_| ScanError::Invalid)?;
            let entry = self
                .rows
                .iter()
                .find(|entry| entry.id.get() == id)
                .filter(|entry| entry.directory && !entry.warning && entry.raw_name.ends_with(b"/"))
                .ok_or(ScanError::Invalid)?;
            entry.raw_name.as_slice()
        };
        let needle = text.to_lowercase();
        let mut count = 0_usize;
        let mut rows = Vec::new();
        for entry in &self.rows {
            control.check()?;
            if !entry.raw_name.starts_with(prefix)
                || (!prefix.is_empty() && entry.raw_name == prefix)
                || (!needle.is_empty() && !entry.display_name.to_lowercase().contains(&needle))
            {
                continue;
            }
            if count as u64 >= request.offset() && rows.len() < usize::from(request.limit()) {
                rows.push(entry);
            }
            count += 1;
        }
        if offset > count as u64 {
            return Err(ScanError::Limit);
        }
        Ok((count, rows))
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

#[cfg(test)]
mod tests {
    use super::*;
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
