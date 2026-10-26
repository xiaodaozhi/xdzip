use super::*;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use xdzip_codec_api::payload::{DecodeError, DecodedSink};
use xdzip_compression::crc32;
#[path = "legacy20_owned_tests.rs"]
mod legacy20_owned;
#[path = "legacy26_owned_tests.rs"]
mod legacy26_owned;
#[path = "legacy_container_tests.rs"]
mod legacy_container;

fn vint(mut n: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let b = (n & 127) as u8;
        n >>= 7;
        out.push(b | if n != 0 { 128 } else { 0 });
        if n == 0 {
            return out;
        }
    }
}
fn modern(kind: u64, flags: u64, fields: &[u8], extra: &[u8], data: &[u8]) -> Vec<u8> {
    let mut body = vint(kind);
    body.extend(vint(flags));
    if flags & 1 != 0 {
        body.extend(vint(extra.len() as u64));
    }
    if flags & 2 != 0 {
        body.extend(vint(data.len() as u64));
    }
    body.extend(fields);
    body.extend(extra);
    let mut checked = vint(body.len() as u64);
    checked.extend(body);
    let mut out = crc32(&checked).to_le_bytes().to_vec();
    out.extend(checked);
    out.extend(data);
    out
}
fn new_main(number: Option<u64>) -> Vec<u8> {
    let mut fields = vint(if number.is_some() { 3 } else { 0 });
    if let Some(n) = number {
        fields.extend(vint(n));
    }
    modern(1, 0, &fields, &[], &[])
}
fn new_end(next: bool) -> Vec<u8> {
    modern(5, 0, &vint(u64::from(next)), &[], &[])
}
#[allow(clippy::too_many_arguments)] // Independently specified physical wire fields.
pub(crate) fn new_file(
    name: &[u8],
    data: &[u8],
    size: u64,
    flags: u64,
    info: u64,
    unknown: bool,
    crc: u32,
    extra: &[u8],
) -> Vec<u8> {
    let mut fields = vint(4 | if unknown { 8 } else { 0 });
    fields.extend(vint(size));
    fields.extend(vint(0x20));
    fields.extend(crc.to_le_bytes());
    fields.extend(vint(info));
    fields.push(0);
    fields.extend(vint(name.len() as u64));
    fields.extend(name);
    modern(
        2,
        flags | 2 | u64::from(!extra.is_empty()),
        &fields,
        extra,
        data,
    )
}
pub(crate) fn new_archive(records: &[Vec<u8>]) -> Vec<u8> {
    let mut out = b"Rar!\x1a\x07\x01\x00".to_vec();
    out.extend(new_main(None));
    for record in records {
        out.extend(record);
    }
    out.extend(new_end(false));
    out
}
fn old_header(kind: u8, flags: u16, fields: &[u8]) -> Vec<u8> {
    let mut bytes = vec![kind];
    bytes.extend(flags.to_le_bytes());
    bytes.extend(((7 + fields.len()) as u16).to_le_bytes());
    bytes.extend(fields);
    let mut out = (crc32(&bytes) as u16).to_le_bytes().to_vec();
    out.extend(bytes);
    out
}
fn old_file(name: &[u8], data: &[u8], flags: u16, version: u8) -> Vec<u8> {
    let mut fields = (data.len() as u32).to_le_bytes().to_vec();
    fields.extend((data.len() as u32).to_le_bytes());
    fields.push(2);
    fields.extend(crc32(data).to_le_bytes());
    fields.extend(0x58210000u32.to_le_bytes());
    fields.push(version);
    fields.push(0x30);
    fields.extend((name.len() as u16).to_le_bytes());
    fields.extend(0x20u32.to_le_bytes());
    fields.extend(name);
    let mut out = old_header(0x74, flags | 0x8000, &fields);
    out.extend(data);
    out
}
fn old_archive(records: &[Vec<u8>]) -> Vec<u8> {
    let mut out = b"Rar!\x1a\x07\x00".to_vec();
    out.extend(old_header(0x73, 0, &[0; 6]));
    for record in records {
        out.extend(record);
    }
    out.extend(old_header(0x7b, 0, &[]));
    out
}
fn scan(bytes: Vec<u8>) -> Result<Vec<Entry>, ScanError> {
    RarCodec.scan(
        &mut Cursor::new(bytes),
        &Cancellation::default(),
        ScanLimits::default(),
    )
}
#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend(bytes);
        Ok(())
    }
}
fn unpack(bytes: Vec<u8>) -> Vec<Vec<u8>> {
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = RarCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    rows.iter()
        .map(|row| {
            let mut sink = Sink::default();
            payload.decode(row, &mut sink, &cancel).unwrap();
            sink.0
        })
        .collect()
}

