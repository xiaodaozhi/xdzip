// crates/xdzip-codec-zip/src/rewrite.rs

use crate::ZipCodec;
use std::io::{self, Read, SeekFrom, Write};
use xdzip_codec_api::{Cancellation, ReadSource, encode::*};

struct Checked<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
}
impl Read for Checked<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.cancel.check().map_err(io::Error::other)?;
        self.source.read(bytes)
    }
}
impl io::Seek for Checked<'_> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.source.seek(pos)
    }
}
impl RewriteCodec for ZipCodec {
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
        let mut source = zip::ZipArchive::new(Checked {
            source: original,
            cancel,
        })
        .map_err(|_| EncodeError::Invalid)?;
        if source.len() != input.keep.len()
            || source.offset() != 0
            || (!input.renames.is_empty() && input.renames.len() != input.keep.len())
        {
            return Err(EncodeError::Unsupported);
        }
        let mut writer = zip::ZipWriter::new(output);
        writer
            .set_raw_comment(source.comment().into())
            .map_err(|_| EncodeError::Unsupported)?;
        for (index, keep) in input.keep.iter().enumerate() {
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            let file = source
                .by_index_raw(index)
                .map_err(|_| EncodeError::Invalid)?;
            if *keep {
                progress(EncodeProgress {
                    name: &file.name().map_err(|_| EncodeError::Unsupported)?,
                    input_bytes: 0,
                });
                if let Some(Some(name)) = input.renames.get(index) {
                    writer.raw_copy_file_rename(file, name)
                } else {
                    writer.raw_copy_file(file)
                }
                .map_err(|_| EncodeError::Unsupported)?;
            }
        }
        let mut bytes = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        for (index, entry) in input.additions.entries().iter().enumerate() {
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            let utc = rawzip::time::UtcDateTime::from_unix(entry.modified_unix);
            let timestamp = zip::DateTime::from_date_and_time(
                utc.year(),
                utc.month(),
                utc.day(),
                utc.hour(),
                utc.minute(),
                utc.second(),
            )
            .unwrap_or_default();
            let options = zip::write::SimpleFileOptions::default()
                .last_modified_time(timestamp)
                .compression_method(zip::CompressionMethod::Deflated)
                .compression_level(Some(i64::from(input.settings.level)))
                .unix_permissions(if entry.directory { 0o755 } else { 0o644 })
                .large_file(entry.size >= u32::MAX as u64);
            if entry.directory {
                writer
                    .add_directory(format!("{}/", entry.name), options)
                    .map_err(|_| EncodeError::Output)?;
            } else {
                writer
                    .start_file(&entry.name, options)
                    .map_err(|_| EncodeError::Output)?;
                let mut reader = input.additions.open(index)?;
                let mut size = 0u64;
                loop {
                    cancel.check().map_err(|_| EncodeError::Cancelled)?;
                    let n = reader.read(&mut buffer).map_err(|_| EncodeError::Source)?;
                    if n == 0 {
                        break;
                    }
                    size = size.checked_add(n as u64).ok_or(EncodeError::Invalid)?;
                    if size > entry.size {
                        return Err(EncodeError::Invalid);
                    }
                    writer
                        .write_all(&buffer[..n])
                        .map_err(|_| EncodeError::Output)?;
                    bytes += n as u64;
                    progress(EncodeProgress {
                        name: &entry.name,
                        input_bytes: bytes,
                    });
                }
                if size != entry.size {
                    return Err(EncodeError::Source);
                }
            }
        }
        writer.finish().map_err(|_| EncodeError::Output)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    struct Empty;
    impl EncodeInput for Empty {
        fn entries(&self) -> &[EncodeEntry] {
            &[]
        }
        fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            Err(EncodeError::Invalid)
        }
    }
    #[test]
    fn raw_copy_preserves_compressed_payload_comments_time_mode_and_custom_metadata() {
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        writer.set_comment("archive comment").unwrap();
        let mut options = zip::write::FullFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .last_modified_time(zip::DateTime::from_date_and_time(2020, 3, 4, 5, 6, 8).unwrap())
            .unix_permissions(0o755)
            .with_file_comment("entry comment");
        options.add_extra_field(0xf00d, b"opaque", false).unwrap();
        writer.start_file("original.txt", options).unwrap();
        writer.write_all(b"unchanged payload").unwrap();
        let original = writer.finish().unwrap().into_inner();
        for rename in [false, true] {
            let renames = [rename.then(|| "移动/renamed.txt".to_owned())];
            let mut output = Cursor::new(Vec::new());
            ZipCodec
                .rewrite(
                    &mut Cursor::new(original.clone()),
                    RewriteInput {
                        keep: &[true],
                        renames: &renames,
                        additions: &Empty,
                        settings: EncodeSettings {
                            method: "deflate",
                            level: 6,
                            password: None,
                        },
                    },
                    &mut output,
                    &Cancellation::default(),
                    &mut |_| {},
                )
                .unwrap();
            let rewritten = output.into_inner();
            let mut old = zip::ZipArchive::new(Cursor::new(&original)).unwrap();
            let mut new = zip::ZipArchive::new(Cursor::new(&rewritten)).unwrap();
            assert_eq!(old.comment(), new.comment());
            let a = old.by_index_raw(0).unwrap();
            let b = new.by_index_raw(0).unwrap();
            assert_eq!(
                b.name().unwrap(),
                if rename {
                    "移动/renamed.txt"
                } else {
                    "original.txt"
                }
            );
            assert_eq!(a.comment(), b.comment());
            assert_eq!(a.last_modified(), b.last_modified());
            assert_eq!(a.unix_mode(), b.unix_mode());
            assert_eq!(a.extra_data(), b.extra_data());
            let a_start = a.data_start().unwrap() as usize;
            let b_start = b.data_start().unwrap() as usize;
            assert_eq!(
                &original[a_start..a_start + a.compressed_size() as usize],
                &rewritten[b_start..b_start + b.compressed_size() as usize]
            );
        }
    }
}
