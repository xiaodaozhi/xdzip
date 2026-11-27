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
mod comment;
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

    /// Read the archive-level comment. Missing, unsupported and damaged
    /// comments are distinct results; entry comments have no bearing on them.
    pub fn archive_comment(
        &mut self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
    ) -> Result<Option<String>, ScanError> {
        use index::ArchiveComment;
        control.check()?;
        let (bytes, utf16) = match &self.0.archive_comment {
            None => return Ok(None),
            Some(ArchiveComment::Unsupported) => return Err(ScanError::Unsupported),
            Some(ArchiveComment::Ambiguous) => return Err(ScanError::Invalid),
            Some(ArchiveComment::Inline(inline)) => (
                comment::inline(inline, control)?.ok_or(ScanError::Invalid)?,
                false,
            ),
            Some(ArchiveComment::Service(at)) => {
                let part = &self.0.parts[*at];
                let d = &part.description;
                if part.before || part.after || d.encrypted || d.redirection || d.special {
                    return Err(ScanError::Unsupported);
                }
                let size = d.size.ok_or(ScanError::Unsupported)?;
                if size > u64::from(u16::MAX) {
                    return Err(ScanError::Limit);
                }
                let mut reader = source::Reader::new(source, control, METADATA_LIMIT)?;
                if !d.legacy {
                    if d.compression != 0 {
                        return Err(ScanError::Unsupported);
                    }
                    if part.packed != size {
                        return Err(ScanError::Invalid);
                    }
                    if reader.bytes(part.volume, part.header_at, part.header.len())? != part.header
                    {
                        return Err(ScanError::SourceChanged);
                    }
                    let bytes = reader.bytes(part.volume, part.data_at, size as usize)?;
                    comment::verify_modern(d, &bytes)?;
                    return String::from_utf8(bytes)
                        .map(|text| Some(text.trim_end_matches('\0').to_owned()))
                        .map_err(|_| ScanError::Invalid);
                }
                let utf16 = d.service_comment_utf16.ok_or(ScanError::Invalid)?;
                if part.packed > 1024 * 1024 {
                    return Err(ScanError::Limit);
                }
                let bytes = comment::service(
                    &mut reader,
                    &self.0.parts,
                    &[*at],
                    control,
                    None,
                    &mut self.0.protection,
                )?
                .ok_or(ScanError::Invalid)?;
                (bytes, utf16)
            }
        };
        let text = if utf16 {
            let (pairs, tail) = bytes.as_chunks::<2>();
            if !tail.is_empty() {
                return Err(ScanError::Invalid);
            }
            let units: Vec<_> = pairs.iter().map(|pair| u16::from_le_bytes(*pair)).collect();
            String::from_utf16(&units).map_err(|_| ScanError::Invalid)?
        } else if let Ok(text) = String::from_utf8(bytes.clone()) {
            text
        } else {
            use xdzip_codec_api::names::{NameSample, decode_legacy};
            let mut sample = NameSample::default();
            sample.observe_bytes(&bytes);
            sample
                .finish(true)
                .decode_unmarked(&bytes)
                .map(|(text, _)| text)
                .or_else(|| decode_legacy(&bytes, "CP437"))
                .ok_or(ScanError::Invalid)?
        };
        Ok(Some(text.trim_end_matches('\0').to_owned()))
    }

    /// Decode only requested entry comments, without using them for file identity.
    pub fn entry_comments(
        &mut self,
        source: &mut dyn ReadSource,
        ids: &[u64],
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<Vec<(u64, Vec<u8>)>, ScanError> {
        if ids.len() > 128 {
            return Err(ScanError::Limit);
        }
        let mut reader = source::Reader::new(source, control, METADATA_LIMIT)?;
        let mut result = Vec::with_capacity(ids.len());
        for &id in ids {
            control.check()?;
            let position = usize::try_from(id.checked_sub(1).ok_or(ScanError::Invalid)?)
                .map_err(|_| ScanError::Invalid)?;
            let row = self.0.rows.get(position).ok_or(ScanError::Invalid)?;
            let file = self.0.files.get(position).ok_or(ScanError::Invalid)?;
            let part = self.0.parts.get(file.parts[0]).ok_or(ScanError::Invalid)?;
            let inline = match &part.description.comment {
                Some(comment) => comment::inline(comment, control)?.unwrap_or_default(),
                None => Vec::new(),
            };
            let using_service = inline.is_empty() && !file.comment_parts.is_empty();
            let bytes = if using_service {
                comment::service(
                    &mut reader,
                    &self.0.parts,
                    &file.comment_parts,
                    control,
                    password,
                    &mut self.0.protection,
                )?
                .unwrap_or_default()
            } else {
                inline
            };
            let label = match row.name_encoding {
                xdzip_codec_api::names::NameEncoding::Legacy(label) => label,
                _ => "CP437",
            };
            let utf16 = using_service
                && self.0.parts[file.comment_parts[0]]
                    .description
                    .service_comment_utf16
                    == Some(true);
            let text = if utf16 {
                if bytes.len() % 2 != 0 {
                    String::new()
                } else {
                    let units: Vec<u16> = bytes
                        .as_chunks::<2>()
                        .0
                        .iter()
                        .map(|pair| u16::from_le_bytes(*pair))
                        .collect();
                    String::from_utf16(&units)
                        .unwrap_or_default()
                        .trim_end_matches('\0')
                        .to_owned()
                }
            } else {
                std::str::from_utf8(&bytes)
                    .map(str::to_owned)
                    .ok()
                    .or_else(|| {
                        if matches!(
                            row.name_encoding,
                            xdzip_codec_api::names::NameEncoding::Legacy(_)
                        ) {
                            xdzip_codec_api::names::decode_legacy(&bytes, label)
                        } else {
                            let mut sample = xdzip_codec_api::names::NameSample::default();
                            sample.observe_bytes(&bytes);
                            sample
                                .finish(true)
                                .decode_unmarked(&bytes)
                                .map(|(text, _)| text)
                        }
                    })
                    .unwrap_or_else(|| xdzip_codec_api::names::escaped_bytes(&bytes))
            };
            let mut bytes = text.into_bytes();
            if bytes.len() > usize::from(u16::MAX) {
                let mut end = usize::from(u16::MAX) - 3;
                while !std::str::from_utf8(&bytes[..end]).is_ok() {
                    end -= 1;
                }
                bytes.truncate(end);
                bytes.extend_from_slice("…".as_bytes());
            }
            result.push((id, bytes));
        }
        Ok(result)
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
