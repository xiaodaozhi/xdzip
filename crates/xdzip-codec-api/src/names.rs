// crates/xdzip-codec-api/src/names.rs

//! Host-independent filename resolution. Raw identity and escaped display remain separate.
use crate::{Cancellation, Entry, ScanError, ScanLimits};
use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use encoding_rs::{Encoding, GB18030, GBK, WINDOWS_1252};
use std::fmt::Write;

/// Origin of an effective filename, retained to bind payload verification to the same spelling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameEncoding {
    /// ASCII or explicitly declared UTF-8, without an alternate name allocation.
    Utf8,
    /// Valid UTF-8 in an unlabelled field.
    UnmarkedUtf8,
    /// ZIP Unicode Path extra with a verified checksum of the original name.
    UnicodePath,
    /// Statistical archive-level result or the ZIP CP437 fallback.
    Legacy(&'static str),
    /// Explicit declaration is damaged, or strict decoding failed.
    Unresolved,
}

/// ZIP's original DOS default. This fixed character map is independent of the host locale.
const CP437_HIGH: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å', 'É', 'æ', 'Æ',
    'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ', 'á', 'í', 'ó', 'ú', 'ñ', 'Ñ',
    'ª', 'º', '¿', '⌐', '¬', '½', '¼', '¡', '«', '»', '░', '▒', '▓', '│', '┤', '╡', '╢', '╖', '╕',
    '╣', '║', '╗', '╝', '╜', '╛', '┐', '└', '┴', '┬', '├', '─', '┼', '╞', '╟', '╚', '╔', '╩', '╦',
    '╠', '═', '╬', '╧', '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫', '╪', '┘', '┌', '█', '▄', '▌', '▐',
    '▀', 'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ', 'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩', '≡', '±',
    '≥', '≤', '⌠', '⌡', '÷', '≈', '°', '∙', '\u{00b7}', '√', 'ⁿ', '²', '■', '\u{00a0}',
];

/// Decode the complete field without replacement characters or BOM stripping.
pub fn decode_legacy(raw: &[u8], label: &str) -> Option<String> {
    if label == "CP437" {
        return Some(
            raw.iter()
                .map(|b| {
                    if *b < 128 {
                        char::from(*b)
                    } else {
                        CP437_HIGH[usize::from(*b - 128)]
                    }
                })
                .collect(),
        );
    }
    Encoding::for_label(label.as_bytes())?
        .decode_without_bom_handling_and_without_replacement(raw)
        .map(|text| text.into_owned())
}

fn guess(detector: &EncodingDetector, zip: bool) -> &'static str {
    let encoding = detector.guess(None, Utf8Detection::Deny);
    if zip && encoding == WINDOWS_1252 {
        "CP437"
    } else if encoding == GBK {
        GB18030.name()
    } else {
        encoding.name()
    }
}
fn local_guess(raw: &[u8], zip: bool) -> &'static str {
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
    detector.feed(raw, true);
    guess(&detector, zip)
}
/// Escape controls, including bidi controls, without changing an output name.
pub fn display(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if display_control(c) {
            write!(out, "\\u{{{:X}}}", u32::from(c)).expect("String formatting");
        } else {
            out.push(c);
        }
    }
    out
}
/// Preserve undecodable original bytes visibly instead of inserting U+FFFD.
pub fn escaped_bytes(raw: &[u8]) -> String {
    let mut out = String::new();
    for b in raw {
        if b.is_ascii_graphic() || *b == b' ' {
            out.push(char::from(*b));
        } else {
            write!(out, "\\x{b:02X}").expect("String formatting");
        }
    }
    out
}
fn display_control(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}
fn suspicious(text: &str, directory: bool) -> bool {
    text.is_empty()
        || text.starts_with('/')
        || text.contains(['\\', ':'])
        || text.split('/').any(|part| part == "..")
        || text.chars().any(display_control)
        || text.ends_with('/') != directory
}

/// Finalize one complete scan. `legacy` admits detection only for fields without a Unicode
/// declaration. Sample at most 1 MiB of complete unlabelled non-UTF8 names, separated so that
/// a truncated multibyte sequence cannot borrow bytes from the next filename. No host paths,
/// system code page, GUI language, content bytes or network services influence the result.
pub fn resolve(
    rows: &mut [Entry],
    legacy: &[bool],
    zip: bool,
    control: &Cancellation,
    limits: ScanLimits,
) -> Result<(), ScanError> {
    resolve_internal(rows, legacy, zip, false, control, limits)
}

/// CAB paths use backslashes as separators. Convert only decoded characters, so a
/// Shift-JIS trailing 0x5c remains part of its character rather than a separator.
pub fn resolve_cab(
    rows: &mut [Entry],
    legacy: &[bool],
    control: &Cancellation,
    limits: ScanLimits,
) -> Result<(), ScanError> {
    resolve_internal(rows, legacy, false, true, control, limits)
}

/// Resolve an unlabelled CAB volume reference without consulting the host locale.
pub fn decode_volume_name(raw: &[u8]) -> Option<String> {
    std::str::from_utf8(raw)
        .map(str::to_owned)
        .ok()
        .or_else(|| decode_legacy(raw, local_guess(raw, false)))
}