#[test]
fn store_legacy_and_modern_duplicate_identity_and_zero_files() {
    for bytes in [
        old_archive(&[
            old_file(b"same", b"abc", 0, 29),
            old_file(b"same", b"defg", 0, 29),
            old_file(b"empty", b"", 0, 29),
        ]),
        new_archive(&[
            new_file(b"same", b"abc", 3, 0, 0, false, crc32(b"abc"), &[]),
            new_file(b"same", b"defg", 4, 0, 1, false, crc32(b"defg"), &[]),
            new_file(b"empty", b"", 0, 0, 0, false, 0, &[]),
        ]),
    ] {
        let rows = scan(bytes.clone()).unwrap();
        assert_eq!(
            rows.iter().map(|r| r.id.get()).collect::<Vec<_>>(),
            [1, 2, 3]
        );
        assert_eq!(rows[0].name_bytes(), rows[1].name_bytes());
        assert_eq!(unpack(bytes), [b"abc".to_vec(), b"defg".to_vec(), vec![]]);
    }
}
#[test]
fn public_unpack_version_is_not_compression_level() {
    let rows = scan(new_archive(&[
        new_file(b"a", b"a", 1, 0, 1, false, crc32(b"a"), &[]),
        new_file(b"b", b"b", 1, 0, 1 | 0x100000, false, crc32(b"b"), &[]),
    ]))
    .unwrap();
    assert_eq!(rows.iter().map(|r| r.method).collect::<Vec<_>>(), [70, 50]);
}

