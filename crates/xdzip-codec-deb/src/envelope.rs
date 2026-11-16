use std::io::SeekFrom;
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits};
use xdzip_codec_stream::controlled::Filter;
/// One common-ar physical member. These fields grant no pathname authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Member {
    /// Common-ar basename, without padding or the optional terminator slash.
    pub name: String,
    /// Absolute physical payload offset within the authorized source.
    pub offset: u64,
    /// Exact stored payload length, excluding odd-byte padding.
    pub size: u64,
    /// Parsed unsigned UTC seconds; no ownership or mode is restored.
    pub modified: u64,
}
/// Validated DEB envelope, before any nested TAR metadata is published.
pub struct Envelope {
    /// Raw ar members in physical order, including transparent extensions.
    pub members: Vec<Member>,
    /// Control member position.
    pub control: usize,
    /// Data member position.
    pub data: usize,
    /// Frozen physical source span.
    pub length: u64,
    /// Actual metadata input already consumed by envelope parsing.
    pub read_work: u64,
}
struct Input<'a> {
    source: &'a mut dyn ReadSource,
    control: &'a Cancellation,
    work: u64,
    limit: u64,
}
impl Input<'_> {
    fn exact(&mut self, mut bytes: &mut [u8]) -> Result<(), ScanError> {
        while !bytes.is_empty() {
            self.control.check()?;
            let available = self.limit.checked_sub(self.work).ok_or(ScanError::Limit)?;
            if available == 0 {
                return Err(ScanError::Limit);
            }
            let take = bytes.len().min(available.min(usize::MAX as u64) as usize);
            let n = self
                .source
                .read(&mut bytes[..take])
                .map_err(|error| ScanError::from_source_error(&error))?;
            if n > take {
                return Err(ScanError::Io);
            }
            if n == 0 {
                return Err(ScanError::Invalid);
            }
            self.work += n as u64;
            bytes = &mut bytes[n..];
        }
        Ok(())
    }
    fn at(&mut self, offset: u64) -> Result<(), ScanError> {
        self.control.check()?;
        let actual = self
            .source
            .seek(SeekFrom::Start(offset))
            .map_err(|error| ScanError::from_source_error(&error))?;
        if actual != offset {
            return Err(ScanError::SourceChanged);
        }
        Ok(())
    }
}
fn number(bytes: &[u8], radix: u8) -> Result<u64, ScanError> {
    let end = bytes
        .iter()
        .rposition(|b| *b != b' ')
        .map_or(0, |at| at + 1);
    if end == 0 || bytes[..end].iter().any(|b| *b < b'0' || *b >= b'0' + radix) {
        return Err(ScanError::Invalid);
    }
    bytes[..end].iter().try_fold(0u64, |value, b| {
        value
            .checked_mul(u64::from(radix))
            .and_then(|n| n.checked_add(u64::from(*b - b'0')))
            .ok_or(ScanError::Invalid)
    })
}
fn name(bytes: &[u8]) -> Result<String, ScanError> {
    let end = bytes
        .iter()
        .rposition(|b| *b != b' ')
        .map_or(0, |at| at + 1);
    let raw = &bytes[..end];
    if raw == b"/" || raw == b"//" || raw.starts_with(b"#1/") {
        return Err(ScanError::Unsupported);
    }
    let raw = raw.strip_suffix(b"/").unwrap_or(raw);
    if raw.is_empty() || raw.iter().any(|b| !(33..=126).contains(b) || *b == b'/') {
        return Err(ScanError::Unsupported);
    }
    String::from_utf8(raw.to_vec()).map_err(|_| ScanError::Invalid)
}
/// Resolve the format-defined control/data compression whitelist, not content validity.
pub fn member_filter(name: &str, control: bool) -> Result<Filter, ScanError> {
    let base = if control { "control.tar" } else { "data.tar" };
    let suffix = name.strip_prefix(base).ok_or(ScanError::Invalid)?;
    match suffix {
        "" => Ok(Filter::Identity),
        ".gz" => Ok(Filter::Gzip),
        ".xz" => Ok(Filter::Xz),
        ".zst" => Ok(Filter::Zstd),
        ".bz2" if !control => Ok(Filter::Bzip2),
        ".lzma" if !control => Ok(Filter::Lzma),
        _ => Err(ScanError::Unsupported),
    }
}
fn version(bytes: &[u8]) -> Result<(), ScanError> {
    if bytes.last() != Some(&b'\n') || bytes.split(|b| *b == b'\n').count() > 65 {
        return Err(ScanError::Invalid);
    }
    let first = bytes
        .split(|b| *b == b'\n')
        .next()
        .ok_or(ScanError::Invalid)?;
    let dot = first
        .iter()
        .position(|b| *b == b'.')
        .ok_or(ScanError::Invalid)?;
    if first.iter().any(|b| !b.is_ascii_digit() && *b != b'.') || first[dot + 1..].contains(&b'.') {
        return Err(ScanError::Invalid);
    }
    let major = number(&first[..dot], 10)?;
    number(&first[dot + 1..], 10)?;
    if major != 2 {
        return Err(ScanError::Unsupported);
    }
    Ok(())
}
/// Parse the complete common-ar envelope with exact bounds and DEB member ordering.
pub fn read_envelope(
    source: &mut dyn ReadSource,
    control: &Cancellation,
    limits: ScanLimits,
) -> Result<Envelope, ScanError> {
    control.check()?;
    if source
        .volume_lengths()
        .is_some_and(|volumes| volumes.len() != 1)
    {
        return Err(ScanError::Unsupported);
    }
    let length = source
        .seek(SeekFrom::End(0))
        .map_err(|error| ScanError::from_source_error(&error))?;
    if length > limits.read_bytes {
        return Err(ScanError::Limit);
    }
    let mut input = Input {
        source,
        control,
        work: 0,
        limit: limits.read_bytes,
    };
    input.at(0)?;
    let mut magic = [0; 8];
    input.exact(&mut magic)?;
    if &magic != b"!<arch>\n" {
        return Err(ScanError::Unsupported);
    }
    let mut position = 8;
    let mut members = Vec::new();
    let mut names = 0usize;
    while position < length {
        if members.len() >= limits.entries {
            return Err(ScanError::Limit);
        }
        if length - position < 60 {
            return Err(ScanError::Invalid);
        }
        input.at(position)?;
        let mut header = [0; 60];
        input.exact(&mut header)?;
        if &header[58..] != b"`\n" {
            return Err(ScanError::Invalid);
        }
        let member_name = name(&header[..16])?;
        names = names
            .checked_add(member_name.len())
            .ok_or(ScanError::Limit)?;
        if member_name.len() > limits.name_bytes || names > limits.total_name_bytes {
            return Err(ScanError::Limit);
        }
        let modified = number(&header[16..28], 10)?;
        number(&header[28..34], 10)?;
        number(&header[34..40], 10)?;
        number(&header[40..48], 8)?;
        let size = number(&header[48..58], 10)?;
        let offset = position.checked_add(60).ok_or(ScanError::Invalid)?;
        let end = offset.checked_add(size).ok_or(ScanError::Invalid)?;
        let next = end
            .checked_add(size & 1)
            .filter(|end| *end <= length)
            .ok_or(ScanError::Invalid)?;
        if members.is_empty() {
            if member_name != "debian-binary" {
                return Err(ScanError::Unsupported);
            }
            if size > 4096 {
                return Err(ScanError::Limit);
            }
            let mut bytes = vec![0; size as usize];
            input.exact(&mut bytes)?;
            version(&bytes)?;
        }
        if size & 1 != 0 {
            input.at(end)?;
            let mut padding = [0];
            input.exact(&mut padding)?;
            if padding != *b"\n" {
                return Err(ScanError::Invalid);
            }
        }
        members.push(Member {
            name: member_name,
            offset,
            size,
            modified,
        });
        position = next;
    }
    if members.is_empty() {
        return Err(ScanError::Invalid);
    }
    let mut next_required = 0;
    let mut indices = [0; 2];
    for (at, member) in members.iter().enumerate().skip(1) {
        if next_required == 2 {
            break;
        }
        if member.name.starts_with('_') {
            continue;
        }
        let control = next_required == 0;
        let prefix = if control { "control.tar" } else { "data.tar" };
        if !member.name.starts_with(prefix) {
            return Err(ScanError::Unsupported);
        }
        member_filter(&member.name, control)?;
        indices[next_required] = at;
        next_required += 1;
    }
    if next_required != 2 {
        return Err(ScanError::Invalid);
    }
    control.check()?;
    Ok(Envelope {
        members,
        control: indices[0],
        data: indices[1],
        length,
        read_work: input.work,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/deb")
                .join(name),
        )
        .unwrap()
    }
    #[test]
    fn independently_verified_envelopes_cover_every_allowed_member_filter_and_extensions() {
        let cancel = Default::default();
        for control in ["none", "gz", "xz", "zst"] {
            for data in ["none", "gz", "xz", "zst", "bz2", "lzma"] {
                let bytes = fixture(&format!("control-{control}-data-{data}.deb"));
                let layout =
                    read_envelope(&mut Cursor::new(&bytes), &cancel, ScanLimits::default())
                        .unwrap();
                assert_eq!(layout.members.len(), 3);
                assert_eq!(layout.control, 1);
                assert_eq!(layout.data, 2);
                assert_eq!(layout.length, bytes.len() as u64);
                assert!(layout.read_work < layout.length);
                for member in layout.members {
                    assert!(member.offset + member.size <= layout.length);
                }
            }
        }
        let layout = read_envelope(
            &mut Cursor::new(fixture("extensions.deb")),
            &cancel,
            ScanLimits::default(),
        )
        .unwrap();
        assert_eq!(layout.members.len(), 6);
        assert_eq!((layout.control, layout.data), (2, 4));
        assert_eq!(layout.members[5].name, "after-data");
    }
    #[test]
    fn envelope_truncation_numbers_names_padding_versions_and_limits_are_explicit() {
        let bytes = fixture("control-gz-data-xz.deb");
        let cancel = Default::default();
        for end in 0..bytes.len() {
            assert!(
                read_envelope(
                    &mut Cursor::new(&bytes[..end]),
                    &cancel,
                    ScanLimits::default()
                )
                .is_err(),
                "prefix {end}"
            );
        }
        for (at, value) in [
            (8, b'/'),
            (8 + 16, b'-'),
            (8 + 28, b'x'),
            (8 + 34, b'x'),
            (8 + 40, b'8'),
            (8 + 48, b'x'),
            (8 + 58, b'x'),
            (68, b'3'),
            (70, b'a'),
        ] {
            let mut bad = bytes.clone();
            bad[at] = value;
            assert!(
                read_envelope(&mut Cursor::new(bad), &cancel, ScanLimits::default()).is_err(),
                "byte {at}"
            );
        }
        let mut bad = fixture("extensions.deb");
        let layout = read_envelope(&mut Cursor::new(&bad), &cancel, ScanLimits::default()).unwrap();
        let odd = &layout.members[1];
        bad[(odd.offset + odd.size) as usize] = 0;
        assert_eq!(
            read_envelope(&mut Cursor::new(bad), &cancel, ScanLimits::default()).err(),
            Some(ScanError::Invalid)
        );
        for limits in [
            ScanLimits {
                entries: 2,
                ..ScanLimits::default()
            },
            ScanLimits {
                read_bytes: bytes.len() as u64 - 1,
                ..ScanLimits::default()
            },
        ] {
            assert_eq!(
                read_envelope(&mut Cursor::new(&bytes), &cancel, limits).err(),
                Some(ScanError::Limit)
            );
        }
        let cancel = Cancellation::default();
        cancel.cancel();
        assert_eq!(
            read_envelope(&mut Cursor::new(bytes), &cancel, ScanLimits::default()).err(),
            Some(ScanError::Cancelled)
        );
    }
}
