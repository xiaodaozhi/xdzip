//! Bounded ordinary-container rewrite, preserving physical identity and raw properties.
use crate::encode::{self, FILE_LIMIT, FileProperties, MEMORY_LIMIT, METADATA_MEMORY, TASK_LIMIT};
use crate::{DecodeLimits, SevenZipCodec, payload::Payload, read_index};
use std::io::{self, Read};
use xdzip_codec_api::encode::{
    EncodeEntry, EncodeError, EncodeInput, EncodeProgress, RewriteCodec, RewriteInput,
    RewriteOutput,
};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadReader};
use xdzip_codec_api::{Cancellation, ReadSource, ScanLimits};
const WORKING_SET: u64 = 512 << 20;
struct BufferedInput {
    entries: Vec<EncodeEntry>,
    bytes: Vec<Vec<u8>>,
}
impl EncodeInput for BufferedInput {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, index: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(io::Cursor::new(
            self.bytes
                .get(index)
                .ok_or(EncodeError::Invalid)?
                .as_slice(),
        )))
    }
}
struct Work {
    remaining: u64,
    credit: u64,
}
impl Work {
    fn charge(&mut self, count: u64) -> Result<(), DecodeError> {
        self.remaining = self
            .remaining
            .checked_sub(count)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
}
struct MemorySink<'a> {
    bytes: &'a mut Vec<u8>,
    size: u64,
    work: &'a mut Work,
    control: &'a Cancellation,
}
impl MemorySink<'_> {
    fn append(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.control.check().map_err(crate::decode::decode_error)?;
        let size = (self.bytes.len() as u64)
            .checked_add(bytes.len() as u64)
            .ok_or(DecodeError::Limit)?;
        if size > self.size {
            return Err(DecodeError::Integrity);
        }
        self.bytes
            .try_reserve(bytes.len())
            .map_err(|_| DecodeError::Limit)?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}
