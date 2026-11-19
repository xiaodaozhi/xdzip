//! Host-owned single-file writer dispatch, with explicit completion only.
use crate::controlled::Filter;
use crate::legacy_writer::{BrotliStoreWriter, CompressWriter, LzipWriter};
use crate::lz4::Lz4Writer;
use crate::xz::XzWriter;
use crate::zstd::ZstdWriter;
use std::io::{self, Write};
use xdzip_compression::{Bzip2Writer, GzipWriter, Lzma1Writer};
/// Shared outer-frame writer. The caller retains host authority and explicit completion.
pub enum FrameWriter<W: Write> {
    /// GZIP framing.
    Gzip(GzipWriter<W>),
    /// XZ framing.
    Xz(Box<XzWriter<W>>),
    /// Standard LZ4 framing.
    Lz4(Box<Lz4Writer<W>>),
    /// Zstandard framing.
    Zstd(Box<ZstdWriter<W>>),
    /// BZip2 block stream.
    Bzip2(Box<Bzip2Writer<W>>),
    /// LZMA Alone with an end marker.
    Lzma(Box<Lzma1Writer<W>>),
    /// Lzip version 1 member.
    Lzip(Box<LzipWriter<W>>),
    /// Unix compress in block mode.
    Compress(Box<CompressWriter<W>>),
    /// Brotli stream carrying uncompressed meta-blocks.
    Brotli(BrotliStoreWriter<W>),
}
impl<W: Write> FrameWriter<W> {
    /// Admit only implemented outer-frame writers and their actual levels.
    pub fn new(output: W, filter: Filter, level: u8) -> io::Result<Self> {
        match filter {
            Filter::Gzip => GzipWriter::new(output, level).map(Self::Gzip),
            Filter::Xz => XzWriter::new(output, level, 128 << 20)
                .map(Box::new)
                .map(Self::Xz),
            Filter::Lz4 => Lz4Writer::new(output, level, 128 << 20)
                .map(Box::new)
                .map(Self::Lz4),
            Filter::Zstd => ZstdWriter::new(output, level, 128 << 20)
                .map(Box::new)
                .map(Self::Zstd),
            Filter::Bzip2 => Bzip2Writer::new(output).map(Box::new).map(Self::Bzip2),
            Filter::Lzma => {
                let mut output = output;
                output.write_all(&Lzma1Writer::<W>::PROPERTIES)?;
                output.write_all(&u64::MAX.to_le_bytes())?;
                Lzma1Writer::new(output).map(Box::new).map(Self::Lzma)
            }
            Filter::Lzip => LzipWriter::new(output).map(Box::new).map(Self::Lzip),
            Filter::Compress => CompressWriter::new(output)
                .map(Box::new)
                .map(Self::Compress),
            Filter::Brotli => Ok(Self::Brotli(BrotliStoreWriter::new(output))),
            Filter::Identity => Err(io::ErrorKind::Unsupported.into()),
        }
    }
    /// Emit the final frame only after the host has validated the complete input.
    pub fn finish(self) -> io::Result<()> {
        match self {
            Self::Gzip(writer) => writer.finish().map(|_| ()),
            Self::Xz(writer) => writer.finish().map(|_| ()),
            Self::Lz4(writer) => writer.finish().map(|_| ()),
            Self::Zstd(writer) => writer.finish().map(|_| ()),
            Self::Brotli(writer) => writer.finish(),
            Self::Compress(writer) => writer.finish(),
            Self::Lzip(writer) => writer.finish(),
            Self::Lzma(writer) => writer.finish().map(|_| ()),
            Self::Bzip2(writer) => writer.finish().map(|_| ()),
        }
    }
}
impl<W: Write> Write for FrameWriter<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Self::Gzip(writer) => writer.write(bytes),
            Self::Xz(writer) => writer.write(bytes),
            Self::Lz4(writer) => writer.write(bytes),
            Self::Zstd(writer) => writer.write(bytes),
            Self::Brotli(writer) => writer.write(bytes),
            Self::Compress(writer) => writer.write(bytes),
            Self::Lzip(writer) => writer.write(bytes),
            Self::Lzma(writer) => writer.write(bytes),
            Self::Bzip2(writer) => writer.write(bytes),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Gzip(writer) => writer.flush(),
            Self::Xz(writer) => writer.flush(),
            Self::Lz4(writer) => writer.flush(),
            Self::Zstd(writer) => writer.flush(),
            Self::Brotli(writer) => writer.flush(),
            Self::Compress(writer) => writer.flush(),
            Self::Lzip(writer) => writer.flush(),
            Self::Lzma(writer) => writer.flush(),
            Self::Bzip2(writer) => writer.flush(),
        }
    }
}
