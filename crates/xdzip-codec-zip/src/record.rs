// crates/xdzip-codec-zip/src/record.rs

//! Independent ZIP records from PKWARE APPNOTE 6.3.10 sections 4.3.12–4.3.16
//! and 4.5.3. Metadata never grants permission to decode a payload.
use std::io::{self, BufReader, Read, Seek, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits};

const MAX_DIRECTORY: u64 = 64 * 1024 * 1024;
const MAX_END_SEARCH: usize = 22 + u16::MAX as usize;

#[derive(Clone, Debug)]
pub(crate) struct Record {
    pub name: Vec<u8>,
    pub extra: Vec<u8>,
    pub comment: Vec<u8>,
    pub flags: u16,
    pub method: u16,
    pub crc: u32,
    pub size: u64,
    pub packed_size: u64,
    /// Absolute source position, including the SFX prefix correction.
    pub local_offset: u64,
    pub central_offset: u64,
    /// Raw DOS date/time; invalid values remain available for header matching.
    pub modified_dos: u32,
    pub version_made_by: u16,
    pub version_needed: u16,
    pub internal_attributes: u16,
    pub external_attributes: u32,
    pub central_wide_sizes: bool,
}
impl Record {
    pub fn utf8(&self) -> bool {
        self.flags & 0x800 != 0
    }
    pub fn encrypted(&self) -> bool {
        self.flags & 1 != 0
    }
    pub fn is_dir(&self) -> bool {
        self.name.ends_with(b"/") || self.external_attributes & 0x10 != 0
    }
}
pub(crate) struct Directory {
    pub records: Vec<Record>,
    pub comment: Vec<u8>,
    pub central_offset: u64,
    pub central_size: u64,
    /// Bias already applied to each Record offset, never added again by callers.
    pub base_offset: u64,
    pub end_offset: u64,
    pub source_length: u64,
}

