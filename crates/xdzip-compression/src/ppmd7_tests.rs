use super::*;
const PROPERTIES: [u8; 5] = [2, 0, 8, 0, 0];

#[test]
fn ppmd7_known_size_root_byte_and_complete_range_profile() {
    for order in [2, 64] {
        for memory in [2048u32, 2049] {
            let mut properties = vec![order];
            properties.extend_from_slice(&memory.to_le_bytes());
            properties.push(99);
            let packed = [0; 6];
            let mut reader =
                Ppmd7Reader::new(packed.as_slice(), 6, Some(1), &properties, 1 << 20).unwrap();
            let mut output = Vec::new();
            reader.read_to_end(&mut output).unwrap();
            assert_eq!(output, [0]);
            reader.finish().unwrap();
            assert_eq!(reader.consumed_bytes(), 6);
        }
    }
    for packed in [
        &[0; 5][..],
        &[0; 7][..],
        &[0, 0, 0xfe, 0x06, 0xdf, 0x98, 0][..],
        &[0, 0xff, 0, 0xff, 0, 0][..],
        &[1, 0, 0, 0, 0, 0][..],
    ] {
        let mut reader =
            Ppmd7Reader::new(packed, packed.len() as u64, Some(1), &PROPERTIES, 1 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
        assert!(reader.read(&mut [0]).is_err());
        assert!(reader.finish().is_err());
    }
}

#[test]
fn ppmd7_unknown_requires_end_known_zero_does_not_decode_end() {
    let end = [0, 0xff, 0, 0xff, 0, 0];
    let mut reader = Ppmd7Reader::new(end.as_slice(), 6, None, &PROPERTIES, 1 << 20).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert!(output.is_empty());
    reader.finish().unwrap();
    let mut reader = Ppmd7Reader::new(&[0; 5][..], 5, Some(0), &PROPERTIES, 1 << 20).unwrap();
    reader.finish().unwrap();
    assert_eq!(reader.consumed_bytes(), 5);
    let mut reader = Ppmd7Reader::new(end.as_slice(), 6, Some(0), &PROPERTIES, 1 << 20).unwrap();
    assert!(reader.finish().is_err());
    let mut reader = Ppmd7Reader::new(&[0; 5][..], 5, None, &PROPERTIES, 1 << 20).unwrap();
    assert!(reader.read_to_end(&mut Vec::new()).is_err());
    let with_end = [0, 0, 0xfe, 0x06, 0xdf, 0x98, 0];
    let mut reader = Ppmd7Reader::new(with_end.as_slice(), 7, None, &PROPERTIES, 1 << 20).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, [0]);
    reader.finish().unwrap();
}

#[test]
fn ppmd7_actual_safe_memory_admission_and_property_domain() {
    let budget = ppmd7_memory_required(&PROPERTIES).unwrap();
    assert!(budget > 2048);
    assert!(std::mem::size_of::<arena::Cell>() <= 32);
    let mut large = vec![2];
    large.extend_from_slice(&(8u32 << 20).to_le_bytes());
    eprintln!(
        "ppmd7_memory cell={} minimum={} eight_mib={}",
        std::mem::size_of::<arena::Cell>(),
        budget,
        ppmd7_memory_required(&large).unwrap()
    );
    assert_eq!(
        Ppmd7Reader::new(&[][..], 0, Some(0), &PROPERTIES, budget - 1)
            .err()
            .unwrap()
            .kind(),
        io::ErrorKind::OutOfMemory
    );
    assert!(Ppmd7Reader::new(&[][..], 0, Some(0), &PROPERTIES, budget).is_ok());
    for order in [0, 1, 65, 255] {
        let mut properties = PROPERTIES;
        properties[0] = order;
        assert!(ppmd7_memory_required(&properties).is_err());
    }
    for memory in [0u32, 2047, u32::MAX - 35, u32::MAX] {
        let mut properties = vec![2];
        properties.extend_from_slice(&memory.to_le_bytes());
        assert!(ppmd7_memory_required(&properties).is_err());
    }
    let mut enormous = vec![2];
    enormous.extend_from_slice(&(u32::MAX - 36).to_le_bytes());
    assert!(ppmd7_memory_required(&enormous).unwrap() > u64::from(u32::MAX));
    assert!(Ppmd7Reader::new(&[][..], 0, Some(0), &enormous, 1 << 20).is_err());
    for length in 0..5 {
        assert!(ppmd7_memory_required(&PROPERTIES[..length]).is_err());
    }
}

