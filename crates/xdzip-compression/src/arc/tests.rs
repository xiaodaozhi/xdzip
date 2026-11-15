use super::*;
use std::io::Cursor;
#[path = "fixture_cases.rs"]
mod cases;
#[path = "../../../../tests/fixtures/arc/known.rs"]
mod known;
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/arc")
            .join(name),
    )
    .unwrap()
}
fn packed(case: cases::Case) -> Vec<u8> {
    let bytes = fixture(case.name);
    let header = if case.method == 1 { 25 } else { 29 };
    assert_eq!(bytes[1], case.method);
    assert_eq!(bytes.len(), header + case.packed + 2);
    bytes[header..header + case.packed].to_vec()
}
#[test]
fn all_independent_native_and_wire_inputs_decode_the_exact_known_bytes() {
    for case in cases::CASES {
        let data = packed(case);
        let expected = known::plain(case.kind);
        assert_eq!(expected.len(), case.decoded);
        assert_eq!(crate::crc16(&expected), case.crc);
        assert!(case.method == case.requested || matches!(case.requested, 8 | 9));
        if case.method < 3 {
            assert_eq!(data, expected);
            continue;
        }
        let mut reader = ArcReader::new(
            Cursor::new(data),
            case.method,
            case.packed as u64,
            case.decoded as u64,
            128 * 1024,
            MAX_WORK,
        )
        .unwrap();
        let mut actual = vec![];
        reader
            .read_to_end(&mut actual)
            .unwrap_or_else(|e| panic!("{}: {e}", case.name));
        assert_eq!(actual, expected, "{}", case.name);
        assert!(reader.work_remaining() < MAX_WORK);
    }
}
#[test]
fn all_truncations_extra_bytes_and_wrong_decoded_sizes_fail_without_resuming() {
    for case in cases::CASES
        .into_iter()
        .filter(|c| c.method >= 3 && c.kind == "phrase")
    {
        let original = packed(case);
        for data in [
            original[..original.len() - 1].to_vec(),
            [original.as_slice(), &[0]].concat(),
        ] {
            let result = ArcReader::new(
                Cursor::new(&data),
                case.method,
                data.len() as u64,
                case.decoded as u64,
                128 * 1024,
                MAX_WORK,
            );
            if let Ok(mut reader) = result {
                let mut actual = vec![];
                assert!(reader.read_to_end(&mut actual).is_err(), "{}", case.name);
                assert!(reader.read(&mut [0; 1]).is_err());
            }
        }
        for decoded in [case.decoded - 1, case.decoded + 1] {
            let mut reader = ArcReader::new(
                Cursor::new(&original),
                case.method,
                case.packed as u64,
                decoded as u64,
                128 * 1024,
                MAX_WORK,
            )
            .unwrap();
            assert!(reader.read_to_end(&mut vec![]).is_err(), "{}", case.name);
            assert!(reader.read(&mut [0; 1]).is_err());
        }
    }
}
#[test]
fn malformed_rle_and_squeeze_trees_fail_with_bounded_work() {
    for bytes in [vec![0x90], vec![0x90, 3], vec![b'A', 0x90, 255]] {
        let mut reader = ArcReader::new(
            Cursor::new(&bytes),
            3,
            bytes.len() as u64,
            2,
            128 * 1024,
            10000,
        )
        .unwrap();
        assert!(reader.read_to_end(&mut vec![]).is_err());
    }
    for bytes in [
        vec![1, 0, 0, 0, 255, 254, 0],     // root cycle
        vec![1, 0, 2, 0, 255, 254, 0],     // out-of-range child
        vec![1, 0, 255, 255, 254, 255, 0], // no EOF
        vec![1, 1],                        // too many nodes
    ] {
        assert!(
            ArcReader::new(
                Cursor::new(&bytes),
                4,
                bytes.len() as u64,
                1,
                128 * 1024,
                10000
            )
            .is_err()
        );
    }
}
#[test]
fn exact_memory_work_and_source_independent_cancellation_are_authoritative() {
    use std::{cell::Cell, rc::Rc};
    for method in 3..=9 {
        let case = cases::CASES
            .into_iter()
            .find(|c| c.method == method && c.kind == "phrase")
            .unwrap();
        let bytes = packed(case);
        assert!(
            ArcReader::new(
                Cursor::new(&bytes),
                method,
                bytes.len() as u64,
                case.decoded as u64,
                arc_memory_required(method).unwrap() - 1,
                MAX_WORK
            )
            .is_err()
        );
        let mut reader = ArcReader::new(
            Cursor::new(&bytes),
            method,
            bytes.len() as u64,
            case.decoded as u64,
            arc_memory_required(method).unwrap(),
            MAX_WORK,
        )
        .unwrap();
        reader.read_to_end(&mut vec![]).unwrap();
        let used = MAX_WORK - reader.work_remaining();
        let exact = ArcReader::new(
            Cursor::new(&bytes),
            method,
            bytes.len() as u64,
            case.decoded as u64,
            128 * 1024,
            used,
        );
        let mut reader = exact.unwrap();
        reader.read_to_end(&mut vec![]).unwrap();
        match ArcReader::new(
            Cursor::new(&bytes),
            method,
            bytes.len() as u64,
            case.decoded as u64,
            128 * 1024,
            used - 1,
        ) {
            Ok(mut short) => assert!(short.read_to_end(&mut vec![]).is_err()),
            Err(e) => assert_eq!(e.kind(), io::ErrorKind::OutOfMemory),
        }
        let calls = Rc::new(Cell::new(0));
        let polled = calls.clone();
        let reader = ArcReader::with_poll(
            Cursor::new(&bytes),
            method,
            bytes.len() as u64,
            case.decoded as u64,
            128 * 1024,
            MAX_WORK,
            move || {
                polled.set(polled.get() + 1);
                if polled.get() > 1 {
                    Err(io::Error::other("owned cancellation"))
                } else {
                    Ok(())
                }
            },
        );
        if let Ok(mut reader) = reader {
            assert!(reader.read_to_end(&mut vec![]).is_err(), "method {method}");
            assert!(reader.read(&mut [0; 1]).is_err());
        }
        assert!(calls.get() > 1);
    }
    assert!(arc_memory_required(0).is_err());
    assert!(ArcReader::new(Cursor::new([]), 3, MAX_BYTES + 1, 0, 128 * 1024, MAX_WORK).is_err());
}

