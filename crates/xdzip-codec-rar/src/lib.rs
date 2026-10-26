//! Independent, bounded, read-only RAR backend used by the isolated worker.
//!
//! Wire facts come from RARLAB's technical note and the frozen factual handoff
//! RAR-CONTAINER-2026-10-26-v1 and subsequent numeric supplements. Legacy and
//! modern algorithms share host-owned source, memory and output budgets.
//! Actual compatibility coverage and source-exposure boundaries are recorded
//! in docs/engineering/rar-format-sources.md; unverified extensions remain explicit.
mod algorithm;
#[cfg(test)]
mod ancient_tests;
pub(crate) mod crypto;
mod filter29;
#[cfg(test)]
mod filter29_owned_tests;
mod filter29_transform;
#[cfg(test)]
mod filter_author_tests;
mod index;
mod legacy;
#[cfg(test)]
mod legacy20_split_owned_tests;
mod legacy_audio20;
mod legacy_crypto;
mod legacy_lz15;
mod legacy_lz20;
mod legacy_password;
mod legacy_stream;
mod lz;
mod lz29;
mod lz29_bits;
mod lz_bits;
mod lz_filters;
mod modern;
mod names;
mod paged_history;
mod payload;
mod ppm29;
mod protection;
#[cfg(test)]
mod protection_tests;
#[cfg(test)]
mod rar29_store_owned_tests;
#[cfg(test)]
mod rar7_genuine_tests;
mod source;
#[cfg(test)]
mod tests;
mod time;
mod wire;

use xdzip_codec_api::payload::{DecodeError, PayloadCodec, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, MetadataCodec, ReadSource, ScanError, ScanLimits};

const METADATA_LIMIT: u64 = 64 * 1024 * 1024;
const INPUT_LIMIT: u64 = 20 * 1024 * 1024 * 1024;
const FILE_LIMIT: u64 = 8 * 1024 * 1024 * 1024;
const MEMORY_LIMIT: u64 = 1024 * 1024 * 1024;

/// Read-only RAR candidate; the host retains all input and output authority.
pub struct RarCodec;

/// Opaque complete index retaining physical headers and continuation ranges.
pub struct RarIndex(index::Index);
impl RarIndex {
    /// Public rows in first-visible physical order. Duplicate names stay distinct.
    pub fn entries(&self) -> &[Entry] {
        &self.0.rows
    }
}

/// Parse completely without publishing a partial index or opening another path.
pub fn read_index(
    source: &mut dyn ReadSource,
    control: &Cancellation,
    limits: ScanLimits,
    password: Option<&str>,
) -> Result<RarIndex, ScanError> {
    let maximum = ScanLimits::default();
    let limits = ScanLimits {
        entries: limits.entries.min(maximum.entries),
        name_bytes: limits.name_bytes.min(maximum.name_bytes),
        total_name_bytes: limits.total_name_bytes.min(maximum.total_name_bytes),
        display_name_bytes: limits.display_name_bytes.min(maximum.display_name_bytes),
        read_bytes: limits.read_bytes.min(maximum.read_bytes),
    };
    let mut reader = source::Reader::new(source, control, limits.read_bytes.min(METADATA_LIMIT))?;
    index::parse(&mut reader, limits, password).map(RarIndex)
}

impl MetadataCodec for RarCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        self.scan_with_password(source, control, limits, None)
    }
    fn scan_with_password(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
        password: Option<&str>,
    ) -> Result<Vec<Entry>, ScanError> {
        read_index(source, control, limits, password).map(|index| index.0.rows)
    }
}

fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Unsupported => DecodeError::Unsupported,
        ScanError::PasswordRequired => DecodeError::PasswordRequired,
        ScanError::WrongPassword => DecodeError::WrongPassword,
        ScanError::PasswordOrIntegrity => DecodeError::PasswordOrIntegrity,
        _ => DecodeError::Integrity,
    }
}
impl PayloadCodec for RarCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        self.open_payload_with_password(source, snapshot, control, None)
    }
    fn open_payload_with_password<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
        password: Option<&str>,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let mut reader =
            source::Reader::new(source, control, METADATA_LIMIT).map_err(decode_error)?;
        let index =
            index::parse(&mut reader, ScanLimits::default(), password).map_err(decode_error)?;
        if index.rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        reader.set_limit(INPUT_LIMIT);
        Ok(Box::new(payload::Payload::new(reader, index)))
    }
}
