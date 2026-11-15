use super::*;
use crate::{test_cases as cases, test_known as known};
use std::{
    cell::Cell,
    io::{Cursor, Seek},
    rc::Rc,
};
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/arc")
            .join(name),
    )
    .unwrap()
}
fn scan(bytes: &[u8]) -> Result<Index, ScanError> {
    inspect(
        &mut Cursor::new(bytes),
        &Cancellation::default(),
        ScanLimits::default(),
    )
}
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    work: u64,
    fail: bool,
}
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self.fail {
            return Err(DecodeError::Output);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn charge_auxiliary(&mut self, work: u64) -> Result<(), DecodeError> {
        self.work = self.work.checked_add(work).ok_or(DecodeError::Limit)?;
        Ok(())
    }
}
fn decoded(bytes: &[u8], index: &Index, at: usize) -> Result<Vec<u8>, DecodeError> {
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let mut sink = Sink::default();
    Reader::open(&mut source, index, &cancel)?.decode(at, &mut sink, &cancel)?;
    Ok(sink.bytes)
}
#[test]
fn every_owned_old_and_compressed_member_has_exact_framing_and_verified_bytes() {
    for case in cases::CASES {
        let bytes = fixture(case.name);
        let index = scan(&bytes).unwrap();
        assert_eq!(index.records.len(), 1);
        assert_eq!(index.source_size, bytes.len() as u64);
        assert_eq!(index.input_used, (bytes.len() - case.packed) as u64);
        let row = &index.records[0];
        assert_eq!(row.raw_name, b"HELLO.BIN");
        assert_eq!(
            (row.method, row.size, row.packed_size, row.crc16),
            (
                case.method,
                case.decoded as u64,
                case.packed as u64,
                case.crc
            )
        );
        assert_eq!(row.payload_offset, if case.method == 1 { 25 } else { 29 });
        assert!(row.payload_supported());
        assert!(case.requested == case.method || matches!(case.requested, 8 | 9));
        assert_eq!(
            decoded(&bytes, &index, 0).unwrap(),
            known::plain(case.kind),
            "{}",
            case.name
        );
    }
    let empty = scan(&[0x1a, 0]).unwrap();
    assert!(empty.records.is_empty());
    assert_eq!(empty.input_used, 2);
    assert_eq!(empty.metadata_digest, {
        let mut hash = xdzip_crypto::SecretSha256::new();
        hash.update(&[0x1a, 0]);
        *hash.finish()
    });
}
#[test]
fn physical_duplicates_preserve_selection_and_later_damage_cannot_publish_a_partial_index() {
    let first = fixture("m2-phrase.arc");
    let second = fixture("m8-random.arc");
    let mut bytes = first[..first.len() - 2].to_vec();
    let later = bytes.len();
    bytes.extend_from_slice(&second);
    let index = scan(&bytes).unwrap();
    assert_eq!(index.records.len(), 2);
    assert_eq!(index.records[0].header_offset, 0);
    assert_eq!(index.records[1].header_offset, later as u64);
    assert_eq!(decoded(&bytes, &index, 1).unwrap(), known::plain("random"));
    bytes[later] = 0;
    assert_eq!(scan(&bytes), Err(ScanError::Invalid));
}
#[test]
fn metadata_digest_binds_ignored_padding_and_forged_snapshots_before_output() {
    let cancel = Cancellation::default();
    let mut bytes = fixture("m2-phrase.arc");
    let original = scan(&bytes).unwrap();
    bytes[14] = b'X';
    let changed = scan(&bytes).unwrap();
    assert_eq!(original.records, changed.records);
    assert_eq!(original.input_used, changed.input_used);
    assert_ne!(original.metadata_digest, changed.metadata_digest);
    assert!(matches!(
        Reader::open(&mut Cursor::new(&bytes), &original, &cancel),
        Err(DecodeError::Integrity)
    ));
    let mut forged = changed;
    forged.records[0].size += 1;
    assert!(matches!(
        Reader::open(&mut Cursor::new(&bytes), &forged, &cancel),
        Err(DecodeError::Integrity)
    ));
}
#[test]
fn mandatory_crc_unknown_methods_sink_errors_and_bad_selection_are_terminal() {
    let cancel = Cancellation::default();
    for failure in 0..4 {
        let mut bytes = fixture("m2-phrase.arc");
        if failure == 0 {
            bytes[29] ^= 1;
        }
        if failure == 1 {
            bytes[1] = 14;
        }
        let index = scan(&bytes).unwrap();
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        let mut sink = Sink {
            fail: failure == 2,
            ..Default::default()
        };
        let at = if failure == 3 { 1 } else { 0 };
        let expected = match failure {
            0 | 3 => DecodeError::Integrity,
            1 => DecodeError::Unsupported,
            _ => DecodeError::Output,
        };
        assert_eq!(reader.decode(at, &mut sink, &cancel), Err(expected));
        assert_eq!(
            reader.decode(0, &mut Sink::default(), &cancel),
            Err(expected)
        );
        if failure == 1 {
            assert!(sink.bytes.is_empty());
            assert_eq!(index.records[0].method, 14);
        }
    }
}
#[test]
fn exact_metadata_quotas_and_cumulative_input_work_and_decoded_ledgers_do_not_refund() {
    let cancel = Cancellation::default();
    let bytes = fixture("m8-phrase.arc");
    let index = scan(&bytes).unwrap();
    assert!(
        inspect(
            &mut Cursor::new(&bytes),
            &cancel,
            ScanLimits {
                read_bytes: index.input_used,
                ..ScanLimits::default()
            }
        )
        .is_ok()
    );
    for limits in [
        ScanLimits {
            read_bytes: index.input_used - 1,
            ..ScanLimits::default()
        },
        ScanLimits {
            entries: 0,
            ..ScanLimits::default()
        },
        ScanLimits {
            name_bytes: 8,
            ..ScanLimits::default()
        },
        ScanLimits {
            total_name_bytes: 8,
            ..ScanLimits::default()
        },
    ] {
        assert_eq!(
            inspect(&mut Cursor::new(&bytes), &cancel, limits),
            Err(ScanError::Limit)
        );
    }
    let mut source = Cursor::new(&bytes);
    let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
    let before = reader.work;
    let mut sink = Sink::default();
    reader.decode(0, &mut sink, &cancel).unwrap();
    let spent = before - reader.work;
    assert_eq!(spent, sink.work);
    assert_eq!(
        reader.input,
        INPUT_LIMIT - index.input_used - index.records[0].packed_size
    );
    for kind in 0..3 {
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        match kind {
            0 => reader.input = index.records[0].packed_size,
            1 => reader.decoded = index.records[0].size,
            _ => reader.work = spent,
        }
        reader.decode(0, &mut Sink::default(), &cancel).unwrap();
        assert_eq!(
            reader.decode(0, &mut Sink::default(), &cancel),
            Err(DecodeError::Limit)
        );
        assert_eq!(
            reader.decode(0, &mut Sink::default(), &cancel),
            Err(DecodeError::Limit)
        );
    }
    let mut source = Cursor::new(&bytes);
    let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
    reader.work = spent - 1;
    assert_eq!(
        reader.decode(0, &mut Sink::default(), &cancel),
        Err(DecodeError::Limit)
    );
}
struct Source {
    bytes: Cursor<Vec<u8>>,
    error: Rc<Cell<Option<ScanError>>>,
    short: bool,
}
impl Read for Source {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if let Some(error) = self.error.get() {
            return Err(io::Error::other(error));
        }
        let n = if self.short {
            out.len().min(1)
        } else {
            out.len()
        };
        self.bytes.read(&mut out[..n])
    }
}
impl Seek for Source {
    fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(at)
    }
}
impl ReadSource for Source {}
#[test]
fn short_reads_and_typed_host_failures_preserve_host_authority_before_and_during_payload() {
    let cancel = Cancellation::default();
    let bytes = fixture("m9-phrase.arc");
    let error = Rc::new(Cell::new(None));
    let mut source = Source {
        bytes: Cursor::new(bytes.clone()),
        error: error.clone(),
        short: true,
    };
    let index = inspect(&mut source, &cancel, ScanLimits::default()).unwrap();
    let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
    let mut sink = Sink::default();
    reader.decode(0, &mut sink, &cancel).unwrap();
    assert_eq!(sink.bytes, known::plain("phrase"));
    drop(reader);
    for kind in [
        ScanError::Cancelled,
        ScanError::Limit,
        ScanError::Io,
        ScanError::SourceChanged,
        ScanError::Invalid,
        ScanError::Unsupported,
    ] {
        error.set(Some(kind));
        let expected = match kind {
            ScanError::Cancelled | ScanError::Limit | ScanError::SourceChanged | ScanError::Io => {
                kind
            }
            _ => ScanError::Io,
        };
        assert_eq!(
            inspect(&mut source, &cancel, ScanLimits::default()),
            Err(expected)
        );
        error.set(None);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        error.set(Some(kind));
        let expected = match kind {
            ScanError::Cancelled => DecodeError::Cancelled,
            ScanError::Limit => DecodeError::Limit,
            _ => DecodeError::SourceIo,
        };
        assert_eq!(
            reader.decode(0, &mut Sink::default(), &cancel),
            Err(expected)
        );
        error.set(None);
        assert_eq!(
            reader.decode(0, &mut Sink::default(), &cancel),
            Err(expected)
        );
    }
}
#[test]
fn truncated_names_sizes_terminators_and_mutations_cannot_escape_complete_admission() {
    let bytes = fixture("m2-phrase.arc");
    for length in [0, 1, 2, 15, 24, 28, bytes.len() - 2, bytes.len() - 1] {
        assert!(scan(&bytes[..length]).is_err());
    }
    let mut extra = bytes.clone();
    extra.push(0);
    assert!(scan(&extra).is_err());
    let mut mismatch = bytes.clone();
    mismatch[25] ^= 1;
    assert!(scan(&mismatch).is_err());
    let mut unnamed = bytes.clone();
    unnamed[2] = 0;
    assert!(scan(&unnamed).is_err());
    let mut unterminated = bytes.clone();
    unterminated[2..15].fill(b'A');
    assert!(scan(&unterminated).is_err());
    for at in 0..bytes.len().min(256) {
        let mut changed = bytes.clone();
        changed[at] ^= 0x80;
        if let Ok(index) = scan(&changed) {
            let _ = decoded(&changed, &index, 0);
        }
    }
}
#[test]
fn cancellation_stops_metadata_and_open_or_task_scopes_without_resume() {
    let bytes = fixture("m8-phrase.arc");
    let index = scan(&bytes).unwrap();
    let cancel = Cancellation::default();
    cancel.cancel();
    assert_eq!(
        inspect(&mut Cursor::new(&bytes), &cancel, ScanLimits::default()),
        Err(ScanError::Cancelled)
    );
    let opened = Cancellation::default();
    let task = Cancellation::default();
    let mut source = Cursor::new(&bytes);
    let mut reader = Reader::open(&mut source, &index, &opened).unwrap();
    task.cancel();
    assert_eq!(
        reader.decode(0, &mut Sink::default(), &task),
        Err(DecodeError::Cancelled)
    );
    assert_eq!(
        reader.decode(0, &mut Sink::default(), &opened),
        Err(DecodeError::Cancelled)
    );
}