#[test]
fn modern_nanosecond_metadata_keeps_raw_high_bits_and_ignores_out_of_range_fraction() {
    let control = Cancellation::default();
    for fraction in [0, 999_999_999, 1_000_000_000, 0xc000_0001, u32::MAX] {
        let mut body = vint(3);
        body.extend(vint(19));
        body.extend(1_704_067_201u32.to_le_bytes());
        body.extend(fraction.to_le_bytes());
        let mut extra = vint(body.len() as u64);
        extra.extend(body);
        let archive = new_archive(&[new_file(b"entry", b"", 0, 0, 0, false, 0, &extra)]);
        let index = read_index(
            &mut Cursor::new(archive),
            &control,
            ScanLimits::default(),
            None,
        )
        .unwrap();
        assert_eq!(
            index.0.parts[0].description.times[0],
            Some(index::RawTime::Unix(1_704_067_201, Some(fraction)))
        );
        assert_eq!(index.entries()[0].modified_dos, 0x5821_0000);
        let duplicate = [extra.as_slice(), extra.as_slice()].concat();
        assert_eq!(
            scan(new_archive(&[new_file(
                b"entry", b"", 0, 0, 0, false, 0, &duplicate
            )])),
            Err(ScanError::Invalid)
        );
    }
}
#[test]
fn packed_crc_and_complete_crc_are_distinct() {
    let bytes = new_archive(&[new_file(
        b"a",
        b"abc",
        3,
        0,
        0,
        false,
        crc32(b"abc") ^ 1,
        &[],
    )]);
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = RarCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn corruption_truncation_and_payload_range_fail_closed() {
    let baseline = new_archive(&[new_file(b"a", b"abc", 3, 0, 0, false, crc32(b"abc"), &[])]);
    for n in 0..baseline.len() {
        assert!(scan(baseline[..n].to_vec()).is_err(), "truncation {n}");
    }
    let file_at = 8 + new_main(None).len();
    let file_header_len = new_file(b"a", b"abc", 3, 0, 0, false, crc32(b"abc"), &[]).len() - 3;
    for at in file_at..file_at + file_header_len {
        let mut bad = baseline.clone();
        bad[at] ^= 1;
        assert!(scan(bad).is_err(), "header byte {at}");
    }
    let mut invalid = new_file(b"a", b"a", 1, 0, 0, false, crc32(b"a"), &[]);
    invalid.truncate(invalid.len() - 1);
    assert!(scan(new_archive(&[invalid])).is_err());
}

#[test]
fn valid_sfx_and_absent_modern_end_have_explicit_policy() {
    let bytes = new_archive(&[new_file(b"a", b"abc", 3, 0, 0, false, crc32(b"abc"), &[])]);
    let mut prefixed = vec![0x31; 32765];
    prefixed.extend(&bytes);
    assert_eq!(unpack(prefixed), [b"abc".to_vec()]);
    assert!(scan(bytes[..bytes.len() - new_end(false).len()].to_vec()).is_err());
    let bytes = old_archive(&[old_file(b"a", b"abc", 0, 29)]);
    assert_eq!(
        unpack(bytes[..bytes.len() - old_header(0x7b, 0, &[]).len()].to_vec()),
        [b"abc".to_vec()]
    );
}

#[test]
fn bounded_deterministic_mutations_keep_index_invariants() {
    let baseline = new_archive(&[
        new_file(b"one", b"abc", 3, 0, 0, false, crc32(b"abc"), &[]),
        new_file(b"two", b"def", 3, 0, 1, false, crc32(b"def"), &[]),
    ]);
    let mut seed = 0x174a_39bdu32;
    for _ in 0..384 {
        seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
        let mut mutated = baseline.clone();
        let at = seed as usize % mutated.len();
        mutated[at] ^= (seed >> 24) as u8 | 1;
        if let Ok(rows) = scan(mutated) {
            for (n, row) in rows.iter().enumerate() {
                assert_eq!(row.id.get(), n as u64 + 1);
                assert!(row.raw_name.len() <= 4096);
                assert!(row.size <= FILE_LIMIT);
            }
        }
    }
}

#[test]
fn header_extra_duplicates_unknown_hash_and_kdf_limit_are_rejected() {
    let extra = [3, 4, 0, 1, 3, 4, 0, 2];
    assert_eq!(
        scan(new_archive(&[new_file(
            b"a", b"", 0, 0, 0, false, 0, &extra
        )])),
        Err(ScanError::Invalid)
    );
    assert_eq!(
        scan(new_archive(&[new_file(
            b"a",
            b"",
            0,
            0,
            0,
            false,
            0,
            &[2, 2, 1]
        )])),
        Err(ScanError::Unsupported)
    );
    let mut crypt = vec![1, 0, 0, 25];
    crypt.extend([0; 32]);
    let mut extra = vint(crypt.len() as u64);
    extra.extend(crypt);
    assert_eq!(
        scan(new_archive(&[new_file(
            b"a", b"", 0, 0, 0, false, 0, &extra
        )])),
        Err(ScanError::Limit)
    );
}

#[test]
fn services_do_not_own_ids_or_interrupt_a_file_continuation() {
    let mut parts = split_parts();
    let last = parts[0].len() - new_end(true).len();
    parts[0].truncate(last);
    let mut fields = vec![4, 0, 0];
    fields.extend(0u32.to_le_bytes());
    fields.extend([0, 0, 3]);
    fields.extend(b"CMT");
    parts[0].extend(modern(3, 2, &fields, &[], &[]));
    parts[0].extend(new_end(true));
    let mut source = Parts::new(parts);
    let control = Cancellation::default();
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 1);
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    payload.decode(&rows[0], &mut sink, &control).unwrap();
    assert_eq!(sink.0, b"abcdef");
}

#[test]
fn unicode_and_unix_names_keep_unsafe_types_and_spelling() {
    let rows = scan(new_archive(&[
        new_file(
            "private\u{fffe}\u{e080}".as_bytes(),
            b"",
            0,
            0,
            0,
            false,
            0,
            &[],
        ),
        new_file(b"windows\\bad", b"", 0, 0, 0, false, 0, &[]),
    ]))
    .unwrap();
    assert!(rows[0].warning && rows[0].link);
    assert_eq!(rows[1].name_bytes(), b"windows\\bad");
    assert!(rows[1].warning);
    let rows = scan(old_archive(&[old_file(b"dir\\file", b"", 0, 29)])).unwrap();
    assert_eq!(rows[0].raw_name, b"dir\\file");
    assert_eq!(rows[0].name_bytes(), b"dir/file");
    assert!(!rows[0].warning);
}

