use crate::*;
use std::io::Cursor;
use xdzip_codec_api::payload::DecodedSink;
#[path = "protection_owned_data.rs"]
mod owned_data;
use owned_data::OWNED;

#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend(bytes);
        Ok(())
    }
}

const CASES: [&[u8]; 4] = [
    include_bytes!("../../../tests/fixtures/native-archives/test_read_format_rar5_encrypted.rar"),
    include_bytes!(
        "../../../tests/fixtures/native-archives/test_read_format_rar5_encrypted_filenames.rar"
    ),
    include_bytes!(
        "../../../tests/fixtures/native-archives/test_read_format_rar5_solid_encrypted.rar"
    ),
    include_bytes!(
        "../../../tests/fixtures/native-archives/test_read_format_rar5_solid_encrypted_filenames.rar"
    ),
];
const LEGACY: [&[u8]; 2] = [
    include_bytes!("../../../tests/fixtures/native-archives/test_read_format_rar4_encrypted.rar"),
    include_bytes!(
        "../../../tests/fixtures/native-archives/test_read_format_rar4_encrypted_filenames.rar"
    ),
];

#[test]
fn fixed_rar3_header_and_content_passwords_reach_verified_lz_output() {
    let cancel = Cancellation::default();
    for (case, bytes) in LEGACY.iter().enumerate() {
        let mut source = Cursor::new(*bytes);
        let rows = RarCodec
            .scan_with_password(
                &mut source,
                &cancel,
                ScanLimits::default(),
                Some("password"),
            )
            .unwrap();
        assert_eq!(rows.len(), 4);
        for selected in [3, 0, 2, 1] {
            let row = &rows[selected];
            let password = if case == 0 && row.name_bytes() == b"d.txt" {
                "password2"
            } else {
                "password"
            };
            let mut payload = RarCodec
                .open_payload_with_password(&mut source, &rows, &cancel, Some("password"))
                .unwrap();
            let mut sink = Sink::default();
            payload
                .decode_with_password(row, &mut sink, &cancel, Some(password))
                .unwrap();
            let mut expected = b"This is from ".to_vec();
            expected.extend(row.name_bytes());
            assert_eq!(sink.0, expected);
        }
    }
}

#[test]
fn rar3_native_password_terminator_preserves_prefix_and_missing_secret_stays_distinct() {
    let cancel = Cancellation::default();
    let mut source = Cursor::new(LEGACY[1]);
    assert_eq!(
        RarCodec.scan(&mut source, &cancel, ScanLimits::default()),
        Err(ScanError::PasswordRequired)
    );
    let rows = RarCodec
        .scan_with_password(
            &mut source,
            &cancel,
            ScanLimits::default(),
            Some("password\0ignored🦀"),
        )
        .unwrap();
    let mut payload = RarCodec
        .open_payload_with_password(&mut source, &rows, &cancel, Some("password\0ignored🦀"))
        .unwrap();
    let mut sink = Sink::default();
    payload
        .decode_with_password(&rows[0], &mut sink, &cancel, Some("password\0ignored🦀"))
        .unwrap();
    assert_eq!(sink.0, b"This is from a.txt");
}

#[test]
fn fixed_modern_password_headers_solid_and_random_selection_match_known_content() {
    let cancel = Cancellation::default();
    for (case, bytes) in CASES.iter().enumerate() {
        let mut source = Cursor::new(*bytes);
        let rows = RarCodec
            .scan_with_password(
                &mut source,
                &cancel,
                ScanLimits::default(),
                Some("password"),
            )
            .unwrap();
        assert_eq!(rows.len(), 4);
        for selected in [3, 0, 2, 1] {
            let row = &rows[selected];
            let password = if case == 0 && row.name_bytes() == b"d.txt" {
                "password2"
            } else {
                "password"
            };
            let mut payload = RarCodec
                .open_payload_with_password(&mut source, &rows, &cancel, Some("password"))
                .unwrap();
            let mut sink = Sink::default();
            payload
                .decode_with_password(row, &mut sink, &cancel, Some(password))
                .unwrap();
            let mut expected = b"This is from ".to_vec();
            expected.extend(row.name_bytes());
            assert_eq!(sink.0, expected);
        }
    }
}

