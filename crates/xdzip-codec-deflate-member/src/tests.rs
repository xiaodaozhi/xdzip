use super::*;
use std::io::{Cursor, Write};
use xdzip_compression::{DeflateWriter, crc32};

#[derive(Default)]
struct Sink {
    output: Vec<u8>,
    charged: u64,
}

impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.output.extend_from_slice(bytes);
        Ok(())
    }

    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.charged += bytes;
        Ok(())
    }
}

fn stored(bytes: &[u8]) -> Member {
    Member {
        offset: 0,
        packed_size: bytes.len() as u64,
        size: bytes.len() as u64,
        crc32: crc32(bytes),
        method: Method::Store,
    }
}

#[test]
fn cumulative_budgets_are_charged_and_terminal() {
    let bytes = b"abc";
    let cancel = Cancellation::default();
    for (input, decoded, work) in [(6, 9, 18), (9, 6, 18), (9, 9, 12)] {
        let mut source = Cursor::new(bytes);
        let mut reader = MemberReader::new(&mut source, &cancel, INPUT_LIMIT - input).unwrap();
        reader.decoded = decoded;
        reader.work = work;
        let mut sink = Sink::default();
        for _ in 0..2 {
            reader
                .decode(Ok(stored(bytes)), &mut sink, &cancel)
                .unwrap();
        }
        assert_eq!(
            reader.decode(Ok(stored(bytes)), &mut sink, &cancel),
            Err(DecodeError::Limit)
        );
        assert_eq!(
            reader.decode(Ok(stored(b"")), &mut sink, &cancel),
            Err(DecodeError::Limit)
        );
        assert_eq!(sink.output, b"abcabc");
        assert_eq!(sink.charged, 12);
    }
}

#[test]
fn cancellation_at_either_scope_precedes_selection_and_sticks() {
    for cancel_open in [false, true] {
        let bytes = b"abc";
        let mut source = Cursor::new(bytes);
        let opened = Cancellation::default();
        let task = Cancellation::default();
        let mut reader = MemberReader::new(&mut source, &opened, 0).unwrap();
        let mut sink = Sink::default();
        if cancel_open {
            opened.cancel();
        } else {
            task.cancel();
        }
        assert_eq!(
            reader.decode(Err(DecodeError::Unsupported), &mut sink, &task),
            Err(DecodeError::Cancelled)
        );
        assert_eq!(
            reader.decode(Ok(stored(bytes)), &mut sink, &Cancellation::default()),
            Err(DecodeError::Cancelled)
        );
        assert!(sink.output.is_empty());
        assert_eq!(sink.charged, 0);
    }
}

#[test]
fn deflate_requires_exact_termination_and_source_short_reads_fail() {
    let cancel = Cancellation::default();
    let bytes = b"verified content";
    let mut packed = Vec::new();
    let mut writer = DeflateWriter::new(&mut packed, 6).unwrap();
    writer.write_all(bytes).unwrap();
    writer.finish().unwrap();
    packed.push(0);
    let member = Member {
        packed_size: packed.len() as u64,
        method: Method::Deflate,
        ..stored(bytes)
    };
    let mut source = Cursor::new(packed);
    let mut sink = Sink::default();
    let mut reader = MemberReader::new(&mut source, &cancel, 0).unwrap();
    assert_eq!(
        reader.decode(Ok(member), &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );

    let mut source = Cursor::new(&bytes[..bytes.len() - 1]);
    let mut reader = MemberReader::new(&mut source, &cancel, 0).unwrap();
    assert_eq!(
        reader.decode(Ok(stored(bytes)), &mut Sink::default(), &cancel),
        Err(DecodeError::SourceIo)
    );
}