#[test]
#[ignore = "requires independent official PPMd7 encoder streams and known plaintext"]
fn ppmd7_independent_official_matrix() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_PPMD7_VECTORS").expect("vector directory"),
    );
    let mut entries: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect();
    entries.sort();
    let mut cases = 0;
    for path in entries {
        if path.extension().is_none_or(|extension| extension != "raw") {
            continue;
        }
        let packed = std::fs::read(&path).unwrap();
        let expected = std::fs::read(path.with_extension("plain")).unwrap();
        let properties = std::fs::read(path.with_extension("props")).unwrap();
        let has_end = std::fs::read_to_string(path.with_extension("profile"))
            .is_ok_and(|profile| profile == "end");
        if has_end {
            let mut known = Ppmd7Reader::new(
                packed.as_slice(),
                packed.len() as u64,
                Some(expected.len() as u64),
                &properties,
                512 << 20,
            )
            .unwrap();
            assert!(
                known.read_to_end(&mut Vec::new()).is_err(),
                "known-size must not consume END to repair code"
            );
            assert!(known.finish().is_err());
        }
        let mut reader = Ppmd7Reader::new(
            packed.as_slice(),
            packed.len() as u64,
            if has_end {
                None
            } else {
                Some(expected.len() as u64)
            },
            &properties,
            512 << 20,
        )
        .unwrap();
        let mut output = Vec::new();
        let mut buffer = [0; 32768];
        loop {
            let count = reader.read(&mut buffer).unwrap_or_else(|error| {
                panic!("{} decoded {}: {error}", path.display(), reader.decoded)
            });
            if count == 0 {
                break;
            }
            assert!(count <= 8192);
            output.extend_from_slice(&buffer[..count]);
            assert!(
                output.len() <= expected.len(),
                "unexpected unbounded output"
            );
        }
        assert_eq!(output, expected, "{}", path.display());
        reader.finish().unwrap();
        assert_eq!(reader.consumed_bytes(), packed.len() as u64);
        let diagnostics = reader.model.diagnostics();
        assert_eq!(diagnostics.symbols, expected.len() as u64);
        if let Ok(witness) = std::fs::read_to_string(path.with_extension("restart")) {
            let witness: Vec<u64> = witness
                .split_whitespace()
                .map(|number| number.parse().unwrap())
                .collect();
            assert_eq!(witness.len(), 3);
            assert_eq!(diagnostics.restarts, witness[0]);
            assert_eq!(diagnostics.first_restarts[0], witness[1]);
            assert_eq!(diagnostics.last_restart, witness[2]);
        }
        eprintln!(
            "ppmd7_case {} decoded={} restarts={} glues={} merges={} borrowed={} failed={} first={:?} last={}",
            path.file_stem().unwrap().to_string_lossy(),
            expected.len(),
            diagnostics.restarts,
            diagnostics.arena.glues,
            diagnostics.arena.merges,
            diagnostics.arena.borrowed,
            diagnostics.arena.failed,
            diagnostics.first_restarts,
            diagnostics.last_restart
        );
        cases += 1;
    }
    assert!(cases != 0);
}

