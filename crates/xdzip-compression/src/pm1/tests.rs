use super::*;
use crate::crc16;
use std::{cell::Cell, io::Cursor, rc::Rc};
#[path = "fixture_cases.rs"]
mod fixture_cases;
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/pma")
            .join(name),
    )
    .unwrap()
}
fn raw(name: &str) -> (Vec<u8>, u64) {
    let bytes = fixture(name);
    assert_eq!(&bytes[2..7], b"-pm1-");
    let packed = u32::from_le_bytes(bytes[7..11].try_into().unwrap()) as usize;
    let size = u32::from_le_bytes(bytes[11..15].try_into().unwrap());
    (bytes[33..33 + packed].to_vec(), u64::from(size))
}
fn decode(bytes: &[u8], size: u64, work: u64) -> io::Result<(Vec<u8>, u64)> {
    let mut reader = Pm1Reader::new(
        Cursor::new(bytes),
        bytes.len() as u64,
        size,
        pm1_memory_required(),
        work,
    )?;
    let mut output = Vec::new();
    reader.read_to_end(&mut output)?;
    Ok((output, reader.work_remaining()))
}
#[test]
fn all_reference_verified_selectors_lengths_distances_wrap_and_final_literals_match_known_bytes() {
    let known = fixture("pm1-known.bin");
    for &(name, packed, size, crc, offset) in fixture_cases::CASES {
        let (bytes, decoded) = raw(name);
        assert_eq!((bytes.len(), decoded), (packed, size as u64));
        let expected = &known[offset..offset + size];
        assert_eq!(crc16(expected), crc);
        for quantum in [1, 37, 8192] {
            let mut reader = Pm1Reader::new(
                Cursor::new(&bytes),
                packed as u64,
                decoded,
                pm1_memory_required(),
                MAX_WORK,
            )
            .unwrap();
            let mut output = Vec::new();
            let mut buffer = vec![0; quantum];
            loop {
                let n = reader
                    .read(&mut buffer)
                    .unwrap_or_else(|e| panic!("{name}: {e}"));
                if n == 0 {
                    break;
                }
                output.extend_from_slice(&buffer[..n]);
            }
            assert_eq!(output, expected, "{name}/{quantum}");
            assert!(reader.work_remaining() < MAX_WORK);
        }
    }
}
#[test]
fn exact_work_memory_and_declared_spans_have_no_extra_credit_or_synthetic_eof() {
    let (bytes, size) = raw("pm1-selector-00.pma");
    let (expected, left) = decode(&bytes, size, MAX_WORK).unwrap();
    let used = MAX_WORK - left;
    assert_eq!(decode(&bytes, size, used).unwrap(), (expected, 0));
    assert_eq!(
        decode(&bytes, size, used - 1).unwrap_err().kind(),
        io::ErrorKind::OutOfMemory
    );
    assert!(
        Pm1Reader::new(
            Cursor::new(&bytes),
            bytes.len() as u64,
            size,
            pm1_memory_required() - 1,
            MAX_WORK
        )
        .is_err()
    );
    assert!(
        Pm1Reader::new(
            Cursor::new([]),
            MAX_BYTES + 1,
            1,
            pm1_memory_required(),
            MAX_WORK
        )
        .is_err()
    );
    assert!(
        Pm1Reader::new(
            Cursor::new([]),
            0,
            MAX_BYTES + 1,
            pm1_memory_required(),
            MAX_WORK
        )
        .is_err()
    );
    assert!(decode(&bytes, size - 1, MAX_WORK).is_err());
    assert!(decode(&bytes, size + 1, MAX_WORK).is_err());
    for end in [0, 1, bytes.len() / 2, bytes.len() - 1] {
        assert!(decode(&bytes[..end], size, MAX_WORK).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(decode(&extra, size, MAX_WORK).is_err());
    assert_eq!(decode(&[], 0, MAX_WORK).unwrap().0, b"");
    assert!(decode(&[0], 0, MAX_WORK).is_err());
    // A copy before any literal has no initialized byte, unlike PM2's space history.
    assert!(decode(&[0, 0], 2, MAX_WORK).is_err());
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
    bytes: Cursor<Vec<u8>>,
    reads: Rc<Cell<usize>>,
    quantum: usize,
    bare: bool,
    stop: bool,
}
impl Read for Source {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        if self.stop {
            return Err(io::Error::new(io::ErrorKind::Interrupted, Stop));
        }
        if self.bare {
            self.bare = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let count = out.len().min(self.quantum);
        self.bytes.read(&mut out[..count])
    }
}
#[test]
fn short_io_retries_only_bare_interruption_and_typed_stops_poison_the_reader() {
    let (bytes, size) = raw("pm1-long-overlap.pma");
    let reads = Rc::new(Cell::new(0));
    let make = |stop: bool| Source {
        bytes: Cursor::new(bytes.clone()),
        reads: reads.clone(),
        quantum: 3,
        bare: !stop,
        stop,
    };
    let mut reader = Pm1Reader::new(
        make(false),
        bytes.len() as u64,
        size,
        pm1_memory_required(),
        MAX_WORK,
    )
    .unwrap();
    let mut output = vec![];
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, vec![b' '; size as usize]);
    let mut reader = Pm1Reader::new(
        make(true),
        bytes.len() as u64,
        size,
        pm1_memory_required(),
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
#[test]
fn cancellation_during_source_independent_overlap_is_terminal_without_another_read() {
    let (bytes, size) = raw("pm1-long-overlap.pma");
    let reads = Rc::new(Cell::new(0));
    let polls = Cell::new(0);
    let source = Source {
        bytes: Cursor::new(bytes.clone()),
        reads: reads.clone(),
        quantum: 8192,
        bare: false,
        stop: false,
    };
    let mut reader = Pm1Reader::with_poll(
        source,
        bytes.len() as u64,
        size,
        pm1_memory_required(),
        MAX_WORK,
        || {
            polls.set(polls.get() + 1);
            if polls.get() >= 3 {
                Err(io::Error::new(io::ErrorKind::Interrupted, Stop))
            } else {
                Ok(())
            }
        },
    )
    .unwrap();
    let mut output = vec![];
    let error = reader.read_to_end(&mut output).unwrap_err();
    assert!(error.get_ref().unwrap().is::<Stop>());
    assert_eq!(reads.get(), 1);
    assert!(reader.produced > 216);
    assert!(reader.read(&mut [0; 1]).is_err());
    assert_eq!(reads.get(), 1);
}
#[test]
fn bounded_mutations_never_panic_overproduce_or_resume_after_a_failure() {
    let (bytes, size) = raw("pm1-copy-lengths.pma");
    for index in 0..256.min(bytes.len() * 8) {
        let mut mutated = bytes.clone();
        mutated[index / 8] ^= 1 << (index % 8);
        let result = std::panic::catch_unwind(|| {
            let mut reader = Pm1Reader::new(
                Cursor::new(&mutated),
                mutated.len() as u64,
                size,
                pm1_memory_required(),
                10_000_000,
            )
            .unwrap();
            let mut output = vec![];
            let result = reader.read_to_end(&mut output);
            assert!(output.len() <= size as usize);
            if result.is_err() {
                assert!(reader.read(&mut [0; 1]).is_err());
            }
        });
        assert!(result.is_ok(), "mutation {index}");
    }
}
