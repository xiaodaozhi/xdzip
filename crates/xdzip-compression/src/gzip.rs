use crate::{
    crc,
    deflate::{BitOutput, Bits, Inflate},
    invalid,
};
use std::io::{self, Read, Write};

const ENCODE_BLOCK: usize = 2 * 1024 * 1024;

/// Streaming RFC 1952 reader. Every member's CRC and ISIZE are verified at EOF.
/// Callers must drain to EOF and enforce their input, output and cancellation budgets.
pub struct GzipReader<R> {
    bits: Bits<R>,
    inflater: Inflate,
    active: bool,
    members: usize,
    checksum: u32,
    size: u32,
    ended: bool,
    failed: bool,
}
impl<R: Read> GzipReader<R> {
    /// Own a stream without filesystem authority. Optional headers are capped at 64 KiB.
    pub fn new(source: R) -> Self {
        Self {
            bits: Bits::new(source),
            inflater: Inflate::new(),
            active: false,
            members: 0,
            checksum: !0,
            size: 0,
            ended: false,
            failed: false,
        }
    }
    fn header(&mut self) -> io::Result<bool> {
        let mut first = [0];
        if self.bits.source.read(&mut first)? == 0 {
            return if self.members == 0 {
                Err(invalid())
            } else {
                Ok(false)
            };
        }
        let mut checksum = crc(!0, &first);
        let mut count = 1;
        let mut byte = |source: &mut Bits<R>| -> io::Result<u8> {
            if count >= 65536 {
                return Err(invalid());
            }
            let value = source.get(8)? as u8;
            count += 1;
            checksum = crc(checksum, &[value]);
            Ok(value)
        };
        if first[0] != 0x1f || byte(&mut self.bits)? != 0x8b || byte(&mut self.bits)? != 8 {
            return Err(invalid());
        }
        let flags = byte(&mut self.bits)?;
        if flags & 0xe0 != 0 {
            return Err(invalid());
        }
        for _ in 0..6 {
            byte(&mut self.bits)?;
        }
        if flags & 4 != 0 {
            let lo = u16::from(byte(&mut self.bits)?);
            let length = lo | (u16::from(byte(&mut self.bits)?) << 8);
            for _ in 0..length {
                byte(&mut self.bits)?;
            }
        }
        for flag in [8, 16] {
            if flags & flag != 0 {
                while byte(&mut self.bits)? != 0 {}
            }
        }
        if flags & 2 != 0 {
            if count > 65534 {
                return Err(invalid());
            }
            let expected = self.bits.get(16)? as u16;
            if expected != (!checksum) as u16 {
                return Err(invalid());
            }
        }
        self.members = self.members.checked_add(1).ok_or_else(invalid)?;
        self.inflater = Inflate::new();
        self.checksum = !0;
        self.size = 0;
        self.active = true;
        Ok(true)
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.ended {
            return Ok(0);
        }
        loop {
            if !self.active && !self.header()? {
                self.ended = true;
                return Ok(0);
            }
            let n = self.inflater.read(&mut self.bits, output)?;
            if n != 0 {
                self.checksum = crc(self.checksum, &output[..n]);
                self.size = self.size.wrapping_add(n as u32);
                return Ok(n);
            }
            // Inflate has consumed the final block and discarded only pad bits.
            let checksum = self.bits.get(16)? | self.bits.get(16)? << 16;
            let size = self.bits.get(16)? | self.bits.get(16)? << 16;
            if checksum != !self.checksum || size != self.size {
                return Err(invalid());
            }
            self.active = false;
        }
    }
}
impl<R: Read> Read for GzipReader<R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if output.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        let result = self.read_inner(output);
        if result.is_err() {
            self.failed = true;
        }
        result
    }
}

