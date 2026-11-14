//! Independently compressed, checksummed 256 KiB blocks in one standard frame.
use super::{MAGIC, MAX_ITEMS, MEMORY, invalid, limit};
use crate::frame_io::{terminal, write_all as emit};
use std::io::{self, Write};
use xdzip_compression::{Xxh32, encode_lz4_block, xxhash32};
const BLOCK: usize = 256 * 1024;
/// Standard LZ4 writer with both block and content XXH32 and bounded matching.
/// Drop never completes a frame; any failed write poisons subsequent operations.
pub struct Lz4Writer<W> {
    sink: W,
    buffer: Vec<u8>,
    checksum: Xxh32,
    level: u8,
    blocks: usize,
    failed: bool,
}
impl<W: Write> Lz4Writer<W> {
    /// Admit all working buffers before writing a standard frame header.
    pub fn new(mut sink: W, level: u8, memory_limit: u64) -> io::Result<Self> {
        if ![1, 6, 9].contains(&level) {
            return Err(invalid());
        }
        if memory_limit < MEMORY {
            return Err(limit());
        }
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(BLOCK).map_err(|_| limit())?;
        emit(&mut sink, &MAGIC.to_le_bytes())?;
        let descriptor = [0x74, 0x50];
        emit(&mut sink, &descriptor)?;
        emit(&mut sink, &[(xxhash32(&descriptor) >> 8) as u8])?;
        Ok(Self {
            sink,
            buffer,
            checksum: Xxh32::new(),
            level,
            blocks: 0,
            failed: false,
        })
    }
    fn block(&mut self) -> io::Result<()> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        if self.blocks >= MAX_ITEMS {
            return Err(limit());
        }
        let encoded = encode_lz4_block(&self.buffer, self.level, MEMORY - BLOCK as u64)?;
        let (data, size) = if encoded.len() < self.buffer.len() {
            (encoded.as_slice(), encoded.len() as u32)
        } else {
            (
                self.buffer.as_slice(),
                self.buffer.len() as u32 | 0x80000000,
            )
        };
        emit(&mut self.sink, &size.to_le_bytes())?;
        emit(&mut self.sink, data)?;
        emit(&mut self.sink, &xxhash32(data).to_le_bytes())?;
        self.buffer.clear();
        self.blocks += 1;
        Ok(())
    }
    /// Complete pending data, end marker and content checksum, then return the sink.
    pub fn finish(mut self) -> io::Result<W> {
        if self.failed {
            return Err(invalid());
        }
        self.block()?;
        emit(&mut self.sink, &[0; 4])?;
        emit(&mut self.sink, &self.checksum.checksum().to_le_bytes())?;
        self.sink.flush().map_err(terminal)?;
        Ok(self.sink)
    }
    fn write_inner(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.buffer.len() == BLOCK {
            self.block()?;
        }
        let n = bytes.len().min(BLOCK - self.buffer.len());
        self.buffer.extend_from_slice(&bytes[..n]);
        self.checksum.update(&bytes[..n]);
        Ok(n)
    }
}
impl<W: Write> Write for Lz4Writer<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.failed {
            return Err(invalid());
        }
        if bytes.is_empty() {
            return Ok(0);
        }
        self.write_inner(bytes)
            .map_err(terminal)
            .inspect_err(|_| self.failed = true)
    }
    fn flush(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        self.block()
            .and_then(|_| self.sink.flush())
            .map_err(terminal)
            .inspect_err(|_| self.failed = true)
    }
}
