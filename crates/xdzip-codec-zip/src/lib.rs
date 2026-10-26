// crates/xdzip-codec-zip/src/lib.rs

//! Bounded central-directory scanning. No payload decoding, CRC claim or extraction.
mod admission;
mod aes;
mod names;
mod payload;
pub(crate) mod record;
mod rewrite;
pub mod split;
mod writer;
#[cfg(test)]
use std::io::{Read, Seek, SeekFrom};
use xdzip_codec_api::{Cancellation, Entry, MetadataCodec, ReadSource, ScanError, ScanLimits};
use xdzip_domain::identity::EntryId;

/// Built-in ZIP metadata adapter; preserves duplicate physical entries.
pub struct ZipCodec;
impl MetadataCodec for ZipCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        let directory = record::read_directory(source, control, limits)?;
        let mut rows = Vec::with_capacity(directory.records.len());
        let mut legacy = Vec::with_capacity(directory.records.len());
        let mut decoded_bytes = 0usize;
        for record in directory.records {
            control.check()?;
            let utf8 = record.utf8();
            let encrypted = record.encrypted();
            let directory = record.is_dir();
            let unicode = names::unicode_path(&record.name, utf8, &record.extra);
            let (decoded_name, name_encoding) = match unicode {
                Ok(Some(name)) => (
                    Some(name),
                    xdzip_codec_api::names::NameEncoding::UnicodePath,
                ),
                Ok(None) => (None, xdzip_codec_api::names::NameEncoding::Utf8),
                Err(()) => (None, xdzip_codec_api::names::NameEncoding::Unresolved),
            };
            let decoded_len = decoded_name.as_ref().map_or(0, String::len);
            decoded_bytes = decoded_bytes
                .checked_add(decoded_len)
                .ok_or(ScanError::Limit)?;
            if decoded_len > limits.name_bytes || decoded_bytes > limits.display_name_bytes {
                return Err(ScanError::Limit);
            }
            let unix_type = matches!(record.version_made_by >> 8, 3 | 19)
                .then_some((record.external_attributes >> 16) & 0o170000);
            let warning =
                unix_type.is_some_and(|kind| !matches!(kind, 0 | 0o040000 | 0o100000 | 0o120000));
            legacy.push(!utf8);
            let id = u64::try_from(rows.len())
                .ok()
                .and_then(|n| n.checked_add(1))
                .and_then(|n| EntryId::try_from(n).ok())
                .ok_or(ScanError::Limit)?;
            rows.push(Entry {
                format: xdzip_codec_api::Format::Zip,
                id,
                raw_name: record.name,
                decoded_name,
                name_encoding,
                display_name: String::new(),
                directory,
                link: unix_type == Some(0o120000),
                unix_mode: unix_type.map_or(0, |_| record.external_attributes >> 16),
                size: record.size,
                size_known: true,
                packed_size: record.packed_size,
                method: record.method,
                modified_dos: validated_dos(record.modified_dos),
                encrypted,
                password_supported: encrypted
                    && record.method == 99
                    && aes::supported(record::ExtraFields::new(&record.extra), record.crc),
                warning,
            });
        }
        xdzip_codec_api::names::resolve(&mut rows, &legacy, true, control, limits)?;
        Ok(rows)
    }
}

// Preserve valid local timestamp bits; invalid components remain unknown.
fn validated_dos(value: u32) -> u32 {
    let date = value >> 16;
    let time = value & 0xffff;
    if !(1..=12).contains(&((date >> 5) & 15))
        || !(1..=31).contains(&(date & 31))
        || time >> 11 > 23
        || (time >> 5) & 63 > 59
        || (time & 31) > 29
    {
        0
    } else {
        value
    }
}

