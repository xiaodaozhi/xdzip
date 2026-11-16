use super::*;
use std::io::Write;
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/streams")
            .join(name),
    )
    .unwrap()
}
fn decoded(bytes: &[u8], chunk: usize) -> io::Result<Vec<u8>> {
    let mut reader = Lz4Reader::new(Cursor::new(bytes), bytes.len() as u64, 128 << 20)?;
    let mut output = Vec::new();
    let mut buffer = vec![0; chunk];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            return Ok(output);
        }
        output.extend_from_slice(&buffer[..n]);
    }
}
fn random_prefix() -> Vec<u8> {
    let mut state = 0x726f6f74u32;
    (0..65535)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            (state >> 24) as u8
        })
        .collect()
}
#[test]
fn independent_frames_checks_history_blocks_and_skippable_members_match() {
    let phrase = b"abracadabra abracadabra\n".repeat(6);
    for name in [
        "phrase.lz4",
        "phrase-block-checksum.lz4",
        "phrase-no-checksum.lz4",
        "empty.lz4",
        "linked.lz4",
        "large-block.lz4",
        "multiple.lz4",
        "leading-skip.lz4",
    ] {
        let expected = match name {
            "empty.lz4" => Vec::new(),
            "linked.lz4" => random_prefix().repeat(5),
            "large-block.lz4" => phrase.repeat(10000),
            "multiple.lz4" => phrase.repeat(2),
            _ => phrase.clone(),
        };
        let encoded = fixture(name);
        for chunk in [1, 73, 65536] {
            assert_eq!(decoded(&encoded, chunk).unwrap(), expected, "{name}");
        }
    }
    for name in ["legacy.lz4", "only-skip.lz4"] {
        assert_eq!(
            decoded(&fixture(name), 8192).unwrap_err().kind(),
            io::ErrorKind::Unsupported
        );
    }
}
#[test]
fn every_truncation_check_damage_and_trailing_garbage_fail_without_resuming() {
    let encoded = fixture("phrase-block-checksum.lz4");
    for end in 0..encoded.len() {
        assert!(decoded(&encoded[..end], 31).is_err(), "prefix {end}");
    }
    for at in 0..encoded.len() {
        let mut data = encoded.clone();
        data[at] ^= 1;
        let mut reader = Lz4Reader::new(Cursor::new(&data), data.len() as u64, 128 << 20).unwrap();
        assert!(reader.read_to_end(&mut Vec::new()).is_err(), "byte {at}");
        assert!(reader.read(&mut [0]).is_err());
    }
    assert!(decoded(&[encoded.as_slice(), b"tail"].concat(), 8192).is_err());
}

