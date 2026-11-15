//! Bounded ARJ methods 1–4, over shared prefix/LZ primitives and explicit history profiles.
use crate::{
    bitstream::Bits,
    invalid,
    lzh::LzhReader,
    stream_io::{filled, preserve_io_error, resource},
};
use std::io::{self, Read};
const WINDOW: usize = 16384;
const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_WORK: u64 = 96 * 1024 * 1024 * 1024;
/// Method 0 belongs to container Store handling, not this compressed reader.
pub fn arj_memory_required(method: u8) -> io::Result<u64> {
    match method {
        1..=3 => Ok(crate::lzh::arj_memory_required()),
        4 => Ok(WINDOW as u64 + 32 * 1024),
        _ => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "unsupported_arj_method",
        )),
    }
}
#[allow(
    clippy::large_enum_variant,
    reason = "Both variants embed an 8 KiB input buffer; a 350-byte table difference does not justify heap allocation"
)]
enum Body<R, P: FnMut() -> io::Result<()>> {
    Static(LzhReader<R, P>),
    Fast(Fast<R, P>),
}
/// One exact compressed ARJ member. The container retains size, CRC and output authority.
pub struct ArjReader<R, P = fn() -> io::Result<()>>
where
    P: FnMut() -> io::Result<()>,
{
    body: Body<R, P>,
    failure: Option<io::ErrorKind>,
}
impl<R: Read> ArjReader<R> {
    /// Validate caller and private hard ceilings before allocating history.
    pub fn new(
        source: R,
        method: u8,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
    ) -> io::Result<Self> {
        Self::with_poll(source, method, packed, decoded, memory, work, || Ok(()))
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> ArjReader<R, P> {
    /// Cooperative controls include source-independent history and prefix work.
    pub fn with_poll(
        source: R,
        method: u8,
        packed: u64,
        decoded: u64,
        memory: u64,
        work: u64,
        poll: P,
    ) -> io::Result<Self> {
        if arj_memory_required(method)? > memory || packed > MAX_BYTES || decoded > MAX_BYTES {
            return Err(resource());
        }
        let body = if method == 4 {
            Body::Fast(Fast::new(source, packed, decoded, work, poll)?)
        } else {
            Body::Static(LzhReader::with_arj_poll(
                source, packed, decoded, memory, work, poll,
            )?)
        };
        Ok(Self {
            body,
            failure: None,
        })
    }
    /// Preserve actual work in a caller-owned ledger across members.
    pub fn work_remaining(&self) -> u64 {
        match &self.body {
            Body::Static(r) => r.work_remaining(),
            Body::Fast(r) => r.bits.work,
        }
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for ArjReader<R, P> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if let Some(kind) = self.failure {
            return Err(io::Error::new(kind, "terminal_arj_error"));
        }
        if output.is_empty() {
            return Ok(0);
        }
        let result = match &mut self.body {
            Body::Static(r) => r.read(output),
            Body::Fast(r) => r.read(output),
        };
        match result {
            Ok(n) => Ok(n),
            Err(error) => {
                let error = preserve_io_error(error);
                self.failure = Some(error.kind());
                Err(error)
            }
        }
    }
}
struct Fast<R, P> {
    bits: Bits<R, P, true>,
    history: Vec<u8>,
    position: usize,
    copying: usize,
    distance: usize,
    produced: u64,
    expected: u64,
    finished: bool,
}
impl<R: Read, P: FnMut() -> io::Result<()>> Fast<R, P> {
    fn new(source: R, packed: u64, decoded: u64, work: u64, mut poll: P) -> io::Result<Self> {
        poll().map_err(preserve_io_error)?;
        let mut bits = Bits::with_poll(source, packed, work.min(MAX_WORK), poll);
        bits.charge(WINDOW as u64)?;
        Ok(Self {
            bits,
            history: filled(WINDOW, 0)?,
            position: 0,
            copying: 0,
            distance: 0,
            produced: 0,
            expected: decoded,
            finished: false,
        })
    }
    fn extended(&mut self, mut width: u8, last: u8) -> io::Result<usize> {
        let mut base = 0usize;
        while width < last {
            if self.bits.get(1)? == 0 {
                break;
            }
            base += 1usize << width;
            width += 1;
        }
        Ok(base + self.bits.get(width)? as usize)
    }
}
impl<R: Read, P: FnMut() -> io::Result<()>> Read for Fast<R, P> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.finished {
            return Ok(0);
        }
        let mut count = 0;
        while count < output.len().min(8192) && self.produced < self.expected {
            let byte = if self.copying != 0 {
                self.bits.charge(1)?;
                self.copying -= 1;
                self.history[(self.position + WINDOW - self.distance) % WINDOW]
            } else {
                let length = self.extended(0, 7)?;
                if length == 0 {
                    self.bits.get(8)? as u8
                } else {
                    let length = length + 2;
                    let distance = self.extended(9, 13)? + 1;
                    if length as u64 > self.expected - self.produced
                        || distance > WINDOW
                        || distance as u64 > self.produced
                    {
                        return Err(invalid());
                    }
                    self.copying = length;
                    self.distance = distance;
                    continue;
                }
            };
            self.bits.charge(1)?;
            self.history[self.position] = byte;
            self.position = (self.position + 1) % WINDOW;
            output[count] = byte;
            count += 1;
            self.produced += 1;
        }
        if self.produced == self.expected {
            if self.copying != 0 {
                return Err(invalid());
            }
            self.bits.finish_ignored_padding()?;
            self.finished = true;
        }
        Ok(count)
    }
}
#[cfg(test)]
mod tests;
