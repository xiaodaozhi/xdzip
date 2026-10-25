// crates/xdzip-codec-zip/src/split.rs

//! Convert disk-relative ZIP metadata into a bounded single-disk input view.
use std::io::SeekFrom;
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};

const MAX_DIRECTORY: usize = 64 * 1024 * 1024;
/// Replacement for the central directory and end records, never the payload.
#[derive(Clone)]
pub struct NormalizedTail {
    /// Absolute start in the concatenated physical volumes.
    pub start: u64,
    /// Single-disk metadata, preserving names, sizes, encryption and comments.
    pub bytes: Vec<u8>,
}
fn n16(b: &[u8], p: usize) -> u16 {
    u16::from_le_bytes(b[p..p + 2].try_into().unwrap())
}
fn n32(b: &[u8], p: usize) -> u32 {
    u32::from_le_bytes(b[p..p + 4].try_into().unwrap())
}
fn n64(b: &[u8], p: usize) -> u64 {
    u64::from_le_bytes(b[p..p + 8].try_into().unwrap())
}
fn set16(b: &mut [u8], p: usize, n: u16) {
    b[p..p + 2].copy_from_slice(&n.to_le_bytes());
}
fn set32(b: &mut [u8], p: usize, n: u32) {
    b[p..p + 4].copy_from_slice(&n.to_le_bytes());
}
fn read(s: &mut dyn ReadSource, at: u64, size: usize) -> Result<Vec<u8>, ScanError> {
    let mut bytes = vec![0; size];
    s.seek(SeekFrom::Start(at)).map_err(|_| ScanError::Io)?;
    s.read_exact(&mut bytes).map_err(|_| ScanError::Invalid)?;
    Ok(bytes)
}
/// Discover the last disk from the final record and its optional ZIP64 locator.
/// This bounded probe does not validate the directory or the ZIP64 end record;
/// callers must still normalize and scan the authorized source streams.
pub fn end_disk(s: &mut dyn ReadSource) -> Result<u32, ScanError> {
    let (at, b) = end(s)?;
    let disk = n16(&b, 4);
    if disk != u16::MAX {
        return Ok(u32::from(disk));
    }
    let locator_at = at.checked_sub(20).ok_or(ScanError::Invalid)?;
    let locator = read(s, locator_at, 20)?;
    if &locator[..4] != b"PK\x06\x07" {
        return Err(ScanError::Invalid);
    }
    let disks = n32(&locator, 16);
    if disks == 0 {
        return Err(ScanError::Invalid);
    }
    if disks > 1024 {
        return Err(ScanError::Limit);
    }
    Ok(disks - 1)
}
fn end(s: &mut dyn ReadSource) -> Result<(u64, Vec<u8>), ScanError> {
    let len = s.seek(SeekFrom::End(0)).map_err(|_| ScanError::Io)?;
    let size = len.min(65557) as usize;
    let b = read(s, len - size as u64, size)?;
    let mut candidates = (0..=size.saturating_sub(22)).rev().filter(|&i| {
        b.get(i..i + 4) == Some(b"PK\x05\x06")
            && i + 22 <= size
            && i + 22 + usize::from(n16(&b, i + 20)) == size
    });
    let at = candidates.next().ok_or(ScanError::Invalid)?;
    if candidates.next().is_some() {
        return Err(ScanError::Invalid);
    }
    Ok((len - size as u64 + at as u64, b[at..].to_vec()))
}
/// Only the host supplies validated, ordered volumes. No codec opens paths.
pub fn normalize(
    s: &mut dyn ReadSource,
    lengths: &[u64],
    control: &Cancellation,
) -> Result<NormalizedTail, ScanError> {
    control.check()?;
    if lengths.len() < 2 || lengths.len() > 1024 {
        return Err(ScanError::Invalid);
    }
    let mut starts = vec![0u64];
    for &length in lengths {
        starts.push(
            starts
                .last()
                .unwrap()
                .checked_add(length)
                .ok_or(ScanError::Limit)?,
        );
    }
    let absolute = |disk: u32, offset: u64| -> Result<u64, ScanError> {
        let d = disk as usize;
        if d >= lengths.len() || offset >= lengths[d] {
            return Err(ScanError::Invalid);
        }
        starts[d].checked_add(offset).ok_or(ScanError::Invalid)
    };
    let (end_at, footer) = end(s)?;
    let last = (lengths.len() - 1) as u32;
    if u32::from(n16(&footer, 4)) != last && n16(&footer, 4) != u16::MAX {
        return Err(ScanError::Invalid);
    }
    let mut disk = u32::from(n16(&footer, 6));
    let mut offset = u64::from(n32(&footer, 16));
    let mut size = u64::from(n32(&footer, 12));
    let mut count = u64::from(n16(&footer, 10));
    let mut records_at = end_at;
    if end_at >= 20 {
        let locator = read(s, end_at - 20, 20)?;
        if &locator[..4] == b"PK\x06\x07" {
            if n32(&locator, 16) != last + 1 {
                return Err(ScanError::Invalid);
            }
            let at = absolute(n32(&locator, 4), n64(&locator, 8))?;
            let z = read(s, at, 56)?;
            if &z[..4] != b"PK\x06\x06"
                || n64(&z, 4) < 44
                || n64(&z, 4) > MAX_DIRECTORY as u64
                || at.checked_add(12).and_then(|n| n.checked_add(n64(&z, 4))) != Some(end_at - 20)
                || n32(&z, 16) != n32(&locator, 4)
            {
                return Err(ScanError::Invalid);
            }
            disk = n32(&z, 20);
            count = n64(&z, 32);
            size = n64(&z, 40);
            offset = n64(&z, 48);
            records_at = at;
        } else if size == u64::from(u32::MAX) || offset == u64::from(u32::MAX) {
            return Err(ScanError::Invalid);
        }
    }
    if size > MAX_DIRECTORY as u64 || count > xdzip_codec_api::MAX_ENTRIES as u64 {
        return Err(ScanError::Limit);
    }
    let start = absolute(disk, offset)?;
    if start.checked_add(size) != Some(records_at) {
        return Err(ScanError::Invalid);
    }
    let directory = read(s, start, size as usize)?;
    let mut output = Vec::with_capacity(directory.len());
    let mut p = 0usize;
    for _ in 0..count {
        control.check()?;
        let fixed = directory.get(p..p + 46).ok_or(ScanError::Invalid)?;
        if &fixed[..4] != b"PK\x01\x02" {
            return Err(ScanError::Invalid);
        }
        let names = usize::from(n16(fixed, 28));
        let extras = usize::from(n16(fixed, 30));
        let comments = usize::from(n16(fixed, 32));
        let tail = directory
            .get(p + 46..p + 46 + names + extras + comments)
            .ok_or(ScanError::Invalid)?;
        let mut header = fixed.to_vec();
        let mut other = Vec::new();
        let mut zip64 = None;
        let mut e = 0;
        let fields = &tail[names..names + extras];
        while e < fields.len() {
            if e + 4 > fields.len() {
                return Err(ScanError::Invalid);
            }
            let len = usize::from(n16(fields, e + 2));
            let data = fields.get(e + 4..e + 4 + len).ok_or(ScanError::Invalid)?;
            if n16(fields, e) == 1 {
                if zip64.replace(data).is_some() {
                    return Err(ScanError::Invalid);
                }
            } else {
                other.extend_from_slice(&fields[e..e + 4 + len]);
            }
            e += 4 + len;
        }
        let z = zip64.unwrap_or(&[]);
        let mut zpos = 0;
        let mut kept = Vec::new();
        for at in [24, 20] {
            if n32(fixed, at) == u32::MAX {
                kept.extend_from_slice(z.get(zpos..zpos + 8).ok_or(ScanError::Invalid)?);
                zpos += 8;
            }
        }
        let local = if n32(fixed, 42) == u32::MAX {
            let b = z.get(zpos..zpos + 8).ok_or(ScanError::Invalid)?;
            zpos += 8;
            n64(b, 0)
        } else {
            u64::from(n32(fixed, 42))
        };
        let volume = if n16(fixed, 34) == u16::MAX {
            n32(z.get(zpos..zpos + 4).ok_or(ScanError::Invalid)?, 0)
        } else {
            u32::from(n16(fixed, 34))
        };
        let local = absolute(volume, local)?;
        if local >= start {
            return Err(ScanError::Invalid);
        }
        set16(&mut header, 34, 0);
        if local >= u64::from(u32::MAX) {
            set32(&mut header, 42, u32::MAX);
            kept.extend_from_slice(&local.to_le_bytes());
        } else {
            set32(&mut header, 42, local as u32);
        }
        if !kept.is_empty() {
            set16(&mut header, 6, n16(fixed, 6).max(45));
            other.extend_from_slice(&1u16.to_le_bytes());
            other.extend_from_slice(&(kept.len() as u16).to_le_bytes());
            other.extend_from_slice(&kept);
        }
        set16(
            &mut header,
            30,
            u16::try_from(other.len()).map_err(|_| ScanError::Limit)?,
        );
        output.extend_from_slice(&header);
        output.extend_from_slice(&tail[..names]);
        output.extend_from_slice(&other);
        output.extend_from_slice(&tail[names + extras..]);
        if output.len() > MAX_DIRECTORY {
            return Err(ScanError::Limit);
        }
        p += 46 + names + extras + comments;
    }
    if p != directory.len() {
        return Err(ScanError::Invalid);
    }
    let directory_size = output.len() as u64;
    let zip64_at = start.checked_add(directory_size).ok_or(ScanError::Limit)?;
    output.extend_from_slice(b"PK\x06\x06");
    output.extend_from_slice(&44u64.to_le_bytes());
    output.extend_from_slice(&45u16.to_le_bytes());
    output.extend_from_slice(&45u16.to_le_bytes());
    output.extend_from_slice(&[0; 8]);
    output.extend_from_slice(&count.to_le_bytes());
    output.extend_from_slice(&count.to_le_bytes());
    output.extend_from_slice(&directory_size.to_le_bytes());
    output.extend_from_slice(&start.to_le_bytes());
    output.extend_from_slice(b"PK\x06\x07");
    output.extend_from_slice(&0u32.to_le_bytes());
    output.extend_from_slice(&zip64_at.to_le_bytes());
    output.extend_from_slice(&1u32.to_le_bytes());
    let mut end = footer;
    set16(&mut end, 4, 0);
    set16(&mut end, 6, 0);
    set16(&mut end, 8, u16::MAX);
    set16(&mut end, 10, u16::MAX);
    set32(&mut end, 12, u32::MAX);
    set32(&mut end, 16, u32::MAX);
    output.extend_from_slice(&end);
    start
        .checked_add(output.len() as u64)
        .ok_or(ScanError::Limit)?;
    Ok(NormalizedTail {
        start,
        bytes: output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor, Read, Seek};
    fn commented_split(comment: &[u8]) -> (Vec<u8>, [u64; 2]) {
        let mut bytes = crate::legacy_fixture::zip(&[(b"owned.txt", 2048, &[])]);
        let end = bytes.len() - 22;
        let central = n32(&bytes, end + 16);
        set16(&mut bytes, end + 4, 1);
        set16(&mut bytes, end + 6, 1);
        set32(&mut bytes, end + 16, central - 10);
        set16(&mut bytes, end + 20, comment.len() as u16);
        bytes.extend(comment);
        let lengths = [10, bytes.len() as u64 - 10];
        (bytes, lengths)
    }
    fn discovery_zip64(disks: u32) -> Vec<u8> {
        let mut bytes = crate::zip64_fixture::archive(1);
        let end = bytes.len() - 22;
        let locator = end - 20;
        set16(&mut bytes, end + 4, u16::MAX);
        set32(&mut bytes, locator + 16, disks);
        bytes
    }
    #[test]
    fn zip64_disk_placeholder_uses_bounded_locator_count_for_discovery() {
        for disks in [1, 2, 9, 1024] {
            assert_eq!(
                end_disk(&mut Cursor::new(discovery_zip64(disks))),
                Ok(disks - 1)
            );
        }
        // Discovery reads the adjacent locator only, without following its offset
        // into a ZIP64 record or trying to infer an SFX correction.
        let bytes = discovery_zip64(1);
        assert_eq!(
            end_disk(&mut Cursor::new(&bytes[bytes.len() - 42..])),
            Ok(0)
        );
    }
    #[test]
    fn zip64_disk_placeholder_requires_a_present_valid_admitted_locator() {
        assert_eq!(
            end_disk(&mut Cursor::new(discovery_zip64(0))),
            Err(ScanError::Invalid)
        );
        for disks in [1025, u32::MAX] {
            assert_eq!(
                end_disk(&mut Cursor::new(discovery_zip64(disks))),
                Err(ScanError::Limit)
            );
        }
        let mut bad = discovery_zip64(1);
        let locator = bad.len() - 42;
        bad[locator] ^= 1;
        assert_eq!(end_disk(&mut Cursor::new(bad)), Err(ScanError::Invalid));
        let mut missing = crate::zip64_fixture::classic(1);
        let end = missing.len() - 22;
        set16(&mut missing, end + 4, u16::MAX);
        assert_eq!(end_disk(&mut Cursor::new(missing)), Err(ScanError::Invalid));
        let mut short = vec![0; 22];
        short[..4].copy_from_slice(b"PK\x05\x06");
        set16(&mut short, 4, u16::MAX);
        assert_eq!(end_disk(&mut Cursor::new(short)), Err(ScanError::Invalid));
    }
    #[test]
    fn classic_disk_discovery_keeps_non_sentinel_values_without_parsing_zip64() {
        for disk in [0, 1, 1023, u16::MAX - 1] {
            let mut bytes = discovery_zip64(0);
            let end = bytes.len() - 22;
            set16(&mut bytes, end + 4, disk);
            assert_eq!(end_disk(&mut Cursor::new(bytes)), Ok(u32::from(disk)));
        }
    }
    #[test]
    fn ordinary_comments_survive_normalization_and_the_independent_directory_parser() {
        let comment = b"owned comment with a PK\x05\x06 marker";
        let (bytes, lengths) = commented_split(comment);
        assert_eq!(end_disk(&mut Cursor::new(&bytes)), Ok(1));
        let normalized =
            normalize(&mut Cursor::new(&bytes), &lengths, &Default::default()).unwrap();
        assert!(normalized.bytes.ends_with(comment));
        let mut view = bytes[..normalized.start as usize].to_vec();
        view.extend(normalized.bytes);
        let directory = crate::record::read_directory(
            &mut Cursor::new(view),
            &Default::default(),
            Default::default(),
        )
        .unwrap();
        assert_eq!(directory.base_offset, 0);
        assert_eq!(directory.comment, comment);
        assert_eq!(directory.records.len(), 1);
        assert_eq!(directory.records[0].name, b"owned.txt");
        assert_eq!(directory.records[0].local_offset, 0);
    }
    #[test]
    fn ambiguous_comment_cannot_be_normalized_into_an_empty_archive() {
        let (mut bytes, lengths) = commented_split(&[0; 22]);
        let shadow_at = bytes.len() - 22;
        let mut shadow = [0; 22];
        shadow[..4].copy_from_slice(b"PK\x05\x06");
        set16(&mut shadow, 4, 1);
        set16(&mut shadow, 6, 1);
        set32(&mut shadow, 16, (shadow_at - lengths[0] as usize) as u32);
        bytes[shadow_at..].copy_from_slice(&shadow);
        assert_eq!(end_disk(&mut Cursor::new(&bytes)), Err(ScanError::Invalid));
        let mut sentinel = bytes.clone();
        let original_end = sentinel.len() - 44;
        set16(&mut sentinel, original_end + 4, u16::MAX);
        set16(&mut sentinel, shadow_at + 4, u16::MAX);
        assert_eq!(
            end_disk(&mut Cursor::new(sentinel)),
            Err(ScanError::Invalid)
        );
        assert!(matches!(
            normalize(&mut Cursor::new(bytes), &lengths, &Default::default()),
            Err(ScanError::Invalid)
        ));
    }
    struct Sparse {
        position: u64,
        base: u64,
        tail: Vec<u8>,
    }
    impl Read for Sparse {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            if self.position < self.base {
                let n = b.len().min((self.base - self.position) as usize);
                b[..n].fill(0);
                self.position += n as u64;
                return Ok(n);
            }
            let offset = (self.position - self.base) as usize;
            let n = b.len().min(self.tail.len().saturating_sub(offset));
            b[..n].copy_from_slice(&self.tail[offset..offset + n]);
            self.position += n as u64;
            Ok(n)
        }
    }
    impl ReadSource for Sparse {}
    impl Seek for Sparse {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.position = match p {
                SeekFrom::Start(n) => n,
                SeekFrom::End(n) => (self.base + self.tail.len() as u64)
                    .checked_add_signed(n)
                    .unwrap(),
                SeekFrom::Current(n) => self.position.checked_add_signed(n).unwrap(),
            };
            Ok(self.position)
        }
    }
    #[test]
    fn disk_offsets_above_four_gib_are_promoted_without_changing_size_sentinels() {
        let base = u64::from(u32::MAX) + 4096;
        let mut central = vec![0; 47];
        central[..4].copy_from_slice(b"PK\x01\x02");
        set16(&mut central, 28, 1);
        set16(&mut central, 34, 1);
        set32(&mut central, 42, 10);
        central[46] = b'a';
        let mut footer = vec![0; 22];
        footer[..4].copy_from_slice(b"PK\x05\x06");
        set16(&mut footer, 4, 1);
        set16(&mut footer, 6, 1);
        set16(&mut footer, 8, 1);
        set16(&mut footer, 10, 1);
        set32(&mut footer, 12, 47);
        set32(&mut footer, 16, 20);
        let mut tail = central;
        tail.extend_from_slice(&footer);
        let mut source = Sparse {
            position: 0,
            base: base + 20,
            tail,
        };
        let normalized = normalize(&mut source, &[base, 89], &Cancellation::default()).unwrap();
        assert_eq!(normalized.start, base + 20);
        assert_eq!(n16(&normalized.bytes, 34), 0);
        assert_eq!(n32(&normalized.bytes, 42), u32::MAX);
        assert_eq!(n32(&normalized.bytes, 20), 0);
        assert_eq!(n32(&normalized.bytes, 24), 0);
        assert_eq!(n16(&normalized.bytes, 47), 1);
        assert_eq!(n64(&normalized.bytes, 51), base + 10);
        let mut invalid = source.tail.clone();
        set16(&mut invalid, 34, 2);
        source.tail = invalid;
        assert!(matches!(
            normalize(&mut source, &[base, 89], &Cancellation::default()),
            Err(ScanError::Invalid)
        ));
    }
    #[test]
    fn truncated_and_inconsistent_end_records_fail_without_panicking() {
        for len in 0..80 {
            let mut source = Cursor::new(vec![0; len]);
            assert!(normalize(&mut source, &[20, 60], &Cancellation::default()).is_err());
        }
        let cancelled = Cancellation::default();
        cancelled.cancel();
        assert!(matches!(
            normalize(&mut Cursor::new(Vec::new()), &[1, 1], &cancelled),
            Err(ScanError::Cancelled)
        ));
    }
}
