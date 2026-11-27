use crate::{content, framing, index, work::Work};
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

/// Authenticated, read-only AES Crypt envelope. The host owns paths and publication.
pub struct AesCryptCodec;
fn project(
    metadata: &index::Metadata,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Vec<Entry>, ScanError> {
    let mut rows = vec![Entry {
        details: xdzip_codec_api::EntryDetails {
            encryption: Some("AES-256".to_owned()),
            ..Default::default()
        },
        format: Format::AesCrypt,
        id: EntryId::try_from(1).expect("nonzero identity"),
        raw_name: metadata.name.as_bytes().to_vec(),
        decoded_name: None,
        name_encoding: NameEncoding::Utf8,
        display_name: String::new(),
        directory: false,
        link: false,
        unix_mode: 0,
        size: metadata.size,
        size_known: true,
        packed_size: metadata.packed_size,
        method: 0,
        modified_dos: 0,
        encrypted: true,
        password_supported: true,
        warning: false,
    }];
    names::resolve(&mut rows, &[true], false, cancel, limits)?;
    Ok(rows)
}
impl AesCryptCodec {
    /// Bounded framing only, without deriving keys or reading ciphertext. Always rewind.
    /// Used solely to disambiguate a TAR member literally named AES.
    pub fn probe_header(
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
    ) -> Result<(), ScanError> {
        let result = framing::parse(source, cancel, &mut Work::new(Default::default())).map(|_| ());
        let rewind = crate::work::seek(source, 0);
        result.and(rewind)
    }
}
impl MetadataCodec for AesCryptCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        self.scan_with_password(source, cancel, limits, None)
    }
    fn scan_with_password(
        &self,
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
        limits: ScanLimits,
        password: Option<&str>,
    ) -> Result<Vec<Entry>, ScanError> {
        let index = index::inspect(
            source,
            password,
            cancel,
            limits,
            &mut Work::new(Default::default()),
        )?;
        project(&index.metadata, cancel, limits)
    }
}
struct Reader<'a> {
    content: content::Reader<'a>,
    row: Entry,
    failure: Option<DecodeError>,
}
impl PayloadCodec for AesCryptCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        cancel: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        self.open_payload_with_password(source, snapshot, cancel, None)
    }
    fn open_payload_with_password<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        cancel: &'a Cancellation,
        password: Option<&str>,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let content = content::Reader::open(source, password, cancel)?;
        let rows = project(content.metadata(), cancel, ScanLimits::default())
            .map_err(content::decode_error)?;
        if rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(Reader {
            content,
            row: rows.into_iter().next().expect("one file"),
            failure: None,
        }))
    }
}
impl PayloadReader for Reader<'_> {
    fn decode(
        &mut self,
        row: &Entry,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = if row != &self.row || !row.payload_supported() {
            Err(DecodeError::Integrity)
        } else {
            self.content.decode(sink, cancel)
        };
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
}
impl CreationCodec for AesCryptCodec {
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
impl RewriteCodec for AesCryptCodec {
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
    use std::io::{Cursor, Seek};
    #[test]
    fn changed_frozen_identity_cannot_deliver_and_poisoning_is_permanent() {
        struct Sink;
        impl DecodedSink for Sink {
            fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
                panic!("unadmitted output");
            }
            fn charge_auxiliary(&mut self, _: u64) -> Result<(), DecodeError> {
                panic!("unadmitted work");
            }
        }
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/aescrypt/v3-phrase.aes"),
        )
        .unwrap();
        let cancel = Cancellation::default();
        let codec = AesCryptCodec;
        let password = Some("owned-aes-fixture");
        let mut source = Cursor::new(&bytes);
        let rows = codec
            .scan_with_password(&mut source, &cancel, ScanLimits::default(), password)
            .unwrap();
        let mut changed = rows.clone();
        changed[0].raw_name.push(b'x');
        assert!(matches!(
            codec.open_payload_with_password(&mut source, &changed, &cancel, password),
            Err(DecodeError::Integrity)
        ));
        let mut reader = codec
            .open_payload_with_password(&mut source, &rows, &cancel, password)
            .unwrap();
        assert_eq!(
            reader.decode(&changed[0], &mut Sink, &cancel),
            Err(DecodeError::Integrity)
        );
        assert_eq!(
            reader.decode(&rows[0], &mut Sink, &cancel),
            Err(DecodeError::Integrity)
        );
        drop(reader);
        assert!(AesCryptCodec::probe_header(&mut source, &cancel).is_ok());
        assert_eq!(source.stream_position().unwrap(), 0);
        let mut bad = Cursor::new(b"AES\x04\0invalid");
        assert_eq!(
            AesCryptCodec::probe_header(&mut bad, &cancel),
            Err(ScanError::Unsupported)
        );
        assert_eq!(bad.stream_position().unwrap(), 0);
    }
}
