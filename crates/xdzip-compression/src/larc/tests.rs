use super::*;
use std::io::Cursor;
#[test]
fn independent_larc_payloads_retain_absolute_history_flags_and_long_wraps() {
    let mut cases = 0;
    for &(name, method, header, packed, decoded, crc) in crate::lzh::fixture_cases::CASES {
        let method = match method {
            "lzs" => LarcMethod::Lzs,
            "lz5" => LarcMethod::Lz5,
            _ => continue,
        };
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/lha")
                .join(name),
        )
        .unwrap();
        let mut reader = LarcReader::new(
            Cursor::new(&bytes[header..header + packed]),
            method,
            packed as u64,
            decoded as u64,
            larc_memory_required(method),
            96 * 1024 * 1024 * 1024,
        )
        .unwrap();
        let mut result = vec![];
        reader
            .read_to_end(&mut result)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(result.len(), decoded, "{name}");
        assert_eq!(crate::crc16(&result), crc, "{name}");
        let expected = crate::lzh::tests::expected(name);
        assert_eq!(result, expected, "{name}");
        cases += 1;
    }
    assert_eq!(cases, 18);
}

#[test]
fn byte_spans_memory_work_unwritten_tail_and_truncated_commands_are_terminal() {
    for (method, bytes, cost) in [
        (LarcMethod::Lzs, vec![0xa0, 0x80], 2058),
        (LarcMethod::Lz5, vec![1, b'A'], 4113),
    ] {
        let memory = larc_memory_required(method);
        let mut reader = LarcReader::new(Cursor::new(&bytes), method, 2, 1, memory, cost).unwrap();
        let mut output = vec![];
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, b"A");
        assert_eq!(reader.bits.work, 0);
        assert!(LarcReader::new(Cursor::new(&bytes), method, 2, 1, memory - 1, cost).is_err());
        for (data, packed, decoded, work) in [
            (bytes.clone(), 2, 1, cost - 1),
            (bytes[..1].to_vec(), 2, 1, cost),
            ([bytes.clone(), vec![0]].concat(), 3, 1, cost),
            (vec![0], 1, 18, 1_000_000),
        ] {
            let mut reader =
                LarcReader::new(Cursor::new(data), method, packed, decoded, memory, work).unwrap();
            assert!(reader.read_to_end(&mut vec![]).is_err());
            assert!(reader.read(&mut [0]).is_err());
        }
        let mut reader = LarcReader::new(Cursor::new([]), method, 0, 0, memory, 10000).unwrap();
        assert_eq!(reader.read(&mut [0]).unwrap(), 0);
        assert!(LarcReader::new(Cursor::new(bytes), method, 2, 0, memory, cost).is_err());
    }
    // Three bytes starting at the unwritten LZ5 tail have no unambiguous
    // historical value. Reading them must fail before any output is granted.
    let mut reader = LarcReader::new(
        Cursor::new([0, 0xee, 0xf0]),
        LarcMethod::Lz5,
        3,
        3,
        65536,
        1_000_000,
    )
    .unwrap();
    assert!(reader.read_to_end(&mut vec![]).is_err());
    assert!(reader.read(&mut [0]).is_err());
}

#[test]
fn typed_source_errors_and_copy_work_cancellation_are_preserved() {
    struct Terminal;
    impl Read for Terminal {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "terminal-source",
            ))
        }
    }
    for method in [LarcMethod::Lzs, LarcMethod::Lz5] {
        let mut reader = LarcReader::new(Terminal, method, 2, 1, 65536, 1_000_000).unwrap();
        let error = reader.read(&mut [0]).unwrap_err();
        assert_ne!(error.kind(), io::ErrorKind::Interrupted);
        assert_eq!(error.to_string(), "terminal-source");
        assert!(reader.read(&mut [0]).is_err());
    }
    let bytes = include_bytes!("../../../../tests/fixtures/lha/lz5-long.lzh");
    let mut polls = 0;
    let mut reader = LarcReader::with_poll(
        Cursor::new(&bytes[43..43 + 69645]),
        LarcMethod::Lz5,
        69645,
        589824,
        65536,
        8_000_000,
        || {
            polls += 1;
            if polls == 3 {
                Err(io::Error::other("copy-cancelled"))
            } else {
                Ok(())
            }
        },
    )
    .unwrap();
    let mut output = vec![];
    assert_eq!(
        reader.read_to_end(&mut output).unwrap_err().to_string(),
        "copy-cancelled"
    );
    assert!(output.len() < 589824);
    assert!(reader.read(&mut [0]).is_err());
}
