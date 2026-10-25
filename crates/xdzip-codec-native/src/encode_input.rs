//! Frozen input delivery and one real EOF check per ordinary input stream.
use super::*;
use std::collections::BTreeMap;

pub(super) struct Inputs<'a> {
    input: &'a dyn EncodeInput,
    streams: BTreeMap<usize, Box<dyn Read + 'a>>,
    consumed: Vec<u64>,
    complete: Vec<bool>,
}
fn read(
    source: &mut dyn Read,
    bytes: &mut [u8],
    control: &Cancellation,
) -> Result<usize, EncodeError> {
    let count = source_call(control, || source.read(bytes)).map_err(encode_error)?;
    if count > bytes.len() {
        return Err(EncodeError::Source);
    }
    Ok(count)
}
impl<'a> Inputs<'a> {
    pub(super) fn new(input: &'a dyn EncodeInput) -> Self {
        Self {
            input,
            streams: BTreeMap::new(),
            consumed: vec![0; input.entries().len()],
            complete: input.entries().iter().map(|row| row.directory).collect(),
        }
    }
    pub(super) fn read(
        &mut self,
        index: usize,
        count: usize,
        control: &Cancellation,
    ) -> Result<Vec<u8>, EncodeError> {
        control.check().map_err(|_| EncodeError::Cancelled)?;
        let entry = self
            .input
            .entries()
            .get(index)
            .ok_or(EncodeError::Invalid)?;
        if count == 0 || count > protocol::CHUNK || entry.directory {
            return Err(EncodeError::Invalid);
        }
        if self.complete[index] {
            return Ok(Vec::new());
        }
        let remaining = entry
            .size
            .checked_sub(self.consumed[index])
            .ok_or(EncodeError::Invalid)?;
        if let std::collections::btree_map::Entry::Vacant(slot) = self.streams.entry(index) {
            slot.insert(self.input.open(index)?);
        }
        let stream = self.streams.get_mut(&index).ok_or(EncodeError::Source)?;
        let mut bytes = vec![0; count.min(usize::try_from(remaining).unwrap_or(usize::MAX))];
        let n = if bytes.is_empty() {
            0
        } else {
            read(stream, &mut bytes, control)?
        };
        bytes.truncate(n);
        if n == 0 && remaining != 0 {
            return Err(EncodeError::Source);
        }
        self.consumed[index] += n as u64;
        if self.consumed[index] == entry.size {
            // InputReader validates its held file version on a nonempty EOF
            // read. Never synthesize that EOF from the declared size.
            if read(stream, &mut [0; 1], control)? != 0 {
                return Err(EncodeError::Source);
            }
            self.complete[index] = true;
            self.streams.remove(&index);
        }
        Ok(bytes)
    }
    pub(super) fn finish(
        &mut self,
        inherited: &[Option<usize>],
        control: &Cancellation,
    ) -> Result<(), EncodeError> {
        for index in 0..self.complete.len() {
            if !self.complete[index]
                && self.input.entries()[index].size == 0
                && inherited.get(index).copied().flatten().is_none()
            {
                self.read(index, 1, control)?;
            }
        }
        if self
            .complete
            .iter()
            .enumerate()
            .any(|(i, complete)| !complete && inherited.get(i).copied().flatten().is_none())
        {
            Err(EncodeError::Source)
        } else {
            Ok(())
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;
    struct Fixture {
        entries: Vec<EncodeEntry>,
        bytes: Vec<u8>,
        eof_error: Option<ScanError>,
        eof_kind: io::ErrorKind,
        bare: bool,
        opens: Cell<u32>,
        eof_reads: Cell<u32>,
    }
    impl Fixture {
        fn new(declared: u64, actual: &[u8]) -> Self {
            Self {
                entries: vec![EncodeEntry {
                    name: "owned".into(),
                    directory: false,
                    size: declared,
                    modified_unix: 0,
                }],
                bytes: actual.to_vec(),
                eof_error: None,
                eof_kind: io::ErrorKind::Interrupted,
                bare: false,
                opens: Cell::new(0),
                eof_reads: Cell::new(0),
            }
        }
    }
    struct Stream<'a> {
        fixture: &'a Fixture,
        at: usize,
        bare: bool,
    }
    impl Read for Stream<'_> {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            if self.bare {
                self.bare = false;
                return Err(io::ErrorKind::Interrupted.into());
            }
            if self.at == self.fixture.bytes.len() {
                self.fixture.eof_reads.set(self.fixture.eof_reads.get() + 1);
                if let Some(error) = self.fixture.eof_error {
                    return Err(io::Error::new(self.fixture.eof_kind, error));
                }
                return Ok(0);
            }
            let n = out.len().min(self.fixture.bytes.len() - self.at);
            out[..n].copy_from_slice(&self.fixture.bytes[self.at..self.at + n]);
            self.at += n;
            Ok(n)
        }
    }
    impl EncodeInput for Fixture {
        fn entries(&self) -> &[EncodeEntry] {
            &self.entries
        }
        fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            self.opens.set(self.opens.get() + 1);
            Ok(Box::new(Stream {
                fixture: self,
                at: 0,
                bare: self.bare,
            }))
        }
    }
    #[test]
    fn exact_and_zero_sized_sources_receive_a_real_eof_check_once() {
        for bytes in [b"".as_slice(), b"abc"] {
            let fixture = Fixture::new(bytes.len() as u64, bytes);
            let mut inputs = Inputs::new(&fixture);
            assert_eq!(
                inputs.read(0, 8192, &Cancellation::default()).unwrap(),
                bytes
            );
            assert_eq!(inputs.finish(&[], &Cancellation::default()), Ok(()));
            assert!(
                inputs
                    .read(0, 1, &Cancellation::default())
                    .unwrap()
                    .is_empty()
            );
            assert_eq!((fixture.opens.get(), fixture.eof_reads.get()), (1, 1));
        }
        let fixture = Fixture::new(0, b"");
        Inputs::new(&fixture)
            .finish(&[], &Cancellation::default())
            .unwrap();
        assert_eq!((fixture.opens.get(), fixture.eof_reads.get()), (1, 1));
    }
    #[test]
    fn truncated_extra_and_changed_after_last_chunk_cannot_finish() {
        for bytes in [b"ab".as_slice(), b"abcd"] {
            let fixture = Fixture::new(3, bytes);
            let mut inputs = Inputs::new(&fixture);
            let first = inputs.read(0, 8192, &Cancellation::default());
            let result = first.and_then(|_| inputs.read(0, 8192, &Cancellation::default()));
            assert_eq!(result, Err(EncodeError::Source));
            assert_eq!(
                inputs.finish(&[], &Cancellation::default()),
                Err(EncodeError::Source)
            );
        }
        let mut fixture = Fixture::new(3, b"abc");
        fixture.eof_error = Some(ScanError::SourceChanged);
        let mut inputs = Inputs::new(&fixture);
        assert_eq!(
            inputs.read(0, 8192, &Cancellation::default()),
            Err(EncodeError::Source)
        );
        assert_eq!(fixture.eof_reads.get(), 1);
        assert_eq!(
            inputs.finish(&[], &Cancellation::default()),
            Err(EncodeError::Source)
        );
    }
    #[test]
    fn only_bare_interrupted_retries_typed_faults_preserve_their_class() {
        for kind in [io::ErrorKind::Interrupted, io::ErrorKind::Other] {
            for (error, expected) in [
                (ScanError::Io, EncodeError::Source),
                (ScanError::SourceChanged, EncodeError::Source),
                (ScanError::Invalid, EncodeError::Invalid),
                (ScanError::Unsupported, EncodeError::Unsupported),
                (ScanError::Limit, EncodeError::Limit),
                (ScanError::Cancelled, EncodeError::Cancelled),
                (ScanError::MissingVolume, EncodeError::Source),
                (ScanError::PasswordRequired, EncodeError::Unsupported),
                (ScanError::WrongPassword, EncodeError::Unsupported),
                (ScanError::PasswordOrIntegrity, EncodeError::Unsupported),
            ] {
                let mut fixture = Fixture::new(0, b"");
                fixture.eof_error = Some(error);
                fixture.eof_kind = kind;
                let mut inputs = Inputs::new(&fixture);
                assert_eq!(inputs.read(0, 1, &Cancellation::default()), Err(expected));
                assert_eq!(fixture.eof_reads.get(), 1);
            }
        }
        let mut fixture = Fixture::new(3, b"abc");
        fixture.bare = true;
        let mut inputs = Inputs::new(&fixture);
        assert_eq!(
            inputs.read(0, 8192, &Cancellation::default()).unwrap(),
            b"abc"
        );
        inputs.finish(&[], &Cancellation::default()).unwrap();
    }
    #[test]
    fn cancellation_and_invalid_requests_do_not_open_or_complete_inputs() {
        let fixture = Fixture::new(3, b"abc");
        let mut inputs = Inputs::new(&fixture);
        let control = Cancellation::default();
        for (index, count) in [(1, 1), (0, 0), (0, protocol::CHUNK + 1)] {
            assert_eq!(
                inputs.read(index, count, &control),
                Err(EncodeError::Invalid)
            );
        }
        control.cancel();
        assert_eq!(inputs.read(0, 1, &control), Err(EncodeError::Cancelled));
        assert_eq!(fixture.opens.get(), 0);
        assert_eq!(
            inputs.finish(&[], &Cancellation::default()),
            Err(EncodeError::Source)
        );
    }
}
