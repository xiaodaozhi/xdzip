//! Strict AES metadata and a bounded zip-rs crypto adapter, with no output authority.
use rawzip::{ReaderAt, extra_fields::ExtraFields};
use std::io::{self, Read, Seek, SeekFrom};
use xdzip_codec_api::payload::DecodeError;

/// WinZip AE-1 requires CRC in addition to authentication; AE-2 uses authentication.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct Profile {
    pub version: u16,
    pub method: u16,
}

pub(super) fn profile(mut fields: ExtraFields<'_>) -> Result<Option<Profile>, DecodeError> {
    let mut found = None;
    for (id, bytes) in fields.by_ref() {
        if id.as_u16() != 0x9901 {
            continue;
        }
        if found.is_some() || bytes.len() != 7 || &bytes[2..4] != b"AE" {
            return Err(DecodeError::Integrity);
        }
        let version = u16::from_le_bytes([bytes[0], bytes[1]]);
        let method = u16::from_le_bytes([bytes[5], bytes[6]]);
        if !matches!(version, 1 | 2) || bytes[4] != 3 || !matches!(method, 0 | 8) {
            return Err(DecodeError::Unsupported);
        }
        found = Some(Profile { version, method });
    }
    if !fields.remaining_bytes().is_empty() {
        return Err(DecodeError::Integrity);
    }
    Ok(found)
}

pub(super) fn supported(fields: ExtraFields<'_>, crc: u32) -> bool {
    profile(fields)
        .is_ok_and(|profile| profile.is_some_and(|profile| profile.version == 1 || crc == 0))
}

