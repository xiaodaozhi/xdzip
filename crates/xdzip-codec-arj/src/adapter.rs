use crate::{Index, Reader, inspect};
use std::io::{ErrorKind, SeekFrom};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
    encode::{
        CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
        RewriteCodec, RewriteInput, RewriteOutput,
    },
    names::{self, NameEncoding, NameSample},
    payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader},
};
use xdzip_domain::identity::EntryId;

/// Read-only ARJ admission. The host retains all output and source authority.
pub struct ArjCodec;
impl ArjCodec {
    /// A bounded main-header proposal; complete metadata validation is mandatory.
    pub fn candidate(
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
    ) -> Result<bool, ScanError> {
        cancel.check()?;
        reset(source)?;
        let result = (|| {
            let mut bytes = [0; 12];
            let mut used = 0;
            while used < bytes.len() {
                cancel.check()?;
                match source.read(&mut bytes[used..]) {
                    Err(e) if e.kind() == ErrorKind::Interrupted && e.get_ref().is_none() => {
                        continue;
                    }
                    Err(e) => return Err(crate::header::source_error(&e)),
                    Ok(0) => break,
                    Ok(n) if n <= bytes.len() - used => used += n,
                    Ok(_) => return Err(ScanError::Io),
                }
            }
            let size = u16::from_le_bytes([bytes[2], bytes[3]]) as usize;
            Ok(used == 12
                && bytes[..2] == [0x60, 0xea]
                && (30..=2600).contains(&size)
                && (30..=size).contains(&(bytes[4] as usize))
                && bytes[10] == 2)
        })();
        reset(source)?;
        result
    }
}
fn reset(source: &mut dyn ReadSource) -> Result<(), ScanError> {
    if source
        .seek(SeekFrom::Start(0))
        .map_err(|e| crate::header::source_error(&e))?
        != 0
    {
        return Err(ScanError::SourceChanged);
    }
    Ok(())
}
fn project(
    index: &Index,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Vec<Entry>, ScanError> {
    cancel.check()?;
    if index.records.len() > limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
        return Err(ScanError::Limit);
    }
    // The ANSI flag does not name a code page. Keep its cohort separate from OEM names,
    // with one shared sampling budget and no ambient locale dependency.
    let mut samples = [NameSample::default(), NameSample::default()];
    let mut sampled = 0usize;
    let mut full = false;
    for record in &index.records {
        cancel.check()?;
        let raw = &record.raw_name;
        if full || std::str::from_utf8(raw).is_ok() {
            continue;
        }
        let Some(next) = sampled
            .checked_add(raw.len())
            .and_then(|n| n.checked_add(1))
            .filter(|&n| n <= 1024 * 1024)
        else {
            full = true;
            continue;
        };
        samples[usize::from(record.flags & 2 != 0)].observe_bytes(raw);
        sampled = next;
    }
    let [oem, ansi] = samples;
    let resolutions = [oem.finish(false), ansi.finish(false)];
    let mut rows = Vec::new();
    rows.try_reserve_exact(index.records.len())
        .map_err(|_| ScanError::Limit)?;
    let mut retained = 0usize;
    for (at, record) in index.records.iter().enumerate() {
        cancel.check()?;
        let directory = record.kind == 3;
        let (decoded_name, name_encoding, display_name, name_warning) =
            match resolutions[usize::from(record.flags & 2 != 0)].decode_unmarked(&record.raw_name)
            {
                Some((mut text, encoding)) => {
                    // Decode whole multibyte names before interpreting separators.
                    if matches!(record.host, 0 | 5 | 10 | 11) && record.flags & 16 == 0 {
                        text = text.replace('\\', "/");
                    }
                    if directory && !text.ends_with('/') {
                        text.push('/');
                    }
                    if text.len() > limits.name_bytes.min(4096) {
                        return Err(ScanError::Limit);
                    }
                    let warning = names::suspicious(&text, directory);
                    let display = names::display(&text);
                    let decoded = (text.as_bytes() != record.raw_name).then_some(text);
                    (decoded, encoding, display, warning)
                }
                None => (
                    None,
                    NameEncoding::Unresolved,
                    names::escaped_bytes(&record.raw_name),
                    true,
                ),
            };
        retained = retained
            .checked_add(display_name.capacity())
            .and_then(|n| n.checked_add(decoded_name.as_ref().map_or(0, String::capacity)))
            .ok_or(ScanError::Limit)?;
        if retained > limits.display_name_bytes.min(16 * 1024 * 1024) {
            return Err(ScanError::Limit);
        }
        rows.push(Entry {
            details: xdzip_codec_api::EntryDetails {
                crc: Some((record.crc32, 32)),
                attributes: match record.host {
                    2 => Some(xdzip_codec_api::display_posix_attributes(
                        record.kind == 3,
                        u32::from(record.attributes),
                    )),
                    0 | 5 | 10 | 11 => Some(xdzip_codec_api::display_dos_attributes(
                        record.kind == 3,
                        u32::from(record.attributes),
                    )),
                    _ => None,
                },
                comment: record.comment.clone(),
                ..Default::default()
            },
            format: Format::Arj,
            id: EntryId::try_from(at as u64 + 1).map_err(|_| ScanError::Limit)?,
            raw_name: record.raw_name.clone(),
            decoded_name,
            name_encoding,
            display_name,
            directory,
            link: record.host == 2 && record.attributes & 0xf000 == 0xa000,
            unix_mode: 0,
            size: record.size,
            size_known: true,
            packed_size: record.packed_size,
            method: u16::from(record.method),
            modified_dos: record.modified_dos,
            encrypted: record.flags & 1 != 0,
            password_supported: false,
            warning: name_warning || !record.payload_supported(),
        });
    }
    Ok(rows)
}
impl MetadataCodec for ArjCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        cancel: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        project(&inspect(source, cancel, limits)?, cancel, limits)
    }
}
struct Payload<'a> {
    reader: Reader<'a>,
    rows: Vec<Entry>,
    failure: Option<DecodeError>,
}
impl PayloadCodec for ArjCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        cancel: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let reader = Reader::fresh(source, cancel)?;
        let rows = project(reader.index(), cancel, ScanLimits::default()).map_err(|e| match e {
            ScanError::Cancelled => DecodeError::Cancelled,
            ScanError::Limit => DecodeError::Limit,
            _ => DecodeError::Integrity,
        })?;
        if rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(Payload {
            reader,
            rows,
            failure: None,
        }))
    }
}
impl PayloadReader for Payload<'_> {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = (|| {
            let at = usize::try_from(entry.id.get() - 1).map_err(|_| DecodeError::Integrity)?;
            if self.rows.get(at) != Some(entry) {
                return Err(DecodeError::Integrity);
            }
            if !entry.payload_supported() {
                return Err(DecodeError::Unsupported);
            }
            self.reader.decode(at, sink, cancel)
        })();
        if let Err(e) = result {
            self.failure = Some(e);
        }
        result
    }
}
impl CreationCodec for ArjCodec {
    fn encode(
        &self,
        _: &dyn EncodeInput,
        _: &mut dyn EncodeOutput,
        _: EncodeSettings<'_>,
        _: &Cancellation,
        _: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}
impl RewriteCodec for ArjCodec {
    fn rewrite(
        &self,
        _: &mut dyn ReadSource,
        _: RewriteInput<'_>,
        _: &mut dyn RewriteOutput,
        _: &Cancellation,
        _: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}
#[cfg(test)]
mod tests;
