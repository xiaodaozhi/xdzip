//! Independent 7z metadata/container candidate, with injected metadata decoding.
//! No paths, Qt, process launching or production routing are exposed.
//! Grammar: official DOC/7zFormat.txt and DOC/Methods.txt. The concise grammar
//! is supplemented by documented code-free reference facts and independent
//! official-writer/reader vectors; no container implementation was translated.
pub mod bcj2;
pub mod crypto;
mod decode;
pub(crate) mod encode;
mod files;
pub mod filters;
mod graph;
mod metadata;
mod payload;
pub(crate) mod rewrite;
mod source;
mod streams;
mod types;
mod wire;
pub use decode::{DecodeLimits, MethodGraphDecoder};
pub use metadata::CopyHeaderDecoder;
use source::{Budget, Reader};
pub use types::*;
use wire::Cursor;
use xdzip_codec_api::payload::{DecodeError, PayloadCodec, PayloadReader};
use xdzip_codec_api::{Cancellation, MetadataCodec, ReadSource, ScanError, ScanLimits};
use xdzip_compression::{Crc32, crc32};

fn checked_crc(bytes: &[u8], control: &Cancellation) -> Result<u32, ScanError> {
    let mut crc = Crc32::new();
    for chunk in bytes.chunks(32768) {
        control.check()?;
        crc.update(chunk);
    }
    control.check()?;
    Ok(crc.checksum())
}

fn encrypted_scan_error(error: ScanError, encrypted: bool) -> ScanError {
    if encrypted && error == ScanError::Invalid {
        ScanError::PasswordOrIntegrity
    } else {
        error
    }
}

pub(crate) fn encrypted_decode_error(error: DecodeError, encrypted: bool) -> DecodeError {
    if encrypted && error == DecodeError::Integrity {
        DecodeError::PasswordOrIntegrity
    } else {
        error
    }
}

