//! Unchanged pinned public data through the public Native codec API only.
//! These Unicode names do not exercise uncorrected high-bit narrow-byte runs.

use crate::{NATIVE_MEMORY_LIMIT, NativeCodec};
use std::io::Cursor;
use std::path::PathBuf;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
use xdzip_codec_api::{Cancellation, Format, MetadataCodec, ScanError, ScanLimits};
use zeroize::Zeroizing;

const CHUNK: usize = 64 * 1024;

fn fixed_data(variable: &str, size: usize) -> Vec<u8> {
    let path = PathBuf::from(std::env::var_os(variable).expect("prepared public fixture path"));
    assert_eq!(std::fs::symlink_metadata(&path).unwrap().len(), size as u64);
    assert!(std::fs::symlink_metadata(&path).unwrap().is_file());
    let data = std::fs::read(path).unwrap();
    assert_eq!(data.len(), size);
    data
}

#[derive(Default)]
struct BoundedSink {
    bytes: Vec<u8>,
    expected: usize,
    work: u64,
    auxiliary: u64,
    prepaid: u64,
    credit: u64,
}

impl DecodedSink for BoundedSink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        assert!(bytes.len() <= CHUNK);
        if self.bytes.len() + bytes.len() > self.expected {
            return Err(DecodeError::Limit);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        assert!((1..=CHUNK as u64).contains(&bytes));
        self.work = self.work.checked_add(bytes).ok_or(DecodeError::Limit)?;
        if self.work > 1024 * 1024 {
            return Err(DecodeError::Limit);
        }
        self.credit += bytes;
        Ok(())
    }

    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        assert!((1..=CHUNK as u64).contains(&bytes));
        self.auxiliary = self
            .auxiliary
            .checked_add(bytes)
            .ok_or(DecodeError::Limit)?;
        if self.auxiliary > 8 * 1024 * 1024 {
            return Err(DecodeError::Limit);
        }
        Ok(())
    }

    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.credit = self
            .credit
            .checked_sub(bytes.len() as u64)
            .ok_or(DecodeError::Limit)?;
        self.prepaid += bytes.len() as u64;
        self.write_chunk(bytes)
    }
}

fn codec() -> NativeCodec {
    assert_eq!(NATIVE_MEMORY_LIMIT, 1024 * 1024 * 1024);
    NativeCodec {
        format: Format::Rar,
    }
}

fn limits() -> ScanLimits {
    ScanLimits {
        entries: 4,
        read_bytes: 1024 * 1024,
        ..Default::default()
    }
}

#[test]
#[ignore = "requires built Rust worker and prepared fixed public Unicode fixture"]
fn rust_rar_public_unicode_names_and_full_original_store_bytes() {
    let mut source = Cursor::new(fixed_data("XDZIP_RAR_PUBLIC_UNICODE_FIXTURE", 236));
    let codec = codec();
    let control = Cancellation::default();
    let rows = codec.scan(&mut source, &control, limits()).unwrap();
    assert_eq!(rows.len(), 2);
    let expected: [(&str, &[u8]); 2] = [
        (
            "\u{30a6}\u{30cb}\u{30b3}\u{30c9}.txt",
            "\u{3053}\u{306e}\u{30d5}\u{30a1}\u{30a4}\u{30eb}\u{306b}\u{306f}Unicode\u{30c6}\u{30ad}\u{30b9}\u{30c8}\u{304c}\u{542b}\u{307e}\u{308c}\u{3066}\u{3044}\u{307e}\u{3059}".as_bytes(),
        ),
        (
            "\u{65b0}\u{5efa}\u{6587}\u{672c}\u{6587}\u{6863}.txt",
            b"aaaaaaaaaa",
        ),
    ];
    for (row, (name, bytes)) in rows.iter().zip(expected) {
        assert_eq!(row.name_bytes(), name.as_bytes());
        assert!(row.size_known && row.payload_supported());
        assert!(!row.directory && !row.encrypted && !row.warning);
        assert_eq!(row.method, 29);
        assert_eq!(row.size, bytes.len() as u64);
    }
    let mut reader = codec.open_payload(&mut source, &rows, &control).unwrap();
    // Reuse one reader in reverse and repeated order as well as normal order.
    let mut outputs = 0;
    let mut delivered = 0;
    for index in [1, 0, 0, 1] {
        let mut sink = BoundedSink {
            expected: expected[index].1.len(),
            ..Default::default()
        };
        reader.decode(&rows[index], &mut sink, &control).unwrap();
        assert_eq!(sink.bytes, expected[index].1);
        assert_eq!(
            (sink.work, sink.auxiliary, sink.prepaid, sink.credit),
            (0, 0, 0, 0)
        );
        outputs += 1;
        delivered += sink.bytes.len();
    }
    assert_eq!((outputs, delivered), (4, 154));
}

#[test]
#[ignore = "requires built Rust worker and prepared public RAR3 nonASCII password data/recipe"]
fn rust_rar_public_nonascii_password_opens_headers_and_full_original_payload() {
    let mut source = Cursor::new(fixed_data("XDZIP_RAR_PUBLIC_PASSWORD_FIXTURE", 148));
    let recipe = Zeroizing::new(
        String::from_utf8(fixed_data("XDZIP_RAR_PUBLIC_PASSWORD_RECIPE", 2799)).unwrap(),
    );
    // The pinned upstream file is an official-author recipe docstring only.
    // Borrow its public test value in-process, never a CLI argument or log.
    let password = recipe
        .lines()
        .find_map(|line| {
            line.strip_prefix("Password: \"")
                .and_then(|s| s.strip_suffix('"'))
        })
        .expect("fixed public test recipe contains its test value");
    assert!(!password.is_ascii());
    assert!(password.chars().all(|ch| u32::from(ch) <= 0xffff));
    let codec = codec();
    let control = Cancellation::default();
    assert_eq!(
        codec.scan(&mut source, &control, limits()),
        Err(ScanError::PasswordRequired)
    );
    let rows = codec
        .scan_with_password(&mut source, &control, limits(), Some(password))
        .unwrap();
    assert_eq!(rows.len(), 1);
    let row = &rows[0];
    assert_eq!(row.name_bytes(), b"file1.txt");
    assert_eq!((row.method, row.size_known, row.size), (29, true, 17));
    assert!(row.encrypted && row.password_supported && row.payload_supported());
    assert!(!row.directory && !row.warning);
    let mut reader = codec
        .open_payload_with_password(&mut source, &rows, &control, Some(password))
        .unwrap();
    let mut delivered = 0;
    for _ in 0..2 {
        let mut sink = BoundedSink {
            expected: 17,
            ..Default::default()
        };
        reader
            .decode_with_password(row, &mut sink, &control, Some(password))
            .unwrap();
        assert_eq!(sink.bytes, b"nonascii-payload\n");
        assert_eq!((sink.work, sink.prepaid, sink.credit), (17, 17, 0));
        delivered += sink.bytes.len();
    }
    assert_eq!(delivered, 34);
}
