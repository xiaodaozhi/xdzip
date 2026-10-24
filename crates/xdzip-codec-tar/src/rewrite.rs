// crates/xdzip-codec-tar/src/rewrite.rs

use crate::{STREAM_LIMIT, TarCodec};
use flate2::{Compression, read::MultiGzDecoder, write::GzEncoder};
use std::io::{Read, SeekFrom, Write};
use xdzip_codec_api::{Cancellation, ReadSource, encode::*};

fn copy_records(
    reader: &mut dyn Read,
    output: &mut dyn Write,
    keep: &[bool],
    cancel: &Cancellation,
) -> Result<(), EncodeError> {
    let mut index = 0usize;
    let mut pending = Vec::new();
    let mut consumed = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        let mut header = [0u8; 512];
        reader
            .read_exact(&mut header)
            .map_err(|_| EncodeError::Source)?;
        consumed += 512;
        if header.iter().all(|b| *b == 0) {
            break;
        }
        let h = tar::Header::from_byte_slice(&header);
        let size = h.entry_size().map_err(|_| EncodeError::Invalid)?;
        let padded = size.checked_add(511).ok_or(EncodeError::Invalid)? / 512 * 512;
        consumed = consumed.checked_add(padded).ok_or(EncodeError::Invalid)?;
        if consumed > STREAM_LIMIT {
            return Err(EncodeError::Invalid);
        }
        let kind = h.entry_type().as_byte();
        if matches!(kind, b'x' | b'L' | b'K') {
            if padded > 64 * 1024 || pending.len() as u64 + padded + 512 > 256 * 1024 {
                return Err(EncodeError::Unsupported);
            }
            pending.extend_from_slice(&header);
            let start = pending.len();
            pending.resize(start + padded as usize, 0);
            reader
                .read_exact(&mut pending[start..])
                .map_err(|_| EncodeError::Source)?;
            continue;
        }
        // Global PAX state would also change newly appended entries. Do not silently
        // normalize it or let archive-supplied attributes affect additions.
        if !matches!(kind, 0 | b'0' | b'5') {
            return Err(EncodeError::Unsupported);
        }
        let selected = *keep.get(index).ok_or(EncodeError::Invalid)?;
        index += 1;
        if selected {
            output
                .write_all(&pending)
                .and_then(|_| output.write_all(&header))
                .map_err(|_| EncodeError::Output)?;
        }
        pending.clear();
        let mut left = padded;
        while left != 0 {
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            let n = left.min(buffer.len() as u64) as usize;
            reader
                .read_exact(&mut buffer[..n])
                .map_err(|_| EncodeError::Source)?;
            if selected {
                output
                    .write_all(&buffer[..n])
                    .map_err(|_| EncodeError::Output)?;
            }
            left -= n as u64;
        }
    }
    if index != keep.len() || !pending.is_empty() {
        return Err(EncodeError::Unsupported);
    }
    // Finish reading to validate GZIP CRC/trailer, including concatenated members.
    loop {
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        let n = reader.read(&mut buffer).map_err(|_| EncodeError::Invalid)?;
        if n == 0 {
            break;
        }
        consumed += n as u64;
        if consumed > STREAM_LIMIT || buffer[..n].iter().any(|b| *b != 0) {
            return Err(EncodeError::Unsupported);
        }
    }
    Ok(())
}
impl RewriteCodec for TarCodec {
    fn rewrite(
        &self,
        original: &mut dyn ReadSource,
        input: RewriteInput<'_>,
        output: &mut dyn RewriteOutput,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        original
            .seek(SeekFrom::Start(0))
            .map_err(|_| EncodeError::Source)?;
        let mut reader: Box<dyn Read + '_> = if self.gzip {
            Box::new(MultiGzDecoder::new(original))
        } else {
            Box::new(original)
        };
        if self.gzip {
            let mut encoder =
                GzEncoder::new(output, Compression::new(u32::from(input.settings.level)));
            copy_records(&mut reader, &mut encoder, input.keep, cancel)?;
            crate::writer::write_tar(&mut encoder, input.additions, cancel, progress)?;
            encoder.finish().map_err(|_| EncodeError::Output)?;
        } else {
            copy_records(&mut reader, output, input.keep, cancel)?;
            crate::writer::write_tar(output, input.additions, cancel, progress)?;
        }
        Ok(())
    }
}