#[test]
fn size_and_snapshot_disagreements_fail_before_success() {
    let bytes = new_archive(&[new_file(b"a", b"abc", 4, 0, 0, false, crc32(b"abc"), &[])]);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let mut rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    {
        let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
        assert_eq!(
            payload.decode(&rows[0], &mut Sink::default(), &control),
            Err(DecodeError::Integrity)
        );
    }
    rows[0].raw_name = b"other".to_vec();
    assert!(RarCodec.open_payload(&mut source, &rows, &control).is_err());
}
pub(crate) struct Parts {
    bytes: Cursor<Vec<u8>>,
    lengths: Vec<u64>,
}
impl Parts {
    pub(crate) fn new(parts: Vec<Vec<u8>>) -> Self {
        Self {
            lengths: parts.iter().map(|p| p.len() as u64).collect(),
            bytes: Cursor::new(parts.concat()),
        }
    }
}
impl Read for Parts {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        self.bytes.read(b)
    }
}
impl Seek for Parts {
    fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(p)
    }
}
impl ReadSource for Parts {
    fn volume_lengths(&self) -> Option<&[u64]> {
        Some(&self.lengths)
    }
}
fn split_parts() -> Vec<Vec<u8>> {
    [b"ab".as_slice(), b"cd", b"ef"]
        .into_iter()
        .enumerate()
        .map(|(i, data)| {
            let mut part = b"Rar!\x1a\x07\x01\x00".to_vec();
            part.extend(new_main(Some(i as u64)));
            part.extend(new_file(
                b"split",
                data,
                6,
                if i == 0 {
                    16
                } else if i == 2 {
                    8
                } else {
                    24
                },
                0,
                i != 2,
                if i == 2 {
                    crc32(b"abcdef")
                } else {
                    crc32(data)
                },
                &[],
            ));
            part.extend(new_end(i != 2));
            part
        })
        .collect()
}
#[test]
fn three_physical_parts_keep_first_packed_size_and_recover_known_size() {
    let mut source = Parts::new(split_parts());
    let cancel = Cancellation::default();
    let rows = RarCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].size, rows[0].packed_size), (6, 2));
    let mut payload = RarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    payload.decode(&rows[0], &mut sink, &cancel).unwrap();
    assert_eq!(sink.0, b"abcdef");
}
#[test]
fn missing_wrong_volume_and_orphan_continuation_reject() {
    let mut parts = split_parts();
    parts.pop();
    assert_eq!(
        RarCodec.scan(
            &mut Parts::new(parts),
            &Cancellation::default(),
            ScanLimits::default()
        ),
        Err(ScanError::MissingVolume)
    );
    let mut parts = split_parts();
    parts.swap(1, 2);
    assert!(
        RarCodec
            .scan(
                &mut Parts::new(parts),
                &Cancellation::default(),
                ScanLimits::default()
            )
            .is_err()
    );
    assert_eq!(
        scan(new_archive(&[new_file(
            b"split",
            b"a",
            1,
            8,
            0,
            false,
            crc32(b"a"),
            &[]
        )])),
        Err(ScanError::Invalid)
    );
}
#[test]
fn split_part_crc_failure_never_becomes_complete_success() {
    let mut parts = split_parts();
    let n = parts[0].len() - new_end(true).len() - 1;
    parts[0][n] ^= 1;
    let mut source = Parts::new(parts);
    let cancel = Cancellation::default();
    let rows = RarCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn known_unknown_policy_and_conflicting_sizes() {
    let unknown = new_archive(&[new_file(b"a", b"abc", 123, 0, 0, true, crc32(b"abc"), &[])]);
    let mut source = Cursor::new(unknown);
    let control = Cancellation::default();
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!((rows[0].size, rows[0].size_known), (0, false));
    assert!(!rows[0].payload_supported());
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Unsupported)
    );
    assert!(sink.0.is_empty());
    drop(payload);
    let mut parts = split_parts();
    parts[1] = {
        let mut out = b"Rar!\x1a\x07\x01\x00".to_vec();
        out.extend(new_main(Some(1)));
        out.extend(new_file(
            b"split",
            b"cd",
            7,
            24,
            0,
            false,
            crc32(b"cd"),
            &[],
        ));
        out.extend(new_end(true));
        out
    };
    assert_eq!(
        RarCodec.scan(
            &mut Parts::new(parts),
            &Cancellation::default(),
            ScanLimits::default()
        ),
        Err(ScanError::Invalid)
    );
}
#[test]
fn vint_accepts_wide_zero_but_rejects_overflow_and_truncation() {
    use crate::wire::Wire;
    for bytes in [&[0x81, 0][..], &[0x81, 0x80, 0][..]] {
        assert_eq!(Wire::new(bytes).vint().unwrap(), 1);
    }
    assert_eq!(Wire::new(&[0xff; 10]).vint(), Err(ScanError::Invalid));
    assert_eq!(Wire::new(&[0x80]).vint(), Err(ScanError::Invalid));
}
#[test]
fn unicode_commands_surrogates_and_ambiguous_legacy_run() {
    assert_eq!(
        names::compressed(b"abc\0\0\xc0\x01").unwrap().unwrap(),
        "abc"
    );
    assert_eq!(
        names::compressed(b"x\0\0\xa0\x3d\xd8\x00\xde")
            .unwrap()
            .unwrap(),
        "😀"
    );
    assert_eq!(
        names::compressed(b"x\0\0\x80\x00\xd8"),
        Err(ScanError::Invalid)
    );
    if cfg!(windows) {
        assert_eq!(
            names::compressed(b"\x80\x81\0\0\xc0\0"),
            Ok(Some("\u{ff80}\u{ff81}".into()))
        );
        assert_eq!(
            names::compressed(b"\xff\xfe\0\0\xc0\0"),
            Ok(Some("\u{ffff}\u{fffe}".into()))
        );
    } else {
        assert_eq!(
            names::compressed(b"\x80x\0\0\xc0\0"),
            Err(ScanError::Unsupported)
        );
    }
    assert_eq!(
        names::compressed(b"\x80\x81\0\0\xc0\x80\0"),
        Ok(Some("\u{80}\u{81}".into()))
    );
    let raw = b"abc\0\0\xc0\x01";
    let rows = scan(old_archive(&[old_file(raw, b"a", 0x200, 29)])).unwrap();
    assert_eq!(rows[0].raw_name, raw);
    assert_eq!(rows[0].name_bytes(), b"abc");
}
#[test]
fn unknown_extras_are_bounded_and_redirection_stays_special() {
    let mut extra = vint(3);
    extra.extend(vint(99));
    extra.extend([1, 2]);
    let bytes = new_archive(&[new_file(b"a", b"a", 1, 0, 0, false, crc32(b"a"), &extra)]);
    assert_eq!(unpack(bytes), [b"a".to_vec()]);
    let mut redir = vint(4);
    redir.extend([5, 5, 0, 0]);
    let rows = scan(new_archive(&[new_file(
        b"copy", b"", 0, 0, 0, false, 0, &redir,
    )]))
    .unwrap();
    assert!(rows[0].link && rows[0].warning);
    assert!(
        scan(new_archive(&[new_file(
            b"a",
            b"",
            0,
            0,
            0,
            false,
            0,
            &[9, 99]
        )]))
        .is_err()
    );
}
#[test]
fn input_name_record_and_dictionary_budgets() {
    let bytes = new_archive(&[new_file(
        b"name",
        b"abc",
        3,
        0,
        0,
        false,
        crc32(b"abc"),
        &[],
    )]);
    for limits in [
        ScanLimits {
            read_bytes: 8,
            ..ScanLimits::default()
        },
        ScanLimits {
            name_bytes: 3,
            ..ScanLimits::default()
        },
        ScanLimits {
            total_name_bytes: 3,
            ..ScanLimits::default()
        },
        ScanLimits {
            entries: 1,
            ..ScanLimits::default()
        },
    ] {
        assert_eq!(
            RarCodec.scan(
                &mut Cursor::new(bytes.clone()),
                &Cancellation::default(),
                limits
            ),
            Err(ScanError::Limit)
        );
    }
    assert_eq!(
        scan(new_archive(&[new_file(
            b"a",
            b"a",
            1,
            0,
            1 | (16 << 10) | (5 << 7),
            false,
            0,
            &[]
        )])),
        Err(ScanError::Limit)
    );
}
struct Fault {
    bytes: Cursor<Vec<u8>>,
    error: ScanError,
    kind: io::ErrorKind,
    after: usize,
    reads: usize,
}
impl Read for Fault {
    fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
        if self.reads >= self.after {
            return Err(io::Error::new(self.kind, self.error));
        }
        self.reads += 1;
        let n = b.len().min(3);
        self.bytes.read(&mut b[..n])
    }
}
impl Seek for Fault {
    fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(p)
    }
}
impl ReadSource for Fault {}
#[test]
fn typed_source_errors_survive_interrupted_and_partial_reads() {
    for error in [
        ScanError::Cancelled,
        ScanError::Limit,
        ScanError::SourceChanged,
        ScanError::Io,
    ] {
        for kind in [io::ErrorKind::Other, io::ErrorKind::Interrupted] {
            for after in [0, 1] {
                let mut source = Fault {
                    bytes: Cursor::new(new_archive(&[])),
                    error,
                    kind,
                    after,
                    reads: 0,
                };
                assert_eq!(
                    RarCodec.scan(&mut source, &Cancellation::default(), ScanLimits::default()),
                    Err(error)
                );
            }
        }
    }
}

