use super::{DecodeError, KeyMaterial, check, stable_error};
use aes::{
    Aes256,
    cipher::{BlockCipherDecrypt, KeyInit},
};
use std::io::{self, Read};
use xdzip_codec_api::Cancellation;
use zeroize::{Zeroize, Zeroizing};

/// Continuous CBC decryption of one admitted logical ciphertext range.
/// Physical-piece readers may split a block at any byte. Every cipher byte is
/// consumed exactly once; new headers/files require a new instance and IV.
pub(crate) struct CbcReader<'a, R> {
    source: R,
    aes: Aes256,
    chain: [u8; 16],
    block: Zeroizing<[u8; 16]>,
    at: usize,
    left: u64,
    consumed: u64,
    decoded: u64,
    failed: bool,
    finished: bool,
    control: &'a Cancellation,
}
impl<'a, R: Read> CbcReader<'a, R> {
    pub fn new(
        source: R,
        cipher_bytes: u64,
        key: &KeyMaterial,
        iv: [u8; 16],
        control: &'a Cancellation,
    ) -> io::Result<Self> {
        check(control).map_err(io::Error::other)?;
        if !cipher_bytes.is_multiple_of(16) {
            return Err(invalid());
        }
        Ok(Self {
            source,
            aes: Aes256::new_from_slice(key.aes.as_ref()).expect("AES256 key size"),
            chain: iv,
            block: Zeroizing::new([0; 16]),
            at: 16,
            left: cipher_bytes,
            consumed: 0,
            decoded: 0,
            failed: false,
            finished: false,
            control,
        })
    }
    pub fn consumed_bytes(&self) -> u64 {
        self.consumed
    }
    pub fn decoded_bytes(&self) -> u64 {
        self.decoded
    }
    pub fn remaining_plaintext(&self) -> u64 {
        self.left + (16 - self.at) as u64
    }
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        if let Err(error) = check(self.control) {
            self.failed = true;
            return Err(io::Error::other(error));
        }
        if self.remaining_plaintext() != 0 {
            self.failed = true;
            return Err(invalid());
        }
        self.finished = true;
        self.block.zeroize();
        Ok(())
    }
    /// Explicitly consumes the alignment tail only after the caller has proved
    /// payload termination. Values are unrestricted; no full extra block fits.
    pub fn finish_padding(&mut self, maximum: u8) -> io::Result<()> {
        if maximum > 15 || self.remaining_plaintext() > u64::from(maximum) {
            self.failed = true;
            return Err(invalid());
        }
        let mut tail = Zeroizing::new([0u8; 15]);
        while self.remaining_plaintext() > 0 {
            let count = self.read(tail.as_mut())?;
            if count == 0 {
                self.failed = true;
                return Err(invalid());
            }
        }
        self.finish()
    }
    fn fill(&mut self) -> io::Result<()> {
        let mut cipher = Zeroizing::new([0u8; 16]);
        let mut count = 0;
        while count < 16 {
            check(self.control).map_err(io::Error::other)?;
            match self.source.read(&mut cipher[count..]) {
                Err(error)
                    if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none() =>
                {
                    continue;
                }
                Err(error) => return Err(stable_error(error)),
                Ok(0) => return Err(invalid()),
                Ok(n) => {
                    count += n;
                    self.consumed += n as u64;
                    self.left -= n as u64;
                }
            }
        }
        self.block.copy_from_slice(cipher.as_ref());
        let block: &mut aes::cipher::Block<Aes256> = self
            .block
            .as_mut_slice()
            .try_into()
            .expect("AES block size");
        self.aes.decrypt_block(block);
        for (plain, &previous) in self.block.iter_mut().zip(&self.chain) {
            *plain ^= previous;
        }
        self.chain.copy_from_slice(cipher.as_ref());
        self.at = 0;
        Ok(())
    }
}
impl<R: Read> Read for CbcReader<'_, R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.failed {
            return Err(invalid());
        }
        if output.is_empty() {
            return Ok(0);
        }
        if self.finished {
            return Ok(0);
        }
        if let Err(error) = check(self.control) {
            self.failed = true;
            return Err(io::Error::other(error));
        }
        // One block per call prevents a later error from hiding already copied
        // plaintext. std read_to_end may call again, preserving the typed fault.
        if self.at == 16 {
            if self.left == 0 {
                return Ok(0);
            }
            if let Err(error) = self.fill() {
                self.failed = true;
                self.block.zeroize();
                return Err(error);
            }
        }
        let count = output.len().min(16 - self.at);
        output[..count].copy_from_slice(&self.block[self.at..self.at + count]);
        self.at += count;
        self.decoded += count as u64;
        Ok(count)
    }
}
impl<R> Drop for CbcReader<'_, R> {
    fn drop(&mut self) {
        self.chain.zeroize();
        self.block.zeroize();
    }
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, DecodeError::Integrity)
}
