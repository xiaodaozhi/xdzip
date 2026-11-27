use crate::{
    line::Encoding,
    text::{self, Index},
};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
    encode::{
        CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
        RewriteCodec, RewriteInput, RewriteOutput,
    },
    names::{self, NameEncoding},
    payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader},
};
use xdzip_domain::identity::EntryId;
/// Read-only transport envelope. The host determines output paths and permissions.
pub struct UuCodec {
    /// UU for ordinary/mixed members, XXE for entirely XX-encoded members.
    pub format: Format,
}
fn format(index: &Index) -> Format {
    if index.records.iter().all(|row| row.encoding == Encoding::Xx) {
        Format::Xxe
    } else {
        Format::Uu
    }
}
fn project(
    index: &Index,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Vec<Entry>, ScanError> {
    let format = format(index);
    let mut rows = index
        .records
        .iter()
        .enumerate()
        .map(|(at, record)| {
            Ok(Entry {
                details: Default::default(),
                format,
                id: EntryId::try_from(at as u64 + 1).map_err(|_| ScanError::Limit)?,
                raw_name: record.name.clone(),
                decoded_name: None,
                name_encoding: NameEncoding::UnmarkedUtf8,
                display_name: String::new(),
                directory: false,
                link: false,
                unix_mode: 0,
                size: record.size,
                size_known: true,
                packed_size: record.end - record.start,
                method: match record.encoding {
                    Encoding::Uu => 0,
                    Encoding::Xx => 1,
                    Encoding::Base64 => 2,
                },
                modified_dos: 0,
                encrypted: false,
                password_supported: false,
                warning: false,
            })
        })
        .collect::<Result<Vec<_>, ScanError>>()?;
    names::resolve(
        &mut rows,
        &vec![true; index.records.len()],
        false,
        cancel,
        limits,
    )?;
    Ok(rows)
}
impl UuCodec {
    /// Determine the actual alphabet only after complete bounded envelope validation.
    pub fn detect(source: &mut dyn ReadSource, cancel: &Cancellation) -> Result<Format, ScanError> {
        Ok(format(&text::inspect(
            source,
            cancel,
            ScanLimits::default(),
        )?))
    }
}
impl MetadataCodec for UuCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        let index = text::inspect(source, cancel, limits)?;
        if format(&index) != self.format {
            return Err(ScanError::Invalid);
        }
        project(&index, cancel, limits)
    }
}
fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}
struct Reader<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    index: Index,
    rows: Vec<Entry>,
    input: u64,
    decoded: u64,
    failure: Option<DecodeError>,
}
impl PayloadCodec for UuCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        cancel: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let index = text::inspect(source, cancel, ScanLimits::default()).map_err(decode_error)?;
        let rows = project(&index, cancel, ScanLimits::default()).map_err(decode_error)?;
        if format(&index) != self.format || rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        let input = text::WORK_LIMIT
            .checked_sub(index.work)
            .ok_or(DecodeError::Limit)?;
        Ok(Box::new(Reader {
            source,
            cancel,
            index,
            rows,
            input,
            decoded: text::WORK_LIMIT,
            failure: None,
        }))
    }
}
impl Reader<'_> {
    fn deliver(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.cancel.check().map_err(decode_error)?;
        cancel.check().map_err(decode_error)?;
        let at = usize::try_from(entry.id.get() - 1).map_err(|_| DecodeError::Integrity)?;
        if self.rows.get(at) != Some(entry) {
            return Err(DecodeError::Integrity);
        }
        if !entry.payload_supported() {
            return Err(DecodeError::Unsupported);
        }
        let record = &self.index.records[at];
        let mut input = text::Input::new(
            self.source,
            self.cancel,
            record.start,
            record.end,
            self.input,
        )
        .map_err(decode_error)?;
        let mut sink_error = None;
        let result = text::body(&mut input, record.encoding == Encoding::Base64, |bytes| {
            let attempt = (|| {
                cancel.check().map_err(decode_error)?;
                self.decoded = self
                    .decoded
                    .checked_sub(bytes.len() as u64)
                    .ok_or(DecodeError::Limit)?;
                sink.write_chunk(bytes)
            })();
            if let Err(error) = attempt {
                sink_error = Some(error);
                return Err(ScanError::Io);
            }
            Ok(())
        });
        self.input = self
            .input
            .checked_sub(input.work)
            .ok_or(DecodeError::Limit)?;
        if let Some(error) = sink_error {
            return Err(error);
        }
        let (encoding, size) = result.map_err(decode_error)?;
        if encoding != record.encoding || size != record.size || input.position != record.end {
            return Err(DecodeError::Integrity);
        }
        Ok(())
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
        let result = self.deliver(entry, sink, cancel);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
}
impl CreationCodec for UuCodec {
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
impl RewriteCodec for UuCodec {
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
mod tests {
    use super::*;
    use std::io::Cursor;
    struct Sink;
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Ok(())
        }
    }
    #[test]
    fn repeated_payload_input_and_output_are_cumulative_exact_and_terminal() {
        let cancel = Cancellation::default();
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/uu/ordinary.uue"),
        )
        .unwrap();
        for (input_delta, output_delta, expected) in [
            (0, 0, Ok(())),
            (1, 0, Err(DecodeError::Limit)),
            (0, 1, Err(DecodeError::Limit)),
        ] {
            let mut source = Cursor::new(&bytes);
            let index = text::inspect(&mut source, &cancel, ScanLimits::default()).unwrap();
            let rows = project(&index, &cancel, ScanLimits::default()).unwrap();
            let row = rows[0].clone();
            let input = index.records[0].end - index.records[0].start - input_delta;
            let mut reader = Reader {
                source: &mut source,
                cancel: &cancel,
                index,
                rows,
                input,
                decoded: row.size - output_delta,
                failure: None,
            };
            assert_eq!(reader.decode(&row, &mut Sink, &cancel), expected);
            assert_eq!(
                reader.decode(&row, &mut Sink, &cancel),
                Err(DecodeError::Limit)
            );
            assert_eq!(
                reader.decode(&row, &mut Sink, &cancel),
                Err(DecodeError::Limit)
            );
        }
        let mut source = Cursor::new(&bytes);
        let codec = UuCodec { format: Format::Uu };
        let rows = codec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        let mut reader = codec.open_payload(&mut source, &rows, &cancel).unwrap();
        let second = Cancellation::default();
        second.cancel();
        assert_eq!(
            reader.decode(&rows[0], &mut Sink, &second),
            Err(DecodeError::Cancelled)
        );
        assert_eq!(
            reader.decode(&rows[0], &mut Sink, &cancel),
            Err(DecodeError::Cancelled)
        );
    }
}
