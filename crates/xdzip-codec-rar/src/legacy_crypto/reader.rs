use super::{DecodeError, LegacyCipher, check, stable_error};
use std::io::{self, Read};
use xdzip_codec_api::Cancellation;
use zeroize::{Zeroize, Zeroizing};

/// One logical historical file/header stream. Its source may join admitted
/// physical pieces, while byte state or block feedback continues unchanged.
pub(crate) struct LegacyReader<'a, R> {
    source: R,
    cipher: LegacyCipher,
    block: Zeroizing<[u8; 16]>,
    at: usize,
    available: usize,
    left: u64,
    consumed: u64,
    decoded: u64,
    failed: bool,
    finished: bool,
    control: &'a Cancellation,
}
impl<'a, R: Read> LegacyReader<'a, R> {
    pub fn new(
        source: R,
        packed: u64,
        cipher: LegacyCipher,
        control: &'a Cancellation,
    ) -> io::Result<Self> {
        check(control).map_err(io::Error::other)?;
        if !packed.is_multiple_of(cipher.width() as u64) {
            return Err(invalid());
        }
        Ok(Self {
            source,
            cipher,
            block: Zeroizing::new([0; 16]),
            at: 0,
            available: 0,
            left: packed,
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
        self.left + (self.available - self.at) as u64
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
    pub fn finish_padding(&mut self, maximum: u8) -> io::Result<()> {
        if maximum > 15
            || self.remaining_plaintext() > u64::from(maximum)
            || (self.cipher.width() == 1 && self.remaining_plaintext() != 0)
        {
            self.failed = true;
            return Err(invalid());
        }
        let mut tail = Zeroizing::new([0; 15]);
        while self.remaining_plaintext() > 0 {
            if self.read(tail.as_mut())? == 0 {
                self.failed = true;
                return Err(invalid());
            }
        }
        self.finish()
    }
    fn fill(&mut self) -> io::Result<()> {
        let width = self.cipher.width();
        let mut count = 0;
        self.block.zeroize();
        while count < width {
            check(self.control).map_err(io::Error::other)?;
            match self.source.read(&mut self.block[count..width]) {
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
        self.cipher.apply(&mut self.block[..width]);
        self.at = 0;
        self.available = width;
        Ok(())
    }
}
impl<R: Read> Read for LegacyReader<'_, R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.failed {
            return Err(invalid());
        }
        if output.is_empty() || self.finished {
            return Ok(0);
        }
        if let Err(error) = check(self.control) {
            self.failed = true;
            return Err(io::Error::other(error));
        }
        if self.cipher.width() == 1 {
            if self.left == 0 {
                return Ok(0);
            }
            let length = output.len().min(8192).min(self.left.min(8192) as usize);
            let count = loop {
                check(self.control).map_err(|error| {
                    self.failed = true;
                    io::Error::other(error)
                })?;
                match self.source.read(&mut output[..length]) {
                    Err(error)
                        if error.kind() == io::ErrorKind::Interrupted
                            && error.get_ref().is_none() =>
                    {
                        continue;
                    }
                    Err(error) => {
                        self.failed = true;
                        return Err(stable_error(error));
                    }
                    Ok(0) => {
                        self.failed = true;
                        return Err(invalid());
                    }
                    Ok(count) => break count,
                }
            };
            for byte in &mut output[..count] {
                self.cipher.apply(std::slice::from_mut(byte));
            }
            self.left -= count as u64;
            self.consumed += count as u64;
            self.decoded += count as u64;
            return Ok(count);
        }
        if self.at == self.available {
            if self.left == 0 {
                return Ok(0);
            }
            if let Err(error) = self.fill() {
                self.failed = true;
                self.block.zeroize();
                return Err(error);
            }
        }
        let count = output.len().min(self.available - self.at);
        output[..count].copy_from_slice(&self.block[self.at..self.at + count]);
        self.at += count;
        self.decoded += count as u64;
        Ok(count)
    }
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, DecodeError::Integrity)
}
