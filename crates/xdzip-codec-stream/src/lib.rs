//! Single-file streams and controlled composition, without filesystem authority.
pub mod controlled;
mod framed;
#[cfg(test)]
mod framed_tests;
#[cfg(test)]
mod tests;
use controlled::{ControlledStream, Filter};
use std::io::Read;
use xdzip_codec_api::encode::{
    CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeOutput, EncodeProgress,
    EncodeSettings, RewriteCodec, RewriteInput, RewriteOutput,
};
use xdzip_codec_api::names::NameEncoding;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
};
use xdzip_compression::GzipWriter;
use xdzip_domain::identity::EntryId;

/// One ordinary-file compression stream. Header filenames never grant output names.
pub struct StreamCodec {
    /// An admitted ordinary-file stream format, independent of its filename.
    pub format: Format,
}
/// Fully validated observed stream metadata used for empty-TAR discrimination.
pub struct StreamInfo {
    /// One virtual ordinary-file row.
    pub entry: Entry,
    /// Whether every decoded byte was zero.
    pub all_zero: bool,
}
const STREAM_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
fn candidate(source: &dyn ReadSource, format: Format) -> String {
    source
        .source_name_hint()
        .and_then(|name| {
            let suffix = format
                .descriptor()
                .extensions
                .iter()
                .filter(|suffix| name.to_ascii_lowercase().ends_with(**suffix))
                .max_by_key(|suffix| suffix.len())?;
            let stem = name.get(..name.len().checked_sub(suffix.len())?)?;
            (!stem.is_empty()
                && stem != "."
                && stem != ".."
                && !stem
                    .chars()
                    .any(|c| c.is_control() || matches!(c, '/' | '\\' | ':')))
            .then(|| stem.to_owned())
        })
        .unwrap_or_else(|| "content".into())
}
impl StreamCodec {
    /// Resolve the admitted single-file outer filter without granting paths.
    pub fn filter(&self) -> Result<Filter, ScanError> {
        match self.format {
            Format::Gzip => Ok(Filter::Gzip),
            Format::Bzip2 => Ok(Filter::Bzip2),
            Format::Lzma => Ok(Filter::Lzma),
            Format::Lzip => Ok(Filter::Lzip),
            _ => Err(ScanError::Unsupported),
        }
    }
    fn method(&self) -> u16 {
        match self.format {
            Format::Gzip => 8,
            Format::Bzip2 => 12,
            _ => 14,
        }
    }

    /// Drain the entire stream with independent input/output bounds before publication.
    pub fn inspect(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<StreamInfo, ScanError> {
        if limits.entries == 0 {
            return Err(ScanError::Limit);
        }
        let filter = self.filter()?;
        let name = candidate(source, self.format);
        let display_name = xdzip_codec_api::names::display(&name);
        if name.len() > limits.name_bytes
            || name.len() > limits.total_name_bytes
            || display_name.len() > limits.display_name_bytes
        {
            return Err(ScanError::Limit);
        }
        let packed_size = source
            .seek(std::io::SeekFrom::End(0))
            .map_err(|_| ScanError::Io)?;
        let mut stream = ControlledStream::new(
            source,
            control,
            filter,
            limits.read_bytes,
            limits.read_bytes,
        )?;
        let mut bytes = [0; 64 * 1024];
        let mut all_zero = true;
        loop {
            let n = stream
                .read(&mut bytes)
                .map_err(|_| stream.failure().unwrap_or(ScanError::Invalid))?;
            if n == 0 {
                break;
            }
            all_zero &= bytes[..n].iter().all(|b| *b == 0);
        }
        Ok(StreamInfo {
            all_zero,
            entry: Entry {
                format: self.format,
                id: EntryId::try_from(1).expect("nonzero"),
                raw_name: name.into_bytes(),
                display_name,
                decoded_name: None,
                name_encoding: NameEncoding::Utf8,
                directory: false,
                link: false,
                unix_mode: 0,
                size: stream.position(),
                size_known: true,
                packed_size,
                method: self.method(),
                modified_dos: 0,
                encrypted: false,
                password_supported: false,
                warning: false,
            },
        })
    }
}
impl MetadataCodec for StreamCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        self.inspect(source, control, limits)
            .map(|info| vec![info.entry])
    }
}
fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Io => DecodeError::SourceIo,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Cancelled => DecodeError::Cancelled,
        _ => DecodeError::Integrity,
    }
}
struct Reader<'a> {
    stream: ControlledStream<'a>,
    entry: Entry,
    consumed: bool,
}
impl PayloadCodec for StreamCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let [entry] = snapshot else {
            return Err(DecodeError::Integrity);
        };
        if entry.format != self.format
            || entry.id.get() != 1
            || entry.directory
            || entry.link
            || entry.encrypted
            || entry.warning
            || !entry.size_known
            || entry.method != self.method()
        {
            return Err(DecodeError::Integrity);
        }
        if source
            .seek(std::io::SeekFrom::End(0))
            .map_err(|_| DecodeError::SourceIo)?
            != entry.packed_size
        {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(Reader {
            stream: ControlledStream::new(
                source,
                control,
                self.filter().map_err(decode_error)?,
                STREAM_LIMIT,
                entry.size.min(STREAM_LIMIT),
            )
            .map_err(decode_error)?,
            entry: entry.clone(),
            consumed: false,
        }))
    }
}
impl PayloadReader for Reader<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if self.consumed || entry != &self.entry {
            return Err(DecodeError::Integrity);
        }
        self.consumed = true;
        let mut bytes = [0; 64 * 1024];
        loop {
            control.check().map_err(decode_error)?;
            let n = self
                .stream
                .read(&mut bytes)
                .map_err(|_| decode_error(self.stream.failure().unwrap_or(ScanError::Invalid)))?;
            if n == 0 {
                break;
            }
            sink.write_chunk(&bytes[..n])?;
        }
        if self.stream.position() != entry.size {
            return Err(DecodeError::Integrity);
        }
        Ok(())
    }
}
impl CreationCodec for StreamCodec {
    fn encode(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        if self.format != Format::Gzip
            || settings.method != "deflate"
            || ![0, 1, 6, 9].contains(&settings.level)
            || settings.password.is_some()
            || settings.solid
            || settings.encrypt_headers
        {
            return Err(EncodeError::Unsupported);
        }
        let [
            EncodeEntry {
                name,
                directory: false,
                size,
                ..
            },
        ] = input.entries()
        else {
            return Err(EncodeError::Unsupported);
        };
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        let mut source = input.open(0)?;
        let mut writer =
            GzipWriter::new(output, settings.level).map_err(|_| EncodeError::Output)?;
        let mut bytes = [0; 64 * 1024];
        let mut total = 0u64;
        loop {
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            let n = source.read(&mut bytes).map_err(|_| EncodeError::Source)?;
            if n == 0 {
                break;
            }
            total = total
                .checked_add(n as u64)
                .filter(|n| n <= size)
                .ok_or(EncodeError::Source)?;
            std::io::Write::write_all(&mut writer, &bytes[..n]).map_err(|_| EncodeError::Output)?;
            progress(EncodeProgress {
                name,
                input_bytes: total,
            });
        }
        if total != *size {
            return Err(EncodeError::Source);
        }
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        writer.finish().map_err(|_| EncodeError::Output)?;
        Ok(())
    }
}
impl RewriteCodec for StreamCodec {
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