impl DecodedSink for MemorySink<'_> {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.work.charge(bytes.len() as u64)?;
        self.append(bytes)
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        let credit = self
            .work
            .credit
            .checked_add(bytes)
            .ok_or(DecodeError::Limit)?;
        self.work.charge(bytes)?;
        self.work.credit = credit;
        Ok(())
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.work.credit = self
            .work
            .credit
            .checked_sub(bytes.len() as u64)
            .ok_or(DecodeError::Integrity)?;
        self.append(bytes)
    }
}
fn allocation(size: u64) -> Result<Vec<u8>, EncodeError> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(usize::try_from(size).map_err(|_| EncodeError::Limit)?)
        .map_err(|_| EncodeError::Limit)?;
    Ok(bytes)
}
impl RewriteCodec for SevenZipCodec {
    fn rewrite(
        &self,
        original: &mut dyn ReadSource,
        input: RewriteInput<'_>,
        output: &mut dyn RewriteOutput,
        cancel: &Cancellation,
        progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        cancel.check().map_err(encode::scan_error)?;
        if original
            .volume_lengths()
            .is_some_and(|volumes| volumes.len() > 1)
        {
            return Err(EncodeError::Unsupported);
        }
        let index = read_index(original, cancel, ScanLimits::default(), None)
            .map_err(encode::scan_error)?;
        if index.rows.len() != input.keep.len()
            || !input.renames.is_empty() && index.rows.len() != input.renames.len()
            || index.has_archive_properties
            || index
                .rows
                .iter()
                .any(|row| row.encrypted || row.link || row.warning)
            || index.files.iter().any(|file| {
                file.anti
                    || file
                        .attributes
                        .is_some_and(|attributes| attributes & 0x400 != 0)
            })
            || index
                .streams
                .folders
                .iter()
                .any(|folder| folder.encrypted())
        {
            return Err(EncodeError::Unsupported);
        }
        let kept = input.keep.iter().filter(|keep| **keep).count();
        let count = kept
            .checked_add(input.additions.entries().len())
            .ok_or(EncodeError::Limit)?;
        if count > xdzip_codec_api::MAX_ENTRIES {
            return Err(EncodeError::Limit);
        }
        let mut buffered = BufferedInput {
            entries: Vec::new(),
            bytes: Vec::new(),
        };
        buffered
            .entries
            .try_reserve_exact(count)
            .map_err(|_| EncodeError::Limit)?;
        buffered
            .bytes
            .try_reserve_exact(count)
            .map_err(|_| EncodeError::Limit)?;
        let mut properties = Vec::new();
        properties
            .try_reserve_exact(count)
            .map_err(|_| EncodeError::Limit)?;
        let mut total = 0u64;
        for (physical, row) in index.rows.iter().enumerate() {
            if !input.keep[physical] {
                continue;
            }
            let rename = input.renames.get(physical).and_then(Option::as_deref);
            let name = rename.map_or_else(
                || std::str::from_utf8(row.name_bytes()).map_err(|_| EncodeError::Invalid),
                Ok,
            )?;
            let name = if row.directory {
                name.strip_suffix('/').unwrap_or(name)
            } else {
                name
            };
            buffered.entries.push(EncodeEntry {
                name: name.to_owned(),
                directory: row.directory,
                size: row.size,
                modified_unix: 0,
            });
            let file = &index.files[physical];
            properties.push(FileProperties {
                attributes: file.attributes,
                creation_time: file.creation_time,
                access_time: file.access_time,
                modified_time: file.modified_time,
            });
            total = total.checked_add(row.size).ok_or(EncodeError::Limit)?;
            if row.size > FILE_LIMIT || total > WORKING_SET {
                return Err(EncodeError::Limit);
            }
        }
        for entry in input.additions.entries() {
            total = total.checked_add(entry.size).ok_or(EncodeError::Limit)?;
            if entry.size > FILE_LIMIT || total > WORKING_SET {
                return Err(EncodeError::Limit);
            }
            buffered.entries.push(entry.clone());
            properties.push(FileProperties::new(entry));
        }
        encode::admit(&buffered.entries, input.settings, total)?;
        // Both indexes, retained names/properties and traversal overhead have a
        // separate reserve. The decoder cannot allocate its default 1GiB on top.
        let limits = DecodeLimits {
            memory_bytes: MEMORY_LIMIT
                .checked_sub(total)
                .and_then(|v| v.checked_sub(METADATA_MEMORY))
                .ok_or(EncodeError::Limit)?,
            ..DecodeLimits::default()
        };
        let mut reader = Payload::open_with_limits(original, &index.rows, cancel, None, limits)
            .map_err(encode::decode_error)?;
        let mut work = Work {
            remaining: TASK_LIMIT,
            credit: 0,
        };
        for (physical, row) in index.rows.iter().enumerate() {
            if !input.keep[physical] {
                continue;
            }
            cancel.check().map_err(encode::scan_error)?;
            let mut bytes = allocation(row.size)?;
            if !row.directory {
                reader
                    .decode(
                        row,
                        &mut MemorySink {
                            bytes: &mut bytes,
                            size: row.size,
                            work: &mut work,
                            control: cancel,
                        },
                        cancel,
                    )
                    .map_err(encode::decode_error)?;
            }
            if bytes.len() as u64 != row.size {
                return Err(EncodeError::Invalid);
            }
            buffered.bytes.push(bytes);
        }
        drop(reader);
        drop(index);
        for (physical, entry) in input.additions.entries().iter().enumerate() {
            cancel.check().map_err(encode::scan_error)?;
            let mut bytes = allocation(entry.size)?;
            if !entry.directory {
                let mut stream = input.additions.open(physical)?;
                let mut buffer = [0; 8192];
                let mut remaining = entry.size;
                while remaining != 0 {
                    let count = encode::source_read(
                        &mut stream,
                        &mut buffer[..remaining.min(8192) as usize],
                        cancel,
                    )?;
                    if count == 0 {
                        return Err(EncodeError::Source);
                    }
                    work.charge(count as u64).map_err(encode::decode_error)?;
                    bytes.extend_from_slice(&buffer[..count]);
                    remaining -= count as u64;
                }
                if encode::source_read(&mut stream, &mut buffer[..1], cancel)? != 0 {
                    return Err(EncodeError::Source);
                }
            }
            buffered.bytes.push(bytes);
        }
        cancel.check().map_err(encode::scan_error)?;
        encode::encode_internal(
            &buffered,
            &mut { output },
            input.settings,
            cancel,
            progress,
            Some(&properties),
            total,
        )
    }
}
#[cfg(test)]
#[path = "rewrite_tests.rs"]
mod tests;
