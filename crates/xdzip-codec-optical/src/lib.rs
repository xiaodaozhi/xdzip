//! Independent read-only ISO9660/Joliet/Rock Ridge and UDF image adapters.
mod image;
mod iso;
mod udf;

use image::{Extent, ImageReader, Index};
use xdzip_codec_api::Format;
use xdzip_codec_api::encode::{
    CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    RewriteCodec, RewriteInput, RewriteOutput,
};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, MetadataCodec, ReadSource, ScanError, ScanLimits};

const METADATA_LIMIT: u64 = 64 * 1024 * 1024;
const INPUT_LIMIT: u64 = 20 * 1024 * 1024 * 1024;
const FILE_LIMIT: u64 = 8 * 1024 * 1024 * 1024;

/// A read-only optical namespace selected by the host's bounded content probe.
pub struct OpticalCodec {
    /// ISO9660/Joliet/Rock Ridge or UDF, never inferred from the extension.
    pub format: Format,
}

fn parse(
    reader: &mut ImageReader<'_>,
    format: Format,
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    let index = match format {
        Format::Iso => iso::scan(reader, limits)?,
        Format::Udf => udf::scan(reader, limits)?,
        _ => return Err(ScanError::Unsupported),
    };
    for record in &index.records {
        reader.check()?;
        for extent in &record.extents {
            if let Extent::Data { offset, length } = extent
                && offset
                    .checked_add(*length)
                    .is_none_or(|end| end > reader.len())
            {
                return Err(ScanError::Invalid);
            }
        }
    }
    reader.resolve_names(index, limits)
}

impl MetadataCodec for OpticalCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        let mut reader = ImageReader::new(source, control, limits.read_bytes.min(METADATA_LIMIT))?;
        let index = parse(&mut reader, self.format, limits)?;
        Ok(index
            .records
            .into_iter()
            .map(|record| record.entry)
            .collect())
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

struct Payload<'a> {
    reader: ImageReader<'a>,
    index: Index,
    work: u64,
    buffer: Vec<u8>,
}
impl PayloadCodec for OpticalCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let mut reader = ImageReader::new(source, control, METADATA_LIMIT).map_err(decode_error)?;
        let index = parse(&mut reader, self.format, ScanLimits::default()).map_err(decode_error)?;
        if index.records.len() != snapshot.len()
            || index
                .records
                .iter()
                .zip(snapshot)
                .any(|(record, expected)| record.entry != *expected)
        {
            return Err(DecodeError::Integrity);
        }
        reader.set_budget(INPUT_LIMIT);
        Ok(Box::new(Payload {
            reader,
            index,
            work: INPUT_LIMIT,
            buffer: vec![0; 128 * 1024],
        }))
    }
}
impl PayloadReader for Payload<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        let index = entry
            .id
            .get()
            .checked_sub(1)
            .and_then(|id| usize::try_from(id).ok())
            .ok_or(DecodeError::Integrity)?;
        let record = self
            .index
            .records
            .get(index)
            .ok_or(DecodeError::Integrity)?;
        if record.entry != *entry {
            return Err(DecodeError::Integrity);
        }
        if entry.warning || entry.link || entry.encrypted {
            return Err(DecodeError::Unsupported);
        }
        if entry.size > FILE_LIMIT {
            return Err(DecodeError::Limit);
        }
        let mut total = 0u64;
        for extent in &record.extents {
            let length = match extent {
                Extent::Data { length, .. } | Extent::Zero { length } => *length,
                Extent::Inline(bytes) => bytes.len() as u64,
            };
            let mut at = 0;
            while at < length {
                control.check().map_err(decode_error)?;
                let take = (length - at).min(self.buffer.len() as u64) as usize;
                if take as u64 > self.work {
                    return Err(DecodeError::Limit);
                }
                match extent {
                    Extent::Data { offset, .. } => self
                        .reader
                        .read_exact_at(
                            offset.checked_add(at).ok_or(DecodeError::Integrity)?,
                            &mut self.buffer[..take],
                        )
                        .map_err(decode_error)?,
                    Extent::Zero { .. } => self.buffer[..take].fill(0),
                    Extent::Inline(bytes) => {
                        self.buffer[..take].copy_from_slice(&bytes[at as usize..at as usize + take])
                    }
                }
                self.work -= take as u64;
                sink.write_chunk(&self.buffer[..take])?;
                at += take as u64;
                total = total.checked_add(take as u64).ok_or(DecodeError::Limit)?;
                if total > entry.size {
                    return Err(DecodeError::Integrity);
                }
            }
        }
        if total != entry.size {
            return Err(DecodeError::Integrity);
        }
        control.check().map_err(decode_error)
    }
}
impl CreationCodec for OpticalCodec {
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
impl RewriteCodec for OpticalCodec {
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
