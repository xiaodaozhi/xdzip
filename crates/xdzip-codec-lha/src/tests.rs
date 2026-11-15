use super::*;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits};
#[path = "../../xdzip-compression/src/lzh/fixture_cases.rs"]
pub(crate) mod fixture_cases;
pub(crate) fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/lha")
            .join(name),
    )
    .unwrap()
}
pub(crate) fn scan(bytes: &[u8]) -> Result<Index, ScanError> {
    inspect(
        &mut Cursor::new(bytes),
        &Cancellation::default(),
        ScanLimits::default(),
    )
}
pub(crate) fn refresh(bytes: &mut [u8], header: usize) {
    match bytes[20] {
        0 => {
            bytes[1] = bytes[2..usize::from(bytes[0]) + 2]
                .iter()
                .fold(0u8, |s, b| s.wrapping_add(*b))
        }
        1 => {
            let base = usize::from(bytes[0]) + 2;
            bytes[1] = bytes[2..base].iter().fold(0u8, |s, b| s.wrapping_add(*b));
            bytes[base + 1..base + 3].fill(0);
            let crc = xdzip_compression::crc16(&bytes[..header]);
            bytes[base + 1..base + 3].copy_from_slice(&crc.to_le_bytes());
        }
        2 => {
            bytes[27..29].fill(0);
            let crc = xdzip_compression::crc16(&bytes[..header]);
            bytes[27..29].copy_from_slice(&crc.to_le_bytes());
        }
        3 => {
            bytes[33..35].fill(0);
            let crc = xdzip_compression::crc16(&bytes[..header]);
            bytes[33..35].copy_from_slice(&crc.to_le_bytes());
        }
        _ => panic!(),
    }
}
#[test]
fn independently_framed_members_preserve_header_sizes_skip_sizes_crc_and_physical_order() {
    for &(name, method, header, packed, decoded, crc) in fixture_cases::CASES {
        let bytes = fixture(name);
        let index = scan(&bytes).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(index.records.len(), 1);
        assert_eq!(index.input_used, header as u64 + 1);
        assert_eq!(index.source_size, bytes.len() as u64);
        let record = &index.records[0];
        assert_eq!(record.method.as_slice(), format!("-{method}-").as_bytes());
        assert_eq!(record.header_offset, 0);
        assert_eq!(record.payload_offset, header as u64);
        assert_eq!(record.packed_size, packed as u64);
        assert_eq!(record.size, decoded as u64);
        assert_eq!(record.crc16, Some(crc));
        if name == "lh5-h2-padding.lzh" {
            assert_eq!(
                record.raw_name,
                [vec![b'x'; 218], b".bin".to_vec()].concat()
            );
        } else {
            assert_eq!(record.raw_name, b"hello.bin");
        }
        assert!(!record.directory);
    }
    let mut combined = fixture("lh5-h0.lzh");
    combined.pop();
    let offset = combined.len() as u64;
    combined.extend_from_slice(&fixture("lh5-h2.lzh"));
    let index = scan(&combined).unwrap();
    assert_eq!(index.records.len(), 2);
    assert_eq!(index.records[1].header_offset, offset);
    assert_eq!(index.records[0].raw_name, index.records[1].raw_name);
}
#[test]
fn checksums_truncated_chains_later_damage_and_extra_tails_do_not_publish_partial_index() {
    for name in ["lh5-h0.lzh", "lh5-h1.lzh", "lh5-h2.lzh", "lh5-h3.lzh"] {
        let bytes = fixture(name);
        for cut in 0..bytes.len() {
            assert!(scan(&bytes[..cut]).is_err(), "{name} truncated at {cut}");
        }
        for at in [2, 7, 11, 15, 21, 25, 30] {
            let mut bad = bytes.clone();
            bad[at] ^= 1;
            assert!(scan(&bad).is_err(), "{name} mutated {at}");
        }
        let mut bad = bytes.clone();
        bad.push(0);
        assert_eq!(scan(&bad), Err(ScanError::Invalid));
        let mut later = bytes.clone();
        later.pop();
        later.extend_from_slice(&bad);
        assert_eq!(scan(&later), Err(ScanError::Invalid));
    }
    let mut bad = fixture("lh5-h1.lzh");
    bad[7..11].copy_from_slice(&4u32.to_le_bytes());
    refresh(&mut bad, 41);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
    let mut bad = fixture("lh5-h2.lzh");
    bad[24..26].copy_from_slice(&2u16.to_le_bytes());
    refresh(&mut bad, 43);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
    let mut bad = fixture("lh5-h2.lzh");
    bad[24..26].copy_from_slice(&100u16.to_le_bytes());
    refresh(&mut bad, 43);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
}
#[test]
fn exact_source_entry_name_budgets_and_cancellation_apply_before_metadata_is_returned() {
    let bytes = fixture("lh5-h2.lzh");
    let cancel = Cancellation::default();
    let exact = ScanLimits {
        entries: 1,
        name_bytes: 9,
        total_name_bytes: 9,
        read_bytes: 44,
        ..ScanLimits::default()
    };
    inspect(&mut Cursor::new(&bytes), &cancel, exact).unwrap();
    for limits in [
        ScanLimits {
            entries: 0,
            ..exact
        },
        ScanLimits {
            name_bytes: 8,
            ..exact
        },
        ScanLimits {
            total_name_bytes: 8,
            ..exact
        },
        ScanLimits {
            read_bytes: 43,
            ..exact
        },
    ] {
        assert_eq!(
            inspect(&mut Cursor::new(&bytes), &cancel, limits),
            Err(ScanError::Limit)
        );
    }
    cancel.cancel();
    assert_eq!(
        inspect(&mut Cursor::new(bytes), &cancel, exact),
        Err(ScanError::Cancelled)
    );
    assert_eq!(scan(&[0]).unwrap().records.len(), 0);
    assert_eq!(scan(&[]), Err(ScanError::Invalid));
    assert_eq!(scan(&[0, 0]), Err(ScanError::Invalid));
}
#[test]
fn optional_ancient_crc_store_sizes_and_unknown_methods_remain_explicit() {
    let mut no_crc = fixture("lh0-h0.lzh");
    no_crc.drain(31..33);
    no_crc[0] -= 2;
    refresh(&mut no_crc, 31);
    let record = scan(&no_crc).unwrap().records.remove(0);
    assert_eq!(record.crc16, None);
    assert_eq!(record.payload_offset, 31);
    let mut wrong = fixture("lh0-h0.lzh");
    wrong[11..15].copy_from_slice(&145u32.to_le_bytes());
    refresh(&mut wrong, 33);
    assert_eq!(scan(&wrong), Err(ScanError::Invalid));
    let mut unknown = fixture("lh0-h0.lzh");
    unknown[2..7].copy_from_slice(b"-lhx-");
    refresh(&mut unknown, 33);
    assert_eq!(scan(&unknown).unwrap().records[0].method, *b"-lhx-");
    let mut unsupported = fixture("lh5-h2.lzh");
    unsupported[31] = 0x42;
    refresh(&mut unsupported, 43);
    assert_eq!(scan(&unsupported), Err(ScanError::Unsupported));
}
struct Short {
    source: Cursor<Vec<u8>>,
    fail: Option<ScanError>,
    bare_once: bool,
}
impl Read for Short {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if let Some(error) = self.fail {
            return Err(io::Error::new(io::ErrorKind::Interrupted, error));
        }
        if self.bare_once {
            self.bare_once = false;
            return Err(io::ErrorKind::Interrupted.into());
        }
        let count = output.len().min(1);
        self.source.read(&mut output[..count])
    }
}
impl Seek for Short {
    fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
        self.source.seek(at)
    }
}
impl ReadSource for Short {}
#[test]
fn short_reads_retry_only_bare_interruption_and_preserve_host_error_categories() {
    let cancel = Cancellation::default();
    let mut source = Short {
        source: Cursor::new(fixture("lh5-h1.lzh")),
        fail: None,
        bare_once: true,
    };
    assert_eq!(
        inspect(&mut source, &cancel, ScanLimits::default())
            .unwrap()
            .records
            .len(),
        1
    );
    for error in [
        ScanError::Cancelled,
        ScanError::SourceChanged,
        ScanError::Limit,
        ScanError::Io,
    ] {
        source.fail = Some(error);
        assert_eq!(
            inspect(&mut source, &cancel, ScanLimits::default()),
            Err(error)
        );
    }
}
pub(crate) fn with_extension(kind: u8, data: &[u8]) -> Vec<u8> {
    let original = fixture("lh5-h2.lzh");
    let mut header = original[..43].to_vec();
    header[41..43].copy_from_slice(&((data.len() + 3) as u16).to_le_bytes());
    header.push(kind);
    header.extend_from_slice(data);
    header.extend_from_slice(&[0, 0]);
    let length = header.len();
    header[..2].copy_from_slice(&(length as u16).to_le_bytes());
    refresh(&mut header, length);
    header.extend_from_slice(&original[43..]);
    header
}
#[test]
fn extended_names_modes_duplicate_declarations_and_necessary_boundary_padding_remain_distinct() {
    let bytes = with_extension(2, b"folder\xff");
    let record = scan(&bytes).unwrap().records.remove(0);
    assert_eq!(record.raw_name, b"folder\\hello.bin");
    let bytes = with_extension(0x50, &0xa1ffu16.to_le_bytes());
    let record = scan(&bytes).unwrap().records.remove(0);
    assert_eq!(record.unix_mode, 0xa1ff);
    assert!(!record.directory);
    assert_eq!(
        scan(&with_extension(1, b"another.bin")),
        Err(ScanError::Invalid)
    );
    assert_eq!(scan(&with_extension(2, b"folder")), Err(ScanError::Invalid));
    let bytes = fixture("lh5-h2-padding.lzh");
    assert_eq!(scan(&bytes).unwrap().records[0].payload_offset, 257);
    let mut bad = bytes.clone();
    bad[256] = 1;
    refresh(&mut bad, 257);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
    let mut bad = bytes.clone();
    bad.insert(257, 0);
    bad[..2].copy_from_slice(&258u16.to_le_bytes());
    refresh(&mut bad, 258);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
    let mut directory = fixture("lh0-h2.lzh");
    directory.truncate(43);
    directory.push(0);
    directory[2..7].copy_from_slice(b"-lhd-");
    directory[7..15].fill(0);
    directory[21..23].fill(0);
    refresh(&mut directory, 43);
    let record = scan(&directory).unwrap().records.remove(0);
    assert!(record.directory);
    assert_eq!(record.raw_name, b"hello.bin\\");
    assert_eq!((record.packed_size, record.size), (0, 0));
}
#[test]
fn level3_header_and_extension_sizes_are_admitted_before_allocation_or_range_use() {
    let bytes = fixture("lh5-h3.lzh");
    let index = scan(&bytes).unwrap();
    assert_eq!(index.input_used, 54);
    assert_eq!(index.records[0].payload_offset, 53);
    assert_eq!(index.records[0].level, 3);
    for (size, error) in [
        (31, ScanError::Invalid),
        (1024 * 1024 + 1, ScanError::Limit),
        (u32::MAX, ScanError::Limit),
    ] {
        let mut bad = bytes.clone();
        bad[24..28].copy_from_slice(&size.to_le_bytes());
        assert_eq!(scan(&bad), Err(error));
    }
    for size in [1, 4, u32::MAX] {
        let mut bad = bytes.clone();
        bad[28..32].copy_from_slice(&size.to_le_bytes());
        refresh(&mut bad, 53);
        assert_eq!(scan(&bad), Err(ScanError::Invalid));
    }
    let mut bad = bytes.clone();
    bad[0] = 8;
    refresh(&mut bad, 53);
    assert_eq!(scan(&bad), Err(ScanError::Unsupported));
    let mut bad = bytes.clone();
    bad[33] ^= 1;
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
    let mut bad = bytes;
    bad.insert(53, 0);
    bad[24..28].copy_from_slice(&54u32.to_le_bytes());
    refresh(&mut bad, 54);
    assert_eq!(scan(&bad), Err(ScanError::Invalid));
}