#[test]
fn bare_interrupted_can_retry_without_hiding_typed_payload_failures() {
    struct Once {
        bytes: Cursor<Vec<u8>>,
        first: bool,
    }
    impl Read for Once {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            if self.first {
                self.first = false;
                Err(io::Error::from(io::ErrorKind::Interrupted))
            } else {
                self.bytes.read(b)
            }
        }
    }
    impl Seek for Once {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.bytes.seek(p)
        }
    }
    impl ReadSource for Once {}
    let mut source = Once {
        bytes: Cursor::new(new_archive(&[])),
        first: true,
    };
    assert!(
        RarCodec
            .scan(&mut source, &Cancellation::default(), ScanLimits::default())
            .unwrap()
            .is_empty()
    );
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct Switch {
        bytes: Cursor<Vec<u8>>,
        enabled: Arc<AtomicBool>,
        at: u64,
        kind: io::ErrorKind,
        error: ScanError,
    }
    impl Read for Switch {
        fn read(&mut self, b: &mut [u8]) -> io::Result<usize> {
            if self.enabled.load(Ordering::Relaxed) && self.bytes.position() >= self.at {
                return Err(io::Error::new(self.kind, self.error));
            }
            let n = b.len().min(3);
            self.bytes.read(&mut b[..n])
        }
    }
    impl Seek for Switch {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.bytes.seek(p)
        }
    }
    impl ReadSource for Switch {}
    let content = vec![0x41; 10000];
    let record = new_file(
        b"a",
        &content,
        content.len() as u64,
        0,
        0,
        false,
        crc32(&content),
        &[],
    );
    let start = (8 + new_main(None).len() + record.len() - content.len()) as u64;
    for error in [
        ScanError::Cancelled,
        ScanError::Limit,
        ScanError::SourceChanged,
        ScanError::Io,
    ] {
        for kind in [io::ErrorKind::Other, io::ErrorKind::Interrupted] {
            for delta in [0, 8195] {
                let enabled = Arc::new(AtomicBool::new(false));
                let mut source = Switch {
                    bytes: Cursor::new(new_archive(std::slice::from_ref(&record))),
                    enabled: enabled.clone(),
                    at: start + delta,
                    kind,
                    error,
                };
                let control = Cancellation::default();
                let rows = RarCodec
                    .scan(&mut source, &control, ScanLimits::default())
                    .unwrap();
                let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
                enabled.store(true, Ordering::Relaxed);
                let expected = match error {
                    ScanError::Cancelled => DecodeError::Cancelled,
                    ScanError::Limit => DecodeError::Limit,
                    _ => DecodeError::SourceIo,
                };
                let mut sink = Sink::default();
                assert_eq!(payload.decode(&rows[0], &mut sink, &control), Err(expected));
                assert_eq!(sink.0.len(), if delta == 0 { 0 } else { 8192 });
                assert_eq!(
                    payload.decode(&rows[0], &mut sink, &control),
                    Err(DecodeError::Integrity)
                );
            }
        }
    }
}
#[test]
fn cancellation_and_failed_sink_poison_instance() {
    let bytes = new_archive(&[new_file(b"a", b"abc", 3, 0, 0, false, crc32(b"abc"), &[])]);
    let control = Cancellation::default();
    control.cancel();
    assert_eq!(
        RarCodec.scan(
            &mut Cursor::new(bytes.clone()),
            &control,
            ScanLimits::default()
        ),
        Err(ScanError::Cancelled)
    );
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = RarCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    struct Broken;
    impl DecodedSink for Broken {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Err(DecodeError::Output)
        }
    }
    assert_eq!(
        payload.decode(&rows[0], &mut Broken, &cancel),
        Err(DecodeError::Output)
    );
    assert_eq!(
        payload.decode(&rows[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn header_encryption_requests_secret_without_partial_rows() {
    let mut fields = vec![0, 0, 1];
    fields.extend([0; 16]);
    let mut bytes = b"Rar!\x1a\x07\x01\x00".to_vec();
    bytes.extend(modern(4, 0, &fields, &[], &[]));
    bytes.extend([0; 32]);
    assert_eq!(scan(bytes.clone()), Err(ScanError::PasswordRequired));
    assert_eq!(
        RarCodec.scan_with_password(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
            Some("x")
        ),
        Err(ScanError::PasswordOrIntegrity)
    );
}
#[test]
fn modern_and_legacy_compression_password_flags_are_metadata_capabilities() {
    let rows = scan(new_archive(&[new_file(
        b"a",
        b"x",
        1,
        0,
        5 << 7,
        false,
        0,
        &[],
    )]))
    .unwrap();
    assert!(rows[0].payload_supported());
    let rows = scan(old_archive(&[old_file(b"secret", b"x", 4, 29)])).unwrap();
    assert!(rows[0].payload_supported());
}

#[test]
fn fixed_project_rar7_store_has_real_known_bytes() {
    let bytes = include_bytes!("../../../tests/fixtures/native-archives/rar7-store-small.rar");
    let rows = scan(bytes.to_vec()).unwrap();
    assert_eq!(rows[0].method, 70);
    assert_eq!(
        unpack(bytes.to_vec()),
        [b"RAR newest algorithm verification sample".to_vec()]
    );
}

#[test]
fn fixed_rar7_store_dictionary_limit_preserves_host_admission() {
    let bytes = include_bytes!("../../../tests/fixtures/native-archives/rar7-dictionary-limit.rar");
    assert_eq!(scan(bytes.to_vec()), Err(ScanError::Limit));
}

#[test]
fn legal_directory_checks_empty_content_while_warning_never_emits() {
    let mut fields = vec![5, 0, 0];
    fields.extend(0u32.to_le_bytes());
    fields.extend([0, 0, 8]);
    fields.extend(b"emptydir");
    let bytes = new_archive(&[modern(2, 2, &fields, &[], &[])]);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert!(rows[0].directory);
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    payload.decode(&rows[0], &mut sink, &control).unwrap();
    assert!(sink.0.is_empty());
    let bytes = new_archive(&[new_file(
        b"../blocked",
        b"abc",
        3,
        0,
        0,
        false,
        crc32(b"abc"),
        &[],
    )]);
    let mut source = Cursor::new(bytes);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert!(!rows[0].payload_supported());
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &control),
        Err(DecodeError::Unsupported)
    );
    assert!(sink.0.is_empty());
}

fn known_math_input(modifier: u32, length: usize) -> Vec<u8> {
    let mut out = Vec::with_capacity(length);
    for k in 1..=length / 4 {
        let k = k as i64;
        let value = (k * k - 3 * k + 1 + i64::from(modifier)).max(0) as u32;
        out.extend(value.to_le_bytes());
    }
    out
}

#[test]
fn fixed_rar5_lz_and_solid_match_independent_mathematical_inputs() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/native-archives/test_read_format_rar5_compressed.rar"
    );
    assert_eq!(unpack(bytes.to_vec()), [known_math_input(0, 1200)]);
    let bytes = include_bytes!(
        "../../../tests/fixtures/native-archives/test_read_format_rar5_multiple_files_solid.rar"
    );
    let expected = (1..=4)
        .map(|n| known_math_input(n, 4096))
        .collect::<Vec<_>>();
    assert_eq!(unpack(bytes.to_vec()), expected);
}

