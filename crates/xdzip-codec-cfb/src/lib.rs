//! Bounded, read-only Microsoft Compound File (CFB) metadata and stream access.
//!
//! MSI files use CFB as their outer container. This codec exposes storage and
//! stream names and can copy raw stream bytes, while deliberately interpreting
//! no MSI tables, transforms, scripts, custom actions, or embedded executables.

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

const SIGNATURE: [u8; 8] = [0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1];
const FREESECT: u32 = 0xffff_ffff;
const ENDOFCHAIN: u32 = 0xffff_fffe;
const FATSECT: u32 = 0xffff_fffd;
const DIFSECT: u32 = 0xffff_fffc;
const NOSTREAM: u32 = FREESECT;
const MAXREGSECT: u32 = 0xffff_fffa;
const MINI_STREAM_CUTOFF: u64 = 4096;
const MINI_SECTOR_SIZE: usize = 64;
const METADATA_LIMIT: u64 = 64 * 1024 * 1024;
const PAYLOAD_LIMIT: u64 = 512 * 1024 * 1024;

/// Read-only CFB codec. MSI execution and mutation are intentionally absent.
pub struct CfbCodec;

#[derive(Clone)]
struct Locator {
    size: u64,
    mini: bool,
    sectors: Vec<u32>,
}

#[derive(Clone)]
struct Record {
    entry: Entry,
    locator: Option<Locator>,
}

struct Parsed {
    rows: Vec<Entry>,
    records: Vec<Record>,
    mini_stream: Vec<u8>,
    sector_size: usize,
}

#[derive(Clone)]
struct DirEntry {
    kind: u8,
    name: String,
    name_warning: bool,
    left: u32,
    right: u32,
    child: u32,
    start_sector: u32,
    size: u64,
}

impl CfbCodec {
    /// Confirm the CFB signature without trusting a filename extension.
    pub fn candidate(
        source: &mut dyn ReadSource,
        control: &Cancellation,
    ) -> Result<bool, ScanError> {
        control.check()?;
        source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
        let mut signature = [0u8; 8];
        let result = match source.read_exact(&mut signature) {
            Ok(()) => Ok(signature == SIGNATURE),
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => Ok(false),
            Err(error) => Err(ScanError::from_source_error(&error)),
        };
        source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
        result
    }
}

impl MetadataCodec for CfbCodec {
    fn scan(
        &self,
        source: &mut dyn ReadSource,
        control: &Cancellation,
        limits: ScanLimits,
    ) -> Result<Vec<Entry>, ScanError> {
        parse(
            &read_source(source, control, limits.read_bytes.min(METADATA_LIMIT))?,
            control,
            limits,
        )
        .map(|parsed| parsed.rows)
    }
}

impl PayloadCodec for CfbCodec {
    fn open_payload<'a>(
        &self,
        source: &'a mut dyn ReadSource,
        snapshot: &[Entry],
        control: &'a Cancellation,
    ) -> Result<Box<dyn PayloadReader + 'a>, DecodeError> {
        let bytes = read_source(source, control, PAYLOAD_LIMIT).map_err(decode_error)?;
        let parsed = parse(&bytes, control, ScanLimits::default()).map_err(decode_error)?;
        if parsed.rows != snapshot {
            return Err(DecodeError::Integrity);
        }
        Ok(Box::new(Payload {
            bytes,
            parsed,
            failure: None,
        }))
    }
}

