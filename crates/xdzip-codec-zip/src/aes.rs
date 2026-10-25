// crates/xdzip-codec-zip/src/aes.rs

//! WinZip AE-1/AE-2 framing from the official AES specification, sections II/III/V.
//! Cryptographic primitives are delegated to RustCrypto; no ZIP parser is delegated.
use crate::record::ExtraFields;
use aes::Aes256;
use aes::cipher::{BlockCipherEncrypt, KeyInit};
use hmac::{Hmac, Mac};
use sha1::Sha1;
use std::io::{self, Read, Write};
use xdzip_codec_api::payload::DecodeError;
use zeroize::{Zeroize, Zeroizing};

type Authentication = Hmac<Sha1>;
const OVERHEAD: u64 = 28;
const CHUNK: usize = 64 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Profile {
    pub version: u16,
    pub method: u16,
}
pub(super) fn profile(fields: ExtraFields<'_>) -> Result<Option<Profile>, DecodeError> {
    let mut found = None;
    for item in fields {
        let (id, bytes) = item.map_err(|_| DecodeError::Integrity)?;
        if id != 0x9901 {
            continue;
        }
        if found.is_some() || bytes.len() != 7 || &bytes[2..4] != b"AE" {
            return Err(DecodeError::Integrity);
        }
        let version = u16::from_le_bytes([bytes[0], bytes[1]]);
        let method = u16::from_le_bytes([bytes[5], bytes[6]]);
        if !matches!(version, 1 | 2) || bytes[4] != 3 || !matches!(method, 0 | 8) {
            return Err(DecodeError::Unsupported);
        }
        found = Some(Profile { version, method });
    }
    Ok(found)
}
pub(super) fn supported(fields: ExtraFields<'_>, crc: u32) -> bool {
    profile(fields).is_ok_and(|value| value.is_some_and(|value| value.version == 1 || crc == 0))
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn derive(password: &str, salt: &[u8; 16]) -> Zeroizing<[u8; 66]> {
    let mut keys = Zeroizing::new([0; 66]);
    pbkdf2::pbkdf2_hmac::<Sha1>(password.as_bytes(), salt, 1000, keys.as_mut());
    keys
}
struct Counter {
    cipher: Aes256,
    next: u128,
    stream: Zeroizing<[u8; 16]>,
    used: usize,
}
impl Counter {
    fn new(key: &[u8]) -> io::Result<Self> {
        Ok(Self {
            cipher: Aes256::new_from_slice(key).map_err(|_| invalid("aes_key_length"))?,
            next: 1,
            stream: Zeroizing::new([0; 16]),
            used: 16,
        })
    }
    fn apply(&mut self, bytes: &mut [u8]) -> io::Result<()> {
        for byte in bytes {
            if self.used == 16 {
                let mut block = aes::Block::from(self.next.to_le_bytes());
                self.cipher.encrypt_block(&mut block);
                self.stream.copy_from_slice(&block);
                block.as_mut_slice().zeroize();
                self.next = self
                    .next
                    .checked_add(1)
                    .ok_or_else(|| invalid("aes_counter_limit"))?;
                self.used = 0;
            }
            *byte ^= self.stream[self.used];
            self.used += 1;
        }
        Ok(())
    }
}
fn authentication(key: &[u8]) -> io::Result<Authentication> {
    <Authentication as hmac::KeyInit>::new_from_slice(key)
        .map_err(|_| invalid("aes_authentication_key"))
}

/// A bounded AES-256 ciphertext range. Authentication is mandatory even when the
/// compressed plaintext is empty or its Deflate end marker was already reached.
pub(super) struct AesReader<R> {
    input: R,
    counter: Counter,
    mac: Option<Authentication>,
    remaining: u64,
    verified: bool,
    failed: bool,
}
impl<R: Read> AesReader<R> {
    pub fn new(mut input: R, packed: u64, password: &str) -> Result<Self, DecodeError> {
        let remaining = packed.checked_sub(OVERHEAD).ok_or(DecodeError::Integrity)?;
        let mut salt = [0; 16];
        let mut verifier = [0; 2];
        input
            .read_exact(&mut salt)
            .map_err(crate::payload::read_error)?;
        input
            .read_exact(&mut verifier)
            .map_err(crate::payload::read_error)?;
        let keys = derive(password, &salt);
        if !constant_time_eq::constant_time_eq(&keys[64..], &verifier) {
            return Err(DecodeError::WrongPassword);
        }
        let counter = Counter::new(&keys[..32]).map_err(crate::payload::read_error)?;
        let mac = authentication(&keys[32..64]).map_err(crate::payload::read_error)?;
        Ok(Self {
            input,
            counter,
            mac: Some(mac),
            remaining,
            verified: false,
            failed: false,
        })
    }
    pub fn finish_auth(&mut self) -> io::Result<()> {
        if self.verified {
            return Ok(());
        }
        if self.failed || self.remaining != 0 {
            self.failed = true;
            return Err(invalid("aes_incomplete_ciphertext"));
        }
        self.failed = true;
        let mut tag = [0; 10];
        self.input.read_exact(&mut tag)?;
        self.mac
            .take()
            .ok_or_else(|| invalid("aes_authentication_state"))?
            .verify_truncated_left(&tag)
            .map_err(|_| invalid("aes_authentication"))?;
        self.verified = true;
        self.failed = false;
        Ok(())
    }
}
impl<R: Read> Read for AesReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid("aes_failed"));
        }
        if self.remaining == 0 {
            self.finish_auth()?;
            return Ok(0);
        }
        self.failed = true;
        let bound = bytes
            .len()
            .min(CHUNK)
            .min(self.remaining.min(usize::MAX as u64) as usize);
        let count = loop {
            match self.input.read(&mut bytes[..bound]) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => break result?,
            }
        };
        if count == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "aes_ciphertext_eof",
            ));
        }
        self.mac
            .as_mut()
            .ok_or_else(|| invalid("aes_authentication_state"))?
            .update(&bytes[..count]);
        self.counter.apply(&mut bytes[..count])?;
        self.remaining -= count as u64;
        self.failed = false;
        Ok(count)
    }
}

