//! Independent 7z AES profile over admitted generic cryptographic primitives.
//! Container graphs own cipher ranges, output sizes, CRCs and publication.

use aes::Aes256;
use aes::cipher::{BlockCipherDecrypt, BlockCipherEncrypt, KeyInit};
use std::io::{self, Read, Write};
use xdzip_codec_api::Cancellation;
use xdzip_codec_api::payload::DecodeError;
use zeroize::{Zeroize, Zeroizing};

const PASSWORD_BYTES: usize = 4096;
const IO_CHUNK: usize = 8192;

/// Parsed parameters of the 7z AES coder, independent of ZIP encryption.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AesProperties {
    cycles: u8,
    salt: Vec<u8>,
    iv: [u8; 16],
}

/// Explicit cumulative CPU-work admission shared by a caller's coder nodes.
pub struct KdfBudget {
    rounds: u64,
    bytes: u64,
}
impl KdfBudget {
    /// Create an explicit work allowance; no output quota grants free hashing.
    pub fn new(rounds: u64, bytes: u64) -> Self {
        Self { rounds, bytes }
    }
    /// Remaining admitted round and input-byte allowances.
    pub fn remaining(&self) -> (u64, u64) {
        (self.rounds, self.bytes)
    }
    fn admit(&mut self, rounds: u64, bytes: u64) -> Result<(), DecodeError> {
        let next_rounds = self.rounds.checked_sub(rounds).ok_or(DecodeError::Limit)?;
        let next_bytes = self.bytes.checked_sub(bytes).ok_or(DecodeError::Limit)?;
        self.rounds = next_rounds;
        self.bytes = next_bytes;
        Ok(())
    }
}
impl AesProperties {
    /// Parse exact short or long properties, applying exponent admission uniformly.
    pub fn parse(bytes: &[u8]) -> Result<Self, DecodeError> {
        let first = bytes.first().copied().unwrap_or(0);
        let cycles = first & 63;
        if cycles > 24 && cycles != 63 {
            return Err(DecodeError::Unsupported);
        }
        let mut result = Self {
            cycles,
            salt: Vec::new(),
            iv: [0; 16],
        };
        if first & 0xc0 == 0 {
            if bytes.len() > 1 {
                return Err(DecodeError::Integrity);
            }
            return Ok(result);
        }
        let second = *bytes.get(1).ok_or(DecodeError::Integrity)?;
        let salt_len = usize::from(second >> 4) + usize::from(first >> 7);
        let iv_len = usize::from(second & 15) + usize::from((first >> 6) & 1);
        if bytes.len() != 2 + salt_len + iv_len {
            return Err(DecodeError::Integrity);
        }
        result.salt.extend_from_slice(&bytes[2..2 + salt_len]);
        result.iv[..iv_len].copy_from_slice(&bytes[2 + salt_len..]);
        Ok(result)
    }
    /// Compatible author profile: exponent 19, no salt, fresh full-length IV.
    pub fn fresh() -> Result<Self, DecodeError> {
        let mut iv = [0; 16];
        getrandom::fill(&mut iv).map_err(|_| DecodeError::SourceIo)?;
        Ok(Self {
            cycles: 19,
            salt: Vec::new(),
            iv,
        })
    }
    /// Serialize equivalent canonical parameters using the full 16-byte IV.
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(18 + self.salt.len());
        let salt_flag = u8::from(!self.salt.is_empty());
        bytes.push(self.cycles | 0x40 | salt_flag << 7);
        bytes.push((self.salt.len().saturating_sub(1) as u8) << 4 | 15);
        bytes.extend_from_slice(&self.salt);
        bytes.extend_from_slice(&self.iv);
        bytes
    }
    /// Derive a task-local key. Owned password/hash/key buffers are erased on drop.
    pub fn derive_key(
        &self,
        password: Option<&str>,
        control: &Cancellation,
        budget: &mut KdfBudget,
    ) -> Result<Zeroizing<[u8; 32]>, DecodeError> {
        check(control)?;
        let password = password.ok_or(DecodeError::PasswordRequired)?;
        if password.len() > PASSWORD_BYTES {
            return Err(DecodeError::Limit);
        }
        let mut encoded = Zeroizing::new(Vec::new());
        encoded
            .try_reserve_exact(password.len() * 2)
            .map_err(|_| DecodeError::Limit)?;
        for unit in password.encode_utf16() {
            encoded.extend_from_slice(&unit.to_le_bytes());
        }
        if self.cycles == 63 {
            budget.admit(1, (self.salt.len() + encoded.len()) as u64)?;
            let mut key = Zeroizing::new([0; 32]);
            for (target, byte) in key.iter_mut().zip(self.salt.iter().chain(encoded.iter())) {
                *target = *byte;
            }
            return Ok(key);
        }
        let rounds = 1u64 << self.cycles;
        let width = (self.salt.len() + encoded.len() + 8) as u64;
        budget.admit(rounds, rounds.checked_mul(width).ok_or(DecodeError::Limit)?)?;
        let mut hash = ErasedHash::new();
        for counter in 0..rounds {
            if counter.is_multiple_of(1024) {
                check(control)?;
            }
            hash.update(&self.salt);
            hash.update(&encoded);
            hash.update(&counter.to_le_bytes());
        }
        check(control)?;
        Ok(hash.finish())
    }
}