fn resolve_internal(
    rows: &mut [Entry],
    legacy: &[bool],
    zip: bool,
    cab_separators: bool,
    control: &Cancellation,
    limits: ScanLimits,
) -> Result<(), ScanError> {
    if rows.len() != legacy.len() {
        return Err(ScanError::Invalid);
    }
    let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
    let mut sample = 0usize;
    for (row, allowed) in rows.iter().zip(legacy) {
        control.check()?;
        if *allowed
            && row.name_encoding != NameEncoding::Unresolved
            && row.decoded_name.is_none()
            && std::str::from_utf8(&row.raw_name).is_err()
        {
            if sample + row.raw_name.len() + 1 > 1024 * 1024 {
                break;
            }
            detector.feed(&row.raw_name, false);
            detector.feed(b"\n", false);
            sample += row.raw_name.len() + 1;
        }
    }
    detector.feed(b"", true);
    let encoding = guess(&detector, zip);
    let mut text_bytes = 0usize;
    for (row, allowed) in rows.iter_mut().zip(legacy) {
        control.check()?;
        if row.name_encoding != NameEncoding::Unresolved && row.decoded_name.is_none() {
            match std::str::from_utf8(&row.raw_name) {
                Ok(_) if !*allowed || row.raw_name.is_ascii() => {}
                Ok(text) if sample == 0 => {
                    row.decoded_name = Some(text.to_owned());
                    row.name_encoding = NameEncoding::UnmarkedUtf8;
                }
                _ if *allowed => {
                    // A GBK component can itself be valid UTF-8. Apply the archive's
                    // legacy result consistently so explicit and implicit parents agree.
                    let decoded = decode_legacy(&row.raw_name, encoding)
                        .map(|name| (name, NameEncoding::Legacy(encoding)))
                        .or_else(|| {
                            std::str::from_utf8(&row.raw_name)
                                .ok()
                                .map(|name| (name.to_owned(), NameEncoding::UnmarkedUtf8))
                        })
                        .or_else(|| {
                            let local = local_guess(&row.raw_name, zip);
                            decode_legacy(&row.raw_name, local)
                                .map(|name| (name, NameEncoding::Legacy(local)))
                        });
                    if let Some((name, encoding)) = decoded {
                        row.decoded_name = Some(name);
                        row.name_encoding = encoding;
                    } else {
                        row.name_encoding = NameEncoding::Unresolved;
                    }
                }
                _ => row.name_encoding = NameEncoding::Unresolved,
            }
        }
        if cab_separators && row.name_encoding != NameEncoding::Unresolved {
            let name = std::str::from_utf8(row.name_bytes()).map_err(|_| ScanError::Invalid)?;
            if name.contains('\\') {
                row.decoded_name = Some(name.replace('\\', "/"));
            }
        }
        let name = std::str::from_utf8(row.name_bytes()).ok();
        let warning = row.name_encoding == NameEncoding::Unresolved
            || name.is_none_or(|s| suspicious(s, row.directory));
        let text = if row.name_encoding == NameEncoding::Unresolved {
            escaped_bytes(&row.raw_name)
        } else {
            display(name.ok_or(ScanError::Invalid)?)
        };
        row.warning |= warning;
        row.display_name = text;
        let decoded = row.decoded_name.as_ref().map_or(0, String::len);
        text_bytes = text_bytes
            .checked_add(row.decoded_name.as_ref().map_or(0, String::capacity))
            .and_then(|n| n.checked_add(row.display_name.capacity()))
            .ok_or(ScanError::Limit)?;
        if decoded > limits.name_bytes || text_bytes > limits.display_name_bytes {
            return Err(ScanError::Limit);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn row(raw_name: Vec<u8>) -> Entry {
        Entry {
            format: crate::Format::Cab,
            id: xdzip_domain::identity::EntryId::try_from(1).unwrap(),
            raw_name,
            decoded_name: None,
            name_encoding: NameEncoding::Utf8,
            display_name: String::new(),
            directory: false,
            link: false,
            unix_mode: 0,
            size: 0,
            packed_size: 0,
            method: 0,
            modified_dos: 0,
            encrypted: false,
            password_supported: false,
            warning: false,
        }
    }
    #[test]
    fn native_names_decode_before_converting_cabinet_separators() {
        let text = "folder\\ソフトウェアの設定ファイルと日本語.txt";
        let (encoded, _, failed) = encoding_rs::SHIFT_JIS.encode(text);
        assert!(!failed);
        assert!(encoded.windows(2).any(|bytes| bytes == [0x83, 0x5c]));
        let raw = encoded.into_owned();
        let mut rows = vec![row(raw.clone())];
        resolve_cab(
            &mut rows,
            &[true],
            &Default::default(),
            ScanLimits::default(),
        )
        .unwrap();
        assert_eq!(rows[0].raw_name, raw);
        assert_eq!(rows[0].name_bytes(), text.replace('\\', "/").as_bytes());
        assert!(!rows[0].warning);
        assert_eq!(rows[0].name_encoding, NameEncoding::Legacy("Shift_JIS"));
    }
    #[test]
    fn declared_native_utf8_damage_and_normalized_traversal_remain_unusable() {
        let mut rows = vec![
            row(vec![0xff]),
            row(b"..\\escape".to_vec()),
            row("nested\\中文.txt".as_bytes().to_vec()),
        ];
        resolve_cab(
            &mut rows,
            &[false; 3],
            &Default::default(),
            ScanLimits::default(),
        )
        .unwrap();
        assert_eq!(rows[0].name_encoding, NameEncoding::Unresolved);
        assert!(rows[0].warning);
        assert_eq!(rows[1].name_bytes(), b"../escape");
        assert!(rows[1].warning);
        assert_eq!(rows[2].name_bytes(), "nested/中文.txt".as_bytes());
        assert!(!rows[2].warning);
    }
}
