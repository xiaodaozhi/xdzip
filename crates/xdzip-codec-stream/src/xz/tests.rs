use super::*;
use std::io::{Cursor, Write};
const MEMORY: u64 = 128 << 20;
const PHRASE: &[u8] = b"abracadabra abracadabra\n";
fn decode(bytes: &[u8], chunk: usize) -> io::Result<(Vec<u8>, bool)> {
    let mut reader = XzReader::new(Cursor::new(bytes), bytes.len() as u64, MEMORY)?;
    let mut result = Vec::new();
    let mut buffer = vec![0; chunk];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        result.extend_from_slice(&buffer[..n]);
    }
    Ok((result, reader.has_content_check()))
}
#[test]
fn independent_checks_blocks_members_and_filters_match_known_bytes() {
    let root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/streams");
    for name in [
        "phrase-none.xz",
        "phrase-crc32.xz",
        "phrase-crc64.xz",
        "phrase-sha256.xz",
        "empty.xz",
        "multiple.xz",
        "multiblock.xz",
    ] {
        let encoded = std::fs::read(root.join(name)).unwrap();
        let expected = match name {
            "empty.xz" => Vec::new(),
            "multiple.xz" => PHRASE.repeat(12),
            "multiblock.xz" => PHRASE.repeat(24),
            _ => PHRASE.repeat(6),
        };
        for chunk in [1, 7, 32768] {
            let (actual, check) = decode(&encoded, chunk).unwrap();
            assert_eq!(actual, expected, "{name}");
            assert_eq!(check, name != "phrase-none.xz");
        }
    }
    let instructions = [
        0xe8, 0, 0, 0, 0, 0xe9, 0xfa, 0xff, 0xff, 0xff, 0, 0, 0, 0xeb, 0, 0, 0, 0x48, 0, 0, 0,
        0x40, 0, 0, 0, 0x90, 0, 0, 0, 0x94, 0, 0, 0, 0xf0, 0, 0xf8, 0, 0, 0, 0,
    ]
    .repeat(300);
    for name in [
        "x86", "ppc", "ia64", "arm", "thumb", "sparc", "arm64", "riscv", "delta", "chain",
    ] {
        let encoded = std::fs::read(root.join(format!("filter-{name}.xz"))).unwrap();
        assert_eq!(decode(&encoded, 13).unwrap().0, instructions, "{name}");
    }
}
#[test]
fn checks_all_truncations_and_damage_poison_without_retry() {
    let original = include_bytes!("../../../../tests/fixtures/streams/phrase-crc64.xz");
    for size in 0..original.len() {
        assert!(decode(&original[..size], 8192).is_err(), "prefix {size}");
    }
    for at in 0..original.len() {
        let mut broken = original.to_vec();
        broken[at] ^= 1;
        let mut reader = XzReader::new(Cursor::new(&broken), broken.len() as u64, MEMORY).unwrap();
        let mut output = Vec::new();
        if reader.read_to_end(&mut output).is_err() {
            assert!(reader.read(&mut [0]).is_err());
            assert!(!reader.has_content_check());
        } else {
            // Some arithmetic-coder input bits are redundant. Accepted changes
            // must still match the independently known content and full check.
            assert_eq!(output, PHRASE.repeat(6), "byte {at}");
            assert!(reader.has_content_check());
        }
    }
    for suffix in [&b"\0"[..], &b"\0\0"[..], &b"\0\0\0"[..], &b"garbage"[..]] {
        assert!(decode(&[original.as_slice(), suffix].concat(), 99).is_err());
    }
    assert!(XzReader::new(Cursor::new(original), original.len() as u64, 1024).is_err());
}
#[test]
fn variable_integers_are_canonical_bounded_and_exact() {
    for number in [0, 127, 128, 16383, 16384, (1 << 56) - 1, i64::MAX as u64] {
        let mut encoded = Vec::new();
        append_vli(&mut encoded, number).unwrap();
        let mut input = encoded.as_slice();
        assert_eq!(vli(&mut input).unwrap(), number);
        assert!(input.is_empty());
    }
    for bytes in [&b"\x80\0"[..], &b"\x81\0"[..], &[255; 9][..], &b"\x80"[..]] {
        assert!(vli(&mut &*bytes).is_err());
    }
    let mut check = Check::new(4).unwrap();
    check.update(b"123456789");
    assert_eq!(check.finish(), 0x995d_c9bb_df19_39fau64.to_le_bytes());
}
#[test]
fn real_writer_levels_empty_and_streaming_blocks() {
    for level in [1, 6, 9] {
        for plain in [Vec::new(), PHRASE.repeat(6000)] {
            let mut writer = XzWriter::new(Vec::new(), level, MEMORY).unwrap();
            for chunk in plain.chunks(71) {
                writer.write_all(chunk).unwrap();
            }
            let encoded = writer.finish().unwrap();
            assert_eq!(decode(&encoded, 97).unwrap(), (plain.clone(), true));
            if !plain.is_empty() {
                assert!(encoded.len() < plain.len() / 10);
            }
        }
    }
}

