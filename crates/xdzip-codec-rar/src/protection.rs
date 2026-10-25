use crate::crypto::{CheckResult, KdfBudget, KdfParameters, KeyCache, KeyMaterial};
use crate::index::Crypt;
use crate::legacy_crypto::{
    LegacyAesCache, LegacyCipher, LegacyKdfBudget, NarrowPassword, Old20Cache,
};
use xdzip_codec_api::{Cancellation, ScanError, payload::DecodeError};

/// One scan/opened payload owns its cache and cumulative KDF cost. IVs belong
/// to individual streams and do not grant a new derivation allowance.
#[derive(Default)]
pub(crate) struct Protection {
    keys: KeyCache,
    budget: KdfBudget,
    legacy_keys: LegacyAesCache,
    legacy_budget: LegacyKdfBudget,
    old20: Old20Cache,
}
impl Protection {
    pub const MEMORY_BYTES: u64 = KeyCache::MEMORY_BYTES
        + LegacyAesCache::MEMORY_BYTES
        + Old20Cache::MEMORY_BYTES
        + LegacyCipher::MEMORY_BYTES;
    pub fn legacy_header(
        &mut self,
        password: Option<&str>,
        salt: [u8; 8],
        control: &Cancellation,
    ) -> Result<LegacyCipher, DecodeError> {
        let password = password.ok_or(DecodeError::PasswordRequired)?;
        let key = self
            .legacy_keys
            .key(password, Some(salt), control, &mut self.legacy_budget)?;
        Ok(LegacyCipher::aes128(key))
    }
    pub fn legacy_content(
        &mut self,
        d: &crate::index::Descriptor,
        password: Option<&str>,
        control: &Cancellation,
    ) -> Result<LegacyCipher, DecodeError> {
        let password = password.ok_or(DecodeError::PasswordRequired)?;
        if password.len() > 4096 {
            return Err(DecodeError::Limit);
        }
        let prefix = password
            .split_once('\0')
            .map_or(password, |(prefix, _)| prefix);
        if cfg!(windows) && d.host == 0 && !prefix.is_ascii() {
            return Err(DecodeError::Unsupported);
        }
        match if d.ancient { 13 } else { d.method } {
            13 => Ok(LegacyCipher::old13(&NarrowPassword::ascii(prefix)?)),
            15 => Ok(LegacyCipher::old15(&NarrowPassword::ascii(prefix)?)),
            20 | 26 => self.old20.cipher(
                &NarrowPassword::ascii(prefix)?,
                control,
                &mut self.legacy_budget,
            ),
            _ => {
                let salt = d
                    .crypt
                    .as_ref()
                    .map(|c| {
                        c.salt
                            .as_slice()
                            .try_into()
                            .map_err(|_| DecodeError::Integrity)
                    })
                    .transpose()?;
                let key = self
                    .legacy_keys
                    .key(password, salt, control, &mut self.legacy_budget)?;
                Ok(LegacyCipher::aes128(key))
            }
        }
    }
    pub fn key(
        &mut self,
        crypt: &Crypt,
        password: Option<&str>,
        service: bool,
        control: &Cancellation,
    ) -> Result<&KeyMaterial, DecodeError> {
        if crypt.version != 0 {
            return Err(DecodeError::Unsupported);
        }
        let password = password.ok_or(DecodeError::PasswordRequired)?;
        let parameters = KdfParameters {
            salt: crypt
                .salt
                .as_slice()
                .try_into()
                .map_err(|_| DecodeError::Integrity)?,
            exponent: crypt.exponent,
        };
        let key = self
            .keys
            .key(parameters, password, control, &mut self.budget)?;
        if crypt
            .check
            .as_ref()
            .is_some_and(|check| key.verify_check(check, service) == CheckResult::TrustedMismatch)
        {
            return Err(DecodeError::WrongPassword);
        }
        Ok(key)
    }
}

pub(crate) fn scan_error(error: DecodeError) -> ScanError {
    match error {
        DecodeError::Cancelled => ScanError::Cancelled,
        DecodeError::Limit => ScanError::Limit,
        DecodeError::Unsupported => ScanError::Unsupported,
        DecodeError::PasswordRequired => ScanError::PasswordRequired,
        DecodeError::WrongPassword => ScanError::WrongPassword,
        DecodeError::SourceIo | DecodeError::Output => ScanError::Io,
        DecodeError::Integrity => ScanError::Invalid,
    }
}
