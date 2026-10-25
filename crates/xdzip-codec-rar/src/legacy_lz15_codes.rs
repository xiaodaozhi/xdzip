//! Finite wire mappings have separate restricted-source provenance in data/.
//! The safe canonical/trie representation below is original project code.
use crate::legacy_stream::Bits;
use crate::lz_bits::Huffman;
use xdzip_codec_api::payload::DecodeError;

#[derive(Default)]
struct Node {
    children: [Option<usize>; 2],
    value: Option<u16>,
}
struct Prefix {
    nodes: Vec<Node>,
}
impl Prefix {
    fn new() -> Self {
        Self {
            nodes: vec![Node::default()],
        }
    }
    fn insert(&mut self, code: &str, value: u16) -> Result<(), DecodeError> {
        if code.is_empty() || code.len() > 15 {
            return Err(DecodeError::Integrity);
        }
        let mut at = 0;
        for byte in code.bytes() {
            if self.nodes[at].value.is_some() {
                return Err(DecodeError::Integrity);
            }
            let bit = match byte {
                b'0' => 0,
                b'1' => 1,
                _ => return Err(DecodeError::Integrity),
            };
            at = match self.nodes[at].children[bit] {
                Some(next) => next,
                None => {
                    let next = self.nodes.len();
                    if next >= 31 {
                        return Err(DecodeError::Integrity);
                    }
                    self.nodes.push(Node::default());
                    self.nodes[at].children[bit] = Some(next);
                    next
                }
            };
        }
        if self.nodes[at].value.is_some() || self.nodes[at].children.iter().any(Option::is_some) {
            return Err(DecodeError::Integrity);
        }
        self.nodes[at].value = Some(value);
        Ok(())
    }
    fn symbol(&self, input: &mut Bits<'_>) -> Result<u16, DecodeError> {
        let mut at = 0;
        for _ in 0..15 {
            at = self.nodes[at].children[input.get(1)? as usize].ok_or(DecodeError::Integrity)?;
            if let Some(value) = self.nodes[at].value {
                return Ok(value);
            }
        }
        Err(DecodeError::Integrity)
    }
}

pub(super) struct Books {
    numbers: Vec<Huffman>,
    shorts: Vec<Prefix>,
}
impl Books {
    pub fn new() -> Result<Self, DecodeError> {
        let mut lengths = [[0u8; 257]; 7];
        let mut ends = [0usize; 7];
        for line in include_str!("../data/rar15-prefix-length-runs.csv")
            .lines()
            .skip(1)
        {
            let fields = line.split(',').collect::<Vec<_>>();
            if fields.len() != 3 {
                return Err(DecodeError::Integrity);
            }
            let table = match fields[0] {
                "L1" => 0,
                "L2" => 1,
                "Hf0" => 2,
                "Hf1" => 3,
                "Hf2" => 4,
                "Hf3" => 5,
                "Hf4" => 6,
                _ => return Err(DecodeError::Integrity),
            };
            let end: usize = fields[1].parse().map_err(|_| DecodeError::Integrity)?;
            let width: u8 = fields[2].parse().map_err(|_| DecodeError::Integrity)?;
            if end <= ends[table] || end > if table < 2 { 256 } else { 257 } {
                return Err(DecodeError::Integrity);
            }
            lengths[table][ends[table]..end].fill(width);
            ends[table] = end;
        }
        let mut numbers = Vec::new();
        for (table, end) in ends.into_iter().enumerate() {
            if end != if table < 2 { 256 } else { 257 } {
                return Err(DecodeError::Integrity);
            }
            numbers.push(Huffman::new(&lengths[table][..end])?);
        }
        let mut shorts = (0..4).map(|_| Prefix::new()).collect::<Vec<_>>();
        for line in include_str!("../data/rar15-short-codewords.csv")
            .lines()
            .skip(1)
        {
            let fields = line.split(',').collect::<Vec<_>>();
            if fields.len() != 4 {
                return Err(DecodeError::Integrity);
            }
            let mode: usize = fields[0].parse().map_err(|_| DecodeError::Integrity)?;
            let bit: usize = fields[1].parse().map_err(|_| DecodeError::Integrity)?;
            let value: u16 = fields[3].parse().map_err(|_| DecodeError::Integrity)?;
            if !(1..=2).contains(&mode) || bit > 1 || value > 14 {
                return Err(DecodeError::Integrity);
            }
            shorts[(mode - 1) * 2 + bit].insert(fields[2], value)?;
        }
        Ok(Self { numbers, shorts })
    }
    pub fn number(&self, table: usize, input: &mut Bits<'_>) -> Result<u16, DecodeError> {
        self.numbers
            .get(table)
            .ok_or(DecodeError::Integrity)?
            .symbol(input)
    }
    pub fn short(&self, mode: usize, bit: bool, input: &mut Bits<'_>) -> Result<u16, DecodeError> {
        self.shorts
            .get(mode * 2 + usize::from(bit))
            .ok_or(DecodeError::Integrity)?
            .symbol(input)
    }
}
