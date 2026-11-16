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
            .join("../../tests/fixtures/arj")
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
fn digest(bytes: &[u8]) -> String {
    let mut hash = xdzip_crypto::SecretSha256::new();
    hash.update(bytes);
    hash.finish().iter().map(|b| format!("{b:02x}")).collect()
}
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    charged: u64,
    error: Option<DecodeError>,
}
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if let Some(error) = self.error {
            return Err(error);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn charge_auxiliary(&mut self, work: u64) -> Result<(), DecodeError> {
        self.charged = self.charged.checked_add(work).ok_or(DecodeError::Limit)?;
        Ok(())
    }
}
fn refresh(bytes: &mut [u8], at: usize) {
    let size = u16::from_le_bytes(bytes[at + 2..at + 4].try_into().unwrap()) as usize;
    let crc = xdzip_compression::crc32(&bytes[at + 4..at + 4 + size]);
    bytes[at + 4 + size..at + 8 + size].copy_from_slice(&crc.to_le_bytes());
}
fn decode(bytes: &[u8], index: &Index, at: usize) -> Result<Vec<u8>, DecodeError> {
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let mut sink = Sink::default();
    Reader::open(&mut source, index, &cancel)?.decode(at, &mut sink, &cancel)?;
    Ok(sink.bytes)
}
#[test]
fn all_owned_native_records_validate_complete_metadata_ranges_and_required_crc() {
    for case in cases::CASES {
        let bytes = fixture(case.name);
        let index = scan(&bytes).unwrap();
        assert_eq!(index.records.len(), 1);
        assert_eq!(index.source_size, bytes.len() as u64);
        assert_eq!(index.input_used, (bytes.len() - case.packed) as u64);
        let row = &index.records[0];
        assert_eq!(row.method, case.method);
        assert!(case.method == 0 || case.method == case.requested);
        assert_eq!(
            (row.payload_offset, row.packed_size, row.size),
            (case.offset as u64, case.packed as u64, case.decoded as u64)
        );
        assert_eq!(row.raw_name, b"HELLO.BIN");
        assert_eq!(row.crc32, case.crc);
        assert_eq!(row.first_size, 46);
        assert_eq!(row.host, 11);
        assert_eq!(row.extended_bytes, 0);
        assert!(row.payload_supported());
        assert_eq!(
            decode(&bytes, &index, 0).unwrap(),
            known::plain(case.kind),
            "{}",
            case.name
        );
    }
}
fn replace_header(bytes: &[u8], at: usize, body: &[u8]) -> Vec<u8> {
    let old = u16::from_le_bytes(bytes[at + 2..at + 4].try_into().unwrap()) as usize;
    let mut result = bytes[..at].to_vec();
    result.extend_from_slice(&[0x60, 0xea]);
    result.extend_from_slice(&(body.len() as u16).to_le_bytes());
    result.extend_from_slice(body);
    result.extend_from_slice(&xdzip_compression::crc32(body).to_le_bytes());
    result.extend_from_slice(&bytes[at + 8 + old..]);
    result
}
#[test]
fn old_first_header_widths_and_exact_empty_archive_keep_the_same_payload_identity() {
    let original = fixture("m1-phrase.arj");
    for (width, identity) in [
        (
            30usize,
            "27287d9d989f6e4b1b9b805febce261fa10186508d86263f6f5e3a5826111b38",
        ),
        (
            34,
            "c64f8b7c6690f8d2939412a7b8e381e4b3e04799629f34080b2f098526fbfc75",
        ),
        (
            46,
            "ea3d4cda03e51d37d91697e2d5bc69847316bb531946d1d01d326d220f41ecf0",
        ),
    ] {
        let index = scan(&original).unwrap();
        let at = index.records[0].header_offset as usize;
        let size = u16::from_le_bytes(original[at + 2..at + 4].try_into().unwrap()) as usize;
        let mut body = [
            original[at + 4..at + 4 + width].to_vec(),
            original[at + 4 + 46..at + 4 + size].to_vec(),
        ]
        .concat();
        body[0] = width as u8;
        let bytes = replace_header(&original, at, &body);
        let size = u16::from_le_bytes(bytes[2..4].try_into().unwrap()) as usize;
        let mut main = [bytes[4..34].to_vec(), bytes[4 + 34..4 + size].to_vec()].concat();
        main[0] = 30;
        let bytes = replace_header(&bytes, 0, &main);
        assert_eq!(digest(&bytes), identity);
        let index = scan(&bytes).unwrap();
        assert_eq!(index.records[0].first_size, width as u8);
        assert_eq!(decode(&bytes, &index, 0).unwrap(), known::plain("phrase"));
    }
    let index = scan(&original).unwrap();
    let empty = [
        original[..index.records[0].header_offset as usize].to_vec(),
        vec![0x60, 0xea, 0, 0],
    ]
    .concat();
    let index = scan(&empty).unwrap();
    assert!(index.records.is_empty());
    assert_eq!(index.input_used, empty.len() as u64);
    for mut bad in [empty.clone(), original.clone()] {
        bad.push(0);
        assert!(scan(&bad).is_err());
    }
    assert!(scan(&[0x60, 0xea, 0, 0]).is_err());
}
#[test]
fn later_damage_never_publishes_a_partial_index_and_physical_duplicate_selection_is_stable() {
    let first = fixture("m0-phrase.arj");
    let index = scan(&first).unwrap();
    let begin = index.records[0].header_offset as usize;
    let mut second = first[begin..first.len() - 4].to_vec();
    let relative = index.records[0].payload_offset as usize - begin;
    second[relative..].fill(b'A');
    let crc = xdzip_compression::crc32(&second[relative..]);
    second[4 + 20..4 + 24].copy_from_slice(&crc.to_le_bytes());
    refresh(&mut second, 0);
    let two = [
        first[..first.len() - 4].to_vec(),
        second,
        vec![0x60, 0xea, 0, 0],
    ]
    .concat();
    let index = scan(&two).unwrap();
    assert_eq!(index.records.len(), 2);
    assert_eq!(index.records[0].raw_name, index.records[1].raw_name);
    assert_ne!(
        index.records[0].payload_offset,
        index.records[1].payload_offset
    );
    assert_eq!(decode(&two, &index, 1).unwrap(), vec![b'A'; 3072]);
    let mut bad = two;
    let at = index.records[1].header_offset as usize;
    bad[at + 10] ^= 1;
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
}
#[test]
fn opaque_extension_crc_digest_and_count_are_validated_before_content_without_restoration() {
    let original = fixture("m1-phrase.arj");
    let index = scan(&original).unwrap();
    let at = index.records[0].payload_offset as usize - 2;
    let opaque = b"Z\0owned opaque metadata";
    let extension = [
        (opaque.len() as u16).to_le_bytes().to_vec(),
        opaque.to_vec(),
        xdzip_compression::crc32(opaque).to_le_bytes().to_vec(),
    ]
    .concat();
    let mut bytes = original.clone();
    bytes.splice(at..at, extension.clone());
    assert_eq!(
        digest(&bytes),
        "d3dbabebd5f271c14309e88bd2fefac74a280a811c8034b66a689254fcc4db66"
    );
    let updated = scan(&bytes).unwrap();
    assert_eq!(updated.records[0].extended_bytes, opaque.len() as u64);
    assert_eq!(
        updated.records[0].payload_offset,
        index.records[0].payload_offset + extension.len() as u64
    );
    assert_eq!(decode(&bytes, &updated, 0).unwrap(), known::plain("phrase"));
    bytes[at + 2] ^= 1;
    assert_eq!(scan(&bytes), Err(ScanError::Invalid));
    let mut many = original.clone();
    many.splice(at..at, extension.repeat(256));
    assert!(scan(&many).is_ok());
    let mut many = original;
    many.splice(at..at, extension.repeat(257));
    assert_eq!(scan(&many), Err(ScanError::Limit));
}
#[test]
fn ignored_metadata_changes_and_forged_snapshots_fail_before_output() {
    let bytes = fixture("m2-phrase.arj");
    let index = scan(&bytes).unwrap();
    let mut changed = bytes.clone();
    changed[11] ^= 1;
    refresh(&mut changed, 0);
    let updated = scan(&changed).unwrap();
    assert_eq!(updated.records, index.records);
    assert_eq!(updated.input_used, index.input_used);
    assert_ne!(updated.metadata_digest, index.metadata_digest);
    let cancel = Cancellation::default();
    let mut source = Cursor::new(&changed);
    assert!(matches!(
        Reader::open(&mut source, &index, &cancel),
        Err(DecodeError::Integrity)
    ));
    let mut forged = index.clone();
    forged.records[0].method = 3;
    assert!(matches!(
        Reader::open(&mut Cursor::new(&bytes), &forged, &cancel),
        Err(DecodeError::Integrity)
    ));
}
#[test]
fn unsupported_profiles_remain_explicit_and_crc_or_output_failures_poison_without_resume() {
    let original = fixture("m1-phrase.arj");
    let index = scan(&original).unwrap();
    let at = index.records[0].header_offset as usize;
    for (field, value) in [(2, 12), (3, 255), (4, 17), (5, 7), (6, 6), (28, 1), (29, 1)] {
        let mut bytes = original.clone();
        bytes[at + 4 + field] = value;
        refresh(&mut bytes, at);
        let index = scan(&bytes).unwrap();
        assert!(!index.records[0].payload_supported());
        assert_eq!(decode(&bytes, &index, 0), Err(DecodeError::Unsupported));
    }
    for flag in [4, 8, 64, 128] {
        let mut bytes = original.clone();
        bytes[8] |= flag;
        refresh(&mut bytes, 0);
        assert_eq!(scan(&bytes), Err(ScanError::Unsupported));
    }
    let cancel = Cancellation::default();
    let mut bad = original.clone();
    bad[at + 24] ^= 1;
    refresh(&mut bad, at);
    let bad_index = scan(&bad).unwrap();
    let mut source = Cursor::new(&bad);
    let mut reader = Reader::open(&mut source, &bad_index, &cancel).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        reader.decode(0, &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        reader.decode(0, &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
    let mut source = Cursor::new(&original);
    let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
    let mut sink = Sink {
        error: Some(DecodeError::Output),
        ..Sink::default()
    };
    assert_eq!(
        reader.decode(0, &mut sink, &cancel),
        Err(DecodeError::Output)
    );
    sink.error = None;
    assert_eq!(
        reader.decode(0, &mut sink, &cancel),
        Err(DecodeError::Output)
    );
}
#[test]
fn metadata_read_name_entry_and_payload_ledgers_count_all_actual_work_without_refunds() {
    let bytes = fixture("m3-phrase.arj");
    let index = scan(&bytes).unwrap();
    let cancel = Cancellation::default();
    let mut limits = ScanLimits {
        read_bytes: index.input_used,
        ..ScanLimits::default()
    };
    assert_eq!(
        inspect(&mut Cursor::new(&bytes), &cancel, limits).unwrap(),
        index
    );
    limits.read_bytes -= 1;
    assert_eq!(
        inspect(&mut Cursor::new(&bytes), &cancel, limits),
        Err(ScanError::Limit)
    );
    for limits in [
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
    let initial = reader.work;
    let mut sink = Sink::default();
    reader.decode(0, &mut sink, &cancel).unwrap();
    let spent = initial - reader.work;
    assert_eq!(sink.charged, spent);
    assert_eq!(
        reader.input,
        INPUT_LIMIT - index.input_used - index.records[0].packed_size
    );
    reader.decode(0, &mut sink, &cancel).unwrap();
    assert_eq!(initial - reader.work, 2 * spent);
    assert_eq!(reader.decoded, INPUT_LIMIT - 6144);
    for kind in 0..3 {
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        match kind {
            0 => reader.input = index.records[0].packed_size,
            1 => reader.decoded = 3072,
            _ => reader.work = 2 * spent - 1,
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
}
struct Source {
    bytes: Cursor<Vec<u8>>,
    bare: bool,
    error: Rc<Cell<Option<ScanError>>>,
}
impl Read for Source {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        assert!(out.len() <= 8192);
        if let Some(error) = self.error.get() {
            return Err(io::Error::new(io::ErrorKind::Interrupted, error));
        }
        if self.bare {
            self.bare = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let n = out.len().min(3);
        self.bytes.read(&mut out[..n])
    }
}
impl Seek for Source {
    fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(to)
    }
}
impl ReadSource for Source {}
#[test]
fn bounded_short_io_and_typed_metadata_or_payload_errors_remain_terminal_host_failures() {
    let bytes = fixture("m4-phrase.arj");
    let cancel = Cancellation::default();
    let index = scan(&bytes).unwrap();
    let mut source = Source {
        bytes: Cursor::new(bytes),
        bare: true,
        error: Rc::new(Cell::new(None)),
    };
    assert_eq!(
        inspect(&mut source, &cancel, ScanLimits::default()).unwrap(),
        index
    );
    let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
    let mut sink = Sink::default();
    reader.decode(0, &mut sink, &cancel).unwrap();
    assert_eq!(sink.bytes, known::plain("phrase"));
    drop(reader);
    for error in [
        ScanError::Cancelled,
        ScanError::Limit,
        ScanError::Io,
        ScanError::SourceChanged,
        ScanError::Invalid,
        ScanError::Unsupported,
    ] {
        source.error.set(Some(error));
        assert_eq!(
            inspect(&mut source, &cancel, ScanLimits::default()),
            Err(match error {
                ScanError::Invalid | ScanError::Unsupported => ScanError::Io,
                e => e,
            })
        );
    }
    source.error.set(None);
    for error in [
        ScanError::Cancelled,
        ScanError::Limit,
        ScanError::Io,
        ScanError::SourceChanged,
        ScanError::Invalid,
        ScanError::Unsupported,
    ] {
        let state = source.error.clone();
        state.set(None);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        state.set(Some(error));
        let expected = match error {
            ScanError::Cancelled => DecodeError::Cancelled,
            ScanError::Limit => DecodeError::Limit,
            _ => DecodeError::SourceIo,
        };
        let mut sink = Sink::default();
        assert_eq!(reader.decode(0, &mut sink, &cancel), Err(expected));
        state.set(None);
        assert_eq!(reader.decode(0, &mut sink, &cancel), Err(expected));
        assert!(sink.bytes.is_empty());
    }
    let cancelled = Cancellation::default();
    cancelled.cancel();
    assert_eq!(
        inspect(&mut source, &cancelled, ScanLimits::default()),
        Err(ScanError::Cancelled)
    );
}
#[test]
fn unix_times_use_only_the_host_calendar_and_host_errors_never_become_format_misses() {
    use xdzip_codec_api::time::{Calendar, Timestamp};
    struct CalendarSource {
        inner: Source,
        result: Result<Option<Calendar>, ScanError>,
        calls: Cell<usize>,
    }
    impl Read for CalendarSource {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            self.inner.read(out)
        }
    }
    impl Seek for CalendarSource {
        fn seek(&mut self, to: SeekFrom) -> io::Result<u64> {
            self.inner.seek(to)
        }
    }
    impl ReadSource for CalendarSource {
        fn local_calendar(&self, timestamp: Timestamp) -> Result<Option<Calendar>, ScanError> {
            assert_eq!(timestamp, Timestamp::UnixSeconds(1_700_000_000));
            self.calls.set(self.calls.get() + 1);
            self.result
        }
    }
    let mut bytes = fixture("m1-phrase.arj");
    let at = scan(&bytes).unwrap().records[0].header_offset as usize;
    bytes[at + 7] = 2;
    bytes[at + 12..at + 16].copy_from_slice(&1_700_000_000u32.to_le_bytes());
    bytes[at + 30..at + 32].copy_from_slice(&0x81a4u16.to_le_bytes());
    refresh(&mut bytes, at);
    let calendar = Calendar {
        year: 2024,
        month: 2,
        day: 29,
        hour: 12,
        minute: 34,
        second: 56,
    };
    let cancel = Cancellation::default();
    for result in [
        Ok(Some(calendar)),
        Ok(None),
        Err(ScanError::Unsupported),
        Err(ScanError::Cancelled),
        Err(ScanError::Limit),
    ] {
        let mut source = CalendarSource {
            inner: Source {
                bytes: Cursor::new(bytes.clone()),
                bare: false,
                error: Rc::new(Cell::new(None)),
            },
            result,
            calls: Cell::new(0),
        };
        let index = inspect(&mut source, &cancel, ScanLimits::default());
        assert_eq!(source.calls.get(), 1);
        match result {
            Ok(value) => {
                let index = index.unwrap();
                assert_eq!(
                    index.records[0].modified_dos,
                    value.and_then(Calendar::wrapping_dos).unwrap_or(0)
                );
                let mut sink = Sink::default();
                Reader::open(&mut source, &index, &cancel)
                    .unwrap()
                    .decode(0, &mut sink, &cancel)
                    .unwrap();
                assert_eq!(sink.bytes, known::plain("phrase"));
            }
            Err(ScanError::Unsupported) => assert_eq!(index, Err(ScanError::Io)),
            Err(error) => assert_eq!(index, Err(error)),
        }
    }
}
#[test]
fn cancellation_and_out_of_range_selection_poison_the_instance_before_any_output() {
    let bytes = fixture("m4-phrase.arj");
    let index = scan(&bytes).unwrap();
    let cancel = Cancellation::default();
    for cancelled in [true, false] {
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open(&mut source, &index, &cancel).unwrap();
        let mut sink = Sink::default();
        let task = Cancellation::default();
        let (at, expected) = if cancelled {
            task.cancel();
            (0, DecodeError::Cancelled)
        } else {
            (1, DecodeError::Integrity)
        };
        assert_eq!(reader.decode(at, &mut sink, &task), Err(expected));
        assert_eq!(reader.decode(0, &mut sink, &cancel), Err(expected));
        assert!(sink.bytes.is_empty());
    }
}
#[test]
fn malformed_lengths_strings_crcs_and_bounded_metadata_mutations_cannot_publish_or_panic() {
    let original = fixture("m1-phrase.arj");
    let index = scan(&original).unwrap();
    let file = index.records[0].header_offset as usize;
    for at in [0, file] {
        let size = u16::from_le_bytes(original[at + 2..at + 4].try_into().unwrap()) as usize;
        let mut bad = original.clone();
        bad[at + 4 + size] ^= 1;
        assert_eq!(scan(&bad), Err(ScanError::Invalid));
        let mut bad = original.clone();
        bad[at + 2..at + 4].copy_from_slice(&2601u16.to_le_bytes());
        assert_eq!(scan(&bad), Err(ScanError::Limit));
        let mut bad = original.clone();
        bad[at + 2..at + 4].copy_from_slice(&29u16.to_le_bytes());
        assert_eq!(scan(&bad), Err(ScanError::Invalid));
        let mut bad = original.clone();
        bad[at + 4] = 29;
        refresh(&mut bad, at);
        assert_eq!(scan(&bad), Err(ScanError::Invalid));
        let mut bad = original.clone();
        bad[at + 4] = 31;
        refresh(&mut bad, at);
        assert_eq!(scan(&bad), Err(ScanError::Unsupported));
    }
    let mut bad = original.clone();
    bad[file + 4 + 46 + 9] = b'X';
    refresh(&mut bad, file);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
    let mut bad = original.clone();
    bad[file + 16..file + 20].copy_from_slice(&u32::MAX.to_le_bytes());
    refresh(&mut bad, file);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
    let mut bad = original.clone();
    bad[file + 28..file + 30].copy_from_slice(&100u16.to_le_bytes());
    refresh(&mut bad, file);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
    for at in 0..512.min(original.len() * 8) {
        let mut bad = original.clone();
        bad[at / 8] ^= 1 << (at % 8);
        assert!(
            std::panic::catch_unwind(|| scan(&bad)).is_ok(),
            "metadata mutation {at}"
        );
    }
}
#[test]
fn directory_and_legacy_text_reference_profiles_preserve_archive_bytes_without_host_translation() {
    let mut text = fixture("m1-phrase.arj");
    let at = scan(&text).unwrap().records[0].header_offset as usize;
    text[at + 10] = 1;
    refresh(&mut text, at);
    assert_eq!(
        digest(&text),
        "0637d05e6d92b7c0efdf5c954f3928c4977fd46cbbb7ff91064edf5566c34e63"
    );
    let index = scan(&text).unwrap();
    assert_eq!(index.records[0].kind, 1);
    assert_eq!(decode(&text, &index, 0).unwrap(), known::plain("phrase"));
    let mut directory = fixture("m0-empty.arj");
    let at = scan(&directory).unwrap().records[0].header_offset as usize;
    directory[at + 10] = 3;
    directory[at + 30..at + 32].copy_from_slice(&16u16.to_le_bytes());
    refresh(&mut directory, at);
    assert_eq!(
        digest(&directory),
        "883ad27bdbb60c959a15fa1b2f27b6f95b1b2f5c4e3f0d4a9ce0010494a41e8f"
    );
    let index = scan(&directory).unwrap();
    assert!(index.records[0].payload_supported());
    assert_eq!(decode(&directory, &index, 0).unwrap(), b"");
}
