use crate::{Index, Reader, inspect};
use std::io::{ErrorKind, SeekFrom};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
    encode::{
        CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
        RewriteCodec, RewriteInput, RewriteOutput,
    },
    names::{self, NameEncoding, NameSample},
    payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader},
};
use xdzip_domain::identity::EntryId;
/// Independent read-only ARC profiles. No output or filesystem authority is acquired.
pub struct ArcCodec;
impl ArcCodec {
    /// Propose sequential ARC or an exact empty marker, then require a complete scan.
    pub fn candidate(
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
    ) -> Result<bool, ScanError> {
        cancel.check()?;
        reset(source)?;
        let result = (|| {
            let mut bytes = [0; 29];
            let mut used = 0;
            while used < bytes.len() {
                cancel.check()?;
                match source.read(&mut bytes[used..]) {
                    Err(e) if e.kind() == ErrorKind::Interrupted && e.get_ref().is_none() => {
                        continue;
                    }
                    Err(e) => return Err(crate::header::source_error(&e)),
                    Ok(0) => break,
                    Ok(n) if n <= bytes.len() - used => used += n,
                    Ok(_) => return Err(ScanError::Io),
                }
            }
            if used < 2 || bytes[0] != 0x1a {
                return Ok(false);
            }
            if bytes[1] == 0 {
                return Ok(used == 2);
            }
            let required = if bytes[1] == 1 { 25 } else { 29 };
            Ok(used >= required && bytes[2] != 0 && bytes[2..15].contains(&0))
        })();
        reset(source)?;
        result
    }
}
fn reset(source: &mut dyn ReadSource) -> Result<(), ScanError> {
    if source
        .seek(SeekFrom::Start(0))
        .map_err(|e| crate::header::source_error(&e))?
        != 0
    {
        return Err(ScanError::SourceChanged);
    }
    Ok(())
}
fn project(
    index: &Index,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Vec<Entry>, ScanError> {
    cancel.check()?;
    if index.records.len() > limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
        return Err(ScanError::Limit);
    }
    let mut sample = NameSample::default();
    for record in &index.records {
        cancel.check()?;
        sample.observe_bytes(&record.raw_name);
    }
    let resolution = sample.finish(false);
    let mut rows = Vec::new();
    rows.try_reserve_exact(index.records.len())
        .map_err(|_| ScanError::Limit)?;
    let mut retained = 0usize;
    for (at, record) in index.records.iter().enumerate() {
        cancel.check()?;
        let (decoded_name, name_encoding, display_name, name_warning) =
            if let Some((text, encoding)) = resolution.decode_unmarked(&record.raw_name) {
                let text = text.replace('\\', "/");
                if text.len() > limits.name_bytes.min(4096) {
                    return Err(ScanError::Limit);
                }
                let warning = names::suspicious(&text, false);
                let display = names::display(&text);
                let decoded = (text.as_bytes() != record.raw_name).then_some(text);
                (decoded, encoding, display, warning)
            } else {
                (
                    None,
                    NameEncoding::Unresolved,
                    names::escaped_bytes(&record.raw_name),
                    true,
                )
            };
        retained = retained
            .checked_add(display_name.capacity())
            .and_then(|n| n.checked_add(decoded_name.as_ref().map_or(0, String::capacity)))
            .ok_or(ScanError::Limit)?;
        if retained > limits.display_name_bytes.min(16 * 1024 * 1024) {
            return Err(ScanError::Limit);
        }
        rows.push(Entry {
            format: Format::Arc,
            id: EntryId::try_from(at as u64 + 1).map_err(|_| ScanError::Limit)?,
            raw_name: record.raw_name.clone(),
            decoded_name,
            name_encoding,
            display_name,
            directory: false,
            link: false,
            unix_mode: 0,
            size: record.size,
            size_known: true,
            packed_size: record.packed_size,
            method: u16::from(record.method),
            modified_dos: record.modified_dos,
            encrypted: false,
            password_supported: false,
            warning: name_warning || !record.payload_supported(),
        });
    }
    Ok(rows)
}
impl MetadataCodec for ArcCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        project(&inspect(source, cancel, limits)?, cancel, limits)
    }
}
struct Payload<'a> {
    reader: Reader<'a>,
    rows: Vec<Entry>,
    failure: Option<DecodeError>,
}
impl PayloadCodec for ArcCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        cancel: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let reader = Reader::fresh(source, cancel)?;
        let rows = project(reader.index(), cancel, ScanLimits::default()).map_err(|e| match e {
            ScanError::Cancelled => DecodeError::Cancelled,
            ScanError::Limit => DecodeError::Limit,
            _ => DecodeError::Integrity,
        })?;
        if rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(Payload {
            reader,
            rows,
            failure: None,
        }))
    }
}
impl PayloadReader for Payload<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = (|| {
            let at = usize::try_from(entry.id.get() - 1).map_err(|_| DecodeError::Integrity)?;
            if self.rows.get(at) != Some(entry) {
                return Err(DecodeError::Integrity);
            }
            if !entry.payload_supported() {
                return Err(DecodeError::Unsupported);
            }
            self.reader.decode(at, sink, cancel)
        })();
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
}
impl CreationCodec for ArcCodec {
    fn encode(
        &self,
        _: &dyn EncodeInput,
        _: &mut dyn EncodeOutput,
        _: EncodeSettings<'_>,
        _: &Cancellation,
        _: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}
impl RewriteCodec for ArcCodec {
    fn rewrite(
        &self,
        _: &mut dyn ReadSource,
        _: RewriteInput<'_>,
        _: &mut dyn RewriteOutput,
        _: &Cancellation,
        _: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}
#[cfg(test)]
mod tests;