/// Writes only the WinZip encrypted-data envelope. ZIP records and compression
/// remain the caller's responsibility. Drop never writes a successful trailer.
pub(crate) struct AesWriter<W> {
    output: W,
    counter: Counter,
    mac: Option<Authentication>,
    buffer: Zeroizing<Vec<u8>>,
    failed: bool,
}
impl<W: Write> AesWriter<W> {
    pub(crate) fn new(output: W, password: &str) -> io::Result<Self> {
        let mut salt = [0; 16];
        getrandom::fill(&mut salt).map_err(|_| io::Error::other("aes_random_source"))?;
        Self::with_salt(output, password, &salt)
    }
    fn with_salt(mut output: W, password: &str, salt: &[u8; 16]) -> io::Result<Self> {
        let keys = derive(password, salt);
        let counter = Counter::new(&keys[..32])?;
        let mac = authentication(&keys[32..64])?;
        output.write_all(salt)?;
        output.write_all(&keys[64..])?;
        Ok(Self {
            output,
            counter,
            mac: Some(mac),
            buffer: Zeroizing::new(vec![0; CHUNK]),
            failed: false,
        })
    }
    pub(crate) fn finish(mut self) -> io::Result<W> {
        if self.failed {
            return Err(invalid("aes_failed"));
        }
        let mac = self
            .mac
            .take()
            .ok_or_else(|| invalid("aes_authentication_state"))?;
        let tag = Zeroizing::new(<[u8; 20]>::from(mac.finalize().into_bytes()));
        self.output.write_all(&tag[..10])?;
        Ok(self.output)
    }
}
impl<W: Write> Write for AesWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failed {
            return Err(invalid("aes_failed"));
        }
        if bytes.is_empty() {
            return Ok(0);
        }
        self.failed = true;
        let count = bytes.len().min(CHUNK);
        self.buffer[..count].copy_from_slice(&bytes[..count]);
        self.counter.apply(&mut self.buffer[..count])?;
        self.output.write_all(&self.buffer[..count])?;
        self.mac
            .as_mut()
            .ok_or_else(|| invalid("aes_authentication_state"))?
            .update(&self.buffer[..count]);
        self.buffer[..count].zeroize();
        self.failed = false;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid("aes_failed"));
        }
        self.failed = true;
        self.output.flush()?;
        self.failed = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn only_unique_complete_supported_aes_profiles_are_admitted() {
        let valid = [0x01, 0x99, 7, 0, 2, 0, b'A', b'E', 3, 8, 0];
        assert_eq!(
            profile(ExtraFields::new(&valid)),
            Ok(Some(Profile {
                version: 2,
                method: 8
            }))
        );
        for index in [4, 6, 8, 9] {
            let mut other = valid;
            other[index] = if index == 8 { 2 } else { 4 };
            assert!(profile(ExtraFields::new(&other)).is_err());
        }
        for length in 1..valid.len() {
            assert_eq!(
                profile(ExtraFields::new(&valid[..length])),
                Err(DecodeError::Integrity)
            );
        }
        let duplicate = [valid.as_slice(), valid.as_slice()].concat();
        assert_eq!(
            profile(ExtraFields::new(&duplicate)),
            Err(DecodeError::Integrity)
        );
    }

    #[test]
    fn ciphertext_envelope_matches_independent_zip_writer_across_chunk_boundaries() {
        let password = "fixture 密码";
        for length in [0, 1, 15, 16, 17, CHUNK + 39] {
            let content = (0..length)
                .map(|index| (index % 251) as u8)
                .collect::<Vec<_>>();
            let mut reference = zip::ZipWriter::new(Cursor::new(Vec::new()));
            reference
                .start_file(
                    "x",
                    zip::write::SimpleFileOptions::default()
                        .compression_method(zip::CompressionMethod::Stored)
                        .with_aes_encryption_and_salt(
                            password.as_bytes(),
                            zip::AesSalt::Aes256([42; 16]),
                        ),
                )
                .unwrap();
            reference.write_all(&content).unwrap();
            let reference = reference.finish().unwrap().into_inner();
            let start = 30
                + u16::from_le_bytes(reference[26..28].try_into().unwrap()) as usize
                + u16::from_le_bytes(reference[28..30].try_into().unwrap()) as usize;
            let expected = &reference[start..start + length + 28];
            for chunk in [1, 15, 17, CHUNK + 3] {
                let mut writer = AesWriter::with_salt(Vec::new(), password, &[42; 16]).unwrap();
                for bytes in content.chunks(chunk) {
                    writer.write_all(bytes).unwrap();
                }
                let actual = writer.finish().unwrap();
                assert_eq!(actual, expected, "length={length} chunk={chunk}");
                let mut reader =
                    AesReader::new(Cursor::new(&actual), actual.len() as u64, password).unwrap();
                let mut decoded = Vec::new();
                let mut buffer = [0; 7];
                loop {
                    let count = reader.read(&mut buffer).unwrap();
                    if count == 0 {
                        break;
                    }
                    decoded.extend_from_slice(&buffer[..count]);
                }
                reader.finish_auth().unwrap();
                assert_eq!(decoded, content);
            }
        }
    }

    #[test]
    fn partial_writer_failure_poisoning_and_empty_record_authentication_are_mandatory() {
        struct Failing {
            remaining: usize,
        }
        impl Write for Failing {
            fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
                if self.remaining == 0 {
                    return Err(io::Error::other("owned_output_failure"));
                }
                let count = bytes.len().min(self.remaining);
                self.remaining -= count;
                Ok(count)
            }
            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }
        let mut writer =
            AesWriter::with_salt(Failing { remaining: 20 }, "password", &[7; 16]).unwrap();
        assert!(writer.write_all(b"longer than remaining output").is_err());
        assert!(writer.write_all(b"retry").is_err());
        assert!(writer.finish().is_err());

        let bytes = AesWriter::with_salt(Vec::new(), "password", &[7; 16])
            .unwrap()
            .finish()
            .unwrap();
        assert_eq!(bytes.len(), 28);
        assert!(matches!(
            AesReader::new(Cursor::new(&bytes), 28, "wrong"),
            Err(DecodeError::WrongPassword)
        ));
        let mut bad = bytes.clone();
        bad[27] ^= 1;
        let mut reader = AesReader::new(Cursor::new(&bad), 28, "password").unwrap();
        assert!(reader.read(&mut [0]).is_err());
        assert!(reader.finish_auth().is_err());
        let mut reader = AesReader::new(Cursor::new(&bytes[..27]), 28, "password").unwrap();
        assert_eq!(
            reader.read(&mut [0]).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
    }
    #[test]
    fn reader_stops_at_the_envelope_boundary_and_cannot_finish_partial_ciphertext() {
        let mut writer = AesWriter::with_salt(Vec::new(), "password", &[9; 16]).unwrap();
        writer.write_all(b"bounded encrypted data").unwrap();
        let encoded = writer.finish().unwrap();
        let mut source =
            Cursor::new([b"prefix".as_slice(), encoded.as_slice(), b"suffix"].concat());
        source.set_position(6);
        {
            let mut reader = AesReader::new(&mut source, encoded.len() as u64, "password").unwrap();
            let mut plain = Vec::new();
            reader.read_to_end(&mut plain).unwrap();
            reader.finish_auth().unwrap();
            reader.finish_auth().unwrap();
            assert_eq!(plain, b"bounded encrypted data");
        }
        assert_eq!(source.position(), 6 + encoded.len() as u64);
        let mut reader =
            AesReader::new(Cursor::new(&encoded), encoded.len() as u64, "password").unwrap();
        assert!(reader.finish_auth().is_err());
        assert!(reader.read(&mut [0; 1]).is_err());
    }
}