#[test]
fn native_random_dynamic_streams_reach_every_width_and_adaptive_clear() {
    for method in [8, 9] {
        let case = cases::CASES
            .into_iter()
            .find(|c| c.requested == method && c.kind == "random")
            .unwrap();
        assert_eq!(case.method, method);
        let mut reader = ArcReader::new(
            Cursor::new(packed(case)),
            method,
            case.packed as u64,
            case.decoded as u64,
            128 * 1024,
            MAX_WORK,
        )
        .unwrap();
        let mut output = vec![];
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, known::plain("random"));
        let maximum = if method == 8 { 12 } else { 13 };
        for width in 9..=maximum {
            assert_ne!(reader.widths & (1 << width), 0);
        }
        assert!(reader.resets > 0, "method {method}");
        let work = reader.work_remaining();
        for _ in 0..3 {
            assert_eq!(reader.read(&mut [0; 1]).unwrap(), 0);
        }
        assert_eq!(reader.work_remaining(), work);
    }
}

#[test]
fn one_byte_reads_and_typed_host_failures_keep_their_original_boundary() {
    use std::fmt;
    #[derive(Debug)]
    struct Host;
    impl fmt::Display for Host {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "owned host stop")
        }
    }
    impl std::error::Error for Host {}
    struct Source {
        bytes: Cursor<Vec<u8>>,
        fail: bool,
    }
    impl Read for Source {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.fail {
                return Err(io::Error::new(io::ErrorKind::Interrupted, Host));
            }
            self.bytes.read(&mut out[..1])
        }
    }
    for case in cases::CASES
        .into_iter()
        .filter(|c| c.method >= 3 && c.kind == "phrase")
    {
        let raw = packed(case);
        let source = Source {
            bytes: Cursor::new(raw.clone()),
            fail: false,
        };
        let mut reader = ArcReader::new(
            source,
            case.method,
            case.packed as u64,
            case.decoded as u64,
            128 * 1024,
            MAX_WORK,
        )
        .unwrap();
        let mut actual = vec![];
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, known::plain(case.kind));
        let source = Source {
            bytes: Cursor::new(raw),
            fail: true,
        };
        let result = ArcReader::new(
            source,
            case.method,
            case.packed as u64,
            case.decoded as u64,
            128 * 1024,
            MAX_WORK,
        );
        let error = match result {
            Ok(mut reader) => {
                let e = reader.read(&mut [0; 8192]).unwrap_err();
                assert!(reader.read(&mut [0; 1]).is_err());
                e
            }
            Err(e) => e,
        };
        assert_ne!(error.kind(), io::ErrorKind::Interrupted);
        assert!(
            error
                .get_ref()
                .and_then(|e| e.downcast_ref::<Host>())
                .is_some()
        );
    }
}

#[test]
fn bounded_tree_and_dictionary_mutations_cannot_panic_or_escape_quotas() {
    for case in cases::CASES
        .into_iter()
        .filter(|c| c.method >= 4 && c.kind == "phrase")
    {
        let raw = packed(case);
        for at in 0..raw.len().min(80) {
            let mut changed = raw.clone();
            changed[at] ^= 0x80;
            let result = ArcReader::new(
                Cursor::new(&changed),
                case.method,
                changed.len() as u64,
                case.decoded as u64,
                128 * 1024,
                128 * 1024,
            );
            if let Ok(mut reader) = result {
                let mut bytes = vec![];
                let _ = reader.read_to_end(&mut bytes);
                assert!(bytes.len() <= case.decoded);
            }
        }
    }
}
