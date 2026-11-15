//! Bounded NSIS `firstheader` metadata inspection.
//!
//! NSIS files are executable installers.  This codec deliberately stops at
//! the PE and installer metadata boundary: it never decompresses the header,
//! interprets installer instructions, opens a plug-in, or exposes a payload
//! decoder.  A matching PE overlay and fixed `firstheader` record are enough
//! to publish one static row for browsing.

use std::io::{ErrorKind, SeekFrom};

use xdzip_codec_api::encode::{
    CreationCodec, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings,
    RewriteCodec, RewriteInput, RewriteOutput,
};
use xdzip_codec_api::names::{self, NameEncoding};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec, PayloadReader};
use xdzip_codec_api::{
    Cancellation, Entry, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
};
use xdzip_domain::identity::EntryId;

const DOS_HEADER_SIZE: usize = 64;
const PE_SIGNATURE_SIZE: usize = 4;
const COFF_HEADER_SIZE: usize = 20;
const SECTION_HEADER_SIZE: usize = 40;
const FIRSTHEADER_SIZE: usize = 28;
const FIRSTHEADER_MAGIC: u32 = 0xdead_beef;
const FIRSTHEADER_MAGIC_ALT: u32 = 0xdead_beed;
const FIRSTHEADER_FLAGS_MASK: u32 = 0x0f;
const MAX_INPUT: u64 = 512 * 1024 * 1024;
const MAX_SCAN: u64 = 64 * 1024 * 1024;
const MAX_HEADER: u32 = 64 * 1024 * 1024;

const FIRSTHEADER_NAME: &str = "NSIS/firstheader";
const NSIS_SIGNATURE: [u32; 3] = [
    u32::from_le_bytes(*b"Null"),
    u32::from_le_bytes(*b"soft"),
    u32::from_le_bytes(*b"Inst"),
];

/// Read-only static NSIS metadata codec.
pub struct NsisCodec;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct FirstHeader {
    flags: u32,
    header_length: u32,
    following_length: u32,
}

impl NsisCodec {
    /// Probe a bounded PE input for a valid NSIS firstheader without using its
    /// suffix as authority.
    pub fn candidate(
        source: &mut dyn ReadSource,
        control: &Cancellation,
    ) -> Result<bool, ScanError> {
        match inspect(source, control) {
            Ok(_) => Ok(true),
            Err(ScanError::Invalid | ScanError::Unsupported) => Ok(false),
            Err(error) => Err(error),
        }
    }
}

impl MetadataCodec for NsisCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        let header = inspect(source, control)?;
        if limits.entries == 0 || FIRSTHEADER_NAME.len() > limits.name_bytes {
            return Err(ScanError::Limit);
        }
        let display_name = names::display(FIRSTHEADER_NAME);
        if display_name.len() > limits.display_name_bytes
            || FIRSTHEADER_NAME.len() > limits.total_name_bytes
        {
            return Err(ScanError::Limit);
        }
        let id = EntryId::try_from(1u64).map_err(|_| ScanError::Limit)?;
        Ok(vec![Entry {
            format: Format::Nsis,
            id,
            raw_name: FIRSTHEADER_NAME.as_bytes().to_vec(),
            decoded_name: None,
            name_encoding: NameEncoding::Utf8,
            display_name,
            directory: false,
            link: false,
            unix_mode: 0,
            size: u64::from(header.header_length),
            size_known: true,
            packed_size: u64::from(header.following_length),
            method: u16::try_from(header.flags & u32::from(u16::MAX)).unwrap_or(u16::MAX),
            modified_dos: 0,
            encrypted: false,
            password_supported: false,
            // The row is metadata only.  Mark it visibly unsupported so no
            // generic extraction/check path can mistake it for a file.
            warning: true,
        }])
    }
}

impl PayloadCodec for NsisCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let rows = self
            .scan(source, control, ScanLimits::default())
            .map_err(decode_error)?;
        if rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(StaticMetadataReader))
    }
}

