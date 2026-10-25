use super::*;
use crate::DeflateReader;

fn literal_codes() -> Vec<(u32, u8)> {
    let mut lengths = [8; 288];
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    wire_codes(&lengths)
}
fn prefix(writer: &mut BitOutput<Vec<u8>>, data: &[u8]) {
    for chunk in data.chunks(65535) {
        writer.block(chunk, false, 0).unwrap();
    }
}
fn sample(length: usize, distance: usize, dynamic: bool) -> (Vec<u8>, Vec<u8>) {
    let data: Vec<u8> = (0..distance)
        .map(|n| ((n * 97 + n / 13) % 256) as u8)
        .collect();
    let mut writer = BitOutput::new(Vec::new());
    prefix(&mut writer, &data);
    if dynamic {
        assert_eq!(distance, 65536);
        writer.put(5, 3).unwrap(); // final dynamic block
        writer.put(29, 5).unwrap();
        writer.put(31, 5).unwrap();
        writer.put(14, 4).unwrap();
        // Complete code-length alphabet containing only 0 and 1.
        for index in 0..18 {
            writer.put(u32::from(index == 3 || index == 17), 3).unwrap();
        }
        for symbol in 0..286 {
            writer
                .put(u32::from(symbol == 256 || symbol == 285), 1)
                .unwrap();
        }
        for symbol in 0..32 {
            writer.put(u32::from(symbol == 31), 1).unwrap();
        }
        writer.put(1, 1).unwrap();
        writer.put((length - 3) as u32, 16).unwrap();
        writer.put(0, 1).unwrap();
        writer.put(16383, 14).unwrap();
        writer.put(0, 1).unwrap();
    } else {
        writer.put(3, 3).unwrap();
        let literals = literal_codes();
        writer.code(&literals, 285).unwrap();
        writer.put((length - 3) as u32, 16).unwrap();
        let (code, extra, bits) = if distance > 49152 {
            (31, distance - 49153, 14)
        } else if distance > 32768 {
            (30, distance - 32769, 14)
        } else {
            let code = (0..30).rev().find(|n| DISTANCES[*n] <= distance).unwrap();
            (code, distance - DISTANCES[code], DISTANCE_BITS[code])
        };
        writer.code(&wire_codes(&[5; 32]), code).unwrap();
        writer.put(extra as u32, bits).unwrap();
        writer.code(&literals, 256).unwrap();
    }
    writer.align().unwrap();
    let mut expected = data;
    for _ in 0..length {
        expected.push(expected[expected.len() - distance]);
    }
    (writer.sink, expected)
}
fn decode(bytes: &[u8], chunk: usize) -> io::Result<Vec<u8>> {
    let mut reader = DeflateReader::new_deflate64(bytes, bytes.len() as u64);
    let mut actual = Vec::new();
    let mut buffer = vec![0; chunk];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        actual.extend_from_slice(&buffer[..n]);
    }
    assert_eq!(reader.consumed_bytes(), bytes.len() as u64);
    Ok(actual)
}

#[test]
fn maximum_matches_distances_and_dynamic_32_symbol_alphabet_share_the_decoder() {
    for length in [3, 258, 65538] {
        for distance in [1, 32768, 32769, 49152, 49153, 65536] {
            let (packed, expected) = sample(length, distance, false);
            for chunk in [1, 113, 8192] {
                assert_eq!(decode(&packed, chunk).unwrap(), expected);
            }
        }
    }
    let (packed, expected) = sample(65538, 65536, true);
    for chunk in [1, 8192] {
        assert_eq!(decode(&packed, chunk).unwrap(), expected);
    }
}

#[test]
fn ordinary_deflate_mode_does_not_gain_extended_codes_or_length_semantics() {
    for distance in [1, 32769, 65536] {
        let (packed, expected) = sample(3, distance, false);
        let mut reader = DeflateReader::new(packed.as_slice(), packed.len() as u64);
        let mut ordinary = Vec::new();
        let result = reader.by_ref().take(1024 * 1024).read_to_end(&mut ordinary);
        assert!(result.is_err() || ordinary != expected);
    }
    let mut writer = crate::DeflateWriter::new(Vec::new(), 6).unwrap();
    writer
        .write_all(&b"normal RFC1951 history".repeat(100))
        .unwrap();
    let packed = writer.finish().unwrap();
    let mut reader = DeflateReader::new(packed.as_slice(), packed.len() as u64);
    let mut actual = Vec::new();
    reader.read_to_end(&mut actual).unwrap();
    assert_eq!(actual, b"normal RFC1951 history".repeat(100));
}

