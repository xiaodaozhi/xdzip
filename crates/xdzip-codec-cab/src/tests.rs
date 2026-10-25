use super::*;
use std::collections::BTreeMap;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use xdzip_codec_api::payload::DecodedSink;

const NAMED: &[u8] = include_bytes!("../../../tests/fixtures/cab-iso/named.cab");
const NORMAL: &[u8] = include_bytes!("../../../tests/fixtures/cab-iso/normal_2files_1folder.cab");
const METHODS: &[u8] = include_bytes!("../../../tests/fixtures/cab-iso/mszip_lzx_qtm.cab");
const PARTS: [&[u8]; 5] = [
    include_bytes!("../../../tests/fixtures/cab-iso/cabd_multi_basic_pt1.cab"),
    include_bytes!("../../../tests/fixtures/cab-iso/cabd_multi_basic_pt2.cab"),
    include_bytes!("../../../tests/fixtures/cab-iso/cabd_multi_basic_pt3.cab"),
    include_bytes!("../../../tests/fixtures/cab-iso/cabd_multi_basic_pt4.cab"),
    include_bytes!("../../../tests/fixtures/cab-iso/cabd_multi_basic_pt5.cab"),
];
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    skipped: u64,
}
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn charge_skipped(&mut self, count: u64) -> Result<(), DecodeError> {
        self.skipped += count;
        Ok(())
    }
}
#[derive(Default)]
struct CountingSink {
    bytes: Vec<u8>,
    decoded: u64,
    credit: u64,
    precharged: u64,
    ordinary: u64,
    limit: Option<u64>,
}
impl CountingSink {
    fn charge_work(&mut self, count: u64) -> Result<(), DecodeError> {
        self.decoded = self
            .decoded
            .checked_add(count)
            .filter(|total| self.limit.is_none_or(|limit| *total <= limit))
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
}
impl DecodedSink for CountingSink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.charge_work(bytes.len() as u64)?;
        self.ordinary += bytes.len() as u64;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn charge_skipped(&mut self, count: u64) -> Result<(), DecodeError> {
        self.charge_work(count)?;
        self.credit += count;
        Ok(())
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.credit = self.credit.checked_sub(bytes.len() as u64).unwrap();
        self.precharged += bytes.len() as u64;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
}
struct Parts {
    bytes: Cursor<Vec<u8>>,
    lengths: Vec<u64>,
    names: Vec<Vec<u8>>,
    read: Arc<AtomicU64>,
}
impl Parts {
    fn new(parts: &[&[u8]]) -> Self {
        Self {
            bytes: Cursor::new(parts.concat()),
            lengths: parts.iter().map(|part| part.len() as u64).collect(),
            names: (1..=parts.len())
                .map(|i| format!("cabd_multi_basic_pt{i}.cab").into_bytes())
                .collect(),
            read: Arc::default(),
        }
    }
}
impl Read for Parts {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let count = self.bytes.read(output)?;
        self.read.fetch_add(count as u64, Ordering::Relaxed);
        Ok(count)
    }
}
impl Seek for Parts {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(offset)
    }
}
impl ReadSource for Parts {
    fn volume_lengths(&self) -> Option<&[u64]> {
        Some(&self.lengths)
    }
    fn volume_names(&self) -> Option<&[Vec<u8>]> {
        Some(&self.names)
    }
}
fn decode(source: &mut dyn ReadSource) -> BTreeMap<Vec<u8>, Vec<u8>> {
    let control = Cancellation::default();
    let rows = CabCodec
        .scan(source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(source, &rows, &control).unwrap();
    let mut result = BTreeMap::new();
    for row in rows {
        assert!(row.payload_supported(), "{}", row.display_name);
        let mut sink = Sink::default();
        payload.decode(&row, &mut sink, &control).unwrap();
        assert_eq!(sink.bytes.len() as u64, row.size);
        result.insert(row.name_bytes().to_vec(), sink.bytes);
    }
    result
}
#[test]
fn independently_generated_unicode_empty_and_store_records_extract() {
    let result = decode(&mut Cursor::new(NAMED));
    assert_eq!(
        result[b"hello.txt".as_slice()],
        b"xdzip read-only archive fixture\n"
    );
    assert_eq!(
        result["nested/中文.txt".as_bytes()],
        "Unicode payload: 中文\n".as_bytes()
    );
    assert!(result[b"empty.txt".as_slice()].is_empty());
    let result = decode(&mut Cursor::new(NORMAL));
    assert_eq!(result.len(), 2);
    assert!(result[b"hello.c".as_slice()].starts_with(b"#include <stdio.h>\r\n"));
}
#[test]
fn fixed_independent_three_algorithms_extract_exact_payloads() {
    let result = decode(&mut Cursor::new(METHODS));
    assert_eq!(
        result[b"mszip.txt".as_slice()],
        b"If you can read this, the MSZIP decompressor is working!\n"
    );
    assert_eq!(
        result[b"qtm.txt".as_slice()],
        b"If you can read this, the Quantum decompressor is working!\n"
    );
    let lzx = [
        b"-".repeat(65),
        b"\nIf you can read this, the LZX decompressor is working!\n".to_vec(),
        b"-".repeat(65),
        b"\n".to_vec(),
    ]
    .concat();
    assert_eq!(result[b"lzx.txt".as_slice()], lzx);
}
#[test]
fn five_volume_split_frame_coalesces_only_explicit_continuations() {
    let result = decode(&mut Parts::new(&PARTS));
    let all = (1..=5)
        .flat_map(|i| format!("This is the data from cabinet part {i}.\n").into_bytes())
        .collect::<Vec<_>>();
    assert_eq!(result.len(), 3);
    assert_eq!(result[b"test1.txt".as_slice()], all[..76]);
    assert_eq!(result[b"test2.txt".as_slice()], all[76..114]);
    assert_eq!(result[b"test3.txt".as_slice()], all[114..]);
}
#[test]
fn missing_wrong_and_inconsistent_volumes_do_not_publish_partial_rows() {
    let control = Cancellation::default();
    for parts in [&PARTS[..4], &PARTS[1..]] {
        assert!(
            CabCodec
                .scan(&mut Parts::new(parts), &control, ScanLimits::default())
                .is_err()
        );
    }
    let mut source = Parts::new(&PARTS);
    source.names[2] = b"wrong.cab".to_vec();
    assert!(
        CabCodec
            .scan(&mut source, &control, ScanLimits::default())
            .is_err()
    );
    let mut broken = PARTS[2].to_vec();
    broken[32] ^= 1;
    assert!(
        CabCodec
            .scan(
                &mut Parts::new(&[PARTS[0], PARTS[1], &broken, PARTS[3], PARTS[4]]),
                &control,
                ScanLimits::default()
            )
            .is_err()
    );
}
#[test]
fn checksum_corruption_is_payload_integrity_failure_and_poisons_reader() {
    let mut bytes = NORMAL.to_vec();
    let last = bytes.len() - 1;
    bytes[last] ^= 1;
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Sink::default(), &control),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        payload.decode(&rows[1], &mut Sink::default(), &control),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn snapshot_mutation_metadata_budget_and_cancel_fail_closed() {
    let control = Cancellation::default();
    let mut source = Cursor::new(NAMED.to_vec());
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    source.get_mut()[44] ^= 1;
    assert!(CabCodec.open_payload(&mut source, &rows, &control).is_err());
    let limits = ScanLimits {
        read_bytes: 36,
        ..ScanLimits::default()
    };
    assert_eq!(
        CabCodec.scan(&mut Cursor::new(NAMED), &control, limits),
        Err(ScanError::Limit)
    );
    control.cancel();
    assert_eq!(
        CabCodec.scan(&mut Cursor::new(NAMED), &control, ScanLimits::default()),
        Err(ScanError::Cancelled)
    );
}

struct FaultOnce<'a> {
    source: Cursor<&'a [u8]>,
    before_error: usize,
    failure: Option<io::Error>,
    cancel_on_error: Option<&'a Cancellation>,
    calls: usize,
    read: usize,
}
impl Read for FaultOnce<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.calls += 1;
        if self.before_error == 0
            && let Some(error) = self.failure.take()
        {
            if let Some(control) = self.cancel_on_error {
                control.cancel();
            }
            return Err(error);
        }
        let mut count = output.len().min(3);
        if self.failure.is_some() {
            count = count.min(self.before_error);
        }
        let read = self.source.read(&mut output[..count])?;
        self.before_error = self.before_error.saturating_sub(read);
        self.read += read;
        Ok(read)
    }
}
impl Seek for FaultOnce<'_> {
    fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
        self.source.seek(offset)
    }
}
impl ReadSource for FaultOnce<'_> {}
#[test]
fn typed_source_errors_survive_interrupted_io_after_partial_reads() {
    let control = Cancellation::default();
    for error in [
        ScanError::Cancelled,
        ScanError::Limit,
        ScanError::SourceChanged,
        ScanError::Io,
    ] {
        for kind in [io::ErrorKind::Other, io::ErrorKind::Interrupted] {
            for prefix in [0, 3] {
                let mut source = FaultOnce {
                    source: Cursor::new(NAMED),
                    before_error: prefix,
                    failure: Some(io::Error::new(kind, error)),
                    cancel_on_error: None,
                    calls: 0,
                    read: 0,
                };
                assert_eq!(
                    CabCodec.scan(&mut source, &control, ScanLimits::default()),
                    Err(error)
                );
                assert_eq!(source.read, prefix);
                assert_eq!(source.calls, if prefix == 0 { 1 } else { 2 });
            }
        }
    }
}
#[test]
fn bare_interruption_retries_without_resetting_partial_read_budget_or_cancellation() {
    let control = Cancellation::default();
    for prefix in [0, 3] {
        let mut source = FaultOnce {
            source: Cursor::new(NAMED),
            before_error: prefix,
            failure: Some(io::ErrorKind::Interrupted.into()),
            cancel_on_error: None,
            calls: 0,
            read: 0,
        };
        let rows = CabCodec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 3);
        assert!(source.read > prefix);
    }
    let mut source = FaultOnce {
        source: Cursor::new(NAMED),
        before_error: 3,
        failure: Some(io::ErrorKind::Interrupted.into()),
        cancel_on_error: None,
        calls: 0,
        read: 0,
    };
    {
        let mut reader = crate::source::Reader::new(&mut source, &control, 6).unwrap();
        assert_eq!(reader.bytes(0, 0, 6).unwrap(), NAMED[..6]);
        assert_eq!(reader.bytes(0, 6, 1), Err(ScanError::Limit));
    }
    assert_eq!((source.calls, source.read), (3, 6));
    source = FaultOnce {
        source: Cursor::new(NAMED),
        before_error: 3,
        failure: Some(io::ErrorKind::Interrupted.into()),
        cancel_on_error: Some(&control),
        calls: 0,
        read: 0,
    };
    assert_eq!(
        CabCodec.scan(&mut source, &control, ScanLimits::default()),
        Err(ScanError::Cancelled)
    );
    assert_eq!((source.calls, source.read), (2, 3));
}

