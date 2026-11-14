use crate::invalid;
use std::io;
pub(super) fn literal_length(code: u8) -> io::Result<(usize, u8)> {
    const BASE: [usize; 20] = [
        16, 18, 20, 22, 24, 28, 32, 40, 48, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384,
        32768, 65536,
    ];
    const BITS: [u8; 20] = [
        1, 1, 1, 1, 2, 2, 3, 3, 4, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ];
    match code {
        0..=15 => Ok((usize::from(code), 0)),
        16..=35 => Ok((BASE[usize::from(code - 16)], BITS[usize::from(code - 16)])),
        _ => Err(invalid()),
    }
}
pub(super) fn match_length(code: u8) -> io::Result<(usize, u8)> {
    const BASE: [usize; 21] = [
        35, 37, 39, 41, 43, 47, 51, 59, 67, 83, 99, 131, 259, 515, 1027, 2051, 4099, 8195, 16387,
        32771, 65539,
    ];
    const BITS: [u8; 21] = [
        1, 1, 1, 1, 2, 2, 3, 3, 4, 4, 5, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16,
    ];
    match code {
        0..=31 => Ok((usize::from(code) + 3, 0)),
        32..=52 => Ok((BASE[usize::from(code - 32)], BITS[usize::from(code - 32)])),
        _ => Err(invalid()),
    }
}
