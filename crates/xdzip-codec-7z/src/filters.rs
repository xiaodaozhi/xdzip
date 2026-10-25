//! Independent 7z byte filters, from documented mathematical wire behavior.
//! A caller retains the unprocessed suffix between calls and verifies node CRCs.
use xdzip_codec_api::Cancellation;
use xdzip_codec_api::payload::DecodeError;

#[derive(Clone, Copy)]
enum Kind {
    X86,
    Ppc,
    Ia64,
    Arm,
    Thumb,
    Sparc,
    Arm64,
    Riscv,
    Delta(usize),
    Swap(usize),
}

/// Stateful, allocation-free single-input filter. Position wraps modulo 2^32.
pub struct ByteFilter {
    kind: Kind,
    position: u32,
    history: [u8; 256],
    cursor: usize,
    x86: u8,
    ended: bool,
    failed: bool,
}
impl ByteFilter {
    /// Validate the 7z method's own property grammar before processing bytes.
    pub fn new(method: &[u8], properties: &[u8]) -> Result<Self, DecodeError> {
        let kind = match method {
            [3, 3, 1, 3] => Kind::X86,
            [3, 3, 2, 5] => Kind::Ppc,
            [3, 3, 4, 1] => Kind::Ia64,
            [3, 3, 5, 1] => Kind::Arm,
            [3, 3, 7, 1] => Kind::Thumb,
            [3, 3, 8, 5] => Kind::Sparc,
            [10] => Kind::Arm64,
            [11] => Kind::Riscv,
            [3] if properties.len() == 1 => Kind::Delta(usize::from(properties[0]) + 1),
            [3] => return Err(DecodeError::Unsupported),
            [2, 3, 2] => Kind::Swap(2),
            [2, 3, 4] => Kind::Swap(4),
            _ => return Err(DecodeError::Unsupported),
        };
        let position = match kind {
            Kind::Delta(_) => 0,
            Kind::Arm64 | Kind::Riscv if properties.len() == 4 => {
                let position = u32::from_le_bytes(properties.try_into().unwrap());
                let alignment = if matches!(kind, Kind::Arm64) { 4 } else { 2 };
                if !position.is_multiple_of(alignment) {
                    return Err(DecodeError::Unsupported);
                }
                position
            }
            _ if properties.is_empty() => 0,
            _ => return Err(DecodeError::Unsupported),
        };
        Ok(Self {
            kind,
            position,
            history: [0; 256],
            cursor: 0,
            x86: 0,
            ended: false,
            failed: false,
        })
    }
    /// Transform a complete prefix and return its byte count. Retain the suffix
    /// verbatim for the next call; only true EOF may commit an incomplete tail.
    /// An error poisons the instance, so partially transformed input is unusable.
    pub fn transform(
        &mut self,
        bytes: &mut [u8],
        eof: bool,
        control: &Cancellation,
    ) -> Result<usize, DecodeError> {
        if self.failed || self.ended {
            return Err(DecodeError::Integrity);
        }
        let result = self.run(bytes, eof, control);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    fn run(
        &mut self,
        bytes: &mut [u8],
        eof: bool,
        control: &Cancellation,
    ) -> Result<usize, DecodeError> {
        check(control)?;
        let mut at = 0;
        let mut next_check = 0;
        while at < bytes.len() {
            if at >= next_check {
                check(control)?;
                next_check = at.saturating_add(1024);
            }
            let position = self.position.wrapping_add(at as u32);
            let remaining = &mut bytes[at..];
            let step = match self.kind {
                Kind::Delta(distance) => {
                    remaining[0] = remaining[0].wrapping_add(self.history[self.cursor]);
                    self.history[self.cursor] = remaining[0];
                    self.cursor = (self.cursor + 1) % distance;
                    1
                }
                Kind::Swap(width) => {
                    if remaining.len() < width {
                        break;
                    }
                    remaining[..width].reverse();
                    width
                }
                Kind::X86 => {
                    if remaining.len() < 5 {
                        break;
                    }
                    self.x86(remaining, position)
                }
                Kind::Ia64 => {
                    if remaining.len() < 16 {
                        break;
                    }
                    ia64(remaining, position);
                    16
                }
                Kind::Thumb => {
                    if remaining.len() < 4 {
                        break;
                    }
                    thumb(remaining, position)
                }
                Kind::Riscv => {
                    if remaining.len() < 8 {
                        break;
                    }
                    riscv(remaining, position)
                }
                _ => {
                    if remaining.len() < 4 {
                        break;
                    }
                    word_filter(self.kind, remaining, position);
                    4
                }
            };
            at += step;
        }
        if eof {
            at = bytes.len();
            self.ended = true;
        }
        self.position = self.position.wrapping_add(at as u32);
        check(control)?;
        Ok(at)
    }
    fn x86(&mut self, bytes: &mut [u8], position: u32) -> usize {
        if bytes[0] != 0xe8 && bytes[0] != 0xe9 {
            self.x86 >>= 1;
            return 1;
        }
        let history = self.x86;
        let ambiguous = history > 4
            || history == 3
            || (history != 0 && canonical_byte(bytes[1 + usize::from(history / 2)]));
        if ambiguous || !canonical_byte(bytes[4]) {
            self.x86 = (history / 2) | 4;
            return 1;
        }
        let original = u32::from_le_bytes(bytes[1..5].try_into().unwrap());
        let base = position.wrapping_add(5);
        let mut address = original.wrapping_add(1 << 24).wrapping_sub(base);
        if history != 0 {
            let shift = 8 * u32::from(history / 2);
            if canonical_byte((address >> shift) as u8) {
                address = (address ^ ((1 << (shift + 8)) - 1)).wrapping_sub(base);
            }
        }
        address = (address & 0x01ff_ffff).wrapping_sub(1 << 24);
        bytes[1..5].copy_from_slice(&address.to_le_bytes());
        self.x86 = 0;
        5
    }
}
fn canonical_byte(byte: u8) -> bool {
    byte == 0 || byte == 255
}
fn check(control: &Cancellation) -> Result<(), DecodeError> {
    control.check().map_err(|_| DecodeError::Cancelled)
}
fn word_filter(kind: Kind, bytes: &mut [u8], position: u32) {
    let be = matches!(kind, Kind::Ppc | Kind::Sparc);
    let raw: [u8; 4] = bytes[..4].try_into().unwrap();
    let word = if be {
        u32::from_be_bytes(raw)
    } else {
        u32::from_le_bytes(raw)
    };
    let output = match kind {
        Kind::Arm if word >> 24 == 0xeb => {
            0xeb00_0000 | word.wrapping_sub(position.wrapping_add(8) >> 2) & 0x00ff_ffff
        }
        Kind::Ppc if word & 0xfc00_0003 == 0x4800_0001 => {
            0x4800_0000 | word.wrapping_sub(position) & 0x03ff_ffff
        }
        Kind::Sparc if matches!(word & 0xffc0_0000, 0x4000_0000 | 0x7fc0_0000) => {
            let value = word.wrapping_sub(position >> 2);
            0x4000_0000
                | value & 0x003f_ffff
                | if value & 0x0040_0000 != 0 {
                    0x3fc0_0000
                } else {
                    0
                }
        }
        Kind::Arm64 if word & 0xfc00_0000 == 0x9400_0000 => {
            0x9400_0000 | word.wrapping_sub(position >> 2) & 0x03ff_ffff
        }
        Kind::Arm64 if word & 0x9f00_0000 == 0x9000_0000 => {
            let immediate = (word >> 5 & 0x7ffff) << 2 | word >> 29 & 3;
            let signed = ((immediate << 11) as i32) >> 11;
            if (-131072..=131071).contains(&signed) {
                let mut value = immediate.wrapping_sub(position >> 12) & 0x3ffff;
                if value & 0x20000 != 0 {
                    value |= 0x1c0000;
                }
                word & !0x60ff_ffe0 | (value & 3) << 29 | (value >> 2) << 5
            } else {
                word
            }
        }
        _ => word,
    };
    bytes[..4].copy_from_slice(&if be {
        output.to_be_bytes()
    } else {
        output.to_le_bytes()
    });
}
fn thumb(bytes: &mut [u8], position: u32) -> usize {
    let first = u16::from_le_bytes(bytes[..2].try_into().unwrap());
    let second = u16::from_le_bytes(bytes[2..4].try_into().unwrap());
    if first & 0xf800 != 0xf000 || second & 0xf800 != 0xf800 {
        return 2;
    }
    let value = ((u32::from(first & 0x7ff) << 11) | u32::from(second & 0x7ff))
        .wrapping_sub(position.wrapping_add(4) >> 1)
        & 0x3f_ffff;
    bytes[..2].copy_from_slice(&(0xf000 | (value >> 11) as u16).to_le_bytes());
    bytes[2..4].copy_from_slice(&(0xf800 | (value & 0x7ff) as u16).to_le_bytes());
    4
}
fn ia64(bytes: &mut [u8], position: u32) {
    let mut bundle = u128::from_le_bytes(bytes[..16].try_into().unwrap());
    let slots = match bundle & 31 {
        0x10 | 0x11 | 0x18 | 0x19 | 0x1c | 0x1d => 4,
        0x12 | 0x13 => 6,
        0x16 | 0x17 => 7,
        _ => 0,
    };
    for slot in 0..3 {
        let shift = 5 + slot * 41;
        let value = bundle >> shift & ((1u128 << 41) - 1);
        if slots & (1 << slot) == 0 || value >> 37 != 5 || value >> 9 & 7 != 0 {
            continue;
        }
        let immediate = ((value >> 13 & 0xfffff) | (value >> 36 & 1) << 20) as u32;
        let decoded = immediate.wrapping_sub(position >> 4) & 0x1f_ffff;
        let mask = ((0xfffffu128 << 13) | (1u128 << 36)) << shift;
        let fields =
            (u128::from(decoded & 0xfffff) << 13 | u128::from(decoded >> 20) << 36) << shift;
        bundle = bundle & !mask | fields;
    }
    bytes[..16].copy_from_slice(&bundle.to_le_bytes());
}
fn riscv(bytes: &mut [u8], position: u32) -> usize {
    let first = u32::from_le_bytes(bytes[..4].try_into().unwrap());
    let rd = first >> 7 & 31;
    match first & 0x7f {
        0x6f if rd == 1 || rd == 5 => {
            let address = (first >> 12 & 15) << 17 | (first >> 16 & 255) << 9 | (first >> 24) << 1;
            let value = address.wrapping_sub(position);
            let word = first & 0xfff
                | (value & 0x10_0000) << 11
                | (value & 0x7fe) << 20
                | (value & 0x800) << 9
                | value & 0xff000;
            bytes[..4].copy_from_slice(&word.to_le_bytes());
            4
        }
        0x17 if rd != 0 && rd != 2 => {
            let second = u32::from_le_bytes(bytes[4..8].try_into().unwrap());
            if second & 3 == 3 && second >> 15 & 31 == rd {
                bytes[..4].copy_from_slice(&((second << 12) | 0x117).to_le_bytes());
                bytes[4..8]
                    .copy_from_slice(&((first & 0xffff_f000) | (second >> 20)).to_le_bytes());
                8
            } else {
                6
            }
        }
        0x17 => {
            let register = first >> 27;
            if first & 0x3fff == 0x3117 && register != 0 && register != 2 {
                let address = u32::from_be_bytes(bytes[4..8].try_into().unwrap());
                let value = address.wrapping_sub(position);
                let word = value.wrapping_add(0x800) & 0xffff_f000 | register << 7 | 0x17;
                bytes[..4].copy_from_slice(&word.to_le_bytes());
                bytes[4..8].copy_from_slice(&((first >> 12) | (value << 20)).to_le_bytes());
                8
            } else {
                4
            }
        }
        _ => 2,
    }
}

#[cfg(test)]
#[path = "filters_tests.rs"]
mod tests;