#[test]
fn solid_random_selection_replays_and_charges_every_real_prefix() {
    let bytes = include_bytes!(
        "../../../tests/fixtures/native-archives/test_read_format_rar5_multiple_files_solid.rar"
    );
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    struct Meter {
        bytes: Vec<u8>,
        skipped: u64,
    }
    impl DecodedSink for Meter {
        fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
            self.bytes.extend(b);
            Ok(())
        }
        fn charge_skipped(&mut self, n: u64) -> Result<(), DecodeError> {
            self.skipped += n;
            Ok(())
        }
    }
    let mut skipped = 0;
    for n in [3, 0, 2, 2, 1] {
        let mut sink = Meter {
            bytes: Vec::new(),
            skipped: 0,
        };
        payload.decode(&rows[n], &mut sink, &control).unwrap();
        assert_eq!(sink.bytes, known_math_input(n as u32 + 1, 4096));
        skipped += sink.skipped;
    }
    // WORK now counts selected decoding as well as replayed prefixes; only
    // completed bytes fund prepaid delivery, so ordinary output is not work again.
    assert_eq!(skipped, 53248);
}

fn stored_records_inside_solid_chain() -> Vec<u8> {
    let bytes = include_bytes!(
        "../../../tests/fixtures/native-archives/test_read_format_rar5_multiple_files_solid.rar"
    );
    let index = read_index(
        &mut Cursor::new(bytes),
        &Cancellation::default(),
        ScanLimits::default(),
        None,
    )
    .unwrap()
    .0;
    let mut out = b"Rar!\x1a\x07\x01\x00".to_vec();
    out.extend(modern(1, 0, &[4], &[], &[]));
    for (i, file) in index.files.iter().enumerate() {
        for p in &file.parts {
            let part = &index.parts[*p];
            out.extend(&part.header);
            let start = part.data_at as usize;
            out.extend(&bytes[start..start + part.packed as usize]);
        }
        if i == 0 {
            for (name, info) in [
                (b"store-reset-bit".as_slice(), 0),
                (b"store-solid-bit".as_slice(), 64),
            ] {
                let content = b"Store never enters the LZ dictionary";
                out.extend(new_file(
                    name,
                    content,
                    content.len() as u64,
                    0,
                    info,
                    false,
                    crc32(content),
                    &[],
                ));
            }
        }
    }
    out.extend(new_end(false));
    out
}

