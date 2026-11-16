use super::{MAGIC, MAX_ITEMS, OVERHEAD, invalid, limit};
use crate::frame_io::{terminal, write_all as emit};
use std::io::{self, Write};
use xdzip_compression::{Xxh64, zstd::encode_zstd_block};
const BLOCK: usize = 128 << 10;
const FRAME_BLOCKS: usize = 65536;
/// Independently compressed Zstandard frames with mandatory content XXH64.
/// Explicit completion owns the end marker; drop never writes or finalizes output.
pub struct ZstdWriter<W> {
    sink: W,
    buffer: Vec<u8>,
    checksum: Xxh64,
    level: u8,
    frames: usize,
    blocks: usize,
    active: bool,
    failed: bool,
}
impl<W: Write> ZstdWriter<W> {
    /// Admit matching, entropy, wire and input buffers before writing the first header.
    pub fn new(sink: W, level: u8, memory_limit: u64) -> io::Result<Self> {
        if ![1, 6, 9].contains(&level) {
            return Err(invalid());
        }
        if memory_limit < OVERHEAD {
            return Err(limit());
        }
        let mut buffer = Vec::new();
        buffer.try_reserve_exact(BLOCK).map_err(|_| limit())?;
        let mut writer = Self {
            sink,
            buffer,
            checksum: Xxh64::new(),
            level,
            frames: 0,
            blocks: 0,
            active: false,
            failed: false,
        };
        writer.start()?;
        Ok(writer)
    }
    fn start(&mut self) -> io::Result<()> {
        if self.frames == MAX_ITEMS {
            return Err(limit());
        }
        emit(&mut self.sink, &MAGIC.to_le_bytes())?;
        // No declared size or dictionary; a 128 KiB window and mandatory checksum.
        emit(&mut self.sink, &[4, 0x38])?;
        self.frames += 1;
        self.blocks = 0;
        self.checksum = Xxh64::new();
        self.active = true;
        Ok(())
    }
    fn block(&mut self, final_block: bool) -> io::Result<()> {
        if self.buffer.is_empty() && !final_block {
            return Ok(());
        }
        let last = final_block || self.blocks + 1 == FRAME_BLOCKS;
        let rle = self.buffer.len() > 1 && self.buffer.iter().all(|b| *b == self.buffer[0]);
        let compressed = if self.buffer.is_empty() || rle {
            None
        } else {
            encode_zstd_block(&self.buffer, self.level, OVERHEAD - BLOCK as u64)?
        };
        let kind = if rle {
            1
        } else if compressed.is_some() {
            2
        } else {
            0
        };
        let payload = compressed.as_deref().unwrap_or(&self.buffer);
        let size = if rle {
            self.buffer.len()
        } else {
            payload.len()
        };
        let header = u32::from(last) | (kind << 1) | ((size as u32) << 3);
        emit(&mut self.sink, &header.to_le_bytes()[..3])?;
        emit(
            &mut self.sink,
            if rle { &self.buffer[..1] } else { payload },
        )?;
        self.blocks += 1;
        self.buffer.clear();
        if last {
            emit(
                &mut self.sink,
                &(self.checksum.checksum() as u32).to_le_bytes(),
            )?;
            self.active = false;
        }
        Ok(())
    }
    fn write_inner(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.buffer.len() == BLOCK {
            self.block(false)?;
        }
        if !self.active {
            self.start()?;
        }
        let count = bytes.len().min(BLOCK - self.buffer.len());
        self.buffer.extend_from_slice(&bytes[..count]);
        self.checksum.update(&bytes[..count]);
        Ok(count)
    }
    /// End the pending frame and flush only after the caller has validated every input byte.
    pub fn finish(mut self) -> io::Result<W> {
        if self.failed {
            return Err(invalid());
        }
        if self.active {
            self.block(true)?;
        }
        self.sink.flush().map_err(terminal)?;
        Ok(self.sink)
    }
}
impl<W: Write> Write for ZstdWriter<W> {
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
        self.block(false)
            .and_then(|_| self.sink.flush())
            .map_err(terminal)
            .inspect_err(|_| self.failed = true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        cell::Cell,
        io::{Cursor, Read},
        rc::Rc,
    };
    #[test]
    fn rollover_closes_a_frame_before_resetting_checksum_and_history() {
        let mut writer = ZstdWriter::new(Vec::new(), 6, 8 << 20).unwrap();
        writer.write_all(b"first frame").unwrap();
        // Advance only the policy counter to exercise the actual wire transition without 8 GiB of input.
        writer.blocks = FRAME_BLOCKS - 1;
        writer.flush().unwrap();
        assert!(!writer.active);
        writer.write_all(b"second frame").unwrap();
        let encoded = writer.finish().unwrap();
        let mut decoder =
            super::super::ZstdReader::new(Cursor::new(&encoded), encoded.len() as u64, 128 << 20)
                .unwrap();
        let mut decoded = Vec::new();
        decoder.read_to_end(&mut decoded).unwrap();
        assert_eq!(decoded, b"first framesecond frame");
        assert_eq!(
            encoded
                .windows(4)
                .filter(|bytes| *bytes == MAGIC.to_le_bytes())
                .count(),
            2
        );
    }
    struct Sink {
        allowance: Rc<Cell<usize>>,
        calls: Rc<Cell<usize>>,
    }
    impl Write for Sink {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.calls.set(self.calls.get() + 1);
            let count = bytes.len().min(self.allowance.get());
            if count == 0 {
                return Err(io::ErrorKind::Other.into());
            }
            self.allowance.set(self.allowance.get() - count);
            Ok(count)
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn writer_rejects_before_output_and_never_resumes_a_failed_sink() {
        let calls = Rc::new(Cell::new(0));
        for (level, memory) in [(0, 8 << 20), (6, OVERHEAD - 1)] {
            assert!(
                ZstdWriter::new(
                    Sink {
                        allowance: Rc::new(Cell::new(100)),
                        calls: calls.clone()
                    },
                    level,
                    memory
                )
                .is_err()
            );
        }
        assert_eq!(calls.get(), 0);
        let allowance = Rc::new(Cell::new(6));
        let mut writer = ZstdWriter::new(
            Sink {
                allowance,
                calls: calls.clone(),
            },
            6,
            8 << 20,
        )
        .unwrap();
        assert!(writer.write_all(&vec![42; BLOCK + 1]).is_err());
        let failed_calls = calls.get();
        assert!(writer.write(b"x").is_err());
        assert!(writer.flush().is_err());
        assert!(writer.finish().is_err());
        assert_eq!(calls.get(), failed_calls);
    }
}
