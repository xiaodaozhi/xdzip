use crate::TarCodec;
use flate2::{Compression, write::GzEncoder};
use std::cell::Cell;
use std::io::{self, Read, Write};
use xdzip_codec_api::{
    Cancellation,
    encode::{
        CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    },
};

struct Input<'a> {
    reader: Box<dyn Read + 'a>,
    failed: &'a Cell<bool>,
    cancel: &'a Cancellation,
    name: &'a str,
    bytes: &'a mut u64,
    progress: &'a mut dyn FnMut(EncodeProgress<'_>),
}
impl Read for Input<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.cancel.check().map_err(io::Error::other)?;
        let n = self
            .reader
            .read(output)
            .inspect_err(|_| self.failed.set(true))?;
        *self.bytes = self
            .bytes
            .checked_add(n as u64)
            .ok_or_else(|| io::Error::other("input_limit"))?;
        (self.progress)(EncodeProgress {
            name: self.name,
            input_bytes: *self.bytes,
        });
        Ok(n)
    }
}
pub(crate) fn write_tar(
    output: &mut dyn Write,
    input: &dyn EncodeInput,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
) -> Result<(), EncodeError> {
    let mut builder = tar::Builder::new(output);
    let mut bytes = 0;
    for (index, entry) in input.entries().iter().enumerate() {
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        let mut header = tar::Header::new_ustar();
        header.set_entry_type(if entry.directory {
            tar::EntryType::Directory
        } else {
            tar::EntryType::Regular
        });
        header.set_size(entry.size);
        header.set_mtime(entry.modified_unix.max(0) as u64);
        header.set_mode(if entry.directory { 0o755 } else { 0o644 });
        header.set_uid(0);
        header.set_gid(0);
        header.set_cksum();
        let name = if entry.directory {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };
        // PAX defines path as UTF-8. GNU raw-name bytes otherwise depend on the
        // reference reader's locale (notably Windows' system bsdtar).
        let size = entry.size.to_string();
        let mut extensions = Vec::new();
        let extended_name = !name.is_ascii() || name.len() > 100;
        if extended_name {
            extensions.push(("path", name.as_bytes()));
        }
        if entry.size >= (1 << 33) {
            extensions.push(("size", size.as_bytes()));
        }
        if !extensions.is_empty() {
            builder
                .append_pax_extensions(extensions)
                .map_err(|_| EncodeError::Output)?;
        }
        let stored_name = if extended_name {
            format!("entry-{index}{}", if entry.directory { "/" } else { "" })
        } else {
            name.clone()
        };
        let reader: Box<dyn Read> = if entry.directory {
            Box::new(io::empty())
        } else {
            input.open(index)?
        };
        let failed = Cell::new(false);
        let mut reader = Input {
            reader,
            failed: &failed,
            cancel,
            name: &name,
            bytes: &mut bytes,
            progress,
        };
        let before = *reader.bytes;
        builder
            .append_data(&mut header, &stored_name, &mut reader)
            .map_err(|_| {
                if cancel.check().is_err() {
                    EncodeError::Cancelled
                } else if failed.get() {
                    EncodeError::Source
                } else {
                    EncodeError::Output
                }
            })?;
        // append_data reads to EOF; compare actual source bytes with the frozen size.
        if *reader.bytes - before != entry.size {
            return Err(EncodeError::Source);
        }
    }
    builder.finish().map_err(|_| EncodeError::Output)
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
            let mut encoder = GzEncoder::new(output, Compression::new(u32::from(settings.level)));
            write_tar(&mut encoder, input, cancel, progress)?;
            cancel.check().map_err(|_| EncodeError::Cancelled)?;
            encoder.finish().map_err(|_| EncodeError::Output)?;
            Ok(())
        } else {
            write_tar(output, input, cancel, progress)
        }
    }
}