impl CreationCodec for CfbCodec {
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

impl RewriteCodec for CfbCodec {
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

struct Payload {
    bytes: Vec<u8>,
    parsed: Parsed,
    failure: Option<DecodeError>,
}

impl PayloadReader for Payload {
    fn decode(
        &mut self,
        entry: &Entry,
        sink: &mut dyn DecodedSink,
        control: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = (|| {
            control.check().map_err(decode_error)?;
            let index = entry
                .id
                .get()
                .checked_sub(1)
                .and_then(|value| usize::try_from(value).ok())
                .ok_or(DecodeError::Integrity)?;
            let record = self
                .parsed
                .records
                .get(index)
                .ok_or(DecodeError::Integrity)?;
            if &record.entry != entry {
                return Err(DecodeError::Integrity);
            }
            let locator = record.locator.as_ref().ok_or(DecodeError::Unsupported)?;
            if !entry.payload_supported() {
                return Err(DecodeError::Unsupported);
            }
            let mut remaining = locator.size;
            for &sector in &locator.sectors {
                control.check().map_err(decode_error)?;
                if remaining == 0 {
                    break;
                }
                let (offset, width) = if locator.mini {
                    let offset = usize::try_from(sector)
                        .ok()
                        .and_then(|value| value.checked_mul(MINI_SECTOR_SIZE))
                        .ok_or(DecodeError::Integrity)?;
                    (offset, MINI_SECTOR_SIZE)
                } else {
                    let offset = sector_offset(sector, self.parsed.sector_size, self.bytes.len())
                        .map_err(|_| DecodeError::Integrity)?;
                    (offset, self.parsed.sector_size)
                };
                let data = if locator.mini {
                    self.parsed.mini_stream.get(offset..offset + width)
                } else {
                    self.bytes.get(offset..offset + width)
                }
                .ok_or(DecodeError::Integrity)?;
                let take = remaining.min(data.len() as u64) as usize;
                sink.write_chunk(&data[..take])?;
                remaining -= take as u64;
            }
            if remaining != 0 {
                return Err(DecodeError::Integrity);
            }
            Ok(())
        })();
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
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

fn read_source(
    source: &mut dyn ReadSource,
    control: &Cancellation,
    limit: u64,
) -> Result<Vec<u8>, ScanError> {
    let length = source.seek(SeekFrom::End(0)).map_err(|_| ScanError::Io)?;
    if length < 512 {
        return Err(ScanError::Invalid);
    }
    if length > limit {
        return Err(ScanError::Limit);
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let capacity = usize::try_from(length).map_err(|_| ScanError::Limit)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| ScanError::Limit)?;
    bytes.resize(capacity, 0);
    let mut offset = 0usize;
    while offset != capacity {
        control.check()?;
        let read = match source.read(&mut bytes[offset..]) {
            Ok(read) => read,
            Err(error) if error.kind() == ErrorKind::Interrupted && error.get_ref().is_none() => {
                continue;
            }
            Err(error) => return Err(ScanError::from_source_error(&error)),
        };
        if read == 0 {
            return Err(ScanError::Invalid);
        }
        offset = offset.checked_add(read).ok_or(ScanError::Limit)?;
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    Ok(bytes)
}

fn parse(bytes: &[u8], control: &Cancellation, limits: ScanLimits) -> Result<Parsed, ScanError> {
    if bytes.get(..8) != Some(&SIGNATURE) {
        return Err(ScanError::Invalid);
    }
    let major = read_u16(bytes, 26)?;
    let sector_shift = read_u16(bytes, 30)?;
    let mini_sector_shift = read_u16(bytes, 32)?;
    if !matches!(major, 3 | 4) || !matches!(sector_shift, 9 | 12) || mini_sector_shift != 6 {
        return Err(ScanError::Unsupported);
    }
    if read_u16(bytes, 28)? != 0xfffe {
        return Err(ScanError::Invalid);
    }
    let sector_size = 1usize << sector_shift;
    if !(bytes.len() - 512).is_multiple_of(sector_size) {
        return Err(ScanError::Invalid);
    }
    let sector_count = (bytes.len() - 512) / sector_size;
    if sector_count == 0 || sector_count > usize::try_from(MAXREGSECT).unwrap_or(usize::MAX) {
        return Err(ScanError::Limit);
    }
    let num_fat = usize::try_from(read_u32(bytes, 44)?).map_err(|_| ScanError::Limit)?;
    let first_dir = read_u32(bytes, 48)?;
    let mini_cutoff = read_u32(bytes, 56)? as u64;
    if mini_cutoff != MINI_STREAM_CUTOFF {
        return Err(ScanError::Unsupported);
    }
    let first_mini_fat = read_u32(bytes, 60)?;
    let num_mini_fat = usize::try_from(read_u32(bytes, 64)?).map_err(|_| ScanError::Limit)?;
    let first_difat = read_u32(bytes, 68)?;
    let num_difat = usize::try_from(read_u32(bytes, 72)?).map_err(|_| ScanError::Limit)?;
    if num_fat == 0 || num_fat > sector_count || num_difat > sector_count {
        return Err(ScanError::Invalid);
    }
    let mut fat_sectors = Vec::new();
    for slot in 0..109usize {
        let value = read_u32(bytes, 76 + slot * 4)?;
        if value != FREESECT {
            fat_sectors.push(value);
        }
    }
    if fat_sectors.len() > num_fat {
        return Err(ScanError::Invalid);
    }
    let mut current = first_difat;
    let mut difat_seen = Vec::new();
    for _ in 0..num_difat {
        if current == ENDOFCHAIN || current == FREESECT {
            return Err(ScanError::Invalid);
        }
        if difat_seen.contains(&current) {
            return Err(ScanError::Invalid);
        }
        difat_seen.push(current);
        let sector = read_sector(bytes, current, sector_size)?;
        let slots = sector_size / 4 - 1;
        for slot in 0..slots {
            let value = read_u32(sector, slot * 4)?;
            if value != FREESECT {
                fat_sectors.push(value);
            }
        }
        current = read_u32(sector, sector_size - 4)?;
    }
    if fat_sectors.len() != num_fat
        || fat_sectors
            .iter()
            .enumerate()
            .any(|(index, &sector)| fat_sectors[..index].contains(&sector))
        || fat_sectors
            .iter()
            .any(|&sector| usize::try_from(sector).map_or(true, |id| id >= sector_count))
    {
        return Err(ScanError::Invalid);
    }
    let mut fat = Vec::new();
    fat.try_reserve_exact(
        num_fat
            .checked_mul(sector_size / 4)
            .ok_or(ScanError::Limit)?,
    )
    .map_err(|_| ScanError::Limit)?;
    for &sector in &fat_sectors {
        let data = read_sector(bytes, sector, sector_size)?;
        for slot in 0..sector_size / 4 {
            fat.push(read_u32(data, slot * 4)?);
        }
    }
    let dir_chain = collect_chain(first_dir, &fat, sector_count)?;
    if dir_chain.is_empty() || dir_chain.len() > limits.entries.saturating_mul(4).max(1024) {
        return Err(ScanError::Limit);
    }
    let directory = chain_bytes(bytes, &dir_chain, sector_size)?;
    if directory.len() % 128 != 0 {
        return Err(ScanError::Invalid);
    }
    let mut entries = Vec::new();
    let (directory_chunks, directory_remainder) = directory.as_chunks::<128>();
    if !directory_remainder.is_empty() {
        return Err(ScanError::Invalid);
    }
    for chunk in directory_chunks {
        control.check()?;
        if let Some(entry) = parse_dir_entry(chunk)? {
            entries.push(entry);
        } else {
            entries.push(DirEntry {
                kind: 0,
                name: String::new(),
                name_warning: false,
                left: NOSTREAM,
                right: NOSTREAM,
                child: NOSTREAM,
                start_sector: ENDOFCHAIN,
                size: 0,
            });
        }
    }
    if entries.first().map(|entry| entry.kind) != Some(5) {
        return Err(ScanError::Invalid);
    }
    let mini_fat = if num_mini_fat == 0 {
        if first_mini_fat != ENDOFCHAIN {
            return Err(ScanError::Invalid);
        }
        Vec::new()
    } else {
        let chain = collect_chain(first_mini_fat, &fat, sector_count)?;
        if chain.len() < num_mini_fat {
            return Err(ScanError::Invalid);
        }
        let data = chain_bytes(bytes, &chain[..num_mini_fat], sector_size)?;
        let (chunks, remainder) = data.as_chunks::<4>();
        if !remainder.is_empty() {
            return Err(ScanError::Invalid);
        }
        chunks
            .iter()
            .map(|chunk| read_u32(chunk, 0))
            .collect::<Result<Vec<_>, _>>()?
    };
    let root = &entries[0];
    let mini_stream = if root.size == 0 {
        if root.start_sector != ENDOFCHAIN {
            return Err(ScanError::Invalid);
        }
        Vec::new()
    } else {
        let needed = sectors_for(root.size, sector_size)?;
        let chain = collect_chain(root.start_sector, &fat, sector_count)?;
        if chain.len() < needed {
            return Err(ScanError::Invalid);
        }
        let mut data = chain_bytes(bytes, &chain[..needed], sector_size)?;
        data.truncate(usize::try_from(root.size).map_err(|_| ScanError::Limit)?);
        data
    };
    let mut records = Vec::new();
    records
        .try_reserve(limits.entries)
        .map_err(|_| ScanError::Limit)?;
    let mut visited = vec![false; entries.len()];
    walk_tree(
        entries[0].child,
        "",
        &entries,
        &fat,
        &mini_fat,
        &mini_stream,
        bytes,
        sector_size,
        &mut visited,
        &mut records,
        &mut Vec::new(),
        control,
        limits,
    )?;
    if records.len() > limits.entries {
        return Err(ScanError::Limit);
    }
    let (raw_names, display_names) =
        records
            .iter()
            .try_fold((0usize, 0usize), |(raw, display), record| {
                Ok::<_, ScanError>((
                    raw.checked_add(record.entry.raw_name.len())
                        .ok_or(ScanError::Limit)?,
                    display
                        .checked_add(record.entry.display_name.len())
                        .ok_or(ScanError::Limit)?,
                ))
            })?;
    if raw_names > limits.total_name_bytes || display_names > limits.display_name_bytes {
        return Err(ScanError::Limit);
    }
    let rows = records.iter().map(|record| record.entry.clone()).collect();
    Ok(Parsed {
        rows,
        records,
        mini_stream,
        sector_size,
    })
}

#[allow(clippy::too_many_arguments)]
fn walk_tree(
    node: u32,
    parent: &str,
    entries: &[DirEntry],
    fat: &[u32],
    mini_fat: &[u32],
    mini_stream: &[u8],
    bytes: &[u8],
    sector_size: usize,
    visited: &mut [bool],
    records: &mut Vec<Record>,
    stack: &mut Vec<u32>,
    control: &Cancellation,
    limits: ScanLimits,
) -> Result<(), ScanError> {
    if node == NOSTREAM {
        return Ok(());
    }
    let index = usize::try_from(node).map_err(|_| ScanError::Invalid)?;
    let entry = entries.get(index).ok_or(ScanError::Invalid)?;
    if entry.kind == 0 || visited[index] || stack.len() > entries.len() {
        return Err(ScanError::Invalid);
    }
    visited[index] = true;
    stack.push(node);
    walk_tree(
        entry.left,
        parent,
        entries,
        fat,
        mini_fat,
        mini_stream,
        bytes,
        sector_size,
        visited,
        records,
        stack,
        control,
        limits,
    )?;
    control.check()?;
    let path_length = if parent.is_empty() {
        entry.name.len()
    } else {
        parent
            .len()
            .checked_add(1)
            .and_then(|length| length.checked_add(entry.name.len()))
            .ok_or(ScanError::Limit)?
    };
    if path_length > limits.name_bytes {
        return Err(ScanError::Limit);
    }
    let path = if parent.is_empty() {
        entry.name.clone()
    } else {
        format!("{parent}/{}", entry.name)
    };
    let warning = entry.name_warning || names::suspicious(&path, entry.kind == 1);
    let id = EntryId::try_from(records.len() as u64 + 1).map_err(|_| ScanError::Limit)?;
    let row = Entry {
        format: Format::Compound,
        id,
        raw_name: path.as_bytes().to_vec(),
        decoded_name: None,
        name_encoding: NameEncoding::Utf8,
        display_name: names::display(&path),
        directory: entry.kind == 1,
        link: false,
        unix_mode: 0,
        size: entry.size,
        size_known: true,
        packed_size: entry.size,
        method: 0,
        modified_dos: 0,
        encrypted: false,
        password_supported: false,
        warning,
    };
    let locator = if entry.kind == 2 {
        Some(locator(
            entry,
            fat,
            mini_fat,
            mini_stream,
            bytes,
            sector_size,
        )?)
    } else {
        None
    };
    records.push(Record {
        entry: row,
        locator,
    });
    if entry.kind == 1 {
        walk_tree(
            entry.child,
            &path,
            entries,
            fat,
            mini_fat,
            mini_stream,
            bytes,
            sector_size,
            visited,
            records,
            stack,
            control,
            limits,
        )?;
    } else if entry.child != NOSTREAM {
        return Err(ScanError::Invalid);
    }
    walk_tree(
        entry.right,
        parent,
        entries,
        fat,
        mini_fat,
        mini_stream,
        bytes,
        sector_size,
        visited,
        records,
        stack,
        control,
        limits,
    )?;
    stack.pop();
    Ok(())
}

fn locator(
    entry: &DirEntry,
    fat: &[u32],
    mini_fat: &[u32],
    mini_stream: &[u8],
    bytes: &[u8],
    sector_size: usize,
) -> Result<Locator, ScanError> {
    if entry.size == 0 {
        if entry.start_sector != ENDOFCHAIN {
            return Err(ScanError::Invalid);
        }
        return Ok(Locator {
            size: 0,
            mini: entry.size < MINI_STREAM_CUTOFF,
            sectors: Vec::new(),
        });
    }
    if entry.size < MINI_STREAM_CUTOFF {
        let needed = sectors_for(entry.size, MINI_SECTOR_SIZE)?;
        let chain = collect_chain(entry.start_sector, mini_fat, mini_fat.len())?;
        if chain.len() < needed
            || chain[..needed].iter().any(|&sector| {
                usize::try_from(sector)
                    .ok()
                    .and_then(|id| id.checked_mul(MINI_SECTOR_SIZE))
                    .is_none_or(|offset| offset >= mini_stream.len())
            })
        {
            return Err(ScanError::Invalid);
        }
        Ok(Locator {
            size: entry.size,
            mini: true,
            sectors: chain[..needed].to_vec(),
        })
    } else {
        let needed = sectors_for(entry.size, sector_size)?;
        let chain = collect_chain(
            entry.start_sector,
            fat,
            bytes.len().saturating_sub(512) / sector_size,
        )?;
        if chain.len() < needed {
            return Err(ScanError::Invalid);
        }
        Ok(Locator {
            size: entry.size,
            mini: false,
            sectors: chain[..needed].to_vec(),
        })
    }
}

fn parse_dir_entry(bytes: &[u8]) -> Result<Option<DirEntry>, ScanError> {
    let kind = *bytes.get(66).ok_or(ScanError::Invalid)?;
    if kind == 0 {
        return Ok(None);
    }
    if !matches!(kind, 1 | 2 | 5) {
        return Err(ScanError::Unsupported);
    }
    let name_bytes = usize::from(read_u16(bytes, 64)?);
    if !(2..=64).contains(&name_bytes) || name_bytes % 2 != 0 {
        return Err(ScanError::Invalid);
    }
    let units = name_bytes / 2 - 1;
    let mut raw_units = Vec::with_capacity(units);
    for index in 0..units {
        raw_units.push(read_u16(bytes, index * 2)?);
    }
    let mut warning = false;
    let name = match String::from_utf16(&raw_units) {
        Ok(text) => text,
        Err(_) => {
            warning = true;
            String::from_utf16_lossy(&raw_units)
        }
    };
    Ok(Some(DirEntry {
        kind,
        name,
        name_warning: warning,
        left: read_u32(bytes, 68)?,
        right: read_u32(bytes, 72)?,
        child: read_u32(bytes, 76)?,
        start_sector: read_u32(bytes, 116)?,
        size: read_u64(bytes, 120)?,
    }))
}

fn collect_chain(start: u32, table: &[u32], max: usize) -> Result<Vec<u32>, ScanError> {
    if start == ENDOFCHAIN {
        return Ok(Vec::new());
    }
    let mut chain = Vec::new();
    let mut current = start;
    while current != ENDOFCHAIN {
        let index = usize::try_from(current).map_err(|_| ScanError::Invalid)?;
        if index >= table.len() || chain.len() == max || chain.contains(&current) {
            return Err(ScanError::Invalid);
        }
        let next = table[index];
        if matches!(next, FREESECT | FATSECT | DIFSECT | MAXREGSECT) {
            return Err(ScanError::Invalid);
        }
        chain.push(current);
        current = next;
    }
    Ok(chain)
}

fn chain_bytes(bytes: &[u8], chain: &[u32], sector_size: usize) -> Result<Vec<u8>, ScanError> {
    let length = chain
        .len()
        .checked_mul(sector_size)
        .ok_or(ScanError::Limit)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .map_err(|_| ScanError::Limit)?;
    for &sector in chain {
        output.extend_from_slice(read_sector(bytes, sector, sector_size)?);
    }
    Ok(output)
}

fn sectors_for(size: u64, sector_size: usize) -> Result<usize, ScanError> {
    let width = u64::try_from(sector_size).map_err(|_| ScanError::Limit)?;
    usize::try_from(size.checked_add(width - 1).ok_or(ScanError::Limit)? / width)
        .map_err(|_| ScanError::Limit)
}

fn sector_offset(sector: u32, sector_size: usize, length: usize) -> Result<usize, ScanError> {
    let offset = 512usize
        .checked_add(
            usize::try_from(sector)
                .map_err(|_| ScanError::Invalid)?
                .checked_mul(sector_size)
                .ok_or(ScanError::Limit)?,
        )
        .ok_or(ScanError::Limit)?;
    if offset
        .checked_add(sector_size)
        .is_none_or(|end| end > length)
    {
        Err(ScanError::Invalid)
    } else {
        Ok(offset)
    }
}

fn read_sector(bytes: &[u8], sector: u32, sector_size: usize) -> Result<&[u8], ScanError> {
    let offset = sector_offset(sector, sector_size, bytes.len())?;
    bytes
        .get(offset..offset + sector_size)
        .ok_or(ScanError::Invalid)
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, ScanError> {
    let raw = bytes.get(offset..offset + 2).ok_or(ScanError::Invalid)?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, ScanError> {
    let raw = bytes.get(offset..offset + 4).ok_or(ScanError::Invalid)?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, ScanError> {
    let raw = bytes.get(offset..offset + 8).ok_or(ScanError::Invalid)?;
    Ok(u64::from_le_bytes([
        raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
    ]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn fixture() -> Vec<u8> {
        let sector_size = 512usize;
        let mut bytes = vec![0u8; 512 + sector_size * 4];
        bytes[..8].copy_from_slice(&SIGNATURE);
        bytes[24..26].copy_from_slice(&0x003e_u16.to_le_bytes());
        bytes[26..28].copy_from_slice(&3u16.to_le_bytes());
        bytes[28..30].copy_from_slice(&0xfffe_u16.to_le_bytes());
        bytes[30..32].copy_from_slice(&9u16.to_le_bytes());
        bytes[32..34].copy_from_slice(&6u16.to_le_bytes());
        bytes[44..48].copy_from_slice(&1u32.to_le_bytes());
        bytes[48..52].copy_from_slice(&1u32.to_le_bytes());
        bytes[56..60].copy_from_slice(&(MINI_STREAM_CUTOFF as u32).to_le_bytes());
        bytes[60..64].copy_from_slice(&3u32.to_le_bytes());
        bytes[64..68].copy_from_slice(&1u32.to_le_bytes());
        bytes[68..72].copy_from_slice(&ENDOFCHAIN.to_le_bytes());
        bytes[72..76].copy_from_slice(&0u32.to_le_bytes());
        bytes[76..80].copy_from_slice(&0u32.to_le_bytes());
        for slot in 1..109 {
            bytes[76 + slot * 4..80 + slot * 4].copy_from_slice(&FREESECT.to_le_bytes());
        }
        let fat = 512;
        bytes[fat..fat + 4].copy_from_slice(&FATSECT.to_le_bytes());
        bytes[fat + 4..fat + 8].copy_from_slice(&ENDOFCHAIN.to_le_bytes());
        bytes[fat + 8..fat + 12].copy_from_slice(&ENDOFCHAIN.to_le_bytes());
        bytes[fat + 12..fat + 16].copy_from_slice(&ENDOFCHAIN.to_le_bytes());
        let directory = 512 + sector_size;
        let root = directory;
        write_name(&mut bytes[root..root + 128], "Root Entry", 5);
        bytes[root + 76..root + 80].copy_from_slice(&1u32.to_le_bytes());
        bytes[root + 116..root + 120].copy_from_slice(&2u32.to_le_bytes());
        bytes[root + 120..root + 128].copy_from_slice(&64u64.to_le_bytes());
        let stream = directory + 128;
        write_name(&mut bytes[stream..stream + 128], "hello", 2);
        bytes[stream + 68..stream + 72].copy_from_slice(&NOSTREAM.to_le_bytes());
        bytes[stream + 72..stream + 76].copy_from_slice(&NOSTREAM.to_le_bytes());
        bytes[stream + 76..stream + 80].copy_from_slice(&NOSTREAM.to_le_bytes());
        bytes[stream + 116..stream + 120].copy_from_slice(&0u32.to_le_bytes());
        bytes[stream + 120..stream + 128].copy_from_slice(&5u64.to_le_bytes());
        let payload = 512 + sector_size * 2;
        bytes[payload..payload + 5].copy_from_slice(b"world");
        let mini_fat = 512 + sector_size * 3;
        bytes[mini_fat..mini_fat + 4].copy_from_slice(&ENDOFCHAIN.to_le_bytes());
        bytes
    }

    fn write_name(slot: &mut [u8], name: &str, kind: u8) {
        let units: Vec<u16> = name.encode_utf16().collect();
        for (index, unit) in units.iter().enumerate() {
            slot[index * 2..index * 2 + 2].copy_from_slice(&unit.to_le_bytes());
        }
        slot[units.len() * 2..units.len() * 2 + 2].copy_from_slice(&0u16.to_le_bytes());
        slot[64..66].copy_from_slice(&u16::try_from((units.len() + 1) * 2).unwrap().to_le_bytes());
        slot[66] = kind;
    }

    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }
    }

    #[test]
    fn parses_stream_and_decodes_payload() {
        let bytes = fixture();
        let cancel = Cancellation::default();
        let mut source = Cursor::new(bytes.clone());
        let rows = CfbCodec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].display_name, "hello");
        assert_eq!(rows[0].format, Format::Compound);
        assert!(rows[0].payload_supported());
        let mut source = Cursor::new(bytes);
        let mut payload = CfbCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink(Vec::new());
        payload.decode(&rows[0], &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, b"world");
    }

    #[test]
    fn rejects_bad_signature_and_cycles() {
        let cancel = Cancellation::default();
        let mut source = Cursor::new(vec![0u8; 512]);
        assert_eq!(CfbCodec::candidate(&mut source, &cancel), Ok(false));
        let mut bad = fixture();
        bad[512 + 1..512 + 5].copy_from_slice(&1u32.to_le_bytes());
        let mut source = Cursor::new(bad);
        assert_eq!(
            CfbCodec.scan(&mut source, &cancel, ScanLimits::default()),
            Err(ScanError::Invalid)
        );
    }
}
