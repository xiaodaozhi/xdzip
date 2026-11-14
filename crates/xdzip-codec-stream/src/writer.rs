//! Host-owned single-file writer dispatch, with explicit completion only.
use crate::xz::XzWriter;
use std::io::{self, Write};
use xdzip_codec_api::Format;
use xdzip_compression::GzipWriter;
pub(crate) enum Writer<W> {
    Gzip(GzipWriter<W>),
    Xz(Box<XzWriter<W>>),
}
impl<W: Write> Writer<W> {
    pub(crate) fn new(output: W, format: Format, level: u8) -> io::Result<Self> {
        match format {
            Format::Gzip => GzipWriter::new(output, level).map(Self::Gzip),
            Format::Xz => XzWriter::new(output, level, 128 << 20)
                .map(Box::new)
                .map(Self::Xz),
            _ => Err(io::ErrorKind::Unsupported.into()),
        }
    }
    pub(crate) fn finish(self) -> io::Result<()> {
        match self {
            Self::Gzip(writer) => writer.finish().map(|_| ()),
            Self::Xz(writer) => writer.finish().map(|_| ()),
        }
    }
}
impl<W: Write> Write for Writer<W> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        match self {
            Self::Gzip(writer) => writer.write(bytes),
            Self::Xz(writer) => writer.write(bytes),
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        match self {
            Self::Gzip(writer) => writer.flush(),
            Self::Xz(writer) => writer.flush(),
        }
    }
}
