// crates/xdzip-codec-tar/src/rewrite.rs

use crate::{STREAM_LIMIT, TarCodec, record};
#[cfg(test)]
use flate2::read::MultiGzDecoder;
#[cfg(test)]
use flate2::{Compression, write::GzEncoder};
use std::io::{Read, Write};
use xdzip_codec_api::{Cancellation, ReadSource, encode::*};
use xdzip_compression::GzipWriter;

// Rewrite path and its charset declaration; preserve other admitted PAX record bytes.
// GNU long paths become an explicit UTF-8 PAX path. Unsupported metadata fails
// before the host can publish the staged archive.
fn renamed_header(
    output: &mut dyn Write,
    pending: &[u8],
    header: &[u8; 512],
    name: &str,
    index: usize,
) -> Result<(), EncodeError> {
    let mut body = Vec::new();
    let mut at = 0usize;
    let mut binary_charset = false;
    while at < pending.len() {
        let block: &[u8; 512] = pending
            .get(at..at + 512)
            .and_then(|b| b.try_into().ok())
            .ok_or(EncodeError::Invalid)?;
        let h = record::Record(block);
        h.validate().map_err(|_| EncodeError::Invalid)?;
        let size = usize::try_from(h.size().map_err(|_| EncodeError::Invalid)?)
            .map_err(|_| EncodeError::Invalid)?;
        let data = pending
            .get(at + 512..at + 512 + size)
            .ok_or(EncodeError::Invalid)?;
        match h.kind() {
            b'L' => {}
            b'x' => {
                let mut pos = 0;
                while pos < data.len() {
                    let space = data[pos..]
                        .iter()
                        .position(|b| *b == b' ')
                        .ok_or(EncodeError::Invalid)?;
                    let length = std::str::from_utf8(&data[pos..pos + space])
                        .map_err(|_| EncodeError::Invalid)?
                        .parse::<usize>()
                        .map_err(|_| EncodeError::Invalid)?;
                    let end = pos
                        .checked_add(length)
                        .filter(|end| *end <= data.len() && *end > pos + space + 2)
                        .ok_or(EncodeError::Invalid)?;
                    let record = &data[pos + space + 1..end];
                    if record.last() != Some(&b'\n') {
                        return Err(EncodeError::Invalid);
                    }
                    let equals = record
                        .iter()
                        .position(|b| *b == b'=')
                        .ok_or(EncodeError::Invalid)?;
                    match &record[..equals] {
                        b"path" => {}
                        b"hdrcharset" => {
                            binary_charset |= &record[equals + 1..record.len() - 1] == b"BINARY";
                        }
                        _ => body.extend_from_slice(&data[pos..end]),
                    }
                    pos = end;
                }
            }
            _ => return Err(EncodeError::Unsupported),
        }
        at += 512 + size.div_ceil(512) * 512;
    }
    // The replacement path is explicit PAX UTF-8. Do not retain a BINARY
    // declaration for it, or silently reinterpret other non-UTF8 PAX values.
    if binary_charset
        && record::Pax(&body).any(|item| {
            item.is_err() || item.is_ok_and(|field| std::str::from_utf8(field.1).is_err())
        })
    {
        return Err(EncodeError::Unsupported);
    }
    let value = format!(" path={name}\n");
    let mut size = value.len() + 1;
    loop {
        let next = value.len() + size.to_string().len();
        if next == size {
            break;
        }
        size = next;
    }
    body.extend_from_slice(format!("{size}{value}").as_bytes());
    if body.len() > 256 * 1024 {
        return Err(EncodeError::Unsupported);
    }
    let extension = record::header(b"PaxHeaders/xdzip", b'x', body.len() as u64, 0, 0o644)
        .map_err(|_| EncodeError::Unsupported)?;
    output
        .write_all(&extension)
        .and_then(|_| output.write_all(&body))
        .map_err(|_| EncodeError::Output)?;
    let padding = (512 - body.len() % 512) % 512;
    output
        .write_all(&[0u8; 512][..padding])
        .map_err(|_| EncodeError::Output)?;
    let mut actual = *header;
    record::set_name(
        &mut actual,
        format!(
            "entry-{index}{}",
            if name.ends_with('/') { "/" } else { "" }
        )
        .as_bytes(),
    )
    .map_err(|_| EncodeError::Unsupported)?;
    record::seal(&mut actual);
    output.write_all(&actual).map_err(|_| EncodeError::Output)
}

