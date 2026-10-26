//! Fixed native timestamp compatibility from the code-free time-projection facts.
use crate::{index::RawTime, source::Reader};
use xdzip_codec_api::{ScanError, time::Timestamp};

const EPOCH_NS: u64 = 11_644_473_600_000_000_000;

fn projection(raw: Option<&RawTime>, windows: bool) -> Timestamp {
    let nanoseconds = match raw {
        Some(RawTime::Unix(seconds, fraction)) => {
            let fraction = fraction.map_or(0, |n| {
                let n = n & 0x3fff_ffff;
                if n < 1_000_000_000 { u64::from(n) } else { 0 }
            });
            EPOCH_NS
                .wrapping_add(u64::from(*seconds) * 1_000_000_000)
                .wrapping_add(fraction)
        }
        Some(RawTime::Filetime(ticks)) => ticks.wrapping_mul(100),
        _ => 0,
    };
    if windows {
        Timestamp::Filetime(nanoseconds / 100)
    } else {
        Timestamp::UnixSeconds(nanoseconds.wrapping_sub(EPOCH_NS) / 1_000_000_000)
    }
}

pub(crate) fn modified(raw: Option<&RawTime>, reader: &Reader<'_>) -> Result<u32, ScanError> {
    if let Some(RawTime::Dos(value) | RawTime::DosExtended(value, _, _)) = raw {
        // The odd-second/fraction additions do not affect the original two-
        // second DOS precision. Keep the full raw values in the private index.
        return Ok(*value);
    }
    Ok(reader
        .local_calendar(projection(raw, cfg!(windows)))?
        .and_then(|calendar| calendar.wrapping_dos())
        .unwrap_or(0))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{self, Cursor, Read, Seek, SeekFrom};
    use xdzip_codec_api::{
        Cancellation, ReadSource,
        time::{Calendar, utc_calendar},
    };

    #[test]
    fn raw_fraction_and_platform_modular_projection_match_frozen_facts() {
        assert_eq!(projection(None, true), Timestamp::Filetime(0));
        assert_eq!(
            projection(None, false),
            Timestamp::UnixSeconds(6_802_270_473)
        );
        assert_eq!(
            projection(Some(&RawTime::Filetime(u64::MAX)), true),
            Timestamp::Filetime(u64::MAX.wrapping_mul(100) / 100)
        );
        for raw in [None, Some(1_000_000_000), Some(0xc000_0001)] {
            assert_eq!(
                projection(Some(&RawTime::Unix(0, raw)), false),
                Timestamp::UnixSeconds(0)
            );
        }
        assert_eq!(
            projection(Some(&RawTime::Unix(u32::MAX, Some(999_999_999))), false),
            Timestamp::UnixSeconds(u64::from(u32::MAX))
        );
        assert_eq!(
            projection(Some(&RawTime::Unix(0, Some(999_999_999))), true),
            Timestamp::Filetime((EPOCH_NS + 999_999_999) / 100)
        );
        assert_eq!(
            utc_calendar(projection(None, false))
                .unwrap()
                .wrapping_dos(),
            Some(2_599_795_792)
        );
    }

    struct Host {
        input: Cursor<Vec<u8>>,
        result: Result<Option<Calendar>, ScanError>,
    }
    impl Read for Host {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            self.input.read(bytes)
        }
    }
    impl Seek for Host {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.input.seek(p)
        }
    }
    impl ReadSource for Host {
        fn local_calendar(&self, _: Timestamp) -> Result<Option<Calendar>, ScanError> {
            self.result
        }
    }
    #[test]
    fn host_projection_failure_and_original_dos_values_have_defined_results() {
        let control = Cancellation::default();
        let date = Calendar {
            year: 2108,
            month: 1,
            day: 1,
            hour: 0,
            minute: 0,
            second: 1,
        };
        for (value, expected) in [
            (Ok(Some(date)), Ok(0x0021_0000)),
            (Ok(None), Ok(0)),
            (Err(ScanError::Io), Err(ScanError::Io)),
        ] {
            let mut host = Host {
                input: Cursor::new(vec![0]),
                result: value,
            };
            let reader = Reader::new(&mut host, &control, 64).unwrap();
            assert_eq!(modified(None, &reader), expected);
            assert_eq!(
                modified(
                    Some(&RawTime::DosExtended(0xff9f_bf7d, true, 9_999_999)),
                    &reader
                ),
                Ok(0xff9f_bf7d)
            );
        }
    }
}
