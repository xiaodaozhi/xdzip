//! Standard Zstandard frames, independently decoded with bounded host-owned input.
use crate::frame_io::FrameInput;
use std::io::{self, Read};
use xdzip_compression::{Xxh64, zstd::ZstdBlockDecoder};
const MAGIC: u32 = 0xfd2fb528;
const MAX_ITEMS: usize = 100_000;
const OVERHEAD: u64 = 8 << 20;
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid_zstandard_frame")
}
fn unsupported() -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, "unsupported_zstandard_profile")
}
fn limit() -> io::Error {
    io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit")
}
fn field<R: Read>(source: &mut R, size: usize) -> io::Result<u64> {
    let mut bytes = [0; 8];
    source.read_exact(&mut bytes[..size])?;
    Ok(u64::from_le_bytes(bytes))
}
/// One standard-frame sequence. Successful EOF includes all declared size/check validation.
pub struct ZstdReader<R> {
    source: FrameInput<R>,
    packed: u64,
    memory: u64,
    decoder: Option<ZstdBlockDecoder>,
    checksum: Xxh64,
    content_check: bool,
    expected: Option<u64>,
    decoded: u64,
    block_max: usize,
    block: Vec<u8>,
    position: usize,
    frames: usize,
    items: usize,
    blocks: usize,
    ended: bool,
    failed: bool,
}
impl<R: Read> ZstdReader<R> {
    /// Admit all fixed framing storage before a header can request a decoder window.
    pub fn new(source: R, packed: u64, memory_limit: u64) -> io::Result<Self> {
        if memory_limit < OVERHEAD {
            return Err(limit());
        }
        let mut block = Vec::new();
        block.try_reserve_exact(128 << 10).map_err(|_| limit())?;
        Ok(Self {
            source: FrameInput::new(source, packed),
            packed,
            memory: memory_limit,
            decoder: None,
            checksum: Xxh64::new(),
            content_check: false,
            expected: None,
            decoded: 0,
            block_max: 0,
            block,
            position: 0,
            frames: 0,
            items: 0,
            blocks: 0,
            ended: false,
            failed: false,
        })
    }
    fn start(&mut self) -> io::Result<()> {
        if self.source.peek()?.is_none() {
            if self.frames == 0 {
                return Err(unsupported());
            }
            if self.source.position != self.packed {
                return Err(invalid());
            }
            self.ended = true;
            return Ok(());
        }
        if self.items == MAX_ITEMS {
            return Err(limit());
        }
        self.items += 1;
        let magic = field(&mut self.source, 4)? as u32;
        if (0x184d2a50..=0x184d2a5f).contains(&magic) {
            let size = field(&mut self.source, 4)?;
            if size > self.packed.saturating_sub(self.source.position) {
                return Err(invalid());
            }
            return self.source.skip(size);
        }
        if magic != MAGIC {
            return Err(invalid());
        }
        let descriptor = self.source.byte()?;
        if descriptor & 8 != 0 {
            return Err(invalid());
        }
        let single = descriptor & 32 != 0;
        let mut window = if single {
            0
        } else {
            let descriptor = self.source.byte()?;
            let base = 1u64 << (10 + (descriptor >> 3));
            base + (base >> 3) * u64::from(descriptor & 7)
        };
        let dictionary = field(&mut self.source, [0, 1, 2, 4][usize::from(descriptor & 3)])?;
        let mode = descriptor >> 6;
        let size_bytes = match mode {
            0 => usize::from(single),
            1 => 2,
            2 => 4,
            _ => 8,
        };
        self.expected = if size_bytes == 0 {
            None
        } else {
            Some(field(&mut self.source, size_bytes)? + if mode == 1 { 256 } else { 0 })
        };
        if dictionary != 0 {
            return Err(unsupported());
        }
        if single {
            window = self.expected.ok_or_else(invalid)?;
        }
        if window > 64 << 20 || window.checked_add(OVERHEAD).is_none_or(|n| n > self.memory) {
            return Err(limit());
        }
        self.decoder = Some(ZstdBlockDecoder::new(window, self.memory - (4 << 20))?);
        self.block_max = usize::try_from(window.min(128 << 10)).map_err(|_| limit())?;
        self.checksum = Xxh64::new();
        self.content_check = descriptor & 4 != 0;
        self.decoded = 0;
        self.blocks = 0;
        self.frames += 1;
        Ok(())
    }
    fn next_block(&mut self) -> io::Result<()> {
        if self.blocks == MAX_ITEMS {
            return Err(limit());
        }
        self.blocks += 1;
        let header = field(&mut self.source, 3)?;
        let kind = (header >> 1) & 3;
        let size = (header >> 3) as usize;
        if kind == 3 || size > self.block_max {
            return Err(invalid());
        }
        let packed = if kind == 1 { 1 } else { size };
        if packed as u64 > self.packed.saturating_sub(self.source.position) {
            return Err(invalid());
        }
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(packed).map_err(|_| limit())?;
        bytes.resize(packed, 0);
        self.source.read_exact(&mut bytes)?;
        let decoder = self.decoder.as_mut().expect("admitted frame decoder");
        let decoded = match kind {
            0 => decoder.raw(&bytes)?,
            1 => decoder.rle(bytes[0], size)?,
            _ => decoder.decode_compressed(&bytes)?,
        };
        self.decoded = self
            .decoded
            .checked_add(decoded.len() as u64)
            .ok_or_else(invalid)?;
        if self.expected.is_some_and(|size| self.decoded > size) {
            return Err(invalid());
        }
        self.checksum.update(decoded);
        self.block.clear();
        self.block.extend_from_slice(decoded);
        self.position = 0;
        if header & 1 != 0 {
            if self.expected.is_some_and(|size| self.decoded != size) {
                return Err(invalid());
            }
            if self.content_check
                && field(&mut self.source, 4)? as u32 != self.checksum.checksum() as u32
            {
                return Err(invalid());
            }
            self.decoder = None;
        }
        Ok(())
    }
    fn read_inner(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.position < self.block.len() {
                let count = bytes.len().min(self.block.len() - self.position).min(8192);
                bytes[..count].copy_from_slice(&self.block[self.position..self.position + count]);
                self.position += count;
                return Ok(count);
            }
            if self.ended {
                return Ok(0);
            }
            if self.decoder.is_none() {
                self.start()?;
            } else {
                self.next_block()?;
            }
        }
    }
}
impl<R: Read> Read for ZstdReader<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if bytes.is_empty() {
            return Ok(0);
        }
        if self.failed {
            return Err(invalid());
        }
        self.read_inner(bytes).inspect_err(|_| self.failed = true)
    }
}

#[cfg(test)]
mod tests;
