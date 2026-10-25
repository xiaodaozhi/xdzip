use super::*;
use std::io::{self, Cursor as IoCursor, Read, Seek, SeekFrom};
#[path = "decode_tests.rs"]
mod decode_tests;

fn number(mut n: u64) -> Vec<u8> {
    for extra in 0..8 {
        if n < (1u64 << (7 * (extra + 1))) {
            let mut bytes = vec![((0xffu16 << (8 - extra)) as u8) | (n >> (8 * extra)) as u8];
            for _ in 0..extra {
                bytes.push(n as u8);
                n >>= 8;
            }
            return bytes;
        }
    }
    [vec![255], n.to_le_bytes().to_vec()].concat()
}
fn property(id: u8, value: &[u8]) -> Vec<u8> {
    [vec![id], number(value.len() as u64), value.to_vec()].concat()
}
fn names(values: &[&str]) -> Vec<u8> {
    let mut bytes = vec![0];
    for name in values {
        for unit in name.encode_utf16().chain([0]) {
            bytes.extend(unit.to_le_bytes());
        }
    }
    property(17, &bytes)
}
fn copy_folder() -> Vec<u8> {
    vec![1, 1, 0]
}
fn digest(values: &[&[u8]]) -> Vec<u8> {
    let mut bytes = vec![10, 1];
    for value in values {
        bytes.extend(crc32(value).to_le_bytes());
    }
    bytes
}
fn streams_info(
    position: usize,
    outputs: &[&[u8]],
    counts: Option<&[usize]>,
    split_sizes: &[usize],
    external_folder: Option<usize>,
) -> Vec<u8> {
    let mut bytes = vec![6];
    bytes.extend(number(position as u64));
    bytes.extend(number(outputs.len() as u64));
    bytes.push(9);
    for output in outputs {
        bytes.extend(number(output.len() as u64));
    }
    bytes.extend(digest(outputs));
    bytes.extend([0, 7, 11]);
    bytes.extend(number(outputs.len() as u64));
    if let Some(index) = external_folder {
        bytes.push(1);
        bytes.extend(number(index as u64));
    } else {
        bytes.push(0);
        for _ in outputs {
            bytes.extend(copy_folder());
        }
    }
    bytes.push(12);
    for output in outputs {
        bytes.extend(number(output.len() as u64));
    }
    bytes.extend(digest(outputs));
    bytes.push(0);
    if let Some(counts) = counts {
        bytes.extend([8, 13]);
        for count in counts {
            bytes.extend(number(*count as u64));
        }
        if !split_sizes.is_empty() {
            bytes.push(9);
            for size in split_sizes {
                bytes.extend(number(*size as u64));
            }
        }
        bytes.push(0);
    }
    bytes.push(0);
    bytes
}
fn image(packed: &[u8], header: &[u8]) -> Vec<u8> {
    let mut tail = (packed.len() as u64).to_le_bytes().to_vec();
    tail.extend((header.len() as u64).to_le_bytes());
    tail.extend(crc32(header).to_le_bytes());
    let mut bytes = b"7z\xbc\xaf\x27\x1c\x00\x04".to_vec();
    bytes.extend(crc32(&tail).to_le_bytes());
    bytes.extend(tail);
    bytes.extend(packed);
    bytes.extend(header);
    bytes
}
fn plain() -> Vec<u8> {
    let data = b"abcdef";
    let mut header = vec![1, 4];
    header.extend(streams_info(0, &[data], Some(&[3]), &[3, 3], None));
    header.extend([5, 5]);
    header.extend(property(14, &[0x50]));
    header.extend(property(15, &[0x80]));
    header.extend(names(&["same", "empty", "same", "directory", "zero-🦀"]));
    header.extend([0, 0]);
    image(data, &header)
}
fn external_names() -> Vec<u8> {
    let data = b"abcdef";
    let mut name_blob = Vec::new();
    for name in ["first", "second"] {
        for unit in name.encode_utf16().chain([0]) {
            name_blob.extend(unit.to_le_bytes());
        }
    }
    let mut header = vec![1, 3];
    // Two substreams deliberately share one metadata folder. DataIndex still
    // addresses the entire folder output, rather than either substream.
    header.extend(streams_info(6, &[&name_blob], Some(&[2]), &[2], None));
    header.push(4);
    header.extend(streams_info(0, &[data], Some(&[2]), &[3], None));
    header.extend([5, 2]);
    header.extend(property(17, &[1, 0]));
    header.extend([0, 0]);
    image(&[data.as_slice(), &name_blob].concat(), &header)
}
fn external_table() -> Vec<u8> {
    let data = b"abcdef";
    let table = copy_folder();
    let mut header = vec![1, 3];
    header.extend(streams_info(6, &[&table], None, &[], None));
    header.push(4);
    header.extend(streams_info(0, &[data], Some(&[2]), &[3], Some(0)));
    header.extend([5, 2]);
    header.extend(names(&["first", "second"]));
    header.extend([0, 0]);
    image(&[data.as_slice(), &table].concat(), &header)
}
fn encoded_copy() -> Vec<u8> {
    let raw = external_names();
    let size = u64::from_le_bytes(raw[20..28].try_into().unwrap()) as usize;
    let start = raw.len() - size;
    let decoded = &raw[start..];
    let mut outer = vec![23];
    outer.extend(streams_info(start - 32, &[decoded], None, &[], None));
    image(&raw[32..], &outer)
}
fn external_flag(mut bytes: Vec<u8>, pattern: &[u8], field: usize) -> Vec<u8> {
    let at = 32 + u64::from_le_bytes(bytes[12..20].try_into().unwrap()) as usize;
    let mut header = bytes.split_off(at);
    let target = header
        .windows(pattern.len())
        .position(|v| v == pattern)
        .unwrap();
    header[target + field] = 2;
    image(&bytes[32..], &header)
}
fn external_values() -> Vec<u8> {
    let data = b"abcdef";
    let table = copy_folder();
    let timestamp = (1_704_164_645u64 + 11_644_473_600) * 10_000_000;
    let times = [timestamp.to_le_bytes(), 0u64.to_le_bytes()].concat();
    let attrs = [
        (0o120777u32 << 16 | 0x8000).to_le_bytes(),
        (0o010600u32 << 16 | 0x8000).to_le_bytes(),
    ]
    .concat();
    let mut header = vec![1, 3];
    header.extend(streams_info(6, &[&table, &times, &attrs], None, &[], None));
    header.push(4);
    header.extend(streams_info(0, &[data], Some(&[2]), &[3], Some(0)));
    header.extend([5, 2]);
    header.extend(names(&["link", "special"]));
    for id in [18, 19, 20] {
        header.extend(property(id, &[1, 1, 1]));
    }
    header.extend(property(21, &[1, 1, 2]));
    header.extend([0, 0]);
    image(&[data.as_slice(), &table, &times, &attrs].concat(), &header)
}
fn scan(bytes: &[u8]) -> Result<Index, ScanError> {
    read_index(
        &mut IoCursor::new(bytes),
        &Cancellation::default(),
        ScanLimits::default(),
        None,
    )
}

