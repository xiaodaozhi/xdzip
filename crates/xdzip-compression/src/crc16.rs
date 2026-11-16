//! Reflected CRC-16/IBM, initial zero, used by LHA and compatible container checks.
/// Incremental legacy content check. This does not authenticate data or provenance.
#[derive(Clone, Copy, Debug, Default)]
pub struct Crc16(u16);
const fn table() -> [u16; 256] {
    let mut result = [0; 256];
    let mut at = 0;
    while at < 256 {
        let mut value = at as u16;
        let mut bit = 0;
        while bit < 8 {
            value = (value >> 1) ^ if value & 1 != 0 { 0xa001 } else { 0 };
            bit += 1;
        }
        result[at] = value;
        at += 1;
    }
    result
}
const TABLE: [u16; 256] = table();
impl Crc16 {
    /// Initialize the format's zero state, without a final complement.
    pub const fn new() -> Self {
        Self(0)
    }
    /// Include bytes in their decoded or declared-header order.
    pub fn update(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = (self.0 >> 8) ^ TABLE[usize::from((self.0 as u8) ^ byte)];
        }
    }
    /// Return the current 16-bit check without altering the state.
    pub const fn checksum(&self) -> u16 {
        self.0
    }
}
/// Compute the reflected legacy check; callers own verification and field-byte order.
pub fn crc16(bytes: &[u8]) -> u16 {
    let mut crc = Crc16::new();
    crc.update(bytes);
    crc.checksum()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standard_check_and_independent_java_archive_vectors_match_fragmented_input() {
        assert_eq!(crc16(b"123456789"), 0xbb3d);
        assert_eq!(crc16(b""), 0);
        let phrase = b"abracadabra abracadabra\n".repeat(6);
        assert_eq!(crc16(&phrase), 55993);
        for size in [1, 3, 17, 4096] {
            let mut crc = Crc16::new();
            for chunk in phrase.repeat(4096).chunks(size) {
                crc.update(chunk);
            }
            assert_eq!(crc.checksum(), 58102);
        }
    }
}
