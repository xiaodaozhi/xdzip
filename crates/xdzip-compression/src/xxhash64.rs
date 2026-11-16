//! Seed-zero XXH64, independently expressed from the published arithmetic specification.
const PRIMES: [u64; 5] = [
    0x9e37_79b1_85eb_ca87,
    0xc2b2_ae3d_27d4_eb4f,
    0x1656_67b1_9e37_79f9,
    0x85eb_ca77_c2b2_ae63,
    0x27d4_eb2f_1656_67c5,
];
fn round(accumulator: u64, lane: u64) -> u64 {
    accumulator
        .wrapping_add(lane.wrapping_mul(PRIMES[1]))
        .rotate_left(31)
        .wrapping_mul(PRIMES[0])
}
fn lane(bytes: &[u8]) -> u64 {
    u64::from_le_bytes(bytes.try_into().expect("eight-byte lane"))
}

/// Incremental non-cryptographic XXH64 with fixed storage and no external state.
#[derive(Clone)]
pub struct Xxh64 {
    lanes: [u64; 4],
    tail: [u8; 32],
    used: usize,
    length: u64,
    striped: bool,
}
impl Default for Xxh64 {
    fn default() -> Self {
        Self::new()
    }
}
impl Xxh64 {
    /// Initialize a seed-zero digest; Zstandard stores its low 32 bits.
    pub fn new() -> Self {
        Self {
            lanes: [
                PRIMES[0].wrapping_add(PRIMES[1]),
                PRIMES[1],
                0,
                0u64.wrapping_sub(PRIMES[0]),
            ],
            tail: [0; 32],
            used: 0,
            length: 0,
            striped: false,
        }
    }
    fn stripe(&mut self, bytes: &[u8]) {
        for (accumulator, bytes) in self.lanes.iter_mut().zip(bytes.as_chunks::<8>().0) {
            *accumulator = round(*accumulator, lane(bytes));
        }
        self.striped = true;
    }
    /// Include physical input in order; the specification uses modulo-2^64 length.
    pub fn update(&mut self, mut bytes: &[u8]) {
        self.length = self.length.wrapping_add(bytes.len() as u64);
        if self.used != 0 {
            let count = bytes.len().min(32 - self.used);
            self.tail[self.used..self.used + count].copy_from_slice(&bytes[..count]);
            self.used += count;
            bytes = &bytes[count..];
            if self.used != 32 {
                return;
            }
            let stripe = self.tail;
            self.stripe(&stripe);
            self.used = 0;
        }
        while bytes.len() >= 32 {
            self.stripe(&bytes[..32]);
            bytes = &bytes[32..];
        }
        self.tail[..bytes.len()].copy_from_slice(bytes);
        self.used = bytes.len();
    }
    /// Finalize without mutating the state, so inspection cannot alter future updates.
    pub fn checksum(&self) -> u64 {
        let mut value = if self.striped {
            let combined = self
                .lanes
                .iter()
                .zip([1, 7, 12, 18])
                .fold(0u64, |sum, (accumulator, shift)| {
                    sum.wrapping_add(accumulator.rotate_left(shift))
                });
            self.lanes.iter().fold(combined, |sum, accumulator| {
                (sum ^ round(0, *accumulator))
                    .wrapping_mul(PRIMES[0])
                    .wrapping_add(PRIMES[3])
            })
        } else {
            PRIMES[4]
        };
        value = value.wrapping_add(self.length);
        let mut bytes = &self.tail[..self.used];
        while bytes.len() >= 8 {
            value = (value ^ round(0, lane(&bytes[..8])))
                .rotate_left(27)
                .wrapping_mul(PRIMES[0])
                .wrapping_add(PRIMES[3]);
            bytes = &bytes[8..];
        }
        if bytes.len() >= 4 {
            let word = u64::from(u32::from_le_bytes(
                bytes[..4].try_into().expect("four-byte word"),
            ));
            value = (value ^ word.wrapping_mul(PRIMES[0]))
                .rotate_left(23)
                .wrapping_mul(PRIMES[1])
                .wrapping_add(PRIMES[2]);
            bytes = &bytes[4..];
        }
        for byte in bytes {
            value = (value ^ u64::from(*byte).wrapping_mul(PRIMES[4]))
                .rotate_left(11)
                .wrapping_mul(PRIMES[0]);
        }
        value ^= value >> 33;
        value = value.wrapping_mul(PRIMES[1]);
        value ^= value >> 29;
        value = value.wrapping_mul(PRIMES[2]);
        value ^ (value >> 32)
    }
}
/// Compute seed-zero XXH64 for a complete borrowed input.
pub fn xxhash64(bytes: &[u8]) -> u64 {
    let mut digest = Xxh64::new();
    digest.update(bytes);
    digest.checksum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independently_authored_full_digests_match_every_lane_and_tail_boundary() {
        let fixtures = include_bytes!("../../../tests/fixtures/streams/zstd-xxh64.bin");
        assert_eq!(fixtures.len(), 18 * 12);
        for record in fixtures.as_chunks::<12>().0 {
            let length = u32::from_le_bytes(record[..4].try_into().unwrap()) as usize;
            let expected = u64::from_le_bytes(record[4..].try_into().unwrap());
            let data: Vec<u8> = (0..length).map(|i| ((i * 73) ^ (i >> 8)) as u8).collect();
            assert_eq!(xxhash64(&data), expected, "length {length}");
            for size in [1, 7, 8, 31, 32, 33, 65536] {
                let mut digest = Xxh64::new();
                for part in data.chunks(size) {
                    digest.update(part);
                    assert_eq!(digest.checksum(), digest.clone().checksum());
                    digest.update(&[]);
                }
                assert_eq!(digest.checksum(), expected, "length {length}, chunk {size}");
            }
        }
    }
    #[test]
    fn zstandard_reference_checksums_use_the_low_bits_after_complete_content() {
        for (bytes, plain) in [
            (
                include_bytes!("../../../tests/fixtures/streams/empty.zst").as_slice(),
                vec![],
            ),
            (
                include_bytes!("../../../tests/fixtures/streams/phrase.zst").as_slice(),
                b"abracadabra abracadabra\n".repeat(6),
            ),
            (
                include_bytes!("../../../tests/fixtures/streams/multiblock.zst").as_slice(),
                b"abracadabra abracadabra\n".repeat(60000),
            ),
        ] {
            let checksum = u32::from_le_bytes(bytes[bytes.len() - 4..].try_into().unwrap());
            assert_eq!(xxhash64(&plain) as u32, checksum);
        }
    }
}
