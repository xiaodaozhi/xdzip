use crate::source::{Budget, Reader};
use crate::types::{HeaderDecoder, HeaderPurpose, HeaderSource, MetadataSink, Streams};
use xdzip_codec_api::{Cancellation, ScanError};
use xdzip_compression::Crc32;

/// Bootstrap decoder for complete Copy-only metadata graphs, including chains.
/// Unsupported algorithms are never delegated to an external program.
pub struct CopyHeaderDecoder;
impl HeaderDecoder for CopyHeaderDecoder {
    fn decode_metadata(
        &mut self,
        _purpose: HeaderPurpose,
        source: &mut dyn HeaderSource,
        streams: &Streams,
        sink: &mut dyn MetadataSink,
        control: &Cancellation,
        password: Option<&str>,
    ) -> Result<(), ScanError> {
        for (index, folder) in streams.folders.iter().enumerate() {
            control.check()?;
            if folder.encrypted() && password.is_none() {
                return Err(ScanError::PasswordRequired);
            }
            if folder.coders.iter().any(|c| {
                c.method.as_bytes() != [0]
                    || !c.properties.is_empty()
                    || c.inputs != 1
                    || c.outputs != 1
            }) {
                return Err(ScanError::Unsupported);
            }
            if folder.packed_inputs.len() != 1 {
                return Err(ScanError::Invalid);
            }
            let pack = streams
                .packs
                .get(folder.packed_inputs[0].pack)
                .ok_or(ScanError::Invalid)?;
            if folder
                .output_sizes
                .iter()
                .any(|size| *size != pack.range.length)
            {
                return Err(ScanError::Invalid);
            }
            let mut crc = Crc32::new();
            let mut offset = 0u64;
            let mut block = [0u8; 32768];
            while offset < pack.range.length {
                control.check()?;
                let count = (pack.range.length - offset).min(block.len() as u64) as usize;
                source.read_exact_at(
                    pack.range
                        .offset
                        .checked_add(offset)
                        .ok_or(ScanError::Invalid)?,
                    &mut block[..count],
                )?;
                crc.update(&block[..count]);
                sink.write_folder(index, &block[..count])?;
                offset += count as u64;
            }
            if pack.crc.is_some_and(|expected| expected != crc.checksum()) {
                return Err(ScanError::Invalid);
            }
        }
        Ok(())
    }
}
struct Collector<'a> {
    streams: &'a Streams,
    control: &'a Cancellation,
    outputs: Vec<Vec<u8>>,
}
impl MetadataSink for Collector<'_> {
    fn write_folder(&mut self, folder: usize, bytes: &[u8]) -> Result<(), ScanError> {
        self.control.check()?;
        let expected = self
            .streams
            .folders
            .get(folder)
            .ok_or(ScanError::Invalid)?
            .unpacked_size();
        let output = self.outputs.get_mut(folder).ok_or(ScanError::Invalid)?;
        if (output.len() as u64)
            .checked_add(bytes.len() as u64)
            .is_none_or(|n| n > expected)
        {
            return Err(ScanError::Invalid);
        }
        output.extend_from_slice(bytes);
        Ok(())
    }
}
pub(crate) fn decode(
    reader: &mut Reader<'_>,
    streams: &Streams,
    budget: &mut Budget,
    decoder: &mut dyn HeaderDecoder,
    purpose: HeaderPurpose,
    password: Option<&str>,
) -> Result<Vec<Vec<u8>>, ScanError> {
    let mut outputs = budget.vec(streams.folders.len())?;
    for folder in &streams.folders {
        reader.control.check()?;
        let count = usize::try_from(folder.unpacked_size()).map_err(|_| ScanError::Limit)?;
        outputs.push(budget.vec(count)?);
    }
    let control = reader.control;
    let mut collector = Collector {
        streams,
        outputs,
        control,
    };
    decoder.decode_metadata(purpose, reader, streams, &mut collector, control, password)?;
    control.check()?;
    for (folder, output) in streams.folders.iter().zip(&collector.outputs) {
        control.check()?;
        if output.len() as u64 != folder.unpacked_size()
            || (folder.crc.is_some() && folder.crc != Some(crate::checked_crc(output, control)?))
        {
            return Err(ScanError::Invalid);
        }
        for stream in &folder.substreams {
            let start = usize::try_from(stream.offset).map_err(|_| ScanError::Invalid)?;
            let end = usize::try_from(
                stream
                    .offset
                    .checked_add(stream.length)
                    .ok_or(ScanError::Invalid)?,
            )
            .map_err(|_| ScanError::Invalid)?;
            let data = output.get(start..end).ok_or(ScanError::Invalid)?;
            if stream.crc.is_some() && stream.crc != Some(crate::checked_crc(data, control)?) {
                return Err(ScanError::Invalid);
            }
        }
    }
    // AdditionalStreams DataIndex addresses whole folder outputs. It does not
    // enumerate the substreams used for ordinary file mapping.
    Ok(collector.outputs)
}
