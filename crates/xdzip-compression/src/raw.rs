use crate::{
    deflate::{BitOutput, Bits, Inflate},
    invalid,
};
use std::io::{self, Read, Take, Write};

const ENCODE_BLOCK: usize = 2 * 1024 * 1024;

/// Streaming raw RFC 1951 reader, bounded to an exact compressed range.
/// A successful EOF requires BFINAL/StreamEnd and exact byte consumption;
/// truncation, concatenated streams and trailing bytes are integrity failures.
/// Callers must drain to EOF, check output size/checksum and enforce work limits.
pub struct DeflateReader<R> {
    bits: Bits<Take<R>>,
    inflater: Inflate,
    packed: u64,
    ended: bool,
    failed: bool,
}
impl<R: Read> DeflateReader<R> {
    /// Restrict reads to the admitted compressed length. This does not grant seek or paths.
    pub fn new(source: R, packed: u64) -> Self {
        Self {
            bits: Bits::new(source.take(packed)),
            inflater: Inflate::new(),
            packed,
            ended: false,
            failed: false,
        }
    }
    /// Start an independent raw stream with an admitted chronological dictionary.
    /// Dictionary bytes are history, never emitted output. The container must
    /// authorize this reset and retain at most the preceding 32 KiB itself.
    pub fn with_dictionary(source: R, packed: u64, history: &[u8]) -> io::Result<Self> {
        if history.len() > 32768 {
            return Err(invalid());
        }
        let mut reader = Self::new(source, packed);
        reader.inflater.remember(history);
        Ok(reader)
    }
    /// Logical compressed bytes consumed, excluding buffered read-ahead.
    pub fn consumed_bytes(&self) -> u64 {
        self.bits.consumed
    }
}
impl<R: Read> Read for DeflateReader<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        if self.ended {
            return Ok(0);
        }
        let result = self
            .inflater
            .read(&mut self.bits, output)
            .and_then(|count| {
                if count == 0 {
                    if self.bits.consumed != self.packed {
                        return Err(invalid());
                    }
                    self.ended = true;
                }
                Ok(count)
            });
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