// Fixed official 7za 26.05 black-box author, order 6 / M=65536.
// Owned plaintext: b"original adaptive context decoder test\n" repeated 30.
// Packed SHA256 a22394637140fec4d621c249b692382a67e6e8a2dcea55bf0bf4387d1082e209.
const PHRASE_RAW: &[u8] = &[
    0x00, 0x6f, 0x05, 0x1c, 0x0b, 0x6f, 0x1e, 0xb2, 0xee, 0xb9, 0xfd, 0xc6, 0x05, 0x29, 0x38, 0xe7,
    0x9a, 0x84, 0xc5, 0x06, 0xba, 0x48, 0xee, 0xf7, 0x90, 0xa3, 0xe8, 0x9b, 0xe8, 0xf4, 0xa3, 0xc3,
    0x82, 0x3a, 0x36, 0x21, 0x5d, 0x34, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const PHRASE_PROPERTIES: [u8; 5] = [6, 0, 0, 1, 0];
fn phrase() -> Vec<u8> {
    b"original adaptive context decoder test\n".repeat(30)
}
fn phrase_reader<R: Read>(source: R, packed: u64) -> Ppmd7Reader<R> {
    Ppmd7Reader::new(source, packed, Some(1170), &PHRASE_PROPERTIES, 1 << 20).unwrap()
}
#[test]
fn ppmd7_official_fixed_phrase_fragmentation_and_exact_span() {
    for fragment in [1, 7, 8192, 32768] {
        let mut reader = phrase_reader(PHRASE_RAW, PHRASE_RAW.len() as u64);
        let mut output = Vec::new();
        let mut buffer = vec![0; fragment];
        loop {
            let count = reader.read(&mut buffer).unwrap();
            assert!(count <= 8192);
            if count == 0 {
                break;
            }
            output.extend_from_slice(&buffer[..count]);
        }
        assert_eq!(output, phrase());
        reader.finish().unwrap();
        reader.finish().unwrap();
        assert_eq!(reader.decoded_bytes(), 1170);
        assert_eq!(reader.consumed_bytes(), PHRASE_RAW.len() as u64);
    }
    let mut beyond = PHRASE_RAW.to_vec();
    beyond.extend_from_slice(b"out-of-span");
    let mut cursor = io::Cursor::new(beyond);
    {
        let mut reader = phrase_reader(&mut cursor, PHRASE_RAW.len() as u64);
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, phrase());
        reader.finish().unwrap();
    }
    assert_eq!(cursor.position(), PHRASE_RAW.len() as u64);
}
#[test]
fn ppmd7_truncation_extra_bytes_and_unread_finish_poison() {
    for length in 0..PHRASE_RAW.len() {
        let mut reader = phrase_reader(&PHRASE_RAW[..length], length as u64);
        assert!(
            reader.read_to_end(&mut Vec::new()).is_err(),
            "prefix {length}"
        );
        assert!(reader.finish().is_err());
        assert!(reader.read(&mut [0]).is_err());
    }
    for tail in [&[0][..], &[255][..], PHRASE_RAW] {
        let mut extra = PHRASE_RAW.to_vec();
        extra.extend_from_slice(tail);
        let mut reader = phrase_reader(extra.as_slice(), extra.len() as u64);
        assert!(reader.read_to_end(&mut Vec::new()).is_err());
        assert!(reader.finish().is_err());
    }
    let mut reader = phrase_reader(PHRASE_RAW, PHRASE_RAW.len() as u64);
    assert!(
        reader.finish().is_err(),
        "finish must not discard unread output"
    );
    assert!(reader.read(&mut [0]).is_err());
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Marker {
    SourceChanged,
    Cancelled,
    Limit,
}
impl std::fmt::Display for Marker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{self:?}")
    }
}
impl std::error::Error for Marker {}
struct FaultSource<'a> {
    bytes: &'a [u8],
    before_error: usize,
    marker: Option<Marker>,
    raised: bool,
    calls: std::rc::Rc<std::cell::Cell<usize>>,
}
impl Read for FaultSource<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.calls.set(self.calls.get() + 1);
        if self.before_error == 0 && !self.raised {
            self.raised = true;
            return Err(match self.marker {
                Some(marker) => io::Error::new(io::ErrorKind::Interrupted, marker),
                None => io::Error::from(io::ErrorKind::Interrupted),
            });
        }
        let count = self.bytes.len().min(output.len()).min(if self.raised {
            usize::MAX
        } else {
            self.before_error
        });
        output[..count].copy_from_slice(&self.bytes[..count]);
        self.bytes = &self.bytes[count..];
        self.before_error = self.before_error.saturating_sub(count);
        Ok(count)
    }
}
#[test]
fn ppmd7_typed_input_errors_survive_first_partial_and_std_helpers() {
    for marker in [Marker::SourceChanged, Marker::Cancelled, Marker::Limit] {
        for partial in [false, true] {
            for method in 0..3 {
                let calls = std::rc::Rc::new(std::cell::Cell::new(0));
                let source = FaultSource {
                    bytes: PHRASE_RAW,
                    before_error: if partial { 10 } else { 0 },
                    marker: Some(marker),
                    raised: false,
                    calls: calls.clone(),
                };
                let mut reader = phrase_reader(source, PHRASE_RAW.len() as u64);
                if partial {
                    let mut first = [0];
                    assert_eq!(reader.read(&mut first).unwrap(), 1);
                    assert_eq!(first, [b'o']);
                    assert_eq!(reader.decoded_bytes(), 1);
                }
                let error = match method {
                    0 => loop {
                        match reader.read(&mut [0; 16]) {
                            Ok(count) => assert!(count != 0, "typed failure must not become EOF"),
                            Err(error) => break error,
                        }
                    },
                    1 => reader.read_to_end(&mut Vec::new()).unwrap_err(),
                    _ => reader.read_exact(&mut [0; 1170]).unwrap_err(),
                };
                assert_eq!(error.kind(), io::ErrorKind::Other);
                assert_eq!(
                    error.get_ref().unwrap().downcast_ref::<Marker>(),
                    Some(&marker)
                );
                let before = calls.get();
                assert!(reader.read(&mut [0]).is_err());
                assert!(reader.finish().is_err());
                assert_eq!(calls.get(), before, "poison must seal the source");
            }
        }
    }
}
#[test]
fn ppmd7_bare_interruptions_retry_before_and_after_partial_input() {
    for boundary in [0, 10] {
        let calls = std::rc::Rc::new(std::cell::Cell::new(0));
        let source = FaultSource {
            bytes: PHRASE_RAW,
            before_error: boundary,
            marker: None,
            raised: false,
            calls,
        };
        let mut reader = phrase_reader(source, PHRASE_RAW.len() as u64);
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, phrase());
        reader.finish().unwrap();
    }
}

