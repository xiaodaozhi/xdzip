//! Built-in capability routing. The host retains all paths and source authority.
use std::io::{Read, SeekFrom};
use xdzip_codec_aescrypt::AesCryptCodec;
use xdzip_codec_alz::AlzCodec;
use xdzip_codec_api::{
    Format, MetadataCodec, ReadSource, ScanError,
    encode::{CreationCodec, RewriteCodec},
    payload::PayloadCodec,
};
use xdzip_codec_arc::ArcCodec;
use xdzip_codec_arj::ArjCodec;
use xdzip_codec_asar::AsarCodec;
use xdzip_codec_bh::BhCodec;
use xdzip_codec_deb::DebCodec;
use xdzip_codec_lha::{LhaCodec, PmaCodec};
use xdzip_codec_native::NativeCodec;
use xdzip_codec_stream::{
    StreamCodec,
    controlled::{ControlledStream, Filter},
};
use xdzip_codec_tar::TarCodec;
use xdzip_codec_uu::UuCodec;
use xdzip_codec_zip::ZipCodec;
pub(crate) trait Codec: MetadataCodec + PayloadCodec + CreationCodec + RewriteCodec {}
impl Codec for ZipCodec {}
impl Codec for TarCodec {}
impl Codec for NativeCodec {}
impl Codec for StreamCodec {}
impl Codec for DebCodec {}
impl Codec for AsarCodec {}
impl Codec for LhaCodec {}
impl Codec for PmaCodec {}
impl Codec for ArjCodec {}
impl Codec for ArcCodec {}
impl Codec for BhCodec {}
impl Codec for AlzCodec {}
impl Codec for UuCodec {}
impl Codec for AesCryptCodec {}
pub(crate) fn codec(format: Format) -> &'static dyn Codec {
    match format {
        Format::Zip => &ZipCodec,
        Format::Deb => &DebCodec,
        Format::Asar => &AsarCodec,
        Format::Lha => &LhaCodec,
        Format::Pma => &PmaCodec,
        Format::Arj => &ArjCodec,
        Format::Arc => &ArcCodec,
        Format::Bh => &BhCodec,
        Format::Alz => &AlzCodec,
        Format::AesCrypt => &AesCryptCodec,
        Format::Brotli => &StreamCodec {
            format: Format::Brotli,
        },
        Format::TarBrotli => &TarCodec {
            filter: Filter::Brotli,
        },
        Format::Compress => &StreamCodec {
            format: Format::Compress,
        },
        Format::TarCompress => &TarCodec {
            filter: Filter::Compress,
        },
        Format::Uu => &UuCodec { format: Format::Uu },
        Format::Xxe => &UuCodec {
            format: Format::Xxe,
        },
        Format::Tar => &TarCodec {
            filter: Filter::Identity,
        },
        Format::TarGzip => &TarCodec {
            filter: Filter::Gzip,
        },
        Format::Gzip => &StreamCodec {
            format: Format::Gzip,
        },
        Format::Xz => &StreamCodec { format: Format::Xz },
        Format::TarXz => &TarCodec { filter: Filter::Xz },
        Format::Lz4 => &StreamCodec {
            format: Format::Lz4,
        },
        Format::TarLz4 => &TarCodec {
            filter: Filter::Lz4,
        },
        Format::Zstd => &StreamCodec {
            format: Format::Zstd,
        },
        Format::TarZstd => &TarCodec {
            filter: Filter::Zstd,
        },
        Format::Bzip2 => &StreamCodec {
            format: Format::Bzip2,
        },
        Format::Lzma => &StreamCodec {
            format: Format::Lzma,
        },
        Format::Lzip => &StreamCodec {
            format: Format::Lzip,
        },
        Format::TarBzip2 => &TarCodec {
            filter: Filter::Bzip2,
        },
        Format::TarLzma => &TarCodec {
            filter: Filter::Lzma,
        },
        Format::TarLzip => &TarCodec {
            filter: Filter::Lzip,
        },
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
pub(crate) fn scan(
    format: &mut Format,
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
    password: Option<&str>,
) -> Result<Vec<xdzip_codec_api::Entry>, ScanError> {
    if format.descriptor().container == "single-file" {
        let stream = StreamCodec { format: *format };
        let info = stream.inspect(source, control, limits(*format))?;
        let ambiguous = info.all_zero && info.entry.size >= 1024;
        let ordinary_hint = ambiguous
            && source
                .source_name_hint()
                .and_then(|name| xdzip_codec_api::registry::compressed_view_hint(name, *format))
                == Some(*format);
        if ambiguous && !ordinary_hint {
            let tar = TarCodec {
                filter: stream.filter()?,
            };
            *format = tar.format();
            tar.scan(source, control, limits(*format))
        } else {
            Ok(vec![info.entry])
        }
    } else {
        codec(*format).scan_with_password(source, control, limits(*format), password)
    }
}
pub(crate) fn detect(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Format, ScanError> {
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let mut magic = [0; 8];
    let mut length = 0;
    while length != magic.len() {
        control.check()?;
        match source.read(&mut magic[length..]) {
            Ok(0) => break,
            Ok(count) if count <= magic.len() - length => length += count,
            Ok(_) => return Err(ScanError::Io),
            Err(error)
                if error.kind() == std::io::ErrorKind::Interrupted && error.get_ref().is_none() =>
            {
                continue;
            }
            Err(error) => return Err(ScanError::from_source_error(&error)),
        }
    }
    if length == 0 {
        return Err(ScanError::Invalid);
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    // A level-0 family's length/checksum can coincide with either two-byte magic.
    // Its '-' cannot be admitted GZIP method 8 or valid Unix compress flags.
    let lha_family =
        length >= 7 && matches!(&magic[2..5], b"-lh" | b"-lz" | b"-pm") && magic[6] == b'-';
    Ok(if magic[..3] == *b"AES" {
        if recognized_tar(source, control)? {
            match AesCryptCodec::probe_header(source, control) {
                Ok(()) => Format::AesCrypt,
                Err(ScanError::Invalid | ScanError::Unsupported) => Format::Tar,
                Err(error) => return Err(error),
            }
        } else {
            Format::AesCrypt
        }
    } else if magic[..2] == [0x1f, 0x9d] && !lha_family {
        compressed_format(source, control, Filter::Compress, Format::Compress)?
    } else if magic[..2] == [0x1f, 0x8b] && !lha_family {
        compressed_format(source, control, Filter::Gzip, Format::Gzip)?
    } else if magic[..6] == *b"\xfd7zXZ\0" {
        compressed_format(source, control, Filter::Xz, Format::Xz)?
    } else if matches!(
        u32::from_le_bytes(magic[..4].try_into().expect("fixed slice")),
        0x184d2204 | 0x184c2102 | 0xfd2fb528 | 0x184d2a50..=0x184d2a5f
    ) {
        match framed_magic(source, control)? {
            0x184d2204 => compressed_format(source, control, Filter::Lz4, Format::Lz4)?,
            0xfd2fb528 => compressed_format(source, control, Filter::Zstd, Format::Zstd)?,
            _ => return Err(ScanError::Unsupported),
        }
    } else if magic[..3] == *b"BZh" {
        compressed_format(source, control, Filter::Bzip2, Format::Bzip2)?
    } else if magic[..4] == *b"LZIP" {
        compressed_format(source, control, Filter::Lzip, Format::Lzip)?
    } else if matches!(&magic[..4], b"PK\x03\x04" | b"PK\x05\x06" | b"PK\x07\x08") {
        Format::Zip
    } else if magic[..6] == *b"7z\xbc\xaf\x27\x1c" {
        Format::SevenZip
    } else if magic[..4] == *b"RE~^"
        || magic[..7] == *b"Rar!\x1a\x07\x00"
        || magic == *b"Rar!\x1a\x07\x01\x00"
    {
        Format::Rar
    } else if magic[..4] == [4, 0, 0, 0] {
        let mut header = [0; 16];
        source.read_exact(&mut header).map_err(|error| {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                ScanError::Invalid
            } else {
                ScanError::from_source_error(&error)
            }
        })?;
        source
            .seek(SeekFrom::Start(0))
            .map_err(|error| ScanError::from_source_error(&error))?;
        if xdzip_codec_asar::index::recognizes_header(&header) {
            Format::Asar
        } else {
            detect_optical(source)?.unwrap_or(Format::Tar)
        }
    } else if magic == *b"!<arch>\n" || magic == *b"!<thin>\n" {
        Format::Deb
    } else if magic[..4] == *b"MSCF" {
        Format::Cab
    } else if magic[..4] == [0x41, 0x4c, 0x5a, 0x01] {
        if AlzCodec::candidate(source, control)? {
            Format::Alz
        } else {
            return Err(ScanError::Invalid);
        }
    } else if let Some(format) = detect_optical(source)? {
        format
    } else if !recognized_tar(source, control)? && xdzip_codec_uu::candidate(source, control)? {
        UuCodec::detect(source, control)?
    } else {
        Format::Tar
    })
}

/// LZ4 and Zstandard share skippable magic. Locate a data-frame signature
/// inside an independently bounded prefix without granting another source.
fn framed_magic(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<u32, ScanError> {
    let length = source.seek(SeekFrom::End(0)).map_err(|_| ScanError::Io)?;
    let mut position = 0u64;
    for _ in 0..100_000 {
        control.check()?;
        if position.checked_add(8).is_none_or(|end| end > length) {
            return Err(ScanError::Invalid);
        }
        if position > 8 * 1024 * 1024 - 8 {
            return Err(ScanError::Limit);
        }
        source
            .seek(SeekFrom::Start(position))
            .map_err(|_| ScanError::Io)?;
        let mut header = [0; 8];
        source.read_exact(&mut header).map_err(|error| {
            if error.kind() == std::io::ErrorKind::UnexpectedEof {
                ScanError::Invalid
            } else {
                ScanError::Io
            }
        })?;
        let magic = u32::from_le_bytes(header[..4].try_into().expect("fixed slice"));
        if !(0x184d2a50..=0x184d2a5f).contains(&magic) {
            source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
            return Ok(magic);
        }
        position = position
            .checked_add(
                8 + u64::from(u32::from_le_bytes(
                    header[4..].try_into().expect("fixed slice"),
                )),
            )
            .ok_or(ScanError::Invalid)?;
    }
    Err(ScanError::Limit)
}

fn compressed_format(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
    filter: Filter,
    ordinary: Format,
) -> Result<Format, ScanError> {
    let input_limit = if filter == Filter::Brotli {
        24 * 1024 * 1024 * 1024
    } else if matches!(filter, Filter::Lz4 | Filter::Zstd) {
        8 * 1024 * 1024
    } else if filter == Filter::Bzip2 {
        2 * 1024 * 1024
    } else {
        256 * 1024
    };
    let mut stream = ControlledStream::probe(source, control, filter, input_limit, 512)?;
    let mut header = [0; 512];
    let mut length = 0;
    while length != header.len() {
        let n = stream
            .read(&mut header[length..])
            .map_err(|_| stream.failure().unwrap_or(ScanError::Invalid))?;
        if n == 0 {
            break;
        }
        length += n;
    }
    drop(stream);
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    Ok(if xdzip_codec_tar::recognizes_header(&header[..length]) {
        TarCodec { filter }.format()
    } else {
        ordinary
    })
}
/// Weak LZMA Alone detection follows failed TAR/ZIP-SFX discovery, never precedes them.
pub(crate) fn detect_lzma(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<Format>, ScanError> {
    control.check()?;
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let mut header = [0; 13];
    match source.read_exact(&mut header) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(_) => return Err(ScanError::Io),
    }
    let size = u64::from_le_bytes(header[5..].try_into().expect("fixed slice"));
    if header[0] > 224 || (size != u64::MAX && size >= 256 * 1024 * 1024 * 1024) {
        return Ok(None);
    }
    compressed_format(source, control, Filter::Lzma, Format::Lzma).map(Some)
}
/// Headerless streams are candidates until complete metadata admission succeeds.
/// Strong containers and ZIP/SFX discovery have already failed without identifying TAR.
/// One ledger covers all weak probes and their repeated reads; terminal host failures
/// and quotas are authoritative. Invalid/unsupported guesses publish no format or rows.
pub(crate) fn discover_weak(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<(Format, Vec<xdzip_codec_api::Entry>)>, ScanError> {
    let mut input = WeakInput {
        source,
        control,
        remaining: 24 * 1024 * 1024 * 1024,
    };
    if let Some(found) = validate_arj(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_arc(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_bh(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_alz(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_lha(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_brotli(&mut input, control)? {
        return Ok(Some(found));
    }
    let Some(mut candidate) = detect_lzma(&mut input, control)? else {
        return Ok(None);
    };
    let rows = scan(&mut candidate, &mut input, control, None)?;
    Ok(Some((candidate, rows)))
}
/// Disambiguate an unvalidated raw TAR marker with complete ARJ, LHA or Brotli admission.
/// A malformed TAR reached through decoded Brotli is never exposed as an ordinary file.
pub(crate) fn discover_ambiguous_tar(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<(Format, Vec<xdzip_codec_api::Entry>)>, ScanError> {
    let mut input = WeakInput {
        source,
        control,
        remaining: 24 * 1024 * 1024 * 1024,
    };
    if let Some(found) = validate_arj(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_arc(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_bh(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_alz(&mut input, control)? {
        return Ok(Some(found));
    }
    if let Some(found) = validate_lha(&mut input, control)? {
        return Ok(Some(found));
    }
    validate_brotli(&mut input, control)
}
fn validate_arc(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<(Format, Vec<xdzip_codec_api::Entry>)>, ScanError> {
    if !ArcCodec::candidate(source, control)? {
        return Ok(None);
    }
    match ArcCodec.scan(source, control, limits(Format::Arc)) {
        Ok(rows) => Ok(Some((Format::Arc, rows))),
        Err(ScanError::Invalid) => Ok(None),
        Err(error) => Err(error),
    }
}
fn validate_bh(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<(Format, Vec<xdzip_codec_api::Entry>)>, ScanError> {
    if !BhCodec::candidate(source, control)? {
        return Ok(None);
    }
    match BhCodec.scan(source, control, limits(Format::Bh)) {
        Ok(rows) => Ok(Some((Format::Bh, rows))),
        Err(ScanError::Invalid) => Ok(None),
        Err(error) => Err(error),
    }
}
fn validate_alz(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<(Format, Vec<xdzip_codec_api::Entry>)>, ScanError> {
    if !AlzCodec::candidate(source, control)? {
        return Ok(None);
    }
    match AlzCodec.scan(source, control, limits(Format::Alz)) {
        Ok(rows) => Ok(Some((Format::Alz, rows))),
        Err(ScanError::Invalid) => Ok(None),
        Err(error) => Err(error),
    }
}
fn validate_arj(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<(Format, Vec<xdzip_codec_api::Entry>)>, ScanError> {
    if !ArjCodec::candidate(source, control)? {
        return Ok(None);
    }
    match ArjCodec.scan(source, control, limits(Format::Arj)) {
        Ok(rows) => Ok(Some((Format::Arj, rows))),
        Err(ScanError::Invalid) => Ok(None),
        Err(error) => Err(error),
    }
}
fn validate_lha(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<(Format, Vec<xdzip_codec_api::Entry>)>, ScanError> {
    for format in [Format::Lha, Format::Pma] {
        let candidate = if format == Format::Pma {
            PmaCodec::candidate(source, control)?
        } else {
            LhaCodec::candidate(source, control)?
        };
        if !candidate {
            continue;
        }
        match codec(format).scan(source, control, limits(format)) {
            Ok(rows) => return Ok(Some((format, rows))),
            Err(ScanError::Invalid) => {}
            Err(error) => return Err(error),
        }
    }
    Ok(None)
}
fn validate_brotli(
    input: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<Option<(Format, Vec<xdzip_codec_api::Entry>)>, ScanError> {
    match compressed_format(input, control, Filter::Brotli, Format::Brotli) {
        Ok(mut candidate) => match scan(&mut candidate, input, control, None) {
            Ok(rows) => return Ok(Some((candidate, rows))),
            Err(ScanError::Invalid | ScanError::Unsupported) if candidate != Format::TarBrotli => {}
            Err(error) => return Err(error),
        },
        Err(ScanError::Invalid | ScanError::Unsupported) => {}
        Err(error) => return Err(error),
    }
    Ok(None)
}
struct WeakInput<'a> {
    source: &'a mut dyn ReadSource,
    control: &'a xdzip_codec_api::Cancellation,
    remaining: u64,
}
impl Read for WeakInput<'_> {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        loop {
            self.control.check().map_err(std::io::Error::other)?;
            if bytes.is_empty() {
                return Ok(0);
            }
            if self.remaining == 0 {
                return Err(std::io::Error::other(ScanError::Limit));
            }
            let take = bytes
                .len()
                .min(self.remaining.min(usize::MAX as u64) as usize);
            match self.source.read(&mut bytes[..take]) {
                Err(error)
                    if error.kind() == std::io::ErrorKind::Interrupted
                        && error.get_ref().is_none() =>
                {
                    continue;
                }
                Err(error) => return Err(error),
                Ok(n) if n <= take => {
                    self.remaining -= n as u64;
                    return Ok(n);
                }
                Ok(_) => return Err(std::io::Error::other(ScanError::Io)),
            }
        }
    }
}
impl std::io::Seek for WeakInput<'_> {
    fn seek(&mut self, at: SeekFrom) -> std::io::Result<u64> {
        self.control.check().map_err(std::io::Error::other)?;
        self.source.seek(at)
    }
}
impl ReadSource for WeakInput<'_> {
    fn source_name_hint(&self) -> Option<&str> {
        self.source.source_name_hint()
    }
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
    fn oem_to_ansi_wide(
        &self,
        input: &[u16],
        output: &mut [u16],
    ) -> Result<Option<usize>, ScanError> {
        self.source.oem_to_ansi_wide(input, output)
    }
    fn wide_to_ansi(&self, input: &[u16], output: &mut [u8]) -> Result<Option<usize>, ScanError> {
        self.source.wide_to_ansi(input, output)
    }
    fn wide_to_local_prefix(
        &self,
        input: &[u32],
        output: &mut [u8],
    ) -> Result<Option<usize>, ScanError> {
        self.source.wide_to_local_prefix(input, output)
    }
}
pub(crate) fn recognized_tar(
    source: &mut dyn ReadSource,
    control: &xdzip_codec_api::Cancellation,
) -> Result<bool, ScanError> {
    control.check()?;
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let mut header = [0; 512];
    let mut length = 0;
    while length != header.len() {
        control.check()?;
        let n = source
            .read(&mut header[length..])
            .map_err(|_| ScanError::Io)?;
        if n == 0 {
            break;
        }
        length += n;
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    Ok(xdzip_codec_tar::recognizes_header(&header[..length]))
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
    xdzip_codec_api::registry::creation_format(name)
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
    fn weak_stream_discovery_retains_one_exact_input_ledger_and_never_retries_host_failures() {
        use std::{cell::Cell, io::Cursor, rc::Rc};
        struct Source {
            data: Cursor<Vec<u8>>,
            reads: Rc<Cell<usize>>,
            error: Option<ScanError>,
        }
        impl Read for Source {
            fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
                self.reads.set(self.reads.get() + 1);
                if let Some(e) = self.error {
                    Err(std::io::Error::other(e))
                } else {
                    self.data.read(bytes)
                }
            }
        }
        impl std::io::Seek for Source {
            fn seek(&mut self, at: SeekFrom) -> std::io::Result<u64> {
                self.data.seek(at)
            }
        }
        impl ReadSource for Source {
            fn source_name_hint(&self) -> Option<&str> {
                Some("owned.br")
            }
        }
        let bytes = std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/brotli/raw.br"),
        )
        .unwrap();
        assert_eq!(bytes.len(), 5);
        let cancel = Default::default();
        for budget in [10, 9] {
            let mut source = Source {
                data: Cursor::new(bytes.clone()),
                reads: Rc::default(),
                error: None,
            };
            let mut input = WeakInput {
                source: &mut source,
                control: &cancel,
                remaining: budget,
            };
            let result = validate_brotli(&mut input, &cancel);
            if budget == 10 {
                let (format, rows) = result.unwrap().unwrap();
                assert_eq!(format, Format::Brotli);
                assert_eq!(rows[0].name_bytes(), b"owned");
            } else {
                assert!(matches!(result, Err(ScanError::Limit)));
            }
            assert_eq!(input.remaining, 0);
        }
        for error in [
            ScanError::Io,
            ScanError::Limit,
            ScanError::Cancelled,
            ScanError::SourceChanged,
        ] {
            let calls = Rc::new(Cell::new(0));
            let mut source = Source {
                data: Cursor::new(bytes.clone()),
                reads: Rc::clone(&calls),
                error: Some(error),
            };
            assert!(matches!(discover_weak(&mut source,&cancel),Err(e) if e==error));
            assert_eq!(calls.get(), 1);
        }
        let calls = Rc::new(Cell::new(0));
        let mut source = Source {
            data: Cursor::new(bytes),
            reads: Rc::clone(&calls),
            error: None,
        };
        cancel.cancel();
        assert!(matches!(
            discover_weak(&mut source, &cancel),
            Err(ScanError::Cancelled)
        ));
        assert_eq!(calls.get(), 0);
    }
    #[test]
    fn unnamed_zero_only_stream_retains_the_empty_tar_default() {
        use std::io::{Cursor, Write};
        let mut writer = flate2::write::GzEncoder::new(Vec::new(), Default::default());
        writer.write_all(&[0; 1024]).unwrap();
        let mut source = Cursor::new(writer.finish().unwrap());
        let control = Default::default();
        let mut format = detect(&mut source, &control).unwrap();
        assert!(
            scan(&mut format, &mut source, &control, None)
                .unwrap()
                .is_empty()
        );
        assert_eq!(format, Format::TarGzip);
    }
    #[test]
    fn shared_skippable_prefix_routes_each_frame_family_without_cross_format_concatenation() {
        use std::io::Cursor;
        let zstd = include_bytes!("../../../tests/fixtures/streams/phrase.zst");
        let lz4 = include_bytes!("../../../tests/fixtures/streams/phrase.lz4");
        let prefix = 0x184d2a50u32
            .to_le_bytes()
            .into_iter()
            .chain([0; 4])
            .collect::<Vec<_>>();
        let cancel = Default::default();
        for (data, format) in [
            (zstd.as_slice(), Format::Zstd),
            (lz4.as_slice(), Format::Lz4),
        ] {
            let mut source = Cursor::new([prefix.as_slice(), data].concat());
            assert_eq!(detect(&mut source, &cancel), Ok(format));
        }
        for data in [
            [zstd.as_slice(), lz4.as_slice()].concat(),
            [lz4.as_slice(), zstd.as_slice()].concat(),
        ] {
            let mut source = Cursor::new(data);
            match detect(&mut source, &cancel) {
                Ok(mut format) => assert!(scan(&mut format, &mut source, &cancel, None).is_err()),
                Err(error) => assert_eq!(error, ScanError::Invalid),
            }
        }
    }
    #[test]
    fn lz4_discovery_bounds_shared_skippable_prefix_and_refuses_legacy() {
        use std::io::Cursor;
        let frame = include_bytes!("../../../tests/fixtures/streams/phrase.lz4");
        let cancel = Default::default();
        assert_eq!(detect(&mut Cursor::new(frame), &cancel), Ok(Format::Lz4));
        let mut skipped = 0x184d2a5fu32.to_le_bytes().to_vec();
        skipped.extend(16u32.to_le_bytes());
        skipped.extend([0; 16]);
        skipped.extend(frame);
        assert_eq!(detect(&mut Cursor::new(&skipped), &cancel), Ok(Format::Lz4));
        for (bytes, error) in [
            (
                include_bytes!("../../../tests/fixtures/streams/legacy.lz4").as_slice(),
                ScanError::Unsupported,
            ),
            (
                include_bytes!("../../../tests/fixtures/streams/only-skip.lz4").as_slice(),
                ScanError::Invalid,
            ),
            (&skipped[..12], ScanError::Invalid),
        ] {
            assert_eq!(detect(&mut Cursor::new(bytes), &cancel), Err(error));
        }
        // A valid physical skip range still cannot move discovery beyond its prefix budget.
        let mut over = 0x184d2a50u32.to_le_bytes().to_vec();
        over.extend((8u32 * 1024 * 1024).to_le_bytes());
        over.resize(8 * 1024 * 1024 + 8, 0);
        over.extend(frame);
        assert_eq!(
            detect(&mut Cursor::new(over), &cancel),
            Err(ScanError::Limit)
        );
    }
    #[test]
    fn gzip_routing_preserves_tar_identity_and_rejects_damage() {
        use std::io::Write;
        fn gzip(bytes: &[u8]) -> Vec<u8> {
            let mut encoder =
                flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
            encoder.write_all(bytes).unwrap();
            encoder.finish().unwrap()
        }
        let cancel = Default::default();
        assert_eq!(
            detect(&mut std::io::Cursor::new(gzip(b"ordinary file")), &cancel),
            Ok(Format::Gzip)
        );
        let mut header = [0; 512];
        header[257..263].copy_from_slice(b"ustar\0");
        // Strong TAR identification stays authoritative even with a bad checksum.
        let mut source = std::io::Cursor::new(gzip(&header));
        assert_eq!(detect(&mut source, &cancel), Ok(Format::TarGzip));
        assert_eq!(
            codec(Format::TarGzip).scan(&mut source, &cancel, Default::default()),
            Err(ScanError::Invalid)
        );
        let mut damaged = gzip(b"ordinary file");
        let length = damaged.len();
        damaged[length - 8] ^= 1;
        assert_eq!(
            detect(&mut std::io::Cursor::new(damaged), &cancel),
            Err(ScanError::Invalid)
        );
    }
    #[test]
    fn native_signatures_route_without_filename_hints() {
        for (bytes, expected) in [
            (b"7z\xbc\xaf\x27\x1c00".as_slice(), Format::SevenZip),
            (b"Rar!\x1a\x07\x00x".as_slice(), Format::Rar),
            (b"RE~^0000".as_slice(), Format::Rar),
            (b"Rar!\x1a\x07\x01\x00".as_slice(), Format::Rar),
            (b"PK\x03\x040000".as_slice(), Format::Zip),
        ] {
            assert_eq!(
                detect(&mut std::io::Cursor::new(bytes), &Default::default()).unwrap(),
                expected
            );
        }
        assert_eq!(creation_format("archive.7Z"), Some(Format::SevenZip));
        assert_eq!(creation_format("archive.rar"), None);
        for name in ["archive.cab", "archive.iso", "archive.udf"] {
            assert_eq!(creation_format(name), None);
        }
    }
    #[test]
    fn optical_routing_uses_descriptors_and_keeps_damaged_udf_hybrids() {
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
                detect(&mut std::io::Cursor::new(&bytes), &Default::default()),
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
                assert_eq!(
                    detect(&mut std::io::Cursor::new(&bytes), &Default::default()),
                    Ok(Format::Udf)
                );
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
        assert_eq!(detect(&mut source, &Default::default()), Ok(Format::Tar));
        assert!(source.read < 512 * 1024);
        assert_eq!(source.position, 0);
    }
}
