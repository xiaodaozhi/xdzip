// crates/xdzip-codec-zip/src/names.rs

//! ZIP Unicode metadata has priority over heuristics, never over an explicit UTF-8 flag.
use rawzip::extra_fields::ExtraFields;
use xdzip_codec_api::{
    Entry,
    names::{NameEncoding, decode_legacy},
};

pub(super) fn unicode_path(
    raw: &[u8],
    utf8: bool,
    mut fields: ExtraFields<'_>,
) -> Result<Option<String>, ()> {
    if utf8 {
        return Ok(None);
    }
    let mut name = None;
    let checksum = rawzip::crc32(raw);
    for (id, bytes) in fields.by_ref() {
        if id.as_u16() != 0x7075 {
            continue;
        }
        let version = bytes.first().ok_or(())?;
        if *version != 1 {
            continue;
        }
        let crc = bytes.get(1..5).ok_or(())?;
        // Stale checksums and unknown versions are ignored as specified by APPNOTE 4.6.9.
        if u32::from_le_bytes(crc.try_into().map_err(|_| ())?) != checksum {
            continue;
        }
        if name.is_some() {
            return Err(());
        }
        name = Some(std::str::from_utf8(&bytes[5..]).map_err(|_| ())?.to_owned());
    }
    if !fields.remaining_bytes().is_empty() {
        return Err(());
    }
    Ok(name)
}

/// Recheck the frozen resolution using the original bytes, flag and Unicode extra. A
/// detector is not run a second time during payload reads, so selection cannot drift.
pub(super) fn matches(entry: &Entry, utf8: bool, fields: ExtraFields<'_>) -> bool {
    let Ok(unicode) = unicode_path(&entry.raw_name, utf8, fields) else {
        return entry.name_encoding == NameEncoding::Unresolved;
    };
    match entry.name_encoding {
        NameEncoding::Utf8 => {
            (utf8 || entry.raw_name.is_ascii())
                && unicode.is_none()
                && std::str::from_utf8(&entry.raw_name).is_ok()
        }
        NameEncoding::UnmarkedUtf8 => {
            !utf8
                && unicode.is_none()
                && entry
                    .decoded_name
                    .as_deref()
                    .is_some_and(|s| s.as_bytes() == entry.raw_name)
        }
        NameEncoding::UnicodePath => {
            !utf8 && unicode.is_some() && unicode.as_ref() == entry.decoded_name.as_ref()
        }
        NameEncoding::Legacy(label) => {
            !utf8
                && unicode.is_none()
                && entry.decoded_name.is_some()
                && decode_legacy(&entry.raw_name, label) == entry.decoded_name
        }
        NameEncoding::Unresolved => entry.warning,
    }
}