// Unchanged official 26.05 C encoder, independently accepted by fixed 7za t/x
// and exact official decoder. Owned plain is (0..=255) repeated twice.
// Raw SHA256 77d9aa3fe2ad79d80dd5d5164174f8d4d66fee58b93bc10791266ced8ae116ad.
const RESTART_RAW: &[u8] = &[
    0x00, 0x00, 0x04, 0xec, 0x15, 0xc5, 0xe7, 0x05, 0x01, 0xfd, 0x6b, 0x52, 0xb0, 0x56, 0x17, 0x54,
    0xc0, 0x6d, 0x27, 0xea, 0xe7, 0x68, 0x9d, 0x46, 0xc5, 0x68, 0x0a, 0x19, 0xc9, 0x89, 0x09, 0x76,
    0x4b, 0x8e, 0xa3, 0x88, 0x3b, 0xeb, 0xb4, 0x53, 0xb3, 0x11, 0xaa, 0x6a, 0x7f, 0xd9, 0x25, 0x89,
    0x6f, 0x36, 0x68, 0x7e, 0x5d, 0x9f, 0xc2, 0x91, 0xa7, 0x08, 0xb4, 0x7f, 0x79, 0xf4, 0x83, 0x43,
    0xab, 0x9a, 0xd8, 0x7d, 0xc9, 0x55, 0xeb, 0xd1, 0xb9, 0x20, 0x58, 0x6b, 0x58, 0x71, 0x5b, 0x5f,
    0x10, 0x6e, 0xc2, 0x6a, 0xa8, 0xe1, 0x36, 0xb9, 0xd4, 0x2b, 0xc5, 0x2b, 0xc6, 0x36, 0x53, 0x5f,
    0x3f, 0x0d, 0xf9, 0xa6, 0x84, 0x9b, 0x5b, 0x08, 0xeb, 0xf3, 0x22, 0x83, 0xb1, 0xbf, 0x60, 0x7b,
    0x32, 0xac, 0xd6, 0x5a, 0xec, 0x90, 0x18, 0x08, 0x33, 0x11, 0x03, 0xd6, 0x72, 0xde, 0x89, 0x21,
    0x11, 0xa8, 0xc3, 0xbf, 0x86, 0x06, 0xf3, 0xc6, 0x1c, 0x86, 0x10, 0x36, 0xd1, 0xcc, 0xba, 0x13,
    0xee, 0x4c, 0x10, 0x5a, 0x96, 0x10, 0xb1, 0x89, 0x4c, 0xe1, 0x46, 0xe2, 0x8d, 0x87, 0xf9, 0x09,
    0xf9, 0x70, 0x38, 0x26, 0x1d, 0xcb, 0x08, 0xa0, 0x60, 0xe1, 0x8d, 0x56, 0xf8, 0xb4, 0xc5, 0xc8,
    0x79, 0x18, 0x00, 0xc3, 0xed, 0x1a, 0xa5, 0xe9, 0x05, 0x4e, 0x16, 0x85, 0xff, 0xf9, 0x6f, 0xfd,
    0x2e, 0xed, 0xb9, 0xc5, 0x1f, 0x01, 0xc6, 0xd5, 0x51, 0x71, 0xf1, 0xb4, 0x69, 0x8a, 0x67, 0x6f,
    0x7e, 0xd7, 0xa6, 0x9a, 0x8e, 0xbb, 0x86, 0xb1, 0xe3, 0x6e, 0x71, 0x8c, 0x15, 0x8c, 0xf6, 0x02,
    0x95, 0x12, 0x5d, 0x6b, 0xd0, 0x27, 0x5e, 0x17, 0x15, 0xc2, 0xed, 0x5d, 0xec, 0xd1, 0xc9, 0x6d,
    0x75, 0xc3, 0xa5, 0xeb, 0x87, 0x02, 0x39, 0xd1, 0xcc, 0xc6, 0xac, 0x8d, 0x86, 0x10, 0x77, 0x36,
    0xbe, 0x20, 0x9f, 0xd5, 0x5c, 0x4e, 0xcc, 0xd8, 0x50, 0x9d, 0xf5, 0x79, 0xa6, 0x7c, 0xd5, 0x37,
    0x1c, 0x96, 0xa6, 0x5d, 0x75, 0x8e, 0x44, 0xe2, 0x8c, 0xe5, 0xc5, 0x18, 0x28, 0x2f, 0x48, 0x82,
    0x14, 0x6b, 0x9a, 0x6f, 0x3a, 0x70, 0x07, 0xf5, 0xb9, 0x8e, 0x56, 0xd9, 0xe3, 0x68, 0x49, 0xe0,
    0xaf, 0xee, 0xfa, 0xae, 0x6a, 0xfd, 0xe8, 0x2e, 0x10, 0x76, 0x45, 0x53, 0xea, 0x76, 0xbb, 0x82,
    0xae, 0x42, 0x09, 0xd1, 0x73, 0x00, 0x1c, 0x3e, 0x03, 0xc8, 0x78, 0x74, 0x96, 0x5a, 0x53, 0xbe,
    0x88, 0x61, 0xc9, 0xda, 0xed, 0xe1, 0xe7, 0xd1, 0x3e, 0xf5, 0xc6, 0xd3, 0xe6, 0xf1, 0xfb, 0x39,
    0x2a, 0x7a, 0x6f, 0x1e, 0xf8, 0x82, 0x15, 0x97, 0xcb, 0xe6, 0x31, 0x70, 0x04, 0x7a, 0xae, 0xed,
    0xb6, 0x46, 0x4c, 0xeb, 0xdb, 0xf3, 0xfc, 0x2e, 0xbb, 0x76, 0x95, 0xdc, 0x9a, 0xf1, 0xc1, 0xff,
    0xc8, 0x8a, 0xde, 0xea, 0xce, 0xa4, 0x33, 0x97, 0x05, 0x4c, 0x1d, 0xe1, 0xd0, 0x16, 0x12, 0xa5,
    0x17, 0x79, 0xb9, 0x94, 0x27, 0x9c, 0x77, 0x2d, 0x10, 0xca, 0xa2, 0x69, 0x64, 0x43, 0x8c, 0x68,
    0x78, 0x39, 0x99, 0xa7, 0xd8, 0x16, 0x3c, 0xbf, 0xd0, 0xe5, 0x2f, 0xce, 0x48, 0x39, 0x67, 0x1f,
    0xc5, 0xff, 0xe6, 0xef, 0xf3, 0xff, 0x94, 0xdc, 0x5f, 0x14, 0xc8, 0x06, 0x42, 0xcd, 0xfd, 0xc1,
    0x95, 0x41, 0x33, 0x1a, 0x33, 0xcb, 0x00, 0x18, 0xfc, 0x41, 0x7d, 0xfd, 0xa9, 0xa6, 0x5e, 0x5d,
    0xe7, 0x29, 0x1b, 0x88, 0x83, 0xca, 0x69, 0xa7, 0x14, 0x40, 0xe6, 0x79, 0x36, 0x39, 0xda, 0x3c,
    0xea, 0xed, 0x23, 0x5e, 0x8b, 0xc1, 0xdb, 0xfb, 0x81, 0x14, 0x4a, 0xcc, 0x3f, 0xe5, 0x9c, 0xa7,
    0x26, 0x60, 0x5e, 0x27, 0xfa, 0xb9, 0x93, 0xf8, 0x38, 0x1f, 0x8f, 0x78, 0x91, 0x5c, 0x77, 0x01,
    0x63, 0xff, 0x7d, 0xcc, 0x78, 0xa5, 0x82, 0x22, 0x2c, 0x3b, 0xdd, 0xdd, 0xdf, 0xaf, 0x41, 0x73,
    0x88, 0x45, 0x2d, 0xa6, 0xf9, 0x07, 0xda, 0x9c, 0x76, 0x5f, 0x41, 0x78, 0xfa, 0x0d, 0x03, 0xc8,
    0x81, 0xb5, 0xe1, 0x9c, 0xc2, 0x65, 0xe5, 0xca, 0xa8, 0xed, 0x92, 0x39, 0xff, 0x03, 0x76, 0xdc,
    0x4a, 0xe8, 0x6a, 0x2a, 0x7f, 0xf8, 0x62, 0x46, 0x88, 0x1e, 0x34, 0x89, 0x99, 0x43, 0x2f, 0x46,
    0x2b, 0xe8, 0xef, 0x18, 0x2b, 0xe8, 0xaf, 0xbb, 0x6a, 0x21, 0x5d, 0x82, 0x71, 0x63, 0xc4, 0xd6,
    0x05, 0x9f, 0x1b, 0x44, 0xaf, 0xb5, 0xef, 0x33, 0xb3, 0xf2, 0x00, 0x00,
];
#[test]
fn ppmd7_official_small_model_restarts_at_the_observed_byte() {
    let properties = [64, 1, 8, 0, 0];
    let mut reader = Ppmd7Reader::new(
        RESTART_RAW,
        RESTART_RAW.len() as u64,
        Some(512),
        &properties,
        1 << 20,
    )
    .unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, (0..=255).cycle().take(512).collect::<Vec<u8>>());
    reader.finish().unwrap();
    let diagnostics = reader.model.diagnostics();
    assert_eq!(diagnostics.restarts, 1);
    assert_eq!(diagnostics.first_restarts[0], 262);
    assert_eq!(reader.consumed_bytes(), 604);
}
#[test]
fn ppmd7_bounded_bit_mutations_do_not_panic_or_resume_after_error() {
    let mut rejected = 0;
    for bit in 0..PHRASE_RAW.len() * 8 {
        let mut changed = PHRASE_RAW.to_vec();
        changed[bit / 8] ^= 1 << (bit % 8);
        let mut reader = phrase_reader(changed.as_slice(), changed.len() as u64);
        let mut output = Vec::new();
        match reader.read_to_end(&mut output) {
            Ok(_) => {
                assert_eq!(output.len(), 1170);
                reader.finish().unwrap();
            }
            Err(_) => {
                rejected += 1;
                assert!(reader.finish().is_err());
                assert!(reader.read(&mut [0]).is_err());
            }
        }
        assert!(reader.decoded_bytes() <= 1170);
        assert!(reader.consumed_bytes() <= changed.len() as u64);
    }
    assert!(rejected != 0);
}
