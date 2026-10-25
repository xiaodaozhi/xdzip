use crate::{TarCodec, record};
use std::io::{Read, Write};
use xdzip_codec_api::{
    Cancellation,
    encode::{
        CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    },
};
use xdzip_compression::GzipWriter;

fn write_record(
    output: &mut dyn Write,
    header: &[u8; 512],
    data: &[u8],
) -> Result<(), EncodeError> {
    output
        .write_all(header)
        .and_then(|_| output.write_all(data))
        .and_then(|_| output.write_all(&[0; 512][..(512 - data.len() % 512) % 512]))
        .map_err(|_| EncodeError::Output)
}

pub(crate) fn write_tar(
    output: &mut dyn Write,
    input: &dyn EncodeInput,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
) -> Result<(), EncodeError> {
    let mut bytes = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    for (index, entry) in input.entries().iter().enumerate() {
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        if entry.directory && entry.size != 0 {
            return Err(EncodeError::Invalid);
        }
        let name = if entry.directory {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };
        if name.contains('\0') {
            return Err(EncodeError::Invalid);
        }
        let extended_name = !name.is_ascii() || name.len() > 100;
        let mut body = Vec::new();
        if extended_name {
            record::pax_field(&mut body, "path", name.as_bytes());
        }
        let extended_size = entry.size >= 1 << 33;
        if extended_size {
            record::pax_field(&mut body, "size", entry.size.to_string().as_bytes());
        }
        let time = entry.modified_unix.max(0) as u64;
        let extended_time = time >= 1 << 33;
        if extended_time {
            record::pax_field(&mut body, "mtime", time.to_string().as_bytes());
        }
        if body.len() > crate::EXTENSION_LIMIT as usize {
            return Err(EncodeError::Limit);
        }
        if !body.is_empty() {
            let header = record::header(b"PaxHeaders/xdzip", b'x', body.len() as u64, 0, 0o644)
                .map_err(|_| EncodeError::Invalid)?;
            write_record(output, &header, &body)?;
        }
        let stored_name = if extended_name {
            format!("entry-{index}{}", if entry.directory { "/" } else { "" })
        } else {
            name.clone()
        };
        let header = record::header(
            stored_name.as_bytes(),
            if entry.directory { b'5' } else { b'0' },
            if extended_size { 0 } else { entry.size },
            if extended_time { 0 } else { time },
            if entry.directory { 0o755 } else { 0o644 },
        )
        .map_err(|_| EncodeError::Invalid)?;
        output.write_all(&header).map_err(|_| EncodeError::Output)?;
        if !entry.directory {
            let mut reader = input.open(index)?;
            let mut left = entry.size;
            while left != 0 {
                cancel.check().map_err(|_| EncodeError::Cancelled)?;
                let take = left.min(buffer.len() as u64) as usize;
                let n = reader
                    .read(&mut buffer[..take])
                    .map_err(|_| EncodeError::Source)?;
                if n == 0 {
                    return Err(EncodeError::Source);
                }
                output
                    .write_all(&buffer[..n])
                    .map_err(|_| EncodeError::Output)?;
                bytes = bytes.checked_add(n as u64).ok_or(EncodeError::Limit)?;
                left -= n as u64;
                progress(EncodeProgress {
                    name: &name,
                    input_bytes: bytes,
                });
            }
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            // Probe one byte only: a growing source must fail without unbounded reads.
            if reader
                .read(&mut buffer[..1])
                .map_err(|_| EncodeError::Source)?
                != 0
            {
                return Err(EncodeError::Source);
            }
        }
        let padding = ((512 - entry.size % 512) % 512) as usize;
        output
            .write_all(&[0u8; 512][..padding])
            .map_err(|_| EncodeError::Output)?;
    }
    cancel.check().map_err(|_| EncodeError::Cancelled)?;
    output
        .write_all(&[0; 1024])
        .map_err(|_| EncodeError::Output)
}

impl CreationCodec for TarCodec {
    fn encode(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        if settings.password.is_some()
            || ![0, 1, 6, 9].contains(&settings.level)
            || settings.method != if self.gzip { "deflate" } else { "store" }
        {
            return Err(EncodeError::Unsupported);
        }
        if self.gzip {
            let mut encoder =
                GzipWriter::new(output, settings.level).map_err(|_| EncodeError::Invalid)?;
            write_tar(&mut encoder, input, cancel, progress)?;
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            encoder.finish().map_err(|_| EncodeError::Output)?;
            Ok(())
        } else {
            write_tar(output, input, cancel, progress)
        }
    }
}
