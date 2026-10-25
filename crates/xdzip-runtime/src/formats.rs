//! Built-in capability routing. The host retains all paths and source authority.
use std::io::SeekFrom;
use xdzip_codec_api::{
    Format, MetadataCodec, ReadSource, ScanError,
    encode::{CreationCodec, RewriteCodec},
    payload::PayloadCodec,
};
use xdzip_codec_native::NativeCodec;
use xdzip_codec_tar::TarCodec;
use xdzip_codec_zip::ZipCodec;
pub(crate) trait Codec: MetadataCodec + PayloadCodec + CreationCodec + RewriteCodec {}
impl Codec for ZipCodec {}
impl Codec for TarCodec {}
impl Codec for NativeCodec {}
pub(crate) fn codec(format: Format) -> &'static dyn Codec {
    match format {
        Format::Zip => &ZipCodec,
        Format::Tar => &TarCodec { gzip: false },
        Format::TarGzip => &TarCodec { gzip: true },
        Format::SevenZip => &NativeCodec {
            format: Format::SevenZip,
        },
        Format::Rar => &NativeCodec {
            format: Format::Rar,
        },
        Format::Cab => &NativeCodec {
            format: Format::Cab,
        },
        Format::Iso => &NativeCodec {
            format: Format::Iso,
        },
        Format::Udf => &NativeCodec {
            format: Format::Udf,
        },
    }
}
pub(crate) fn detect(source: &mut dyn ReadSource) -> Result<Format, ScanError> {
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let mut magic = [0; 8];
    source
        .read_exact(&mut magic)
        .map_err(|_| ScanError::Invalid)?;
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    Ok(if magic[..2] == [0x1f, 0x8b] {
        Format::TarGzip
    } else if matches!(&magic[..4], b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08") {
        Format::Zip
    } else if magic[..6] == *b"7z\xbc\xaf\x27\x1c" {
        Format::SevenZip
    } else if magic[..4] == *b"RE~^"
        || magic[..7] == *b"Rar!\x1a\x07\x00"
        || magic == *b"Rar!\x1a\x07\x01\x00"
    {
        Format::Rar
    } else if magic[..4] == *b"MSCF" {
        Format::Cab
    } else {
        detect_optical(source)?.unwrap_or(Format::Tar)
    })
}

