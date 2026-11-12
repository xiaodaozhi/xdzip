//! Explicit, authenticated, in-memory previews. No temporary file or external application.
use crate::{Archive, extraction::Password};
use std::io::{self, Read, Seek, SeekFrom};
use xdzip_codec_api::payload::{DecodeError, DecodedSink};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError};

/// Maximum complete text content, before UTF decoding.
pub const TEXT_BYTES: usize = 256 * 1024;
/// Maximum complete encoded image content. Rendering has a separate pixel budget.
pub const IMAGE_BYTES: usize = 16 * 1024 * 1024;
const WORK_BYTES: u64 = 64 * 1024 * 1024;

/// Complete verified content, with the source session and physical identity.
pub struct Content {
    /// Immutable source session.
    pub session: u64,
    /// Physical identity, independent of display order.
    pub entry: u64,
    /// 1 is plain text; image kinds 2..12 are fixed by the preview bridge contract.
    pub kind: u8,
    /// Bounded bytes. Text is never interpreted as markup.
    pub bytes: Vec<u8>,
}
impl std::fmt::Debug for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Content")
            .field("session", &self.session)
            .field("entry", &self.entry)
            .field("kind", &self.kind)
            .field("byte_count", &self.bytes.len())
            .finish()
    }
}
struct Sink {
    bytes: zeroize::Zeroizing<Vec<u8>>,
    limit: usize,
    work: u64,
    auxiliary: u64,
}
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.work = self
            .work
            .checked_add(bytes.len() as u64)
            .filter(|n| *n <= WORK_BYTES)
            .ok_or(DecodeError::Limit)?;
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > self.limit)
        {
            return Err(DecodeError::Limit);
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.work = self
            .work
            .checked_add(bytes)
            .filter(|n| *n <= WORK_BYTES)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.auxiliary = self
            .auxiliary
            .checked_add(bytes)
            .filter(|n| *n <= 256 * 1024 * 1024)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
}
struct Source<'a> {
    source: &'a mut dyn ReadSource,
    remaining: u64,
    control: &'a Cancellation,
}
impl Read for Source<'_> {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        self.control.check().map_err(io::Error::other)?;
        if self.remaining == 0 && !output.is_empty() {
            return Err(io::Error::other(ScanError::Limit));
        }
        let len = output.len().min(self.remaining as usize);
        let count = self.source.read(&mut output[..len])?;
        self.remaining -= count as u64;
        Ok(count)
    }
}
impl Seek for Source<'_> {
    fn seek(&mut self, at: SeekFrom) -> io::Result<u64> {
        self.control.check().map_err(io::Error::other)?;
        self.source.seek(at)
    }
}
impl ReadSource for Source<'_> {
    fn volume_lengths(&self) -> Option<&[u64]> {
        self.source.volume_lengths()
    }
    fn volume_names(&self) -> Option<&[Vec<u8>]> {
        self.source.volume_names()
    }
    fn local_calendar(
        &self,
        t: xdzip_codec_api::time::Timestamp,
    ) -> Result<Option<xdzip_codec_api::time::Calendar>, ScanError> {
        self.source.local_calendar(t)
    }
    fn oem_to_ansi_wide(&self, i: &[u16], o: &mut [u16]) -> Result<Option<usize>, ScanError> {
        self.source.oem_to_ansi_wide(i, o)
    }
    fn wide_to_ansi(&self, i: &[u16], o: &mut [u8]) -> Result<Option<usize>, ScanError> {
        self.source.wide_to_ansi(i, o)
    }
    fn wide_to_local_prefix(&self, i: &[u32], o: &mut [u8]) -> Result<Option<usize>, ScanError> {
        self.source.wide_to_local_prefix(i, o)
    }
}
fn profile(name: &str) -> Option<(u8, usize)> {
    let extension = name.rsplit('.').next()?.to_ascii_lowercase();
    match extension.as_str() {
        "txt" | "md" | "log" | "json" | "csv" | "tsv" | "toml" | "yaml" | "yml" | "ini" | "xml"
        | "rs" | "c" | "cpp" | "h" | "py" => Some((1, TEXT_BYTES)),
        "png" => Some((2, IMAGE_BYTES)),
        "jpg" | "jpeg" | "jfif" | "jpe" => Some((3, IMAGE_BYTES)),
        "gif" => Some((4, IMAGE_BYTES)),
        "bmp" => Some((5, IMAGE_BYTES)),
        "ico" | "cur" => Some((6, IMAGE_BYTES)),
        "webp" => Some((7, IMAGE_BYTES)),
        "tif" | "tiff" => Some((8, IMAGE_BYTES)),
        "tga" => Some((9, IMAGE_BYTES)),
        "icns" => Some((10, IMAGE_BYTES)),
        "svg" => Some((11, IMAGE_BYTES)),
        "heic" | "heif" if cfg!(target_os = "macos") => Some((12, IMAGE_BYTES)),
        _ => None,
    }
}
fn text(bytes: &[u8]) -> Result<Vec<u8>, &'static str> {
    let text = if bytes.starts_with(&[0xff, 0xfe]) || bytes.starts_with(&[0xfe, 0xff]) {
        if !bytes.len().is_multiple_of(2) {
            return Err("preview_encoding");
        }
        let little = bytes[0] == 0xff;
        let units = bytes[2..]
            .as_chunks::<2>()
            .0
            .iter()
            .map(|pair| {
                if little {
                    u16::from_le_bytes([pair[0], pair[1]])
                } else {
                    u16::from_be_bytes([pair[0], pair[1]])
                }
            })
            .collect::<Vec<_>>();
        String::from_utf16(&units).map_err(|_| "preview_encoding")?
    } else {
        std::str::from_utf8(bytes.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(bytes))
            .map_err(|_| "preview_encoding")?
            .to_owned()
    };
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
    {
        return Err("preview_encoding");
    }
    if text.len() > TEXT_BYTES {
        return Err("preview_limit");
    }
    Ok(text.into_bytes())
}
fn image_dimensions(kind: u8, bytes: &[u8]) -> Result<(), &'static str> {
    if kind > 3 {
        let valid = match kind {
            4 => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
            5 => bytes.starts_with(b"BM"),
            6 => bytes.starts_with(&[0, 0, 1, 0]) || bytes.starts_with(&[0, 0, 2, 0]),
            7 => bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WEBP"),
            8 => [b"II*\0", b"MM\0*", b"II+\0", b"MM\0+"]
                .iter()
                .any(|s| bytes.starts_with(*s)),
            9 => bytes.len() >= 18 && bytes[2] == 2,
            10 => bytes.starts_with(b"icns"),
            // XML structure, resources and complexity are checked before SVG rendering.
            11 => std::str::from_utf8(bytes).is_ok(),
            12 if cfg!(target_os = "macos") => {
                // HEIF admission never permits an AVIF/AVIS brand, even under a renamed suffix.
                if bytes.get(4..8) != Some(b"ftyp") || bytes.len() < 16 {
                    false
                } else {
                    let length = u32::from_be_bytes(bytes[..4].try_into().unwrap()) as usize;
                    length >= 16
                        && length <= bytes.len()
                        && length <= 4096
                        && (length - 16).is_multiple_of(4)
                        && !bytes[8..length]
                            .windows(4)
                            .any(|b| matches!(b, b"avif" | b"avis"))
                        && std::iter::once(&bytes[8..12])
                            .chain(
                                bytes[16..length]
                                    .as_chunks::<4>()
                                    .0
                                    .iter()
                                    .map(|b| b.as_slice()),
                            )
                            .any(|b| {
                                matches!(
                                    b,
                                    b"heic" | b"heix" | b"hevc" | b"hevx" | b"mif1" | b"msf1"
                                )
                            })
                }
            }
            _ => false,
        };
        return if valid {
            Ok(())
        } else {
            Err("preview_unsupported")
        };
    }
    let (width, height) = if kind == 2 {
        if bytes.len() < 33 || !bytes.starts_with(b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR") {
            return Err("preview_unsupported");
        }
        (
            u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
            u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
        )
    } else {
        if !bytes.starts_with(b"\xff\xd8") {
            return Err("preview_unsupported");
        }
        let mut at = 2;
        let mut size = None;
        while at < bytes.len() {
            if bytes[at] != 0xff {
                return Err("preview_unsupported");
            }
            while at < bytes.len() && bytes[at] == 0xff {
                at += 1;
            }
            let marker = *bytes.get(at).ok_or("preview_unsupported")?;
            at += 1;
            if matches!(marker, 0 | 0xd8..=0xda) {
                return Err("preview_unsupported");
            }
            if marker == 1 || (0xd0..=0xd7).contains(&marker) {
                continue;
            }
            let length = bytes
                .get(at..at + 2)
                .map(|b| u16::from_be_bytes([b[0], b[1]]) as usize)
                .ok_or("preview_unsupported")?;
            if length < 2 || at.checked_add(length).is_none_or(|end| end > bytes.len()) {
                return Err("preview_unsupported");
            }
            if matches!(marker, 0xc0 | 0xc2) {
                if length < 8 || bytes[at + 2] != 8 {
                    return Err("preview_unsupported");
                }
                size = Some((
                    u32::from(u16::from_be_bytes([bytes[at + 5], bytes[at + 6]])),
                    u32::from(u16::from_be_bytes([bytes[at + 3], bytes[at + 4]])),
                ));
                break;
            }
            at += length;
        }
        size.ok_or("preview_unsupported")?
    };
    if width == 0
        || height == 0
        || width > 16384
        || height > 16384
        || u64::from(width) * u64::from(height) > 33_554_432
    {
        return Err("preview_limit");
    }
    Ok(())
}
impl Archive {
    /// Decode one supported regular file completely and verify authentication/integrity
    /// before returning bytes. Failure/cancellation drops private partial plaintext.
    pub fn preview(
        &self,
        id: u64,
        password: Option<&Password>,
        control: &Cancellation,
    ) -> Result<Content, &'static str> {
        control.check().map_err(|_| "cancelled")?;
        let entry = self.entry(id).map_err(|_| "source_changed")?;
        if entry.directory
            || entry.link
            || entry.warning
            || !entry.size_known
            || !entry.payload_supported()
        {
            return Err("preview_unsupported");
        }
        let (kind, limit) = profile(&entry.display_name).ok_or("preview_unsupported")?;
        if entry.size > limit as u64 {
            return Err("preview_limit");
        }
        let mut source = self.source.fork().map_err(|_| "source_changed")?;
        let mut bounded = Source {
            source: source.file(),
            remaining: 256 * 1024 * 1024,
            control,
        };
        let secret = password.or(self.password.as_ref()).map(Password::expose);
        let mut decoder = if self.disk.is_some() {
            xdzip_codec_zip::ZipCodec.open_selected_payload(
                &mut bounded,
                std::slice::from_ref(&entry),
                control,
            )
        } else {
            crate::formats::codec(self.format).open_payload_with_password(
                &mut bounded,
                &self.rows,
                control,
                secret,
            )
        }
        .map_err(crate::task_interaction::decoding)?;
        let mut sink = Sink {
            bytes: zeroize::Zeroizing::new(Vec::new()),
            limit,
            work: 0,
            auxiliary: 0,
        };
        decoder
            .decode_with_password(&entry, &mut sink, control, secret)
            .map_err(crate::task_interaction::decoding)?;
        drop(decoder);
        control.check().map_err(|_| "cancelled")?;
        self.source.validate().map_err(|_| "source_changed")?;
        let bytes = if kind == 1 {
            text(&sink.bytes)?
        } else {
            image_dimensions(kind, &sink.bytes)?;
            sink.bytes.to_vec()
        };
        Ok(Content {
            session: self.key.session.get(),
            entry: id,
            kind,
            bytes,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fixed_image_kinds_reject_foreign_formats_and_gate_heif_by_platform() {
        for (name, kind) in [
            ("owned.jfif", 3),
            ("owned.jpe", 3),
            ("owned.GIF", 4),
            ("owned.bmp", 5),
            ("owned.cur", 6),
            ("owned.webp", 7),
            ("owned.tif", 8),
            ("owned.tga", 9),
            ("owned.icns", 10),
            ("owned.svg", 11),
        ] {
            assert_eq!(profile(name), Some((kind, IMAGE_BYTES)));
        }
        for extension in ["avif", "raw", "psd", "exr", "pdf", "svgz"] {
            assert!(profile(&format!("owned.{extension}")).is_none());
        }
        assert_eq!(profile("owned.heif").is_some(), cfg!(target_os = "macos"));
        assert!(image_dimensions(4, b"GIF89a").is_ok());
        assert!(image_dimensions(4, b"RIFF1234WEBP").is_err());
        assert!(image_dimensions(7, b"RIFF1234WEBP").is_ok());
        assert!(image_dimensions(7, b"GIF89a").is_err());
        let mut heif = b"\0\0\0\x18ftypheic\0\0\0\0mif1heic".to_vec();
        assert_eq!(
            image_dimensions(12, &heif).is_ok(),
            cfg!(target_os = "macos")
        );
        heif[20..24].copy_from_slice(b"avif");
        assert!(image_dimensions(12, &heif).is_err());
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&4032u32.to_be_bytes());
        png.extend_from_slice(&3024u32.to_be_bytes());
        png.extend([0; 9]);
        assert!(image_dimensions(2, &png).is_ok());
    }
    #[test]
    fn text_is_strict_plain_data_and_limits_include_skipped_work() {
        assert_eq!(
            text(b"<script>example</script>").unwrap(),
            b"<script>example</script>"
        );
        assert_eq!(text(&[0xff, 0xfe, 0x2d, 0x4e]).unwrap(), "中".as_bytes());
        assert!(text(&[0xff]).is_err());
        assert!(text(b"binary\0").is_err());
        assert!(profile("archive.html").is_none());
        assert_eq!(profile("image.svg"), Some((11, IMAGE_BYTES)));
        let mut sink = Sink {
            bytes: zeroize::Zeroizing::new(Vec::new()),
            limit: 3,
            work: 0,
            auxiliary: 0,
        };
        assert_eq!(sink.write_chunk(b"four"), Err(DecodeError::Limit));
        assert!(sink.bytes.is_empty());
        assert_eq!(sink.charge_skipped(WORK_BYTES + 1), Err(DecodeError::Limit));
    }
    #[test]
    fn actual_preview_checks_physical_identity_password_integrity_and_cancellation() {
        let control = Cancellation::default();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/zip/normal.zip");
        let archive = Archive::open(&path, &control).unwrap();
        let content = archive.preview(2, None, &control).unwrap();
        assert_eq!(content.entry, 2);
        assert_eq!(content.kind, 1);
        assert!(!content.bytes.is_empty());
        assert!(archive.preview(1, None, &control).is_err());
        control.cancel();
        assert_eq!(archive.preview(2, None, &control).unwrap_err(), "cancelled");
    }
    #[test]
    fn encrypted_preview_authenticates_before_releasing_any_plaintext() {
        let root = std::env::temp_dir().join(format!(
            "xdzip-preview-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&root).unwrap();
        let path = root.join("owned.zip");
        let mut bytes = crate::extraction::password_tests::archive_bytes(
            &[(
                "note.txt",
                b"<h1>plain text</h1>",
                Some("owned-preview-password"),
                false,
            )],
            0,
        );
        std::fs::write(&path, &bytes).unwrap();
        let control = Cancellation::default();
        let archive = Archive::open(&path, &control).unwrap();
        assert_eq!(
            archive.preview(1, None, &control).unwrap_err(),
            "password_required"
        );
        let wrong = Password::new("different".into()).unwrap();
        assert_eq!(
            archive.preview(1, Some(&wrong), &control).unwrap_err(),
            "wrong_password"
        );
        let correct = Password::new("owned-preview-password".into()).unwrap();
        assert_eq!(
            archive.preview(1, Some(&correct), &control).unwrap().bytes,
            b"<h1>plain text</h1>"
        );
        drop(archive);
        let central = bytes.windows(4).position(|b| b == b"PK\x01\x02").unwrap();
        bytes[central - 1] ^= 1;
        std::fs::write(&path, bytes).unwrap();
        let archive = Archive::open(&path, &control).unwrap();
        assert!(archive.preview(1, Some(&correct), &control).is_err());
        drop(archive);
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir(root).unwrap();
    }
    #[test]
    fn images_reject_oversize_headers_and_non_raster_formats() {
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend_from_slice(&16384u32.to_be_bytes());
        png.extend_from_slice(&16384u32.to_be_bytes());
        png.extend([0; 9]);
        assert_eq!(image_dimensions(2, &png), Err("preview_limit"));
        assert_eq!(image_dimensions(2, b"<svg>"), Err("preview_unsupported"));
        assert_eq!(
            image_dimensions(3, &[0xff, 0xd8, 0xff, 0xe0, 0xff, 0xff]),
            Err("preview_unsupported")
        );
    }
}
