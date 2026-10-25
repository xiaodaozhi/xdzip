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

#[cfg(test)]
mod robustness_tests {
    use super::*;
    use std::io::{self, Cursor, Read, Seek, SeekFrom};

    const SOURCE_BUDGET: u64 = 2 * 1024 * 1024;
    const SINK_BUDGET: u64 = 1024 * 1024;

    struct BoundedSource<'a> {
        cursor: Cursor<&'a [u8]>,
        bytes_read: u64,
    }
    impl Read for BoundedSource<'_> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let remaining = SOURCE_BUDGET - self.bytes_read;
            if remaining == 0 && !output.is_empty() {
                return Err(io::Error::other(ScanError::Limit));
            }
            let take = output.len().min(remaining as usize);
            let count = self.cursor.read(&mut output[..take])?;
            self.bytes_read += count as u64;
            Ok(count)
        }
    }
    impl Seek for BoundedSource<'_> {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.cursor.seek(position)
        }
    }
    impl ReadSource for BoundedSource<'_> {}

    #[derive(Default)]
    struct BoundedSink {
        written: u64,
    }
    impl DecodedSink for BoundedSink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            let next = self
                .written
                .checked_add(bytes.len() as u64)
                .ok_or(DecodeError::Limit)?;
            if next > SINK_BUDGET {
                return Err(DecodeError::Limit);
            }
            self.written = next;
            Ok(())
        }
    }

    fn fixture(compressed: &[u8], expected_length: usize) -> Vec<u8> {
        let mut image = Vec::new();
        xdzip_compression::GzipReader::new(Cursor::new(compressed))
            .take(SOURCE_BUDGET + 1)
            .read_to_end(&mut image)
            .expect("fixed independent fixture gzip must verify");
        assert_eq!(image.len(), expected_length);
        image
    }

    // These are complete independently authored images, not parser-generated
    // field vectors. All phases share one physically metered source and sink.
    fn exercise(image: &[u8], format: Format) -> (bool, u64) {
        let control = Cancellation::default();
        let codec = OpticalCodec { format };
        let mut source = BoundedSource {
            cursor: Cursor::new(image),
            bytes_read: 0,
        };
        let limits = ScanLimits {
            entries: 256,
            name_bytes: 4096,
            total_name_bytes: 64 * 1024,
            display_name_bytes: 256 * 1024,
            read_bytes: 1024 * 1024,
        };
        let mut sink = BoundedSink::default();
        let mut complete = false;
        if let Ok(rows) = codec.scan(&mut source, &control, limits)
            && let Ok(mut payload) = codec.open_payload(&mut source, &rows, &control)
        {
            complete = true;
            for row in rows
                .iter()
                .filter(|row| !row.directory && row.payload_supported())
            {
                if payload.decode(row, &mut sink, &control).is_err() {
                    complete = false;
                    break;
                }
            }
        }
        assert!(source.bytes_read <= SOURCE_BUDGET);
        assert!(sink.written <= SINK_BUDGET);
        (complete, sink.written)
    }

    fn random(seed: &mut u32) -> u32 {
        *seed ^= *seed << 13;
        *seed ^= *seed >> 17;
        *seed ^= *seed << 5;
        *seed
    }

    fn bounded_fixture_mutations(compressed: &[u8], length: usize, format: Format, mut seed: u32) {
        let original = fixture(compressed, length);
        let (valid, payload_bytes) = exercise(&original, format);
        assert!(
            valid && payload_bytes != 0,
            "unmodified independent image must reach decode"
        );
        // Bias flips toward blocks that actually contain recorded information,
        // while selecting bytes without knowing descriptor offsets or layouts.
        let recorded: Vec<_> = original
            .chunks(2048)
            .enumerate()
            .filter(|(_, block)| block.iter().any(|byte| *byte != 0))
            .map(|(index, _)| index)
            .collect();
        let mut successes = 0usize;
        let mut rejections = 0usize;
        for case in 0..192usize {
            let mut changed = original.clone();
            if case < 128 {
                for _ in 0..1 + (case % 4) {
                    let block = recorded[random(&mut seed) as usize % recorded.len()];
                    let position =
                        (block * 2048 + random(&mut seed) as usize % 2048).min(changed.len() - 1);
                    changed[position] ^= 1 << (random(&mut seed) % 8);
                }
            } else {
                let cut = match case - 128 {
                    0 => 0,
                    1 => 1,
                    2 => 2047,
                    3 => 2048,
                    4 => 32767,
                    5 => 32768,
                    6 => length - 1,
                    7 => length,
                    _ => {
                        let boundary = random(&mut seed) as usize % (length / 2048 + 1) * 2048;
                        match random(&mut seed) % 3 {
                            0 => boundary.saturating_sub(1),
                            1 => boundary,
                            _ => boundary.saturating_add(1),
                        }
                        .min(length)
                    }
                };
                changed.truncate(cut);
            }
            let result = std::panic::catch_unwind(|| exercise(&changed, format));
            assert!(
                result.is_ok(),
                "bounded {format:?} mutation case {case}, seed {seed:08x} panicked"
            );
            if result.unwrap().0 {
                successes += 1;
            } else {
                rejections += 1;
            }
        }
        // Payload-only changes are allowed, while broken/truncated metadata
        // must also exercise unsuccessful scan/open/decode paths.
        assert!(successes != 0 && rejections != 0);
    }

    #[test]
    fn bounded_fixture_mutations_pycdlib_iso() {
        bounded_fixture_mutations(
            include_bytes!("../../../tests/fixtures/cab-iso/iso9660.iso.gz"),
            57_344,
            Format::Iso,
            0x1190_25a1,
        );
    }

    #[test]
    fn bounded_fixture_mutations_imapi_250_metadata() {
        bounded_fixture_mutations(
            include_bytes!("../../../tests/fixtures/optical/imapi/imapi-250.iso.gz"),
            1_507_328,
            Format::Udf,
            0x1670_260b,
        );
    }
}