/// A malformed field yields one error and then terminates, never hiding a tail.
pub(crate) struct ExtraFields<'a> {
    bytes: &'a [u8],
}
impl<'a> ExtraFields<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}
impl<'a> Iterator for ExtraFields<'a> {
    type Item = Result<(u16, &'a [u8]), ScanError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.bytes.is_empty() {
            return None;
        }
        let bytes = std::mem::take(&mut self.bytes);
        if bytes.len() < 4 {
            return Some(Err(ScanError::Invalid));
        }
        let size = usize::from(n16(bytes, 2));
        let Some(data) = bytes.get(4..4 + size) else {
            return Some(Err(ScanError::Invalid));
        };
        self.bytes = &bytes[4 + size..];
        Some(Ok((n16(bytes, 0), data)))
    }
}
fn n16(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().expect("checked ZIP field"))
}
fn n32(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().expect("checked ZIP field"))
}
fn n64(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at + 8].try_into().expect("checked ZIP field"))
}
fn source_error(error: &io::Error) -> ScanError {
    error
        .get_ref()
        .and_then(|error| error.downcast_ref::<ScanError>())
        .copied()
        .unwrap_or(ScanError::Io)
}
fn io_error(error: io::Error) -> ScanError {
    if error.get_ref().is_some_and(|error| error.is::<ScanError>()) {
        return source_error(&error);
    }
    if error.kind() == io::ErrorKind::UnexpectedEof {
        ScanError::Invalid
    } else {
        ScanError::Io
    }
}
pub(crate) fn read_at(
    source: &mut dyn ReadSource,
    at: u64,
    bytes: &mut [u8],
) -> Result<(), ScanError> {
    read_exact_at(source, at, bytes)
}
fn read_exact_at<R: Read + Seek + ?Sized>(
    source: &mut R,
    at: u64,
    bytes: &mut [u8],
) -> Result<(), ScanError> {
    source.seek(SeekFrom::Start(at)).map_err(io_error)?;
    source.read_exact(bytes).map_err(io_error)
}
struct Budget<'a> {
    source: &'a mut dyn ReadSource,
    control: &'a Cancellation,
    remaining: u64,
    failure: Option<ScanError>,
}
impl Read for Budget<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if let Some(error) = self
            .control
            .check()
            .err()
            .or_else(|| (self.remaining == 0 && !output.is_empty()).then_some(ScanError::Limit))
        {
            self.failure = Some(error);
            return Err(io::Error::other(error));
        }
        let count = output
            .len()
            .min(usize::try_from(self.remaining).unwrap_or(usize::MAX));
        match self.source.read(&mut output[..count]) {
            Ok(count) => {
                self.remaining -= count as u64;
                Ok(count)
            }
            Err(error) => {
                self.failure = Some(source_error(&error));
                Err(error)
            }
        }
    }
}
impl Seek for Budget<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.source
            .seek(position)
            .inspect_err(|error| self.failure = Some(source_error(error)))
    }
}
pub(crate) fn read_directory(
    source: &mut dyn ReadSource,
    control: &Cancellation,
    limits: ScanLimits,
) -> Result<Directory, ScanError> {
    control.check()?;
    let mut source = Budget {
        source,
        control,
        remaining: limits.read_bytes,
        failure: None,
    };
    let result = parse_directory(&mut source, control, limits);
    match source.failure {
        Some(error) => Err(error),
        None => result,
    }
}
struct Footer {
    count: u64,
    size: u64,
    offset: u64,
    records_end: u64,
    base: Option<u64>,
}
fn zip64_header(
    source: &mut Budget<'_>,
    at: u64,
    locator_at: u64,
) -> Result<Option<[u8; 56]>, ScanError> {
    if at.checked_add(56).is_none_or(|end| end > locator_at) {
        return Ok(None);
    }
    let mut header = [0; 56];
    read_exact_at(source, at, &mut header)?;
    Ok((&header[..4] == b"PK\x06\x06"
        && n64(&header, 4) >= 44
        && at
            .checked_add(12)
            .and_then(|at| at.checked_add(n64(&header, 4)))
            == Some(locator_at))
    .then_some(header))
}
fn zip64_location(
    source: &mut Budget<'_>,
    stored: u64,
    locator_at: u64,
    control: &Cancellation,
) -> Result<(u64, [u8; 56]), ScanError> {
    if let Some(header) = zip64_header(source, stored, locator_at)? {
        return Ok((stored, header));
    }
    if let Some(at) = locator_at.checked_sub(56)
        && let Some(header) = zip64_header(source, at, locator_at)?
    {
        return Ok((at, header));
    }
    // Only unusual SFX archives with an extensible ZIP64 end need this search.
    // Search windows bound memory; the read budget charges repeated source bytes.
    let mut end = locator_at;
    loop {
        control.check()?;
        let start = end.saturating_sub(64 * 1024);
        let count = usize::try_from(end - start).map_err(|_| ScanError::Limit)?;
        let mut bytes = vec![0; count];
        read_exact_at(source, start, &mut bytes)?;
        for offset in (0..bytes.len().saturating_sub(3)).rev() {
            if &bytes[offset..offset + 4] == b"PK\x06\x06" {
                let at = start + offset as u64;
                if let Some(header) = zip64_header(source, at, locator_at)? {
                    return Ok((at, header));
                }
            }
        }
        if start == 0 {
            return Err(ScanError::Invalid);
        }
        end = start + 3;
    }
}
fn footer(
    source: &mut Budget<'_>,
    end: &[u8],
    end_at: u64,
    control: &Cancellation,
) -> Result<Footer, ScanError> {
    let mut locator = [0; 20];
    let locator_at = end_at.checked_sub(20);
    if let Some(at) = locator_at {
        read_exact_at(source, at, &mut locator)?;
    }
    if &locator[..4] != b"PK\x06\x07" {
        // Without ZIP64, disk placeholders cannot be resolved. A count of exactly
        // 65,535 remains a valid classic single-volume archive, not an implicit sentinel.
        super::admission::single_volume(
            u32::from(n16(end, 4)),
            u32::from(n16(end, 6)),
            u64::from(n16(end, 8)),
            u64::from(n16(end, 10)),
        )?;
        if n32(end, 12) == u32::MAX || n32(end, 16) == u32::MAX {
            return Err(ScanError::Invalid);
        }
        return Ok(Footer {
            count: u64::from(n16(end, 10)),
            size: u64::from(n32(end, 12)),
            offset: u64::from(n32(end, 16)),
            records_end: end_at,
            base: None,
        });
    }
    if n32(&locator, 4) != 0 || n32(&locator, 16) != 1 {
        return Err(ScanError::Unsupported);
    }
    let stored = n64(&locator, 8);
    let locator_at = locator_at.ok_or(ScanError::Invalid)?;
    let (at, zip64) = zip64_location(source, stored, locator_at, control)?;
    super::admission::single_volume(
        n32(&zip64, 16),
        n32(&zip64, 20),
        n64(&zip64, 24),
        n64(&zip64, 32),
    )?;
    let count = n64(&zip64, 32);
    let size = n64(&zip64, 40);
    let offset = n64(&zip64, 48);
    // APPNOTE 4.4.19-4.4.22 permits each classic disk/count field independently
    // to defer to ZIP64. Every non-sentinel declaration must match its true value.
    if [
        (4, u64::from(n32(&zip64, 16))),
        (6, u64::from(n32(&zip64, 20))),
        (8, n64(&zip64, 24)),
        (10, count),
    ]
    .into_iter()
    .any(|(field, actual)| n16(end, field) != u16::MAX && u64::from(n16(end, field)) != actual)
        || (n32(end, 12) != u32::MAX && u64::from(n32(end, 12)) != size)
        || (n32(end, 16) != u32::MAX && u64::from(n32(end, 16)) != offset)
    {
        return Err(ScanError::Invalid);
    }
    Ok(Footer {
        count,
        size,
        offset,
        records_end: at,
        base: Some(at.checked_sub(stored).ok_or(ScanError::Invalid)?),
    })
}