fn copy_records(
    reader: &mut dyn Read,
    output: &mut dyn Write,
    keep: &[bool],
    renames: &[Option<String>],
    cancel: &Cancellation,
) -> Result<(), EncodeError> {
    if !renames.is_empty() && renames.len() != keep.len() {
        return Err(EncodeError::Invalid);
    }
    let mut index = 0usize;
    let mut pending = Vec::new();
    let mut consumed = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    let mut next_size = None;
    loop {
        cancel.check().map_err(|_| EncodeError::Cancelled)?;
        let mut header = [0u8; 512];
        reader
            .read_exact(&mut header)
            .map_err(|_| EncodeError::Source)?;
        consumed += 512;
        if header.iter().all(|b| *b == 0) {
            reader
                .read_exact(&mut header)
                .map_err(|_| EncodeError::Source)?;
            consumed += 512;
            if header.iter().any(|b| *b != 0) {
                return Err(EncodeError::Invalid);
            }
            break;
        }
        let h = record::Record(&header);
        h.validate().map_err(|_| EncodeError::Invalid)?;
        let kind = h.kind();
        let size = if matches!(kind, b'x' | b'L' | b'K') {
            h.size().map_err(|_| EncodeError::Invalid)?
        } else {
            next_size
                .take()
                .unwrap_or(h.size().map_err(|_| EncodeError::Invalid)?)
        };
        let padded = size.checked_add(511).ok_or(EncodeError::Invalid)? / 512 * 512;
        consumed = consumed.checked_add(padded).ok_or(EncodeError::Invalid)?;
        if consumed > STREAM_LIMIT {
            return Err(EncodeError::Invalid);
        }
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
            if kind == b'x' {
                for field in record::Pax(&pending[start..start + size as usize]) {
                    let (key, value) = field.map_err(|_| EncodeError::Invalid)?;
                    if key == b"size" {
                        if next_size.is_some() {
                            return Err(EncodeError::Invalid);
                        }
                        next_size = Some(
                            std::str::from_utf8(value)
                                .map_err(|_| EncodeError::Invalid)?
                                .parse::<u64>()
                                .map_err(|_| EncodeError::Invalid)?,
                        );
                    }
                }
            }
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
            if let Some(Some(name)) = renames.get(index - 1) {
                renamed_header(output, &pending, &header, name, index - 1)?;
            } else {
                output
                    .write_all(&pending)
                    .and_then(|_| output.write_all(&header))
                    .map_err(|_| EncodeError::Output)?;
            }
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
        let mut reader = self
            .stream(original, cancel, STREAM_LIMIT)
            .map_err(|_| EncodeError::Source)?;
        if self.gzip {
            let mut encoder =
                GzipWriter::new(output, input.settings.level).map_err(|_| EncodeError::Invalid)?;
            copy_records(&mut reader, &mut encoder, input.keep, input.renames, cancel).map_err(
                |error| {
                    if cancel.check().is_err() {
                        EncodeError::Cancelled
                    } else {
                        error
                    }
                },
            )?;
            crate::writer::write_tar(&mut encoder, input.additions, cancel, progress)?;
            encoder.finish().map_err(|_| EncodeError::Output)?;
        } else {
            copy_records(&mut reader, output, input.keep, input.renames, cancel).map_err(
                |error| {
                    if cancel.check().is_err() {
                        EncodeError::Cancelled
                    } else {
                        error
                    }
                },
            )?;
            crate::writer::write_tar(output, input.additions, cancel, progress)?;
        }
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
    fn pax_size_controls_rewrite_payload_and_two_end_blocks_are_required() {
        let mut pax = Vec::new();
        record::pax_field(&mut pax, "size", b"7");
        let mut source = Vec::new();
        source.extend_from_slice(
            &record::header(b"PaxHeaders/size", b'x', pax.len() as u64, 0, 0o644).unwrap(),
        );
        source.extend_from_slice(&pax);
        source.resize(source.len().div_ceil(512) * 512, 0);
        // The PAX size overrides the zero-sized ordinary header.
        source.extend_from_slice(&record::header(b"file", b'0', 0, 0, 0o644).unwrap());
        source.extend_from_slice(b"payload");
        source.resize(source.len().div_ceil(512) * 512, 0);
        source.extend_from_slice(&[0; 1024]);
        let mut output = Vec::new();
        copy_records(
            &mut source.as_slice(),
            &mut output,
            &[true],
            &[Some("renamed".into())],
            &Cancellation::default(),
        )
        .unwrap();
        output.extend_from_slice(&[0; 1024]);
        let mut oracle = tar::Archive::new(output.as_slice());
        let mut entries = oracle.entries().unwrap();
        let mut entry = entries.next().unwrap().unwrap();
        assert_eq!(entry.path_bytes().as_ref(), b"renamed");
        let mut data = Vec::new();
        entry.read_to_end(&mut data).unwrap();
        assert_eq!(data, b"payload");
        assert!(entries.next().is_none());
        source.truncate(source.len() - 512);
        assert!(
            copy_records(
                &mut source.as_slice(),
                &mut Vec::new(),
                &[true],
                &[],
                &Cancellation::default()
            )
            .is_err()
        );
    }
    #[test]
    fn renamed_pax_paths_preserve_unknown_attributes_mode_time_and_contents() {
        let original_name = format!("来源/{}/file.txt", "长目录".repeat(30));
        let new_name = format!("目标/{original_name}");
        let mut builder = tar::Builder::new(Vec::new());
        builder
            .append_pax_extensions([
                ("path", original_name.as_bytes()),
                ("XDZIP.owned", b"opaque metadata".as_slice()),
            ])
            .unwrap();
        let mut header = tar::Header::new_ustar();
        header.set_mode(0o751);
        header.set_uid(42);
        header.set_gid(43);
        header.set_mtime(1234567);
        header.set_size(7);
        header.set_cksum();
        builder
            .append_data(&mut header, "placeholder", b"payload".as_slice())
            .unwrap();
        let original = builder.into_inner().unwrap();
        for gzip in [false, true] {
            let bytes = if gzip {
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&original).unwrap();
                encoder.finish().unwrap()
            } else {
                original.clone()
            };
            let mut result = Cursor::new(Vec::new());
            TarCodec { gzip }
                .rewrite(
                    &mut Cursor::new(bytes),
                    RewriteInput {
                        keep: &[true],
                        renames: &[Some(new_name.clone())],
                        additions: &Empty,
                        settings: EncodeSettings {
                            solid: false,
                            encrypt_headers: false,
                            method: "deflate",
                            level: 6,
                            password: None,
                        },
                    },
                    &mut result,
                    &Cancellation::default(),
                    &mut |_| {},
                )
                .unwrap();
            let raw = if gzip {
                let mut raw = Vec::new();
                MultiGzDecoder::new(result.get_ref().as_slice())
                    .read_to_end(&mut raw)
                    .unwrap();
                raw
            } else {
                result.into_inner()
            };
            let mut archive = tar::Archive::new(raw.as_slice());
            let mut entries = archive.entries().unwrap();
            let mut entry = entries.next().unwrap().unwrap();
            assert_eq!(entry.path_bytes().as_ref(), new_name.as_bytes());
            assert_eq!(entry.header().mode().unwrap(), 0o751);
            assert_eq!(entry.header().uid().unwrap(), 42);
            assert_eq!(entry.header().gid().unwrap(), 43);
            assert_eq!(entry.header().mtime().unwrap(), 1234567);
            let attrs: Vec<_> = entry
                .pax_extensions()
                .unwrap()
                .unwrap()
                .map(|a| {
                    let a = a.unwrap();
                    (a.key_bytes().to_vec(), a.value_bytes().to_vec())
                })
                .collect();
            assert!(attrs.contains(&(b"XDZIP.owned".to_vec(), b"opaque metadata".to_vec())));
            let mut body = Vec::new();
            entry.read_to_end(&mut body).unwrap();
            assert_eq!(body, b"payload");
            assert!(entries.next().is_none());
        }
    }
}
