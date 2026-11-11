//! One cached immutable query order; transport batches only resolve row identities.
use crate::{Archive, index::RowRef, listing::ListingRow};
use xdzip_codec_api::{Cancellation, ScanError};
use xdzip_domain::identity::{QueryGeneration, QueryId};
use xdzip_domain::query::{PageRequest, QueryKey};

/// Name bytes per transport batch, in addition to the 128-row bound.
pub const MAX_BATCH_NAME_BYTES: usize = 64 * 1024;

/// Query failure distinct from a codec failure.
#[derive(Debug, Eq, PartialEq)]
pub enum QueryError {
    /// Metadata, source version or admission failure.
    Scan(ScanError),
    /// Continuation refers to a superseded result or generation.
    Stale,
}
impl std::fmt::Display for QueryError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Scan(error) => error.fmt(formatter),
            Self::Stale => formatter.write_str("stale_query"),
        }
    }
}
impl std::error::Error for QueryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Scan(error) => Some(error),
            Self::Stale => None,
        }
    }
}
impl From<ScanError> for QueryError {
    fn from(error: ScanError) -> Self {
        Self::Scan(error)
    }
}
pub(crate) struct QuerySnapshot {
    pub key: QueryKey,
    directory: String,
    text: String,
    sort: u8,
    rows: Vec<RowRef>,
    disk: Option<crate::disk_index::DiskQuery>,
}
/// Owned metadata for one bounded result page, independent of the index storage.
pub enum QueryRow {
    /// Physical identity and metadata.
    Physical(xdzip_codec_api::Entry),
    /// Navigation identity without output authority.
    Directory(crate::navigation::Directory),
}
impl QueryRow {
    /// Escaped archive-relative display path.
    pub fn path(&self) -> &str {
        match self {
            Self::Physical(e) => &e.display_name,
            Self::Directory(d) => &d.path,
        }
    }
}
/// A complete identity and bounded slice of one worker-owned result.
pub struct QueryPage {
    /// Stable across all batches of this query.
    pub key: QueryKey,
    /// Total presentation rows before transport slicing.
    pub total: usize,
    /// Borrowed rows, copied only when published across the native boundary.
    pub rows: Vec<QueryRow>,
}
/// Validated at the worker boundary, with a complete continuation identity.
pub struct QueryRequest<'a> {
    /// Session-local directory ID, or root.
    pub directory: &'a str,
    /// Literal archive-relative name search.
    pub text: &'a str,
    /// Shared sort code.
    pub sort: u8,
    /// First row of this transport batch.
    pub offset: u64,
    /// Exact result identity for a continuation; None starts a new result.
    pub expected: Option<QueryKey>,
    /// Nonzero UI intent generation for a new result.
    pub generation: u64,
}
impl Archive {
    /// Freeze logical ranges against the complete immutable query identity.
    /// Returned selectors retain the independent 100,000-item task limit.
    pub(crate) fn resolve_query_selectors(
        &self,
        selected: &[String],
        control: &Cancellation,
    ) -> Result<Vec<String>, &'static str> {
        let mut result = std::collections::BTreeSet::new();
        for selector in selected {
            control.check().map_err(|_| "cancelled")?;
            let Some(range) = selector.strip_prefix('q') else {
                result.insert(selector.clone());
                continue;
            };
            let mut fields = range.split(':');
            let identity = fields.next().ok_or("invalid_selection")?;
            let first = fields
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .ok_or("invalid_selection")?;
            let last = fields
                .next()
                .and_then(|n| n.parse::<usize>().ok())
                .ok_or("invalid_selection")?;
            if fields.next().is_some()
                || first > last
                || last - first >= xdzip_codec_api::MAX_ENTRIES
            {
                return Err("resource_limit");
            }
            let cache = self.cached_query.as_ref().ok_or("stale_query")?;
            if identity
                != format!(
                    "{}.{}.{}",
                    cache.key.query.get(),
                    cache.key.generation.get(),
                    cache.key.revision.get()
                )
            {
                return Err("stale_query");
            }
            let total = cache.disk.as_ref().map_or(cache.rows.len(), |q| q.total);
            if last >= total {
                return Err("invalid_selection");
            }
            let mut offset = first;
            while offset <= last {
                control.check().map_err(|_| "cancelled")?;
                if let Some(disk) = &cache.disk {
                    let page = disk.page(offset).map_err(|_| "resource_limit")?;
                    for token in page.into_iter().take(last - offset + 1) {
                        result.insert(
                            if let Some(at) = crate::disk_index::navigation_token(token) {
                                format!("d{}", at + 1)
                            } else {
                                token.to_string()
                            },
                        );
                        offset += 1;
                    }
                } else {
                    match cache.rows[offset] {
                        RowRef::Physical(at) => {
                            result.insert((at + 1).to_string());
                        }
                        RowRef::Directory(at) => {
                            result.insert(format!("d{}", at + 1));
                        }
                    }
                    offset += 1;
                }
                if result.len() > xdzip_codec_api::MAX_ENTRIES {
                    return Err("resource_limit");
                }
            }
        }
        Ok(result.into_iter().collect())
    }
    /// A missing expected identity starts a new result. Continuations supply its exact key.
    pub fn query_page(
        &mut self,
        request: QueryRequest<'_>,
        control: &Cancellation,
    ) -> Result<QueryPage, QueryError> {
        let QueryRequest {
            directory,
            text,
            sort,
            offset,
            expected,
            generation,
        } = request;
        control.check()?;
        self.source.validate()?;
        if let Some(expected) = expected {
            let cache = self.cached_query.as_ref().ok_or(QueryError::Stale)?;
            if cache.key != expected
                || cache.directory != directory
                || cache.text != text
                || cache.sort != sort
            {
                return Err(QueryError::Stale);
            }
        } else {
            if offset != 0 {
                return Err(QueryError::Stale);
            }
            let generation =
                QueryGeneration::try_from(generation).map_err(|_| QueryError::Stale)?;
            if self
                .cached_query
                .as_ref()
                .is_some_and(|cache| generation <= cache.key.generation)
            {
                return Err(QueryError::Stale);
            }
            let id = self.next_query.checked_add(1).ok_or(ScanError::Limit)?;
            let key = QueryKey {
                query: QueryId::try_from(id).map_err(|_| ScanError::Limit)?,
                generation,
                ..self.key
            };
            let (rows, disk) = if let Some(index) = &self.disk {
                let prefix = self.index_prefix(directory)?;
                (
                    Vec::new(),
                    Some(index.query(
                        &prefix,
                        text,
                        sort,
                        self.directories.as_deref().unwrap_or_default(),
                        control,
                    )?),
                )
            } else {
                (
                    self.listing_rows(directory, text, sort, control)?
                        .into_iter()
                        .map(RowRef::from_row)
                        .collect(),
                    None,
                )
            };
            self.source.validate()?;
            control.check()?;
            self.cached_query = Some(QuerySnapshot {
                key,
                directory: directory.into(),
                text: text.into(),
                sort,
                rows,
                disk,
            });
            self.next_query = id;
        }
        let cache = self.cached_query.as_ref().expect("validated query");
        let request = PageRequest::new(cache.key, offset, 128).map_err(|_| ScanError::Limit)?;
        let total = cache.disk.as_ref().map_or(cache.rows.len(), |q| q.total);
        if offset > total as u64 {
            return Err(ScanError::Limit.into());
        }
        let mut rows = Vec::new();
        let mut bytes = 0;
        let tokens = cache
            .disk
            .as_ref()
            .map(|q| q.page(offset as usize))
            .transpose()?;
        let candidates = if let Some(tokens) = tokens {
            tokens
                .into_iter()
                .map(|token| {
                    if let Some(at) = crate::disk_index::navigation_token(token) {
                        Ok(QueryRow::Directory(
                            self.directories
                                .as_ref()
                                .and_then(|d| d.get(at))
                                .ok_or(ScanError::Invalid)?
                                .clone(),
                        ))
                    } else {
                        Ok(QueryRow::Physical(
                            self.disk.as_ref().ok_or(ScanError::Invalid)?.get(token)?,
                        ))
                    }
                })
                .collect::<Result<Vec<_>, ScanError>>()?
        } else {
            cache
                .rows
                .iter()
                .skip(request.offset() as usize)
                .take(usize::from(request.limit()))
                .map(|row| match row.resolve(self) {
                    ListingRow::Physical(e) => QueryRow::Physical(e.clone()),
                    ListingRow::Directory(d) => QueryRow::Directory(d.clone()),
                })
                .collect()
        };
        for row in candidates {
            control.check()?;
            let next = bytes + row.path().len();
            if !rows.is_empty() && next > MAX_BATCH_NAME_BYTES {
                break;
            }
            if next > MAX_BATCH_NAME_BYTES {
                return Err(ScanError::Limit.into());
            }
            bytes = next;
            rows.push(row);
        }
        self.source.validate()?;
        Ok(QueryPage {
            key: cache.key,
            total,
            rows,
        })
    }
}

