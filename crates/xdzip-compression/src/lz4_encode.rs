//! Project-owned bounded greedy parser for standard LZ4 block sequences.
use crate::{
    invalid,
    match_finder::{MatchFinder, search_depth},
    stream_io::resource,
};
use std::io;
const MAX_BLOCK: usize = 4 * 1024 * 1024;
fn extension(output: &mut Vec<u8>, mut length: usize) {
    while length >= 255 {
        output.push(255);
        length -= 255;
    }
    output.push(length as u8);
}
fn sequence(output: &mut Vec<u8>, literals: &[u8], matched: Option<(usize, usize)>) {
    let length = matched.map_or(0, |(_, length)| length - 4);
    output.push(((literals.len().min(15) as u8) << 4) | length.min(15) as u8);
    if literals.len() >= 15 {
        extension(output, literals.len() - 15);
    }
    output.extend_from_slice(literals);
    if let Some((offset, _)) = matched {
        output.extend_from_slice(&(offset as u16).to_le_bytes());
        if length >= 15 {
            extension(output, length - 15);
        }
    }
}
/// Compress an independent block of at most 4 MiB using real LZ4 matches.
/// Levels 1/6/9 bound the candidate search to 1/16/64 positions. The caller may
/// store the original block when this encoded representation does not reduce it.
pub fn encode_lz4_block(input: &[u8], level: u8, memory_limit: u64) -> io::Result<Vec<u8>> {
    let depth = search_depth(level)?;
    if input.len() > MAX_BLOCK {
        return Err(invalid());
    }
    let bound = input.len() + input.len() / 255 + 16;
    if (MatchFinder::MEMORY + bound + 16384) as u64 > memory_limit {
        return Err(resource());
    }
    let mut output = Vec::new();
    output.try_reserve_exact(bound).map_err(|_| resource())?;
    let mut finder = MatchFinder::new(depth)?;
    let mut anchor = 0;
    let mut position = 0;
    while position + 12 <= input.len() {
        let best = finder.find(input, position, input.len() - 5)?;
        if best.1 < 4 {
            position += 1;
            continue;
        }
        sequence(&mut output, &input[anchor..position], Some(best));
        finder.seed(input, position + 1, position + best.1)?;
        position += best.1;
        anchor = position;
    }
    sequence(&mut output, &input[anchor..], None);
    if output.len() > bound {
        return Err(invalid());
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Lz4BlockReader;
    use std::io::{Cursor, Read};
    #[test]
    fn real_matches_short_inputs_long_lengths_and_search_levels_roundtrip() {
        for level in [1, 6, 9] {
            for length in [0, 1, 4, 5, 12, 13, 17, 65535, 65536, 65537, 200000] {
                for pattern in [false, true] {
                    let plain: Vec<u8> = (0..length)
                        .map(|i| {
                            if pattern {
                                (i % 7) as u8
                            } else {
                                ((i * 73) ^ (i >> 8)) as u8
                            }
                        })
                        .collect();
                    let encoded = encode_lz4_block(&plain, level, 8 << 20).unwrap();
                    if pattern && length > 100 {
                        assert!(encoded.len() < plain.len() / 3);
                    }
                    let mut reader = Lz4BlockReader::new(
                        Cursor::new(&encoded),
                        encoded.len() as u64,
                        plain.len() as u64,
                        &[],
                        1 << 20,
                    )
                    .unwrap();
                    let mut actual = Vec::new();
                    reader.read_to_end(&mut actual).unwrap();
                    assert_eq!(actual, plain, "level {level}, size {length}");
                }
            }
        }
        assert_eq!(encode_lz4_block(b"", 1, 1 << 20).unwrap(), [0]);
        assert!(encode_lz4_block(b"short", 1, 10).is_err());
        assert!(encode_lz4_block(b"short", 0, 1 << 20).is_err());
    }
}
