use super::*;
use std::io::{Cursor, Write};
use xdzip_codec_api::{MetadataCodec, ScanLimits};
use zip::{CompressionMethod, write::SimpleFileOptions};

const PASSWORD: &str = "fixture 密码";

#[derive(Default)]
struct Memory(Vec<u8>);
impl DecodedSink for Memory {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

fn options(method: CompressionMethod) -> SimpleFileOptions {
    SimpleFileOptions::default()
        .compression_method(method)
        // Deterministic salts belong only to synthetic tests, never production.
        .with_aes_encryption_and_salt(PASSWORD.as_bytes(), zip::AesSalt::Aes256([42; 16]))
}
fn archive(data: &[u8], method: CompressionMethod, streaming: bool) -> Vec<u8> {
    if streaming {
        let mut zip = zip::ZipWriter::new_stream(Vec::new());
        zip.start_file("content.txt", options(method)).unwrap();
        zip.write_all(data).unwrap();
        zip.finish().unwrap().into_inner()
    } else {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        zip.start_file("content.txt", options(method)).unwrap();
        zip.write_all(data).unwrap();
        zip.finish().unwrap().into_inner()
    }
}
fn rows(bytes: &[u8]) -> Vec<Entry> {
    ZipCodec
        .scan(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap()
}
fn decode(bytes: &[u8], password: Option<&str>) -> Result<Vec<u8>, DecodeError> {
    let control = Cancellation::default();
    let rows = rows(bytes);
    let mut source = Cursor::new(bytes);
    let mut decoder = ZipCodec.open_payload(&mut source, &rows, &control)?;
    let mut output = Memory::default();
    decoder.decode_with_password(&rows[0], &mut output, &control, password)?;
    Ok(output.0)
}
fn central(bytes: &[u8]) -> usize {
    bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap()
}
fn data_range(bytes: &[u8]) -> (usize, usize) {
    let start = 30
        + u16::from_le_bytes(bytes[26..28].try_into().unwrap()) as usize
        + u16::from_le_bytes(bytes[28..30].try_into().unwrap()) as usize;
    let packed = rows(bytes)[0].packed_size as usize;
    (start, start + packed)
}
fn aes_at(bytes: &[u8], central: bool) -> usize {
    let (start, length) = if central {
        let at = self::central(bytes);
        let name = u16::from_le_bytes(bytes[at + 28..at + 30].try_into().unwrap()) as usize;
        let length = u16::from_le_bytes(bytes[at + 30..at + 32].try_into().unwrap()) as usize;
        (at + 46 + name, length)
    } else {
        let name = u16::from_le_bytes(bytes[26..28].try_into().unwrap()) as usize;
        let length = u16::from_le_bytes(bytes[28..30].try_into().unwrap()) as usize;
        (30 + name, length)
    };
    let mut at = start;
    while at < start + length {
        let id = u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap());
        let length = u16::from_le_bytes(bytes[at + 2..at + 4].try_into().unwrap()) as usize;
        if id == 0x9901 {
            return at;
        }
        at += 4 + length;
    }
    panic!("fixture AES metadata missing");
}

#[test]
fn aes256_store_deflate_ae1_ae2_empty_and_streaming_descriptors_roundtrip() {
    for method in [CompressionMethod::Stored, CompressionMethod::Deflated] {
        for streaming in [false, true] {
            for data in [
                Vec::new(),
                b"tiny".to_vec(),
                "actual 密码解压\n".repeat(100).into_bytes(),
            ] {
                let bytes = archive(&data, method, streaming);
                let entry = &rows(&bytes)[0];
                assert!(entry.encrypted && entry.password_supported);
                assert_eq!(entry.method, 99);
                assert_eq!(decode(&bytes, Some(PASSWORD)).unwrap(), data);
                assert_eq!(decode(&bytes, None), Err(DecodeError::PasswordRequired));
                assert_eq!(
                    decode(&bytes, Some("incorrect")),
                    Err(DecodeError::WrongPassword)
                );
            }
        }
    }
}

#[test]
fn aes_with_forced_local_zip64_descriptor_keeps_original_width_checks() {
    let mut archive = zip::ZipWriter::new_stream(Vec::new());
    archive
        .start_file(
            "content.txt",
            options(CompressionMethod::Deflated).large_file(true),
        )
        .unwrap();
    archive.write_all(b"zip64 descriptor AES").unwrap();
    let bytes = archive.finish().unwrap().into_inner();
    let central = central(&bytes);
    assert_eq!(&bytes[central + 20..central + 24], &[255; 4]);
    assert_eq!(
        decode(&bytes, Some(PASSWORD)).unwrap(),
        b"zip64 descriptor AES"
    );
    let mut bad = bytes.clone();
    let (_, end) = data_range(&bad);
    bad[end + 12] ^= 1;
    assert_eq!(decode(&bad, Some(PASSWORD)), Err(DecodeError::Integrity));
}

#[test]
fn authentication_corruption_including_empty_records_never_succeeds() {
    for data in [Vec::new(), vec![7; BLOCK * 3]] {
        for method in [CompressionMethod::Stored, CompressionMethod::Deflated] {
            let mut bytes = archive(&data, method, false);
            let (_, end) = data_range(&bytes);
            bytes[end - 1] ^= 1;
            assert_eq!(decode(&bytes, Some(PASSWORD)), Err(DecodeError::Integrity));
        }
    }
}

#[test]
fn ae1_crc_and_local_central_aes_profile_must_agree() {
    let valid = archive(&[3; 100], CompressionMethod::Stored, false);
    let central = central(&valid);
    assert_eq!(valid[aes_at(&valid, false) + 4], 1);
    for at in [
        aes_at(&valid, false) + 4,
        aes_at(&valid, false) + 9,
        6,
        8,
        14,
    ] {
        let mut bad = valid.clone();
        bad[at] ^= 1;
        assert_eq!(
            decode(&bad, Some(PASSWORD)),
            Err(DecodeError::Integrity),
            "offset {at}"
        );
    }
    let mut bad_crc = valid.clone();
    bad_crc[14] ^= 1;
    bad_crc[central + 16] ^= 1;
    assert_eq!(
        decode(&bad_crc, Some(PASSWORD)),
        Err(DecodeError::Integrity)
    );
}

#[test]
fn ae2_must_store_zero_crc_and_cannot_hide_bad_original_metadata() {
    let mut bytes = archive(b"tiny", CompressionMethod::Stored, false);
    assert_eq!(bytes[aes_at(&bytes, false) + 4], 2);
    let central = central(&bytes);
    bytes[14..18].copy_from_slice(&0x12345678u32.to_le_bytes());
    bytes[central + 16..central + 20].copy_from_slice(&0x12345678u32.to_le_bytes());
    assert!(!rows(&bytes)[0].password_supported);
    assert_eq!(decode(&bytes, Some(PASSWORD)), Err(DecodeError::Integrity));
}

#[test]
fn aes256_is_the_only_password_profile_and_descriptors_are_not_skipped() {
    let valid = archive(b"content", CompressionMethod::Stored, true);
    for strength in [1, 2] {
        let mut bytes = valid.clone();
        for is_central in [false, true] {
            let aes = aes_at(&bytes, is_central);
            bytes[aes + 8] = strength;
        }
        assert!(!rows(&bytes)[0].password_supported);
        assert_eq!(
            decode(&bytes, Some(PASSWORD)),
            Err(DecodeError::Unsupported)
        );
    }
    let mut bad = valid.clone();
    let (_, end) = data_range(&bad);
    assert_eq!(&bad[end..end + 4], b"PK\x07\x08");
    bad[end + 4] ^= 1;
    assert_eq!(decode(&bad, Some(PASSWORD)), Err(DecodeError::Integrity));
}

#[test]
fn frozen_password_admission_and_declared_size_cannot_change() {
    let bytes = archive(b"content", CompressionMethod::Stored, false);
    let mut changed = rows(&bytes);
    changed[0].password_supported = false;
    let control = Cancellation::default();
    let mut input = Cursor::new(&bytes);
    assert!(matches!(
        ZipCodec.open_payload(&mut input, &changed, &control),
        Err(DecodeError::Integrity)
    ));
    let mut bad = bytes.clone();
    let central = central(&bad);
    bad[22..26].copy_from_slice(&8u32.to_le_bytes());
    bad[central + 24..central + 28].copy_from_slice(&8u32.to_le_bytes());
    assert_eq!(decode(&bad, Some(PASSWORD)), Err(DecodeError::Integrity));
}

#[test]
fn authenticated_deflate_requires_stream_end_and_no_trailing_compressed_bytes() {
    let expected = b"strict deflate content".repeat(100);
    let mut compressor =
        flate2::write::DeflateEncoder::new(Vec::new(), flate2::Compression::default());
    compressor.write_all(&expected).unwrap();
    let compressed = compressor.finish().unwrap();
    for broken in [
        compressed[..compressed.len() - 1].to_vec(),
        [compressed.as_slice(), b"trailing"].concat(),
    ] {
        // Encrypt intentionally malformed compressed bytes using zip-rs Store.
        // Changing only non-authenticated metadata preserves a valid AES HMAC.
        let mut bytes = archive(&broken, CompressionMethod::Stored, false);
        let central = central(&bytes);
        for is_central in [false, true] {
            let aes = aes_at(&bytes, is_central);
            bytes[aes + 9..aes + 11].copy_from_slice(&8u16.to_le_bytes());
        }
        bytes[22..26].copy_from_slice(&(expected.len() as u32).to_le_bytes());
        bytes[central + 24..central + 28].copy_from_slice(&(expected.len() as u32).to_le_bytes());
        let crc = rawzip::crc32(&expected).to_le_bytes();
        bytes[14..18].copy_from_slice(&crc);
        bytes[central + 16..central + 20].copy_from_slice(&crc);
        assert_eq!(decode(&bytes, Some(PASSWORD)), Err(DecodeError::Integrity));
    }
}

#[test]
fn mixed_plain_and_password_records_preserve_physical_identity() {
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    archive
        .start_file("plain.txt", SimpleFileOptions::default())
        .unwrap();
    archive.write_all(b"plain").unwrap();
    archive
        .start_file("encrypted.txt", options(CompressionMethod::Stored))
        .unwrap();
    archive.write_all(b"encrypted").unwrap();
    let bytes = archive.finish().unwrap().into_inner();
    let rows = rows(&bytes);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let mut payload = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
    for (index, expected) in [b"plain".as_slice(), b"encrypted"].into_iter().enumerate() {
        let mut output = Memory::default();
        payload
            .decode_with_password(&rows[index], &mut output, &control, Some(PASSWORD))
            .unwrap();
        assert_eq!(output.0, expected);
    }
}

#[test]
fn duplicate_names_decode_their_distinct_physical_ciphertext_records() {
    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    archive
        .start_file("one.txt", options(CompressionMethod::Stored))
        .unwrap();
    archive.write_all(b"first payload").unwrap();
    archive
        .start_file("two.txt", options(CompressionMethod::Deflated))
        .unwrap();
    archive.write_all(b"second payload").unwrap();
    let mut bytes = archive.finish().unwrap().into_inner();
    let names = bytes
        .windows(7)
        .enumerate()
        .filter_map(|(at, name)| (name == b"two.txt").then_some(at))
        .collect::<Vec<_>>();
    assert_eq!(names.len(), 2);
    for at in names {
        bytes[at..at + 7].copy_from_slice(b"one.txt");
    }
    let rows = rows(&bytes);
    assert_eq!(rows[0].raw_name, rows[1].raw_name);
    assert_ne!(rows[0].id, rows[1].id);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let mut payload = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
    for (index, expected) in [b"first payload".as_slice(), b"second payload"]
        .into_iter()
        .enumerate()
    {
        let mut output = Memory::default();
        payload
            .decode_with_password(&rows[index], &mut output, &control, Some(PASSWORD))
            .unwrap();
        assert_eq!(output.0, expected);
    }
}

#[test]
fn encrypted_ciphertext_source_failures_preserve_input_budget_and_io_errors() {
    struct Failing {
        cursor: Cursor<Vec<u8>>,
        failure: Rc<Cell<Option<ScanError>>>,
        start: u64,
        end: u64,
    }
    impl Read for Failing {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let position = self.cursor.position();
            if let Some(failure) = self.failure.get()
                && position >= self.start
                && position < self.end
            {
                return Err(io::Error::other(failure));
            }
            self.cursor.read(output)
        }
    }
    impl Seek for Failing {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.cursor.seek(position)
        }
    }
    for (cause, expected) in [
        (ScanError::Limit, DecodeError::Limit),
        (ScanError::Io, DecodeError::SourceIo),
    ] {
        let bytes = archive(&vec![8; BLOCK * 3], CompressionMethod::Stored, false);
        let (start, end) = data_range(&bytes);
        let rows = rows(&bytes);
        let failure = Rc::new(Cell::new(None));
        let mut source = Failing {
            cursor: Cursor::new(bytes),
            failure: Rc::clone(&failure),
            start: start as u64,
            end: end as u64,
        };
        let control = Cancellation::default();
        let mut payload = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
        failure.set(Some(cause));
        assert_eq!(
            payload.decode_with_password(
                &rows[0],
                &mut Memory::default(),
                &control,
                Some(PASSWORD)
            ),
            Err(expected)
        );
    }
}

#[test]
fn output_limit_and_cooperative_cancel_still_stop_encrypted_streams() {
    struct Stop<'a>(&'a Cancellation, bool);
    impl DecodedSink for Stop<'_> {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            if self.1 {
                Err(DecodeError::Limit)
            } else {
                self.0.cancel();
                Ok(())
            }
        }
    }
    for limit in [true, false] {
        let bytes = archive(&vec![8; BLOCK * 3], CompressionMethod::Stored, false);
        let rows = rows(&bytes);
        let control = Cancellation::default();
        let mut source = Cursor::new(bytes);
        let mut payload = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
        assert_eq!(
            payload.decode_with_password(
                &rows[0],
                &mut Stop(&control, limit),
                &control,
                Some(PASSWORD)
            ),
            Err(if limit {
                DecodeError::Limit
            } else {
                DecodeError::Cancelled
            })
        );
    }
}