#[test]
fn integers_bitmap_and_noncanonical_encodings_follow_documented_geometry() {
    for n in [
        0,
        1,
        127,
        128,
        16383,
        16384,
        1 << 21,
        1 << 28,
        1 << 35,
        1 << 42,
        1 << 49,
        1 << 56,
        u64::MAX,
    ] {
        let bytes = number(n);
        let mut c = wire::Cursor::new(&bytes);
        assert_eq!(c.number(), Ok(n));
        assert!(c.end().is_ok());
        for end in 0..bytes.len() {
            assert!(wire::Cursor::new(&bytes[..end]).number().is_err());
        }
    }
    assert_eq!(wire::Cursor::new(&[128, 0]).number(), Ok(0));
    assert_eq!(
        wire::Cursor::new(&[0xff, 0, 0, 0, 0, 0, 0, 0, 0]).number(),
        Ok(0)
    );
    assert_eq!(
        wire::Cursor::new(&[0x87]).bits(5).unwrap(),
        [true, false, false, false, false]
    );
    assert_eq!(wire::Cursor::new(&[2]).digests(0).unwrap(), []);
}
#[test]
fn physical_duplicates_empty_records_zero_length_stream_and_unicode_stay_distinct() {
    let index = scan(&plain()).unwrap();
    assert_eq!(
        index.rows.iter().map(|r| r.id.get()).collect::<Vec<_>>(),
        [1, 2, 3, 4, 5]
    );
    assert_eq!(
        index.rows.iter().map(|r| r.size).collect::<Vec<_>>(),
        [3, 0, 3, 0, 0]
    );
    assert_eq!(index.rows[0].raw_name, index.rows[2].raw_name);
    assert_eq!(index.rows[3].name_bytes(), b"directory/");
    assert!(index.rows[3].directory);
    assert!(index.files[1].empty_file && index.files[1].data.is_none());
    assert!(index.files[4].data.is_some());
    assert_eq!(index.rows[4].name_bytes(), "zero-🦀".as_bytes());
    assert!(
        index
            .rows
            .iter()
            .all(|r| r.method == 0 && r.format == xdzip_codec_api::Format::SevenZip)
    );
}
#[test]
fn copy_external_properties_and_encoded_header_decode_complete_metadata_graphs() {
    for bytes in [external_names(), external_table(), encoded_copy()] {
        let index = scan(&bytes).unwrap();
        assert_eq!(
            index
                .rows
                .iter()
                .map(|r| r.name_bytes())
                .collect::<Vec<_>>(),
            [b"first".as_slice(), b"second"]
        );
        assert_eq!(index.streams.folders[0].substreams[1].offset, 3);
        assert_eq!(index.rows[0].packed_size, 6);
        assert_eq!(index.rows[1].packed_size, 0);
    }
}
#[test]
fn nonzero_external_selectors_share_the_same_exact_blob_semantics() {
    let names = external_flag(external_names(), &[17, 2, 1, 0], 2);
    let table = external_flag(external_table(), &[7, 11, 1, 1, 0], 3);
    for bytes in [names, table] {
        assert_eq!(scan(&bytes).unwrap().rows.len(), 2);
    }
}
#[test]
fn external_fields_restart_at_blob_zero_and_retain_defined_properties() {
    let index = scan(&external_values()).unwrap();
    assert_eq!(index.files[0].creation_time, index.files[0].access_time);
    assert_eq!(index.files[0].access_time, index.files[0].modified_time);
    assert_eq!(index.files[1].modified_time, Some(0));
    assert_ne!(index.rows[0].modified_dos, 0);
    assert_eq!(index.rows[1].modified_dos, 0);
    assert!(index.rows[0].link && !index.rows[0].payload_supported());
    assert!(index.rows[1].warning && !index.rows[1].payload_supported());
    assert_eq!(index.files[0].attributes, Some(0o120777u32 << 16 | 0x8000));
}
#[test]
fn all_structural_truncations_and_both_header_crcs_fail_closed() {
    let bytes = external_names();
    for n in 0..bytes.len() {
        assert!(scan(&bytes[..n]).is_err(), "truncation {n}");
    }
    for at in [8, 12, 20, 28, bytes.len() - 1] {
        let mut corrupt = bytes.clone();
        corrupt[at] ^= 1;
        assert_eq!(scan(&corrupt), Err(ScanError::Invalid));
    }
    let mut corrupt = bytes;
    corrupt[12..20].copy_from_slice(&u64::MAX.to_le_bytes());
    let crc = crc32(&corrupt[12..32]);
    corrupt[8..12].copy_from_slice(&crc.to_le_bytes());
    assert_eq!(scan(&corrupt), Err(ScanError::Invalid));
}
#[test]
fn metadata_pack_and_unpacked_checksums_are_not_ignored() {
    let mut bytes = external_names();
    bytes[38] ^= 1;
    assert_eq!(scan(&bytes), Err(ScanError::Invalid));
    let mut bytes = encoded_copy();
    bytes[38] ^= 1;
    assert_eq!(scan(&bytes), Err(ScanError::Invalid));
}
#[test]
fn graph_validation_rejects_cycles_duplicate_ports_and_uncovered_inputs() {
    let control = Cancellation::default();
    let valid = [2, 1, 0, 1, 0, 1, 0];
    let f = graph::folder(
        &mut wire::Cursor::new(&valid),
        &mut source::Budget::new(),
        &control,
    )
    .unwrap();
    assert_eq!(f.topological_order, [0, 1]);
    assert_eq!(f.terminal_output, 1);
    for invalid in [
        vec![2, 1, 0, 1, 0, 0, 0],
        vec![2, 1, 0, 1, 0, 2, 0],
        vec![3, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1],
        vec![3, 1, 0, 1, 0, 1, 0, 1, 0, 2, 0],
    ] {
        assert!(
            graph::folder(
                &mut wire::Cursor::new(&invalid),
                &mut source::Budget::new(),
                &control
            )
            .is_err()
        );
    }
    let multiple = [1, 0x14, 3, 3, 1, 0x1b, 4, 1, 3, 1, 0, 2];
    let f = graph::folder(
        &mut wire::Cursor::new(&multiple),
        &mut source::Budget::new(),
        &control,
    )
    .unwrap();
    assert_eq!(
        f.packed_inputs.iter().map(|p| p.input).collect::<Vec<_>>(),
        [3, 1, 0, 2]
    );
    let duplicate = [1, 0x14, 3, 3, 1, 0x1b, 4, 1, 3, 1, 0, 0];
    assert!(
        graph::folder(
            &mut wire::Cursor::new(&duplicate),
            &mut source::Budget::new(),
            &control
        )
        .is_err()
    );
}
#[test]
fn zero_substreams_and_single_crc_inheritance_use_no_extra_digest_slot() {
    for count in [0usize, 1] {
        let mut data = streams_info(0, &[b""], Some(&[count]), &[], None);
        // Explicit CRC section with zero unknown digests still has its flag.
        let at = data.len() - 2;
        data.splice(at..at, [10, 1]);
        let control = Cancellation::default();
        let mut input = IoCursor::new(vec![0; 32]);
        let reader = source::Reader::new(&mut input, &control, 64).unwrap();
        let parsed = streams::parse(
            &mut wire::Cursor::new(&data),
            &reader,
            &mut source::Budget::new(),
            &[],
        )
        .unwrap();
        assert_eq!(parsed.folders[0].substreams.len(), count);
        if count == 1 {
            assert_eq!(parsed.folders[0].substreams[0].crc, Some(0));
        }
    }
}
#[test]
fn names_are_strict_utf16_and_no_synthetic_name_is_created() {
    let make = |props: Vec<u8>| {
        let mut h = vec![1, 5, 1];
        h.extend(property(14, &[0x80]));
        h.extend(property(15, &[0x80]));
        h.extend(props);
        h.extend([0, 0]);
        image(&[], &h)
    };
    for value in [
        vec![],
        property(17, &[0, 0, 0xd8, 0, 0]),
        property(17, &[0, 1]),
        property(17, &[0, 0, 0, 0, 0]),
    ] {
        assert_eq!(scan(&make(value)), Err(ScanError::Invalid));
    }
    let mut duplicate = names(&["one"]);
    duplicate.extend(names(&["two"]));
    assert_eq!(scan(&make(duplicate)), Err(ScanError::Invalid));
    assert_eq!(
        scan(&make(property(99, &[1, 2, 3]))),
        Err(ScanError::Unsupported)
    );
}

