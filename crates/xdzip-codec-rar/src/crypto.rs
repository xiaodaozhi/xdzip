//! RAR5/7 crypto-version-zero primitives, without container or path authority.
//!
//! Frozen mathematical facts define the three checkpoints of one PBKDF2 chain.
//! CBC emits every authorized cipher block; callers authenticate true payload
//! and explicitly discard at most their admitted alignment tail. No PKCS#7 or
//! zero-padding heuristic is applied. This module does not implement a writer.

mod cbc;
mod password;
#[cfg(test)]
mod tests;

pub(crate) use cbc::CbcReader;
use password::{PasswordBytes, Profile, encode_password};
use xdzip_codec_api::{Cancellation, payload::DecodeError};
use xdzip_crypto::{HmacSha256, SecretSha256};
use zeroize::Zeroizing;

/// Salt and work exponent for RAR crypto version zero. The IV is not KDF input.
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) struct KdfParameters {
    pub salt: [u8; 16],
    pub exponent: u8,
}

/// Cumulative task quota, shared by all header and content key derivations.
/// Hash bytes count input to the underlying hash, including HMAC pads once per
/// key and each inner-message / outer-digest, but not SHA's own padding.
pub(crate) struct KdfBudget {
    max_rounds: u64,
    max_hash_bytes: u64,
    rounds: u64,
    hash_bytes: u64,
}
impl Default for KdfBudget {
    fn default() -> Self {
        Self::new(1 << 26, 4 << 30)
    }
}
impl KdfBudget {
    pub fn new(max_rounds: u64, max_hash_bytes: u64) -> Self {
        Self {
            max_rounds,
            max_hash_bytes,
            rounds: 0,
            hash_bytes: 0,
        }
    }
    #[cfg(test)]
    pub fn rounds(&self) -> u64 {
        self.rounds
    }
    #[cfg(test)]
    pub fn hash_bytes(&self) -> u64 {
        self.hash_bytes
    }
    fn admit(&self, rounds: u64, bytes: u64) -> Result<(), DecodeError> {
        if self
            .rounds
            .checked_add(rounds)
            .is_none_or(|n| n > self.max_rounds)
            || self
                .hash_bytes
                .checked_add(bytes)
                .is_none_or(|n| n > self.max_hash_bytes)
        {
            return Err(DecodeError::Limit);
        }
        Ok(())
    }
}

/// Independent evidence from the optional fast verifier, not content integrity.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum CheckResult {
    TrustedMatch,
    TrustedMismatch,
    Untrusted,
}

/// Task-secret AES key, keyed-hash key and folded password check. No Debug or
/// Clone implementation can accidentally log or retain another copy.
pub(crate) struct KeyMaterial {
    aes: Zeroizing<[u8; 32]>,
    hash: Zeroizing<[u8; 32]>,
    check: Zeroizing<[u8; 8]>,
}
impl KeyMaterial {
    /// Invalid verifier checksum and the historical all-zero service verifier
    /// are untrusted. They never independently prove a wrong password.
    pub fn verify_check(&self, check: &[u8; 12], service: bool) -> CheckResult {
        let mut hash = SecretSha256::new();
        hash.update(&check[..8]);
        if !equal(&hash.finish()[..4], &check[8..])
            || (service && check[..8].iter().all(|&b| b == 0))
        {
            return CheckResult::Untrusted;
        }
        if equal(self.check.as_ref(), &check[..8]) {
            CheckResult::TrustedMatch
        } else {
            CheckResult::TrustedMismatch
        }
    }
    /// Applies the optional keyed CRC transform. The caller decides which
    /// complete plaintext or physical split-ciphertext bytes the CRC covers.
    pub fn tweak_crc(&self, crc: u32) -> u32 {
        let mac = HmacSha256::new(self.hash.as_ref()).authenticate(&crc.to_le_bytes());
        mac.as_chunks::<4>()
            .0
            .iter()
            .fold(0, |value, part| value ^ u32::from_le_bytes(*part))
    }
    /// Applies the optional full-width HMAC transform to a normal BLAKE2sp hash.
    pub fn tweak_blake(&self, hash: [u8; 32]) -> [u8; 32] {
        *HmacSha256::new(self.hash.as_ref()).authenticate(&hash)
    }
}
fn equal(a: &[u8], b: &[u8]) -> bool {
    // Fixed-work comparison for equal-sized protocol values; no early exit on
    // the first differing password-check byte. This is not a compiler timing
    // guarantee and is not a replacement for a full content integrity check.
    if a.len() != b.len() {
        return false;
    }
    let mut difference = Zeroizing::new(0u8);
    for (&x, &y) in a.iter().zip(b) {
        *difference |= x ^ y;
    }
    *difference == 0
}