#[test]
fn store_between_solid_compressed_files_preserves_dictionary_and_tables() {
    let bytes = stored_records_inside_solid_chain();
    let mut expected = vec![
        known_math_input(1, 4096),
        b"Store never enters the LZ dictionary".to_vec(),
        b"Store never enters the LZ dictionary".to_vec(),
    ];
    expected.extend((2..=4).map(|m| known_math_input(m, 4096)));
    assert_eq!(unpack(bytes.clone()), expected);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    payload
        .decode(rows.last().unwrap(), &mut sink, &control)
        .unwrap();
    assert_eq!(sink.0, known_math_input(4, 4096));
}

#[test]
fn official_authored_e8_filters_match_owned_branch_pattern() {
    for (mixed, archive) in [
        (
            false,
            include_bytes!("../tests/fixtures/e8-only.rar").as_slice(),
        ),
        (
            true,
            include_bytes!("../tests/fixtures/e8e9-input-e8-filter.rar").as_slice(),
        ),
    ] {
        let mut expected = Vec::with_capacity(524288);
        for i in 0..65536i32 {
            let position = i * 8;
            let target = (i * 97) % 524288;
            expected.push(if mixed && i % 2 != 0 { 0xe9 } else { 0xe8 });
            expected.extend((target - position - 5).to_le_bytes());
            expected.extend([0x90, 0x50, 0x58]);
        }
        assert_eq!(unpack(archive.to_vec()), [expected]);
    }
}