fn checksum(bytes: &[u8]) -> u32 {
    let full = bytes.len() / 4 * 4;
    let mut sum = 0;
    for at in (0..full).step_by(4) {
        sum ^= u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
    }
    sum ^ bytes[full..]
        .iter()
        .fold(0u32, |word, byte| word * 256 + u32::from(*byte))
}
fn store(items: &[(Vec<u8>, Vec<u8>, u16)], reserve: bool) -> Vec<u8> {
    let mut rows = Vec::new();
    let mut content = Vec::new();
    for (name, bytes, attributes) in items {
        rows.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        rows.extend_from_slice(&(content.len() as u32).to_le_bytes());
        rows.extend_from_slice(&[0, 0, 0x21, 0x58, 0, 0]);
        rows.extend_from_slice(&attributes.to_le_bytes());
        rows.extend_from_slice(name);
        rows.push(0);
        content.extend_from_slice(bytes);
    }
    let header_length = if reserve { 43 } else { 36 };
    let file_offset = header_length + 8 + if reserve { 2 } else { 0 };
    let data_offset = file_offset + rows.len();
    let mut blocks = Vec::new();
    for chunk in content.chunks(32768) {
        let mut fields = Vec::from((chunk.len() as u16).to_le_bytes());
        fields.extend_from_slice(&(chunk.len() as u16).to_le_bytes());
        if reserve {
            fields.extend_from_slice(&[0xaa, 0xbb, 0xcc]);
        }
        blocks.extend_from_slice(&(checksum(&fields) ^ checksum(chunk)).to_le_bytes());
        blocks.extend_from_slice(&fields);
        blocks.extend_from_slice(chunk);
    }
    let mut header = vec![0; 36];
    header[..4].copy_from_slice(b"MSCF");
    header[8..12].copy_from_slice(&((data_offset + blocks.len()) as u32).to_le_bytes());
    header[16..20].copy_from_slice(&(file_offset as u32).to_le_bytes());
    header[24..30].copy_from_slice(&[3, 1, 1, 0, items.len() as u8, (items.len() >> 8) as u8]);
    if reserve {
        header[30] = 4;
        header.extend_from_slice(&[3, 0, 2, 3, 0x11, 0x22, 0x33]);
    }
    header.extend_from_slice(&(data_offset as u32).to_le_bytes());
    header.extend_from_slice(&(content.len().div_ceil(32768) as u16).to_le_bytes());
    header.extend_from_slice(&[0, 0]);
    if reserve {
        header.extend_from_slice(&[0x44, 0x55]);
    }
    [header, rows, blocks].concat()
}
#[test]
fn ordinary_duplicate_names_keep_identity_and_original_utf_declaration() {
    let bytes = store(
        &[
            (b"same.txt".to_vec(), b"first".to_vec(), 0),
            (b"same.txt".to_vec(), b"second".to_vec(), 0x80),
        ],
        false,
    );
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let (rows, flags) = CabCodec
        .scan_with_encoding_flags(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!(flags, [false, true]);
    assert_eq!(rows[0].name_encoding, rows[1].name_encoding);
    assert_eq!(
        rows.iter().map(|row| row.id.get()).collect::<Vec<_>>(),
        [1, 2]
    );
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    for (row, expected) in rows.iter().zip([b"first".as_slice(), b"second".as_slice()]) {
        let mut sink = Sink::default();
        payload.decode(row, &mut sink, &control).unwrap();
        assert_eq!(sink.bytes, expected);
    }
}
#[test]
fn reserved_regions_and_data_reserve_checksum_have_exact_boundaries() {
    let bytes = store(
        &[(
            b"reserved.txt".to_vec(),
            b"independent reserve payload".to_vec(),
            0,
        )],
        true,
    );
    assert_eq!(
        decode(&mut Cursor::new(&bytes))[b"reserved.txt".as_slice()],
        b"independent reserve payload"
    );
    let mut corrupt = bytes;
    let start = u32::from_le_bytes(corrupt[43..47].try_into().unwrap()) as usize;
    corrupt[start + 8] ^= 1;
    let control = Cancellation::default();
    let mut source = Cursor::new(corrupt);
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Sink::default(), &control),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn sequential_same_folder_files_read_a_shared_frame_once() {
    let items = (0..128)
        .map(|i| {
            (
                format!("file-{i:03}.bin").into_bytes(),
                vec![i as u8; 256],
                0,
            )
        })
        .collect::<Vec<_>>();
    let bytes = store(&items, false);
    let mut source = Parts::new(&[&bytes]);
    let read = source.read.clone();
    let control = Cancellation::default();
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    let baseline = read.load(Ordering::Relaxed);
    let mut sink = CountingSink::default();
    for (row, (_, expected, _)) in rows.iter().zip(&items) {
        sink.bytes.clear();
        payload.decode(row, &mut sink, &control).unwrap();
        assert_eq!(&sink.bytes, expected);
    }
    assert_eq!(read.load(Ordering::Relaxed) - baseline, 32768 + 8);
    assert_eq!(
        (sink.decoded, sink.precharged, sink.ordinary),
        (32768, 32768, 0)
    );
    assert_eq!(sink.credit, 0);
}
#[test]
fn nonaligned_files_charge_each_complete_frame_once_even_for_one_selection() {
    let items = [
        (b"first.bin".to_vec(), vec![b'A'; 17003], 0),
        (b"middle.bin".to_vec(), vec![b'B'; 80002], 0),
        (b"last.bin".to_vec(), vec![b'C'; 34067], 0),
    ];
    let bytes = store(&items, false);
    let control = Cancellation::default();
    let mut source = Cursor::new(&bytes);
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = CountingSink::default();
    for ((row, (_, expected, _)), work) in rows.iter().zip(&items).zip([32768, 98304, 131072]) {
        sink.bytes.clear();
        payload.decode(row, &mut sink, &control).unwrap();
        assert_eq!(&sink.bytes, expected);
        assert_eq!(sink.decoded, work);
    }
    assert_eq!((sink.precharged, sink.ordinary), (131072, 0));
    drop(payload);
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = CountingSink::default();
    payload.decode(&rows[1], &mut sink, &control).unwrap();
    assert_eq!(sink.bytes, items[1].1);
    assert_eq!(
        (sink.decoded, sink.precharged, sink.ordinary),
        (98304, 80002, 0)
    );
}
fn set_store_file_region(bytes: &mut [u8], record: usize, offset: u32, size: u32) {
    let mut at = u32::from_le_bytes(bytes[16..20].try_into().unwrap()) as usize;
    for _ in 0..record {
        at += 16;
        at += bytes[at..].iter().position(|byte| *byte == 0).unwrap() + 1;
    }
    bytes[at..at + 4].copy_from_slice(&size.to_le_bytes());
    bytes[at + 4..at + 8].copy_from_slice(&offset.to_le_bytes());
}
#[test]
fn duplicate_and_overlapping_records_exhaust_frame_credit_without_losing_output() {
    let items = [
        (b"same.bin".to_vec(), vec![b'A'; 16000], 0),
        (b"same.bin".to_vec(), vec![b'B'; 16000], 0),
        (b"overlap.bin".to_vec(), vec![b'C'; 768], 0),
    ];
    let mut bytes = store(&items, false);
    set_store_file_region(&mut bytes, 1, 0, 16000);
    set_store_file_region(&mut bytes, 2, 8000, 16000);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 3);
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    let expected = [
        vec![b'A'; 16000],
        vec![b'A'; 16000],
        [vec![b'A'; 8000], vec![b'B'; 8000]].concat(),
    ];
    let mut sink = CountingSink::default();
    for (row, expected) in rows.iter().zip(expected) {
        sink.bytes.clear();
        payload.decode(row, &mut sink, &control).unwrap();
        assert_eq!(sink.bytes, expected);
    }
    assert_eq!(
        (sink.decoded, sink.precharged, sink.ordinary),
        (48000, 32768, 15232)
    );
    assert_eq!(sink.credit, 0);
}
#[test]
fn unused_old_frame_credit_cannot_cover_repeated_output_from_a_new_frame() {
    let items = [
        (b"prefix.bin".to_vec(), vec![b'A'; 32768], 0),
        (b"same.bin".to_vec(), vec![b'B'; 32768], 0),
        (b"same.bin".to_vec(), vec![b'C'; 32768], 0),
    ];
    let mut bytes = store(&items, false);
    set_store_file_region(&mut bytes, 2, 32768, 32768);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = CountingSink::default();
    payload.decode(&rows[1], &mut sink, &control).unwrap();
    assert_eq!(sink.bytes, items[1].1);
    assert_eq!(
        (sink.decoded, sink.precharged, sink.ordinary),
        (65536, 32768, 0)
    );
    sink.bytes.clear();
    payload.decode(&rows[2], &mut sink, &control).unwrap();
    assert_eq!(sink.bytes, items[1].1);
    // Frame 0 was skipped and frame 1 was delivered twice. Its second delivery
    // is ordinary output; frame 0's unused allowance cannot be reused for it.
    // The last physical record also validates the otherwise-unselected frame 2.
    assert_eq!(
        (sink.decoded, sink.precharged, sink.ordinary),
        (131072, 32768, 32768)
    );
}
#[test]
fn exhausted_frame_credit_cannot_bypass_output_work_limit() {
    let bytes = store(&[(b"same.bin".to_vec(), vec![b'A'; 32768], 0)], false);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = CountingSink {
        limit: Some(32768),
        ..CountingSink::default()
    };
    payload.decode(&rows[0], &mut sink, &control).unwrap();
    assert_eq!(
        (sink.decoded, sink.precharged, sink.ordinary),
        (32768, 32768, 0)
    );
    sink.bytes.clear();
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Limit)
    );
    assert!(sink.bytes.is_empty());
    assert_eq!(sink.decoded, 32768);
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn a_sink_without_precharged_support_keeps_output_and_conservative_metering() {
    #[derive(Default)]
    struct ConservativeSink {
        bytes: Vec<u8>,
        decoded: u64,
    }
    impl DecodedSink for ConservativeSink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.charge_skipped(bytes.len() as u64)?;
            self.bytes.extend_from_slice(bytes);
            Ok(())
        }
        fn charge_skipped(&mut self, count: u64) -> Result<(), DecodeError> {
            self.decoded += count;
            Ok(())
        }
    }
    let bytes = store(&[(b"small.bin".to_vec(), b"payload".to_vec(), 0)], false);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = ConservativeSink::default();
    payload.decode(&rows[0], &mut sink, &control).unwrap();
    assert_eq!(sink.bytes, b"payload");
    assert_eq!(sink.decoded, 14);
}
#[test]
fn continuation_attributes_sizes_and_names_must_match_initial_record() {
    let control = Cancellation::default();
    for relative in [0, 14, 16] {
        let mut part = PARTS[1].to_vec();
        let offset = u32::from_le_bytes(part[16..20].try_into().unwrap()) as usize;
        part[offset + relative] ^= 0x80;
        let mut source = Parts::new(&[PARTS[0], &part, PARTS[2], PARTS[3], PARTS[4]]);
        assert!(
            CabCodec
                .scan(&mut source, &control, ScanLimits::default())
                .is_err()
        );
    }
}
#[test]
fn actual_mid_output_cancellation_and_sink_failure_stop_before_next_frame() {
    struct Cancel<'a>(&'a Cancellation, usize);
    impl DecodedSink for Cancel<'_> {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.1 += bytes.len();
            self.0.cancel();
            Ok(())
        }
    }
    struct Reject;
    impl DecodedSink for Reject {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Err(DecodeError::Output)
        }
    }
    let bytes = store(&[(b"large.bin".to_vec(), vec![b'X'; 65536], 0)], false);
    let mut source = Cursor::new(&bytes);
    let control = Cancellation::default();
    let rows = CabCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Cancel(&control, 0);
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Cancelled)
    );
    assert_eq!(sink.1, 32768);
    drop(payload);
    let control = Cancellation::default();
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Reject, &control),
        Err(DecodeError::Output)
    );
}