#[test]
fn repeated_zero_dummy_padding_is_accepted_but_nonzero_padding_is_not() {
    for padding in [0u8, 1] {
        let mut h = vec![1, 5, 1];
        h.extend(property(14, &[0x80]));
        h.extend(property(15, &[0x80]));
        h.extend(property(25, &[0, 0]));
        h.extend(names(&["empty"]));
        h.extend(property(25, &[padding]));
        h.extend([0, 0]);
        let result = scan(&image(&[], &h));
        if padding == 0 {
            assert_eq!(result.unwrap().rows.len(), 1);
        } else {
            assert_eq!(result, Err(ScanError::Invalid));
        }
    }
}

#[test]
fn mixed_substream_crc_slots_skip_zero_counts_and_inherited_values() {
    let mut s = vec![6, 0, 4, 9, 1, 1, 2, 1, 0, 7, 11, 4, 0];
    for _ in 0..4 {
        s.extend(copy_folder());
    }
    s.extend([12, 1, 1, 2, 1, 10, 0, 0x60]);
    s.extend(crc32(b"A").to_le_bytes());
    s.extend(crc32(b"BC").to_le_bytes());
    s.extend([0, 8, 13, 0, 1, 2, 1, 9, 1, 10, 0, 0xa7]);
    s.extend(crc32(b"B").to_le_bytes());
    s.extend(crc32(b"D").to_le_bytes());
    s.extend([0, 0]);
    let control = Cancellation::default();
    let mut source = IoCursor::new(vec![0; 37]);
    let reader = source::Reader::new(&mut source, &control, 64).unwrap();
    let value = streams::parse(
        &mut wire::Cursor::new(&s),
        &reader,
        &mut source::Budget::new(),
        &[],
    )
    .unwrap();
    assert!(value.folders[0].substreams.is_empty());
    assert_eq!(value.folders[0].unpacked_size(), 1);
    assert_eq!(value.folders[1].substreams[0].crc, Some(crc32(b"A")));
    assert_eq!(
        value.folders[2]
            .substreams
            .iter()
            .map(|x| x.crc)
            .collect::<Vec<_>>(),
        [Some(crc32(b"B")), None]
    );
    assert_eq!(value.folders[2].crc, Some(crc32(b"BC")));
    assert_eq!(value.folders[3].substreams[0].crc, Some(crc32(b"D")));
}

