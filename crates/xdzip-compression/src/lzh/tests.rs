use super::*;
use std::io::Cursor;
#[test]
fn independently_encoded_static_lha_methods_cover_all_header_levels_and_real_payloads() {
    let mut cases = 0;
    for &(name, method, header, packed, decoded, crc) in fixture_cases::CASES {
        let method = match method {
            "lh4" => 4,
            "lh5" => 5,
            "lh6" => 6,
            "lh7" => 7,
            _ => continue,
        };
        let archive = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/lha")
                .join(name),
        )
        .unwrap();
        let payload = &archive[header..header + packed];
        let mut reader = LzhReader::new(
            Cursor::new(payload),
            method,
            packed as u64,
            decoded as u64,
            lzh_memory_required(method).unwrap(),
            MAX_WORK,
        )
        .unwrap();
        let mut result = vec![];
        reader
            .read_to_end(&mut result)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(result.len(), decoded);
        let expected = expected(name);
        assert_eq!(result, expected, "{name}");
        assert_eq!(crate::crc16(&result), crc, "{name}");
        cases += 1;
    }
    assert_eq!(cases, 33);
}
fn single_literal(commands: u16) -> Vec<u8> {
    let mut bits = vec![];
    for (value, width) in [
        (u32::from(commands), 16),
        (0, 5),
        (0, 5),
        (0, 9),
        (65, 9),
        (0, 4),
        (0, 4),
    ] {
        for bit in (0..width).rev() {
            bits.push(((value >> bit) & 1) as u8);
        }
    }
    bits.resize(bits.len().div_ceil(8) * 8, 0);
    bits.chunks(8)
        .map(|chunk| chunk.iter().fold(0, |value, bit| (value << 1) | bit))
        .collect()
}
#[test]
fn fixed_one_literal_has_exact_work_memory_output_and_byte_span_admission() {
    let bytes = single_literal(1);
    assert_eq!(bytes.len(), 7);
    let memory = lzh_memory_required(5).unwrap();
    for quantum in [1, 8192] {
        let mut reader = LzhReader::new(Cursor::new(&bytes), 5, 7, 1, memory, 8789).unwrap();
        let mut result = vec![];
        let mut buffer = vec![0; quantum];
        loop {
            let n = reader.read(&mut buffer).unwrap();
            if n == 0 {
                break;
            }
            result.extend_from_slice(&buffer[..n]);
        }
        assert_eq!(result, b"A");
        assert_eq!(reader.bits.work, 0);
    }
    assert!(
        matches!(LzhReader::new(Cursor::new(&bytes),5,7,1,memory-1,8789),Err(e) if e.kind()==io::ErrorKind::OutOfMemory)
    );
    let mut reader = LzhReader::new(Cursor::new(&bytes), 5, 7, 1, memory, 8788).unwrap();
    assert_eq!(
        reader.read_to_end(&mut vec![]).unwrap_err().kind(),
        io::ErrorKind::OutOfMemory
    );
    assert_eq!(
        reader.read(&mut [0]).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    let mut padding = bytes.clone();
    padding[6] |= 15;
    let mut reader = LzhReader::new(Cursor::new(&padding), 5, 7, 1, memory, 8789).unwrap();
    let mut result = vec![];
    reader.read_to_end(&mut result).unwrap();
    assert_eq!(result, b"A");
    for bytes in [
        bytes[..6].to_vec(),
        [bytes.clone(), vec![0]].concat(),
        single_literal(0),
        single_literal(2),
    ] {
        let mut reader = LzhReader::new(
            Cursor::new(&bytes),
            5,
            bytes.len() as u64,
            1,
            memory,
            1_000_000,
        )
        .unwrap();
        assert!(reader.read_to_end(&mut vec![]).is_err());
        assert!(reader.read(&mut [0]).is_err());
    }
    let mut empty = LzhReader::new(Cursor::new([]), 5, 0, 0, memory, 8192).unwrap();
    assert_eq!(empty.read(&mut [0]).unwrap(), 0);
    assert!(LzhReader::new(Cursor::new(&bytes), 5, 7, 0, memory, 8789).is_err());
}
#[test]
fn work_poll_and_typed_input_failures_cannot_resume_or_retry() {
    let bytes = single_literal(1);
    let memory = lzh_memory_required(5).unwrap();
    let result = LzhReader::with_poll(Cursor::new(&bytes), 5, 7, 1, memory, 1_000_000, || {
        Err(io::Error::other("owned-cancel"))
    });
    assert_eq!(result.err().unwrap().to_string(), "owned-cancel");
    struct Terminal;
    impl Read for Terminal {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Interrupted, "owned-terminal"))
        }
    }
    let mut reader = LzhReader::new(Terminal, 5, 7, 1, memory, 8789).unwrap();
    let error = reader.read(&mut [0]).unwrap_err();
    assert_ne!(error.kind(), io::ErrorKind::Interrupted);
    assert_eq!(error.to_string(), "owned-terminal");
    assert!(reader.read(&mut [0]).is_err());
}

