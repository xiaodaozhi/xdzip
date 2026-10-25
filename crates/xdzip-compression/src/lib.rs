//! Independently implemented compression primitives. No production dependencies.
//! Wire definitions: RFC 1951 sections 3.1–3.2 and RFC 1952 section 2.3.
//! Constants below describe wire code ranges, not a reference implementation.
mod deflate;
mod gzip;
mod raw;
pub use gzip::{GzipReader, GzipWriter};
pub use raw::{DeflateReader, DeflateWriter};

/// Incremental IEEE CRC-32 used by ZIP and GZIP containers.
#[derive(Clone, Copy, Debug)]
pub struct Crc32(u32);
impl Crc32 {
    /// Initialize the reflected CRC with all bits set.
    pub fn new() -> Self {
        Self(!0)
    }
    /// Include the next chunk in physical order.
    pub fn update(&mut self, bytes: &[u8]) {
        self.0 = crc(self.0, bytes);
    }
    /// Return the finalized value without changing the incremental state.
    pub fn checksum(&self) -> u32 {
        !self.0
    }
}
impl Default for Crc32 {
    fn default() -> Self {
        Self::new()
    }
}
/// Compute IEEE CRC-32 without a third-party format or checksum implementation.
pub fn crc32(bytes: &[u8]) -> u32 {
    !crc(!0, bytes)
}

const LENGTHS: [usize; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LENGTH_BITS: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DISTANCES: [usize; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DISTANCE_BITS: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn invalid() -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, "invalid_compressed_stream")
}

// Reflected IEEE CRC-32 polynomial; table is derived mathematically at compile time.
const fn crc_table() -> [u32; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut value = i as u32;
        let mut bit = 0;
        while bit < 8 {
            value = (value >> 1) ^ if value & 1 != 0 { 0xedb88320 } else { 0 };
            bit += 1;
        }
        table[i] = value;
        i += 1;
    }
    table
}
const CRC_TABLE: [u32; 256] = crc_table();
fn crc(mut state: u32, bytes: &[u8]) -> u32 {
    for byte in bytes {
        state = (state >> 8) ^ CRC_TABLE[((state ^ u32::from(*byte)) & 255) as usize];
    }
    state
}
