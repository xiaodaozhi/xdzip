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
        if let Some(password) = settings.password {
            return self.encode_encrypted(input, output, settings, password, cancel, progress);
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

impl ZipCodec {
    fn encode_encrypted(
        &self,
        input: &dyn EncodeInput,
        output: &mut dyn EncodeOutput,
        settings: EncodeSettings<'_>,
        password: &str,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        use zip::write::SimpleFileOptions;
        // WinZip AES-256 uses library-owned random salts and authentication. The
        // stream wrapper grants no seek, path or publication authority to zip-rs.
        let mut archive = zip::ZipWriter::new_stream(output);
        let mut total = 0;
        for (index, entry) in input.entries().iter().enumerate() {
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            let utc = rawzip::time::UtcDateTime::from_unix(entry.modified_unix);
            let time = zip::DateTime::from_date_and_time(
                utc.year(),
                utc.month(),
                utc.day(),
                utc.hour(),
                utc.minute(),
                utc.second(),
            )
            .unwrap_or_default();
            let base = SimpleFileOptions::default().last_modified_time(time);
            if entry.directory {
                archive
                    .add_directory(format!("{}/", entry.name), base.unix_permissions(0o755))
                    .map_err(|_| EncodeError::Output)?;
                continue;
            }
            let options = base
                .unix_permissions(0o644)
                .large_file(entry.size >= u32::MAX as u64)
                .compression_method(if settings.method == "store" {
                    zip::CompressionMethod::Stored
                } else {
                    zip::CompressionMethod::Deflated
                })
                .compression_level(if settings.method == "store" {
                    None
                } else {
                    Some(settings.level as i64)
                })
                .with_aes_encryption(zip::AesMode::Aes256, password);
            archive
                .start_file(entry.name.as_str(), options)
                .map_err(|_| EncodeError::Output)?;
            let mut reader = input.open(index)?;
            copying(
                &mut reader,
                &mut archive,
                entry.size,
                &entry.name,
                &mut total,
                cancel,
                progress,
            )?;
        }
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        archive.finish().map_err(|_| EncodeError::Output)?;
        Ok(())
    }

    /// Authenticate and decode each encrypted record in a host-owned staging file.
    /// This is deliberately separate from granting extraction authority to user archives.
    pub fn verify_encrypted<R: std::io::Read + std::io::Seek>(
        &self,
        reader: R,
        entries: &[xdzip_codec_api::encode::EncodeEntry],
        password: &str,
        cancel: &Cancellation,
    ) -> Result<(), EncodeError> {
        let mut archive = zip::ZipArchive::new(reader).map_err(|_| EncodeError::Invalid)?;
        if archive.len() != entries.len() {
            return Err(EncodeError::Invalid);
        }
        let mut buffer = [0u8; 64 * 1024];
        for (index, expected) in entries.iter().enumerate() {
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            let mut file = archive
                .by_index_decrypt(index, password.as_bytes())
                .map_err(|_| EncodeError::Invalid)?;
            let name = if expected.directory {
                format!("{}/", expected.name)
            } else {
                expected.name.clone()
            };
            if file.name_raw() != name.as_bytes()
                || file.is_dir() != expected.directory
                || file.size() != expected.size
                || (!expected.directory && !file.encrypted())
            {
                return Err(EncodeError::Invalid);
            }
            let mut bytes = 0u64;
            loop {
                cancel.check().map_err(|_| EncodeError::Cancelled)?;
                // Reading through EOF also verifies the AES authentication code.
                let count = file.read(&mut buffer).map_err(|_| EncodeError::Invalid)?;
                if count == 0 {
                    break;
                }
                bytes = bytes
                    .checked_add(count as u64)
                    .ok_or(EncodeError::Invalid)?;
                if bytes > expected.size {
                    return Err(EncodeError::Invalid);
                }
            }
            if bytes != expected.size {
                return Err(EncodeError::Invalid);
            }
        }
        Ok(())
    }
}
