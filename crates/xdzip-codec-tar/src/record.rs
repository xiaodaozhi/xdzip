//! Independent TAR record primitives. Layout: POSIX pax, ustar interchange
//! format; GNU base-256 numeric fields are admitted only when nonnegative and
//! representable in u64. No filesystem operations or reference parser calls.
use xdzip_codec_api::ScanError;

pub(crate) const BLOCK: usize = 512;

fn terminated(bytes: &[u8]) -> &[u8] {
    &bytes[..bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len())]
}

pub(crate) fn number(bytes: &[u8]) -> Result<u64, ScanError> {
    let Some(&first) = bytes.first() else {
        return Err(ScanError::Invalid);
    };
    if first & 0x80 != 0 {
        if first & 0x40 != 0 {
            return Err(ScanError::Invalid);
        }
        return bytes[1..].iter().try_fold(u64::from(first & 0x7f), |n, b| {
            n.checked_mul(256)
                .and_then(|n| n.checked_add(u64::from(*b)))
                .ok_or(ScanError::Invalid)
        });
    }
    let field = bytes.trim_ascii();
    let end = field.iter().position(|b| *b == 0).unwrap_or(field.len());
    if field[end..].iter().any(|b| !matches!(b, 0 | b' ')) {
        return Err(ScanError::Invalid);
    }
    let digits = field[..end].trim_ascii();
    digits.iter().try_fold(0u64, |n, b| {
        if !(b'0'..=b'7').contains(b) {
            return Err(ScanError::Invalid);
        }
        n.checked_mul(8)
            .and_then(|n| n.checked_add(u64::from(b - b'0')))
            .ok_or(ScanError::Invalid)
    })
}

pub(crate) struct Record<'a>(pub(crate) &'a [u8; BLOCK]);
impl Record<'_> {
    pub(crate) fn validate(&self) -> Result<(), ScanError> {
        let sum: u64 = self
            .0
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if (148..156).contains(&i) {
                    32
                } else {
                    u64::from(*b)
                }
            })
            .sum();
        if number(&self.0[148..156])? != sum {
            return Err(ScanError::Invalid);
        }
        Ok(())
    }
    pub(crate) fn kind(&self) -> u8 {
        self.0[156]
    }
    pub(crate) fn size(&self) -> Result<u64, ScanError> {
        number(&self.0[124..136])
    }
    pub(crate) fn timestamp(&self) -> Result<u64, ScanError> {
        number(&self.0[136..148])
    }
    pub(crate) fn name(&self) -> Vec<u8> {
        let mut name = Vec::new();
        // GNU headers use the ustar prefix area for unrelated fields.
        if &self.0[257..263] == b"ustar\0" {
            let prefix = terminated(&self.0[345..500]);
            if !prefix.is_empty() {
                name.extend_from_slice(prefix);
                name.push(b'/');
            }
        }
        name.extend_from_slice(terminated(&self.0[..100]));
        name
    }
}

/// A PAX record's decimal length includes its own digits, separator and newline.
pub(crate) struct Pax<'a>(pub(crate) &'a [u8]);
impl<'a> Iterator for Pax<'a> {
    type Item = Result<(&'a [u8], &'a [u8]), ScanError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.0.is_empty() {
            return None;
        }
        let data = std::mem::take(&mut self.0);
        let parsed = (|| {
            let space = data
                .iter()
                .position(|b| *b == b' ')
                .ok_or(ScanError::Invalid)?;
            if space == 0 {
                return Err(ScanError::Invalid);
            }
            let length = data[..space].iter().try_fold(0usize, |n, b| {
                if !b.is_ascii_digit() {
                    return Err(ScanError::Invalid);
                }
                n.checked_mul(10)
                    .and_then(|n| n.checked_add(usize::from(b - b'0')))
                    .ok_or(ScanError::Invalid)
            })?;
            let record = data.get(space + 1..length).ok_or(ScanError::Invalid)?;
            if record.last() != Some(&b'\n') || record.contains(&0) {
                return Err(ScanError::Invalid);
            }
            let pair = &record[..record.len() - 1];
            let equals = pair
                .iter()
                .position(|b| *b == b'=')
                .filter(|i| *i != 0)
                .ok_or(ScanError::Invalid)?;
            Ok((length, (&pair[..equals], &pair[equals + 1..])))
        })();
        Some(parsed.map(|(length, pair)| {
            self.0 = &data[length..];
            pair
        }))
    }
}

