//! Independent four-input BCJ2 decoder. Each port has a separate admitted span.
use std::io::{self, Read};
use xdzip_codec_api::Cancellation;
use xdzip_codec_api::payload::DecodeError;

const BUFFER: usize = 4096;
struct Port<R> {
    source: R,
    left: u64,
    consumed: u64,
    bytes: [u8; BUFFER],
    at: usize,
    end: usize,
}
impl<R: Read> Port<R> {
    fn byte(&mut self) -> io::Result<u8> {
        if self.at == self.end {
            if self.left == 0 {
                return Err(invalid());
            }
            let count = self.left.min(BUFFER as u64) as usize;
            loop {
                match self.source.read(&mut self.bytes[..count]) {
                    Ok(0) => return Err(invalid()),
                    Ok(n) if n <= count => {
                        self.left -= n as u64;
                        self.consumed += n as u64;
                        self.at = 0;
                        self.end = n;
                        break;
                    }
                    Ok(_) => return Err(invalid()),
                    Err(error) if retry(&error) => continue,
                    Err(error) => return Err(stable(error)),
                }
            }
        }
        let byte = self.bytes[self.at];
        self.at += 1;
        Ok(byte)
    }
    fn exhausted(&self) -> bool {
        self.left == 0 && self.at == self.end
    }
    fn word(&mut self) -> io::Result<u32> {
        Ok(u32::from_be_bytes([
            self.byte()?,
            self.byte()?,
            self.byte()?,
            self.byte()?,
        ]))
    }
}

/// A bounded reader for MAIN, CALL, JUMP and RC, in that order. The caller must
/// also finish upstream nodes and validate the enclosing packed/folder CRCs.
pub struct Bcj2Reader<'a, R: Read> {
    ports: [Port<R>; 4],
    probabilities: [u16; 258],
    range: u32,
    code: u32,
    initialized: bool,
    pending: [u8; 4],
    pending_at: usize,
    previous: u8,
    produced: u64,
    expected: u64,
    control: &'a Cancellation,
    failed: bool,
    finished: bool,
}
impl<'a, R: Read> Bcj2Reader<'a, R> {
    /// Check side-stream widths and the exact output equation before input I/O.
    pub fn new(
        sources: [R; 4],
        lengths: [u64; 4],
        expected: u64,
        control: &'a Cancellation,
    ) -> io::Result<Self> {
        if !lengths[1].is_multiple_of(4)
            || !lengths[2].is_multiple_of(4)
            || lengths[3] < 5
            || lengths[0]
                .checked_add(lengths[1])
                .and_then(|n| n.checked_add(lengths[2]))
                != Some(expected)
        {
            return Err(invalid());
        }
        let mut indices = 0;
        let ports = sources.map(|source| {
            let left = lengths[indices];
            indices += 1;
            Port {
                source,
                left,
                consumed: 0,
                bytes: [0; BUFFER],
                at: 0,
                end: 0,
            }
        });
        Ok(Self {
            ports,
            probabilities: [1024; 258],
            range: u32::MAX,
            code: 0,
            initialized: false,
            pending: [0; 4],
            pending_at: 4,
            previous: 0,
            produced: 0,
            expected,
            control,
            failed: false,
            finished: false,
        })
    }
    /// Actual reads from the four separately bounded sources, including prefetch.
    pub fn consumed_bytes(&self) -> [u64; 4] {
        self.ports.each_ref().map(|port| port.consumed)
    }
    /// Bytes delivered to the caller, independent of modulo-32 branch positions.
    pub fn decoded_bytes(&self) -> u64 {
        self.produced
    }
    /// Check completion without discarding any pending or unread output.
    pub fn finish(&mut self) -> io::Result<()> {
        if self.failed {
            return Err(invalid());
        }
        let result = self.complete();
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(stable)
    }
    fn initialize(&mut self) -> io::Result<()> {
        check(self.control)?;
        if !self.initialized {
            if self.ports[3].byte()? != 0 {
                return Err(invalid());
            }
            self.code = self.ports[3].word()?;
            if self.code == u32::MAX {
                return Err(invalid());
            }
            self.initialized = true;
        }
        Ok(())
    }
    fn normalize(&mut self) -> io::Result<()> {
        if self.range < 1 << 24 {
            self.range <<= 8;
            self.code = (self.code << 8) | u32::from(self.ports[3].byte()?);
        }
        Ok(())
    }
    fn selection(&mut self, context: usize) -> io::Result<bool> {
        self.normalize()?;
        let probability = u32::from(self.probabilities[context]);
        let bound = (self.range >> 11) * probability;
        let converted = self.code >= bound;
        self.probabilities[context] = if converted {
            self.code -= bound;
            self.range -= bound;
            (probability - (probability >> 5)) as u16
        } else {
            self.range = bound;
            (probability + ((2048 - probability) >> 5)) as u16
        };
        // Normalization is part of the current decision, including the last one.
        self.normalize()?;
        Ok(converted)
    }
    fn complete(&mut self) -> io::Result<()> {
        self.initialize()?;
        if self.produced != self.expected
            || self.pending_at != 4
            || self.code != 0
            || self.ports.iter().any(|port| !port.exhausted())
        {
            return Err(invalid());
        }
        check(self.control)?;
        self.finished = true;
        Ok(())
    }
    fn next(&mut self) -> io::Result<u8> {
        check(self.control)?;
        if self.pending_at != 4 {
            let byte = self.pending[self.pending_at];
            self.pending_at += 1;
            self.previous = byte;
            self.produced += 1;
            return Ok(byte);
        }
        let byte = self.ports[0].byte()?;
        let context = if byte == 0xe8 {
            Some(2 + usize::from(self.previous))
        } else if byte == 0xe9 {
            Some(1)
        } else if self.previous == 0x0f && byte & 0xf0 == 0x80 {
            Some(0)
        } else {
            None
        };
        self.produced += 1;
        self.previous = byte;
        if let Some(context) = context
            && self.selection(context)?
        {
            if self.expected - self.produced < 4 {
                return Err(invalid());
            }
            let port = if byte == 0xe8 { 1 } else { 2 };
            let absolute = self.ports[port].word()?;
            self.pending = absolute
                .wrapping_sub((self.produced as u32).wrapping_add(4))
                .to_le_bytes();
            self.pending_at = 0;
        }
        Ok(byte)
    }
    fn read_inner(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.initialize()?;
        if self.finished {
            check(self.control)?;
            return Ok(0);
        }
        if output.is_empty() {
            return Ok(0);
        }
        if self.produced == self.expected {
            self.complete()?;
            return Ok(0);
        }
        let count = output
            .len()
            .min(8192)
            .min((self.expected - self.produced).min(8192) as usize);
        for target in &mut output[..count] {
            *target = self.next()?;
        }
        Ok(count)
    }
}
impl<R: Read> Read for Bcj2Reader<'_, R> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.failed {
            return Err(invalid());
        }
        let result = self.read_inner(output);
        if result.is_err() {
            self.failed = true;
        }
        result.map_err(stable)
    }
}
fn check(control: &Cancellation) -> io::Result<()> {
    control
        .check()
        .map_err(|_| io::Error::other(DecodeError::Cancelled))
}
fn invalid() -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, DecodeError::Integrity)
}
fn retry(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none()
}
fn stable(error: io::Error) -> io::Error {
    if error.kind() != io::ErrorKind::Interrupted || error.get_ref().is_none() {
        error
    } else {
        io::Error::other(error.into_inner().unwrap())
    }
}

#[cfg(test)]
#[path = "bcj2_tests.rs"]
mod tests;