impl CreationCodec for NsisCodec {
    fn encode(
        &self,
        _input: &dyn EncodeInput,
        _output: &mut dyn EncodeOutput,
        _settings: EncodeSettings<'_>,
        _cancel: &Cancellation,
        _progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}

impl RewriteCodec for NsisCodec {
    fn rewrite(
        &self,
        _original: &mut dyn ReadSource,
        _input: RewriteInput<'_>,
        _output: &mut dyn RewriteOutput,
        _cancel: &Cancellation,
        _progress: &mut dyn FnMut(EncodeProgress<'_>),
    ) -> Result<(), EncodeError> {
        Err(EncodeError::Unsupported)
    }
}

struct StaticMetadataReader;

impl PayloadReader for StaticMetadataReader {
    fn decode(
        &mut self,
        _entry: &Entry,
        _sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        control.check().map_err(decode_error)?;
        Err(DecodeError::Unsupported)
    }
}

fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}

fn inspect(source: &mut dyn ReadSource, control: &Cancellation) -> Result<FirstHeader, ScanError> {
    let length = source.seek(SeekFrom::End(0)).map_err(|_| ScanError::Io)?;
    if !(DOS_HEADER_SIZE as u64..=MAX_INPUT).contains(&length) {
        reset(source)?;
        return Err(if length < DOS_HEADER_SIZE as u64 {
            ScanError::Invalid
        } else {
            ScanError::Limit
        });
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let mut dos = [0u8; DOS_HEADER_SIZE];
    read_exact(source, &mut dos, control)?;
    if dos[..2] != *b"MZ" {
        reset(source)?;
        return Err(ScanError::Invalid);
    }
    let pe_offset = u64::from(u32::from_le_bytes(dos[0x3c..0x40].try_into().unwrap()));
    if pe_offset < DOS_HEADER_SIZE as u64 || pe_offset > 1024 * 1024 {
        reset(source)?;
        return Err(ScanError::Invalid);
    }
    let nt_end = pe_offset
        .checked_add(PE_SIGNATURE_SIZE as u64 + COFF_HEADER_SIZE as u64)
        .ok_or(ScanError::Invalid)?;
    if nt_end > length {
        reset(source)?;
        return Err(ScanError::Invalid);
    }
    source
        .seek(SeekFrom::Start(pe_offset))
        .map_err(|_| ScanError::Io)?;
    let mut nt = [0u8; PE_SIGNATURE_SIZE + COFF_HEADER_SIZE];
    read_exact(source, &mut nt, control)?;
    if nt[..4] != *b"PE\0\0" {
        reset(source)?;
        return Err(ScanError::Invalid);
    }
    let sections = usize::from(u16::from_le_bytes(nt[6..8].try_into().unwrap()));
    let optional_size = u64::from(u16::from_le_bytes(nt[20..22].try_into().unwrap()));
    if sections == 0 || sections > 96 {
        reset(source)?;
        return Err(ScanError::Invalid);
    }
    let section_table = pe_offset
        .checked_add((PE_SIGNATURE_SIZE + COFF_HEADER_SIZE) as u64)
        .and_then(|value| value.checked_add(optional_size))
        .ok_or(ScanError::Invalid)?;
    let section_bytes = u64::try_from(sections)
        .ok()
        .and_then(|count| count.checked_mul(SECTION_HEADER_SIZE as u64))
        .ok_or(ScanError::Limit)?;
    if section_table
        .checked_add(section_bytes)
        .is_none_or(|end| end > length)
    {
        reset(source)?;
        return Err(ScanError::Invalid);
    }
    let mut overlay = section_table + section_bytes;
    let mut section = [0u8; SECTION_HEADER_SIZE];
    for index in 0..sections {
        control.check()?;
        let offset = section_table
            .checked_add(
                u64::try_from(index).map_err(|_| ScanError::Limit)? * SECTION_HEADER_SIZE as u64,
            )
            .ok_or(ScanError::Limit)?;
        source
            .seek(SeekFrom::Start(offset))
            .map_err(|_| ScanError::Io)?;
        read_exact(source, &mut section, control)?;
        let raw_size = u64::from(u32::from_le_bytes(section[16..20].try_into().unwrap()));
        let raw_offset = u64::from(u32::from_le_bytes(section[20..24].try_into().unwrap()));
        if raw_size == 0 {
            continue;
        }
        let end = raw_offset.checked_add(raw_size).ok_or(ScanError::Invalid)?;
        if end > length {
            reset(source)?;
            return Err(ScanError::Invalid);
        }
        overlay = overlay.max(end);
    }
    let scan_end = length.min(overlay.saturating_add(MAX_SCAN));
    let found = scan_overlay(source, control, overlay, scan_end, length)?;
    reset(source)?;
    found.ok_or(ScanError::Invalid)
}

fn scan_overlay(
    source: &mut dyn ReadSource,
    control: &Cancellation,
    start: u64,
    end: u64,
    physical_length: u64,
) -> Result<Option<FirstHeader>, ScanError> {
    let mut position = start & !3;
    let mut carry = Vec::new();
    let mut buffer = vec![0u8; 64 * 1024];
    while position < end {
        control.check()?;
        let count = usize::try_from((end - position).min(buffer.len() as u64))
            .map_err(|_| ScanError::Limit)?;
        source
            .seek(SeekFrom::Start(position))
            .map_err(|_| ScanError::Io)?;
        read_exact(source, &mut buffer[..count], control)?;
        let mut combined = Vec::with_capacity(carry.len() + count);
        combined.extend_from_slice(&carry);
        combined.extend_from_slice(&buffer[..count]);
        let base = position.saturating_sub(carry.len() as u64);
        for (index, window) in combined.windows(FIRSTHEADER_SIZE).enumerate() {
            let absolute = base + index as u64;
            if absolute & 0x1ff != 0 {
                continue;
            }
            if let Some(header) = parse_firstheader(window, physical_length - absolute) {
                return Ok(Some(header));
            }
        }
        carry.clear();
        carry.extend_from_slice(&combined[combined.len().saturating_sub(FIRSTHEADER_SIZE - 1)..]);
        position = position.checked_add(count as u64).ok_or(ScanError::Limit)?;
    }
    Ok(None)
}

fn parse_firstheader(bytes: &[u8], available: u64) -> Option<FirstHeader> {
    if bytes.len() < FIRSTHEADER_SIZE
        || u32::from_le_bytes(bytes[..4].try_into().ok()?) & !FIRSTHEADER_FLAGS_MASK != 0
        || !matches!(
            u32::from_le_bytes(bytes[4..8].try_into().ok()?),
            FIRSTHEADER_MAGIC | FIRSTHEADER_MAGIC_ALT
        )
        || NSIS_SIGNATURE
            != [
                u32::from_le_bytes(bytes[8..12].try_into().ok()?),
                u32::from_le_bytes(bytes[12..16].try_into().ok()?),
                u32::from_le_bytes(bytes[16..20].try_into().ok()?),
            ]
    {
        return None;
    }
    let header_length = u32::from_le_bytes(bytes[20..24].try_into().ok()?);
    let following_length = u32::from_le_bytes(bytes[24..28].try_into().ok()?);
    if header_length < FIRSTHEADER_SIZE as u32
        || header_length > MAX_HEADER
        || following_length < header_length
        || u64::from(following_length) > available
    {
        return None;
    }
    Some(FirstHeader {
        flags: u32::from_le_bytes(bytes[..4].try_into().ok()?),
        header_length,
        following_length,
    })
}

fn read_exact(
    source: &mut dyn ReadSource,
    bytes: &mut [u8],
    control: &Cancellation,
) -> Result<(), ScanError> {
    let mut offset = 0;
    while offset < bytes.len() {
        control.check()?;
        match source.read(&mut bytes[offset..]) {
            Ok(0) => return Err(ScanError::Invalid),
            Ok(count) => offset = offset.checked_add(count).ok_or(ScanError::Limit)?,
            Err(error) if error.kind() == ErrorKind::Interrupted && error.get_ref().is_none() => {}
            Err(error) => return Err(ScanError::from_source_error(&error)),
        }
    }
    Ok(())
}

fn reset(source: &mut dyn ReadSource) -> Result<(), ScanError> {
    source
        .seek(SeekFrom::Start(0))
        .map(|_| ())
        .map_err(|_| ScanError::Io)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> Vec<u8> {
        let pe_offset = 0x80usize;
        let section_table = pe_offset + 4 + COFF_HEADER_SIZE + 0xe0;
        let overlay = 0x400usize;
        let mut bytes = vec![0u8; overlay + 128];
        bytes[..2].copy_from_slice(b"MZ");
        bytes[0x3c..0x40].copy_from_slice(&(pe_offset as u32).to_le_bytes());
        bytes[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");
        bytes[pe_offset + 4..pe_offset + 6].copy_from_slice(&0x14cu16.to_le_bytes());
        bytes[pe_offset + 6..pe_offset + 8].copy_from_slice(&1u16.to_le_bytes());
        bytes[pe_offset + 20..pe_offset + 22].copy_from_slice(&0xe0u16.to_le_bytes());
        let section = section_table;
        bytes[section + 8..section + 12].copy_from_slice(&0x200u32.to_le_bytes());
        bytes[section + 16..section + 20].copy_from_slice(&0x200u32.to_le_bytes());
        bytes[section + 20..section + 24].copy_from_slice(&0x200u32.to_le_bytes());
        let first = overlay;
        bytes[first..first + 4].copy_from_slice(&0x02u32.to_le_bytes());
        bytes[first + 4..first + 8].copy_from_slice(&FIRSTHEADER_MAGIC.to_le_bytes());
        for (index, word) in NSIS_SIGNATURE.iter().enumerate() {
            let begin = first + 8 + index * 4;
            bytes[begin..begin + 4].copy_from_slice(&word.to_le_bytes());
        }
        bytes[first + 20..first + 24].copy_from_slice(&64u32.to_le_bytes());
        bytes[first + 24..first + 28].copy_from_slice(&128u32.to_le_bytes());
        bytes
    }

    #[test]
    fn scans_only_the_static_firstheader_row() {
        let rows = NsisCodec
            .scan(
                &mut Cursor::new(fixture()),
                &Cancellation::default(),
                ScanLimits::default(),
            )
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].format, Format::Nsis);
        assert_eq!(rows[0].raw_name, FIRSTHEADER_NAME.as_bytes());
        assert_eq!(rows[0].size, 64);
        assert_eq!(rows[0].packed_size, 128);
        assert!(rows[0].warning);
        assert!(!rows[0].payload_supported());
    }

    #[test]
    fn rejects_non_pe_and_bad_signature_without_execution() {
        let mut bytes = fixture();
        bytes[0] = b'X';
        assert_eq!(
            NsisCodec::candidate(&mut Cursor::new(bytes), &Cancellation::default()),
            Ok(false)
        );
        let mut bytes = fixture();
        bytes[0x400 + 4..0x400 + 8].copy_from_slice(&0xdead_beeeu32.to_le_bytes());
        assert_eq!(
            NsisCodec::candidate(&mut Cursor::new(bytes), &Cancellation::default()),
            Ok(false)
        );
    }

    #[test]
    fn static_row_cannot_be_decoded() {
        let bytes = fixture();
        let mut source = Cursor::new(bytes);
        let rows = NsisCodec
            .scan(&mut source, &Cancellation::default(), ScanLimits::default())
            .unwrap();
        let open_control = Cancellation::default();
        let mut reader = NsisCodec
            .open_payload(&mut source, &rows, &open_control)
            .unwrap();
        struct Sink;
        impl DecodedSink for Sink {
            fn write_chunk(&mut self, _bytes: &[u8]) -> Result<(), DecodeError> {
                panic!("NSIS static metadata must never write payload bytes")
            }
        }
        assert_eq!(
            reader.decode(&rows[0], &mut Sink, &Cancellation::default()),
            Err(DecodeError::Unsupported)
        );
    }
}