#[test]
fn trusted_wrong_or_missing_password_emits_nothing_and_poison_is_terminal() {
    let cancel = Cancellation::default();
    for bytes in [CASES[1], CASES[3]] {
        for (password, expected) in [
            (None, ScanError::PasswordRequired),
            (Some("incorrect"), ScanError::WrongPassword),
        ] {
            assert_eq!(
                RarCodec.scan_with_password(
                    &mut Cursor::new(bytes),
                    &cancel,
                    ScanLimits::default(),
                    password
                ),
                Err(expected)
            );
        }
    }
    let mut source = Cursor::new(CASES[0]);
    let rows = RarCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let row = rows
        .iter()
        .find(|row| row.name_bytes() == b"b.txt")
        .unwrap();
    for (password, expected) in [
        (None, DecodeError::PasswordRequired),
        (Some("incorrect"), DecodeError::WrongPassword),
    ] {
        let mut payload = RarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink::default();
        assert_eq!(
            payload.decode_with_password(row, &mut sink, &cancel, password),
            Err(expected)
        );
        assert!(sink.0.is_empty());
        assert_eq!(
            payload.decode_with_password(row, &mut sink, &cancel, Some("password")),
            Err(DecodeError::Integrity)
        );
        assert!(sink.0.is_empty());
    }
}

#[test]
fn invalid_verifier_checksum_does_not_reject_correct_key_or_prove_wrong_password() {
    let cancel = Cancellation::default();
    let mut bytes = CASES[0].to_vec();
    let index = read_index(
        &mut Cursor::new(&bytes),
        &cancel,
        ScanLimits::default(),
        None,
    )
    .unwrap();
    let part = index
        .0
        .parts
        .iter()
        .find(|p| p.description.name == b"b.txt")
        .unwrap();
    let check = part.description.crypt.as_ref().unwrap().check.unwrap();
    let positions: Vec<_> = part
        .header
        .windows(check.len())
        .enumerate()
        .filter_map(|(at, value)| (value == check).then_some(at))
        .collect();
    assert_eq!(positions.len(), 1);
    let start = part.header_at as usize;
    bytes[start + positions[0] + 11] ^= 1;
    let crc = xdzip_compression::crc32(&bytes[start + 4..start + part.header.len()]);
    bytes[start..start + 4].copy_from_slice(&crc.to_le_bytes());
    let mut source = Cursor::new(bytes);
    let rows = RarCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let row = rows
        .iter()
        .find(|row| row.name_bytes() == b"b.txt")
        .unwrap();
    for password in ["password", "incorrect"] {
        let mut payload = RarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink::default();
        let result = payload.decode_with_password(row, &mut sink, &cancel, Some(password));
        if password == "password" {
            result.unwrap();
            assert_eq!(sink.0, b"This is from b.txt");
        } else {
            assert_eq!(result, Err(DecodeError::PasswordOrIntegrity));
        }
    }
}

#[test]
fn encrypted_physical_header_is_rechecked_before_selected_output() {
    use std::cell::RefCell;
    use std::io::{self, Read, Seek, SeekFrom};
    use std::rc::Rc;
    struct Shared(Rc<RefCell<Cursor<Vec<u8>>>>);
    impl Read for Shared {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            self.0.borrow_mut().read(bytes)
        }
    }
    impl Seek for Shared {
        fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
            self.0.borrow_mut().seek(at)
        }
    }
    impl ReadSource for Shared {}
    let cancel = Cancellation::default();
    let index = read_index(
        &mut Cursor::new(CASES[1]),
        &cancel,
        ScanLimits::default(),
        Some("password"),
    )
    .unwrap();
    let part = &index.0.parts[0];
    assert!(part.header.len() >= 32);
    let position = part.header_at as usize + 16;
    let rows = index.0.rows.clone();
    let shared = Rc::new(RefCell::new(Cursor::new(CASES[1].to_vec())));
    let mut source = Shared(Rc::clone(&shared));
    let mut payload = RarCodec
        .open_payload_with_password(&mut source, &rows, &cancel, Some("password"))
        .unwrap();
    shared.borrow_mut().get_mut()[position] ^= 1;
    let mut sink = Sink::default();
    assert_eq!(
        payload.decode_with_password(&rows[0], &mut sink, &cancel, Some("password")),
        Err(DecodeError::SourceIo)
    );
    assert!(sink.0.is_empty());
}

