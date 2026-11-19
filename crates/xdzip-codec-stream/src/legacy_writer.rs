//! Small write profiles for legacy TAR filters. No filesystem authority.
use std::{
    collections::HashMap,
    io::{self, BufWriter, Write},
};
use xdzip_compression::{Crc32, Lzma1Writer};

pub struct LzipWriter<W: Write> {
    raw: Lzma1Writer<W>,
    crc: Crc32,
    decoded: u64,
}
impl<W: Write> LzipWriter<W> {
    pub(crate) fn new(mut output: W) -> io::Result<Self> {
        // Lzip 1 with a power-of-two 8 MiB dictionary, lc/lp/pb = 3/0/2.
        output.write_all(b"LZIP\x01\x17")?;
        Ok(Self {
            raw: Lzma1Writer::new(output)?,
            crc: Crc32::new(),
            decoded: 0,
        })
    }
    pub(crate) fn finish(self) -> io::Result<()> {
        let (mut output, packed) = self.raw.finish()?;
        output.write_all(&self.crc.checksum().to_le_bytes())?;
        output.write_all(&self.decoded.to_le_bytes())?;
        output.write_all(&(packed + 26).to_le_bytes())
    }
}
impl<W: Write> Write for LzipWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let count = self.raw.write(bytes)?;
        self.crc.update(&bytes[..count]);
        self.decoded += count as u64;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.raw.flush()
    }
}

pub struct CompressWriter<W: Write> {
    output: BufWriter<W>,
    codes: HashMap<(u16, u8), u16>,
    prefix: Option<u16>,
    next: u16,
    group: [u8; 9],
    used: usize,
}
impl<W: Write> CompressWriter<W> {
    pub(crate) fn new(mut output: W) -> io::Result<Self> {
        // Block mode, with a standard 16-bit maximum. Reset early enough to
        // keep every group at nine bits, including the CLEAR group.
        output.write_all(&[0x1f, 0x9d, 0x90])?;
        Ok(Self {
            output: BufWriter::new(output),
            codes: HashMap::with_capacity(254),
            prefix: None,
            next: 257,
            group: [0; 9],
            used: 0,
        })
    }
    fn flush_group(&mut self, padded: bool) -> io::Result<()> {
        if self.used != 0 {
            let bytes = if padded { 9 } else { self.used.div_ceil(8) };
            self.output.write_all(&self.group[..bytes])?;
            self.group = [0; 9];
            self.used = 0;
        }
        Ok(())
    }
    fn emit(&mut self, code: u16) -> io::Result<()> {
        let offset = self.used / 8;
        let bits = code << (self.used % 8);
        self.group[offset] |= bits as u8;
        self.group[offset + 1] |= (bits >> 8) as u8;
        self.used += 9;
        if self.used == 72 {
            self.flush_group(true)?;
        }
        Ok(())
    }
    pub(crate) fn finish(mut self) -> io::Result<()> {
        if let Some(code) = self.prefix {
            self.emit(code)?;
        }
        self.flush_group(false)?;
        self.output.flush()
    }
}
impl<W: Write> Write for CompressWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let bytes = &bytes[..bytes.len().min(65536)];
        for &byte in bytes {
            if let Some(prefix) = self.prefix {
                let key = (prefix, byte);
                if let Some(&code) = self.codes.get(&key) {
                    self.prefix = Some(code);
                    continue;
                }
                self.emit(prefix)?;
                if self.next == 511 {
                    self.emit(256)?;
                    self.flush_group(true)?;
                    self.codes.clear();
                    self.next = 257;
                } else {
                    self.codes.insert(key, self.next);
                    self.next += 1;
                }
            }
            self.prefix = Some(u16::from(byte));
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}

pub struct BrotliStoreWriter<W> {
    output: W,
    started: bool,
}
impl<W: Write> BrotliStoreWriter<W> {
    pub(crate) fn new(output: W) -> Self {
        Self {
            output,
            started: false,
        }
    }
    pub(crate) fn finish(mut self) -> io::Result<()> {
        // With WBITS=16, its zero window bit is needed only for empty streams.
        self.output
            .write_all(&[if self.started { 0b11 } else { 0b110 }])
    }
}
impl<W: Write> Write for BrotliStoreWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        let count = bytes.len().min(65536);
        if count == 0 {
            return Ok(0);
        }
        // RFC 7932 §9.2. Non-final block, four MLEN nibbles, then ISUNCOMPRESSED.
        let shift = if self.started { 3 } else { 4 };
        let header = (((count as u32 - 1) + (1 << 16)) << shift).to_le_bytes();
        self.output.write_all(&header[..3])?;
        self.output.write_all(&bytes[..count])?;
        self.started = true;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.output.flush()
    }
}