fn header(flags: u8, bd: u8, extra: &[u8]) -> Vec<u8> {
    let mut bytes = MAGIC.to_le_bytes().to_vec();
    let mut descriptor = vec![flags, bd];
    descriptor.extend_from_slice(extra);
    bytes.extend_from_slice(&descriptor);
    bytes.push((xxhash32(&descriptor) >> 8) as u8);
    bytes
}
#[test]
fn descriptor_grammar_sizes_empty_blocks_and_physical_ranges_are_enforced() {
    for (flags, bd, extra) in [
        (0x61, 0x40, vec![0; 4]),
        (0x62, 0x40, vec![]),
        (0xa0, 0x40, vec![]),
        (0x60, 0x30, vec![]),
        (0x60, 0x41, vec![]),
    ] {
        let mut data = header(flags, bd, &extra);
        data.extend_from_slice(&[0; 4]);
        assert!(decoded(&data, 9).is_err());
    }
    let mut empty = header(0x70, 0x40, &[]);
    empty.extend_from_slice(&0x80000000u32.to_le_bytes());
    empty.extend_from_slice(&xxhash32(&[]).to_le_bytes());
    empty.extend_from_slice(&[0; 4]);
    assert!(decoded(&empty, 1).unwrap().is_empty());
    let mut too_large = header(0x60, 0x40, &[]);
    too_large.extend_from_slice(&65537u32.to_le_bytes());
    assert!(decoded(&too_large, 1).is_err());
    for expected in [0u64, 2] {
        let mut data = header(0x68, 0x40, &expected.to_le_bytes());
        data.extend_from_slice(&0x80000001u32.to_le_bytes());
        data.push(b'X');
        data.extend_from_slice(&[0; 4]);
        assert!(decoded(&data, 5).is_err());
    }
    let encoded = fixture("phrase.lz4");
    let packed = encoded.len() as u64;
    let framed = [encoded.as_slice(), b"outside admitted range"].concat();
    let mut source = Cursor::new(&framed);
    let mut reader = Lz4Reader::new(&mut source, packed, 128 << 20).unwrap();
    reader.read_to_end(&mut Vec::new()).unwrap();
    drop(reader);
    assert_eq!(source.position(), packed);
    assert!(Lz4Reader::new(Cursor::new(&encoded), packed, MEMORY - 1).is_err());
}
#[test]
fn partial_output_failures_poison_and_admission_touches_no_sink() {
    struct Sink {
        left: usize,
        calls: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.calls.set(self.calls.get() + 1);
            if self.left == 0 {
                return Err(io::Error::other("authored output failure"));
            }
            let n = bytes.len().min(self.left);
            self.left -= n;
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    assert!(
        Lz4Writer::new(
            Sink {
                left: 100,
                calls: calls.clone()
            },
            1,
            MEMORY - 1
        )
        .is_err()
    );
    assert_eq!(calls.get(), 0);
    let mut writer = Lz4Writer::new(
        Sink {
            left: 20,
            calls: calls.clone(),
        },
        1,
        128 << 20,
    )
    .unwrap();
    writer
        .write_all(&b"LZ4 failure fixture".repeat(30000))
        .unwrap_err();
    let count = calls.get();
    assert!(writer.write(b"retry").is_err());
    assert!(writer.flush().is_err());
    assert!(writer.finish().is_err());
    assert_eq!(calls.get(), count);
}
#[test]
fn empty_metadata_cannot_bypass_frame_or_block_limits() {
    let mut frames = header(0x60, 0x40, &[]);
    frames.extend_from_slice(&[0; 4]);
    for _ in 0..MAX_ITEMS - 1 {
        frames.extend_from_slice(&0x184d2a50u32.to_le_bytes());
        frames.extend_from_slice(&[0; 4]);
    }
    assert!(decoded(&frames, 1).unwrap().is_empty());
    frames.extend_from_slice(&0x184d2a50u32.to_le_bytes());
    frames.extend_from_slice(&[0; 4]);
    assert_eq!(
        decoded(&frames, 1).unwrap_err().kind(),
        io::ErrorKind::OutOfMemory
    );
    let mut blocks = header(0x60, 0x40, &[]);
    for _ in 0..MAX_ITEMS {
        blocks.extend_from_slice(&0x80000000u32.to_le_bytes());
    }
    let mut exact = blocks.clone();
    exact.extend_from_slice(&[0; 4]);
    assert!(decoded(&exact, 1).unwrap().is_empty());
    blocks.extend_from_slice(&0x80000000u32.to_le_bytes());
    blocks.extend_from_slice(&[0; 4]);
    assert_eq!(
        decoded(&blocks, 1).unwrap_err().kind(),
        io::ErrorKind::OutOfMemory
    );
}
#[test]
fn offered_writer_levels_compress_real_multiblock_content() {
    let plain = b"LZ4 independent writer content\n".repeat(30000);
    for level in [1, 6, 9] {
        for data in [Vec::new(), plain.clone()] {
            let mut writer = Lz4Writer::new(Vec::new(), level, 128 << 20).unwrap();
            for part in data.chunks(997) {
                writer.write_all(part).unwrap();
            }
            let encoded = writer.finish().unwrap();
            assert_eq!(decoded(&encoded, 713).unwrap(), data);
            if !data.is_empty() {
                assert!(encoded.len() < data.len() / 10);
            }
        }
    }
}
#[test]
#[ignore = "requires pinned independent LZ4 tool and a local output directory"]
fn author_oracle_decodes_each_writer_level_and_raw_fallback() {
    let oracle = std::env::var_os("XDZIP_LZ4_REFERENCE").expect("reference");
    let root = std::path::PathBuf::from(std::env::var_os("XDZIP_LZ4_EXPORT_DIR").expect("output"));
    std::fs::create_dir_all(&root).unwrap();
    let inputs = [
        Vec::new(),
        b"LZ4 independent writer content\n".repeat(30000),
        random_prefix(),
    ];
    for level in [1, 6, 9] {
        for (index, plain) in inputs.iter().enumerate() {
            let mut writer = Lz4Writer::new(Vec::new(), level, 128 << 20).unwrap();
            writer.write_all(plain).unwrap();
            let data = writer.finish().unwrap();
            if index != 0 {
                let raw = u32::from_le_bytes(data[7..11].try_into().unwrap()) >> 31 != 0;
                assert_eq!(raw, index == 2);
            }
            let path = root.join(format!("level-{level}-{index}.lz4"));
            std::fs::write(&path, &data).unwrap();
            let output = std::process::Command::new(&oracle)
                .args(["-d", "-c", "-q"])
                .arg(path)
                .output()
                .unwrap();
            assert!(output.status.success());
            assert_eq!(&output.stdout, plain);
        }
    }
}
