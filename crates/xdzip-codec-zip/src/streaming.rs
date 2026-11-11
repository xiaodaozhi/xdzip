//! Two bounded central-directory passes share the ordinary scanner's parser and name policy.
use crate::{ZipCodec, metadata, record};
use xdzip_codec_api::names::NameSample;
use xdzip_codec_api::{Cancellation, Entry, MAX_INDEX_ENTRIES, ReadSource, ScanError, ScanLimits};

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
    ) -> Result<(), ScanError> {
        let pass = ScanLimits {
            read_bytes: limits.read_bytes / 2,
            ..limits
        };
        let mut sample = NameSample::default();
        let mut index = 0;
        let mut decoded = 0;
        record::visit_directory(
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
        record::visit_directory(
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
        if index != expected_count {
            return Err(ScanError::SourceChanged);
        }
        Ok(())
    }
}