/// Bounded streaming raw RFC 1951 writer shared by ZIP and GZIP.
/// Drop never writes or finishes. Only successful finish emits the last block.
pub struct DeflateWriter<W> {
    bits: BitOutput<W>,
    buffer: Vec<u8>,
    level: u8,
    prefix: &'static [u8],
    started: bool,
    failed: bool,
}
impl<W: Write> DeflateWriter<W> {
    /// Create a raw compressor. Supported levels are 0, 1, 6 and 9.
    pub fn new(sink: W, level: u8) -> io::Result<Self> {
        Self::with_prefix(sink, level, &[])
    }
    pub(crate) fn with_prefix(sink: W, level: u8, prefix: &'static [u8]) -> io::Result<Self> {
        if ![0, 1, 6, 9].contains(&level) {
            return Err(invalid());
        }
        Ok(Self {
            bits: BitOutput::new(sink),
            buffer: Vec::with_capacity(if level == 0 { 65535 } else { ENCODE_BLOCK }),
            level,
            prefix,
            started: false,
            failed: false,
        })
    }
    fn block(&mut self, last: bool) -> io::Result<()> {
        if !self.started {
            self.bits.sink.write_all(self.prefix)?;
            self.started = true;
        }
        self.bits.block(&self.buffer, last, self.level)?;
        self.buffer.clear();
        Ok(())
    }
    /// Write the final block and pad bits, then return the owned sink.
    pub fn finish(mut self) -> io::Result<W> {
        if self.failed {
            return Err(invalid());
        }
        self.block(true)?;
        self.bits.align()?;
        Ok(self.bits.sink)
    }
}
impl<W: Write> Write for DeflateWriter<W> {
    fn write(&mut self, input: &[u8]) -> io::Result<usize> {
        if input.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        let capacity = if self.level == 0 { 65535 } else { ENCODE_BLOCK };
        if self.buffer.len() == capacity
            && let Err(error) = self.block(false)
        {
            self.failed = true;
            return Err(error);
        }
        let take = input.len().min(capacity - self.buffer.len());
        self.buffer.extend_from_slice(&input[..take]);
        Ok(take)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        let result = self
            .block(false)
            .and_then(|_| self.bits.flush_bytes())
            .and_then(|_| self.bits.sink.flush());
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preset_history_is_not_output_and_admits_the_full_rfc_distance() {
        // RFC 1951 fixed tree: length 3, distance 32768, then EOB. This wire
        // vector was separately decoded by Python zlib with the same dictionary.
        let packed = [3, 222, 255, 15, 0];
        let mut history = vec![0; 32768];
        history[..3].copy_from_slice(b"abc");
        let mut output = Vec::new();
        let mut reader = DeflateReader::with_dictionary(&packed[..], 5, &history).unwrap();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, b"abc");
        assert_eq!(reader.consumed_bytes(), 5);
        for short in [0, 32767] {
            let mut reader =
                DeflateReader::with_dictionary(&packed[..], 5, &history[..short]).unwrap();
            assert!(reader.read_to_end(&mut Vec::new()).is_err());
        }
        history.push(0);
        assert!(DeflateReader::with_dictionary(&packed[..], 5, &history).is_err());
    }
    #[test]
    fn interrupted_stored_read_retries_without_losing_output_or_poisoning() {
        struct InterruptedOnce {
            bytes: &'static [u8],
            at: usize,
            interrupted: bool,
        }
        impl Read for InterruptedOnce {
            fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
                if self.at == 5 && !self.interrupted {
                    self.interrupted = true;
                    return Err(io::ErrorKind::Interrupted.into());
                }
                let count = output.len().min(self.bytes.len() - self.at);
                output[..count].copy_from_slice(&self.bytes[self.at..self.at + count]);
                self.at += count;
                Ok(count)
            }
        }
        let source = InterruptedOnce {
            bytes: &[1, 1, 0, 254, 255, b'x'],
            at: 0,
            interrupted: false,
        };
        let mut reader = DeflateReader::new(source, 6);
        let mut output = Vec::new();
        reader.read_to_end(&mut output).unwrap();
        assert_eq!(output, b"x");
        assert_eq!(reader.consumed_bytes(), 6);
        assert_eq!(reader.read(&mut [0]).unwrap(), 0);
    }
    #[test]
    fn raw_streams_require_exact_consumption_and_never_read_beyond_the_range() {
        for level in [0, 1, 6, 9] {
            let data = b"independently encoded payload".repeat(1000);
            let mut encoder = DeflateWriter::new(Vec::new(), level).unwrap();
            encoder.write_all(&data).unwrap();
            let compressed = encoder.finish().unwrap();
            let mut oracle = Vec::new();
            flate2::read::DeflateDecoder::new(compressed.as_slice())
                .read_to_end(&mut oracle)
                .unwrap();
            assert_eq!(oracle, data);
            let mut reader = DeflateReader::new(compressed.as_slice(), compressed.len() as u64);
            let mut actual = Vec::new();
            reader.read_to_end(&mut actual).unwrap();
            assert_eq!(actual, data);
            assert_eq!(reader.consumed_bytes(), compressed.len() as u64);
            let mut with_tail = compressed.clone();
            with_tail.extend_from_slice(b"adjacent bytes");
            let mut source = with_tail.as_slice();
            let mut actual = Vec::new();
            DeflateReader::new(&mut source, compressed.len() as u64)
                .read_to_end(&mut actual)
                .unwrap();
            assert_eq!(source, b"adjacent bytes");
            for packed in [compressed.len() - 1, compressed.len() + 1] {
                assert!(
                    DeflateReader::new(with_tail.as_slice(), packed as u64)
                        .read_to_end(&mut Vec::new())
                        .is_err()
                );
            }
            let mut concatenated = compressed.clone();
            concatenated.extend_from_slice(&compressed);
            assert!(
                DeflateReader::new(concatenated.as_slice(), concatenated.len() as u64)
                    .read_to_end(&mut Vec::new())
                    .is_err()
            );
        }
    }
}
