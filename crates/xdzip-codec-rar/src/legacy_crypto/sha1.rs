//! Private erasable SHA-1 for the legacy RAR3 compatibility KDF only.
//! Independently organized from FIPS 180-4 sections 4, 5 and 6.1 mathematics.

use zeroize::{Zeroize, Zeroizing};

pub(super) fn schedule(block: &[u8]) -> Zeroizing<[u32; 80]> {
    debug_assert_eq!(block.len(), 64);
    let mut words = Zeroizing::new([0u32; 80]);
    for (word, bytes) in words[..16].iter_mut().zip(block.as_chunks::<4>().0) {
        *word = u32::from_be_bytes(*bytes);
    }
    for i in 16..80 {
        words[i] = (words[i - 3] ^ words[i - 8] ^ words[i - 14] ^ words[i - 16]).rotate_left(1);
    }
    words
}

#[derive(Clone)]
pub(super) struct Sha1 {
    state: [u32; 5],
    buffer: [u8; 64],
    filled: usize,
    bytes: u64,
}
impl Sha1 {
    pub fn new() -> Self {
        Self {
            state: [0x67452301, 0xefcdab89, 0x98badcfe, 0x10325476, 0xc3d2e1f0],
            buffer: [0; 64],
            filled: 0,
            bytes: 0,
        }
    }
    pub fn update(&mut self, mut bytes: &[u8]) {
        self.bytes = self.bytes.wrapping_add(bytes.len() as u64);
        while !bytes.is_empty() {
            let count = bytes.len().min(64 - self.filled);
            self.buffer[self.filled..self.filled + count].copy_from_slice(&bytes[..count]);
            self.filled += count;
            bytes = &bytes[count..];
            if self.filled == 64 {
                self.compress();
                self.buffer.zeroize();
                self.filled = 0;
            }
        }
    }
    fn compress(&mut self) {
        let words = schedule(&self.buffer);
        let mut working = Zeroizing::new(self.state);
        for (round, &word) in words.iter().enumerate() {
            let [a, b, c, d, e] = *working;
            let (function, constant) = match round {
                0..20 => ((b & c) | (!b & d), 0x5a827999u32),
                20..40 => (b ^ c ^ d, 0x6ed9eba1),
                40..60 => ((b & c) | (b & d) | (c & d), 0x8f1bbcdc),
                _ => (b ^ c ^ d, 0xca62c1d6),
            };
            *working = [
                a.rotate_left(5)
                    .wrapping_add(function)
                    .wrapping_add(e)
                    .wrapping_add(constant)
                    .wrapping_add(word),
                a,
                b.rotate_left(30),
                c,
                d,
            ];
        }
        for (state, &word) in self.state.iter_mut().zip(working.iter()) {
            *state = state.wrapping_add(word);
        }
    }
    pub fn finish(mut self) -> Zeroizing<[u8; 20]> {
        let bits = self.bytes.wrapping_mul(8);
        self.update(&[0x80]);
        let zeros = (120 - self.filled) % 64;
        self.update(&[0; 64][..zeros]);
        self.update(&bits.to_be_bytes());
        let mut result = Zeroizing::new([0; 20]);
        for (bytes, word) in result.as_chunks_mut::<4>().0.iter_mut().zip(&self.state) {
            bytes.copy_from_slice(&word.to_be_bytes());
        }
        result
    }
}
impl Drop for Sha1 {
    fn drop(&mut self) {
        self.state.zeroize();
        self.buffer.zeroize();
        self.filled.zeroize();
        self.bytes.zeroize();
    }
}