/// Independent read/write codec with bounded stateful solid-folder streams.
/// The isolated worker supplies borrowed streams; the host owns publication.
pub struct SevenZipCodec;
impl MetadataCodec for SevenZipCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<xdzip_codec_api::Entry>, ScanError> {
        read_index(source, control, limits, None).map(|index| index.rows)
    }
    fn scan_with_password(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
        password: Option<&str>,
    ) -> Result<Vec<xdzip_codec_api::Entry>, ScanError> {
        read_index(source, control, limits, password).map(|index| index.rows)
    }
}
/// Parse a complete index with the bounded independent method graph decoder.
pub fn read_index(
    source: &mut dyn ReadSource,
    control: &Cancellation,
    limits: ScanLimits,
    password: Option<&str>,
) -> Result<Index, ScanError> {
    read_index_with_decoder(
        source,
        control,
        limits,
        password,
        &mut MethodGraphDecoder::default(),
    )
}
impl PayloadCodec for SevenZipCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[xdzip_codec_api::Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        self.open_payload_with_password(source, snapshot, control, None)
    }
    fn open_payload_with_password<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[xdzip_codec_api::Entry],
        control: &'a Cancellation,
        password: Option<&str>,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        Ok(Box::new(payload::Payload::open(
            source, snapshot, control, password,
        )?))
    }
}
/// Parse a complete index, injecting graph decoding for header/property streams.
/// All decoder input still passes through cumulative metadata source accounting.
pub fn read_index_with_decoder(
    source: &mut dyn ReadSource,
    control: &Cancellation,
    limits: ScanLimits,
    password: Option<&str>,
    decoder: &mut dyn HeaderDecoder,
) -> Result<Index, ScanError> {
    let mut budget = Budget::new();
    let mut reader = Reader::new(source, control, limits.read_bytes)?;
    let signature = reader.bytes(0, 32, &mut budget)?;
    if signature[..6] != *b"7z\xbc\xaf\x27\x1c" {
        return Err(ScanError::Invalid);
    }
    if signature[6] != 0 || signature[7] > 4 {
        return Err(ScanError::Unsupported);
    }
    let mut start = Cursor::new(&signature[8..]);
    let checksum = start.u32()?;
    if crc32(&signature[12..]) != checksum {
        return Err(ScanError::Invalid);
    }
    let offset = 32u64.checked_add(start.u64()?).ok_or(ScanError::Invalid)?;
    let size = start.u64()?;
    let next_crc = start.u32()?;
    let count = usize::try_from(size).map_err(|_| ScanError::Limit)?;
    let header = reader.bytes(offset, count, &mut budget)?;
    if checked_crc(&header, control)? != next_crc {
        return Err(ScanError::Invalid);
    }
    if header.is_empty() {
        return Ok(empty());
    }
    let mut c = Cursor::new(&header);
    if c.peek() == Some(23) {
        c.byte()?;
        let encoded = streams::parse(&mut c, &reader, &mut budget, &[])?;
        c.end()?;
        if encoded.folders.is_empty() {
            return Ok(empty());
        }
        if encoded.folders.len() != 1 {
            return Err(ScanError::Invalid);
        }
        let encrypted = password.is_some() && encoded.folders.iter().any(Folder::encrypted);
        let decoded = metadata::decode(
            &mut reader,
            &encoded,
            &mut budget,
            decoder,
            HeaderPurpose::EncodedHeader,
            password,
        )?;
        let header = decoded
            .first()
            .ok_or_else(|| encrypted_scan_error(ScanError::Invalid, encrypted))?;
        if header.first() != Some(&1) {
            return Err(encrypted_scan_error(ScanError::Invalid, encrypted));
        }
        parse_header(header, &mut reader, &mut budget, limits, password, decoder)
            .map_err(|error| encrypted_scan_error(error, encrypted))
    } else {
        parse_header(&header, &mut reader, &mut budget, limits, password, decoder)
    }
}
fn empty() -> Index {
    Index {
        rows: Vec::new(),
        files: Vec::new(),
        streams: Streams::default(),
        has_archive_properties: false,
    }
}
fn parse_header(
    data: &[u8],
    reader: &mut Reader<'_>,
    budget: &mut Budget,
    limits: ScanLimits,
    password: Option<&str>,
    decoder: &mut dyn HeaderDecoder,
) -> Result<Index, ScanError> {
    let mut c = Cursor::new(data);
    c.id(1)?;
    let mut has_archive_properties = false;
    if c.peek() == Some(2) {
        c.byte()?;
        loop {
            reader.check()?;
            if c.byte()? == 0 {
                break;
            }
            has_archive_properties = true;
            let size = usize::try_from(c.number()?).map_err(|_| ScanError::Limit)?;
            c.take(size)?;
        }
    }
    let mut encrypted_properties = false;
    let external = if c.peek() == Some(3) {
        c.byte()?;
        let additional = streams::parse(&mut c, reader, budget, &[])?;
        encrypted_properties =
            password.is_some() && additional.folders.iter().any(Folder::encrypted);
        metadata::decode(
            reader,
            &additional,
            budget,
            decoder,
            HeaderPurpose::AdditionalProperties,
            password,
        )?
    } else {
        Vec::new()
    };
    let result = (|| {
        let streams = if c.peek() == Some(4) {
            c.byte()?;
            streams::parse(&mut c, reader, budget, &external)?
        } else {
            Streams::default()
        };
        let mut index = if c.peek() == Some(5) {
            c.byte()?;
            files::parse(&mut c, streams, &external, budget, limits, reader.control)?
        } else {
            if streams
                .folders
                .iter()
                .any(|folder| !folder.substreams.is_empty())
            {
                return Err(ScanError::Invalid);
            }
            Index {
                rows: Vec::new(),
                files: Vec::new(),
                streams,
                has_archive_properties: false,
            }
        };
        c.id(0)?;
        c.end()?;
        reader.check()?;
        index.has_archive_properties = has_archive_properties;
        Ok(index)
    })();
    result.map_err(|error| encrypted_scan_error(error, encrypted_properties))
}

#[cfg(test)]
mod tests;