/// Optical descriptors start at sector 16. Bound discovery independently from
/// the decoder, and retain UDF admission even when its metadata is damaged: a
/// hybrid's ISO tree must never hide a failed UDF scan.
fn detect_optical(source: &mut dyn ReadSource) -> Result<Option<Format>, ScanError> {
    const SECTOR: u64 = 2048;
    let length = source.seek(SeekFrom::End(0)).map_err(|_| ScanError::Io)?;
    let mut iso = false;
    let mut udf = false;
    let mut sector = [0; SECTOR as usize];
    for at in 16..256u64.min(length / SECTOR) {
        source
            .seek(SeekFrom::Start(at * SECTOR))
            .map_err(|_| ScanError::Io)?;
        source.read_exact(&mut sector).map_err(|_| ScanError::Io)?;
        iso |= matches!(sector[0], 0..=3 | 255) && &sector[1..6] == b"CD001";
        udf |= matches!(&sector[1..6], b"BEA01" | b"NSR02" | b"NSR03" | b"TEA01");
    }
    // UDF anchors also identify images with a damaged volume recognition
    // sequence. The engine subsequently validates tags, descriptors and ranges.
    for at in [
        256,
        length / SECTOR - u64::from(length >= SECTOR),
        (length / SECTOR).saturating_sub(257),
    ] {
        if at >= 256 && (at + 1).saturating_mul(SECTOR) <= length {
            source
                .seek(SeekFrom::Start(at * SECTOR))
                .map_err(|_| ScanError::Io)?;
            source.read_exact(&mut sector).map_err(|_| ScanError::Io)?;
            udf |= sector[..2] == [2, 0] && matches!(sector[2..4], [2, 0] | [3, 0]);
        }
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    Ok(if udf {
        Some(Format::Udf)
    } else if iso {
        Some(Format::Iso)
    } else {
        None
    })
}
pub(crate) fn creation_format(name: &str) -> Option<Format> {
    let name = name.to_ascii_lowercase();
    if name.ends_with(".tar.gz") {
        Some(Format::TarGzip)
    } else if name.ends_with(".tar") {
        Some(Format::Tar)
    } else if name.ends_with(".zip") {
        Some(Format::Zip)
    } else if name.ends_with(".7z") {
        Some(Format::SevenZip)
    } else {
        None
    }
}

/// Sequential containers meter decoded and encoded work, including skipped payloads.
pub(crate) fn limits(format: Format) -> xdzip_codec_api::ScanLimits {
    xdzip_codec_api::ScanLimits {
        read_bytes: if format == Format::Zip {
            64 * 1024 * 1024
        } else {
            24 * 1024 * 1024 * 1024
        },
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_signatures_route_without_filename_hints() {
        for (bytes, expected) in [
            (b"7z\xbc\xaf\x27\x1c00".as_slice(), Format::SevenZip),
            (b"Rar!\x1a\x07\x00x".as_slice(), Format::Rar),
            (b"RE~^0000".as_slice(), Format::Rar),
            (b"Rar!\x1a\x07\x01\x00".as_slice(), Format::Rar),
            (b"PK\x03\x040000".as_slice(), Format::Zip),
        ] {
            assert_eq!(detect(&mut std::io::Cursor::new(bytes)).unwrap(), expected);
        }
        assert_eq!(creation_format("archive.7Z"), Some(Format::SevenZip));
        assert_eq!(creation_format("archive.rar"), None);
        for name in ["archive.cab", "archive.iso", "archive.udf"] {
            assert_eq!(creation_format(name), None);
        }
    }
    #[test]
    fn optical_routing_uses_descriptors_and_keeps_damaged_udf_hybrids() {
        use std::io::Read;
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/cab-iso");
        for (name, format) in [
            ("iso9660", Format::Iso),
            ("joliet", Format::Iso),
            ("rock-ridge", Format::Iso),
            ("udf", Format::Udf),
            ("hybrid", Format::Udf),
        ] {
            let mut bytes = Vec::new();
            flate2::read::GzDecoder::new(
                std::fs::File::open(root.join(format!("{name}.iso.gz"))).unwrap(),
            )
            .read_to_end(&mut bytes)
            .unwrap();
            assert_eq!(
                detect(&mut std::io::Cursor::new(&bytes)),
                Ok(format),
                "{name}"
            );
            if format == Format::Udf {
                for sector in bytes.chunks_mut(2048) {
                    if sector
                        .get(1..6)
                        .is_some_and(|magic| matches!(magic, b"NSR02" | b"NSR03"))
                    {
                        sector[1..6].fill(0);
                    }
                }
                assert_eq!(detect(&mut std::io::Cursor::new(&bytes)), Ok(Format::Udf));
            }
        }
    }
    #[test]
    fn optical_magic_search_is_bounded_for_large_unknown_sources() {
        struct Probe {
            position: u64,
            read: usize,
        }
        impl std::io::Read for Probe {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                bytes.fill(0);
                self.read += bytes.len();
                self.position += bytes.len() as u64;
                Ok(bytes.len())
            }
        }
        impl std::io::Seek for Probe {
            fn seek(&mut self, position: SeekFrom) -> std::io::Result<u64> {
                self.position = match position {
                    SeekFrom::Start(at) => at,
                    SeekFrom::End(0) => 10 * 1024 * 1024 * 1024,
                    _ => return Err(std::io::ErrorKind::InvalidInput.into()),
                };
                Ok(self.position)
            }
        }
        impl ReadSource for Probe {}
        let mut source = Probe {
            position: 0,
            read: 0,
        };
        assert_eq!(detect(&mut source), Ok(Format::Tar));
        assert!(source.read < 512 * 1024);
        assert_eq!(source.position, 0);
    }
}
