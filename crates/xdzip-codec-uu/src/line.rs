use xdzip_codec_api::ScanError;
/// Actual six-bit alphabet of one complete member.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Encoding {
    Uu,
    Xx,
    Base64,
}
fn value(byte: u8, encoding: Encoding) -> Option<u8> {
    match encoding {
        Encoding::Uu => match byte {
            b' '..=b'_' => Some(byte - b' '),
            b'`' => Some(0),
            _ => None,
        },
        Encoding::Xx => match byte {
            b'+' => Some(0),
            b'-' => Some(1),
            b'0'..=b'9' => Some(byte - b'0' + 2),
            b'A'..=b'Z' => Some(byte - b'A' + 12),
            b'a'..=b'z' => Some(byte - b'a' + 38),
            _ => None,
        },
        Encoding::Base64 => match byte {
            b'A'..=b'Z' => Some(byte - b'A'),
            b'a'..=b'z' => Some(byte - b'a' + 26),
            b'0'..=b'9' => Some(byte - b'0' + 52),
            b'+' => Some(62),
            b'/' => Some(63),
            _ => None,
        },
    }
}
/// Decode one UU/XX line. Unused final sextets are padding, not file bytes;
/// historical UU encoders need not zero them. No short-line repair is attempted.
pub(crate) fn counted(line: &[u8], encoding: Encoding) -> Result<Vec<u8>, ScanError> {
    let size = usize::from(
        value(*line.first().ok_or(ScanError::Invalid)?, encoding).ok_or(ScanError::Invalid)?,
    );
    if line.len() != 1 + size.div_ceil(3) * 4 {
        return Err(ScanError::Invalid);
    }
    let mut output = Vec::with_capacity(size.div_ceil(3) * 3);
    for group in line[1..].as_chunks::<4>().0 {
        let mut values = [0u8; 4];
        for (decoded, byte) in values.iter_mut().zip(group) {
            *decoded = value(*byte, encoding).ok_or(ScanError::Invalid)?;
        }
        output.extend_from_slice(&[
            (values[0] << 2) | (values[1] >> 4),
            (values[1] << 4) | (values[2] >> 2),
            (values[2] << 6) | values[3],
        ]);
    }
    output.truncate(size);
    Ok(output)
}
/// A complete multiple-of-four Base64 line. Padding closes the member body.
pub(crate) fn base64(line: &[u8]) -> Result<(Vec<u8>, bool), ScanError> {
    if line.is_empty() || line.len() > 76 || !line.len().is_multiple_of(4) {
        return Err(ScanError::Invalid);
    }
    let mut output = Vec::with_capacity(line.len() / 4 * 3);
    let mut padded = false;
    for group in line.as_chunks::<4>().0 {
        if padded {
            return Err(ScanError::Invalid);
        }
        let a = value(group[0], Encoding::Base64).ok_or(ScanError::Invalid)?;
        let b = value(group[1], Encoding::Base64).ok_or(ScanError::Invalid)?;
        output.push((a << 2) | (b >> 4));
        if group[2] == b'=' {
            if group[3] != b'=' || b & 15 != 0 {
                return Err(ScanError::Invalid);
            }
            padded = true;
        } else {
            let c = value(group[2], Encoding::Base64).ok_or(ScanError::Invalid)?;
            output.push((b << 4) | (c >> 2));
            if group[3] == b'=' {
                if c & 3 != 0 {
                    return Err(ScanError::Invalid);
                }
                padded = true;
            } else {
                let d = value(group[3], Encoding::Base64).ok_or(ScanError::Invalid)?;
                output.push((c << 6) | d);
            }
        }
    }
    Ok((output, padded))
}
/// A counted line's exact width resolves the otherwise shared header syntax.
pub(crate) fn identify(line: &[u8]) -> Result<Encoding, ScanError> {
    match (
        counted(line, Encoding::Uu).is_ok(),
        counted(line, Encoding::Xx).is_ok(),
    ) {
        (true, false) => Ok(Encoding::Uu),
        (false, true) => Ok(Encoding::Xx),
        _ => Err(ScanError::Invalid),
    }
}
