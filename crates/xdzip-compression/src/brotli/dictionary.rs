use super::data::{DEPTHS, DICTIONARY, LUT0, LUT1, LUT2, TRANSFORMS};
use crate::invalid;
use std::io;
pub(super) fn context(mode: u8, last: u8, before: u8) -> io::Result<usize> {
    let last = usize::from(last);
    let before = usize::from(before);
    Ok(usize::from(match mode {
        0 => (last as u8) & 63,
        1 => (last as u8) >> 2,
        2 => LUT0[last] | LUT1[before],
        3 => (LUT2[last] << 3) | LUT2[before],
        _ => return Err(invalid()),
    }))
}
/// Apply only normative byte transforms; no Unicode parsing or external dictionaries.
pub(super) fn word(length: usize, id: usize) -> io::Result<([u8; 37], usize)> {
    if !(4..=24).contains(&length) {
        return Err(invalid());
    }
    let depth = DEPTHS[length];
    let transform = id >> depth;
    let &(prefix, op, suffix) = TRANSFORMS.get(transform).ok_or_else(invalid)?;
    let offset: usize = (4..length).map(|n| n * (1 << DEPTHS[n])).sum();
    let offset = offset + (id & ((1 << depth) - 1)) * length;
    let base = DICTIONARY
        .get(offset..offset + length)
        .ok_or_else(invalid)?;
    let base = match op {
        3..=11 => &base[usize::from(op - 2).min(length)..],
        12..=20 => &base[..length.saturating_sub(usize::from(op - 11))],
        _ => base,
    };
    let mut output = [0; 37];
    let start = prefix.len();
    let end = start + base.len();
    let size = end + suffix.len();
    if size > output.len() {
        return Err(invalid());
    }
    output[..start].copy_from_slice(prefix);
    output[start..end].copy_from_slice(base);
    output[end..size].copy_from_slice(suffix);
    if matches!(op, 1 | 2) {
        let mut at = start;
        while at < end {
            let n = match output[at] {
                0..=191 => {
                    if output[at].is_ascii_lowercase() {
                        output[at] ^= 32;
                    }
                    1
                }
                192..=223 => {
                    if at + 1 < end {
                        output[at + 1] ^= 32;
                    }
                    2
                }
                _ => {
                    if at + 2 < end {
                        output[at + 2] ^= 5;
                    }
                    3
                }
            };
            if op == 1 {
                break;
            }
            at += n;
        }
    }
    Ok((output, size))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normative_crcs_and_word_ranges_are_exact() {
        for (bytes, crc) in [
            (DICTIONARY.as_slice(), 0x5136cb04),
            (LUT0.as_slice(), 0x8e91efb7),
            (LUT1.as_slice(), 0xd01a32f4),
            (LUT2.as_slice(), 0x0dd7a0d6),
        ] {
            assert_eq!(crate::crc32(bytes), crc);
        }
        let mut count = 0;
        for (length, &depth) in DEPTHS.iter().enumerate().skip(4) {
            for index in [0, (1 << depth) - 1] {
                for transform in 0..121 {
                    let (bytes, n) = word(length, (transform << depth) + index).unwrap();
                    assert!(n <= 37);
                    assert_eq!(bytes[n..], [0; 37][n..]);
                    count += 1;
                }
            }
            assert!(word(length, 121 << depth).is_err());
        }
        assert_eq!(count, 5082);
        assert!(word(3, 0).is_err());
        assert!(word(25, 0).is_err());
        let (bytes, n) = word(4, 0).unwrap();
        assert_eq!(&bytes[..n], b"time");
        let (bytes, n) = word(4, 9 << 10).unwrap();
        assert_eq!(&bytes[..n], b"Time");
        let (bytes, n) = word(4, 44 << 10).unwrap();
        assert_eq!(&bytes[..n], b"TIME");
        let (_, n) = word(4, 64 << 10).unwrap();
        assert_eq!(n, 0);
    }
    #[test]
    fn all_literal_contexts_stay_within_the_declared_64_rows() {
        for last in 0..=255 {
            for before in 0..=255 {
                for mode in 0..4 {
                    assert!(context(mode, last, before).unwrap() < 64);
                }
            }
        }
        assert!(context(4, 0, 0).is_err());
    }
}