#[test]
fn unverified_legacy_header_failure_is_ambiguous_and_payload_source_classes_survive() {
    use std::cell::Cell;
    use std::io::{self, Read, Seek, SeekFrom};
    use std::rc::Rc;
    struct Injected<'a> {
        bytes: Cursor<&'a [u8]>,
        error: Rc<Cell<Option<ScanError>>>,
    }
    impl Read for Injected<'_> {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if let Some(error) = self.error.get() {
                return Err(io::Error::new(io::ErrorKind::Interrupted, error));
            }
            self.bytes.read(out)
        }
    }
    impl Seek for Injected<'_> {
        fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
            self.bytes.seek(position)
        }
    }
    impl ReadSource for Injected<'_> {}
    let control = Cancellation::default();
    assert_eq!(
        RarCodec.scan_with_password(
            &mut Cursor::new(LEGACY[1]),
            &control,
            ScanLimits::default(),
            Some("incorrect")
        ),
        Err(ScanError::PasswordOrIntegrity)
    );
    for (source_error, expected) in [
        (ScanError::SourceChanged, DecodeError::SourceIo),
        (ScanError::Cancelled, DecodeError::Cancelled),
        (ScanError::Limit, DecodeError::Limit),
        (ScanError::Io, DecodeError::SourceIo),
    ] {
        let error = Rc::new(Cell::new(None));
        let mut source = Injected {
            bytes: Cursor::new(LEGACY[0]),
            error: Rc::clone(&error),
        };
        let rows = RarCodec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        let row = rows.iter().find(|row| row.encrypted).unwrap();
        let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
        error.set(Some(source_error));
        let mut sink = Sink::default();
        assert_eq!(
            payload.decode_with_password(row, &mut sink, &control, Some("password")),
            Err(expected)
        );
        assert!(sink.0.is_empty());
        error.set(None);
        assert_eq!(
            payload.decode_with_password(row, &mut sink, &control, Some("password")),
            Err(DecodeError::Integrity)
        );
    }
}

fn owned_input(length: usize, index: usize, noise: bool) -> Vec<u8> {
    let mut state = 0x9e37_79b9u32 ^ index as u32;
    (0..length)
        .map(|i| {
            let mut byte = (i * 29 + (i >> 8) * 3 + index * 11 + 7) as u8;
            if noise {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                if i % 1024 < 512 {
                    byte = state as u8;
                }
            }
            byte
        })
        .collect()
}

#[test]
fn owned_modern_crc_blake_store_and_solid_cipher_cross_authorized_volumes() {
    let cancel = Cancellation::default();
    for case in OWNED {
        let mut source =
            crate::tests::Parts::new(case.parts.iter().map(|part| part.to_vec()).collect());
        let password = case.encrypted.then_some("owned-test");
        let index = read_index(&mut source, &cancel, ScanLimits::default(), password).unwrap();
        let rows = index.entries();
        assert_eq!(rows.len(), case.lengths.len());
        if case.parts.len() > 1 {
            assert_ne!(rows[0].packed_size % 16, 0);
        }
        let mut payload = RarCodec
            .open_payload_with_password(&mut source, rows, &cancel, password)
            .unwrap();
        for selected in (0..rows.len()).rev() {
            let mut sink = Sink::default();
            payload
                .decode_with_password(&rows[selected], &mut sink, &cancel, password)
                .unwrap();
            assert_eq!(
                sink.0,
                owned_input(case.lengths[selected], selected, case.noise)
            );
        }
    }
}

#[test]
fn encrypted_split_corruption_and_missing_authorized_volume_never_succeed() {
    let cancel = Cancellation::default();
    for case in OWNED.iter().filter(|case| case.parts.len() > 1) {
        let mut parts: Vec<Vec<u8>> = case.parts.iter().map(|part| part.to_vec()).collect();
        let index = read_index(
            &mut crate::tests::Parts::new(parts.clone()),
            &cancel,
            ScanLimits::default(),
            Some("owned-test"),
        )
        .unwrap();
        let first = &index.0.parts[0];
        parts[0][first.data_at as usize + 5] ^= 1;
        let mut source = crate::tests::Parts::new(parts);
        let rows = RarCodec
            .scan_with_password(
                &mut source,
                &cancel,
                ScanLimits::default(),
                Some("owned-test"),
            )
            .unwrap();
        let mut payload = RarCodec
            .open_payload_with_password(&mut source, &rows, &cancel, Some("owned-test"))
            .unwrap();
        assert_eq!(
            payload.decode_with_password(
                &rows[0],
                &mut Sink::default(),
                &cancel,
                Some("owned-test")
            ),
            Err(DecodeError::Integrity)
        );
        let mut missing = crate::tests::Parts::new(
            case.parts[..case.parts.len() - 1]
                .iter()
                .map(|part| part.to_vec())
                .collect(),
        );
        assert_eq!(
            RarCodec.scan_with_password(
                &mut missing,
                &cancel,
                ScanLimits::default(),
                Some("owned-test")
            ),
            Err(ScanError::MissingVolume)
        );
    }
}
