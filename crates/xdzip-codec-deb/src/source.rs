use std::{
    cell::{Cell, RefCell},
    io::{self, Read, Seek, SeekFrom},
    rc::Rc,
};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};

struct State<'a> {
    source: &'a mut dyn ReadSource,
    remaining: u64,
}
/// Thread-confined capabilities share actual I/O admission and a terminal failure.
pub(crate) struct Root<'a> {
    state: Rc<RefCell<State<'a>>>,
    failure: Rc<Cell<Option<ScanError>>>,
    control: &'a Cancellation,
    length: u64,
}
impl Clone for Root<'_> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
            failure: self.failure.clone(),
            control: self.control,
            length: self.length,
        }
    }
}
impl<'a> Root<'a> {
    pub(crate) fn new(
        source: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        length: u64,
        remaining: u64,
    ) -> Self {
        Self {
            state: Rc::new(RefCell::new(State { source, remaining })),
            failure: Rc::new(Cell::new(None)),
            control,
            length,
        }
    }
    fn stop(&self, error: ScanError) -> io::Error {
        let error = self.failure.get().unwrap_or(error);
        self.failure.set(Some(error));
        io::Error::other(error)
    }
    fn check(&self) -> io::Result<()> {
        if let Some(error) = self.failure.get() {
            return Err(self.stop(error));
        }
        self.control.check().map_err(|error| self.stop(error))
    }
    pub(crate) fn range(&self, start: u64, length: u64) -> Result<Range<'a>, ScanError> {
        self.check()
            .map_err(|error| ScanError::from_source_error(&error))?;
        start
            .checked_add(length)
            .filter(|end| *end <= self.length)
            .ok_or(ScanError::Invalid)?;
        Ok(Range {
            root: self.clone(),
            start,
            length,
            position: 0,
        })
    }
    /// The caller ends preparation only after constructing all retained readers.
    /// Prefetched bytes remain charged to preparation and grant no input credit.
    pub(crate) fn begin_delivery(&self, remaining: u64) -> Result<(), ScanError> {
        self.check()
            .map_err(|error| ScanError::from_source_error(&error))?;
        self.state
            .try_borrow_mut()
            .map_err(|_| ScanError::Io)?
            .remaining = remaining;
        Ok(())
    }
}
pub(crate) struct Range<'a> {
    root: Root<'a>,
    start: u64,
    length: u64,
    position: u64,
}
impl Read for Range<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        self.root.check()?;
        if bytes.is_empty() || self.position == self.length {
            return Ok(0);
        }
        let mut state = self
            .root
            .state
            .try_borrow_mut()
            .map_err(|_| self.root.stop(ScanError::Io))?;
        if state.remaining == 0 {
            return Err(self.root.stop(ScanError::Limit));
        }
        let take = bytes.len().min(
            (self.length - self.position)
                .min(state.remaining)
                .min(usize::MAX as u64) as usize,
        );
        let absolute = self.start + self.position;
        let actual = state
            .source
            .seek(SeekFrom::Start(absolute))
            .map_err(|error| self.root.stop(ScanError::from_source_error(&error)))?;
        if actual != absolute {
            return Err(self.root.stop(ScanError::SourceChanged));
        }
        let n = state
            .source
            .read(&mut bytes[..take])
            .map_err(|error| self.root.stop(ScanError::from_source_error(&error)))?;
        if n > take {
            return Err(self.root.stop(ScanError::Io));
        }
        if n == 0 {
            return Err(self.root.stop(ScanError::SourceChanged));
        }
        self.position += n as u64;
        state.remaining -= n as u64;
        Ok(n)
    }
}
impl Seek for Range<'_> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        self.root.check()?;
        let position = match offset {
            SeekFrom::Start(value) => i128::from(value),
            SeekFrom::End(value) => i128::from(self.length) + i128::from(value),
            SeekFrom::Current(value) => i128::from(self.position) + i128::from(value),
        };
        if !(0..=i128::from(self.length)).contains(&position) {
            return Err(self.root.stop(ScanError::Invalid));
        }
        self.position = position as u64;
        Ok(self.position)
    }
}
impl ReadSource for Range<'_> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    #[test]
    fn independent_cursors_remain_bounded_and_share_actual_input_admission() {
        let mut source = Cursor::new(b"outside|abcd|012345|outside".to_vec());
        let cancel = Cancellation::default();
        let root = Root::new(&mut source, &cancel, 26, 10);
        let mut a = root.range(8, 4).unwrap();
        let mut b = root.range(13, 6).unwrap();
        let mut bytes = [0; 2];
        a.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"ab");
        b.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"01");
        a.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"cd");
        b.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"23");
        b.read_exact(&mut bytes).unwrap();
        assert_eq!(&bytes, b"45");
        assert_eq!(a.read(&mut bytes).unwrap(), 0);
        assert_eq!(b.seek(SeekFrom::End(-6)).unwrap(), 0);
        assert_eq!(
            ScanError::from_source_error(&b.read(&mut bytes).unwrap_err()),
            ScanError::Limit
        );
        assert_eq!(root.begin_delivery(100), Err(ScanError::Limit));
        assert!(a.seek(SeekFrom::Start(0)).is_err());
    }
    #[test]
    fn invalid_spans_seek_overflow_cancellation_and_source_failures_cannot_escape() {
        let mut source = Cursor::new(b"abcdefgh".to_vec());
        let cancel = Cancellation::default();
        let root = Root::new(&mut source, &cancel, 8, 20);
        assert!(root.range(7, 2).is_err());
        assert!(root.range(u64::MAX, 1).is_err());
        let mut a = root.range(2, 3).unwrap();
        root.begin_delivery(3).unwrap();
        assert_eq!(
            a.seek(SeekFrom::End(i64::MIN))
                .unwrap_err()
                .get_ref()
                .unwrap()
                .to_string(),
            "invalid_archive"
        );
        assert!(a.read(&mut [0]).is_err());
        let mut source = Cursor::new(b"ab".to_vec());
        let cancel = Cancellation::default();
        let root = Root::new(&mut source, &cancel, 8, 20);
        let mut a = root.range(0, 8).unwrap();
        assert_eq!(a.read(&mut [0; 8]).unwrap(), 2);
        assert_eq!(
            ScanError::from_source_error(&a.read(&mut [0]).unwrap_err()),
            ScanError::SourceChanged
        );
        let mut source = Cursor::new(b"ab".to_vec());
        let cancel = Cancellation::default();
        let root = Root::new(&mut source, &cancel, 2, 20);
        let mut a = root.range(0, 2).unwrap();
        cancel.cancel();
        assert_eq!(
            ScanError::from_source_error(&a.read(&mut [0]).unwrap_err()),
            ScanError::Cancelled
        );
    }
    #[test]
    fn typed_errors_impossible_counts_and_borrow_conflicts_are_terminal() {
        struct Bad {
            error: Option<ScanError>,
            impossible: bool,
        }
        impl Read for Bad {
            fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
                if let Some(error) = self.error {
                    Err(io::Error::other(error))
                } else if self.impossible {
                    Ok(bytes.len() + 1)
                } else {
                    Err(io::Error::from(io::ErrorKind::Unsupported))
                }
            }
        }
        impl Seek for Bad {
            fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
                if let SeekFrom::Start(at) = offset {
                    Ok(at)
                } else {
                    unreachable!()
                }
            }
        }
        impl ReadSource for Bad {}
        for (error, impossible, expected) in [
            (Some(ScanError::Limit), false, ScanError::Limit),
            (Some(ScanError::Cancelled), false, ScanError::Cancelled),
            (None, false, ScanError::Io),
            (None, true, ScanError::Io),
        ] {
            let mut bad = Bad { error, impossible };
            let cancel = Cancellation::default();
            let root = Root::new(&mut bad, &cancel, 4, 10);
            let mut a = root.range(0, 4).unwrap();
            assert_eq!(
                ScanError::from_source_error(&a.read(&mut [0]).unwrap_err()),
                expected
            );
            assert_eq!(root.begin_delivery(100), Err(expected));
        }
        let mut source = Cursor::new(b"ab".to_vec());
        let cancel = Cancellation::default();
        let root = Root::new(&mut source, &cancel, 2, 10);
        let mut a = root.range(0, 2).unwrap();
        let borrow = root.state.borrow_mut();
        assert_eq!(
            ScanError::from_source_error(&a.read(&mut [0]).unwrap_err()),
            ScanError::Io
        );
        drop(borrow);
        assert!(a.read(&mut [0]).is_err());
    }
}
