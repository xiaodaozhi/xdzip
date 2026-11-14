//! Bounded XZ encoding; each block independently resets the LZMA2 dictionary.
use super::{
    Check, FIXED_MEMORY, MAGIC, MAX_RECORDS, crc32, index_bytes, invalid, limit, terminal,
};
use std::io::{self, Write};
use xdzip_compression::{Lzma2Writer, lzma2_writer_memory_required};
const BLOCK_SIZE: u64 = 16 * 1024 * 1024;
fn emit(sink: &mut dyn Write, mut bytes: &[u8]) -> io::Result<()> {
    while !bytes.is_empty() {
        match sink.write(bytes) {
            Err(error)
                if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none() =>
            {
                continue;
            }
            Err(error) => return Err(terminal(error)),
            Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
            Ok(n) if n <= bytes.len() => bytes = &bytes[n..],
            Ok(_) => return Err(invalid()),
        }
    }
    Ok(())
}

/// Streaming XZ with CRC64 and real LZMA2 compression at levels 1, 6 or 9.
/// Drop never finishes a frame; failed writes poison all further operations.
pub struct XzWriter<W> {
    sink: Option<W>,
    encoder: Option<Lzma2Writer<W>>,
    records: Vec<(u64, u64)>,
    check: Check,
    level: u8,
    memory_limit: u64,
    failed: bool,
}
impl<W: Write> XzWriter<W> {
    /// Admit the complete working set before writing the stream header.
    pub fn new(mut sink: W, level: u8, memory_limit: u64) -> io::Result<Self> {
        if lzma2_writer_memory_required(level)?
            .checked_add(FIXED_MEMORY)
            .ok_or_else(limit)?
            > memory_limit
        {
            return Err(limit());
        }
        emit(&mut sink, MAGIC)?;
        emit(&mut sink, &[0, 4])?;
        emit(&mut sink, &crc32(&[0, 4]).to_le_bytes())?;
        Ok(Self {
            sink: Some(sink),
            encoder: None,
            records: Vec::new(),
            check: Check::new(4)?,
            level,
            memory_limit,
            failed: false,
        })
    }
    fn start_block(&mut self) -> io::Result<()> {
        if self.records.len() >= MAX_RECORDS {
            return Err(limit());
        }
        let mut sink = self.sink.take().ok_or_else(invalid)?;
        // Fixed LZMA2 writer property: 8 MiB dictionary. Header size is 12 bytes.
        let mut header = vec![2, 0, 0x21, 1, 22, 0, 0, 0];
        header.extend_from_slice(&crc32(&header).to_le_bytes());
        emit(&mut sink, &header)?;
        self.encoder = Some(Lzma2Writer::new(
            sink,
            self.level,
            self.memory_limit - FIXED_MEMORY,
        )?);
        self.check = Check::new(4)?;
        Ok(())
    }
    fn finish_block(&mut self) -> io::Result<()> {
        let Some(encoder) = self.encoder.take() else {
            return Ok(());
        };
        let size = encoder.input_bytes();
        // finish writes exactly the final zero marker after pending chunk output.
        // flush first so encoded_bytes includes that pending compressed chunk.
        let mut encoder = encoder;
        encoder.flush()?;
        let packed = encoder.encoded_bytes().checked_add(1).ok_or_else(invalid)?;
        let mut sink = encoder.finish()?;
        emit(&mut sink, &[0; 3][..((4 - packed % 4) % 4) as usize])?;
        let check = std::mem::replace(&mut self.check, Check::None).finish();
        emit(&mut sink, &check)?;
        self.records.try_reserve(1).map_err(|_| limit())?;
        self.records.push((12 + packed + check.len() as u64, size));
        self.sink = Some(sink);
        Ok(())
    }
    /// Complete blocks, the exact index and footer, then return the host sink.
    pub fn finish(mut self) -> io::Result<W> {
        if self.failed {
            return Err(invalid());
        }
        self.finish_block()?;
        let index = index_bytes(&self.records)?;
        let mut sink = self.sink.take().ok_or_else(invalid)?;
        emit(&mut sink, &index)?;
        let mut footer = Vec::with_capacity(12);
        footer.extend_from_slice(&((index.len() / 4 - 1) as u32).to_le_bytes());
        footer.extend_from_slice(&[0, 4]);
        emit(&mut sink, &crc32(&footer).to_le_bytes())?;
        emit(&mut sink, &footer)?;
        emit(&mut sink, b"YZ")?;
        sink.flush().map_err(terminal)?;
        Ok(sink)
    }
    fn write_inner(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.encoder.is_none() {
            self.start_block()?;
        }
        let encoder = self.encoder.as_mut().ok_or_else(invalid)?;
        let take = bytes
            .len()
            .min((BLOCK_SIZE - encoder.input_bytes()) as usize);
        let n = encoder.write(&bytes[..take])?;
        self.check.update(&bytes[..n]);
        if encoder.input_bytes() == BLOCK_SIZE {
            self.finish_block()?;
        }
        Ok(n)
    }
}
impl<W: Write> Write for XzWriter<W> {
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
        let result = if let Some(encoder) = &mut self.encoder {
            encoder.flush()
        } else {
            self.sink.as_mut().ok_or_else(invalid)?.flush()
        };
        result.map_err(terminal).inspect_err(|_| self.failed = true)
    }
}
