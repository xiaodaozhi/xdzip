use super::*;
use std::cell::Cell;
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};

struct Host {
    bytes: Cursor<Vec<u8>>,
    oem_calls: Cell<usize>,
    ansi_calls: Cell<usize>,
    local_calls: Cell<usize>,
    error: Option<ScanError>,
}
impl Default for Host {
    fn default() -> Self {
        Self {
            bytes: Cursor::new(vec![0]),
            oem_calls: Cell::new(0),
            ansi_calls: Cell::new(0),
            local_calls: Cell::new(0),
            error: None,
        }
    }
}
impl Read for Host {
    fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
        self.bytes.read(out)
    }
}
impl Seek for Host {
    fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
        self.bytes.seek(at)
    }
}
impl ReadSource for Host {
    fn oem_to_ansi_wide(
        &self,
        input: &[u16],
        output: &mut [u16],
    ) -> Result<Option<usize>, ScanError> {
        self.oem_calls.set(self.oem_calls.get() + 1);
        if let Some(error) = self.error {
            return Err(error);
        }
        assert_eq!((input.len(), output.len()), (512, 512));
        let end = input.iter().position(|u| *u == 0).unwrap();
        self.oem_calls.set(end);
        for (to, from) in output.iter_mut().zip(&input[..end]) {
            *to = if *from < 128 { *from } else { u16::from(b'X') };
        }
        output[end] = 0;
        Ok(Some(end))
    }
    fn wide_to_ansi(&self, input: &[u16], output: &mut [u8]) -> Result<Option<usize>, ScanError> {
        self.ansi_calls.set(self.ansi_calls.get() + 1);
        if let Some(error) = self.error {
            return Err(error);
        }
        assert!(input.len() <= 128 && input.last() == Some(&0));
        assert_eq!(output.len(), 512);
        let mut at = 0;
        for &unit in &input[..input.len() - 1] {
            if unit < 128 {
                output[at] = unit as u8;
                at += 1;
            } else {
                output[at..at + 2].copy_from_slice(&[0x81, 0x40]);
                at += 2;
            }
        }
        output[at] = 0;
        Ok(Some(at))
    }
    fn wide_to_local_prefix(
        &self,
        input: &[u32],
        output: &mut [u8],
    ) -> Result<Option<usize>, ScanError> {
        self.local_calls.set(self.local_calls.get() + 1);
        if let Some(error) = self.error {
            return Err(error);
        }
        assert!(input.len() <= 128 && input.last() == Some(&0));
        let count = input
            .iter()
            .take_while(|unit| **unit != 0 && **unit < 128)
            .count();
        for (to, unit) in output.iter_mut().zip(&input[..count]) {
            *to = *unit as u8;
        }
        output[count] = 0;
        Ok(Some(count))
    }
}

#[test]
fn legacy_full_callback_admission_precedes_nul_and_crypt_prefixes() {
    assert_eq!(admit(&"a".repeat(511)), Ok(()));
    assert_eq!(admit(&"a".repeat(512)), Err(DecodeError::Limit));
    assert_eq!(
        admit(&("\0".to_owned() + &"a".repeat(511))),
        Err(DecodeError::Limit)
    );
    assert_eq!(valid_count(&[0u8; 512], 512), Err(DecodeError::Unsupported));
    assert_eq!(valid_count(&[1u8, 1], 1), Err(DecodeError::Unsupported));
    assert_eq!(valid_count(&[0u8, 0], 1), Err(DecodeError::Unsupported));
}

#[test]
fn windows_oem_receives_full_prefix_and_crypt_boundary_keeps_split_surrogate() {
    let control = Cancellation::default();
    let mut host = Host::default();
    let reader = Reader::new(&mut host, &control, 64).unwrap();
    let dos = Descriptor {
        host: 0,
        ..Descriptor::default()
    };
    let projected = windows_wide(&"密".repeat(200), &dos, &reader).unwrap();
    assert_eq!(&projected[..127], &[u16::from(b'X'); 127]);
    assert_eq!(projected[127], 0);
    let other = Descriptor {
        host: 2,
        ..Descriptor::default()
    };
    let boundary = windows_wide(&("A".repeat(126) + "😀"), &other, &reader).unwrap();
    assert_eq!((boundary[126], boundary[127]), (0xd83d, 0));
    drop(reader);
    assert_eq!(host.oem_calls.get(), 200);
}

#[cfg(windows)]
#[test]
fn windows_host_projection_precedes_wide_and_raw_dbcs_byte_truncation() {
    let control = Cancellation::default();
    let mut host = Host::default();
    let reader = Reader::new(&mut host, &control, 64).unwrap();
    let dos = Descriptor {
        host: 0,
        ..Descriptor::default()
    };
    assert_eq!(
        narrow(&"密".repeat(200), &dos, &reader)
            .unwrap()
            .projected_bytes(),
        &[b'X'; 127]
    );
    let other = Descriptor {
        host: 2,
        ..Descriptor::default()
    };
    let plain = narrow(&"密".repeat(200), &other, &reader).unwrap();
    assert_eq!(&plain.projected_bytes()[..126], &[0x81, 0x40].repeat(63));
    assert_eq!(plain.projected_bytes()[126], 0x81);
    assert_eq!(
        wide(&"密".repeat(200), &dos, &reader).unwrap().as_slice(),
        &[u16::from(b'X'); 127]
    );
    assert_eq!(
        wide(&"密".repeat(200), &other, &reader).unwrap().as_slice(),
        &[0x5bc6; 127]
    );
    let ancient = Descriptor {
        ancient: true,
        host: 0,
        ..Descriptor::default()
    };
    assert_eq!(
        narrow("密", &ancient, &reader).unwrap().projected_bytes(),
        [0x81, 0x40]
    );
}

#[cfg(all(not(windows), not(target_os = "macos")))]
#[test]
fn unix_plain_locale_failure_keeps_prefix_and_marker_path_recovers_raw_bytes() {
    let control = Cancellation::default();
    let mut host = Host::default();
    let reader = Reader::new(&mut host, &control, 64).unwrap();
    let d = Descriptor::default();
    assert_eq!(narrow("A密B", &d, &reader).unwrap().projected_bytes(), b"A");
    assert_eq!(
        narrow("A\u{fffe}密B\u{e080}", &d, &reader)
            .unwrap()
            .projected_bytes(),
        b"A_B\x80"
    );
    assert_eq!(wide("\u{10000}", &d, &reader).unwrap().as_slice(), [0]);
}

#[cfg(target_os = "macos")]
#[test]
fn mac_utf8_prefix_is_cut_by_raw_bytes_after_scalar_truncation() {
    let control = Cancellation::default();
    let mut host = Host::default();
    let reader = Reader::new(&mut host, &control, 64).unwrap();
    let d = Descriptor::default();
    let projected = narrow(&"密".repeat(127), &d, &reader).unwrap();
    assert_eq!(
        &projected.projected_bytes()[..126],
        "密".repeat(42).as_bytes()
    );
    assert_eq!(projected.projected_bytes()[126], 0xe5);
}

#[test]
fn host_projection_source_classes_are_not_password_or_integrity_failures() {
    let control = Cancellation::default();
    let mut host = Host {
        error: Some(ScanError::SourceChanged),
        ..Host::default()
    };
    let reader = Reader::new(&mut host, &control, 64).unwrap();
    let result = if cfg!(target_os = "macos") {
        windows_wide("密", &Descriptor::default(), &reader).map(|_| ())
    } else {
        narrow("密", &Descriptor::default(), &reader).map(|_| ())
    };
    assert_eq!(result, Err(DecodeError::SourceIo));
}
