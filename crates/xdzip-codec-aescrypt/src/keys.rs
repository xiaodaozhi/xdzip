use crate::{framing::Header, work::Work};
use aes::{
    Aes256,
    cipher::{BlockCipherDecrypt, KeyInit},
};
use xdzip_codec_api::{Cancellation, ScanError};
use xdzip_crypto::{HmacSha256, HmacSha512, SecretSha256};
use zeroize::{Zeroize, Zeroizing};
pub(crate) struct Material {
    pub key: Zeroizing<[u8; 32]>,
    pub iv: Zeroizing<[u8; 16]>,
}
pub(crate) struct Cbc {
    cipher: Aes256,
    chain: Zeroizing<[u8; 16]>,
}
impl Cbc {
    pub fn new(key: &[u8; 32], iv: [u8; 16]) -> Self {
        Self {
            cipher: Aes256::new_from_slice(key).expect("fixed AES256 key"),
            chain: Zeroizing::new(iv),
        }
    }
    pub fn decrypt(
        &mut self,
        bytes: &mut [u8],
        cancel: &Cancellation,
        work: &mut Work,
    ) -> Result<(), ScanError> {
        if !bytes.len().is_multiple_of(16) {
            return Err(ScanError::Invalid);
        }
        for bytes in bytes.as_chunks_mut::<16>().0 {
            cancel.check()?;
            work.aes(16)?;
            let input = *bytes;
            let mut block = aes::cipher::Block::<Aes256>::from(input);
            self.cipher.decrypt_block(&mut block);
            for (byte, previous) in block.iter_mut().zip(self.chain.iter()) {
                *byte ^= *previous;
            }
            bytes.copy_from_slice(&block);
            block.as_mut_slice().zeroize();
            self.chain.copy_from_slice(&input);
        }
        Ok(())
    }
}
pub(crate) struct Mac {
    value: HmacSha256,
    bytes: u64,
}
impl Mac {
    pub fn new(key: &[u8; 32], work: &mut Work) -> Result<Self, ScanError> {
        work.hash(128)?;
        Ok(Self {
            value: HmacSha256::new(key),
            bytes: 0,
        })
    }
    pub fn update(&mut self, bytes: &[u8], work: &mut Work) -> Result<(), ScanError> {
        let next = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or(ScanError::Limit)?;
        work.hash((next / 64 - self.bytes / 64) * 64)?;
        self.value.update(bytes);
        self.bytes = next;
        Ok(())
    }
    pub fn finish(self, work: &mut Work) -> Result<Zeroizing<[u8; 32]>, ScanError> {
        work.hash(if self.bytes % 64 <= 55 { 128 } else { 192 })?;
        Ok(self.value.finish())
    }
}
fn derived(
    header: &Header,
    password: &str,
    cancel: &Cancellation,
    work: &mut Work,
) -> Result<Zeroizing<[u8; 32]>, ScanError> {
    cancel.check()?;
    if password.len() > 4096 {
        return Err(ScanError::Limit);
    }
    if header.version < 3 {
        let mut encoded = Zeroizing::new(Vec::with_capacity(password.len() * 2));
        for unit in password.encode_utf16() {
            encoded.extend_from_slice(&unit.to_le_bytes());
        }
        let mut key = Zeroizing::new([0; 32]);
        key[..16].copy_from_slice(&header.iv);
        let blocks = (32 + encoded.len() as u64 + 9).div_ceil(64) * 64;
        for at in 0..8192 {
            if at % 128 == 0 {
                cancel.check()?;
            }
            work.hash(blocks)?;
            let mut hash = SecretSha256::new();
            hash.update(key.as_ref());
            hash.update(encoded.as_ref());
            key.copy_from_slice(hash.finish().as_ref());
        }
        cancel.check()?;
        return Ok(key);
    }
    let password = password.as_bytes();
    if password.len() > 128 {
        work.hash((password.len() as u64 + 17).div_ceil(128) * 128)?;
    }
    work.hash(256)?;
    let mac = HmacSha512::new(password);
    let mut first = [0; 20];
    first[..16].copy_from_slice(&header.iv);
    first[19] = 1;
    work.hash(256)?;
    let mut u = mac.authenticate(&first);
    let mut combined = Zeroizing::new(*u);
    for at in 1..header.iterations {
        if at % 128 == 0 {
            cancel.check()?;
        }
        work.hash(256)?;
        u = mac.authenticate(u.as_ref());
        for (out, byte) in combined.iter_mut().zip(u.iter()) {
            *out ^= *byte;
        }
    }
    cancel.check()?;
    let mut key = Zeroizing::new([0; 32]);
    key.copy_from_slice(&combined[..32]);
    Ok(key)
}
pub(crate) fn unwrap(
    header: &Header,
    password: &str,
    cancel: &Cancellation,
    work: &mut Work,
) -> Result<Material, ScanError> {
    let key = derived(header, password, cancel, work)?;
    let Some(wrapped) = header.wrapped else {
        return Ok(Material {
            key,
            iv: Zeroizing::new(header.iv),
        });
    };
    let mut mac = Mac::new(&key, work)?;
    mac.update(&wrapped, work)?;
    if header.version == 3 {
        mac.update(&[3], work)?;
    }
    if !constant_time_eq::constant_time_eq_32(
        mac.finish(work)?.as_ref().try_into().expect("MAC width"),
        header.session_tag.as_ref().ok_or(ScanError::Invalid)?,
    ) {
        return Err(ScanError::WrongPassword);
    }
    let mut session = Zeroizing::new(wrapped);
    Cbc::new(&key, header.iv).decrypt(session.as_mut(), cancel, work)?;
    let mut iv = Zeroizing::new([0; 16]);
    iv.copy_from_slice(&session[..16]);
    let mut key = Zeroizing::new([0; 32]);
    key.copy_from_slice(&session[16..]);
    Ok(Material { key, iv })
}
