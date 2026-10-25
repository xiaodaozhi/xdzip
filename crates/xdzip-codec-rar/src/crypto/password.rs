use super::DecodeError;
use zeroize::Zeroizing;

#[derive(Clone, Copy)]
pub(super) enum Profile {
    Windows,
    Unix,
}
impl Profile {
    pub fn native() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}
pub(super) struct PasswordBytes {
    bytes: Zeroizing<[u8; 508]>,
    length: usize,
}
impl PasswordBytes {
    pub fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
    fn push_scalar(&mut self, scalar: u32) {
        // This also deliberately emits the three-byte form of an unpaired
        // UTF-16 surrogate at the Windows reference's truncation boundary.
        let (count, prefix) = if scalar < 0x80 {
            (1, scalar)
        } else if scalar < 0x800 {
            (2, 0xc0 | (scalar >> 6))
        } else if scalar < 0x10000 {
            (3, 0xe0 | (scalar >> 12))
        } else {
            (4, 0xf0 | (scalar >> 18))
        };
        self.bytes[self.length] = prefix as u8;
        for i in 1..count {
            self.bytes[self.length + i] = 0x80 | ((scalar >> (6 * (count - 1 - i))) & 63) as u8;
        }
        self.length += count;
    }
}
pub(super) fn encode_password(
    password: &str,
    profile: Profile,
) -> Result<PasswordBytes, DecodeError> {
    if password.len() > 4096 {
        return Err(DecodeError::Limit);
    }
    // Native SecPassword terminates before projection; outer admission above
    // still counts the complete original UTF8 input, including ignored suffix.
    let password = password
        .split_once('\0')
        .map_or(password, |(prefix, _)| prefix);
    let mut result = PasswordBytes {
        bytes: Zeroizing::new([0; 508]),
        length: 0,
    };
    match profile {
        Profile::Unix => {
            for scalar in password.chars().take(127) {
                result.push_scalar(scalar as u32);
            }
        }
        Profile::Windows => {
            let mut units = Zeroizing::new([0u16; 127]);
            let mut count = 0;
            for unit in password.encode_utf16().take(127) {
                units[count] = unit;
                count += 1;
            }
            let mut at = 0;
            while at < count {
                let unit = units[at];
                at += 1;
                if (0xd800..=0xdbff).contains(&unit)
                    && at < count
                    && (0xdc00..=0xdfff).contains(&units[at])
                {
                    result.push_scalar(
                        0x10000 + (((unit as u32 - 0xd800) << 10) | (units[at] as u32 - 0xdc00)),
                    );
                    at += 1;
                } else {
                    result.push_scalar(unit.into());
                }
            }
        }
    }
    Ok(result)
}