#[test]
fn exact_termination_truncation_and_reserved_codes_remain_required() {
    let (packed, _) = sample(258, 1, false);
    for n in 0..packed.len() {
        assert!(decode(&packed[..n], 8192).is_err());
    }
    for tail in [0, 255] {
        let mut bad = packed.clone();
        bad.push(tail);
        assert!(decode(&bad, 8192).is_err());
    }
    for symbol in [286, 287] {
        let mut writer = BitOutput::new(Vec::new());
        writer.put(3, 3).unwrap();
        writer.code(&literal_codes(), symbol).unwrap();
        writer.align().unwrap();
        assert!(decode(&writer.sink, 8192).is_err());
    }
    let mut writer = BitOutput::new(Vec::new());
    writer.put(3, 3).unwrap();
    writer.code(&literal_codes(), 285).unwrap();
    writer.put(0, 16).unwrap();
    writer.code(&wire_codes(&[5; 32]), 31).unwrap();
    writer.put(16383, 14).unwrap();
    writer.code(&literal_codes(), 256).unwrap();
    writer.align().unwrap();
    assert!(decode(&writer.sink, 8192).is_err());
}

#[derive(Debug)]
struct SourceChanged;
impl std::fmt::Display for SourceChanged {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("source_changed")
    }
}
impl std::error::Error for SourceChanged {}
struct Fault<'a> {
    bytes: &'a [u8],
    at: usize,
    after: usize,
    kind: io::ErrorKind,
    emitted: bool,
    bare: bool,
}
impl Read for Fault<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.at >= self.after && !self.emitted {
            self.emitted = true;
            return Err(if self.bare {
                self.kind.into()
            } else {
                io::Error::new(self.kind, SourceChanged)
            });
        }
        let before = if self.emitted {
            usize::MAX
        } else {
            self.after - self.at
        };
        let count = output.len().min(self.bytes.len() - self.at).min(before);
        output[..count].copy_from_slice(&self.bytes[self.at..self.at + count]);
        self.at += count;
        Ok(count)
    }
}
#[test]
fn shared_header_and_stored_reads_preserve_typed_interruption_and_only_retry_bare() {
    let bytes = [1, 1, 0, 254, 255, b'x'];
    for wide in [false, true] {
        for after in [0, 5] {
            for kind in [io::ErrorKind::Other, io::ErrorKind::Interrupted] {
                let source = Fault {
                    bytes: &bytes,
                    at: 0,
                    after,
                    kind,
                    emitted: false,
                    bare: false,
                };
                let mut reader = if wide {
                    DeflateReader::new_deflate64(source, 6)
                } else {
                    DeflateReader::new(source, 6)
                };
                let error = reader.read_to_end(&mut Vec::new()).unwrap_err();
                assert!(
                    error
                        .get_ref()
                        .and_then(|e| e.downcast_ref::<SourceChanged>())
                        .is_some()
                );
                assert!(reader.read(&mut [0]).is_err());
            }
        }
    }
    for after in [0, 5] {
        let source = Fault {
            bytes: &bytes,
            at: 0,
            after,
            kind: io::ErrorKind::Interrupted,
            emitted: false,
            bare: true,
        };
        let mut reader = DeflateReader::new_deflate64(source, 6);
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        assert_eq!(actual, b"x");
    }
}

#[test]
#[ignore = "requires an owned export directory and independent official-reader verification"]
fn export_owned_deflate64_vectors() {
    let root = std::path::PathBuf::from(
        std::env::var_os("XDZIP_DEFLATE64_OUTPUT").expect("owned evidence directory required"),
    );
    std::fs::create_dir_all(&root).unwrap();
    for (index, (length, distance, dynamic)) in [
        (3, 1, false),
        (258, 32768, false),
        (65538, 32769, false),
        (65538, 49152, false),
        (65538, 49153, false),
        (65538, 65536, false),
        (65538, 65536, true),
    ]
    .into_iter()
    .enumerate()
    {
        let (packed, expected) = sample(length, distance, dynamic);
        assert_eq!(decode(&packed, 113).unwrap(), expected);
        std::fs::write(root.join(format!("{index}.raw")), packed).unwrap();
        std::fs::write(root.join(format!("{index}.known")), expected).unwrap();
    }
}