fn hex(value: &str) -> Vec<u8> {
    (0..value.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&value[at..at + 2], 16).unwrap())
        .collect()
}
#[test]
fn full_archive_legacy_names_and_shift_jis_trail_separator_decode_before_output() {
    let items = [
        (
            "d6d0cec4cec4bcfed3ebd1b9cbf5b2e2cad42e747874",
            "中文文件与压缩测试.txt",
        ),
        (
            "bfe7c6bdcca8bde2d1b9cab9d3c3cbb5c3f72e747874",
            "跨平台解压使用说明.txt",
        ),
        ("c2a92e747874", "漏.txt"),
    ];
    let bytes = store(
        &items
            .iter()
            .map(|(raw, _)| (hex(raw), b"legacy payload".to_vec(), 0))
            .collect::<Vec<_>>(),
        false,
    );
    let result = decode(&mut Cursor::new(bytes));
    for (_, name) in items {
        assert_eq!(result[name.as_bytes()], b"legacy payload");
    }
    let items = [
        (
            "835c8374836783458346834190e096be8f912e747874",
            "folder/ソフトウェア説明書.txt",
        ),
        (
            "82d082e782aa82c8834a835e834a83692e747874",
            "folder/ひらがなカタカナ.txt",
        ),
    ];
    let bytes = store(
        &items
            .iter()
            .map(|(raw, _)| {
                (
                    [b"folder\\".as_slice(), &hex(raw)].concat(),
                    b"sjis payload".to_vec(),
                    0,
                )
            })
            .collect::<Vec<_>>(),
        false,
    );
    let result = decode(&mut Cursor::new(bytes));
    for (_, name) in items {
        assert_eq!(result[name.as_bytes()], b"sjis payload");
    }
}
#[test]
fn malformed_explicit_utf8_is_visible_but_never_decoded_as_legacy() {
    let mut source = Cursor::new(store(
        &[(vec![0xff, b'.', b't', b'x', b't'], b"data".to_vec(), 0x80)],
        false,
    ));
    let control = Cancellation::default();
    let (rows, flags) = CabCodec
        .scan_with_encoding_flags(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!(flags, [true]);
    assert!(rows[0].warning);
    assert_eq!(
        rows[0].name_encoding,
        xdzip_codec_api::names::NameEncoding::Unresolved
    );
    let mut payload = CabCodec.open_payload(&mut source, &rows, &control).unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Sink::default(), &control),
        Err(DecodeError::Unsupported)
    );
}
#[test]
fn bounded_fixed_seed_bad_inputs_never_panic_in_scan_open_or_decode() {
    struct Limited<'a> {
        bytes: Cursor<&'a [u8]>,
        count: u64,
    }
    impl Read for Limited<'_> {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let room = (1024 * 1024u64).saturating_sub(self.count) as usize;
            if room == 0 && !output.is_empty() {
                return Err(io::Error::other(ScanError::Limit));
            }
            let take = room.min(output.len());
            let count = self.bytes.read(&mut output[..take])?;
            self.count += count as u64;
            Ok(count)
        }
    }
    impl Seek for Limited<'_> {
        fn seek(&mut self, offset: SeekFrom) -> io::Result<u64> {
            self.bytes.seek(offset)
        }
    }
    impl ReadSource for Limited<'_> {}
    struct LimitedSink(u64);
    impl DecodedSink for LimitedSink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.charge_skipped(bytes.len() as u64)
        }
        fn charge_skipped(&mut self, count: u64) -> Result<(), DecodeError> {
            self.0 = self
                .0
                .checked_add(count)
                .filter(|total| *total <= 1024 * 1024)
                .ok_or(DecodeError::Limit)?;
            Ok(())
        }
    }
    fn exercise(bytes: &[u8]) -> bool {
        let mut source = Limited {
            bytes: Cursor::new(bytes),
            count: 0,
        };
        let control = Cancellation::default();
        let limits = ScanLimits {
            entries: 256,
            read_bytes: 128 * 1024,
            ..ScanLimits::default()
        };
        let mut sink = LimitedSink(0);
        let success = if let Ok(rows) = CabCodec.scan(&mut source, &control, limits) {
            if let Ok(mut payload) = CabCodec.open_payload(&mut source, &rows, &control) {
                rows.iter()
                    .filter(|row| row.payload_supported())
                    .all(|row| payload.decode(row, &mut sink, &control).is_ok())
            } else {
                false
            }
        } else {
            false
        };
        assert!(source.count <= 1024 * 1024 && sink.0 <= 1024 * 1024);
        success
    }
    for (original, mut seed) in [(NAMED, 0x43414211u32), (METHODS, 0x43414229u32)] {
        assert!(exercise(original));
        let mut accepted = 0;
        let mut rejected = 0;
        for case in 0..192 {
            let mut bytes = original.to_vec();
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            if case < 128 {
                let at = seed as usize % bytes.len();
                bytes[at] ^= 1 << ((seed >> 16) & 7);
            } else {
                bytes.truncate(seed as usize % bytes.len());
            }
            let result = std::panic::catch_unwind(|| exercise(&bytes));
            assert!(result.is_ok(), "fixed mutation {case}");
            if result.unwrap() {
                accepted += 1;
            } else {
                rejected += 1;
            }
        }
        assert!(accepted > 0 && rejected > 0);
    }
}
