//! Shared bounded sequential framing input, retaining terminal error payloads.
use std::io::{self, BufRead, BufReader, Read, Take, Write};
pub(crate) fn write_all(sink: &mut dyn Write, mut bytes: &[u8]) -> io::Result<()> {
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
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid_write_count",
                ));
            }
        }
    }
    Ok(())
}
pub(crate) fn terminal(error: io::Error) -> io::Error {
    if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_some() {
        io::Error::other(error.into_inner().expect("payload checked"))
    } else {
        error
    }
}
pub(crate) struct FrameInput<R> {
    input: BufReader<Take<R>>,
    pub(crate) position: u64,
}
impl<R: Read> FrameInput<R> {
    pub(crate) fn new(source: R, limit: u64) -> Self {
        Self {
            input: BufReader::new(source.take(limit)),
            position: 0,
        }
    }
    pub(crate) fn byte(&mut self) -> io::Result<u8> {
        let mut byte = [0];
        self.read_exact(&mut byte)?;
        Ok(byte[0])
    }
    pub(crate) fn peek(&mut self) -> io::Result<Option<u8>> {
        loop {
            match self.input.fill_buf() {
                Err(error)
                    if error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none() =>
                {
                    continue;
                }
                Err(error) => return Err(terminal(error)),
                Ok(bytes) => return Ok(bytes.first().copied()),
            }
        }
    }
    pub(crate) fn skip(&mut self, mut count: u64) -> io::Result<()> {
        let mut buffer = [0; 65536];
        while count != 0 {
            let n = count.min(buffer.len() as u64) as usize;
            self.read_exact(&mut buffer[..n])?;
            count -= n as u64;
        }
        Ok(())
    }
}
impl<R: Read> Read for FrameInput<R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let n = self.input.read(bytes).map_err(terminal)?;
        self.position = self
            .position
            .checked_add(n as u64)
            .ok_or_else(|| io::Error::other("frame_position_overflow"))?;
        Ok(n)
    }
}