#[cfg(test)]
#[path = "../../../tests/support/zip64.rs"]
mod zip64_fixture;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    const NORMAL: &[u8] = include_bytes!("../../../tests/fixtures/zip/normal.zip");
    fn scan_bytes(bytes: &[u8], limits: ScanLimits) -> Result<Vec<Entry>, ScanError> {
        ZipCodec.scan(&mut Cursor::new(bytes), &Cancellation::default(), limits)
    }
    #[test]
    fn classic_zip_accepts_the_exact_65535_record_boundary() {
        let bytes = zip64_fixture::classic(u16::MAX as usize);
        let rows = scan_bytes(&bytes, ScanLimits::default()).unwrap();
        assert_eq!(rows.len(), u16::MAX as usize);
    }
    #[test]
    fn zip64_hundred_thousand_records_and_exact_64_bit_sizes() {
        let bytes = zip64_fixture::archive(xdzip_codec_api::MAX_ENTRIES);
        let rows = scan_bytes(&bytes, ScanLimits::default()).unwrap();
        assert_eq!(rows.len(), 100_000);
        assert_eq!(rows.last().unwrap().id.get(), 100_000);
        assert_eq!(rows.last().unwrap().display_name, "item-099999.txt");
        assert!(matches!(
            scan_bytes(
                &bytes,
                ScanLimits {
                    entries: 99_999,
                    ..Default::default()
                }
            ),
            Err(ScanError::Limit)
        ));
        let bytes = zip64_fixture::declared_sizes((1 << 54) + 7, (1 << 53) + 3, 0);
        let rows = scan_bytes(&bytes, ScanLimits::default()).unwrap();
        assert_eq!(
            (rows[0].size, rows[0].packed_size),
            ((1 << 54) + 7, (1 << 53) + 3)
        );
    }
    #[test]
    fn zip64_malformed_and_split_volumes_fail_closed() {
        let bytes = zip64_fixture::archive(1);
        let mut split = bytes.clone();
        let len = split.len();
        split[len - 26..len - 22].copy_from_slice(&2u32.to_le_bytes());
        assert!(matches!(
            scan_bytes(&split, ScanLimits::default()),
            Err(ScanError::Unsupported)
        ));
        let split = zip64_fixture::declared_sizes(1, 1, 1);
        assert!(matches!(
            scan_bytes(&split, ScanLimits::default()),
            Err(ScanError::Unsupported)
        ));
        let mut broken = bytes.clone();
        broken[len - 94] = 43;
        assert!(scan_bytes(&broken, ScanLimits::default()).is_err());
        let mut missing = bytes.clone();
        missing[len - 42..len - 38].fill(0);
        assert!(scan_bytes(&missing, ScanLimits::default()).is_err());
        let limited = ScanLimits {
            display_name_bytes: 1,
            ..Default::default()
        };
        assert!(matches!(scan_bytes(&bytes, limited), Err(ScanError::Limit)));
    }
    #[test]
    fn physical_duplicates_unicode_and_declared_methods_survive() {
        let rows = scan_bytes(NORMAL, ScanLimits::default()).unwrap();
        assert_eq!(rows.len(), 5);
        assert_eq!(rows[1].display_name, "旅行/说明.txt");
        assert_eq!(rows[1].method, 8);
        assert_eq!(rows[2].raw_name, rows[3].raw_name);
        assert_ne!(rows[2].id, rows[3].id);
        assert_eq!(rows[4].size, 0);
        assert!(rows[0].directory);
        assert!(!rows[0].warning);
    }
    #[test]
    fn filename_encoding_preserves_raw_identity_and_resolves_unlabelled_names() {
        let name = "中文.txt";
        let bytes = zip64_fixture::archive_names(&[name.into()]);
        let rows = scan_bytes(&bytes, ScanLimits::default()).unwrap();
        assert_eq!(rows[0].display_name, name);
        assert!(!rows[0].warning);
        for (raw, expected) in [
            (name.as_bytes(), name),
            (b"\xD6\xD0\xCE\xC4.txt".as_slice(), name),
            (b"caf\x82.txt".as_slice(), "café.txt"),
        ] {
            let mut bytes = zip64_fixture::archive_names(&["a".repeat(raw.len())]);
            let central = bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
            bytes[6..8].copy_from_slice(&0u16.to_le_bytes());
            bytes[central + 8..central + 10].copy_from_slice(&0u16.to_le_bytes());
            bytes[30..30 + raw.len()].copy_from_slice(raw);
            bytes[central + 46..central + 46 + raw.len()].copy_from_slice(raw);
            let rows = scan_bytes(&bytes, ScanLimits::default()).unwrap();
            assert_eq!(rows[0].raw_name, raw);
            assert!(!rows[0].warning);
            assert_eq!(rows[0].display_name, expected);
            assert_eq!(rows[0].name_bytes(), expected.as_bytes());
        }
    }
    #[test]
    fn local_dos_date_preserves_valid_bits_and_rejects_invalid_components() {
        let rows = scan_bytes(NORMAL, ScanLimits::default()).unwrap();
        assert_eq!(rows[0].modified_dos, ((46u32 << 9) | (1 << 5) | 1) << 16);
        let mut bytes = NORMAL.to_vec();
        let central = bytes.windows(4).position(|w| w == b"PK\x01\x02").unwrap();
        bytes[central + 14..central + 16].copy_from_slice(&0u16.to_le_bytes());
        assert_eq!(
            scan_bytes(&bytes, ScanLimits::default()).unwrap()[0].modified_dos,
            0
        );
    }
    #[test]
    fn empty_is_valid_but_truncation_and_count_mismatch_are_not_ready() {
        assert!(
            scan_bytes(
                include_bytes!("../../../tests/fixtures/zip/empty.zip"),
                ScanLimits::default()
            )
            .unwrap()
            .is_empty()
        );
        for data in [
            include_bytes!("../../../tests/fixtures/zip/truncated.zip").as_slice(),
            include_bytes!("../../../tests/fixtures/zip/wrong-count.zip").as_slice(),
            b"not a zip",
        ] {
            assert!(matches!(
                scan_bytes(data, ScanLimits::default()),
                Err(ScanError::Invalid)
            ));
        }
    }
    #[test]
    fn every_admission_budget_fails_closed() {
        for limits in [
            ScanLimits {
                entries: 1,
                ..Default::default()
            },
            ScanLimits {
                name_bytes: 3,
                ..Default::default()
            },
            ScanLimits {
                total_name_bytes: 3,
                ..Default::default()
            },
            ScanLimits {
                read_bytes: 1,
                ..Default::default()
            },
        ] {
            assert!(matches!(scan_bytes(NORMAL, limits), Err(ScanError::Limit)));
        }
    }
    #[test]
    fn dangerous_paths_are_retained_and_flagged_never_normalized_into_output() {
        let rows = scan_bytes(
            include_bytes!("../../../tests/fixtures/zip/unsafe-names.zip"),
            ScanLimits::default(),
        )
        .unwrap();
        assert_eq!(rows[0].raw_name, b"../escape.txt");
        assert!(rows[..3].iter().all(|row| row.warning));
        assert!(rows[3].display_name.contains("\\u{202E}"));
    }
    #[test]
    fn unix_links_are_identified_separately_from_other_warnings() {
        let mut bytes = NORMAL.to_vec();
        let central = bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
        bytes[central + 5] = 3;
        bytes[central + 38..central + 42].copy_from_slice(&((0o120777u32) << 16).to_le_bytes());
        let rows = scan_bytes(&bytes, ScanLimits::default()).unwrap();
        assert!(rows[0].link && !rows[0].warning);
        assert_eq!(rows[0].unix_mode, 0o120777);
        assert!(rows[1..].iter().all(|row| !row.link));
    }
    #[test]
    fn split_disk_and_legacy_encrypted_metadata() {
        let central = NORMAL.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
        let mut split = NORMAL.to_vec();
        split[central + 34] = 1;
        assert!(matches!(
            scan_bytes(&split, ScanLimits::default()),
            Err(ScanError::Unsupported)
        ));
        let mut legacy = NORMAL.to_vec();
        legacy[central + 8..central + 10].copy_from_slice(&0x0801_u16.to_le_bytes());
        legacy[central + 46] = 255;
        let rows = scan_bytes(&legacy, ScanLimits::default()).unwrap();
        assert!(rows[0].encrypted && rows[0].warning);
        assert_eq!(rows[0].raw_name[0], 255);
        assert!(rows[0].display_name.starts_with("\\xFF"));
    }
    #[test]
    fn cancellation_before_and_during_io_discards_scan() {
        let control = Cancellation::default();
        control.cancel();
        assert!(matches!(
            ZipCodec.scan(&mut Cursor::new(NORMAL), &control, ScanLimits::default()),
            Err(ScanError::Cancelled)
        ));
        struct CancelOnRead<'a>(Cursor<&'a [u8]>, &'a Cancellation);
        impl ReadSource for CancelOnRead<'_> {}
        impl Read for CancelOnRead<'_> {
            fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
                self.1.cancel();
                self.0.read(b)
            }
        }
        impl Seek for CancelOnRead<'_> {
            fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
                self.0.seek(p)
            }
        }
        let control = Cancellation::default();
        let mut source = CancelOnRead(Cursor::new(NORMAL), &control);
        assert!(matches!(
            ZipCodec.scan(&mut source, &control, ScanLimits::default()),
            Err(ScanError::Cancelled)
        ));
    }
}

#[cfg(test)]
#[path = "../../../tests/support/legacy_names.rs"]
mod legacy_fixture;
#[cfg(test)]
mod names_tests;
