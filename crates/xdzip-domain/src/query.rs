//! Bounded page requests and complete identity checks for asynchronous responses.
use crate::identity::{QueryGeneration, QueryId, SessionId, SnapshotRevision};

/// Maximum rows per page, independent of Qt's signed model row limit.
pub const MAX_PAGE_ROWS: u16 = 256;

/// Every field participates in response identity; offsets alone are never cache keys.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct QueryKey {
    /// Archive session owning the query.
    pub session: SessionId,
    /// Immutable result snapshot identity.
    pub query: QueryId,
    /// UI request generation that produced this snapshot.
    pub generation: QueryGeneration,
    /// Version of the immutable result.
    pub revision: SnapshotRevision,
}

/// Rejection before any allocation or backend query occurs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageError {
    /// Zero rows or a page exceeding the configured maximum.
    InvalidLimit,
    /// The exclusive end offset cannot be represented in 64 bits.
    OffsetOverflow,
}

impl std::fmt::Display for PageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidLimit => "page limit must be between 1 and 256",
            Self::OffsetOverflow => "page end offset exceeds the supported range",
        })
    }
}
impl std::error::Error for PageError {}

/// Validated, immutable request. Construct before narrowing counts for Qt.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageRequest {
    key: QueryKey,
    offset: u64,
    limit: u16,
}

impl PageRequest {
    /// Validate a foreign 64-bit limit and checked end offset without truncation.
    pub fn new(key: QueryKey, offset: u64, limit: u64) -> Result<Self, PageError> {
        let limit = u16::try_from(limit).map_err(|_| PageError::InvalidLimit)?;
        if !(1..=MAX_PAGE_ROWS).contains(&limit) {
            return Err(PageError::InvalidLimit);
        }
        offset
            .checked_add(u64::from(limit))
            .ok_or(PageError::OffsetOverflow)?;
        Ok(Self { key, offset, limit })
    }

    /// Full identity needed by a page cache or an asynchronous response.
    pub const fn key(self) -> QueryKey {
        self.key
    }
    /// First logical row; no conversion to a Qt model row has occurred.
    pub const fn offset(self) -> u64 {
        self.offset
    }
    /// Validated row budget, including incomplete snapshots.
    pub const fn limit(self) -> u16 {
        self.limit
    }

    /// Reject stale identity, wrong position or over-budget responses before applying rows.
    /// A short or empty page is valid; completeness remains a separate response property.
    pub fn accepts(self, key: QueryKey, offset: u64, row_count: u64) -> bool {
        self.key == key && self.offset == offset && row_count <= u64::from(self.limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key() -> QueryKey {
        QueryKey {
            session: SessionId::try_from(1).unwrap(),
            query: QueryId::try_from(1).unwrap(),
            generation: QueryGeneration::try_from(1).unwrap(),
            revision: SnapshotRevision::try_from(1).unwrap(),
        }
    }

    #[test]
    fn foreign_counts_never_truncate_or_wrap() {
        for limit in [0, 257, 65537, u64::MAX] {
            assert_eq!(
                PageRequest::new(key(), 0, limit),
                Err(PageError::InvalidLimit)
            );
        }
        assert_eq!(
            PageRequest::new(key(), u64::MAX, 1),
            Err(PageError::OffsetOverflow)
        );
        assert!(PageRequest::new(key(), u64::MAX - 256, 256).is_ok());
    }

    #[test]
    fn each_stale_identity_dimension_is_rejected() {
        let request = PageRequest::new(key(), 256, 128).unwrap();
        let mut stale = [key(); 4];
        stale[0].session = SessionId::try_from(2).unwrap();
        stale[1].query = QueryId::try_from(2).unwrap();
        stale[2].generation = QueryGeneration::try_from(2).unwrap();
        stale[3].revision = SnapshotRevision::try_from(2).unwrap();
        for key in stale {
            assert!(!request.accepts(key, 256, 128));
        }
        assert!(!request.accepts(key(), 0, 128));
        assert!(!request.accepts(key(), 256, 129));
        assert!(request.accepts(key(), 256, 0));
        assert!(request.accepts(key(), 256, 128));
    }
}
