//! Regenerate the tiny owned password-retry input, never a production archive.
//! Fixed salts and public fixture secrets are exclusively for regression tests.
use std::io::{Cursor, Write};
use xdzip_codec_api::{
    Cancellation, MetadataCodec, ScanLimits,
    payload::{DecodeError, DecodedSink, PayloadCodec},
};
use xdzip_codec_zip::ZipCodec;
use zip::{CompressionMethod, write::SimpleFileOptions};

const PASSWORD: &str = " owned AES 密码 ";
const WRONG: &str = "wrong owned fixture";
const PAYLOAD: &[u8] = b"owned AES payload\n";

#[derive(Default)]
struct Memory(Vec<u8>);
impl DecodedSink for Memory {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        assert!(self.0.len() + bytes.len() <= PAYLOAD.len());
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

fn fixture(salt: u8) -> Vec<u8> {
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .compression_level(Some(6))
        .last_modified_time(zip::DateTime::default())
        .unix_permissions(0o644)
        // Fixed salts are exclusively for this owned synthetic regression input.
        .with_aes_encryption_and_salt(PASSWORD.as_bytes(), zip::AesSalt::Aes256([salt; 16]));
    archive.start_file("owned.txt", options).unwrap();
    archive.write_all(PAYLOAD).unwrap();
    archive.finish().unwrap().into_inner()
}
fn decode(bytes: &[u8], password: Option<&str>) -> Result<Vec<u8>, DecodeError> {
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = ZipCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].raw_name, b"owned.txt");
    assert_eq!(rows[0].size, PAYLOAD.len() as u64);
    assert!(rows[0].encrypted && rows[0].password_supported && !rows[0].directory);
    let mut codec = ZipCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut output = Memory::default();
    codec.decode_with_password(&rows[0], &mut output, &cancel, password)?;
    Ok(output.0)
}
fn main() {
    for salt in 0x42..=u8::MAX {
        let bytes = fixture(salt);
        if decode(&bytes, Some(WRONG)) != Err(DecodeError::WrongPassword) {
            continue;
        }
        assert_eq!(decode(&bytes, None), Err(DecodeError::PasswordRequired));
        assert_eq!(decode(&bytes, Some(PASSWORD)).unwrap(), PAYLOAD);
        let path = std::path::Path::new("tests/fixtures/extract/password-retry.zip");
        std::fs::write(path, &bytes).unwrap();
        let saved = std::fs::read(path).unwrap();
        assert_eq!(saved, bytes);
        assert_eq!(decode(&saved, Some(WRONG)), Err(DecodeError::WrongPassword));
        assert_eq!(decode(&saved, Some(PASSWORD)).unwrap(), PAYLOAD);
        println!(
            "fixture generated and reopened: {length} bytes, AE-2 AES-256 Deflate, salt byte 0x{salt:02x}; required/wrong/correct codec assertions passed",
            length = saved.len()
        );
        return;
    }
    panic!("no fixed fixture salt rejected the wrong password verifier");
}
