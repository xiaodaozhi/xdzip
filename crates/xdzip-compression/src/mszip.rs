//! MS-MCI section 2: independent CK-prefixed RFC 1951 frames with folder history.
use crate::{DeflateReader, invalid};
use std::io::{self, Read};

/// Bounded MSZIP folder state. A new folder requires a new decoder.
/// Frame errors poison the state; callers must not resume a failed folder.
pub struct MszipDecoder {
    history: Vec<u8>,
    failed: bool,
    finished: bool,
}
impl Default for MszipDecoder {
    fn default() -> Self {
        Self::new()
    }
}
impl MszipDecoder {
    /// Reset Deflate framing and the folder's 32 KiB chronological history.
    pub fn new() -> Self {
        Self {
            history: Vec::with_capacity(32768),
            failed: false,
            finished: false,
        }
    }
    /// Decode and validate one complete frame, after CAB fragment checksums.
    /// `packed` includes CK. The output and retained history are each at most
    /// 32 KiB; split physical fragments must be assembled by the container.
    pub fn decode_frame(&mut self, packed: &[u8], expected: usize) -> io::Result<Vec<u8>> {
        if self.failed || self.finished {
            return Err(invalid());
        }
        let result = self.frame(packed, expected);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
    /// Close a complete folder. Each accepted frame already checked its EOF.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        self.finished = true;
        Ok(())
    }
    fn frame(&mut self, packed: &[u8], expected: usize) -> io::Result<Vec<u8>> {
        if expected > 32768 || packed.len() > 32780 || !packed.starts_with(b"CK") {
            return Err(invalid());
        }
        let deflate = &packed[2..];
        let mut reader =
            DeflateReader::with_dictionary(deflate, deflate.len() as u64, &self.history)?;
        let mut output = Vec::with_capacity(expected + 1);
        reader
            .by_ref()
            .take(expected as u64 + 1)
            .read_to_end(&mut output)?;
        if output.len() != expected {
            return Err(invalid());
        }
        // The extra permitted byte makes exact-sized output drain through
        // BFINAL/EOF as well; the shared reader rejects trailing packed bytes.
        if output.len() == 32768 {
            self.history.clear();
        } else {
            let discard = (self.history.len() + output.len()).saturating_sub(32768);
            self.history.drain(..discard);
        }
        self.history.extend_from_slice(&output);
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Python zlib 1.3.2 raw compressor, level 9; the second stream was authored
    // with the first output as zdict and independently fails without that zdict.
    const FIRST: &[u8] = &[
        43, 40, 202, 204, 47, 82, 112, 118, 116, 82, 72, 203, 207, 73, 73, 45, 82, 200, 200, 44,
        46, 201, 47, 170, 84, 208, 87, 40, 24, 149, 26, 149, 26, 149, 26, 149, 26, 149, 26, 149,
        26, 149, 26, 149, 26, 149, 26, 149, 162, 165, 20, 0,
    ];
    const SECOND: &[u8] = &[
        237, 201, 33, 1, 0, 0, 0, 128, 160, 255, 175, 125, 97, 130, 138, 82, 74, 41, 165, 148, 82,
        74, 41, 165, 148, 82, 74, 41, 165, 148, 82, 74, 41, 165, 158, 10,
    ];
    fn frame(bytes: &[u8]) -> Vec<u8> {
        [b"CK".as_slice(), bytes].concat()
    }
    #[test]
    fn frames_reuse_folder_history_but_not_deflate_framing() {
        let first = b"prior CAB folder history / ".repeat(100);
        let second = b"prior CAB folder history / ".repeat(300);
        let mut decoder = MszipDecoder::new();
        assert_eq!(
            decoder.decode_frame(&frame(FIRST), first.len()).unwrap(),
            first
        );
        assert_eq!(
            decoder.decode_frame(&frame(SECOND), second.len()).unwrap(),
            second
        );
        decoder.finish().unwrap();
        assert!(decoder.decode_frame(&frame(FIRST), first.len()).is_err());
        assert!(
            MszipDecoder::new()
                .decode_frame(&frame(SECOND), second.len())
                .is_err()
        );
    }
    #[test]
    fn invalid_sizes_endings_and_history_failure_poison_the_folder() {
        let valid = frame(FIRST);
        let mut trailing = valid.clone();
        trailing.push(0);
        for (bytes, expected) in [
            (&valid[..valid.len() - 1], 2700),
            (&trailing[..], 2700),
            (&valid[..], 2699),
            (&valid[..], 2701),
            (&valid[..], 32769),
            (&valid[2..], 2700),
        ] {
            let mut decoder = MszipDecoder::new();
            assert!(decoder.decode_frame(bytes, expected).is_err());
            assert!(decoder.decode_frame(&valid, 2700).is_err());
        }
    }
}