// The admitted SHA-256 compression primitive supplies the cryptographic core.
// This small streaming wrapper owns erasable state; it does not retain a
// third-party opaque incremental buffer containing task password bytes.
struct ErasedHash {
    state: [u32; 8],
    buffer: [u8; 64],
    filled: usize,
    bytes: u64,
}
impl ErasedHash {
    fn new() -> Self {
        Self {
            state: [
                0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab,
                0x5be0cd19,
            ],
            buffer: [0; 64],
            filled: 0,
            bytes: 0,
        }
    }
    fn update(&mut self, mut bytes: &[u8]) {
        // The profile bounds this private input by 2^24 * (8192 + 16 + 8).
        self.bytes += bytes.len() as u64;
        while !bytes.is_empty() {
            let count = bytes.len().min(64 - self.filled);
            self.buffer[self.filled..self.filled + count].copy_from_slice(&bytes[..count]);
            self.filled += count;
            bytes = &bytes[count..];
            if self.filled == 64 {
                self.compress();
                self.buffer.zeroize();
                self.filled = 0;
            }
        }
    }
    #[allow(deprecated)] // Existing admitted SHA-2 0.10 uses generic-array 0.14.
    fn compress(&mut self) {
        let block = sha2::digest::generic_array::GenericArray::from_slice(&self.buffer);
        sha2::compress256(&mut self.state, std::slice::from_ref(block));
    }
    fn finish(mut self) -> Zeroizing<[u8; 32]> {
        let bits = self.bytes * 8;
        self.update(&[0x80]);
        let zeros = (120 - self.filled) % 64;
        self.update(&[0; 64][..zeros]);
        self.update(&bits.to_be_bytes());
        let mut result = Zeroizing::new([0; 32]);
        for (part, value) in result.as_chunks_mut::<4>().0.iter_mut().zip(self.state) {
            part.copy_from_slice(&value.to_be_bytes());
        }
        result
    }
}
impl Drop for ErasedHash {
    fn drop(&mut self) {
        self.state.zeroize();
        self.buffer.zeroize();
        self.filled.zeroize();
        self.bytes.zeroize();
    }
}
fn check(control: &Cancellation) -> Result<(), DecodeError> {
    control.check().map_err(|_| DecodeError::Cancelled)
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, DecodeError::Integrity)
}
fn check_io(control: &Cancellation) -> io::Result<()> {
    check(control).map_err(io::Error::other)
}
fn retry(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none()
}
fn stable_error(error: io::Error) -> io::Error {
    if error.kind() != io::ErrorKind::Interrupted || error.get_ref().is_none() {
        return error;
    }
    // std's read_exact/read_to_end/write_all retry Interrupted unconditionally.
    // Preserve typed host failures through those combinators and poisoned state.
    io::Error::other(error.into_inner().expect("typed error was present"))
}