pub(crate) fn expected(name: &str) -> Vec<u8> {
    if name == "lh5-initial.lzh" {
        b"   ".to_vec()
    } else if name == "lzs-initial.lzh" {
        vec![b' '; 17]
    } else if name == "lz5-initial.lzh" {
        [vec![0; 13], vec![1; 5]].concat()
    } else if name.contains("empty") {
        vec![]
    } else if name.contains("tail1") {
        b"A".to_vec()
    } else if name.contains("spaces") {
        vec![32; 4096]
    } else if name.contains("long") {
        b"abracadabra abracadabra\n".repeat(6 * 4096)
    } else if name.contains("random") {
        let mut state = 0x6d2b79f5u32;
        (0..65000)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 17;
                state ^= state << 5;
                (state >> 24) as u8
            })
            .collect()
    } else {
        b"abracadabra abracadabra\n".repeat(6)
    }
}

#[test]
fn mutations_truncation_and_one_byte_sources_keep_exact_terminal_boundaries() {
    let bytes = include_bytes!("../../../../tests/fixtures/lha/lh5-h0.lzh");
    let raw = &bytes[33..60];
    struct Short<'a>(&'a [u8]);
    impl Read for Short<'_> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let amount = output.len().min(self.0.len()).min(1);
            output[..amount].copy_from_slice(&self.0[..amount]);
            self.0 = &self.0[amount..];
            Ok(amount)
        }
    }
    let mut reader = LzhReader::new(Short(raw), 5, 27, 144, 128 * 1024, 1_000_000).unwrap();
    let mut decoded = vec![];
    reader.read_to_end(&mut decoded).unwrap();
    assert_eq!(decoded, expected("lh5-h0.lzh"));
    for length in 0..raw.len() {
        let mut reader = LzhReader::new(
            Cursor::new(&raw[..length]),
            5,
            27,
            144,
            128 * 1024,
            1_000_000,
        )
        .unwrap();
        assert!(reader.read_to_end(&mut vec![]).is_err(), "length {length}");
        assert!(reader.read(&mut [0]).is_err());
    }
    for bit in 0..raw.len() * 8 {
        let mut bytes = raw.to_vec();
        bytes[bit / 8] ^= 1 << (bit % 8);
        let mut reader =
            LzhReader::new(Cursor::new(bytes), 5, 27, 144, 128 * 1024, 1_000_000).unwrap();
        let mut output = vec![];
        let result = reader.read_to_end(&mut output);
        assert!(output.len() <= 144);
        if result.is_err() {
            assert!(reader.read(&mut [0]).is_err());
        } else {
            assert_eq!(output.len(), 144);
            assert_eq!(reader.read(&mut [0]).unwrap(), 0);
        }
    }
}

#[test]
fn cancellation_during_long_copies_does_not_depend_on_input_reads() {
    let bytes = include_bytes!("../../../../tests/fixtures/lha/lh5-long.lzh");
    let mut polls = 0;
    let mut reader = LzhReader::with_poll(
        Cursor::new(&bytes[43..1801]),
        5,
        1758,
        589824,
        128 * 1024,
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
