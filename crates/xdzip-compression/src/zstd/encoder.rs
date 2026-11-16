use super::{
    FseSequenceKind, FseTable,
    length::{literal_length, match_length},
};
use crate::{
    invalid,
    match_finder::{MatchFinder, search_depth},
    stream_io::resource,
};
use std::io;
#[derive(Clone, Copy)]
struct Field {
    code: u8,
    extra: u32,
    bits: u8,
}
struct Sequence {
    fields: [Field; 3],
}
fn length_field(length: usize, matched: bool) -> io::Result<Field> {
    let max = if matched { 52 } else { 35 };
    for code in (0..=max).rev() {
        let (base, bits) = if matched {
            match_length(code)?
        } else {
            literal_length(code)?
        };
        if length >= base && length - base < (1usize << bits) {
            return Ok(Field {
                code,
                extra: (length - base) as u32,
                bits,
            });
        }
    }
    Err(invalid())
}
struct Bits {
    output: Vec<u8>,
    value: u64,
    used: u8,
    bound: usize,
}
impl Bits {
    fn new(bound: usize) -> io::Result<Self> {
        let mut output = Vec::new();
        output.try_reserve_exact(bound).map_err(|_| resource())?;
        Ok(Self {
            output,
            value: 0,
            used: 0,
            bound,
        })
    }
    fn put(&mut self, value: u32, width: u8) -> io::Result<()> {
        if width > 32 || u64::from(value) >= (1u64 << width) {
            return Err(invalid());
        }
        self.value |= u64::from(value) << self.used;
        self.used += width;
        while self.used >= 8 {
            if self.output.len() == self.bound {
                return Err(resource());
            }
            self.output.push(self.value as u8);
            self.value >>= 8;
            self.used -= 8;
        }
        Ok(())
    }
    fn finish(mut self) -> io::Result<Vec<u8>> {
        self.put(1, 1)?;
        if self.used != 0 {
            if self.output.len() == self.bound {
                return Err(resource());
            }
            self.output.push(self.value as u8);
        }
        Ok(self.output)
    }
}
/// Encode one independent Zstandard block with real LZ matches and predefined FSE.
/// Raw literals and explicit non-repeat offsets keep frame history unnecessary.
/// `None` asks the frame writer for a raw block when this encoding cannot reduce size.
pub fn encode_zstd_block(
    input: &[u8],
    level: u8,
    memory_limit: u64,
) -> io::Result<Option<Vec<u8>>> {
    let depth = search_depth(level)?;
    if input.len() > 128 << 10 {
        return Err(invalid());
    }
    let sequence_bound = input.len() / 4 + 1;
    let wire_bound = input.len() * 8 + 1024;
    let needed = MatchFinder::MEMORY
        + sequence_bound * std::mem::size_of::<Sequence>()
        + input.len()
        + 2 * wire_bound
        + 65536;
    if needed as u64 > memory_limit {
        return Err(resource());
    }
    let mut finder = MatchFinder::new(depth)?;
    let mut literals = Vec::new();
    literals
        .try_reserve_exact(input.len())
        .map_err(|_| resource())?;
    let mut sequences = Vec::new();
    sequences
        .try_reserve_exact(sequence_bound)
        .map_err(|_| resource())?;
    let mut position = 0;
    let mut anchor = 0;
    while position + 4 <= input.len() {
        let (offset, length) = finder.find(input, position, input.len())?;
        if length < 4 {
            position += 1;
            continue;
        }
        literals.extend_from_slice(&input[anchor..position]);
        let value = offset as u32 + 3;
        let code = (31 - value.leading_zeros()) as u8;
        sequences.push(Sequence {
            fields: [
                length_field(position - anchor, false)?,
                Field {
                    code,
                    extra: value - (1u32 << code),
                    bits: code,
                },
                length_field(length, true)?,
            ],
        });
        finder.seed(input, position + 1, position + length)?;
        position += length;
        anchor = position;
    }
    if sequences.is_empty() {
        return Ok(None);
    }
    literals.extend_from_slice(&input[anchor..]);
    let mut output = Vec::new();
    output
        .try_reserve_exact(wire_bound)
        .map_err(|_| resource())?;
    let header = if literals.len() < 32 {
        (literals.len() << 3, 1)
    } else if literals.len() < 4096 {
        ((literals.len() << 4) | 4, 2)
    } else {
        ((literals.len() << 4) | 12, 3)
    };
    output.extend_from_slice(&(header.0 as u32).to_le_bytes()[..header.1]);
    output.extend_from_slice(&literals);
    let count = sequences.len();
    if count < 128 {
        output.push(count as u8);
    } else if count < 0x7f00 {
        output.extend([(128 + (count >> 8)) as u8, count as u8]);
    } else {
        output.push(255);
        output.extend_from_slice(&((count - 0x7f00) as u16).to_le_bytes());
    }
    output.push(0); // All three normative predefined distributions.
    let tables = [
        FseTable::predefined(FseSequenceKind::LiteralLength)?,
        FseTable::predefined(FseSequenceKind::Offset)?,
        FseTable::predefined(FseSequenceKind::MatchLength)?,
    ];
    let last = sequences.last().expect("nonempty matches");
    let mut states = [
        tables[0].symbol_state(last.fields[0].code)?,
        tables[1].symbol_state(last.fields[1].code)?,
        tables[2].symbol_state(last.fields[2].code)?,
    ];
    let mut bits = Bits::new(wire_bound)?;
    for (index, sequence) in sequences.iter().enumerate().rev() {
        if index + 1 != sequences.len() {
            for state in [1, 2, 0] {
                let (next, value, width) = tables[state]
                    .encoding_transition(sequence.fields[state].code, states[state])?;
                bits.put(value, width)?;
                states[state] = next;
            }
        }
        for index in [0, 2, 1] {
            let field = sequence.fields[index];
            bits.put(field.extra, field.bits)?;
        }
    }
    for index in [2, 1, 0] {
        bits.put(states[index] as u32, tables[index].state_bits())?;
    }
    output.extend(bits.finish()?);
    if output.len() >= input.len() || output.len() > 128 << 10 {
        Ok(None)
    } else {
        Ok(Some(output))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::zstd::ZstdBlockDecoder;
    fn pattern(size: usize, repeated: bool) -> Vec<u8> {
        (0..size)
            .map(|i| {
                if repeated {
                    (i % 7) as u8
                } else {
                    ((i * 73) ^ (i >> 8)) as u8
                }
            })
            .collect()
    }
    #[test]
    fn real_sequence_blocks_roundtrip_all_levels_lengths_and_literal_header_forms() {
        for level in [1, 6, 9] {
            for size in [
                0, 1, 3, 4, 5, 31, 32, 4095, 4096, 65535, 65536, 65791, 65792, 131072,
            ] {
                for repeated in [false, true] {
                    let plain = pattern(size, repeated);
                    let encoded = encode_zstd_block(&plain, level, 8 << 20).unwrap();
                    if repeated && size > 100 {
                        assert!(encoded.as_ref().unwrap().len() < size / 4);
                    }
                    let mut decoder = ZstdBlockDecoder::new(128 << 10, 8 << 20).unwrap();
                    let decoded = if let Some(encoded) = encoded {
                        decoder.decode_compressed(&encoded).unwrap()
                    } else {
                        decoder.raw(&plain).unwrap()
                    };
                    assert_eq!(decoded, plain, "level {level}, length {size}");
                }
            }
        }
        assert!(encode_zstd_block(b"x", 0, 8 << 20).is_err());
        assert!(encode_zstd_block(b"x", 1, 10).is_err());
        assert!(encode_zstd_block(&vec![0; 131073], 1, 8 << 20).is_err());
    }
    fn envelope(plain: &[u8], level: u8) -> Vec<u8> {
        let mut bytes = 0xfd2fb528u32.to_le_bytes().to_vec();
        let (flag, size, width) = if plain.len() < 256 {
            (0, plain.len(), 1)
        } else if plain.len() < 65792 {
            (64, plain.len() - 256, 2)
        } else {
            (128, plain.len(), 4)
        };
        bytes.push(flag | 32 | 4);
        bytes.extend_from_slice(&(size as u64).to_le_bytes()[..width]);
        let compressed = encode_zstd_block(plain, level, 8 << 20).unwrap();
        let kind = if compressed.is_some() { 2 } else { 0 };
        let payload = compressed.as_deref().unwrap_or(plain);
        let header = 1 | (kind << 1) | ((payload.len() as u32) << 3);
        bytes.extend_from_slice(&header.to_le_bytes()[..3]);
        bytes.extend_from_slice(payload);
        bytes.extend_from_slice(&(crate::xxhash64(plain) as u32).to_le_bytes());
        bytes
    }
    #[test]
    #[ignore = "requires explicit independent Python/Zstandard oracle and owned output directory"]
    fn independent_reader_decodes_every_real_block_encoder_level() {
        let python = std::env::var_os("XDZIP_ZSTD_REFERENCE_PYTHON").expect("reference Python");
        let root = std::path::PathBuf::from(
            std::env::var_os("XDZIP_ZSTD_EXPORT_DIR").expect("owned output"),
        );
        std::fs::create_dir_all(&root).unwrap();
        for level in [1, 6, 9] {
            for (index, plain) in [
                vec![],
                vec![42],
                pattern(65535, false),
                pattern(131072, true),
                pattern(131072, false),
            ]
            .into_iter()
            .enumerate()
            {
                let encoded = envelope(&plain, level);
                let source = root.join(format!("block-{level}-{index}.zst"));
                let expected = root.join(format!("block-{level}-{index}.bin"));
                std::fs::write(&source, encoded).unwrap();
                std::fs::write(&expected, plain).unwrap();
                let status = std::process::Command::new(&python).args(["-c","import compression.zstd as z,sys; assert z.zstd_version_info==(1,5,7); assert z.decompress(open(sys.argv[1],'rb').read())==open(sys.argv[2],'rb').read()"]).arg(source).arg(expected).status().unwrap();
                assert!(status.success(), "level {level}, pattern {index}");
            }
        }
    }
}
