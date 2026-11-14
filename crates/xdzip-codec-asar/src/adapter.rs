use crate::{content, index};
use xdzip_codec_api::encode::{
    CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    RewriteCodec, RewriteInput, RewriteOutput,
};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
    names::NameEncoding,
};
use xdzip_domain::identity::EntryId;
/// Read-only ASAR archive export, without application or external-file authority.
pub struct AsarCodec;
fn project(index: &index::Index) -> Result<Vec<Entry>, ScanError> {
    index
        .records
        .iter()
        .enumerate()
        .map(|(at, record)| {
            Ok(Entry {
                format: Format::Asar,
                id: EntryId::try_from(at as u64 + 1).map_err(|_| ScanError::Limit)?,
                raw_name: record.path.as_bytes().to_vec(),
                decoded_name: None,
                name_encoding: NameEncoding::Utf8,
                display_name: record.display.clone(),
                directory: record.directory,
                link: record.link,
                unix_mode: 0,
                size: record.size,
                size_known: true,
                packed_size: if record.offset.is_some() {
                    record.size
                } else {
                    0
                },
                method: 0,
                modified_dos: 0,
                encrypted: false,
                password_supported: false,
                warning: record.warning,
            })
        })
        .collect()
}
impl MetadataCodec for AsarCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        project(&index::inspect(source, cancel, limits)?)
    }
}
struct Reader<'a> {
    content: content::Reader<'a>,
    rows: Vec<Entry>,
    failure: Option<DecodeError>,
}
impl PayloadCodec for AsarCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        cancel: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let content = content::Reader::open(source, cancel)?;
        let rows = project(content.index()).map_err(|_| DecodeError::Limit)?;
        if rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(Reader {
            content,
            rows,
            failure: None,
        }))
    }
}
impl PayloadReader for Reader<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = usize::try_from(entry.id.get() - 1)
            .map_err(|_| DecodeError::Integrity)
            .and_then(|at| {
                if self.rows.get(at) != Some(entry) {
                    return Err(DecodeError::Integrity);
                }
                self.content.decode_record(at, sink, cancel)
            });
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
}
impl CreationCodec for AsarCodec {
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
impl RewriteCodec for AsarCodec {
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn source() -> Cursor<Vec<u8>> {
        Cursor::new(
            std::fs::read(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/asar/shared.asar"),
            )
            .unwrap(),
        )
    }
    struct Sink;
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Ok(())
        }
    }
    #[test]
    fn host_snapshot_and_physical_identity_are_verified_before_content() {
        let cancel = Cancellation::default();
        let mut source = source();
        let rows = AsarCodec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows.iter().all(|row| row.format == Format::Asar
            && row.payload_supported()
            && row.unix_mode == 0));
        let mut reader = AsarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        reader.decode(&rows[0], &mut Sink, &cancel).unwrap();
        reader.decode(&rows[1], &mut Sink, &cancel).unwrap();
        drop(reader);
        let mut altered = rows.clone();
        altered[0].size += 1;
        assert_eq!(
            AsarCodec.open_payload(&mut source, &altered, &cancel).err(),
            Some(DecodeError::Integrity)
        );
        let mut reader = AsarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        assert_eq!(
            reader.decode(&altered[0], &mut Sink, &cancel),
            Err(DecodeError::Integrity)
        );
        assert_eq!(
            reader.decode(&rows[0], &mut Sink, &cancel),
            Err(DecodeError::Integrity)
        );
    }
}