#[test]
fn metadata_callback_reads_share_the_same_cumulative_scan_budget() {
    struct Repeated;
    impl HeaderDecoder for Repeated {
        fn decode_metadata(
            &mut self,
            _: HeaderPurpose,
            source: &mut dyn HeaderSource,
            _: &Streams,
            _: &mut dyn MetadataSink,
            _: &Cancellation,
            _: Option<&str>,
        ) -> Result<(), ScanError> {
            for _ in 0..1024 {
                source.read_exact_at(0, &mut [0u8; 32])?;
            }
            Err(ScanError::Invalid)
        }
    }
    let limits = ScanLimits {
        read_bytes: 1024,
        ..ScanLimits::default()
    };
    assert_eq!(
        read_index_with_decoder(
            &mut IoCursor::new(encoded_copy()),
            &Cancellation::default(),
            limits,
            None,
            &mut Repeated
        ),
        Err(ScanError::Limit)
    );
}
#[test]
fn every_scan_quota_and_pre_cancellation_is_observed() {
    let bytes = plain();
    let control = Cancellation::default();
    for field in 0..5 {
        let mut limits = ScanLimits::default();
        match field {
            0 => limits.entries = 0,
            1 => limits.name_bytes = 0,
            2 => limits.total_name_bytes = 0,
            3 => limits.display_name_bytes = 0,
            _ => limits.read_bytes = 31,
        }
        assert_eq!(
            read_index(&mut IoCursor::new(&bytes), &control, limits, None),
            Err(ScanError::Limit)
        );
    }
    control.cancel();
    assert_eq!(
        read_index(
            &mut IoCursor::new(&bytes),
            &control,
            ScanLimits::default(),
            None
        ),
        Err(ScanError::Cancelled)
    );
}
struct Injection {
    bytes: IoCursor<Vec<u8>>,
    failure: Option<(usize, io::ErrorKind, ScanError)>,
    read: usize,
}
impl Read for Injection {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        if let Some((at, kind, error)) = self.failure {
            if self.read == at {
                self.failure = None;
                return Err(io::Error::new(kind, error));
            }
            let take = out.len().min(at.saturating_sub(self.read));
            if take > 0 {
                let n = self.bytes.read(&mut out[..take])?;
                self.read += n;
                return Ok(n);
            }
        }
        let n = self.bytes.read(out)?;
        self.read += n;
        Ok(n)
    }
}
impl Seek for Injection {
    fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(p)
    }
}
impl ReadSource for Injection {}
#[test]
fn typed_source_errors_survive_interrupted_and_partial_reads() {
    for at in [0, 3] {
        for kind in [io::ErrorKind::Other, io::ErrorKind::Interrupted] {
            for error in [
                ScanError::Cancelled,
                ScanError::Limit,
                ScanError::SourceChanged,
                ScanError::Io,
            ] {
                let mut source = Injection {
                    bytes: IoCursor::new(plain()),
                    failure: Some((at, kind, error)),
                    read: 0,
                };
                assert_eq!(
                    read_index(
                        &mut source,
                        &Cancellation::default(),
                        ScanLimits::default(),
                        None
                    ),
                    Err(error)
                );
            }
        }
    }
}
struct Oversize;
impl HeaderDecoder for Oversize {
    fn decode_metadata(
        &mut self,
        _: HeaderPurpose,
        _: &mut dyn HeaderSource,
        s: &Streams,
        sink: &mut dyn MetadataSink,
        _: &Cancellation,
        _: Option<&str>,
    ) -> Result<(), ScanError> {
        sink.write_folder(0, &vec![0; s.folders[0].unpacked_size() as usize + 1])
    }
}
#[test]
fn injected_decoder_cannot_overproduce_or_hide_truncated_metadata() {
    assert_eq!(
        read_index_with_decoder(
            &mut IoCursor::new(encoded_copy()),
            &Cancellation::default(),
            ScanLimits::default(),
            None,
            &mut Oversize
        ),
        Err(ScanError::Invalid)
    );
    struct Short;
    impl HeaderDecoder for Short {
        fn decode_metadata(
            &mut self,
            _: HeaderPurpose,
            _: &mut dyn HeaderSource,
            _: &Streams,
            _: &mut dyn MetadataSink,
            _: &Cancellation,
            _: Option<&str>,
        ) -> Result<(), ScanError> {
            Ok(())
        }
    }
    assert_eq!(
        read_index_with_decoder(
            &mut IoCursor::new(encoded_copy()),
            &Cancellation::default(),
            ScanLimits::default(),
            None,
            &mut Short
        ),
        Err(ScanError::Invalid)
    );
}
#[test]
fn decoded_metadata_allocation_is_admitted_before_calling_a_decoder() {
    let mut h = vec![23, 6, 0, 1, 9, 0, 0, 7, 11, 1, 0, 1, 1, 0, 12];
    h.extend(number(u64::MAX));
    h.extend([0, 0]);
    assert_eq!(
        read_index_with_decoder(
            &mut IoCursor::new(image(&[], &h)),
            &Cancellation::default(),
            ScanLimits::default(),
            None,
            &mut Oversize
        ),
        Err(ScanError::Limit)
    );
}
#[test]
fn header_injection_preserves_cancellation_source_errors_and_folder_bounds() {
    struct Inject(u8);
    impl HeaderDecoder for Inject {
        fn decode_metadata(
            &mut self,
            _: HeaderPurpose,
            source: &mut dyn HeaderSource,
            _: &Streams,
            sink: &mut dyn MetadataSink,
            control: &Cancellation,
            _: Option<&str>,
        ) -> Result<(), ScanError> {
            match self.0 {
                0 => Err(ScanError::SourceChanged),
                1 => {
                    control.cancel();
                    sink.write_folder(0, b"x")
                }
                2 => sink.write_folder(usize::MAX, b"x"),
                _ => source.read_exact_at(source.length(), &mut [0u8; 1]),
            }
        }
    }
    for (case, error) in [
        (0, ScanError::SourceChanged),
        (1, ScanError::Cancelled),
        (2, ScanError::Invalid),
        (3, ScanError::Invalid),
    ] {
        assert_eq!(
            read_index_with_decoder(
                &mut IoCursor::new(encoded_copy()),
                &Cancellation::default(),
                ScanLimits::default(),
                None,
                &mut Inject(case)
            ),
            Err(error)
        );
    }
}
#[test]
fn ordinary_data_is_not_read_while_scanning_plain_metadata() {
    struct Observed {
        cursor: IoCursor<Vec<u8>>,
        ranges: Vec<(u64, usize)>,
    }
    impl Read for Observed {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            let at = self.cursor.position();
            let n = self.cursor.read(b)?;
            self.ranges.push((at, n));
            Ok(n)
        }
    }
    impl Seek for Observed {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.cursor.seek(p)
        }
    }
    impl ReadSource for Observed {}
    let mut source = Observed {
        cursor: IoCursor::new(plain()),
        ranges: Vec::new(),
    };
    read_index(
        &mut source,
        &Cancellation::default(),
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert!(
        source
            .ranges
            .iter()
            .all(|(at, n)| *at + *n as u64 <= 32 || *at >= 38)
    );
}
#[test]
fn source_and_pack_offsets_above_four_gib_are_not_truncated_or_allocated() {
    struct Sparse {
        position: u64,
        length: u64,
        segments: Vec<(u64, Vec<u8>)>,
        read: u64,
    }
    impl Read for Sparse {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            let count = out
                .len()
                .min(self.length.saturating_sub(self.position) as usize);
            for (i, value) in out[..count].iter_mut().enumerate() {
                let at = self.position + i as u64;
                *value = 0;
                for (start, data) in &self.segments {
                    if at >= *start
                        && let Some(byte) = data.get((at - *start) as usize)
                    {
                        *value = *byte;
                        break;
                    }
                }
            }
            self.position += count as u64;
            self.read += count as u64;
            Ok(count)
        }
    }
    impl Seek for Sparse {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            let next = match p {
                SeekFrom::Start(n) => i128::from(n),
                SeekFrom::End(n) => i128::from(self.length) + i128::from(n),
                SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
            };
            self.position = u64::try_from(next).map_err(|_| io::ErrorKind::InvalidInput)?;
            Ok(self.position)
        }
    }
    impl ReadSource for Sparse {}
    let far = 1usize << 32;
    let mut h = vec![1, 4];
    h.extend(streams_info(far, &[b"abc"], None, &[], None));
    h.extend([5, 1]);
    h.extend(names(&["far"]));
    h.extend([0, 0]);
    let mut signature = image(b"abc", &h)[..32].to_vec();
    signature[12..20].copy_from_slice(&(far as u64 + 3).to_le_bytes());
    let crc = crc32(&signature[12..32]);
    signature[8..12].copy_from_slice(&crc.to_le_bytes());
    let mut input = Sparse {
        position: 0,
        length: far as u64 + 35 + h.len() as u64,
        segments: vec![
            (0, signature),
            (far as u64 + 32, b"abc".to_vec()),
            (far as u64 + 35, h.clone()),
        ],
        read: 0,
    };
    let index = read_index(
        &mut input,
        &Cancellation::default(),
        ScanLimits::default(),
        None,
    )
    .unwrap();
    assert_eq!(index.streams.packs[0].range.offset, far as u64 + 32);
    assert_eq!(index.rows[0].size, 3);
    assert_eq!(input.read, 32 + h.len() as u64);
}
#[test]
#[ignore = "Explicit fixed official reader, self-owned Copy metadata images only"]
fn official_reference_accepts_copy_metadata_images() {
    let reference = std::env::var_os("XDZIP_7Z_REFERENCE").expect("explicit reference executable");
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build/container-official-vectors");
    std::fs::create_dir_all(&output).unwrap();
    for (name, bytes) in [
        ("external-names", external_names()),
        ("external-folder", external_table()),
        ("encoded-header", encoded_copy()),
        (
            "external-names-flag2",
            external_flag(external_names(), &[17, 2, 1, 0], 2),
        ),
        (
            "external-folder-flag2",
            external_flag(external_table(), &[7, 11, 1, 1, 0], 3),
        ),
    ] {
        let path = output.join(format!("{name}.7z"));
        std::fs::write(&path, &bytes).unwrap();
        let status = std::process::Command::new(&reference)
            .args(["t", "-bd", "-y"])
            .arg(&path)
            .output()
            .unwrap();
        std::fs::write(
            output.join(format!("{name}.log")),
            [status.stdout.as_slice(), status.stderr.as_slice()].concat(),
        )
        .unwrap();
        assert!(status.status.success(), "official reader rejected {name}");
        assert_eq!(scan(&bytes).unwrap().rows.len(), 2);
        let decoded = std::process::Command::new(&reference)
            .args(["e", "-so", "-y"])
            .arg(&path)
            .arg("first")
            .output()
            .unwrap();
        assert!(decoded.status.success());
        assert_eq!(decoded.stdout, b"abc");
    }
    for (name, bytes) in [("empty", image(&[], &[])), ("physical-records", plain())] {
        let path = output.join(format!("{name}.7z"));
        std::fs::write(&path, &bytes).unwrap();
        let result = std::process::Command::new(&reference)
            .args(["t", "-bd", "-y"])
            .arg(&path)
            .output()
            .unwrap();
        std::fs::write(
            output.join(format!("{name}.log")),
            [result.stdout.as_slice(), result.stderr.as_slice()].concat(),
        )
        .unwrap();
        assert!(result.status.success(), "official reader rejected {name}");
    }
}
#[test]
#[ignore = "Explicit copied independent official-writer baseline, no algorithm decoding"]
fn independent_official_writer_plain_headers_preserve_all_known_file_names_and_sizes() {
    let folder = std::path::PathBuf::from(
        std::env::var_os("XDZIP_7Z_BASELINE").expect("fixed baseline directory"),
    );
    let mut cases = 0;
    for path in std::fs::read_dir(&folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
    {
        if path.extension().is_none_or(|ext| ext != "7z") {
            continue;
        }
        assert!(std::fs::metadata(&path).unwrap().len() <= 8 * 1024 * 1024);
        let data = std::fs::read(&path).unwrap();
        let at = 32 + u64::from_le_bytes(data[12..20].try_into().unwrap()) as usize;
        if data[at] != 1 {
            continue;
        }
        let index = scan(&data).unwrap_or_else(|error| {
            panic!("{}: {error}", path.file_name().unwrap().to_string_lossy())
        });
        assert_eq!(index.rows.len(), 3);
        let names = index
            .rows
            .iter()
            .map(|r| (r.name_bytes().to_vec(), r.size))
            .collect::<std::collections::BTreeMap<_, _>>();
        assert_eq!(
            names,
            std::collections::BTreeMap::from([
                (b"empty.txt".to_vec(), 0),
                (b"hello.txt".to_vec(), 27),
                (b"payload.bin".to_vec(), 9987)
            ])
        );
        cases += 1;
    }
    assert_eq!(cases, 21);
}