/// Bounded RFC 1952 writer using independent LZ77 and Huffman blocks.
/// Level 0 stores bytes; levels 1/6/9 bound match search to 4/32/128 candidates.
/// Drop never writes or finalizes; only a successful finish produces a trailer.
pub struct GzipWriter<W> {
    bits: BitOutput<W>,
    buffer: Vec<u8>,
    checksum: u32,
    size: u32,
    level: u8,
    started: bool,
    failed: bool,
}
impl<W: Write> GzipWriter<W> {
    /// Create a writer with an explicit supported level. Invalid levels are rejected.
    pub fn new(sink: W, level: u8) -> io::Result<Self> {
        if ![0, 1, 6, 9].contains(&level) {
            return Err(invalid());
        }
        Ok(Self {
            bits: BitOutput::new(sink),
            buffer: Vec::with_capacity(if level == 0 { 65535 } else { ENCODE_BLOCK }),
            checksum: !0,
            size: 0,
            level,
            started: false,
            failed: false,
        })
    }
    fn start(&mut self) -> io::Result<()> {
        if !self.started {
            self.bits
                .sink
                .write_all(&[0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 255])?;
            self.started = true;
        }
        Ok(())
    }
    fn block(&mut self, last: bool) -> io::Result<()> {
        self.start()?;
        self.bits.block(&self.buffer, last, self.level)?;
        self.buffer.clear();
        Ok(())
    }
    /// Finalize the last block and CRC/ISIZE trailer, returning the owned sink.
    pub fn finish(mut self) -> io::Result<W> {
        if self.failed {
            return Err(invalid());
        }
        self.block(true)?;
        self.bits.align()?;
        self.bits.sink.write_all(&(!self.checksum).to_le_bytes())?;
        self.bits.sink.write_all(&self.size.to_le_bytes())?;
        Ok(self.bits.sink)
    }
}
impl<W: Write> Write for GzipWriter<W> {
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
        self.checksum = crc(self.checksum, &input[..take]);
        self.size = self.size.wrapping_add(take as u32);
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
    use flate2::{Compression, read::MultiGzDecoder, write::GzEncoder};
    fn payloads() -> Vec<Vec<u8>> {
        let mut seed = 0x517cc1b727220a95u64;
        let random: Vec<u8> = (0..150000)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as u8
            })
            .collect();
        vec![
            Vec::new(),
            b"123456789".to_vec(),
            vec![b'x'; 150000],
            (0..=255).cycle().take(150000).collect(),
            random,
        ]
    }
    #[test]
    fn known_crc_vector() {
        assert_eq!(!crc(!0, b"123456789"), 0xcbf43926);
    }
    #[test]
    fn multiblock_window_distance_and_high_ratio_are_interoperable() {
        let mut seed = 0x517cc1b727220a95u64;
        let mut data: Vec<u8> = (0..32768)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 7;
                seed ^= seed << 17;
                seed as u8
            })
            .collect();
        data.extend_from_within(..32768);
        data.resize(2 * 1024 * 1024 + 37, 0);
        for level in [0, 6] {
            let mut writer = GzipWriter::new(Vec::new(), level).unwrap();
            writer.write_all(&data).unwrap();
            let bytes = writer.finish().unwrap();
            let mut actual = Vec::new();
            MultiGzDecoder::new(bytes.as_slice())
                .read_to_end(&mut actual)
                .unwrap();
            assert_eq!(actual, data);
            let mut actual = Vec::new();
            GzipReader::new(bytes.as_slice())
                .read_to_end(&mut actual)
                .unwrap();
            assert_eq!(actual, data);
        }
        let mut writer = GzipWriter::new(Vec::new(), 6).unwrap();
        writer.write_all(&vec![0; 4 * 1024 * 1024]).unwrap();
        let bytes = writer.finish().unwrap();
        assert!(bytes.len() < 4 * 1024 * 1024 / 1000);
    }
    #[test]
    fn independent_reader_writer_interoperation_all_levels_and_chunk_boundaries() {
        for data in payloads() {
            for level in [0, 1, 6, 9] {
                let mut writer = GzipWriter::new(Vec::new(), level).unwrap();
                for chunk in data.chunks(997) {
                    writer.write_all(chunk).unwrap();
                }
                let compressed = writer.finish().unwrap();
                let mut decoded = Vec::new();
                MultiGzDecoder::new(compressed.as_slice())
                    .read_to_end(&mut decoded)
                    .unwrap();
                assert_eq!(decoded, data);
                let mut reference = GzEncoder::new(Vec::new(), Compression::new(u32::from(level)));
                reference.write_all(&data).unwrap();
                let reference = reference.finish().unwrap();
                let mut reader = GzipReader::new(reference.as_slice());
                let mut decoded = Vec::new();
                let mut chunk = [0; 113];
                loop {
                    let n = reader.read(&mut chunk).unwrap();
                    if n == 0 {
                        break;
                    }
                    decoded.extend_from_slice(&chunk[..n]);
                }
                assert_eq!(decoded, data);
                let mut roundtrip = Vec::new();
                GzipReader::new(compressed.as_slice())
                    .read_to_end(&mut roundtrip)
                    .unwrap();
                assert_eq!(roundtrip, data);
            }
        }
    }
    #[test]
    fn trailers_concatenation_optional_headers_and_truncation() {
        let mut writer = GzipWriter::new(Vec::new(), 6).unwrap();
        writer.write_all(b"payload").unwrap();
        let compressed = writer.finish().unwrap();
        for length in 0..compressed.len() {
            assert!(
                GzipReader::new(&compressed[..length])
                    .read_to_end(&mut Vec::new())
                    .is_err(),
                "{length}"
            );
        }
        for offset in [compressed.len() - 8, compressed.len() - 4] {
            let mut bad = compressed.clone();
            bad[offset] ^= 1;
            let mut reader = GzipReader::new(bad.as_slice());
            assert!(reader.read_to_end(&mut Vec::new()).is_err());
            assert!(reader.read(&mut [0]).is_err());
        }
        let mut members = compressed.clone();
        members.extend_from_slice(&compressed);
        let mut data = Vec::new();
        GzipReader::new(members.as_slice())
            .read_to_end(&mut data)
            .unwrap();
        assert_eq!(data, b"payloadpayload");
        members.push(0);
        assert!(
            GzipReader::new(members.as_slice())
                .read_to_end(&mut Vec::new())
                .is_err()
        );
        let mut optional = compressed[..10].to_vec();
        optional[3] = 4 | 8 | 16 | 2;
        optional.extend_from_slice(&[3, 0, 1, 2, 3]);
        optional.extend_from_slice(b"file\0comment\0");
        let checksum = (!crc(!0, &optional)) as u16;
        optional.extend_from_slice(&checksum.to_le_bytes());
        optional.extend_from_slice(&compressed[10..]);
        let mut data = Vec::new();
        GzipReader::new(optional.as_slice())
            .read_to_end(&mut data)
            .unwrap();
        assert_eq!(data, b"payload");
        optional[10] ^= 1;
        assert!(
            GzipReader::new(optional.as_slice())
                .read_to_end(&mut Vec::new())
                .is_err()
        );
    }
    #[test]
    fn reserved_block_bad_stored_length_and_invalid_distance_fail() {
        for raw in [vec![7], vec![1, 1, 0, 0, 0], vec![3, 2, 0]] {
            let mut gzip = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 255];
            gzip.extend(raw);
            gzip.extend([0; 8]);
            assert!(
                GzipReader::new(gzip.as_slice())
                    .read_to_end(&mut Vec::new())
                    .is_err()
            );
        }
    }
    #[test]
    fn bounded_arbitrary_blocks_never_panic() {
        let mut seed = 0x6a09e667f3bcc909u64;
        for length in 0..256 {
            for _ in 0..8 {
                let mut gzip = vec![0x1f, 0x8b, 8, 0, 0, 0, 0, 0, 0, 255];
                for _ in 0..length {
                    seed ^= seed << 13;
                    seed ^= seed >> 7;
                    seed ^= seed << 17;
                    gzip.push(seed as u8);
                }
                let _ = GzipReader::new(gzip.as_slice())
                    .take(65536)
                    .read_to_end(&mut Vec::new());
            }
        }
    }
    #[test]
    fn flush_empty_writes_and_sink_failure_do_not_finalize_on_drop() {
        let mut writer = GzipWriter::new(Vec::new(), 6).unwrap();
        assert_eq!(writer.write(&[]).unwrap(), 0);
        writer.flush().unwrap();
        writer.write_all(b"before flush").unwrap();
        writer.flush().unwrap();
        writer.write_all(b"after flush").unwrap();
        let bytes = writer.finish().unwrap();
        let mut decoded = Vec::new();
        MultiGzDecoder::new(bytes.as_slice())
            .read_to_end(&mut decoded)
            .unwrap();
        assert_eq!(decoded, b"before flushafter flush");
        struct Failed;
        impl Write for Failed {
            fn write(&mut self, _: &[u8]) -> io::Result<usize> {
                Err(io::Error::other("owned_fault"))
            }
            fn flush(&mut self) -> io::Result<()> {
                Err(io::Error::other("owned_fault"))
            }
        }
        let mut writer = GzipWriter::new(Failed, 6).unwrap();
        writer.write_all(b"payload").unwrap();
        assert!(writer.flush().is_err());
        assert!(writer.write(b"more").is_err());
        assert!(writer.finish().is_err());
        let mut sink = Vec::new();
        let mut writer = GzipWriter::new(&mut sink, 6).unwrap();
        writer.write_all(b"payload").unwrap();
        drop(writer);
        assert!(sink.is_empty());
    }
}
