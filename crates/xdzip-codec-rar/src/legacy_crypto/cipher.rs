use super::{DecodeError, LegacyAesKey, LegacyKdfBudget, check};
use aes::{
    Aes128,
    cipher::{BlockCipherDecrypt, KeyInit},
};
use xdzip_codec_api::Cancellation;
use zeroize::Zeroizing;

/// Explicit host-projected historical narrow password. UTF8 is not implicitly
/// treated as ACP/OEM/locale encoding. This preparation truncates at 127 bytes
/// and at the first NUL, as required by the old byte profiles.
pub(crate) struct NarrowPassword {
    bytes: Zeroizing<[u8; 127]>,
    len: usize,
}
impl NarrowPassword {
    pub fn from_projected(bytes: &[u8]) -> Result<Self, DecodeError> {
        if bytes.len() > 4096 {
            return Err(DecodeError::Limit);
        }
        let len = bytes
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(bytes.len())
            .min(127);
        let mut result = Self {
            bytes: Zeroizing::new([0; 127]),
            len,
        };
        result.bytes[..len].copy_from_slice(&bytes[..len]);
        Ok(result)
    }
    /// Cross-platform ASCII is unambiguous; nonASCII needs explicit projection.
    pub fn ascii(password: &str) -> Result<Self, DecodeError> {
        if password.len() > 4096 {
            return Err(DecodeError::Limit);
        }
        let mut count = 0;
        for scalar in password.chars().take(127) {
            if scalar == '\0' {
                break;
            }
            if !scalar.is_ascii() {
                return Err(DecodeError::Unsupported);
            }
            count += 1;
        }
        Self::from_projected(&password.as_bytes()[..count])
    }
    fn as_slice(&self) -> &[u8] {
        &self.bytes[..self.len]
    }
}

