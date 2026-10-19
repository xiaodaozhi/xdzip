//! Bounded central-directory scanning. No payload decoding, CRC claim or extraction.
mod admission;
mod payload;
mod writer;
use std::io::{Read, Seek, SeekFrom};
use xdzip_codec_api::{Cancellation, Entry, MetadataCodec, ReadSource, ScanError, ScanLimits};
use xdzip_domain::identity::EntryId;

/// Built-in ZIP metadata adapter; preserves duplicate physical entries.
pub struct ZipCodec;

struct Budget<'a> {
    source: &'a mut dyn ReadSource,
    control: &'a Cancellation,
    remaining: u64,
    failure: Option<ScanError>,
}
impl Read for Budget<'_> {
    fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
        let fault =
            self.control.check().err().or_else(|| {
                (self.remaining == 0 && !output.is_empty()).then_some(ScanError::Limit)
            });
        if let Some(error) = fault {
            self.failure = Some(error);
            return Err(std::io::Error::other(error));
        }
        let count = output
            .len()
            .min(usize::try_from(self.remaining).unwrap_or(usize::MAX));
        match self.source.read(&mut output[..count]) {
            Ok(read) => {
                self.remaining -= read as u64;
                Ok(read)
            }
            Err(error) => {
                self.failure = Some(ScanError::Io);
                Err(error)
            }
        }
    }
}
impl Seek for Budget<'_> {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        self.source
            .seek(pos)
            .inspect_err(|_| self.failure = Some(ScanError::Io))
    }
}

impl MetadataCodec for ZipCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        control.check()?;
        let mut reader = Budget {
            source,
            control,
            remaining: limits.read_bytes,
            failure: None,
        };
        let result = scan(&mut reader, control, limits);
        // Preserve host budget/cancellation errors even if the parser wraps its I/O error.
        match reader.failure {
            Some(error) => Err(error),
            None => result,
        }
    }
}

fn scan(
    reader: &mut Budget<'_>,
    control: &Cancellation,
    limits: ScanLimits,
) -> Result<Vec<Entry>, ScanError> {
    let mut buffer = vec![0; 256 * 1024]; // fits the largest standard central header
    let archive =
        rawzip::ZipArchive::from_seekable(reader, &mut buffer).map_err(|_| ScanError::Invalid)?;
    if archive.entries_hint() > limits.entries as u64 {
        return Err(ScanError::Limit);
    }
    // The parser owns ZIP/ZIP64 decoding; this gate admits one physical volume.
    admission::single_volume(&archive)?;
    let mut iterator = archive.entries(&mut buffer);
    let mut rows = Vec::new();
    let mut name_bytes = 0_usize;
    let mut display_bytes = 0_usize;
    while let Some(record) = iterator.next_entry().map_err(|_| ScanError::Invalid)? {
        control.check()?;
        admission::entry_disk(&archive, &record)?;
        let name = record.file_path().as_bytes();
        name_bytes = name_bytes.checked_add(name.len()).ok_or(ScanError::Limit)?;
        if rows.len() >= limits.entries
            || name.len() > limits.name_bytes
            || name_bytes > limits.total_name_bytes
        {
            return Err(ScanError::Limit);
        }
        let utf8 = std::str::from_utf8(name).ok();
        // Escape legacy bytes rather than silently guessing an encoding or losing raw identity.
        let display = match utf8 {
            Some(text) if record.flags().is_utf8() || name.is_ascii() => escape_display(text),
            _ => name
                .iter()
                .map(|b| {
                    if b.is_ascii_graphic() || *b == b' ' {
                        char::from(*b).to_string()
                    } else {
                        format!("\\x{b:02X}")
                    }
                })
                .collect(),
        };
        display_bytes = display_bytes
            .checked_add(display.len())
            .ok_or(ScanError::Limit)?;
        if display_bytes > limits.display_name_bytes {
            return Err(ScanError::Limit);
        }
        // Inspect declared Unix type directly: synthesized modes may combine the
        // default regular type with a trailing-slash directory hint.
        let unix_type = matches!(record.version_made_by().creator_system().as_u8(), 3 | 19)
            .then_some((record.external_attributes() >> 16) & 0o170000);
        let warning = name.starts_with(b"/")
            || name.starts_with(b"\\")
            || name.contains(&b':')
            || name
                .split(|b| *b == b'/' || *b == b'\\')
                .any(|part| part == b"..")
            || name.iter().any(|b| b.is_ascii_control())
            || utf8.is_none()
            || (!record.flags().is_utf8() && !name.is_ascii())
            || unix_type.is_some_and(|kind| !matches!(kind, 0 | 0o040000 | 0o100000 | 0o120000))
            || utf8.is_some_and(|text| text.chars().any(is_display_control));
        let id = u64::try_from(rows.len())
            .ok()
            .and_then(|n| n.checked_add(1))
            .and_then(|n| EntryId::try_from(n).ok())
            .ok_or(ScanError::Limit)?;
        rows.push(Entry {
            format: xdzip_codec_api::Format::Zip,
            id,
            raw_name: name.to_vec(),
            display_name: display,
            directory: record.is_dir(),
            link: unix_type == Some(0o120000),
            unix_mode: unix_type.map_or(0, |_| record.external_attributes() >> 16),
            size: record.uncompressed_size_hint(),
            packed_size: record.compressed_size_hint(),
            method: record.compression_method().as_u16(),
            modified_dos: validated_dos(record.last_modified_dos()),
            encrypted: record.flags().is_encrypted(),
            warning,
        });
    }
    if rows.len() as u64 != archive.entries_hint() {
        return Err(ScanError::Invalid);
    }
    Ok(rows)
}

fn is_display_control(c: char) -> bool {
    c.is_control()
        || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')
}

fn escape_display(text: &str) -> String {
    use std::fmt::Write;
    let mut output = String::with_capacity(text.len());
    for c in text.chars() {
        if is_display_control(c) {
            // Writing to a String is infallible apart from allocation failure.
            write!(output, "\\u{{{:X}}}", u32::from(c)).expect("String formatting");
        } else {
            output.push(c);
        }
    }
    output
}

// rawzip's component accessors clamp invalid values. Preserve only valid raw
// local timestamps; never turn an absent/invalid ZIP date into an invented date.
fn validated_dos(value: rawzip::time::DosDateTime) -> u32 {
    let date = value.packed_date();
    let time = value.packed_time();
    if ((date >> 5) & 15) != u16::from(value.month())
        || (date & 31) != u16::from(value.day())
        || (time >> 11) != u16::from(value.hour())
        || ((time >> 5) & 63) != u16::from(value.minute())
        || (time & 31) * 2 != u16::from(value.second())
    {
        return 0;
    }
    u32::from(date) << 16 | u32::from(time)
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
        legacy[central + 8..central + 10].copy_from_slice(&1_u16.to_le_bytes());
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
