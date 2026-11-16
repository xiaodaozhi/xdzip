use super::*;
use crate::crc16;
use std::{cell::Cell, io::Cursor, rc::Rc};
#[path = "fixture_cases.rs"]
pub(crate) mod fixture_cases;
fn archive(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/pma")
            .join(name),
    )
    .unwrap()
}
fn raw(name: &str) -> (Vec<u8>, u64, u16) {
    let data = archive(name);
    let header = usize::from(data[0]) + 2;
    let packed = u32::from_le_bytes(data[7..11].try_into().unwrap()) as usize;
    let decoded = u32::from_le_bytes(data[11..15].try_into().unwrap());
    let crc = u16::from_le_bytes(data[31..33].try_into().unwrap());
    (
        data[header..header + packed].to_vec(),
        u64::from(decoded),
        crc,
    )
}
fn plain(name: &str, size: usize) -> Vec<u8> {
    match name {
        "pm2-spaces.pma" => vec![b' '; size],
        "pm2-zeros.pma" => vec![0; size],
        "pm2-allbytes.pma" => (0..=255).cycle().take(size).collect(),
        "pm2-deep28.pma" => (32..=39).rev().cycle().take(size).collect(),
        _ => [
            (0..=255).collect::<Vec<u8>>(),
            b"abracadabra abracadabra\n".repeat(128),
        ]
        .concat()
        .into_iter()
        .cycle()
        .take(size)
        .collect(),
    }
}
fn decode(data: &[u8], size: u64, work: u64) -> io::Result<(Vec<u8>, u64)> {
    let mut reader = Pm2Reader::new(
        Cursor::new(data),
        data.len() as u64,
        size,
        pm2_memory_required(),
        work,
    )?;
    let mut out = vec![];
    reader.read_to_end(&mut out)?;
    Ok((out, reader.work_remaining()))
}
#[test]
fn original_author_inputs_match_complete_independent_bytes_crc_and_every_rebuild_boundary() {
    for &(name, method, header, packed, decoded, crc) in fixture_cases::CASES {
        if method != 2 {
            continue;
        }
        let bytes = archive(name);
        assert_eq!(header, 33);
        assert_eq!(bytes[2..7], *b"-pm2-");
        let (actual, _) = decode(&bytes[header..header + packed], decoded as u64, MAX_WORK)
            .unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(actual, plain(name, decoded), "{name}");
        assert_eq!(crc16(&actual), crc);
    }
}
#[test]
fn exact_work_memory_sizes_and_final_control_fields_are_not_repaired_or_skipped() {
    let (bytes, size, _) = raw("pm2-boundary-512.pma");
    let (expected, left) = decode(&bytes, size, MAX_WORK).unwrap();
    let used = MAX_WORK - left;
    assert_eq!(decode(&bytes, size, used).unwrap(), (expected, 0));
    assert!(decode(&bytes, size, used - 1).is_err());
    assert!(
        Pm2Reader::new(
            Cursor::new(&bytes),
            bytes.len() as u64,
            size,
            pm2_memory_required() - 1,
            MAX_WORK
        )
        .is_err()
    );
    for (packed, decoded) in [(MAX_BYTES + 1, size), (bytes.len() as u64, MAX_BYTES + 1)] {
        assert!(
            Pm2Reader::new(
                Cursor::new(&bytes),
                packed,
                decoded,
                pm2_memory_required(),
                MAX_WORK
            )
            .is_err()
        );
    }
    for name in [
        "pm2-boundary-1024.pma",
        "pm2-boundary-2048.pma",
        "pm2-boundary-4096.pma",
        "pm2-boundary-8192.pma",
    ] {
        let (bytes, size, _) = raw(name);
        for cut in [0, 1, bytes.len() / 2, bytes.len() - 1] {
            assert!(
                decode(&bytes[..cut], size, MAX_WORK).is_err(),
                "{name} truncated at {cut}"
            );
        }
        let mut extra = bytes;
        extra.push(0);
        assert!(decode(&extra, size, MAX_WORK).is_err());
    }
    assert!(decode(&bytes, 0, MAX_WORK).is_err());
    assert_eq!(decode(&[], 0, MAX_WORK).unwrap().0, b"");
}
fn fields(values: &[(u32, u8)]) -> Vec<u8> {
    let mut out = vec![];
    let mut byte = 0u8;
    let mut bits = 0;
    for &(value, width) in values {
        for bit in (0..width).rev() {
            byte = (byte << 1) | ((value >> bit) & 1) as u8;
            bits += 1;
            if bits == 8 {
                out.push(byte);
                byte = 0;
                bits = 0;
            }
        }
    }
    if bits != 0 {
        out.push(byte << (8 - bits));
    }
    out
}
#[test]
fn constant_history_commands_empty_position_tables_and_invalid_tree_counts_remain_distinct() {
    let constant = fields(&[(0, 1), (29, 5), (0, 3)]);
    assert_eq!(decode(&constant, 256, MAX_WORK).unwrap().0, vec![b' '; 256]);
    assert!(decode(&constant, 255, MAX_WORK).is_err());
    for count in [0, 30, 31] {
        assert!(decode(&fields(&[(0, 1), (count, 5), (0, 3)]), 1, MAX_WORK).is_err());
    }
    let missing = fields(&[
        (0, 1),
        (10, 5),
        (0, 3),
        (0, 3),
        (0, 3),
        (0, 3),
        (0, 3),
        (0, 3),
    ]);
    assert!(decode(&missing, 3, MAX_WORK).is_err());
}
#[derive(Debug)]
struct Stop;
impl std::fmt::Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("owned stop")
    }
}
impl std::error::Error for Stop {}
struct Short {
    source: Cursor<Vec<u8>>,
    bare: bool,
    typed: bool,
    reads: Rc<Cell<usize>>,
}
impl Read for Short {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.reads.set(self.reads.get() + 1);
        if self.typed {
            return Err(io::Error::new(io::ErrorKind::Interrupted, Stop));
        }
        if self.bare {
            self.bare = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let n = out.len().min(3);
        self.source.read(&mut out[..n])
    }
}
#[test]
fn short_io_retries_only_bare_interruption_and_typed_failures_poison_without_resume() {
    let (bytes, size, _) = raw("pm2-boundary-512.pma");
    let reads = Rc::new(Cell::new(0));
    let mut reader = Pm2Reader::new(
        Short {
            source: Cursor::new(bytes.clone()),
            bare: true,
            typed: false,
            reads: reads.clone(),
        },
        bytes.len() as u64,
        size,
        pm2_memory_required(),
        MAX_WORK,
    )
    .unwrap();
    let mut out = vec![];
    reader.read_to_end(&mut out).unwrap();
    assert_eq!(out, plain("pm2-boundary-512.pma", size as usize));
    let mut reader = Pm2Reader::new(
        Short {
            source: Cursor::new(bytes.clone()),
            bare: false,
            typed: true,
            reads: reads.clone(),
        },
        bytes.len() as u64,
        size,
        pm2_memory_required(),
        MAX_WORK,
    )
    .unwrap();
    reads.set(0);
    let error = reader.read(&mut [0; 16]).unwrap_err();
    assert!(error.get_ref().unwrap().is::<Stop>());
    assert_eq!(reads.get(), 1);
    assert!(reader.read(&mut [0; 16]).is_err());
    assert_eq!(reads.get(), 1);
}
#[test]
fn source_independent_copy_and_recency_work_remain_cancellable_and_terminal() {
    let (bytes, size, _) = raw("pm2-zeros.pma");
    struct Count {
        bytes: Cursor<Vec<u8>>,
        reads: Rc<Cell<usize>>,
    }
    impl Read for Count {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            self.reads.set(self.reads.get() + 1);
            self.bytes.read(output)
        }
    }
    let reads = Rc::new(Cell::new(0));
    let packed = bytes.len() as u64;
    let source = Count {
        bytes: Cursor::new(bytes),
        reads: reads.clone(),
    };
    let polls = Rc::new(Cell::new(0));
    let seen = polls.clone();
    let mut reader = Pm2Reader::with_poll(
        source,
        packed,
        size,
        pm2_memory_required(),
        MAX_WORK,
        move || {
            seen.set(seen.get() + 1);
            if seen.get() == 4 {
                Err(io::Error::new(io::ErrorKind::Interrupted, Stop))
            } else {
                Ok(())
            }
        },
    )
    .unwrap();
    let error = reader.read_to_end(&mut vec![]).unwrap_err();
    assert!(error.get_ref().unwrap().is::<Stop>());
    assert!(reader.produced > 0 && reader.produced < size);
    assert_eq!(reads.get(), 1);
    assert!(reader.read(&mut [0; 16]).is_err());
}
#[test]
fn bounded_header_and_payload_bit_mutations_cannot_panic_or_resume_after_failure() {
    let (bytes, size, _) = raw("pm2-boundary-512.pma");
    for at in 0..bytes.len().min(32) {
        for bit in 0..8 {
            let mut damaged = bytes.clone();
            damaged[at] ^= 1 << bit;
            let mut reader = Pm2Reader::new(
                Cursor::new(&damaged),
                damaged.len() as u64,
                size,
                pm2_memory_required(),
                2_000_000,
            )
            .unwrap();
            let mut out = vec![];
            if reader.read_to_end(&mut out).is_err() {
                assert!(reader.read(&mut [0; 1]).is_err());
            } else {
                assert_eq!(out.len() as u64, size);
            }
        }
    }
}