fn parse_directory(
    source: &mut Budget<'_>,
    control: &Cancellation,
    limits: ScanLimits,
) -> Result<Directory, ScanError> {
    let source_length = source.seek(SeekFrom::End(0)).map_err(io_error)?;
    if source_length < 22 {
        return Err(ScanError::Invalid);
    }
    let tail_size =
        usize::try_from(source_length.min(MAX_END_SEARCH as u64)).map_err(|_| ScanError::Limit)?;
    let mut tail = vec![0; tail_size];
    read_exact_at(source, source_length - tail_size as u64, &mut tail)?;
    let mut candidates = (0..=tail.len() - 22).rev().filter(|&at| {
        &tail[at..at + 4] == b"PK\x05\x06"
            && at + 22 + usize::from(n16(&tail, at + 20)) == tail.len()
    });
    let end = candidates.next().ok_or(ScanError::Invalid)?;
    if candidates.next().is_some() {
        return Err(ScanError::Invalid);
    }
    let end_offset = source_length - tail_size as u64 + end as u64;
    let comment = tail[end + 22..].to_vec();
    let footer = footer(source, &tail[end..end + 22], end_offset, control)?;
    if footer.count > limits.entries.min(xdzip_codec_api::MAX_ENTRIES) as u64
        || footer.size > MAX_DIRECTORY
    {
        return Err(ScanError::Limit);
    }
    if footer
        .count
        .checked_mul(46)
        .is_none_or(|size| size > footer.size)
    {
        return Err(ScanError::Invalid);
    }
    let central_offset = footer
        .records_end
        .checked_sub(footer.size)
        .ok_or(ScanError::Invalid)?;
    let base_offset = central_offset
        .checked_sub(footer.offset)
        .ok_or(ScanError::Invalid)?;
    if footer.base.is_some_and(|base| base != base_offset) {
        return Err(ScanError::Invalid);
    }
    source
        .seek(SeekFrom::Start(central_offset))
        .map_err(io_error)?;
    let mut stream = BufReader::with_capacity(64 * 1024, (&mut *source).take(footer.size));
    let mut records = Vec::with_capacity(footer.count as usize);
    let mut position = central_offset;
    let mut names = 0usize;
    for _ in 0..footer.count {
        control.check()?;
        let mut fixed = [0; 46];
        stream.read_exact(&mut fixed).map_err(io_error)?;
        if &fixed[..4] != b"PK\x01\x02" {
            return Err(ScanError::Invalid);
        }
        let name_len = usize::from(n16(&fixed, 28));
        let extra_len = usize::from(n16(&fixed, 30));
        let comment_len = usize::from(n16(&fixed, 32));
        names = names.checked_add(name_len).ok_or(ScanError::Limit)?;
        if name_len > limits.name_bytes || names > limits.total_name_bytes {
            return Err(ScanError::Limit);
        }
        let total = 46 + name_len + extra_len + comment_len;
        let next = position
            .checked_add(total as u64)
            .filter(|end| *end <= footer.records_end)
            .ok_or(ScanError::Invalid)?;
        let mut variable = vec![0; total - 46];
        stream.read_exact(&mut variable).map_err(io_error)?;
        let name = variable[..name_len].to_vec();
        let extra = variable[name_len..name_len + extra_len].to_vec();
        let comment = variable[name_len + extra_len..].to_vec();
        let (size, packed_size, local, disk) = wide_fields(&fixed, &extra)?;
        super::admission::entry_disk(disk)?;
        let local_offset = local
            .checked_add(base_offset)
            .filter(|at| *at < central_offset)
            .ok_or(ScanError::Invalid)?;
        records.push(Record {
            name,
            extra,
            comment,
            flags: n16(&fixed, 8),
            method: n16(&fixed, 10),
            crc: n32(&fixed, 16),
            size,
            packed_size,
            local_offset,
            central_offset: position,
            modified_dos: n32(&fixed, 12),
            version_made_by: n16(&fixed, 4),
            version_needed: n16(&fixed, 6),
            internal_attributes: n16(&fixed, 36),
            external_attributes: n32(&fixed, 38),
            central_wide_sizes: n32(&fixed, 20) == u32::MAX || n32(&fixed, 24) == u32::MAX,
        });
        position = next;
    }
    if position != footer.records_end {
        // APPNOTE 4.3.13 permits one bounded digital-signature record after all entries.
        let mut signature = [0; 6];
        stream.read_exact(&mut signature).map_err(io_error)?;
        if &signature[..4] != b"PK\x05\x05"
            || position.checked_add(6 + u64::from(n16(&signature, 4))) != Some(footer.records_end)
        {
            return Err(ScanError::Invalid);
        }
        let mut data = vec![0; usize::from(n16(&signature, 4))];
        stream.read_exact(&mut data).map_err(io_error)?;
    }
    drop(stream);
    if base_offset != 0
        && let Some(local) = records.iter().map(|record| record.local_offset).min()
    {
        // Bind an inferred prefix correction to an actual local record. A
        // corrupted directory offset alone must not invent an SFX prefix.
        let mut signature = [0; 4];
        read_exact_at(source, local, &mut signature)?;
        if &signature != b"PK\x03\x04" {
            return Err(ScanError::Invalid);
        }
    }
    control.check()?;
    Ok(Directory {
        records,
        comment,
        central_offset,
        central_size: footer.size,
        base_offset,
        end_offset,
        source_length,
    })
}
fn wide_fields(fixed: &[u8; 46], extra: &[u8]) -> Result<(u64, u64, u64, u32), ScanError> {
    let required = [20, 24, 42]
        .into_iter()
        .any(|at| n32(fixed, at) == u32::MAX)
        || n16(fixed, 34) == u16::MAX;
    let mut wide = None;
    for field in ExtraFields::new(extra) {
        let (id, bytes) = match field {
            Ok(value) => value,
            // A damaged Unicode declaration remains inspectable but unresolved.
            // ZIP64 locators cannot tolerate a malformed field envelope.
            Err(error) if required => return Err(error),
            Err(_) => break,
        };
        if id == 1 && wide.replace(bytes).is_some() {
            return Err(ScanError::Invalid);
        }
    }
    let wide = wide.unwrap_or(&[]);
    let mut at = 0;
    let mut value = |offset| -> Result<u64, ScanError> {
        let value = n32(fixed, offset);
        if value != u32::MAX {
            return Ok(u64::from(value));
        }
        let bytes = wide.get(at..at + 8).ok_or(ScanError::Invalid)?;
        at += 8;
        Ok(n64(bytes, 0))
    };
    let size = value(24)?;
    let packed = value(20)?;
    let offset = value(42)?;
    let disk = if n16(fixed, 34) == u16::MAX {
        n32(wide.get(at..at + 4).ok_or(ScanError::Invalid)?, 0)
    } else {
        u32::from(n16(fixed, 34))
    };
    Ok((size, packed, offset, disk))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use xdzip_codec_api::MetadataCodec;
    fn directory(bytes: &[u8]) -> Result<Directory, ScanError> {
        read_directory(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
    }
    fn classic() -> Vec<u8> {
        crate::legacy_fixture::zip(&[(b"owned.txt", 2048, &[])])
    }
    #[test]
    fn eof_comment_and_file_comment_are_preserved_without_accepting_garbage() {
        let mut bytes = classic();
        let before = directory(&bytes).unwrap();
        let central = before.central_offset as usize;
        let comment = b"opaque\xff file comment";
        let end = bytes.len() - 22;
        bytes[central + 32..central + 34].copy_from_slice(&(comment.len() as u16).to_le_bytes());
        let size = n32(&bytes, end + 12) + comment.len() as u32;
        bytes[end + 12..end + 16].copy_from_slice(&size.to_le_bytes());
        bytes.splice(end..end, comment.iter().copied());
        let end = bytes.len() - 22;
        let archive_comment = b"owned PK\x05\x06 comment";
        bytes[end + 20..end + 22].copy_from_slice(&(archive_comment.len() as u16).to_le_bytes());
        bytes.extend(archive_comment);
        let parsed = directory(&bytes).unwrap();
        assert_eq!(parsed.comment, archive_comment);
        assert_eq!(parsed.records[0].comment, comment);
        assert_eq!(
            parsed.end_offset + 22 + parsed.comment.len() as u64,
            parsed.source_length
        );
        bytes.push(0);
        assert!(matches!(directory(&bytes), Err(ScanError::Invalid)));
    }
    #[test]
    fn sfx_bias_applies_once_to_classic_and_zip64_local_offsets() {
        let prefix = b"MZ independently owned executable prefix";
        for bytes in [classic(), crate::zip64_fixture::archive(1)] {
            let before = directory(&bytes).unwrap();
            let prefixed = [prefix.as_slice(), bytes.as_slice()].concat();
            let parsed = directory(&prefixed).unwrap();
            assert_eq!(parsed.base_offset, prefix.len() as u64);
            assert_eq!(
                parsed.central_offset,
                before.central_offset + prefix.len() as u64
            );
            assert_eq!(
                parsed.records[0].local_offset,
                before.records[0].local_offset + prefix.len() as u64
            );
        }
        let mut bytes = classic();
        let end = bytes.len() - 22;
        let offset = n32(&bytes, end + 16) - 1;
        bytes[end + 16..end + 20].copy_from_slice(&offset.to_le_bytes());
        assert!(matches!(directory(&bytes), Err(ScanError::Invalid)));
    }
    #[test]
    fn zip64_sfx_extensible_end_and_conflicting_locator_offsets() {
        let mut bytes = crate::zip64_fixture::archive(1);
        let zip64 = bytes.len() - 98;
        let extension = [0x34, 0x12, 3, 0, 0, 0, b'a', b'b', b'c'];
        bytes[zip64 + 4..zip64 + 12].copy_from_slice(&(44 + extension.len() as u64).to_le_bytes());
        bytes.splice(zip64 + 56..zip64 + 56, extension);
        let prefix = b"MZ owned prefix";
        let prefixed = [prefix.as_slice(), bytes.as_slice()].concat();
        assert_eq!(
            directory(&prefixed).unwrap().base_offset,
            prefix.len() as u64
        );
        let mut corrupt = bytes.clone();
        let locator = corrupt.len() - 42;
        corrupt[locator + 8..locator + 16].copy_from_slice(&(zip64 as u64 + 1).to_le_bytes());
        assert!(matches!(directory(&corrupt), Err(ScanError::Invalid)));
    }
    #[test]
    fn zip64_classic_disk_and_count_sentinels_are_independent() {
        for count in [0, 1] {
            let original = crate::zip64_fixture::archive(count);
            for mask in 0..16 {
                let mut bytes = original.clone();
                let end = bytes.len() - 22;
                for (bit, field, actual) in [(1, 4, 0), (2, 6, 0), (4, 8, count), (8, 10, count)] {
                    let value = if mask & bit == 0 {
                        actual as u16
                    } else {
                        u16::MAX
                    };
                    bytes[end + field..end + field + 2].copy_from_slice(&value.to_le_bytes());
                }
                for prefix in [b"".as_slice(), b"MZ owned SFX prefix".as_slice()] {
                    let source = [prefix, bytes.as_slice()].concat();
                    let parsed = directory(&source)
                        .unwrap_or_else(|error| panic!("count={count} mask={mask}: {error:?}"));
                    assert_eq!(parsed.records.len(), count);
                    assert_eq!(parsed.base_offset, prefix.len() as u64);
                }
            }
        }
    }
    #[test]
    fn zip64_non_sentinel_classic_disk_and_count_conflicts_are_invalid() {
        for field in [4, 6, 8, 10] {
            let mut bytes = crate::zip64_fixture::archive(1);
            let end = bytes.len() - 22;
            let value: u16 = if field < 8 { 1 } else { 0 };
            bytes[end + field..end + field + 2].copy_from_slice(&value.to_le_bytes());
            assert!(
                matches!(directory(&bytes), Err(ScanError::Invalid)),
                "field={field}"
            );
        }
    }
    #[test]
    fn zip64_real_multivolume_values_remain_unsupported() {
        for field in [16, 20, 24, 32] {
            let mut bytes = crate::zip64_fixture::archive(1);
            let end = bytes.len() - 22;
            for classic in [4, 6, 8, 10] {
                bytes[end + classic..end + classic + 2].copy_from_slice(&u16::MAX.to_le_bytes());
            }
            let wide = bytes.len() - 98;
            if field < 24 {
                bytes[wide + field..wide + field + 4].copy_from_slice(&1u32.to_le_bytes());
            } else {
                bytes[wide + field..wide + field + 8].copy_from_slice(&2u64.to_le_bytes());
            }
            assert!(
                matches!(directory(&bytes), Err(ScanError::Unsupported)),
                "field={field}"
            );
        }
    }
    #[test]
    fn classic_disk_placeholders_without_zip64_are_unsupported() {
        for field in [4, 6] {
            let mut bytes = classic();
            let end = bytes.len() - 22;
            bytes[end + field..end + field + 2].copy_from_slice(&u16::MAX.to_le_bytes());
            assert!(
                matches!(directory(&bytes), Err(ScanError::Unsupported)),
                "field={field}"
            );
        }
    }
    #[test]
    fn zip64_optional_fields_follow_sentinels_in_specification_order() {
        for mask in 0..16 {
            let mut fixed = [0; 46];
            let mut payload = Vec::new();
            for (bit, at, value) in [(1, 24, 11u32), (2, 20, 22), (4, 42, 33)] {
                let raw = if mask & bit != 0 {
                    payload.extend(u64::from(value).to_le_bytes());
                    u32::MAX
                } else {
                    value
                };
                fixed[at..at + 4].copy_from_slice(&raw.to_le_bytes());
            }
            if mask & 8 != 0 {
                fixed[34..36].copy_from_slice(&u16::MAX.to_le_bytes());
                payload.extend(0u32.to_le_bytes());
            }
            let mut extra = vec![1, 0];
            extra.extend((payload.len() as u16).to_le_bytes());
            extra.extend(payload);
            assert_eq!(
                wide_fields(&fixed, &extra),
                Ok((11, 22, 33, 0)),
                "mask={mask}"
            );
            if mask != 0 {
                for end in 0..extra.len() {
                    assert!(
                        wide_fields(&fixed, &extra[..end]).is_err(),
                        "mask={mask} end={end}"
                    );
                }
            }
            extra.extend(extra.clone());
            assert!(wide_fields(&fixed, &extra).is_err());
        }
    }
    #[test]
    fn malformed_unicode_field_remains_inspectable_but_never_resolves_a_name() {
        let bytes = crate::legacy_fixture::zip(&[(b"owned.txt", 0, &[0x75, 0x70, 9, 0, 1])]);
        let parsed = directory(&bytes).unwrap();
        assert_eq!(parsed.records[0].extra, [0x75, 0x70, 9, 0, 1]);
        let rows = crate::ZipCodec
            .scan(
                &mut Cursor::new(bytes),
                &Default::default(),
                Default::default(),
            )
            .unwrap();
        assert_eq!(
            rows[0].name_encoding,
            xdzip_codec_api::names::NameEncoding::Unresolved
        );
        assert!(rows[0].warning);
    }
    #[test]
    fn central_tail_is_exact_and_extra_field_errors_cannot_be_skipped() {
        let mut fields = ExtraFields::new(&[1, 0, 2, 0, 0]);
        assert_eq!(fields.next(), Some(Err(ScanError::Invalid)));
        assert_eq!(fields.next(), None);
        let mut bytes = classic();
        let end = bytes.len() - 22;
        bytes.splice(end..end, *b"x");
        assert!(matches!(directory(&bytes), Err(ScanError::Invalid)));
        let mut bytes = classic();
        let end = bytes.len() - 22;
        let signature = [b'P', b'K', 5, 5, 3, 0, b'a', b'b', b'c'];
        let size = n32(&bytes, end + 12) + signature.len() as u32;
        bytes[end + 12..end + 16].copy_from_slice(&size.to_le_bytes());
        bytes.splice(end..end, signature);
        assert!(directory(&bytes).is_ok());
    }
    #[test]
    fn competing_eof_records_cannot_hide_the_physical_directory() {
        let mut bytes = classic();
        let end = bytes.len() - 22;
        bytes[end + 20..end + 22].copy_from_slice(&22u16.to_le_bytes());
        let mut shadow = [0; 22];
        shadow[..4].copy_from_slice(b"PK\x05\x06");
        bytes.extend(shadow);
        assert!(matches!(directory(&bytes), Err(ScanError::Invalid)));
    }
    #[test]
    fn typed_lower_source_budget_and_cancellation_errors_keep_their_identity() {
        struct Fault {
            error: ScanError,
            read: bool,
        }
        impl ReadSource for Fault {}
        impl Read for Fault {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other(self.error))
            }
        }
        impl Seek for Fault {
            fn seek(&mut self, _: SeekFrom) -> io::Result<u64> {
                if self.read {
                    Ok(1024)
                } else {
                    Err(io::Error::other(self.error))
                }
            }
        }
        for error in [
            ScanError::Limit,
            ScanError::Cancelled,
            ScanError::SourceChanged,
            ScanError::Io,
        ] {
            for read in [false, true] {
                let result = read_directory(
                    &mut Fault { error, read },
                    &Default::default(),
                    Default::default(),
                );
                assert!(matches!(result, Err(actual) if actual == error));
            }
        }
    }
    #[test]
    fn finite_mutations_and_every_truncation_remain_bounded_and_do_not_panic() {
        for original in [
            classic(),
            crate::zip64_fixture::archive(2),
            include_bytes!("../../../tests/fixtures/zip/normal.zip").to_vec(),
        ] {
            for end in 0..original.len() {
                assert!(directory(&original[..end]).is_err(), "truncated at {end}");
            }
            let mut state = 0x7a19_04bdu32;
            for _ in 0..2048 {
                let mut bytes = original.clone();
                for _ in 0..4 {
                    state ^= state << 13;
                    state ^= state >> 17;
                    state ^= state << 5;
                    let at = state as usize % bytes.len();
                    bytes[at] ^= (state >> 24) as u8 | 1;
                }
                let _ = read_directory(
                    &mut Cursor::new(bytes),
                    &Default::default(),
                    ScanLimits {
                        entries: 64,
                        total_name_bytes: 4096,
                        read_bytes: 64 * 1024,
                        ..Default::default()
                    },
                );
            }
        }
    }
}
