use super::*;
use std::io::{Cursor, Seek};
#[path = "../../xdzip-compression/src/pm2/fixture_cases.rs"]
mod fixture_cases;
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/pma")
            .join(name),
    )
    .unwrap()
}
fn scan(bytes: &[u8]) -> Result<Index, ScanError> {
    inspect_pma(
        &mut Cursor::new(bytes),
        &Cancellation::default(),
        ScanLimits::default(),
    )
}
fn plain(name: &str, size: usize) -> Vec<u8> {
    match name {
        "pm2-spaces.pma" => vec![b' '; size],
        "pm2-zeros.pma" => vec![0; size],
        "pm0-allbytes.pma" | "pm2-allbytes.pma" => (0..=255).cycle().take(size).collect(),
        "pm2-deep28.pma" => (32..=39).rev().cycle().take(size).collect(),
        _ => [
            (0..=255).collect::<Vec<u8>>(),
            b"abracadabra abracadabra\n".repeat(128),
        ]
        .concat()
        .into_iter()
        .cycle()
        .take(size)
        .collect(),
    }
}
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    fail: Option<DecodeError>,
}
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if let Some(error) = self.fail {
            return Err(error);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}
#[test]
fn complete_original_and_authored_pma_members_use_exact_ranges_crc_and_physical_padding() {
    let cancel = Cancellation::default();
    for &(name, method, header, packed, decoded, crc) in fixture_cases::CASES {
        let bytes = fixture(name);
        let index = scan(&bytes).unwrap();
        assert_eq!(index.source_size, bytes.len() as u64);
        assert_eq!(index.input_used, (bytes.len() - packed) as u64);
        if method == 255 {
            assert!(index.records.is_empty());
            continue;
        }
        assert_eq!(index.records.len(), 1);
        let record = &index.records[0];
        assert_eq!(record.method.as_slice(), format!("-pm{method}-").as_bytes());
        assert_eq!(record.raw_name, b"HELLO.BIN");
        assert_eq!(record.payload_offset, header as u64);
        assert_eq!(record.packed_size, packed as u64);
        assert_eq!(record.size, decoded as u64);
        assert_eq!(record.crc16, Some(crc));
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open_pma(&mut source, &index, &cancel).unwrap();
        let mut sink = Sink::default();
        reader.decode(0, &mut sink, &cancel).unwrap();
        assert_eq!(sink.bytes, plain(name, decoded), "{name}");
    }
}
#[test]
fn only_pma_record_fill_or_an_exact_zero_terminator_can_end_the_container() {
    for name in [
        "empty.pma",
        "pm0-allbytes.pma",
        "pm2-boundary-512.pma",
        "pm2-deep28.pma",
    ] {
        let bytes = fixture(name);
        let index = scan(&bytes).unwrap();
        let term = index
            .records
            .last()
            .map_or(0, |r| (r.payload_offset + r.packed_size) as usize);
        assert_eq!(bytes[term], 0);
        assert_eq!(scan(&bytes[..=term]).unwrap().records, index.records);
        if term + 1 < bytes.len() {
            assert_eq!(
                inspect(
                    &mut Cursor::new(&bytes),
                    &Cancellation::default(),
                    ScanLimits::default()
                ),
                Err(ScanError::Invalid)
            );
            for value in [0, 1, 0x19, 0x1b, 0xff] {
                let mut bad = bytes.clone();
                bad[term + 1] = value;
                assert!(scan(&bad).is_err());
            }
            assert!(scan(&bytes[..bytes.len() - 1]).is_err());
        }
        let mut extra = bytes.clone();
        extra.extend([0x1a; 128]);
        assert!(scan(&extra).is_err());
        let mut extra = bytes.clone();
        extra.push(0x1a);
        assert!(scan(&extra).is_err());
        if term > 0 {
            assert!(scan(&bytes[..term]).is_err());
        }
        let mut missing = bytes;
        missing[term] = 0x1a;
        assert!(scan(&missing).is_err());
    }
    assert!(scan(&[]).is_err());
    assert!(scan(&[0; 128]).is_err());
}
#[test]
fn duplicate_names_keep_physical_order_and_later_damage_or_forged_snapshots_never_partially_succeed()
 {
    let cancel = Cancellation::default();
    let first = fixture("pm0-allbytes.pma");
    let index = scan(&first).unwrap();
    let end = (index.records[0].payload_offset + index.records[0].packed_size) as usize;
    let mut second = first[..end].to_vec();
    second[33..].fill(b'A');
    let crc = xdzip_compression::crc16(&second[33..]);
    second[31..33].copy_from_slice(&crc.to_le_bytes());
    second[1] = second[2..33].iter().fold(0u8, |s, b| s.wrapping_add(*b));
    let mut combined = first[..end].to_vec();
    combined.extend_from_slice(&second);
    combined.push(0);
    combined.resize(combined.len().next_multiple_of(128), 0x1a);
    let index = scan(&combined).unwrap();
    assert_eq!(index.records.len(), 2);
    assert_eq!(index.records[0].raw_name, index.records[1].raw_name);
    assert_eq!(index.records[1].header_offset, end as u64);
    let mut source = Cursor::new(&combined);
    let mut reader = Reader::open_pma(&mut source, &index, &cancel).unwrap();
    let mut sink = Sink::default();
    reader.decode(1, &mut sink, &cancel).unwrap();
    assert_eq!(sink.bytes, vec![b'A'; 256]);
    let mut forged = index.clone();
    forged.records[1].header_offset += 1;
    assert!(matches!(
        Reader::open_pma(&mut Cursor::new(&combined), &forged, &cancel),
        Err(DecodeError::Integrity)
    ));
    combined[end + 1] ^= 1;
    assert!(scan(&combined).is_err());
}
#[test]
fn metadata_input_name_entry_budgets_include_record_fill_before_index_publication() {
    let bytes = fixture("pm2-boundary-512.pma");
    let index = scan(&bytes).unwrap();
    let cancel = Cancellation::default();
    let mut limits = ScanLimits {
        read_bytes: index.input_used,
        ..ScanLimits::default()
    };
    assert_eq!(
        inspect_pma(&mut Cursor::new(&bytes), &cancel, limits).unwrap(),
        index
    );
    limits.read_bytes -= 1;
    assert_eq!(
        inspect_pma(&mut Cursor::new(&bytes), &cancel, limits),
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
            inspect_pma(&mut Cursor::new(&bytes), &cancel, limits),
            Err(ScanError::Limit)
        );
    }
    cancel.cancel();
    assert_eq!(
        inspect_pma(&mut Cursor::new(&bytes), &cancel, ScanLimits::default()),
        Err(ScanError::Cancelled)
    );
}
#[test]
fn crc_sink_and_cumulative_input_output_work_failures_poison_without_new_credit() {
    let cancel = Cancellation::default();
    let bytes = fixture("pm2-boundary-512.pma");
    let index = scan(&bytes).unwrap();
    let mut source = Cursor::new(&bytes);
    let mut reader = Reader::open_pma(&mut source, &index, &cancel).unwrap();
    reader.input = index.records[0].packed_size;
    reader.decoded = index.records[0].size;
    let mut sink = Sink::default();
    reader.decode(0, &mut sink, &cancel).unwrap();
    let used = WORK_LIMIT - reader.work;
    assert_eq!((reader.input, reader.decoded), (0, 0));
    assert_eq!(
        reader.decode(0, &mut sink, &cancel),
        Err(DecodeError::Limit)
    );
    assert_eq!(sink.bytes.len(), 512);
    let mut source = Cursor::new(&bytes);
    let mut reader = Reader::open_pma(&mut source, &index, &cancel).unwrap();
    reader.work = used;
    reader.decode(0, &mut Sink::default(), &cancel).unwrap();
    assert_eq!(reader.work, 0);
    let mut source = Cursor::new(&bytes);
    let mut reader = Reader::open_pma(&mut source, &index, &cancel).unwrap();
    reader.work = used - 1;
    assert_eq!(
        reader.decode(0, &mut Sink::default(), &cancel),
        Err(DecodeError::Limit)
    );
    let mut source = Cursor::new(&bytes);
    let mut reader = Reader::open_pma(&mut source, &index, &cancel).unwrap();
    let mut sink = Sink {
        fail: Some(DecodeError::Output),
        ..Sink::default()
    };
    assert_eq!(
        reader.decode(0, &mut sink, &cancel),
        Err(DecodeError::Output)
    );
    sink.fail = None;
    assert_eq!(
        reader.decode(0, &mut sink, &cancel),
        Err(DecodeError::Output)
    );
    let mut bad = bytes;
    bad[31] ^= 1;
    bad[1] = bad[2..33].iter().fold(0u8, |s, b| s.wrapping_add(*b));
    let index = scan(&bad).unwrap();
    let mut source = Cursor::new(&bad);
    let mut reader = Reader::open_pma(&mut source, &index, &cancel).unwrap();
    assert_eq!(
        reader.decode(0, &mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        reader.decode(0, &mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
}
struct Short {
    bytes: Cursor<Vec<u8>>,
    bare: bool,
    error: Option<ScanError>,
}
impl Read for Short {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if let Some(error) = self.error {
            return Err(io::Error::new(io::ErrorKind::Interrupted, error));
        }
        if self.bare {
            self.bare = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let count = out.len().min(3);
        self.bytes.read(&mut out[..count])
    }
}
impl Seek for Short {
    fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(at)
    }
}
impl ReadSource for Short {}
#[test]
fn short_metadata_and_typed_interruptions_keep_the_original_failure_categories() {
    let bytes = fixture("pm2-boundary-512.pma");
    let cancel = Cancellation::default();
    let expected = scan(&bytes).unwrap();
    let mut source = Short {
        bytes: Cursor::new(bytes),
        bare: true,
        error: None,
    };
    assert_eq!(
        inspect_pma(&mut source, &cancel, ScanLimits::default()).unwrap(),
        expected
    );
    for error in [
        ScanError::Cancelled,
        ScanError::Limit,
        ScanError::Io,
        ScanError::SourceChanged,
    ] {
        source.error = Some(error);
        assert_eq!(
            inspect_pma(&mut source, &cancel, ScanLimits::default()),
            Err(error)
        );
    }
}
