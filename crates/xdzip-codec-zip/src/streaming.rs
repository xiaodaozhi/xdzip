//! Two bounded central-directory passes share the ordinary scanner's parser and name policy.
use crate::{ZipCodec, metadata, record};
use xdzip_codec_api::names::NameSample;
use xdzip_codec_api::{Cancellation, Entry, MAX_INDEX_ENTRIES, ReadSource, ScanError, ScanLimits};

/// Comment bytes and location from a fully admitted ZIP central directory.
pub struct ArchiveComment {
    /// Original bytes, with no encoding assumed.
    pub bytes: Vec<u8>,
    /// The unchanged prefix ends immediately before this two-byte length field.
    pub length_offset: u64,
    /// The layout has no SFX prefix or known archive signature.
    pub ordinary_layout: bool,
}

impl ZipCodec {
    /// Stream a complete ZIP index to a host-owned sink. A sink remains private until
    /// this returns success; a later malformed record must invalidate earlier output.
    /// Both passes together consume at most `limits.read_bytes`, including repeated reads.
    pub fn scan_index(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
        sink: &mut dyn FnMut(Entry) -> Result<(), ScanError>,
    ) -> Result<ArchiveComment, ScanError> {
        let pass = ScanLimits {
            read_bytes: limits.read_bytes / 2,
            ..limits
        };
        let mut sample = NameSample::default();
        let mut index = 0;
        let mut decoded = 0;
        let original = record::visit_directory(
            source,
            control,
            pass,
            MAX_INDEX_ENTRIES,
            256 * 1024 * 1024,
            &mut |record| {
                let (entry, legacy) = metadata(record, index, limits, &mut decoded)?;
                sample.observe(&entry, legacy);
                index += 1;
                Ok(())
            },
        )?;
        let expected_count = index;
        let resolution = sample.finish(true);
        index = 0;
        decoded = 0;
        let mut text_bytes = 0;
        let directory = record::visit_directory(
            source,
            control,
            pass,
            MAX_INDEX_ENTRIES,
            256 * 1024 * 1024,
            &mut |record| {
                let (mut entry, legacy) = metadata(record, index, limits, &mut decoded)?;
                resolution.resolve(&mut entry, legacy, false, limits, &mut text_bytes)?;
                sink(entry)?;
                index += 1;
                Ok(())
            },
        )?;
        control.check()?;
        if index != expected_count
            || directory.end_offset != original.end_offset
            || directory.comment != original.comment
            || directory.ordinary_comment_layout != original.ordinary_comment_layout
        {
            return Err(ScanError::SourceChanged);
        }
        Ok(ArchiveComment {
            bytes: directory.comment,
            length_offset: directory.end_offset + 20,
            ordinary_layout: directory.ordinary_comment_layout,
        })
    }
}

impl ArchiveComment {
    /// Verify the same end-record location and the exact expected comment.
    /// Callers separately verify that the immutable prefix is byte-identical.
    pub fn verify(
        &self,
        source: &mut (impl std::io::Read + std::io::Seek),
        control: &Cancellation,
        expected: &[u8],
    ) -> Result<(), ScanError> {
        let end = record::read_end_record(source, control)?;
        if end.offset + 20 != self.length_offset || end.comment != expected {
            return Err(ScanError::Invalid);
        }
        Ok(())
    }
}