fn authored(fields: &[u8], raw: &[u8], plain: &[u8]) -> Vec<u8> {
    let mut header = vec![0];
    header.extend_from_slice(fields);
    while !(header.len() + 4).is_multiple_of(4) {
        header.push(0);
    }
    header[0] = ((header.len() + 4) / 4 - 1) as u8;
    header.extend_from_slice(&crc32(&header).to_le_bytes());
    let mut bytes = MAGIC.to_vec();
    bytes.extend_from_slice(&[0, 1]);
    bytes.extend_from_slice(&crc32(&[0, 1]).to_le_bytes());
    bytes.extend_from_slice(&header);
    bytes.extend_from_slice(raw);
    bytes.extend_from_slice(&[0; 3][..(4 - raw.len() % 4) % 4]);
    bytes.extend_from_slice(&crc32(plain).to_le_bytes());
    let index =
        index_bytes(&[((header.len() + raw.len() + 4) as u64, plain.len() as u64)]).unwrap();
    bytes.extend_from_slice(&index);
    let mut footer = ((index.len() / 4 - 1) as u32).to_le_bytes().to_vec();
    footer.extend_from_slice(&[0, 1]);
    bytes.extend_from_slice(&crc32(&footer).to_le_bytes());
    bytes.extend_from_slice(&footer);
    bytes.extend_from_slice(b"YZ");
    bytes
}
#[test]
fn optional_sizes_are_assertions_and_supported_grammar_is_exact() {
    let plain = PHRASE.repeat(6);
    let mut writer = xdzip_compression::Lzma2Writer::new(Vec::new(), 1, MEMORY).unwrap();
    let property = writer.property();
    writer.write_all(&plain).unwrap();
    let raw = writer.finish().unwrap();
    for flags in [0u8, 64, 128, 192] {
        let mut fields = vec![flags];
        if flags & 64 != 0 {
            append_vli(&mut fields, raw.len() as u64).unwrap();
        }
        if flags & 128 != 0 {
            append_vli(&mut fields, plain.len() as u64).unwrap();
        }
        fields.extend_from_slice(&[0x21, 1, property]);
        assert_eq!(
            decode(&authored(&fields, &raw, &plain), 1).unwrap().0,
            plain
        );
        for delta in [-1i64, 1] {
            let mut wrong = vec![flags];
            if flags & 64 != 0 {
                append_vli(&mut wrong, (raw.len() as i64 + delta) as u64).unwrap();
            }
            if flags & 128 != 0 {
                append_vli(&mut wrong, (plain.len() as i64 + delta) as u64).unwrap();
            }
            wrong.extend_from_slice(&[0x21, 1, property]);
            if flags != 0 {
                assert!(decode(&authored(&wrong, &raw, &plain), 91).is_err());
            }
        }
    }
    for fields in [
        vec![0, 0x21, 1, 41],
        vec![0, 0x21, 1, 38],
        vec![4, 0x21, 1, property],
        vec![0, 0x21, 2, property, 0],
        vec![0, 0x20, 1, property],
        vec![1, 7, 4, 1, 0, 0, 0, 0x21, 1, property],
        vec![0, 0x21, 1, property, 1],
        vec![64, 0, 0x21, 1, property],
    ] {
        assert!(decode(&authored(&fields, &raw, &plain), 99).is_err());
    }
}
#[test]
fn physical_bounds_short_reads_and_terminal_source_failures_remain_exact() {
    struct Short<R>(R);
    impl<R: Read> Read for Short<R> {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            let take = bytes.len().min(1);
            self.0.read(&mut bytes[..take])
        }
    }
    let original = include_bytes!("../../../../tests/fixtures/streams/phrase-crc64.xz");
    let framed = [original.as_slice(), b"PRIVATE OUTSIDE RANGE"].concat();
    let mut source = Cursor::new(&framed);
    let mut reader = XzReader::new(Short(&mut source), original.len() as u64, MEMORY).unwrap();
    let mut output = Vec::new();
    reader.read_to_end(&mut output).unwrap();
    assert_eq!(output, PHRASE.repeat(6));
    drop(reader);
    assert_eq!(source.position(), original.len() as u64);
    #[derive(Debug)]
    struct Fault;
    impl std::fmt::Display for Fault {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("terminal test source")
        }
    }
    impl std::error::Error for Fault {}
    struct Failure(usize);
    impl Read for Failure {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            self.0 += 1;
            assert_eq!(self.0, 1);
            Err(io::Error::new(io::ErrorKind::Interrupted, Fault))
        }
    }
    let mut reader = XzReader::new(Failure(0), 32, MEMORY).unwrap();
    let error = reader.read(&mut [0]).unwrap_err();
    assert!(error.get_ref().unwrap().downcast_ref::<Fault>().is_some());
    assert!(reader.read(&mut [0]).is_err());
}
#[test]
fn failed_writer_never_replays_or_finalizes() {
    struct Sink {
        allowed: usize,
        calls: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.calls.set(self.calls.get() + 1);
            if self.allowed == 0 {
                return Err(io::Error::other("output fault"));
            }
            let n = self.allowed.min(bytes.len());
            self.allowed -= n;
            Ok(n)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let calls = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut writer = XzWriter::new(
        Sink {
            allowed: 24,
            calls: calls.clone(),
        },
        1,
        MEMORY,
    )
    .unwrap();
    writer.write_all(&PHRASE.repeat(6000)).unwrap_err();
    let frozen = calls.get();
    assert!(writer.write(b"retry").is_err());
    assert!(writer.flush().is_err());
    assert!(writer.finish().is_err());
    assert_eq!(calls.get(), frozen);
    let untouched = std::rc::Rc::new(std::cell::Cell::new(0));
    assert!(
        XzWriter::new(
            Sink {
                allowed: 100,
                calls: untouched.clone()
            },
            1,
            8
        )
        .is_err()
    );
    assert_eq!(untouched.get(), 0);
}
#[test]
#[ignore = "requires explicit independent Python/liblzma oracle and owned output directory"]
fn independent_oracle_reads_real_writer_multiple_blocks() {
    let python = std::env::var_os("XDZIP_XZ_ORACLE_PYTHON").expect("oracle");
    let root = std::path::PathBuf::from(std::env::var_os("XDZIP_XZ_EXPORT_DIR").expect("output"));
    std::fs::create_dir_all(&root).unwrap();
    let plain = PHRASE.repeat(750000);
    for level in [1, 6, 9] {
        let mut writer = XzWriter::new(Vec::new(), level, MEMORY).unwrap();
        writer.write_all(&plain).unwrap();
        let encoded = writer.finish().unwrap();
        assert!(encoded.len() < plain.len() / 10);
        assert_eq!(decode(&encoded, 8192).unwrap().0, plain);
        let path = root.join(format!("own-level-{level}.xz"));
        std::fs::write(&path, &encoded).unwrap();
        let status = std::process::Command::new(&python).args(["-c", "import lzma,sys; assert lzma.decompress(open(sys.argv[1],'rb').read())==b'abracadabra abracadabra\\n'*750000"]).arg(path).status().unwrap();
        assert!(status.success());
    }
}