struct Prepared20 {
    password: NarrowPassword,
    words: Zeroizing<[u32; 4]>,
    permutation: Zeroizing<[u8; 256]>,
}
/// One active projected password has one immutable old20 initial state. Every
/// file receives a fresh copy, while split pieces keep the same running cipher.
#[derive(Default)]
pub(crate) struct Old20Cache {
    entry: Option<Prepared20>,
}
impl Old20Cache {
    pub const MEMORY_BYTES: u64 = 1024;
    pub fn cipher(
        &mut self,
        password: &NarrowPassword,
        control: &Cancellation,
        budget: &mut LegacyKdfBudget,
    ) -> Result<LegacyCipher, DecodeError> {
        check(control)?;
        let matches = self
            .entry
            .as_ref()
            .is_some_and(|old| super::same(old.password.as_slice(), password.as_slice()));
        if !matches {
            self.entry = None;
            let initial = LegacyCipher::old20(password, control, budget)?;
            self.entry = Some(Prepared20 {
                password: NarrowPassword::from_projected(password.as_slice())?,
                words: initial.words.clone(),
                permutation: initial.permutation.clone(),
            });
        }
        let initial = self.entry.as_ref().expect("prepared password");
        let mut cipher = LegacyCipher::blank(Kind::Old20);
        cipher.words.copy_from_slice(initial.words.as_ref());
        cipher
            .permutation
            .copy_from_slice(initial.permutation.as_ref());
        Ok(cipher)
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Old13,
    Old15,
    Old20,
    Aes128,
}
/// Fixed-storage read transform. File boundaries create fresh state; physical
/// piece boundaries do not. Unused state fields remain zero, avoiding heap/OOM
/// or a secret-containing boxed enum solely for the smaller historical modes.
pub(crate) struct LegacyCipher {
    kind: Kind,
    words: Zeroizing<[u32; 4]>,
    permutation: Zeroizing<[u8; 256]>,
    aes: Option<Aes128>,
    chain: Zeroizing<[u8; 16]>,
}
impl LegacyCipher {
    pub const MEMORY_BYTES: u64 = 4096;
    fn blank(kind: Kind) -> Self {
        Self {
            kind,
            words: Zeroizing::new([0; 4]),
            permutation: Zeroizing::new([0; 256]),
            aes: None,
            chain: Zeroizing::new([0; 16]),
        }
    }
    pub fn old13(password: &NarrowPassword) -> Self {
        let mut result = Self::blank(Kind::Old13);
        for &p in password.as_slice() {
            result.words[0] = u32::from((result.words[0] as u8).wrapping_add(p));
            result.words[1] ^= u32::from(p);
            result.words[2] = u32::from((result.words[2] as u8).wrapping_add(p).rotate_left(1));
        }
        result
    }
    pub fn old15(password: &NarrowPassword) -> Self {
        let mut result = Self::blank(Kind::Old15);
        let mut crc = 0xffff_ffffu32;
        for &p in password.as_slice() {
            crc = (crc >> 8) ^ table((crc as u8) ^ p);
            result.words[2] ^= (u32::from(p) ^ table(p)) & 0xffff;
            result.words[3] = u32::from(
                (result.words[3] as u16)
                    .wrapping_add(u16::from(p).wrapping_add((table(p) >> 16) as u16)),
            );
        }
        result.words[0] = crc & 0xffff;
        result.words[1] = crc >> 16;
        result
    }
    pub fn old20(
        password: &NarrowPassword,
        control: &Cancellation,
        budget: &mut LegacyKdfBudget,
    ) -> Result<Self, DecodeError> {
        check(control)?;
        let bytes = password.as_slice();
        let mut operations = 0u64;
        // The permutation schedule depends on password/CRC, not on prior swaps,
        // so its entire bounded cost can be admitted before mutating key state.
        for j in 0..256u16 {
            for i in (0..bytes.len()).step_by(2) {
                let u = table(bytes[i].wrapping_sub(j as u8)) as u8;
                let v = table(bytes.get(i + 1).copied().unwrap_or(0).wrapping_add(j as u8)) as u8;
                operations += u64::from(v.wrapping_sub(u)) + 1;
            }
        }
        budget.admit(
            operations + bytes.len().div_ceil(16) as u64 * 32,
            bytes.len() as u64,
        )?;
        let mut result = Self::blank(Kind::Old20);
        *result.words = [0xd3a3b879, 0x3f6d12f7, 0x7515a235, 0xa4e7f123];
        result.permutation.copy_from_slice(&PERMUTATION);
        for j in 0..256u16 {
            for i in (0..bytes.len()).step_by(2) {
                let u = table(bytes[i].wrapping_sub(j as u8)) as u8;
                let v = table(bytes.get(i + 1).copied().unwrap_or(0).wrapping_add(j as u8)) as u8;
                for t in 0..usize::from(v.wrapping_sub(u)) {
                    result.permutation.swap(
                        (usize::from(u) + t) % 256,
                        (usize::from(u) + 2 * t + i + 1) % 256,
                    );
                }
            }
            if j.is_multiple_of(8) {
                check(control)?;
            }
        }
        for group in bytes.chunks(16) {
            let mut block = Zeroizing::new([0; 16]);
            block[..group.len()].copy_from_slice(group);
            result.old20_map(block.as_mut(), true);
            result.feedback(block.as_ref());
        }
        check(control)?;
        Ok(result)
    }
    pub fn aes128(key: &LegacyAesKey) -> Self {
        let mut result = Self::blank(Kind::Aes128);
        result.aes = Some(Aes128::new_from_slice(key.key.as_ref()).expect("AES128 key size"));
        result.chain.copy_from_slice(key.iv.as_ref());
        result
    }
    pub(super) fn width(&self) -> usize {
        match self.kind {
            Kind::Old13 | Kind::Old15 => 1,
            _ => 16,
        }
    }
    pub(super) fn apply(&mut self, block: &mut [u8]) {
        debug_assert_eq!(block.len(), self.width());
        match self.kind {
            Kind::Old13 => {
                self.words[1] = u32::from((self.words[1] as u8).wrapping_add(self.words[2] as u8));
                self.words[0] = u32::from((self.words[0] as u8).wrapping_add(self.words[1] as u8));
                block[0] = block[0].wrapping_sub(self.words[0] as u8);
            }
            Kind::Old15 => {
                let a = (self.words[0] as u16).wrapping_add(0x1234);
                let crc = table(((a >> 1) & 255) as u8);
                let b = self.words[1] as u16 ^ crc as u16;
                let c = (self.words[2] as u16).wrapping_sub((crc >> 16) as u16);
                let d = ((self.words[3] as u16).rotate_right(1) ^ b).rotate_right(1);
                *self.words = [
                    u32::from(a ^ c ^ d),
                    u32::from(b),
                    u32::from(c),
                    u32::from(d),
                ];
                block[0] ^= (self.words[0] >> 8) as u8;
            }
            Kind::Old20 => {
                let original = Zeroizing::new(<[u8; 16]>::try_from(&*block).expect("block size"));
                self.old20_map(block, false);
                self.feedback(original.as_ref());
            }
            Kind::Aes128 => {
                let original = Zeroizing::new(<[u8; 16]>::try_from(&*block).expect("block size"));
                let aes_block: &mut aes::cipher::Block<Aes128> =
                    block.try_into().expect("block size");
                self.aes
                    .as_ref()
                    .expect("AES state")
                    .decrypt_block(aes_block);
                for (byte, &previous) in block.iter_mut().zip(self.chain.iter()) {
                    *byte ^= previous;
                }
                self.chain.copy_from_slice(original.as_ref());
            }
        }
    }
    fn substitute(&self, word: u32) -> u32 {
        word.to_le_bytes()
            .iter()
            .enumerate()
            .fold(0, |value, (i, &byte)| {
                value | (u32::from(self.permutation[usize::from(byte)]) << (8 * i))
            })
    }
    fn old20_map(&self, block: &mut [u8], forward: bool) {
        let mut words = Zeroizing::new([0u32; 4]);
        for ((word, bytes), &key) in words
            .iter_mut()
            .zip(block.as_chunks::<4>().0)
            .zip(self.words.iter())
        {
            *word = u32::from_le_bytes(*bytes) ^ key;
        }
        for step in 0..32 {
            let round = if forward { step } else { 31 - step };
            let [a, b, c, d] = *words;
            let key = self.words[round % 4];
            *words = [
                c,
                d,
                a ^ self.substitute(c.wrapping_add(d.rotate_left(11)) ^ key),
                b ^ self.substitute((d ^ c.rotate_left(17)).wrapping_add(key)),
            ];
        }
        let order = [2, 3, 0, 1];
        for ((part, &index), &key) in block
            .as_chunks_mut::<4>()
            .0
            .iter_mut()
            .zip(&order)
            .zip(self.words.iter())
        {
            part.copy_from_slice(&(words[index] ^ key).to_le_bytes());
        }
    }
    fn feedback(&mut self, bytes: &[u8]) {
        for (i, word) in self.words.iter_mut().enumerate() {
            for at in (i..16).step_by(4) {
                *word ^= table(bytes[at]);
            }
        }
    }
}
fn table(byte: u8) -> u32 {
    let mut value = u32::from(byte);
    for _ in 0..8 {
        value = (value >> 1) ^ if value & 1 != 0 { 0xedb88320 } else { 0 };
    }
    value
}

// Functional finite mapping from frozen rar20-cipher-permutation-v1.csv.
// SHA256 c38eb742ed1231b555a35ff7ee882b829eabaff62958109d7b8b84b68d96e928.
// Its source is the fixed LicenseRef-UnRAR reference crypt2.cpp9..26, not an
// independent primary specification or a claim of project-original constants.
// Restricted origin and specialized license review remain separately recorded.
const PERMUTATION: [u8; 256] = include!("permutation.rs");
