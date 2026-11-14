use super::controlled::{ControlledStream, Filter};
use std::{
    cell::Cell,
    io::{self, Cursor, Read, Seek, SeekFrom, Write},
    rc::Rc,
};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};
fn sample(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/streams")
            .join(name),
    )
    .unwrap()
}
#[test]
fn owned_and_borrowed_admission_preserve_typed_seek_failures_and_exact_rewind() {
    struct Source {
        error: Option<ScanError>,
        end: bool,
        wrong_start: bool,
    }
    impl Read for Source {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            unreachable!()
        }
    }
    impl Seek for Source {
        fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
            let end = matches!(at, SeekFrom::End(0));
            if end == self.end {
                return Err(self.error.map_or_else(
                    || io::Error::from(io::ErrorKind::Unsupported),
                    io::Error::other,
                ));
            }
            Ok(if end { 4 } else { u64::from(self.wrong_start) })
        }
    }
    impl ReadSource for Source {}
    let cancel = Cancellation::default();
    for end in [false, true] {
        for error in [
            None,
            Some(ScanError::Limit),
            Some(ScanError::Cancelled),
            Some(ScanError::SourceChanged),
        ] {
            let expected = error.unwrap_or(ScanError::Io);
            let mut source = Source {
                error,
                end,
                wrong_start: false,
            };
            assert_eq!(
                ControlledStream::new(&mut source, &cancel, Filter::Identity, 4, 4).err(),
                Some(expected)
            );
            assert_eq!(
                ControlledStream::new_owned(
                    Box::new(Source {
                        error,
                        end,
                        wrong_start: false
                    }),
                    &cancel,
                    Filter::Identity,
                    4,
                    4
                )
                .err(),
                Some(expected)
            );
        }
    }
    struct Wrong;
    impl Read for Wrong {
        fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
            unreachable!()
        }
    }
    impl Seek for Wrong {
        fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
            Ok(if matches!(at, SeekFrom::End(0)) { 4 } else { 1 })
        }
    }
    impl ReadSource for Wrong {}
    assert_eq!(
        ControlledStream::new_owned(Box::new(Wrong), &cancel, Filter::Identity, 4, 4).err(),
        Some(ScanError::SourceChanged)
    );
}
#[test]
fn owned_filters_share_exact_output_quota_and_do_not_resume_after_exhaustion() {
    let phrase = b"abracadabra abracadabra\n".repeat(6);
    let mut gzip = flate2::write::GzEncoder::new(Vec::new(), Default::default());
    gzip.write_all(&phrase).unwrap();
    for (filter, bytes) in [
        (Filter::Identity, phrase.clone()),
        (Filter::Gzip, gzip.finish().unwrap()),
        (Filter::Bzip2, sample("phrase.bz2")),
        (Filter::Lzma, sample("phrase.lzma")),
        (Filter::Lzip, sample("phrase.lz")),
        (Filter::Xz, sample("phrase-crc64.xz")),
        (Filter::Lz4, sample("phrase.lz4")),
        (Filter::Zstd, sample("phrase.zst")),
    ] {
        let control = Cancellation::default();
        let budget = Rc::new(Cell::new(2 * phrase.len() as u64));
        for _ in 0..2 {
            let mut stream = ControlledStream::new_owned(
                Box::new(Cursor::new(bytes.clone())),
                &control,
                filter,
                1 << 20,
                1 << 20,
            )
            .unwrap()
            .with_output_budget(budget.clone());
            let mut output = Vec::new();
            stream.read_to_end(&mut output).unwrap();
            assert_eq!(output, phrase);
        }
        assert_eq!(budget.get(), 0);
        let mut overflow = ControlledStream::new_owned(
            Box::new(Cursor::new(bytes)),
            &control,
            filter,
            1 << 20,
            1 << 20,
        )
        .unwrap()
        .with_output_budget(budget.clone());
        assert!(overflow.read(&mut [0]).is_err());
        assert_eq!(overflow.failure(), Some(ScanError::Limit));
        budget.set(1 << 20);
        assert!(overflow.read(&mut [0]).is_err());
        assert_eq!(overflow.failure(), Some(ScanError::Limit));
    }
}
struct Fault {
    error: Option<ScanError>,
    count: bool,
    cursor: Cursor<Vec<u8>>,
}
impl Read for Fault {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        if self.count {
            return Ok(bytes.len() + 1);
        }
        Err(self
            .error
            .map_or_else(|| io::ErrorKind::Unsupported.into(), io::Error::other))
    }
}
impl Seek for Fault {
    fn seek(&mut self, from: SeekFrom) -> io::Result<u64> {
        self.cursor.seek(from)
    }
}
impl ReadSource for Fault {}
#[test]
fn owned_source_errors_preserve_typed_failures_and_reject_impossible_counts() {
    for (error, count, expected) in [
        (Some(ScanError::Limit), false, ScanError::Limit),
        (Some(ScanError::Cancelled), false, ScanError::Cancelled),
        (
            Some(ScanError::SourceChanged),
            false,
            ScanError::SourceChanged,
        ),
        (None, false, ScanError::Io),
        (None, true, ScanError::Io),
    ] {
        let control = Cancellation::default();
        let source = Fault {
            error,
            count,
            cursor: Cursor::new(vec![0]),
        };
        let mut stream =
            ControlledStream::new_owned(Box::new(source), &control, Filter::Identity, 1, 1)
                .unwrap();
        assert!(stream.read(&mut [0]).is_err());
        assert_eq!(stream.failure(), Some(expected));
        assert!(stream.read(&mut [0]).is_err());
    }
}
#[test]
fn owned_source_admission_and_cancellation_stay_independent_of_shared_quota() {
    let control = Cancellation::default();
    assert!(matches!(
        ControlledStream::new_owned(
            Box::new(Cursor::new(vec![1, 2])),
            &control,
            Filter::Identity,
            1,
            2
        ),
        Err(ScanError::Limit)
    ));
    let budget = Rc::new(Cell::new(2));
    let mut stream = ControlledStream::new_owned(
        Box::new(Cursor::new(vec![1, 2])),
        &control,
        Filter::Identity,
        2,
        2,
    )
    .unwrap()
    .with_output_budget(budget.clone());
    assert_eq!(stream.read(&mut [0]).unwrap(), 1);
    control.cancel();
    assert!(stream.read(&mut [0]).is_err());
    assert_eq!(stream.failure(), Some(ScanError::Cancelled));
    assert_eq!(budget.get(), 1);
}