struct Cached {
    parameters: KdfParameters,
    password: PasswordBytes,
    key: KeyMaterial,
}
/// Bounded task-local cache. A password-profile change erases all old entries;
/// salt/exponent changes require another admitted derivation. IVs may differ.
pub(crate) struct KeyCache {
    entries: [Option<Cached>; 16],
    next: usize,
}
impl Default for KeyCache {
    fn default() -> Self {
        Self {
            entries: std::array::from_fn(|_| None),
            next: 0,
        }
    }
}
impl KeyCache {
    /// Conservative storage preflight including inline cache and KDF scratch.
    pub const MEMORY_BYTES: u64 = 16 * 1024;
    pub fn key(
        &mut self,
        parameters: KdfParameters,
        password: &str,
        control: &Cancellation,
        budget: &mut KdfBudget,
    ) -> Result<&KeyMaterial, DecodeError> {
        check(control)?;
        if parameters.exponent > 24 {
            return Err(DecodeError::Limit);
        }
        let encoded = encode_password(password, Profile::native())?;
        if self
            .entries
            .iter()
            .flatten()
            .next()
            .is_some_and(|old| !equal(old.password.as_slice(), encoded.as_slice()))
        {
            for entry in &mut self.entries {
                *entry = None;
            }
            self.next = 0;
        }
        if let Some(index) = self.entries.iter().position(|entry| {
            entry.as_ref().is_some_and(|old| {
                old.parameters == parameters && equal(old.password.as_slice(), encoded.as_slice())
            })
        }) {
            return Ok(&self.entries[index].as_ref().expect("matched entry").key);
        }
        let key = derive(parameters, &encoded, control, budget)?;
        let index = self.next;
        self.entries[index] = Some(Cached {
            parameters,
            password: encoded,
            key,
        });
        self.next = (index + 1) % self.entries.len();
        Ok(&self.entries[index].as_ref().expect("inserted entry").key)
    }
}

fn derive(
    p: KdfParameters,
    password: &PasswordBytes,
    control: &Cancellation,
    budget: &mut KdfBudget,
) -> Result<KeyMaterial, DecodeError> {
    check(control)?;
    let base = 1u64
        .checked_shl(p.exponent.into())
        .filter(|_| p.exponent <= 24)
        .ok_or(DecodeError::Limit)?;
    let calls = base + 32;
    let normalize = if password.as_slice().len() > 64 {
        password.as_slice().len() as u64
    } else {
        0
    };
    let setup_bytes = normalize + 128;
    // First inner message is salt16 || BE32(1); subsequent ones are U[32].
    budget.admit(calls, setup_bytes + 52 + 64 * (calls - 1))?;
    let hmac = HmacSha256::new(password.as_slice());
    budget.hash_bytes += setup_bytes;
    let mut first = Zeroizing::new([0u8; 20]);
    first[..16].copy_from_slice(&p.salt);
    first[19] = 1;
    let mut previous = hmac.authenticate(first.as_ref());
    let mut accumulated = Zeroizing::new([0u8; 32]);
    let mut aes = Zeroizing::new([0u8; 32]);
    let mut hash = Zeroizing::new([0u8; 32]);
    for round in 1..=calls {
        if round != 1 {
            previous = hmac.authenticate(previous.as_ref());
        }
        budget.rounds += 1;
        budget.hash_bytes += if round == 1 { 52 } else { 64 };
        for (result, &value) in accumulated.iter_mut().zip(previous.iter()) {
            *result ^= value;
        }
        if round == base {
            aes.copy_from_slice(accumulated.as_ref());
        }
        if round == base + 16 {
            hash.copy_from_slice(accumulated.as_ref());
        }
        if round & 255 == 0 {
            check(control)?;
        }
    }
    check(control)?;
    let mut folded = Zeroizing::new([0u8; 8]);
    for (i, &byte) in accumulated.iter().enumerate() {
        folded[i % 8] ^= byte;
    }
    Ok(KeyMaterial {
        aes,
        hash,
        check: folded,
    })
}
fn check(control: &Cancellation) -> Result<(), DecodeError> {
    control.check().map_err(|_| DecodeError::Cancelled)
}
fn stable_error(error: std::io::Error) -> std::io::Error {
    if error.kind() == std::io::ErrorKind::Interrupted && error.get_ref().is_some() {
        std::io::Error::other(error.into_inner().expect("typed error"))
    } else {
        error
    }
}
