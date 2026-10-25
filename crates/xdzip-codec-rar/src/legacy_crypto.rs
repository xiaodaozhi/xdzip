//! Historical read-only RAR password profiles, separate from modern crypto.
//! Legacy finite constants retain their restricted-reference provenance. These
//! primitives alone do not establish historical archive interoperability.

mod cipher;
mod reader;
mod sha1;
#[cfg(test)]
mod tests;

pub(crate) use cipher::{LegacyCipher, NarrowPassword, Old20Cache};
pub(crate) use reader::LegacyReader;
use xdzip_codec_api::{Cancellation, payload::DecodeError};
use zeroize::Zeroizing;

const ROUNDS: u64 = 1 << 18;

/// Separate cumulative task allowance for the legacy archive generation.
/// Work counts SHA input rounds and old20 permutation/mixing operations.
pub(crate) struct LegacyKdfBudget {
    work_left: u64,
    bytes_left: u64,
}
impl Default for LegacyKdfBudget {
    fn default() -> Self {
        Self::new(1 << 26, 4 << 30)
    }
}
impl LegacyKdfBudget {
    pub fn new(work: u64, bytes: u64) -> Self {
        Self {
            work_left: work,
            bytes_left: bytes,
        }
    }
    pub(super) fn admit(&mut self, work: u64, bytes: u64) -> Result<(), DecodeError> {
        let next_work = self.work_left.checked_sub(work).ok_or(DecodeError::Limit)?;
        let next_bytes = self
            .bytes_left
            .checked_sub(bytes)
            .ok_or(DecodeError::Limit)?;
        self.work_left = next_work;
        self.bytes_left = next_bytes;
        Ok(())
    }
}

/// Sealed RAR3 AES128 key and derived IV. Neither is a RAR5 parameter.
pub(crate) struct LegacyAesKey {
    key: Zeroizing<[u8; 16]>,
    iv: Zeroizing<[u8; 16]>,
}
struct WideBytes {
    bytes: Zeroizing<[u8; 254]>,
    len: usize,
}
impl WideBytes {
    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}
