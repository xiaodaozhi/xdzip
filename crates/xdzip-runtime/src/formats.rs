//! Built-in capability routing. The host retains all paths and source authority.
use std::io::SeekFrom;
use xdzip_codec_api::{
    Format, MetadataCodec, ReadSource, ScanError,
    encode::{CreationCodec, RewriteCodec},
    payload::PayloadCodec,
};
use xdzip_codec_tar::TarCodec;
use xdzip_codec_zip::ZipCodec;
pub(crate) trait Codec: MetadataCodec + PayloadCodec + CreationCodec + RewriteCodec {}
impl Codec for ZipCodec {}
impl Codec for TarCodec {}
pub(crate) fn codec(format: Format) -> &'static dyn Codec {
    match format {
        Format::Zip => &ZipCodec,
        Format::Tar => &TarCodec { gzip: false },
        Format::TarGzip => &TarCodec { gzip: true },
    }
}
pub(crate) fn detect(source: &mut dyn ReadSource) -> Result<Format, ScanError> {
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let mut magic = [0; 4];
    source
        .read_exact(&mut magic)
        .map_err(|_| ScanError::Invalid)?;
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    Ok(if magic[..2] == [0x1f, 0x8b] {
        Format::TarGzip
    } else if matches!(&magic, b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08") {
        Format::Zip
    } else {
        Format::Tar
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