/// Bounded CBC decoder. Padding is truncated by the graph's declared node size.
/// All declared cipher blocks are consumed, including nonzero truncated padding;
/// authentication and plaintext CRCs belong to the surrounding container graph.
pub struct AesReader<'a, R> {
    source: R,
    cipher: Aes256,
    chain: [u8; 16],
    plain: Zeroizing<[u8; 16]>,
    at: usize,
    available: usize,
    cipher_left: u64,
    expected: u64,
    produced: u64,
    consumed: u64,
    failed: bool,
    finished: bool,
    control: &'a Cancellation,
}
impl<'a, R: Read> AesReader<'a, R> {
    /// Admit complete cipher blocks and derive a key before reading the source.
    pub fn new(
        source: R,
        cipher_bytes: u64,
        plain_bytes: u64,
        properties: &AesProperties,
        password: Option<&str>,
        control: &'a Cancellation,
        budget: &mut KdfBudget,
    ) -> Result<Self, DecodeError> {
        let minimum = plain_bytes.checked_add(15).ok_or(DecodeError::Limit)? / 16 * 16;
        if !cipher_bytes.is_multiple_of(16) || cipher_bytes < minimum {
            return Err(DecodeError::Integrity);
        }
        let key = properties.derive_key(password, control, budget)?;
        let cipher = Aes256::new_from_slice(key.as_ref()).map_err(|_| DecodeError::Integrity)?;
        Ok(Self {
            source,
            cipher,
            chain: properties.iv,
            plain: Zeroizing::new([0; 16]),
            at: 0,
            available: 0,
            cipher_left: cipher_bytes,
            expected: plain_bytes,
            produced: 0,
            consumed: 0,
            failed: false,
            finished: false,
            control,
        })
    }
    /// Actual bytes read within the declared cipher range.
    pub fn consumed_bytes(&self) -> u64 {
        self.consumed
    }
    /// Actual plaintext delivered, excluding truncated padding.
    pub fn decoded_bytes(&self) -> u64 {
        self.produced
    }
    fn block(&mut self) -> io::Result<()> {
        let mut filled = 0;
        while filled < 16 {
            check_io(self.control)?;
            let count = match self.source.read(&mut self.plain[filled..]) {
                Err(error) if retry(&error) => continue,
                result => result?,
            };
            if count == 0 || count > 16 - filled {
                return Err(invalid());
            }
            self.consumed += count as u64;
            self.cipher_left -= count as u64;
            filled += count;
        }
        let next = *self.plain;
        let mut block = aes::Block::from(next);
        self.cipher.decrypt_block(&mut block);
        self.plain.copy_from_slice(&block);
        block[..].zeroize();
        for (byte, previous) in self.plain.iter_mut().zip(self.chain) {
            *byte ^= previous;
        }
        self.chain = next;
        self.at = 0;
        self.available = (self.expected - self.produced).min(16) as usize;
        self.plain[self.available..].zeroize();
        Ok(())
    }
    fn complete(&mut self) -> io::Result<()> {
        check_io(self.control)?;
        if self.produced != self.expected {
            return Err(invalid());
        }
        let mut ignored = [0; IO_CHUNK];
        while self.cipher_left > 0 {
            check_io(self.control)?;
            let width = self.cipher_left.min(IO_CHUNK as u64) as usize;
            let count = match self.source.read(&mut ignored[..width]) {
                Err(error) if retry(&error) => continue,
                result => result?,
            };
            if count == 0 || count > width {
                return Err(invalid());
            }
            self.cipher_left -= count as u64;
            self.consumed += count as u64;
        }
        self.plain.zeroize();
        self.finished = true;
        Ok(())
    }
    /// Validate completion without discarding undelivered plaintext.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        let result = self.complete();
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(stable_error)
    }
}
impl<R: Read> Read for AesReader<'_, R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        if self.finished {
            return Ok(0);
        }
        let result = (|| {
            check_io(self.control)?;
            if self.produced == self.expected {
                self.complete()?;
                return Ok(0);
            }
            let width = output
                .len()
                .min((self.expected - self.produced).min(IO_CHUNK as u64) as usize);
            let mut written = 0;
            while written < width {
                if self.at == self.available {
                    self.block()?;
                }
                let count = (width - written).min(self.available - self.at);
                output[written..written + count]
                    .copy_from_slice(&self.plain[self.at..self.at + count]);
                self.plain[self.at..self.at + count].zeroize();
                self.at += count;
                self.produced += count as u64;
                written += count;
            }
            Ok(written)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(stable_error)
    }
}

