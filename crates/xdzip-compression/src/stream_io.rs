//! Bounded buffered input and terminal typed I/O errors for streaming codecs.
use crate::invalid;
use std::io::{self, Read, Take};

pub(crate) const READ_QUANTUM: usize = 8192;

pub(crate) fn resource() -> io::Error {
    io::Error::new(io::ErrorKind::OutOfMemory, "resource_limit")
}

pub(crate) fn retryable_io(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::Interrupted && error.get_ref().is_none()
}

pub(crate) fn preserve_io_error(error: io::Error) -> io::Error {
    if error.kind() != io::ErrorKind::Interrupted || error.get_ref().is_none() {
        return error;
    }
    // std Read/Write convenience methods automatically retry Interrupted. A
    // payload-bearing interruption instead represents the caller's terminal
    // condition. Keep its concrete error object available for downcasting.
    let payload = error.into_inner().expect("payload checked above");
    io::Error::other(payload)
}

pub(crate) fn filled<T: Clone>(count: usize, value: T) -> io::Result<Vec<T>> {
    let mut result = Vec::new();
    result.try_reserve_exact(count).map_err(|_| resource())?;
    result.resize(count, value);
    Ok(result)
}

/// Input buffering never crosses the admitted packed span. `consumed` measures
/// logical bytes, rather than physical read-ahead inside that span.
pub(crate) struct Input<R> {
    source: Take<R>,
    buffer: [u8; READ_QUANTUM],
    start: usize,
    end: usize,
    pub(crate) consumed: u64,
    pub(crate) packed: u64,
}
impl<R: Read> Input<R> {
    pub(crate) fn new(source: R, packed: u64) -> Self {
        Self {
            source: source.take(packed),
            buffer: [0; READ_QUANTUM],
            start: 0,
            end: 0,
            consumed: 0,
            packed,
        }
    }
    pub(crate) fn byte(&mut self, boundary: u64) -> io::Result<u8> {
        if self.consumed >= boundary || boundary > self.packed {
            return Err(invalid());
        }
        if self.start == self.end {
            loop {
                match self.source.read(&mut self.buffer) {
                    Err(error) if retryable_io(&error) => continue,
                    Err(error) => return Err(preserve_io_error(error)),
                    Ok(0) => return Err(invalid()),
                    Ok(count) => {
                        self.start = 0;
                        self.end = count;
                        break;
                    }
                }
            }
        }
        let byte = self.buffer[self.start];
        self.start += 1;
        self.consumed += 1;
        Ok(byte)
    }
    pub(crate) fn word(&mut self) -> io::Result<u16> {
        Ok(u16::from_be_bytes([
            self.byte(self.packed)?,
            self.byte(self.packed)?,
        ]))
    }
}