fn octal(field: &mut [u8], value: u64) -> Result<(), ScanError> {
    let text = format!("{value:o}");
    if text.len() >= field.len() {
        return Err(ScanError::Limit);
    }
    field.fill(b'0');
    let end = field.len() - 1;
    field[end] = 0;
    field[end - text.len()..end].copy_from_slice(text.as_bytes());
    Ok(())
}

pub(crate) fn seal(header: &mut [u8; BLOCK]) {
    header[148..156].fill(b' ');
    let sum: u64 = header.iter().map(|b| u64::from(*b)).sum();
    // A 512-byte record's unsigned checksum fits six octal digits.
    let text = format!("{sum:06o}\0 ");
    header[148..156].copy_from_slice(text.as_bytes());
}

pub(crate) fn set_name(header: &mut [u8; BLOCK], name: &[u8]) -> Result<(), ScanError> {
    if name.len() > 100 || name.contains(&0) {
        return Err(ScanError::Limit);
    }
    header[..100].fill(0);
    header[..name.len()].copy_from_slice(name);
    if &header[257..263] == b"ustar\0" {
        header[345..500].fill(0);
    }
    Ok(())
}

pub(crate) fn header(
    name: &[u8],
    kind: u8,
    size: u64,
    time: u64,
    mode: u32,
) -> Result<[u8; BLOCK], ScanError> {
    let mut bytes = [0u8; BLOCK];
    bytes[257..263].copy_from_slice(b"ustar\0");
    bytes[263..265].copy_from_slice(b"00");
    set_name(&mut bytes, name)?;
    octal(&mut bytes[100..108], u64::from(mode))?;
    octal(&mut bytes[108..116], 0)?;
    octal(&mut bytes[116..124], 0)?;
    octal(&mut bytes[124..136], size)?;
    octal(&mut bytes[136..148], time)?;
    bytes[156] = kind;
    seal(&mut bytes);
    Ok(bytes)
}

pub(crate) fn pax_field(body: &mut Vec<u8>, key: &str, value: &[u8]) {
    let content = key.len() + value.len() + 3; // space, equals, newline
    let mut length = content + 1;
    loop {
        let next = content + length.to_string().len();
        if length == next {
            break;
        }
        length = next;
    }
    body.extend_from_slice(format!("{length} {key}=").as_bytes());
    body.extend_from_slice(value);
    body.push(b'\n');
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_numeric_overflow_negative_and_hidden_suffix() {
        assert!(number(&[0xff; 12]).is_err());
        assert!(number(&[0x80, 1, 0, 0, 0, 0, 0, 0, 0, 0]).is_err());
        assert!(number(b"1\0x").is_err());
        assert!(number(b"8\0").is_err());
        assert_eq!(number(&[0x80, 1, 0]), Ok(256));
        assert_eq!(number(b" 00017\0 "), Ok(15));
    }
    #[test]
    fn rejects_invalid_pax_framing_without_looping() {
        for data in [
            b"0 x=y\n".as_slice(),
            b"999999999999999999999 x=y\n",
            b"8 x=y\n",
            b"7 =abc\n",
            b"7 x=\0\n",
        ] {
            let mut pax = Pax(data);
            assert!(pax.next().unwrap().is_err(), "{data:?}");
            assert!(pax.next().is_none());
        }
    }
    #[test]
    fn distinguishes_ustar_prefix_from_gnu_fields() {
        let mut bytes = header(b"file", b'0', 5, 0, 0o644).unwrap();
        bytes[345..348].copy_from_slice(b"dir");
        assert_eq!(Record(&bytes).name(), b"dir/file");
        bytes[262] = b' ';
        assert_eq!(Record(&bytes).name(), b"file");
    }
    #[test]
    fn bounded_mutations_never_panic_and_pax_records_progress() {
        let mut seed = 0x517cc1b727220a95u64;
        for length in 0..512 {
            let mut bytes = [0u8; 512];
            for byte in &mut bytes[..length] {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                *byte = seed as u8;
            }
            let record = Record(&bytes);
            let _ = record.validate();
            let _ = record.size();
            let _ = record.timestamp();
            let _ = record.name();
            let mut pax = Pax(&bytes[..length]);
            for _ in 0..=length {
                let before = pax.0.len();
                if pax.next().is_none() {
                    break;
                }
                assert!(pax.0.len() < before);
            }
        }
    }
}
