//! Fixed-storage, seed-zero XXH32 from the published mathematical specification.
//! This detects accidental corruption; it is not a cryptographic authenticator.
const P1: u32 = 0x9e37_79b1;
const P2: u32 = 0x85eb_ca77;
const P3: u32 = 0xc2b2_ae3d;
const P4: u32 = 0x27d4_eb2f;
const P5: u32 = 0x1656_67b1;
fn word(bytes: &[u8]) -> u32 {
    u32::from_le_bytes(bytes.try_into().expect("four-byte lane"))
}
/// Incremental seed-zero XXH32 with at most fifteen pending bytes.
#[derive(Clone)]
pub struct Xxh32 {
    lanes: [u32; 4],
    tail: [u8; 16],
    used: usize,
    length: u32,
    striped: bool,
}
impl Default for Xxh32 {
    fn default() -> Self {
        Self::new()
    }
}
impl Xxh32 {
    /// Start an empty seed-zero digest.
    pub fn new() -> Self {
        Self {
            lanes: [P1.wrapping_add(P2), P2, 0, 0u32.wrapping_sub(P1)],
            tail: [0; 16],
            used: 0,
            length: 0,
            striped: false,
        }
    }
    fn stripe(&mut self, bytes: &[u8]) {
        for (lane, bytes) in self.lanes.iter_mut().zip(bytes.as_chunks::<4>().0) {
            *lane = lane
                .wrapping_add(word(bytes).wrapping_mul(P2))
                .rotate_left(13)
                .wrapping_mul(P1);
        }
        self.striped = true;
    }
    /// Include input in physical order; lengths intentionally wrap modulo 2^32.
    pub fn update(&mut self, mut bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u32);
        if self.used != 0 {
            let count = bytes.len().min(16 - self.used);
            self.tail[self.used..self.used + count].copy_from_slice(&bytes[..count]);
            self.used += count;
            bytes = &bytes[count..];
            if self.used != 16 {
                return;
            }
            let stripe = self.tail;
            self.stripe(&stripe);
            self.used = 0;
        }
        while bytes.len() >= 16 {
            self.stripe(&bytes[..16]);
            bytes = &bytes[16..];
        }
        self.tail[..bytes.len()].copy_from_slice(bytes);
        self.used = bytes.len();
    }
    /// Finalize without altering the state, allowing further updates or inspection.
    pub fn checksum(&self) -> u32 {
        let mut value = if self.striped {
            self.lanes
                .iter()
                .zip([1, 7, 12, 18])
                .fold(0u32, |sum, (lane, shift)| {
                    sum.wrapping_add(lane.rotate_left(shift))
                })
        } else {
            P5
        };
        value = value.wrapping_add(self.length);
        let mut bytes = &self.tail[..self.used];
        while bytes.len() >= 4 {
            value = value
                .wrapping_add(word(&bytes[..4]).wrapping_mul(P3))
                .rotate_left(17)
                .wrapping_mul(P4);
            bytes = &bytes[4..];
        }
        for &byte in bytes {
            value = value
                .wrapping_add(u32::from(byte).wrapping_mul(P5))
                .rotate_left(11)
                .wrapping_mul(P1);
        }
        value ^= value >> 15;
        value = value.wrapping_mul(P2);
        value ^= value >> 13;
        value = value.wrapping_mul(P3);
        value ^ (value >> 16)
    }
}
/// Compute the seed-zero XXH32 checksum used by LZ4 framing.
pub fn xxhash32(bytes: &[u8]) -> u32 {
    let mut digest = Xxh32::new();
    digest.update(bytes);
    digest.checksum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_digests_and_split_stripes_match() {
        assert_eq!(xxhash32(b""), 0x02cc_5d05);
        assert_eq!(xxhash32(b"abc"), 0x32d1_53ff);
        let data: Vec<u8> = (0..5000).map(|i| ((i * 73) ^ (i >> 8)) as u8).collect();
        for chunk in [1, 3, 15, 16, 17, 63, 256, 8192] {
            let mut digest = Xxh32::new();
            for part in data.chunks(chunk) {
                digest.update(part);
                digest.update(&[]);
            }
            assert_eq!(digest.checksum(), xxhash32(&data));
            assert_eq!(digest.checksum(), digest.clone().checksum());
        }
    }
    #[test]
    fn full_digests_match_independent_lz4_author_outputs() {
        for record in include_bytes!("../../../tests/fixtures/streams/lz4-xxh32.bin")
            .as_chunks::<8>()
            .0
        {
            let length = u32::from_le_bytes(record[..4].try_into().unwrap()) as usize;
            let expected = u32::from_le_bytes(record[4..].try_into().unwrap());
            let plain: Vec<u8> = (0..length).map(|i| ((i * 73) ^ (i >> 8)) as u8).collect();
            for chunk in [1, 15, 16, 17, 8192] {
                let mut digest = Xxh32::new();
                for bytes in plain.chunks(chunk) {
                    digest.update(bytes);
                }
                assert_eq!(digest.checksum(), expected, "{length} {chunk}");
            }
        }
    }
}