/// Presents only one already-validated ciphertext range as an AE-2 Store record.
/// The caller first checks the original local/central headers, AES profile,
/// descriptor and the exact physical range before constructing this view. Its
/// fixed private headers are solely a crypto adapter, never admission evidence.
/// zip-rs owns key derivation, AES and HMAC; the outer decoder owns the actual
/// compression method, StreamEnd, size and AE-1 CRC. This avoids its permissive
/// decompression adapter without copying ciphertext or granting filesystem access.
pub(super) struct CryptoView<'a, R> {
    source: &'a R,
    start: u64,
    packed: u64,
    position: u64,
    header: Vec<u8>,
    trailer: Vec<u8>,
}
impl<'a, R: ReaderAt> CryptoView<'a, R> {
    pub fn new(source: &'a R, start: u64, packed: u64) -> Result<Self, DecodeError> {
        start.checked_add(packed).ok_or(DecodeError::Integrity)?;
        let plain = packed.checked_sub(28).ok_or(DecodeError::Integrity)?;
        // Always use ZIP64 in this private framing, including for small records.
        // No attacker-supplied filename or extra field is delegated to zip-rs.
        let mut extra = vec![0x01, 0x99, 7, 0, 2, 0, b'A', b'E', 3, 0, 0];
        extra.extend_from_slice(&[1, 0, 16, 0]);
        extra.extend_from_slice(&plain.to_le_bytes());
        extra.extend_from_slice(&packed.to_le_bytes());
        let mut header = vec![0; 30];
        header[..4].copy_from_slice(b"PK\x03\x04");
        header[4..6].copy_from_slice(&51u16.to_le_bytes());
        header[6..8].copy_from_slice(&1u16.to_le_bytes());
        header[8..10].copy_from_slice(&99u16.to_le_bytes());
        header[18..26].fill(255);
        header[26..28].copy_from_slice(&1u16.to_le_bytes());
        header[28..30].copy_from_slice(&(extra.len() as u16).to_le_bytes());
        header.push(b'x');
        header.extend_from_slice(&extra);
        let directory_at = (header.len() as u64)
            .checked_add(packed)
            .ok_or(DecodeError::Integrity)?;
        directory_at
            .checked_add(256)
            .ok_or(DecodeError::Integrity)?;
        let mut trailer = vec![0; 46];
        trailer[..4].copy_from_slice(b"PK\x01\x02");
        trailer[4..6].copy_from_slice(&51u16.to_le_bytes());
        trailer[6..8].copy_from_slice(&51u16.to_le_bytes());
        trailer[8..10].copy_from_slice(&1u16.to_le_bytes());
        trailer[10..12].copy_from_slice(&99u16.to_le_bytes());
        trailer[20..28].fill(255);
        trailer[28..30].copy_from_slice(&1u16.to_le_bytes());
        trailer[30..32].copy_from_slice(&(extra.len() as u16).to_le_bytes());
        trailer.push(b'x');
        trailer.extend_from_slice(&extra);
        let directory_size = trailer.len() as u64;
        let zip64_at = directory_at + directory_size;
        let mut end = vec![0; 56];
        end[..4].copy_from_slice(b"PK\x06\x06");
        end[4..12].copy_from_slice(&44u64.to_le_bytes());
        end[12..14].copy_from_slice(&51u16.to_le_bytes());
        end[14..16].copy_from_slice(&51u16.to_le_bytes());
        end[24..32].copy_from_slice(&1u64.to_le_bytes());
        end[32..40].copy_from_slice(&1u64.to_le_bytes());
        end[40..48].copy_from_slice(&directory_size.to_le_bytes());
        end[48..56].copy_from_slice(&directory_at.to_le_bytes());
        trailer.extend_from_slice(&end);
        let mut locator = [0; 20];
        locator[..4].copy_from_slice(b"PK\x06\x07");
        locator[8..16].copy_from_slice(&zip64_at.to_le_bytes());
        locator[16..20].copy_from_slice(&1u32.to_le_bytes());
        trailer.extend_from_slice(&locator);
        let mut end = [0; 22];
        end[..4].copy_from_slice(b"PK\x05\x06");
        end[8..12].fill(255);
        end[12..20].fill(255);
        trailer.extend_from_slice(&end);
        Ok(Self {
            source,
            start,
            packed,
            position: 0,
            header,
            trailer,
        })
    }
    fn len(&self) -> u64 {
        self.header.len() as u64 + self.packed + self.trailer.len() as u64
    }
}
impl<R: ReaderAt> Read for CryptoView<'_, R> {
    fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
        let header = self.header.len() as u64;
        let payload_end = header + self.packed;
        let count = if self.position < header {
            (&self.header[self.position as usize..]).read(bytes)?
        } else if self.position < payload_end {
            let bound = (payload_end - self.position).min(bytes.len() as u64) as usize;
            self.source
                .read_at(&mut bytes[..bound], self.start + (self.position - header))?
        } else if self.position < self.len() {
            (&self.trailer[(self.position - payload_end) as usize..]).read(bytes)?
        } else {
            0
        };
        self.position += count as u64;
        Ok(count)
    }
}
impl<R: ReaderAt> Seek for CryptoView<'_, R> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.position = match position {
            SeekFrom::Start(at) => Some(at),
            SeekFrom::Current(delta) => self.position.checked_add_signed(delta),
            SeekFrom::End(delta) => self.len().checked_add_signed(delta),
        }
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid_crypto_seek"))?;
        Ok(self.position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_unique_complete_supported_aes_profiles_are_admitted() {
        let valid = [0x01, 0x99, 7, 0, 2, 0, b'A', b'E', 3, 8, 0];
        assert_eq!(
            profile(ExtraFields::new(&valid)),
            Ok(Some(Profile {
                version: 2,
                method: 8
            }))
        );
        for index in [4, 6, 8, 9] {
            let mut other = valid;
            other[index] = if index == 8 { 2 } else { 4 };
            assert!(profile(ExtraFields::new(&other)).is_err());
        }
        for length in 1..valid.len() {
            assert_eq!(
                profile(ExtraFields::new(&valid[..length])),
                Err(DecodeError::Integrity)
            );
        }
        let duplicate = [valid.as_slice(), valid.as_slice()].concat();
        assert_eq!(
            profile(ExtraFields::new(&duplicate)),
            Err(DecodeError::Integrity)
        );
    }

    #[test]
    fn crypto_view_is_bounded_and_never_exposes_adjacent_input() {
        let bytes = [b"prefix".as_slice(), &[7; 28], b"suffix"].concat();
        let mut view = CryptoView::new(&bytes, 6, 28).unwrap();
        let start = view.header.len() as u64;
        view.seek(SeekFrom::Start(start)).unwrap();
        let mut output = [0; 100];
        assert_eq!(view.read(&mut output).unwrap(), 28);
        assert_eq!(&output[..28], &[7; 28]);
        let end = view.len();
        view.seek(SeekFrom::End(0)).unwrap();
        assert_eq!(view.read(&mut output).unwrap(), 0);
        view.seek(SeekFrom::End(100)).unwrap();
        assert_eq!(view.read(&mut output).unwrap(), 0);
        view.seek(SeekFrom::End(0)).unwrap();
        assert_eq!(view.seek(SeekFrom::Current(-1)).unwrap(), end - 1);
        assert!(view.seek(SeekFrom::Start(0)).is_ok());
        assert!(view.seek(SeekFrom::Current(-1)).is_err());
        assert!(CryptoView::new(&bytes, 0, 27).is_err());
        assert!(CryptoView::new(&bytes, 0, u64::MAX).is_err());
        assert!(CryptoView::new(&bytes, u64::MAX, 28).is_err());
    }
}