#[test]
#[ignore = "explicit native-fixture metadata inventory, not algorithm success"]
fn fixed_external_container_inventory() {
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/native-archives");
    for name in [
        "test_read_format_rar.rar",
        "test_read_format_rar_compress_normal.rar",
        "test_read_format_rar5_compressed.rar",
        "test_read_format_rar5_multiple_files_solid.rar",
        "rar-7.23-rar5-compatible.rar",
    ] {
        let rows =
            scan(std::fs::read(base.join(name)).unwrap()).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(!rows.is_empty(), "{name}");
    }
}

#[test]
#[ignore = "writes only project-owned mathematical fixtures into an explicit build directory"]
fn export_authored_vectors() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_RAR_FIXTURE_OUTPUT").expect("explicit fixture output"),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let old = old_archive(&[
        old_file(b"same", b"abc", 0, 29),
        old_file(b"same", b"defg", 0, 29),
        old_file(b"empty", b"", 0, 29),
    ]);
    let new = new_archive(&[
        new_file(b"same", b"abc", 3, 0, 0, false, crc32(b"abc"), &[]),
        new_file(b"same", b"defg", 4, 0, 1, false, crc32(b"defg"), &[]),
        new_file(b"empty", b"", 0, 0, 0, false, 0, &[]),
    ]);
    std::fs::write(directory.join("authored-old-store.rar"), old).unwrap();
    std::fs::write(directory.join("authored-modern-store.rar"), new).unwrap();
    let unknown = new_archive(&[new_file(
        b"all-unknown",
        b"abc",
        0,
        0,
        0,
        true,
        crc32(b"abc"),
        &[],
    )]);
    std::fs::write(directory.join("authored-all-unknown.rar"), unknown).unwrap();
    let unicode = old_archive(&[old_file(b"abc\0\0\xc0\x01", b"unicode-content", 0x200, 29)]);
    std::fs::write(directory.join("authored-unicode.rar"), unicode).unwrap();
    std::fs::write(
        directory.join("authored-store-in-solid.rar"),
        stored_records_inside_solid_chain(),
    )
    .unwrap();
    for (i, part) in split_parts().into_iter().enumerate() {
        std::fs::write(
            directory.join(format!("authored-split.part{:02}.rar", i + 1)),
            part,
        )
        .unwrap();
    }
}
