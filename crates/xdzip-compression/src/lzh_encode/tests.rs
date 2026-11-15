use super::*;
use crate::{Crc16, LzhReader};
use std::{
    cell::Cell,
    io::{Cursor, Read},
    rc::Rc,
};
fn random(size: usize) -> Vec<u8> {
    let mut state = 0x6d2b79f5u32;
    (0..size)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state >> 24) as u8
        })
        .collect()
}
fn compress(bytes: &[u8], level: u8, work: u64) -> io::Result<(Vec<u8>, u64)> {
    let mut writer = LzhWriter::new(Vec::new(), level, 1 << 20, work)?;
    writer.write_all(bytes)?;
    writer.finish_with_work()
}
#[test]
fn real_lh5_matches_exact_tails_multiple_blocks_and_retained_history_roundtrip() {
    for level in [1, 6, 9] {
        for size in [
            0, 1, 3, 4, 5, 255, 256, 257, 8192, 32767, 32768, 32769, 65536, 200000,
        ] {
            for plain in [vec![b'A'; size], random(size)] {
                let (packed, _) = compress(&plain, level, MAX_WORK).unwrap();
                let mut actual = Vec::new();
                LzhReader::new(
                    Cursor::new(&packed),
                    5,
                    packed.len() as u64,
                    plain.len() as u64,
                    1 << 20,
                    MAX_WORK,
                )
                .unwrap()
                .read_to_end(&mut actual)
                .unwrap();
                assert_eq!(actual, plain, "level {level}, size {size}");
                if size > 1000 && plain.iter().all(|b| *b == b'A') {
                    assert!(packed.len() < size / 10);
                }
            }
        }
    }
    let history = random(8192);
    let plain = history.repeat(9);
    let (packed, _) = compress(&plain, 9, MAX_WORK).unwrap();
    let mut actual = Vec::new();
    LzhReader::new(
        Cursor::new(&packed),
        5,
        packed.len() as u64,
        plain.len() as u64,
        1 << 20,
        MAX_WORK,
    )
    .unwrap()
    .read_to_end(&mut actual)
    .unwrap();
    assert_eq!(actual, plain);
    assert!(packed.len() < plain.len() / 3);
}
#[test]
fn allocation_work_input_limits_and_level_effort_are_real_and_exact() {
    assert!(LzhWriter::new(Vec::new(), 1, (1 << 20) - 1, MAX_WORK).is_err());
    assert!(LzhWriter::new(Vec::new(), 0, 1 << 20, MAX_WORK).is_err());
    let plain = random(65000);
    let (packed, remaining) = compress(&plain, 9, MAX_WORK).unwrap();
    let used = MAX_WORK - remaining;
    assert_eq!(compress(&plain, 9, used).unwrap(), (packed, 0));
    assert!(compress(&plain, 9, used - 1).is_err());
    let (_, remaining_fast) = compress(&plain, 1, MAX_WORK).unwrap();
    assert!(
        remaining_fast > remaining,
        "higher search depth must perform more actual work"
    );
    let mut writer = LzhWriter::new(Vec::new(), 1, 1 << 20, MAX_WORK).unwrap();
    writer.total = MAX_BYTES;
    assert!(writer.write(b"x").is_err());
    assert!(writer.write(&[]).is_err());
    assert!(writer.finish().is_err());
}
#[derive(Debug)]
struct Stop;
impl std::fmt::Display for Stop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("owned stop")
    }
}
impl std::error::Error for Stop {}
#[test]
fn cancellation_during_matching_and_typed_output_failures_poison_the_writer() {
    let polls = Rc::new(Cell::new(0));
    let counter = polls.clone();
    let mut writer = LzhWriter::with_poll(Vec::new(), 9, 1 << 20, MAX_WORK, move || {
        counter.set(counter.get() + 1);
        if counter.get() == 40 {
            Err(io::Error::new(io::ErrorKind::Interrupted, Stop))
        } else {
            Ok(())
        }
    })
    .unwrap();
    let error = writer.write_all(&random(100000)).unwrap_err();
    assert_eq!(writer.total, 32768);
    assert!(error.get_ref().unwrap().is::<Stop>());
    assert!(writer.write(b"retry").is_err());
    assert!(writer.flush().is_err());
    assert!(writer.finish().is_err());
    struct Reject;
    impl Write for Reject {
        fn write(&mut self, _: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(io::ErrorKind::Interrupted, Stop))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = LzhWriter::new(Reject, 1, 1 << 20, MAX_WORK).unwrap();
    writer.write_all(b"hello").unwrap();
    let error = writer.flush().unwrap_err();
    assert!(error.get_ref().unwrap().is::<Stop>());
    assert!(writer.write(b"again").is_err());
}
#[test]
fn short_output_bare_retry_and_nonfinal_flush_preserve_one_continuous_bitstream() {
    struct Short {
        bytes: Vec<u8>,
        bare: bool,
    }
    impl Write for Short {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if self.bare {
                self.bare = false;
                return Err(io::ErrorKind::Interrupted.into());
            }
            let count = bytes.len().min(3);
            self.bytes.extend_from_slice(&bytes[..count]);
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let plain = random(65000);
    let mut writer = LzhWriter::new(
        Short {
            bytes: vec![],
            bare: true,
        },
        6,
        1 << 20,
        MAX_WORK,
    )
    .unwrap();
    for chunk in plain.chunks(999) {
        writer.write_all(chunk).unwrap();
        writer.flush().unwrap();
    }
    let packed = writer.finish().unwrap().bytes;
    let mut actual = vec![];
    LzhReader::new(
        Cursor::new(&packed),
        5,
        packed.len() as u64,
        plain.len() as u64,
        1 << 20,
        MAX_WORK,
    )
    .unwrap()
    .read_to_end(&mut actual)
    .unwrap();
    assert_eq!(actual, plain);
}
#[test]
#[ignore = "exports owned writer inputs for the explicitly invoked independent-reference check"]
fn export_owned_lh5_reference_outputs() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build")
        .canonicalize()
        .unwrap();
    let out = std::path::PathBuf::from(
        std::env::var_os("XDZIP_LH5_REFERENCE_OUTPUT").expect("explicit owned output required"),
    );
    assert!(out.is_absolute());
    assert!(
        out.parent()
            .unwrap()
            .canonicalize()
            .unwrap()
            .starts_with(&root)
    );
    std::fs::create_dir(&out).unwrap();
    assert!(out.canonicalize().unwrap().starts_with(root));
    for level in [1, 6, 9] {
        for (name, plain) in [
            ("empty", vec![]),
            ("tail1", vec![b'A']),
            ("allbytes", (0..=255).collect()),
            ("spaces", vec![b' '; 100000]),
            ("random", random(65000)),
            ("long", b"abracadabra abracadabra\n".repeat(6 * 4096)),
            ("window", random(8192).repeat(9)),
        ] {
            let (packed, _) = compress(&plain, level, MAX_WORK).unwrap();
            let mut crc = Crc16::new();
            crc.update(&plain);
            let mut header = vec![31, 0];
            header.extend_from_slice(b"-lh5-");
            header.extend_from_slice(&(packed.len() as u32).to_le_bytes());
            header.extend_from_slice(&(plain.len() as u32).to_le_bytes());
            header.extend_from_slice(&0x576eb1aau32.to_le_bytes());
            header.extend_from_slice(&[0x20, 0, 9]);
            header.extend_from_slice(b"hello.bin");
            header.extend_from_slice(&crc.checksum().to_le_bytes());
            header[1] = header[2..].iter().fold(0u8, |s, b| s.wrapping_add(*b));
            header.extend_from_slice(&packed);
            header.push(0);
            std::fs::write(out.join(format!("{name}-{level}.lzh")), header).unwrap();
        }
    }
}
