use super::*;
use crate::{
    framing,
    work::{Limits, Usage},
};
use std::io::{Cursor, Read, Seek, SeekFrom};
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/aescrypt")
            .join(name),
    )
    .unwrap()
}
fn password(profile: &str) -> String {
    match profile {
        "unicode" => "owned-資料-🔑".into(),
        "long" => "p".repeat(128) + "資料-🔑",
        _ => "owned-aes-fixture".into(),
    }
}
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    auxiliary: u64,
    calls: usize,
}
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.calls += 1;
        self.bytes.extend(bytes);
        Ok(())
    }
    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.auxiliary += bytes;
        Ok(())
    }
}
fn cases() -> Vec<(String, Vec<u8>, String)> {
    let phrase = b"abracadabra abracadabra\n".repeat(6);
    let mut cases = vec![];
    for version in 0..4 {
        for (label, payload) in [
            ("phrase", phrase.clone()),
            ("empty", vec![]),
            ("tail1", (0..1).collect()),
            ("tail15", (0..15).collect()),
            ("block16", (0..16).collect()),
            ("tail17", (0..17).collect()),
            ("unicode", phrase.clone()),
        ] {
            cases.push((format!("v{version}-{label}.aes"), payload, password(label)));
        }
    }
    for version in [2, 3] {
        for profile in ["long", "extensions"] {
            cases.push((
                format!("v{version}-{profile}.aes"),
                phrase.clone(),
                password(profile),
            ));
        }
    }
    cases.extend([
        ("v3-iter1.aes".into(), (0..17).collect(), password("ascii")),
        (
            "v3-all-bytes.aes".into(),
            (0..=255).collect(),
            password("ascii"),
        ),
        (
            "v3-multi.aes".into(),
            (0usize..131077)
                .map(|at| ((at * 73) ^ (at >> 8)) as u8)
                .collect(),
            password("ascii"),
        ),
    ]);
    cases
}
#[test]
fn independent_reference_inputs_decode_all_versions_boundaries_and_password_encodings() {
    let cancel = Cancellation::default();
    let cases = cases();
    assert_eq!(cases.len(), 35);
    for (name, expected, password) in cases {
        let bytes = fixture(&name);
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, Some(&password), &cancel)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(reader.metadata().name, "content");
        assert_eq!(reader.metadata().size, expected.len() as u64);
        assert_eq!(reader.metadata().packed_size, bytes.len() as u64);
        let mut sink = Sink::default();
        reader
            .decode(&mut sink, &cancel)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(sink.bytes, expected, "{name}");
        assert!(sink.auxiliary > 0);
        if expected.is_empty() {
            assert_eq!(sink.calls, 0);
        }
    }
}
#[test]
fn actual_default_six_hundred_thousand_iteration_profile_is_accepted() {
    let cancel = Cancellation::default();
    let mut source = Cursor::new(fixture("v3-default.aes"));
    let mut reader = Reader::open(&mut source, Some(&password("default")), &cancel).unwrap();
    let mut sink = Sink::default();
    reader.decode(&mut sink, &cancel).unwrap();
    assert_eq!(sink.bytes, b"abracadabra abracadabra\n".repeat(6));
    assert!(reader.work.used.hash >= 600000 * 256);
}
#[test]
fn missing_and_wrong_credentials_never_publish_metadata() {
    let cancel = Cancellation::default();
    for version in 0..4 {
        let bytes = fixture(&format!("v{version}-phrase.aes"));
        assert_eq!(
            Reader::open(&mut Cursor::new(&bytes), None, &cancel).err(),
            Some(DecodeError::PasswordRequired)
        );
        assert_eq!(
            Reader::open(&mut Cursor::new(&bytes), Some("incorrect"), &cancel).err(),
            Some(if version == 0 {
                DecodeError::PasswordOrIntegrity
            } else {
                DecodeError::WrongPassword
            })
        );
    }
}
#[test]
fn exact_input_hash_aes_and_decoded_quotas_include_metadata_and_delivery() {
    let cancel = Cancellation::default();
    let bytes = fixture("v3-tail1.aes");
    assert_eq!(bytes.len(), 155);
    for (limits, result) in [
        (
            Limits {
                input: 171,
                hash: 1536,
                aes: 80,
                decoded: 1,
            },
            Ok(()),
        ),
        (
            Limits {
                input: 170,
                hash: 1536,
                aes: 80,
                decoded: 1,
            },
            Err(DecodeError::Limit),
        ),
        (
            Limits {
                input: 171,
                hash: 1535,
                aes: 80,
                decoded: 1,
            },
            Err(DecodeError::Limit),
        ),
        (
            Limits {
                input: 171,
                hash: 1536,
                aes: 79,
                decoded: 1,
            },
            Err(DecodeError::Limit),
        ),
        (
            Limits {
                input: 171,
                hash: 1536,
                aes: 80,
                decoded: 0,
            },
            Err(DecodeError::Limit),
        ),
    ] {
        let mut source = Cursor::new(&bytes);
        let mut reader =
            Reader::open_with_limits(&mut source, Some(&password("ascii")), &cancel, limits)
                .unwrap();
        assert_eq!(reader.work.used.input, 155);
        assert_eq!(reader.work.used.hash, 1280);
        assert_eq!(reader.work.used.aes, 64);
        assert_eq!(reader.decode(&mut Sink::default(), &cancel), result);
        if result.is_ok() {
            let Usage {
                input,
                hash,
                aes,
                decoded,
            } = reader.work.used;
            assert_eq!((input, hash, aes, decoded), (171, 1536, 80, 1));
        }
        assert_eq!(
            reader.decode(&mut Sink::default(), &cancel),
            Err(DecodeError::Limit)
        );
    }
}
#[test]
fn corrupted_ciphertext_is_authenticated_before_any_padding_decryption() {
    let cancel = Cancellation::default();
    for version in 0..4 {
        let mut bytes = fixture(&format!("v{version}-tail17.aes"));
        let mut work = Work::new(Limits::default());
        let header = framing::parse(&mut Cursor::new(&bytes), &cancel, &mut work).unwrap();
        bytes[header.start as usize] ^= 0x80;
        let mut work = Work::new(Limits::default());
        assert_eq!(
            index::inspect(
                &mut Cursor::new(&bytes),
                Some(&password("ascii")),
                &cancel,
                ScanLimits::default(),
                &mut work
            )
            .err()
            .map(decode_error),
            Some(if version == 0 {
                DecodeError::PasswordOrIntegrity
            } else {
                DecodeError::Integrity
            })
        );
        assert_eq!(work.used.aes, if version == 0 { 0 } else { 48 });
        let end = bytes.len() - 1;
        bytes[end] ^= 0x80;
        assert!(Reader::open(&mut Cursor::new(bytes), Some(&password("ascii")), &cancel).is_err());
    }
    for count in 1..=16usize {
        let mut block = [0x51; 16];
        block[16 - count..].fill(count as u8);
        assert_eq!(index::padding_size(&block, 32), Ok(32 - count as u64));
        block[16 - count] ^= 1;
        assert!(index::padding_size(&block, 32).is_err());
    }
    assert!(index::padding_size(&[0; 16], 16).is_err());
    assert!(index::padding_size(&[17; 16], 16).is_err());
}
#[test]
fn unauthenticated_legacy_modulo_and_extensions_grant_no_additional_integrity() {
    let cancel = Cancellation::default();
    for version in 0..3 {
        let mut bytes = fixture(&format!("v{version}-tail17.aes"));
        let at = if version == 0 { 4 } else { bytes.len() - 33 };
        assert_eq!(bytes[at], 1);
        bytes[at] = 2;
        let mut source = Cursor::new(bytes);
        let mut reader = Reader::open(&mut source, Some(&password("ascii")), &cancel).unwrap();
        assert_eq!(reader.metadata().size, 18);
        let mut sink = Sink::default();
        reader.decode(&mut sink, &cancel).unwrap();
        assert_eq!(&sink.bytes[..17], (0..17).collect::<Vec<u8>>());
        assert_eq!(sink.bytes.len(), 18);
    }
    for version in [2, 3] {
        let bytes = fixture(&format!("v{version}-extensions.aes"));
        let mut source = Cursor::new(bytes);
        let reader = Reader::open(&mut source, Some(&password("ascii")), &cancel).unwrap();
        assert_eq!(reader.metadata().name, "content");
    }
}
#[test]
fn cancellation_output_failures_and_repeated_calls_are_terminal() {
    let cancel = Cancellation::default();
    let bytes = fixture("v3-phrase.aes");
    struct Failing;
    impl DecodedSink for Failing {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Err(DecodeError::Output)
        }
    }
    let mut source = Cursor::new(&bytes);
    let mut reader = Reader::open(&mut source, Some(&password("ascii")), &cancel).unwrap();
    assert_eq!(
        reader.decode(&mut Failing, &cancel),
        Err(DecodeError::Output)
    );
    assert_eq!(
        reader.decode(&mut Sink::default(), &cancel),
        Err(DecodeError::Output)
    );
    drop(reader);
    let mut reader = Reader::open(&mut source, Some(&password("ascii")), &cancel).unwrap();
    let second = Cancellation::default();
    second.cancel();
    assert_eq!(
        reader.decode(&mut Sink::default(), &second),
        Err(DecodeError::Cancelled)
    );
    assert_eq!(
        reader.decode(&mut Sink::default(), &cancel),
        Err(DecodeError::Cancelled)
    );
    drop(reader);
    cancel.cancel();
    assert_eq!(
        Reader::open(&mut source, Some(&password("ascii")), &cancel).err(),
        Some(DecodeError::Cancelled)
    );
}
#[test]
fn short_reads_and_typed_interrupted_failures_are_preserved_without_retry() {
    use std::{cell::Cell, rc::Rc};
    struct Source {
        bytes: Cursor<Vec<u8>>,
        failure: Rc<Cell<Option<ScanError>>>,
        calls: Rc<Cell<usize>>,
    }
    impl Read for Source {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            self.calls.set(self.calls.get() + 1);
            if let Some(error) = self.failure.get() {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, error));
            }
            self.bytes.read(&mut bytes[..1])
        }
    }
    impl Seek for Source {
        fn seek(&mut self, at: SeekFrom) -> std::io::Result<u64> {
            self.bytes.seek(at)
        }
    }
    impl ReadSource for Source {}
    let cancel = Cancellation::default();
    let failure = Rc::new(Cell::new(None));
    let calls = Rc::new(Cell::new(0));
    let mut source = Source {
        bytes: Cursor::new(fixture("v2-phrase.aes")),
        failure: failure.clone(),
        calls: calls.clone(),
    };
    let mut reader = Reader::open(&mut source, Some(&password("ascii")), &cancel).unwrap();
    let mut sink = Sink::default();
    reader.decode(&mut sink, &cancel).unwrap();
    assert_eq!(sink.bytes, b"abracadabra abracadabra\n".repeat(6));
    failure.set(Some(ScanError::Limit));
    let before = calls.get();
    assert_eq!(
        reader.decode(&mut Sink::default(), &cancel),
        Err(DecodeError::Limit)
    );
    assert_eq!(calls.get(), before + 1);
    assert_eq!(
        reader.decode(&mut Sink::default(), &cancel),
        Err(DecodeError::Limit)
    );
    assert_eq!(calls.get(), before + 1);
}
#[test]
fn malformed_headers_extensions_and_iteration_limits_fail_before_kdf_work() {
    let cancel = Cancellation::default();
    let base = fixture("v3-phrase.aes");
    let mut cases = Vec::new();
    for (at, value, expected) in [(3, 4, ScanError::Unsupported), (4, 1, ScanError::Invalid)] {
        let mut bytes = base.clone();
        bytes[at] = value;
        cases.push((bytes, expected));
    }
    for (value, expected) in [(0, ScanError::Invalid), (2_000_001, ScanError::Limit)] {
        let mut bytes = base.clone();
        bytes[7..11].copy_from_slice(&u32::to_be_bytes(value));
        cases.push((bytes, expected));
    }
    let mut bytes = base.clone();
    bytes[5..7].copy_from_slice(&65535u16.to_be_bytes());
    cases.push((bytes, ScanError::Invalid));
    let mut bytes = base[..5].to_vec();
    for _ in 0..257 {
        bytes.extend([0, 1, 0]);
    }
    bytes.extend(&base[5..]);
    cases.push((bytes, ScanError::Limit));
    let mut bytes = fixture("v3-extensions.aes");
    let size = usize::from(u16::from_be_bytes(bytes[5..7].try_into().unwrap()));
    for byte in &mut bytes[7..7 + size] {
        if *byte == 0 {
            *byte = 1;
        }
    }
    cases.push((bytes, ScanError::Invalid));
    let mut bytes = base.clone();
    bytes.pop();
    cases.push((bytes, ScanError::Invalid));
    for (bytes, expected) in cases {
        let mut work = Work::new(Limits::default());
        assert_eq!(
            index::inspect(
                &mut Cursor::new(bytes),
                Some(&password("ascii")),
                &cancel,
                ScanLimits::default(),
                &mut work
            )
            .err(),
            Some(expected)
        );
        assert_eq!(work.used.hash, 0);
    }
    let mut source = Cursor::new(base);
    assert_eq!(
        Reader::open(&mut source, Some(&"p".repeat(4097)), &cancel).err(),
        Some(DecodeError::Limit)
    );
}
#[test]
fn kdf_cancellation_is_observed_after_actual_work_without_refunding_it() {
    use std::sync::Arc;
    for (name, profile) in [("v2-phrase.aes", "ascii"), ("v3-default.aes", "default")] {
        let cancel = Arc::new(Cancellation::default());
        let mut work = Work::new(Limits::default());
        let mut source = Cursor::new(fixture(name));
        let header = framing::parse(&mut source, &cancel, &mut work).unwrap();
        let observed = cancel.clone();
        work.hash_observer = Some(Box::new(move |spent| {
            if spent >= 65536 {
                observed.cancel();
            }
        }));
        assert_eq!(
            crate::keys::unwrap(&header, &password(profile), &cancel, &mut work)
                .err()
                .map(decode_error),
            Some(DecodeError::Cancelled)
        );
        assert!(work.used.hash >= 65536 && work.used.hash < 100000);
        assert!(work.left.hash < Limits::default().hash);
    }
}
#[test]
fn changed_ciphertext_in_second_pass_checks_mac_before_padding_and_poisoning() {
    use std::{cell::RefCell, rc::Rc};
    struct Source {
        bytes: Rc<RefCell<Vec<u8>>>,
        position: u64,
    }
    impl Read for Source {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            let bytes = self.bytes.borrow();
            let count = output
                .len()
                .min(bytes.len().saturating_sub(self.position as usize));
            output[..count]
                .copy_from_slice(&bytes[self.position as usize..self.position as usize + count]);
            self.position += count as u64;
            Ok(count)
        }
    }
    impl Seek for Source {
        fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
            self.position = match position {
                SeekFrom::Start(at) => at,
                SeekFrom::End(0) => self.bytes.borrow().len() as u64,
                _ => return Err(std::io::ErrorKind::InvalidInput.into()),
            };
            Ok(self.position)
        }
    }
    impl ReadSource for Source {}
    let cancel = Cancellation::default();
    let bytes = Rc::new(RefCell::new(fixture("v3-tail1.aes")));
    let mut source = Source {
        bytes: bytes.clone(),
        position: 0,
    };
    let mut reader = Reader::open(&mut source, Some(&password("ascii")), &cancel).unwrap();
    let last = (reader.index.header.start + reader.index.header.cipher - 1) as usize;
    bytes.borrow_mut()[last] ^= 0x80;
    assert_eq!(
        reader.decode(&mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
    assert_eq!(reader.work.used.hash, 1536);
    assert_eq!(
        reader.decode(&mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn opening_secret_is_not_retained_and_auxiliary_failure_precedes_payload_delivery() {
    let cancel = Cancellation::default();
    let bytes = fixture("v3-phrase.aes");
    let mut source = Cursor::new(&bytes);
    let mut secret = password("ascii");
    let mut reader = Reader::open(&mut source, Some(&secret), &cancel).unwrap();
    secret.zeroize();
    assert!(secret.is_empty());
    let mut sink = Sink::default();
    reader.decode(&mut sink, &cancel).unwrap();
    assert_eq!(sink.bytes, b"abracadabra abracadabra\n".repeat(6));
    drop(reader);
    struct Reject;
    impl DecodedSink for Reject {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            panic!("auxiliary rejection must precede content")
        }
        fn charge_auxiliary(&mut self, _: u64) -> Result<(), DecodeError> {
            Err(DecodeError::Limit)
        }
    }
    let mut reader = Reader::open(&mut source, Some(&password("ascii")), &cancel).unwrap();
    let before = reader.work.used.input;
    assert_eq!(reader.decode(&mut Reject, &cancel), Err(DecodeError::Limit));
    assert_eq!(reader.work.used.input, before);
}