#[derive(Clone, Copy)]
enum Profile {
    Windows,
    Unix,
}
impl Profile {
    fn native() -> Self {
        if cfg!(windows) {
            Self::Windows
        } else {
            Self::Unix
        }
    }
}
fn wide_password(password: &str, profile: Profile) -> Result<WideBytes, DecodeError> {
    if password.len() > 4096 {
        return Err(DecodeError::Limit);
    }
    // Native SecPassword terminates before projection; outer admission above
    // still counts the complete original UTF8 input, including ignored suffix.
    let password = password
        .split_once('\0')
        .map_or(password, |(prefix, _)| prefix);
    let mut result = WideBytes {
        bytes: Zeroizing::new([0; 254]),
        len: 0,
    };
    match profile {
        Profile::Windows => {
            for unit in password.encode_utf16().take(127) {
                result.bytes[result.len..result.len + 2].copy_from_slice(&unit.to_le_bytes());
                result.len += 2;
            }
        }
        Profile::Unix => {
            for scalar in password.chars().take(127) {
                result.bytes[result.len..result.len + 2]
                    .copy_from_slice(&(scalar as u32 as u16).to_le_bytes());
                result.len += 2;
            }
        }
    }
    Ok(result)
}
struct Cached {
    password: WideBytes,
    salt: Option<[u8; 8]>,
    key: LegacyAesKey,
}
/// Task-local fixed cache; absent salt and present zero salt remain distinct.
pub(crate) struct LegacyAesCache {
    entries: [Option<Cached>; 16],
    next: usize,
}
impl Default for LegacyAesCache {
    fn default() -> Self {
        Self {
            entries: std::array::from_fn(|_| None),
            next: 0,
        }
    }
}
impl LegacyAesCache {
    pub const MEMORY_BYTES: u64 = 16 * 1024;
    pub fn key(
        &mut self,
        password: &str,
        salt: Option<[u8; 8]>,
        control: &Cancellation,
        budget: &mut LegacyKdfBudget,
    ) -> Result<&LegacyAesKey, DecodeError> {
        check(control)?;
        let encoded = wide_password(password, Profile::native())?;
        self.key_encoded(encoded, salt, control, budget)
    }
    /// Accepts the already terminated/truncated native-wide low16 units of an
    /// explicit host projection, for example Windows' conditional OEM step.
    /// A zero low16 unit is valid here (Unix nonBMP can project to zero); the
    /// host must resolve its original native NUL prefix before losing high bits.
    #[cfg(test)]
    pub fn key_projected_units(
        &mut self,
        units: &[u16],
        salt: Option<[u8; 8]>,
        control: &Cancellation,
        budget: &mut LegacyKdfBudget,
    ) -> Result<&LegacyAesKey, DecodeError> {
        check(control)?;
        if units.len() > 127 {
            return Err(DecodeError::Limit);
        }
        let mut encoded = WideBytes {
            bytes: Zeroizing::new([0; 254]),
            len: 2 * units.len(),
        };
        for (part, unit) in encoded.bytes[..encoded.len]
            .as_chunks_mut::<2>()
            .0
            .iter_mut()
            .zip(units)
        {
            part.copy_from_slice(&unit.to_le_bytes());
        }
        self.key_encoded(encoded, salt, control, budget)
    }
    fn key_encoded(
        &mut self,
        encoded: WideBytes,
        salt: Option<[u8; 8]>,
        control: &Cancellation,
        budget: &mut LegacyKdfBudget,
    ) -> Result<&LegacyAesKey, DecodeError> {
        if self
            .entries
            .iter()
            .flatten()
            .next()
            .is_some_and(|old| !same(old.password.as_slice(), encoded.as_slice()))
        {
            for entry in &mut self.entries {
                *entry = None;
            }
            self.next = 0;
        }
        if let Some(index) = self.entries.iter().position(|old| {
            old.as_ref().is_some_and(|old| {
                old.salt == salt && same(old.password.as_slice(), encoded.as_slice())
            })
        }) {
            return Ok(&self.entries[index].as_ref().expect("matched key").key);
        }
        let key = derive(&encoded, salt, control, budget)?;
        let index = self.next;
        self.entries[index] = Some(Cached {
            password: encoded,
            salt,
            key,
        });
        self.next = (index + 1) % 16;
        Ok(&self.entries[index].as_ref().expect("inserted key").key)
    }
}
fn same(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut difference = Zeroizing::new(0u8);
    for (&x, &y) in a.iter().zip(b) {
        *difference |= x ^ y;
    }
    *difference == 0
}
fn derive(
    password: &WideBytes,
    salt: Option<[u8; 8]>,
    control: &Cancellation,
    budget: &mut LegacyKdfBudget,
) -> Result<LegacyAesKey, DecodeError> {
    check(control)?;
    let len = password.len + if salt.is_some() { 8 } else { 0 };
    // Include at most two SHA padding blocks for each checkpoint and the final
    // digest in addition to every real input byte. Admission precedes hashing.
    budget.admit(ROUNDS, (len as u64 + 3) * ROUNDS + 17 * 128)?;
    let mut raw = Zeroizing::new([0u8; 262]);
    raw[..password.len].copy_from_slice(password.as_slice());
    if let Some(salt) = salt {
        raw[password.len..len].copy_from_slice(&salt);
    }
    let mut hash = sha1::Sha1::new();
    let mut iv = Zeroizing::new([0u8; 16]);
    for round in 0..ROUNDS {
        hash.update(&raw[..len]);
        hash.update(&(round as u32).to_le_bytes()[..3]);
        if round.is_multiple_of(ROUNDS / 16) {
            iv[(round / (ROUNDS / 16)) as usize] = hash.clone().finish()[19];
        }
        // The first fragment completes a pre-existing SHA block and is never
        // rewritten, including when its length is exactly 64.
        let first = 64 - ((round * (len as u64 + 3)) % 64) as usize;
        for at in (first..len).step_by(64) {
            if len - at < 64 {
                break;
            }
            let words = sha1::schedule(&raw[at..at + 64]);
            for (part, &word) in raw[at..at + 64]
                .as_chunks_mut::<4>()
                .0
                .iter_mut()
                .zip(&words[64..])
            {
                part.copy_from_slice(&word.to_le_bytes());
            }
        }
        if round & 255 == 0 {
            check(control)?;
        }
    }
    check(control)?;
    let digest = hash.finish();
    let mut key = Zeroizing::new([0u8; 16]);
    for (dst, src) in key
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(digest[..16].as_chunks::<4>().0)
    {
        for i in 0..4 {
            dst[i] = src[3 - i];
        }
    }
    Ok(LegacyAesKey { key, iv })
}
pub(super) fn check(control: &Cancellation) -> Result<(), DecodeError> {
    control.check().map_err(|_| DecodeError::Cancelled)
}
pub(super) fn stable_error(error: std::io::Error) -> std::io::Error {
    if error.kind() == std::io::ErrorKind::Interrupted && error.get_ref().is_some() {
        std::io::Error::other(error.into_inner().expect("typed error"))
    } else {
        error
    }
}
