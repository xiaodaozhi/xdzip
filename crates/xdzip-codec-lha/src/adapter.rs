use crate::{Index, Reader, inspect, resolve_names};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
    encode::{
        CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
        RewriteCodec, RewriteInput, RewriteOutput,
    },
    payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader},
};
use xdzip_domain::identity::EntryId;
/// Read-only LHA/LZH export. The host retains names, filesystem and publication authority.
pub struct LhaCodec;
impl LhaCodec {
    /// Bounded family-marker proposal. The host must validate the complete container.
    pub fn candidate(
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
    ) -> Result<bool, ScanError> {
        use std::io::SeekFrom;
        cancel.check()?;
        if source
            .seek(SeekFrom::Start(0))
            .map_err(|e| ScanError::from_source_error(&e))?
            != 0
        {
            return Err(ScanError::SourceChanged);
        }
        let result = (|| {
            let mut bytes = [0; 8];
            let mut size = 0;
            while size < bytes.len() {
                cancel.check()?;
                match source.read(&mut bytes[size..]) {
                    Err(e)
                        if e.kind() == std::io::ErrorKind::Interrupted && e.get_ref().is_none() =>
                    {
                        continue;
                    }
                    Err(e) => return Err(ScanError::from_source_error(&e)),
                    Ok(0) => break,
                    Ok(n) if n <= bytes.len() - size => size += n,
                    Ok(_) => return Err(ScanError::Io),
                }
            }
            Ok((size == 1 && bytes[0] == 0)
                || (size >= 7 && matches!(&bytes[2..5], b"-lh" | b"-lz") && bytes[6] == b'-'))
        })();
        if source
            .seek(SeekFrom::Start(0))
            .map_err(|e| ScanError::from_source_error(&e))?
            != 0
        {
            return Err(ScanError::SourceChanged);
        }
        result
    }
}
fn method(marker: &[u8; 5]) -> u16 {
    match marker {
        b"-lh0-" | b"-lhd-" => 0,
        b"-lh1-" => 1,
        b"-lh2-" => 2,
        b"-lh3-" => 3,
        b"-lh4-" => 4,
        b"-lh5-" => 5,
        b"-lh6-" => 6,
        b"-lh7-" => 7,
        b"-lzs-" => 8,
        b"-lz5-" => 9,
        b"-lz4-" => 10,
        _ => u16::MAX,
    }
}
fn project(
    index: &Index,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Vec<Entry>, ScanError> {
    let names = resolve_names(index, cancel, limits)?;
    index
        .records
        .iter()
        .zip(names)
        .enumerate()
        .map(|(at, (record, name))| {
            cancel.check()?;
            let method = method(&record.method);
            let kind = record.unix_mode & 0xf000;
            let object = kind == 0
                || (kind == 0x8000 && !record.directory)
                || (kind == 0x4000 && record.directory);
            let warning = name.warning
                || !object
                || method == u16::MAX
                || (method == 2 && record.crc16.is_none());
            Ok(Entry {
                format: Format::Lha,
                id: EntryId::try_from(at as u64 + 1).map_err(|_| ScanError::Limit)?,
                raw_name: record.raw_name.clone(),
                decoded_name: name.decoded,
                name_encoding: name.encoding,
                display_name: name.display,
                directory: record.directory,
                link: kind == 0xa000,
                unix_mode: 0,
                size: record.size,
                size_known: true,
                packed_size: record.packed_size,
                method,
                modified_dos: record.modified_dos,
                encrypted: false,
                password_supported: false,
                warning,
            })
        })
        .collect()
}
impl MetadataCodec for LhaCodec {
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
impl PayloadCodec for LhaCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        cancel: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let reader = Reader::fresh(source, cancel)?;
        let rows =
            project(reader.index(), cancel, ScanLimits::default()).map_err(
                |error| match error {
                    ScanError::Cancelled => DecodeError::Cancelled,
                    ScanError::Limit => DecodeError::Limit,
                    _ => DecodeError::Integrity,
                },
            )?;
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
impl CreationCodec for LhaCodec {
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
impl RewriteCodec for LhaCodec {
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
