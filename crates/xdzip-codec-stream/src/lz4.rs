//! Standard LZ4 frames with bounded block storage and independent/shared history.
mod writer;
use crate::frame_io::FrameInput;
use std::io::{self, Cursor, Read};
pub use writer::Lz4Writer;
use xdzip_compression::{Lz4BlockReader, Xxh32, xxhash32};
const MAGIC: u32 = 0x184d2204;
const LEGACY: u32 = 0x184c2102;
const MAX_ITEMS: usize = 100_000;
const MEMORY: u64 = 8 << 20;
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, "invalid_lz4_frame")
}
fn unsupported() -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, "unsupported_lz4_profile")
}
fn limit() -> io::Error {
    io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit")
}
fn word<R: Read>(source: &mut R) -> io::Result<u32> {
    let mut bytes = [0; 4];
    source.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}
enum Block {
    Raw(Cursor<Vec<u8>>),
    Compressed(Box<Lz4BlockReader<Cursor<Vec<u8>>>>),
}
impl Read for Block {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        match self {
            Self::Raw(reader) => reader.read(bytes),
            Self::Compressed(reader) => reader.read(bytes),
        }
    }
}
/// Standard version-1 frame reader. Dictionaries and legacy frames are refused.
/// All declared checks and physical boundaries must validate before EOF succeeds.
pub struct Lz4Reader<R> {
    source: FrameInput<R>,
    packed: u64,
    block: Option<Block>,
    history: Vec<u8>,
    content: Xxh32,
    expected: Option<u64>,
    decoded: u64,
    block_max: usize,
    independent: bool,
    block_check: bool,
    content_check: bool,
    in_frame: bool,
    data_frames: usize,
    items: usize,
    blocks: usize,
    ended: bool,
    failed: bool,
}
impl<R: Read> Lz4Reader<R> {
    /// Admit all block/history storage before allocating or reading a header.
    pub fn new(source: R, packed: u64, memory_limit: u64) -> io::Result<Self> {
        if memory_limit < MEMORY {
            return Err(limit());
        }
        let mut history = Vec::new();
        history.try_reserve_exact(65536).map_err(|_| limit())?;
        Ok(Self {
            source: FrameInput::new(source, packed),
            packed,
            block: None,
            history,
            content: Xxh32::new(),
            expected: None,
            decoded: 0,
            block_max: 0,
            independent: true,
            block_check: false,
            content_check: false,
            in_frame: false,
            data_frames: 0,
            items: 0,
            blocks: 0,
            ended: false,
            failed: false,
        })
    }
    fn start(&mut self) -> io::Result<()> {
        if self.source.peek()?.is_none() {
            if self.data_frames == 0 {
                return Err(unsupported());
            }
            if self.source.position != self.packed {
                return Err(invalid());
            }
            self.ended = true;
            return Ok(());
        }
        if self.items >= MAX_ITEMS {
            return Err(limit());
        }
        self.items += 1;
        let magic = word(&mut self.source)?;
        if magic & 0xfffffff0 == 0x184d2a50 {
            let length = u64::from(word(&mut self.source)?);
            if length > self.packed.saturating_sub(self.source.position) {
                return Err(invalid());
            }
            self.source.skip(length)?;
            return Ok(());
        }
        if magic == LEGACY {
            return Err(unsupported());
        }
        if magic != MAGIC {
            return Err(invalid());
        }
        let mut descriptor = vec![self.source.byte()?, self.source.byte()?];
        let flags = descriptor[0];
        let bd = descriptor[1];
        if flags >> 6 != 1 || flags & 2 != 0 || bd & 0x8f != 0 {
            return Err(unsupported());
        }
        let maximum = match (bd >> 4) & 7 {
            4 => 64 << 10,
            5 => 256 << 10,
            6 => 1 << 20,
            7 => 4 << 20,
            _ => return Err(invalid()),
        };
        self.expected = if flags & 8 != 0 {
            let mut size = [0; 8];
            self.source.read_exact(&mut size)?;
            descriptor.extend_from_slice(&size);
            Some(u64::from_le_bytes(size))
        } else {
            None
        };
        if flags & 1 != 0 {
            let mut dictionary = [0; 4];
            self.source.read_exact(&mut dictionary)?;
            descriptor.extend_from_slice(&dictionary);
        }
        if ((xxhash32(&descriptor) >> 8) as u8) != self.source.byte()? {
            return Err(invalid());
        }
        if flags & 1 != 0 {
            return Err(unsupported());
        }
        self.independent = flags & 32 != 0;
        self.block_check = flags & 16 != 0;
        self.content_check = flags & 4 != 0;
        self.history.clear();
        self.content = Xxh32::new();
        self.decoded = 0;
        self.block_max = maximum;
        self.in_frame = true;
        self.data_frames += 1;
        self.blocks = 0;
        Ok(())
    }
    fn next_block(&mut self) -> io::Result<()> {
        let encoded = word(&mut self.source)?;
        if encoded == 0 {
            if self.expected.is_some_and(|n| n != self.decoded) {
                return Err(invalid());
            }
            if self.content_check && word(&mut self.source)? != self.content.checksum() {
                return Err(invalid());
            }
            self.in_frame = false;
            return Ok(());
        }
        if self.blocks >= MAX_ITEMS {
            return Err(limit());
        }
        self.blocks += 1;
        let length = (encoded & 0x7fffffff) as usize;
        if length > self.block_max
            || length as u64 > self.packed.saturating_sub(self.source.position)
        {
            return Err(invalid());
        }
        let mut data = Vec::new();
        data.try_reserve_exact(length).map_err(|_| limit())?;
        data.resize(length, 0);
        self.source.read_exact(&mut data)?;
        if self.block_check && word(&mut self.source)? != xxhash32(&data) {
            return Err(invalid());
        }
        if self.independent {
            self.history.clear();
        }
        self.block = Some(if encoded >> 31 != 0 {
            Block::Raw(Cursor::new(data))
        } else {
            Block::Compressed(Box::new(Lz4BlockReader::new(
                Cursor::new(data),
                length as u64,
                self.block_max as u64,
                &self.history,
                256 << 10,
            )?))
        });
        Ok(())
    }
    fn history(&mut self, bytes: &[u8]) {
        if bytes.len() >= 65536 {
            self.history.clear();
            self.history
                .extend_from_slice(&bytes[bytes.len() - 65536..]);
        } else {
            let remove = (self.history.len() + bytes.len()).saturating_sub(65536);
            let remaining = self.history.len() - remove;
            self.history.copy_within(remove.., 0);
            self.history.truncate(remaining);
            self.history.extend_from_slice(bytes);
        }
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        loop {
            if self.ended {
                return Ok(0);
            }
            if let Some(block) = &mut self.block {
                let take = output.len().min(8192);
                let n = block.read(&mut output[..take])?;
                if n == 0 {
                    self.block = None;
                    continue;
                }
                self.decoded = self.decoded.checked_add(n as u64).ok_or_else(invalid)?;
                if self.expected.is_some_and(|size| self.decoded > size) {
                    return Err(invalid());
                }
                self.content.update(&output[..n]);
                self.history(&output[..n]);
                return Ok(n);
            }
            if self.in_frame {
                self.next_block()?;
            } else {
                self.start()?;
            }
        }
    }
}
impl<R: Read> Read for Lz4Reader<R> {
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
