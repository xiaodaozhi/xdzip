//! Debian common-ar envelopes and bounded composition, without installation authority.
/// Common-ar envelope admission, without nested TAR interpretation.
pub mod envelope;
mod index;
mod payload;
mod source;
#[cfg(test)]
mod tests;
use std::{cell::Cell, rc::Rc};
use xdzip_codec_api::encode::{
    CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    RewriteCodec, RewriteInput, RewriteOutput,
};
use xdzip_codec_api::payload::{DecodeError, PayloadCodec, PayloadReader};
use xdzip_codec_api::{Cancellation, Entry, MetadataCodec, ReadSource, ScanError, ScanLimits};
use xdzip_codec_tar::TarCodec;

const STREAM_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
/// Read-only Debian package export views; never an installer or script runner.
pub struct DebCodec;
fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}
impl MetadataCodec for DebCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        let envelope = envelope::read_envelope(source, control, limits)?;
        let root = source::Root::new(
            source,
            control,
            envelope.length,
            limits
                .read_bytes
                .checked_sub(envelope.read_work)
                .ok_or(ScanError::Limit)?,
        );
        let decoded = Rc::new(Cell::new(limits.read_bytes));
        let mut children = [Vec::new(), Vec::new()];
        for (group, at) in [envelope.control, envelope.data].into_iter().enumerate() {
            let member = &envelope.members[at];
            let codec = TarCodec {
                filter: envelope::member_filter(&member.name, group == 0)?,
            };
            children[group] = codec.scan_with_budget(
                &mut root.range(member.offset, member.size)?,
                control,
                limits,
                decoded.clone(),
            )?;
        }
        control.check()?;
        index::assemble(&envelope, children, limits).map(|index| index.rows)
    }
}
impl PayloadCodec for DebCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let limits = ScanLimits {
            read_bytes: STREAM_LIMIT,
            ..ScanLimits::default()
        };
        let envelope = envelope::read_envelope(source, control, limits).map_err(decode_error)?;
        let root = source::Root::new(
            source,
            control,
            envelope.length,
            STREAM_LIMIT
                .checked_sub(envelope.read_work)
                .ok_or(DecodeError::Limit)?,
        );
        let preparation = Rc::new(Cell::new(STREAM_LIMIT));
        let delivery = Rc::new(Cell::new(STREAM_LIMIT));
        let mut children = [Vec::new(), Vec::new()];
        let mut readers = Vec::with_capacity(2);
        for (group, at) in [envelope.control, envelope.data].into_iter().enumerate() {
            let member = &envelope.members[at];
            let codec = TarCodec {
                filter: envelope::member_filter(&member.name, group == 0).map_err(decode_error)?,
            };
            let (rows, reader) = codec.index_payload_owned(
                Box::new(
                    root.range(member.offset, member.size)
                        .map_err(decode_error)?,
                ),
                control,
                preparation.clone(),
                delivery.clone(),
            )?;
            children[group] = rows;
            readers.push(reader);
        }
        let index = index::assemble(&envelope, children, limits).map_err(decode_error)?;
        if index.rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        root.begin_delivery(STREAM_LIMIT).map_err(decode_error)?;
        Ok(Box::new(payload::Reader {
            root,
            index,
            readers,
            members: envelope.members,
            delivery,
            failure: None,
        }))
    }
}
impl CreationCodec for DebCodec {
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
impl RewriteCodec for DebCodec {
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
