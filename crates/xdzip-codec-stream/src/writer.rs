//! Host-owned single-file writer dispatch, with explicit completion only.
use crate::controlled::Filter;
use crate::lz4::Lz4Writer;
use crate::xz::XzWriter;
use crate::zstd::ZstdWriter;
use std::io::{self, Write};
use xdzip_compression::GzipWriter;
/// Shared outer-frame writer. The caller retains host authority and explicit completion.
pub enum FrameWriter<W> {
    /// GZIP framing.
    Gzip(GzipWriter<W>),
    /// XZ framing.
    Xz(Box<XzWriter<W>>),
    /// Standard LZ4 framing.
    Lz4(Box<Lz4Writer<W>>),
    /// Zstandard framing.
    Zstd(Box<ZstdWriter<W>>),
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
            _ => Err(io::ErrorKind::Unsupported.into()),
        }
    }
    /// Emit the final frame only after the host has validated the complete input.
    pub fn finish(self) -> io::Result<()> {
        match self {
            Self::Gzip(writer) => writer.finish().map(|_| ()),
            Self::Xz(writer) => writer.finish().map(|_| ()),
            Self::Lz4(writer) => writer.finish().map(|_| ()),
            Self::Zstd(writer) => writer.finish().map(|_| ()),
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
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Gzip(writer) => writer.flush(),
            Self::Xz(writer) => writer.flush(),
            Self::Lz4(writer) => writer.flush(),
            Self::Zstd(writer) => writer.flush(),
        }
    }
}