/// Streaming author with zero padding only for a partial final block.
/// Dropping or failing a writer never finalizes its ciphertext.
pub struct AesWriter<'a, W> {
    sink: Option<W>,
    cipher: Aes256,
    chain: [u8; 16],
    pending: Zeroizing<[u8; 16]>,
    filled: usize,
    input: u64,
    encoded: u64,
    failed: bool,
    control: &'a Cancellation,
}
impl<'a, W: Write> AesWriter<'a, W> {
    /// Derive a task-local key. The caller stores matching node properties/sizes.
    pub fn new(
        sink: W,
        properties: &AesProperties,
        password: Option<&str>,
        control: &'a Cancellation,
        budget: &mut KdfBudget,
    ) -> Result<Self, DecodeError> {
        let key = properties.derive_key(password, control, budget)?;
        let cipher = Aes256::new_from_slice(key.as_ref()).map_err(|_| DecodeError::Integrity)?;
        Ok(Self {
            sink: Some(sink),
            cipher,
            chain: properties.iv,
            pending: Zeroizing::new([0; 16]),
            filled: 0,
            input: 0,
            encoded: 0,
            failed: false,
            control,
        })
    }
    /// Plaintext accepted from the frozen input stream.
    pub fn input_bytes(&self) -> u64 {
        self.input
    }
    /// Actual ciphertext writes accepted by the host-owned output.
    pub fn encoded_bytes(&self) -> u64 {
        self.encoded
    }
    fn emit(&mut self) -> io::Result<()> {
        check_io(self.control)?;
        for (byte, previous) in self.pending.iter_mut().zip(self.chain) {
            *byte ^= previous;
        }
        let mut block = aes::Block::from(*self.pending);
        self.cipher.encrypt_block(&mut block);
        self.chain.copy_from_slice(&block);
        self.pending.zeroize();
        self.filled = 0;
        let mut written = 0;
        while written < 16 {
            check_io(self.control)?;
            let count = match self
                .sink
                .as_mut()
                .ok_or_else(invalid)?
                .write(&block[written..])
            {
                Err(error) if retry(&error) => continue,
                result => result?,
            };
            if count == 0 || count > 16 - written {
                return Err(io::Error::new(io::ErrorKind::WriteZero, "ciphertext_write"));
            }
            self.encoded = self.encoded.checked_add(count as u64).ok_or_else(invalid)?;
            written += count;
        }
        Ok(())
    }
    /// Write the partial final block, then return the output only on success.
    pub fn finish(mut self) -> io::Result<W> {
        if self.failed {
            return Err(invalid());
        }
        check_io(self.control)?;
        if self.filled != 0 {
            self.emit().map_err(stable_error)?;
        }
        self.sink.take().ok_or_else(invalid)
    }
}
impl<W: Write> Write for AesWriter<'_, W> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if self.failed {
            return Err(invalid());
        }
        let result = (|| {
            check_io(self.control)?;
            let input = &input[..input.len().min(IO_CHUNK)];
            let mut copied = 0;
            while copied < input.len() {
                let count = (input.len() - copied).min(16 - self.filled);
                self.pending[self.filled..self.filled + count]
                    .copy_from_slice(&input[copied..copied + count]);
                self.filled += count;
                copied += count;
                self.input = self.input.checked_add(count as u64).ok_or_else(invalid)?;
                if self.filled == 16 {
                    self.emit()?;
                }
            }
            Ok(copied)
        })();
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(stable_error)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        let result =
            check_io(self.control).and_then(|()| self.sink.as_mut().ok_or_else(invalid)?.flush());
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(stable_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};
    use std::io::Cursor;
    use xdzip_codec_api::ScanError;

    fn hex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
            .collect()
    }
    fn budget() -> KdfBudget {
        KdfBudget::new(1 << 24, 1024 * 1024 * 1024)
    }
    // NIST SP 800-38A F.2.5/F.2.6, numerical AES-256-CBC example values.
    // https://nvlpubs.nist.gov/nistpubs/Legacy/SP/nistspecialpublication800-38a.pdf
    fn nist() -> (AesProperties, String, Vec<u8>, Vec<u8>) {
        let key = hex("603deb1015ca71be2b73aef0857d77811f352c073b6108d72d9810a30914dff4");
        let units: Vec<_> = key
            .as_chunks::<2>()
            .0
            .iter()
            .map(|p| u16::from_le_bytes(*p))
            .collect();
        let password = String::from_utf16(&units).unwrap();
        let mut properties = vec![0x7f, 0x0f];
        properties.extend(hex("000102030405060708090a0b0c0d0e0f"));
        let plain = hex(concat!(
            "6bc1bee22e409f96e93d7e117393172a",
            "ae2d8a571e03ac9c9eb76fac45af8e51",
            "30c81c46a35ce411e5fbc1191a0a52ef",
            "f69f2445df4f9b17ad2b417be66c3710"
        ));
        let cipher = hex(concat!(
            "f58c4c04d6e5f1ba779eabfb5f7bfbd6",
            "9cfc4e967edb808d679f777bc6702c7d",
            "39f23369a9d9bacfa530e26304231461",
            "b2eb05e2c39be9fcda6c19078c6a9d1b"
        ));
        (
            AesProperties::parse(&properties).unwrap(),
            password,
            plain,
            cipher,
        )
    }
    #[test]
    fn property_layouts_are_exact_and_exponents_are_admitted_for_every_layout() {
        for input in [
            vec![],
            vec![0],
            vec![0x3f],
            vec![0x80, 0, 0xaa],
            vec![0x40, 0x10, 0xaa, 0xbb],
        ] {
            let p = AesProperties::parse(&input).unwrap();
            assert_eq!(AesProperties::parse(&p.encode()).unwrap(), p);
        }
        let p = AesProperties::parse(&[0x40, 0x10, 0xaa, 0xbb]).unwrap();
        assert_eq!(p.salt, [0xaa]);
        assert_eq!(p.iv[0], 0xbb);
        assert_eq!(p.iv[1..], [0; 15]);
        for input in [
            vec![0, 0],
            vec![0x40],
            vec![0x40, 0],
            vec![0x80, 0],
            vec![0x80, 0, 1, 2],
        ] {
            assert_eq!(AesProperties::parse(&input), Err(DecodeError::Integrity));
        }
        for exponent in 25..63 {
            assert_eq!(
                AesProperties::parse(&[exponent]),
                Err(DecodeError::Unsupported)
            );
            assert_eq!(
                AesProperties::parse(&[0x80 | exponent, 0, 1]),
                Err(DecodeError::Unsupported)
            );
        }
    }
    #[test]
    fn erased_hash_handles_padding_and_split_updates_against_standard_sha256() {
        for size in [0, 1, 55, 56, 63, 64, 65, 119, 120, 127, 128, 1024] {
            let data: Vec<_> = (0..size).map(|i| (i * 37) as u8).collect();
            let mut hash = ErasedHash::new();
            for chunk in data.chunks(3) {
                hash.update(chunk);
            }
            assert_eq!(hash.finish().as_slice(), Sha256::digest(&data).as_slice());
        }
    }
    #[test]
    fn sevenzip_kdf_utf16_counter_and_direct_key_match_math_vectors() {
        let control = Cancellation::default();
        for (properties, password, expected) in [
            (
                vec![0],
                "",
                "af5570f5a1810b7af78caf4bc70a660f0df51e42baf91d4de5b2328de0e83dfc",
            ),
            (
                vec![0],
                "A",
                "3226c70aa6c93bb3f9eef909a0c9a48be286a63b4de9bf6325948b8a7f01687a",
            ),
            (
                vec![0x81, 0x20, 1, 2, 3],
                "päss🔒",
                "9dcc97adf691b6c9138f7824ae7acf64c62cbb2bae50c1cf7ff3ae2ac5d7dd0c",
            ),
        ] {
            let p = AesProperties::parse(&properties).unwrap();
            assert_eq!(
                p.derive_key(Some(password), &control, &mut budget())
                    .unwrap()
                    .as_slice(),
                hex(expected)
            );
        }
        let p = AesProperties::parse(&[0xbf, 0x20, 1, 2, 3]).unwrap();
        let key = p.derive_key(Some("A"), &control, &mut budget()).unwrap();
        assert_eq!(key[..5], [1, 2, 3, 65, 0]);
        assert_eq!(key[5..], [0; 27]);
    }
    #[test]
    fn kdf_budget_is_cumulative_nonrefundable_and_cancel_precedes_work() {
        let control = Cancellation::default();
        let p = AesProperties::parse(&[1]).unwrap();
        let mut b = KdfBudget::new(3, 40);
        p.derive_key(Some("A"), &control, &mut b).unwrap();
        assert_eq!(b.remaining(), (1, 20));
        assert_eq!(
            p.derive_key(Some("A"), &control, &mut b).unwrap_err(),
            DecodeError::Limit
        );
        assert_eq!(b.remaining(), (1, 20));
        assert_eq!(
            p.derive_key(None, &control, &mut b).unwrap_err(),
            DecodeError::PasswordRequired
        );
        control.cancel();
        assert_eq!(
            p.derive_key(Some("A"), &control, &mut b).unwrap_err(),
            DecodeError::Cancelled
        );
        assert_eq!(b.remaining(), (1, 20));
    }
    struct OneByte<R>(R);
    impl<R: Read> Read for OneByte<R> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let n = output.len().min(1);
            self.0.read(&mut output[..n])
        }
    }
    #[test]
    fn independent_nist_cbc_encrypt_decrypt_match_at_byte_boundaries() {
        let (p, password, plain, cipher) = nist();
        let control = Cancellation::default();
        let mut writer =
            AesWriter::new(Vec::new(), &p, Some(&password), &control, &mut budget()).unwrap();
        for byte in plain.chunks(1) {
            writer.write_all(byte).unwrap();
        }
        assert_eq!(writer.input_bytes(), 64);
        assert_eq!(writer.encoded_bytes(), 64);
        assert_eq!(writer.finish().unwrap(), cipher);
        let mut reader = AesReader::new(
            OneByte(Cursor::new(&cipher)),
            64,
            64,
            &p,
            Some(&password),
            &control,
            &mut budget(),
        )
        .unwrap();
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        reader.finish().unwrap();
        assert_eq!(output, plain);
        assert_eq!(reader.consumed_bytes(), 64);
        assert_eq!(reader.decoded_bytes(), 64);
    }
    #[test]
    fn graph_size_truncates_nonzero_padding_but_consumes_complete_cipher_range() {
        let (p, password, plain, cipher) = nist();
        let control = Cancellation::default();
        let mut r = AesReader::new(
            Cursor::new(cipher),
            64,
            17,
            &p,
            Some(&password),
            &control,
            &mut budget(),
        )
        .unwrap();
        let mut output = Vec::new();
        r.read_to_end(&mut output).unwrap();
        assert_eq!(output, plain[..17]);
        assert_eq!(r.consumed_bytes(), 64);
        assert_eq!(r.decoded_bytes(), 17);
        r.finish().unwrap();
    }
    #[test]
    fn incomplete_source_and_early_finish_poison_without_discarding_plaintext() {
        let (p, password, _, cipher) = nist();
        let control = Cancellation::default();
        let mut r = AesReader::new(
            Cursor::new(&cipher[..63]),
            64,
            64,
            &p,
            Some(&password),
            &control,
            &mut budget(),
        )
        .unwrap();
        assert!(r.read_to_end(&mut Vec::new()).is_err());
        assert_eq!(r.consumed_bytes(), 63);
        assert!(r.read(&mut [0; 1]).is_err());
        let mut r = AesReader::new(
            Cursor::new(&cipher),
            64,
            64,
            &p,
            Some(&password),
            &control,
            &mut budget(),
        )
        .unwrap();
        r.read_exact(&mut [0; 1]).unwrap();
        assert!(r.finish().is_err());
        assert_eq!(r.consumed_bytes(), 16);
        assert!(r.read(&mut [0; 1]).is_err());
    }

    #[test]
    fn cancellation_after_last_plaintext_chunk_still_rejects_completion() {
        let (p, password, _, cipher) = nist();
        let control = Cancellation::default();
        let mut r = AesReader::new(
            Cursor::new(&cipher),
            64,
            64,
            &p,
            Some(&password),
            &control,
            &mut budget(),
        )
        .unwrap();
        r.read_exact(&mut [0; 64]).unwrap();
        control.cancel();
        let error = r.finish().unwrap_err();
        assert_eq!(
            error.get_ref().unwrap().downcast_ref::<DecodeError>(),
            Some(&DecodeError::Cancelled)
        );
        assert!(r.read(&mut [0; 1]).is_err());
    }
    #[test]
    fn typed_interrupted_source_errors_survive_without_retry() {
        struct Broken(usize);
        impl Read for Broken {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                self.0 += 1;
                if self.0 == 1 {
                    Err(io::Error::new(
                        io::ErrorKind::Interrupted,
                        ScanError::SourceChanged,
                    ))
                } else {
                    panic!("typed domain errors must not be retried")
                }
            }
        }
        let (p, password, _, _) = nist();
        let control = Cancellation::default();
        let mut r = AesReader::new(
            Broken(0),
            16,
            1,
            &p,
            Some(&password),
            &control,
            &mut budget(),
        )
        .unwrap();
        let error = r.read(&mut [0; 1]).unwrap_err();
        assert_eq!(
            error.get_ref().unwrap().downcast_ref::<ScanError>(),
            Some(&ScanError::SourceChanged)
        );
        assert!(r.read(&mut [0; 1]).is_err());
        let mut r = AesReader::new(
            Broken(0),
            16,
            1,
            &p,
            Some(&password),
            &control,
            &mut budget(),
        )
        .unwrap();
        let error = r.read_to_end(&mut Vec::new()).unwrap_err();
        assert_eq!(
            error.get_ref().unwrap().downcast_ref::<ScanError>(),
            Some(&ScanError::SourceChanged)
        );
    }
    #[test]
    fn author_padding_lengths_and_drop_do_not_finish_an_incomplete_stream() {
        let (p, password, _, _) = nist();
        let control = Cancellation::default();
        for size in [0, 1, 15, 16, 17, 31, 32] {
            let plain = vec![0x73; size];
            let mut writer =
                AesWriter::new(Vec::new(), &p, Some(&password), &control, &mut budget()).unwrap();
            writer.write_all(&plain).unwrap();
            let cipher = writer.finish().unwrap();
            assert_eq!(cipher.len(), size.div_ceil(16) * 16);
            let mut r = AesReader::new(
                Cursor::new(&cipher),
                cipher.len() as u64,
                size as u64,
                &p,
                Some(&password),
                &control,
                &mut budget(),
            )
            .unwrap();
            let mut output = Vec::new();
            r.read_to_end(&mut output).unwrap();
            assert_eq!(output, plain);
        }
        let mut output = Vec::new();
        {
            let mut w =
                AesWriter::new(&mut output, &p, Some(&password), &control, &mut budget()).unwrap();
            w.write_all(&[0x61; 17]).unwrap();
            w.flush().unwrap();
        }
        assert_eq!(output.len(), 16);
    }
    #[test]
    fn partial_output_failure_retains_actual_count_and_cannot_be_replayed() {
        struct Broken {
            count: usize,
        }
        impl Write for Broken {
            fn write(&mut self, input: &[u8]) -> io::Result<usize> {
                if self.count == 5 {
                    return Err(io::Error::other("controlled_output_failure"));
                }
                let n = input.len().min(5 - self.count);
                self.count += n;
                Ok(n)
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let (p, password, _, _) = nist();
        let control = Cancellation::default();
        let mut w = AesWriter::new(
            Broken { count: 0 },
            &p,
            Some(&password),
            &control,
            &mut budget(),
        )
        .unwrap();
        assert!(w.write_all(&[1; 16]).is_err());
        assert_eq!(w.encoded_bytes(), 5);
        assert!(w.write_all(&[1; 16]).is_err());
        assert!(w.finish().is_err());
        struct TypedOutput;
        impl Write for TypedOutput {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::new(
                    io::ErrorKind::Interrupted,
                    DecodeError::Output,
                ))
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut w =
            AesWriter::new(TypedOutput, &p, Some(&password), &control, &mut budget()).unwrap();
        let error = w.write_all(&[1; 16]).unwrap_err();
        assert_eq!(
            error.get_ref().unwrap().downcast_ref::<DecodeError>(),
            Some(&DecodeError::Output)
        );
    }

    #[test]
    #[ignore = "requires separately prepared official 7-Zip encrypted inputs"]
    fn official_copy_aes_nodes_match_owned_known_payloads() {
        use std::io::{Seek, SeekFrom};
        let root = std::path::PathBuf::from(
            std::env::var_os("XDZIP_7Z_AES_FIXTURES").expect("explicit owned fixture directory"),
        );
        let control = Cancellation::default();
        for size in [1usize, 15, 16, 17, 31, 32, 8193] {
            let mut source = std::fs::File::open(root.join(format!("copy-aes-{size}.7z"))).unwrap();
            assert!(source.metadata().unwrap().len() < 1024 * 1024);
            let index = crate::read_index(&mut source, &control, Default::default(), None).unwrap();
            assert_eq!(index.rows.len(), 1);
            assert_eq!(index.streams.folders.len(), 1);
            let folder = &index.streams.folders[0];
            assert_eq!(folder.packed_inputs.len(), 1);
            let mut output_port = 0;
            let mut encrypted = None;
            for coder in &folder.coders {
                assert_eq!((coder.inputs, coder.outputs), (1, 1));
                if coder.method.as_bytes() == [6, 0xf1, 7, 1] {
                    assert!(encrypted.is_none());
                    encrypted = Some((
                        AesProperties::parse(&coder.properties).unwrap(),
                        folder.output_sizes[output_port],
                    ));
                } else {
                    assert_eq!(coder.method.as_bytes(), [0]);
                }
                output_port += coder.outputs;
            }
            let (profile, output_size) = encrypted.expect("official encrypted content coder");
            assert_eq!(output_size, size as u64);
            let range = &index.streams.packs[folder.packed_inputs[0].pack].range;
            source.seek(SeekFrom::Start(range.offset)).unwrap();
            let mut reader = AesReader::new(
                (&mut source).take(range.length),
                range.length,
                output_size,
                &profile,
                Some("xdzip-cbc-profile"),
                &control,
                &mut budget(),
            )
            .unwrap();
            let mut actual = Vec::new();
            reader.read_to_end(&mut actual).unwrap();
            reader.finish().unwrap();
            let expected: Vec<_> = (0..size)
                .map(|i| (i.wrapping_mul(37).wrapping_add(size) & 255) as u8)
                .collect();
            assert_eq!(actual, expected, "official AES node length {size}");
            assert_eq!(reader.consumed_bytes(), range.length);
        }
    }
}
