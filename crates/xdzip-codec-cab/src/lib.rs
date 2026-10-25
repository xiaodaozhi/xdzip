//! Read-only Cabinet containers over host-authorized streams and shared algorithms.
mod index;
mod payload;
mod source;
#[cfg(test)]
mod tests;

use xdzip_codec_api::encode::{
    CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    RewriteCodec, RewriteInput, RewriteOutput,
};
use xdzip_codec_api::payload::{DecodeError, PayloadCodec, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, MetadataCodec, ReadSource, ScanError, ScanLimits};

const METADATA_LIMIT: u64 = 64 * 1024 * 1024;
const INPUT_LIMIT: u64 = 20 * 1024 * 1024 * 1024;
const FILE_LIMIT: u64 = 8 * 1024 * 1024 * 1024;

/// Microsoft Cabinet reader. The host owns volume discovery and output authority.
pub struct CabCodec;

impl CabCodec {
    /// Preserve the original CFFILE UTF-8 declaration independently of resolved names.
    /// Flags correspond to logical records after verified continuation coalescing.
    pub fn scan_with_encoding_flags(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<(Vec<Entry>, Vec<bool>), ScanError> {
        let mut reader =
            source::Reader::new(source, control, limits.read_bytes.min(METADATA_LIMIT))?;
        let index = index::parse(&mut reader, limits)?;
        let flags = index
            .files
            .iter()
            .map(|file| file.attributes & 0x80 != 0)
            .collect();
        Ok((index.rows, flags))
    }
}
impl MetadataCodec for CabCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        self.scan_with_encoding_flags(source, control, limits)
            .map(|result| result.0)
    }
}
fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}
impl PayloadCodec for CabCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let mut reader =
            source::Reader::new(source, control, METADATA_LIMIT).map_err(decode_error)?;
        let index = index::parse(&mut reader, ScanLimits::default()).map_err(decode_error)?;
        if index.rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        reader.set_limit(INPUT_LIMIT);
        Ok(Box::new(payload::Payload::new(reader, index)))
    }
}
impl CreationCodec for CabCodec {
    fn encode(
        &self,
        _input: &dyn EncodeInput,
        _output: &mut dyn EncodeOutput,
        _settings: EncodeSettings<'_>,
        _cancel: &Cancellation,
        _progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}
impl RewriteCodec for CabCodec {
    fn rewrite(
        &self,
        _original: &mut dyn ReadSource,
        _input: RewriteInput<'_>,
        _output: &mut dyn RewriteOutput,
        _cancel: &Cancellation,
        _progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}