#[cfg(test)]
#[path = "../../../tests/support/zip64.rs"]
mod zip64_fixture;
#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    #[test]
    fn logical_selection_freezes_query_identity_and_resolves_sorted_rows() {
        let (_owned, path) = owned(&zip64_fixture::archive(1200));
        let control = Cancellation::default();
        let mut archive = Archive::open(&path, &control).unwrap();
        let page = archive
            .query_page(
                QueryRequest {
                    directory: "",
                    text: "",
                    sort: 2,
                    offset: 0,
                    expected: None,
                    generation: 1,
                },
                &control,
            )
            .unwrap();
        let identity = format!(
            "{}.{}.{}",
            page.key.query.get(),
            page.key.generation.get(),
            page.key.revision.get()
        );
        let selected = format!("q{identity}:127:129");
        assert_eq!(
            archive
                .resolve_query_selectors(std::slice::from_ref(&selected), &control)
                .unwrap(),
            ["1071", "1072", "1073"]
        );
        archive
            .query_page(
                QueryRequest {
                    directory: "",
                    text: "1199",
                    sort: 0,
                    offset: 0,
                    expected: None,
                    generation: 2,
                },
                &control,
            )
            .unwrap();
        assert_eq!(
            archive.resolve_query_selectors(&[selected], &control),
            Err("stale_query")
        );
    }
    struct Owned(PathBuf);
    impl Drop for Owned {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    fn owned(bytes: &[u8]) -> (Owned, PathBuf) {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "xdzip-m2-query-{}-{nonce}-{serial}",
            std::process::id()
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("archive.zip");
        std::fs::write(&path, bytes).unwrap();
        (Owned(root), path)
    }
    #[test]
    fn escaped_names_split_batches_by_bytes_without_dropping_records() {
        let names = vec!["\u{1}".repeat(4000); 4];
        let (_owned, path) = owned(&zip64_fixture::archive_names(&names));
        let control = Cancellation::default();
        let mut archive = Archive::open(&path, &control).unwrap();
        let request = |offset, expected| QueryRequest {
            directory: "",
            text: "",
            sort: 0,
            offset,
            expected,
            generation: 1,
        };
        let first = archive.query_page(request(0, None), &control).unwrap();
        assert_eq!(first.total, 4);
        assert_eq!(first.rows.len(), 3);
        let key = first.key;
        assert!(
            first.rows.iter().map(|row| row.path().len()).sum::<usize>() <= MAX_BATCH_NAME_BYTES
        );
        let last = archive.query_page(request(3, Some(key)), &control).unwrap();
        assert_eq!(last.rows.len(), 1);
        assert_eq!(last.total, 4);
    }
    #[test]
    fn hundred_thousand_rows_share_one_order_and_reject_each_stale_key() {
        let (_owned, path) = owned(&zip64_fixture::archive(100_000));
        let control = Cancellation::default();
        let mut archive = Archive::open(&path, &control).unwrap();
        let request = |offset, expected, generation, text| QueryRequest {
            directory: "",
            text,
            sort: 2,
            offset,
            expected,
            generation,
        };
        let page = archive
            .query_page(request(0, None, 1, ""), &control)
            .unwrap();
        assert_eq!(page.total, 100_000);
        assert_eq!(page.rows[0].path(), "item-099999.txt");
        let key = page.key;
        let mut received = page.rows.len();
        while received < 100_000 {
            let page = archive
                .query_page(request(received as u64, Some(key), 1, ""), &control)
                .unwrap();
            assert_eq!(page.key, key);
            received += page.rows.len();
        }
        assert_eq!(archive.next_query, 1);
        assert_eq!(received, 100_000);
        let mut stale = [key; 4];
        stale[0].session = (key.session.get() + 1).try_into().unwrap();
        stale[1].query = 2.try_into().unwrap();
        stale[2].generation = 2.try_into().unwrap();
        stale[3].revision = 2.try_into().unwrap();
        for key in stale {
            assert!(matches!(
                archive.query_page(request(128, Some(key), 1, ""), &control),
                Err(QueryError::Stale)
            ));
        }
        let latest = archive
            .query_page(request(0, None, 2, "099999"), &control)
            .unwrap();
        assert_eq!(latest.total, 1);
        assert_ne!(latest.key, key);
        assert!(matches!(
            archive.query_page(request(128, Some(key), 1, ""), &control),
            Err(QueryError::Stale)
        ));
    }
    #[test]
    fn source_modified_or_replaced_never_returns_cached_rows() {
        let bytes = zip64_fixture::archive(2);
        let (_owned, path) = owned(&bytes);
        let control = Cancellation::default();
        let mut archive = Archive::open(&path, &control).unwrap();
        let request = |expected, generation| QueryRequest {
            directory: "",
            text: "",
            sort: 0,
            offset: 0,
            expected,
            generation,
        };
        let key = archive.query_page(request(None, 1), &control).unwrap().key;
        std::fs::write(&path, b"changed").unwrap();
        assert!(matches!(
            archive.query_page(request(Some(key), 1), &control),
            Err(QueryError::Scan(ScanError::SourceChanged))
        ));
        assert!(matches!(
            archive.preflight(&[1.try_into().unwrap()], &control),
            Err(ScanError::SourceChanged)
        ));
        std::fs::write(&path, &bytes).unwrap();
        let mut archive = Archive::open(&path, &control).unwrap();
        let key = archive.query_page(request(None, 1), &control).unwrap().key;
        let old = path.with_extension("old");
        std::fs::rename(&path, &old).unwrap();
        std::fs::write(&path, &bytes).unwrap();
        {
            let modified = std::fs::metadata(&old).unwrap().modified().unwrap();
            std::fs::File::options()
                .write(true)
                .open(&path)
                .unwrap()
                .set_times(std::fs::FileTimes::new().set_modified(modified))
                .unwrap();
        }
        assert!(matches!(
            archive.query_page(request(Some(key), 1), &control),
            Err(QueryError::Scan(ScanError::SourceChanged))
        ));
    }
}
