use super::*;
use crate::crc32;
use std::{cell::Cell, io::Cursor, rc::Rc};
#[path = "fixture_cases.rs"]
mod fixture_cases;
use fixture_cases::Case;
#[path = "../../../../tests/fixtures/arj/known.rs"]
mod known;
use known::plain;
fn archive(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/arj")
            .join(name),
    )
    .unwrap()
}
fn raw(name: &str) -> (Vec<u8>, u8, u64) {
    let &Case {
        method,
        offset,
        packed,
        decoded,
        ..
    } = fixture_cases::CASES
        .iter()
        .find(|row| row.name == name)
        .unwrap();
    (
        archive(name)[offset..offset + packed].to_vec(),
        method,
        decoded as u64,
    )
}
#[test]
fn every_native_compressed_method_matches_the_entire_independent_known_content() {
    for &Case {
        name,
        kind,
        requested,
        method,
        offset,
        packed,
        decoded,
        crc,
    } in fixture_cases::CASES
    {
        let bytes = archive(name);
        let raw = &bytes[offset..offset + packed];
        let expected = plain(kind);
        assert_eq!(expected.len(), decoded);
        assert_eq!(crc32(&expected), crc);
        if method == 0 {
            assert_eq!(raw, expected);
            continue;
        }
        assert_eq!(method, requested);
        let mut reader = ArjReader::new(
            Cursor::new(raw),
            method,
            packed as u64,
            decoded as u64,
            arj_memory_required(method).unwrap(),
            MAX_WORK,
        )
        .unwrap();
        let mut actual = vec![];
        reader
            .read_to_end(&mut actual)
            .unwrap_or_else(|error| panic!("{name}: {error}; decoded {}", actual.len()));
        assert_eq!(actual, expected, "{name}");
    }
}
#[test]
fn partial_reads_preserve_static_blocks_and_fast_overlapping_history() {
    for (name, kind) in [
        ("m1-window21k.arj", "window21k"),
        ("m4-window14k.arj", "window14k"),
    ] {
        let (bytes, method, decoded) = raw(name);
        assert_ne!(method, 0);
        for quantum in [1, 37] {
            let mut reader = ArjReader::new(
                Cursor::new(&bytes),
                method,
                bytes.len() as u64,
                decoded,
                arj_memory_required(method).unwrap(),
                MAX_WORK,
            )
            .unwrap();
            let mut actual = vec![];
            let mut buffer = vec![0; quantum];
            loop {
                let n = reader.read(&mut buffer).unwrap();
                if n == 0 {
                    break;
                }
                actual.extend_from_slice(&buffer[..n]);
            }
            assert_eq!(actual, plain(kind));
        }
    }
}
fn fields(values: &[(u32, u8)]) -> Vec<u8> {
    let mut result = vec![];
    let mut byte = 0;
    let mut count = 0;
    for &(value, width) in values {
        for bit in (0..width).rev() {
            byte = (byte << 1) | ((value >> bit) & 1) as u8;
            count += 1;
            if count == 8 {
                result.push(byte);
                byte = 0;
                count = 0;
            }
        }
    }
    if count != 0 {
        result.push(byte << (8 - count));
    }
    result
}
#[test]
fn shared_static_prefixes_keep_arj_history_limits_separate_from_lha_initial_spaces() {
    let initial = fields(&[(1, 16), (0, 5), (0, 5), (0, 9), (256, 9), (0, 5), (0, 5)]);
    let mut lha = LzhReader::new(
        Cursor::new(&initial),
        7,
        initial.len() as u64,
        3,
        256 * 1024,
        MAX_WORK,
    )
    .unwrap();
    let mut output = vec![];
    lha.read_to_end(&mut output).unwrap();
    assert_eq!(output, b"   ");
    for method in 1..=3 {
        let mut arj = ArjReader::new(
            Cursor::new(&initial),
            method,
            initial.len() as u64,
            3,
            arj_memory_required(method).unwrap(),
            MAX_WORK,
        )
        .unwrap();
        assert!(arj.read_to_end(&mut vec![]).is_err());
    }
    for offset in [26623, 26624] {
        let bytes = fields(&[
            (30000, 16),
            (0, 5),
            (0, 5),
            (0, 9),
            (65, 9),
            (0, 5),
            (0, 5),
            (1, 16),
            (0, 5),
            (0, 5),
            (0, 9),
            (256, 9),
            (0, 5),
            (15, 5),
            (offset - 16384, 14),
        ]);
        let mut reader = ArjReader::new(
            Cursor::new(&bytes),
            1,
            bytes.len() as u64,
            30003,
            arj_memory_required(1).unwrap(),
            MAX_WORK,
        )
        .unwrap();
        let mut output = vec![];
        let result = reader.read_to_end(&mut output);
        if offset == 26623 {
            result.unwrap();
            assert_eq!(output, vec![65; 30003]);
        } else {
            assert!(result.is_err());
        }
    }
    let initial_fast = fields(&[(0b100, 3), (0, 10)]);
    let mut reader = ArjReader::new(
        Cursor::new(&initial_fast),
        4,
        initial_fast.len() as u64,
        3,
        arj_memory_required(4).unwrap(),
        MAX_WORK,
    )
    .unwrap();
    assert!(reader.read_to_end(&mut vec![]).is_err());
}
#[test]
fn exact_memory_work_input_and_output_ceilings_never_supply_more_credit_or_eof() {
    for method in 1..=4 {
        let (bytes, actual, size) = raw(&format!("m{method}-phrase.arj"));
        assert_eq!(actual, method);
        let make = |work| {
            ArjReader::new(
                Cursor::new(&bytes),
                method,
                bytes.len() as u64,
                size,
                arj_memory_required(method).unwrap(),
                work,
            )
        };
        let mut reader = make(MAX_WORK).unwrap();
        reader.read_to_end(&mut vec![]).unwrap();
        let used = MAX_WORK - reader.work_remaining();
        let mut reader = make(used).unwrap();
        reader.read_to_end(&mut vec![]).unwrap();
        assert_eq!(reader.work_remaining(), 0);
        let mut reader = make(used - 1).unwrap();
        assert_eq!(
            reader.read_to_end(&mut vec![]).unwrap_err().kind(),
            io::ErrorKind::OutOfMemory
        );
        assert!(
            ArjReader::new(
                Cursor::new(&bytes),
                method,
                bytes.len() as u64,
                size,
                arj_memory_required(method).unwrap() - 1,
                MAX_WORK
            )
            .is_err()
        );
        assert!(
            ArjReader::new(
                Cursor::new([]),
                method,
                MAX_BYTES + 1,
                1,
                arj_memory_required(method).unwrap(),
                MAX_WORK
            )
            .is_err()
        );
        assert!(
            ArjReader::new(
                Cursor::new([]),
                method,
                0,
                MAX_BYTES + 1,
                arj_memory_required(method).unwrap(),
                MAX_WORK
            )
            .is_err()
        );
        for (packed, decoded) in [
            (bytes[..bytes.len() - 1].to_vec(), size),
            (bytes.clone(), size - 1),
            (bytes.clone(), size + 1),
            ([bytes.clone(), vec![0]].concat(), size),
        ] {
            let mut reader = ArjReader::new(
                Cursor::new(&packed),
                method,
                packed.len() as u64,
                decoded,
                arj_memory_required(method).unwrap(),
                MAX_WORK,
            )
            .unwrap();
            assert!(
                reader.read_to_end(&mut vec![]).is_err(),
                "method {method}, packed {}, size {decoded}",
                packed.len()
            );
        }
    }
    for method in [0, 5, 255] {
        assert_eq!(
            arj_memory_required(method).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}
#[derive(Debug)]
struct Stop;
impl std::fmt::Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("owned stop")
    }
}
impl std::error::Error for Stop {}
struct Source {
    data: Cursor<Vec<u8>>,
    reads: Rc<Cell<usize>>,
    bare: bool,
    typed: bool,
    quantum: usize,
}
impl Read for Source {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        if self.typed {
            return Err(io::Error::new(io::ErrorKind::Interrupted, Stop));
        }
        if self.bare {
            self.bare = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let count = out.len().min(self.quantum);
        self.data.read(&mut out[..count])
    }
}
#[test]
fn short_and_typed_interruption_preserve_original_stop_and_poison_without_another_read() {
    for method in 1..=4 {
        let (bytes, _, size) = raw(&format!("m{method}-phrase.arj"));
        let reads = Rc::new(Cell::new(0));
        let source = |typed: bool| Source {
            data: Cursor::new(bytes.clone()),
            reads: reads.clone(),
            bare: !typed,
            typed,
            quantum: 3,
        };
        let mut reader = ArjReader::new(
            source(false),
            method,
            bytes.len() as u64,
            size,
            arj_memory_required(method).unwrap(),
            MAX_WORK,
        )
        .unwrap();
        let mut output = vec![];
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, plain("phrase"));
        let mut reader = ArjReader::new(
            source(true),
            method,
            bytes.len() as u64,
            size,
            arj_memory_required(method).unwrap(),
            MAX_WORK,
        )
        .unwrap();
        reads.set(0);
        let error = reader.read(&mut [0; 32]).unwrap_err();
        assert!(error.get_ref().unwrap().is::<Stop>());
        assert_eq!(reads.get(), 1);
        assert!(reader.read(&mut [0; 32]).is_err());
        assert_eq!(reads.get(), 1);
    }
}
#[test]
fn cancellation_during_buffered_history_work_stops_all_methods_without_source_reads() {
    for method in 1..=4 {
        let (bytes, _, size) = raw(&format!("m{method}-spaces.arj"));
        let reads = Rc::new(Cell::new(0));
        let stop = Cell::new(false);
        let source = Source {
            data: Cursor::new(bytes.clone()),
            reads: reads.clone(),
            bare: false,
            typed: false,
            quantum: 8192,
        };
        let mut reader = ArjReader::with_poll(
            source,
            method,
            bytes.len() as u64,
            size,
            arj_memory_required(method).unwrap(),
            MAX_WORK,
            || {
                if stop.get() {
                    Err(io::Error::new(io::ErrorKind::Interrupted, Stop))
                } else {
                    Ok(())
                }
            },
        )
        .unwrap();
        stop.set(true);
        let error = reader.read_to_end(&mut vec![]).unwrap_err();
        assert!(error.get_ref().unwrap().is::<Stop>());
        assert_eq!(reads.get(), 1);
        assert!(reader.read(&mut [0; 1]).is_err());
        assert_eq!(reads.get(), 1);
    }
}
#[test]
fn bounded_mutations_cannot_panic_overproduce_or_resume_failed_readers() {
    for method in [1, 4] {
        let (bytes, _, size) = raw(&format!("m{method}-phrase.arj"));
        for index in 0..256.min(bytes.len() * 8) {
            let mut changed = bytes.clone();
            changed[index / 8] ^= 1 << (index % 8);
            assert!(
                std::panic::catch_unwind(|| {
                    let mut reader = ArjReader::new(
                        Cursor::new(&changed),
                        method,
                        changed.len() as u64,
                        size,
                        arj_memory_required(method).unwrap(),
                        10_000_000,
                    )
                    .unwrap();
                    let mut output = vec![];
                    let result = reader.read_to_end(&mut output);
                    assert!(output.len() <= size as usize);
                    if result.is_err() {
                        assert!(reader.read(&mut [0; 1]).is_err());
                    }
                })
                .is_ok(),
                "method {method} mutation {index}"
            );
        }
    }
}
#[test]
fn every_fast_unary_width_boundary_leaves_the_following_bits_untouched() {
    for (start, last, cases) in [
        (
            0,
            7,
            vec![
                (0, 1, 0, 0),
                (2, 2, 1, 1),
                (6, 3, 2, 3),
                (14, 4, 3, 7),
                (30, 5, 4, 15),
                (62, 6, 5, 31),
                (126, 7, 6, 63),
                (127, 7, 7, 127),
            ],
        ),
        (
            9,
            13,
            vec![
                (0, 1, 9, 0),
                (2, 2, 10, 512),
                (6, 3, 11, 1536),
                (14, 4, 12, 3584),
                (15, 4, 13, 7680),
            ],
        ),
    ] {
        for (prefix, prefix_width, extra, base) in cases {
            for low in [0, (1u32 << extra) - 1] {
                let bytes = fields(&[(prefix, prefix_width), (low, extra), (5, 3)]);
                let mut reader =
                    Fast::new(Cursor::new(&bytes), bytes.len() as u64, 0, MAX_WORK, || {
                        Ok(())
                    })
                    .unwrap();
                assert_eq!(reader.extended(start, last).unwrap(), (base + low) as usize);
                assert_eq!(reader.bits.get(3).unwrap(), 5);
            }
        }
    }
}
