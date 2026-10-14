//! ZIP/ZIP64 streaming encoding using the admitted rawzip writer and Rust Deflate backend.
use crate::ZipCodec;
use flate2::{Compression, write::DeflateEncoder};
use std::io::{Read, Write};
use xdzip_codec_api::{
    Cancellation,
    encode::{
        CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    },
};

fn copying(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    expected: u64,
    name: &str,
    total: &mut u64,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
) -> Result<(), EncodeError> {
    let mut buffer = [0u8; 64 * 1024];
    let mut bytes = 0u64;
    loop {
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        let n = reader.read(&mut buffer).map_err(|_| EncodeError::Source)?;
        if n == 0 {
            break;
        }
        bytes = bytes.checked_add(n as u64).ok_or(EncodeError::Invalid)?;
        if bytes > expected {
            return Err(EncodeError::Source);
        }
        writer
            .write_all(&buffer[..n])
            .map_err(|_| EncodeError::Output)?;
        *total = total.checked_add(n as u64).ok_or(EncodeError::Invalid)?;
        progress(EncodeProgress {
            name,
            input_bytes: *total,
        });
    }
    if bytes != expected {
        return Err(EncodeError::Source);
    }
    Ok(())
}
impl CreationCodec for ZipCodec {
    fn encode(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        if !["store", "deflate"].contains(&settings.method) || ![1, 6, 9].contains(&settings.level)
        {
            return Err(EncodeError::Unsupported);
        }
        let mut zip = rawzip::ZipArchiveWriter::builder()
            .with_capacity(input.entries().len())
            .build(output);
        let mut total = 0u64;
        for (index, entry) in input.entries().iter().enumerate() {
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            let time = rawzip::time::UtcDateTime::from_unix(entry.modified_unix);
            if entry.directory {
                zip.new_dir(format!("{}/", entry.name))
                    .unix_permissions(0o755)
                    .last_modified(time)
                    .create()
                    .map_err(|_| EncodeError::Output)?;
                continue;
            }
            let method = if settings.method == "store" {
                rawzip::CompressionMethod::STORE
            } else {
                rawzip::CompressionMethod::DEFLATE
            };
            let (entry_writer, config) = zip
                .new_file(entry.name.as_str())
                .compression_method(method)
                .unix_permissions(0o644)
                .last_modified(time)
                .start()
                .map_err(|_| EncodeError::Output)?;
            let mut reader = input.open(index)?;
            if method == rawzip::CompressionMethod::STORE {
                let mut data = config.wrap(entry_writer);
                copying(
                    &mut reader,
                    &mut data,
                    entry.size,
                    &entry.name,
                    &mut total,
                    cancel,
                    progress,
                )?;
                let (writer, descriptor) = data.finish().map_err(|_| EncodeError::Output)?;
                writer.finish(descriptor).map_err(|_| EncodeError::Output)?;
            } else {
                let encoder =
                    DeflateEncoder::new(entry_writer, Compression::new(u32::from(settings.level)));
                let mut data = config.wrap(encoder);
                copying(
                    &mut reader,
                    &mut data,
                    entry.size,
                    &entry.name,
                    &mut total,
                    cancel,
                    progress,
                )?;
                let (encoder, descriptor) = data.finish().map_err(|_| EncodeError::Output)?;
                // flush is not StreamEnd: finish the compressor before writing its descriptor.
                let writer = encoder.finish().map_err(|_| EncodeError::Output)?;
                writer.finish(descriptor).map_err(|_| EncodeError::Output)?;
            }
        }
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        zip.finish().map_err(|_| EncodeError::Output)?;
        Ok(())
    }
}
