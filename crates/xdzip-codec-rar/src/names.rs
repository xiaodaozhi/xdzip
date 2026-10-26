use crate::source::Reader;
use crate::wire::Wire;
use xdzip_codec_api::{Entry, ScanError, ScanLimits};

pub(crate) fn compressed(raw: &[u8]) -> Result<Option<String>, ScanError> {
    let Some(split) = raw.iter().position(|b| *b == 0) else {
        return Ok(None);
    };
    let narrow = &raw[..split];
    let encoded = &raw[split + 1..];
    if encoded.is_empty() {
        return Ok(std::str::from_utf8(narrow).ok().map(str::to_owned));
    }
    let mut input = Wire::new(encoded);
    let high = u16::from(input.byte()?) << 8;
    let mut output = Vec::<u16>::new();
    while input.remaining() > 0 {
        let flags = input.byte()?;
        for command in 0..4 {
            if input.remaining() == 0 {
                break;
            }
            match (flags >> (6 - command * 2)) & 3 {
                0 => output.push(u16::from(input.byte()?)),
                1 => output.push(high | u16::from(input.byte()?)),
                2 => output.push(input.u16()?),
                3 => {
                    let n = input.byte()?;
                    let corrected = n & 128 != 0;
                    let count = usize::from(n & 127) + 2;
                    let correction = if corrected { input.byte()? } else { 0 };
                    let end = output
                        .len()
                        .checked_add(count)
                        .filter(|end| *end <= narrow.len())
                        .ok_or(ScanError::Invalid)?;
                    for byte in &narrow[output.len()..end] {
                        if !corrected && *byte >= 128 && !cfg!(windows) {
                            return Err(ScanError::Unsupported);
                        }
                        output.push(if corrected {
                            high | u16::from(byte.wrapping_add(correction))
                        } else if *byte < 128 {
                            u16::from(*byte)
                        } else {
                            // The frozen Windows native profile extends
                            // a signed plain-char into its 16-bit unit.
                            u16::from_le_bytes([*byte, 0xff])
                        });
                    }
                }
                _ => unreachable!(),
            }
            if output.len() > 4096 {
                return Err(ScanError::Limit);
            }
        }
    }
    if output.is_empty() {
        return Ok(std::str::from_utf8(narrow).ok().map(str::to_owned));
    }
    let result = String::from_utf16(&output).map_err(|_| ScanError::Invalid)?;
    if result.contains('\0') {
        return Err(ScanError::Invalid);
    }
    Ok(Some(result))
}
pub(crate) fn resolve_rows(
    rows: &mut [Entry],
    legacy: &[bool],
    warnings: &[bool],
    reader: &Reader<'_>,
    limits: ScanLimits,
) -> Result<(), ScanError> {
    // The shared resolver detects the archive's legacy character set before any
    // backslash conversion, preserving multibyte characters containing 0x5c.
    let control = reader.control();
    reader.check()?;
    xdzip_codec_api::names::resolve(rows, legacy, false, control, limits)?;
    for (row, warning) in rows.iter_mut().zip(warnings) {
        reader.check()?;
        if row.name_encoding != xdzip_codec_api::names::NameEncoding::Unresolved {
            let name = std::str::from_utf8(row.name_bytes()).map_err(|_| ScanError::Invalid)?;
            let old = row.method < 50;
            if old && name.contains('\\') {
                row.decoded_name = Some(name.replace('\\', "/"));
            }
            if row.directory && !row.name_bytes().ends_with(b"/") {
                let mut name = std::str::from_utf8(row.name_bytes())
                    .map_err(|_| ScanError::Invalid)?
                    .to_owned();
                name.push('/');
                row.decoded_name = Some(name);
            }
        }
        row.warning = *warning;
    }
    xdzip_codec_api::names::resolve(rows, &vec![false; rows.len()], false, control, limits)?;
    reader.check()
}
