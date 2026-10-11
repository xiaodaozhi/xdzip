//! First read-only session service. Called only on a bounded background worker.
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
        assert_eq!(before, std::fs::read(&path).unwrap());
    }
}
