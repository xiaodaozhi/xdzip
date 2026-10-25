//! ISO records are decoded directly from ECMA-119 (5th edition), §§7–10.
//! Joliet follows its Microsoft specification reproduced in ECMA-119 Annex C.
//! SUSP and RRIP follow IEEE P1281/P1282 draft 1.12, §§4–6 and §4 respectively.
//! No mount, executable boot operation, or filesystem authority is provided here.
use crate::image::{Extent, ImageReader, Index, IndexBuilder, Node};
use std::collections::{BTreeMap, BTreeSet};
use xdzip_codec_api::{Format, ScanError, ScanLimits};

const SECTOR: u64 = 2048;
const MAX_DEPTH: usize = 256;
const MAX_CE_BYTES: usize = 1024 * 1024;
const MAX_CE_AREAS: usize = 128;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
struct RockRidge {
    name: Option<Vec<u8>>,
    px: Option<Vec<u32>>,
    symlink: bool,
    modified: Option<u32>,
    child: Option<u32>,
    parent: Option<u32>,
    relocated: bool,
}

#[derive(Clone, Debug)]
struct FileRecord {
    identifier: Vec<u8>,
    lbn: u32,
    xattr: u8,
    length: u64,
    flags: u8,
    time: u32,
    system: Vec<u8>,
    rr: RockRidge,
    extents: Vec<Extent>,
}

impl FileRecord {
    fn directory(&self) -> bool {
        self.flags & 2 != 0
    }
    fn key(&self) -> (u32, u8, u64) {
        (self.lbn, self.xattr, self.length)
    }
    fn data_offset(&self, volume: &Volume) -> Result<u64, ScanError> {
        (u64::from(self.lbn) + u64::from(self.xattr))
            .checked_mul(volume.block)
            .ok_or(ScanError::Invalid)
    }
    fn mode(&self) -> Option<u32> {
        self.rr.px.as_ref().map(|px| px[0] & 0o170000)
    }
    fn link(&self) -> bool {
        self.rr.symlink
            || self.mode() == Some(0o120000)
            || (!self.directory() && self.rr.px.as_ref().is_some_and(|px| px[1] > 1))
    }
    fn special(&self) -> bool {
        self.mode()
            .is_some_and(|mode| !matches!(mode, 0o100000 | 0o040000 | 0o120000))
    }
    fn modified(&self) -> u32 {
        self.rr.modified.unwrap_or(self.time)
    }
}

#[derive(Clone)]
struct Volume {
    block: u64,
    bytes: u64,
    sequence: u16,
    joliet: bool,
    root: FileRecord,
}

#[derive(Default)]
struct Susp {
    skip: usize,
    root_offset: usize,
    // SUSP §5.6 assigns 1-based sequence numbers in ER recording order.
    extensions: Vec<bool>,
    active: bool,
}

struct Directory {
    dot: FileRecord,
    parent: FileRecord,
    children: Vec<FileRecord>,
}

struct ParseBudget {
    records: usize,
    names: usize,
    limits: ScanLimits,
}

impl ParseBudget {
    fn record(&mut self, record: &FileRecord) -> Result<(), ScanError> {
        self.records = self.records.checked_add(1).ok_or(ScanError::Limit)?;
        self.names = self
            .names
            .checked_add(record.identifier.len())
            .and_then(|n| n.checked_add(record.rr.name.as_ref().map_or(0, Vec::len)))
            .ok_or(ScanError::Limit)?;
        if self.records > self.limits.entries.min(xdzip_codec_api::MAX_ENTRIES)
            || self.names > self.limits.total_name_bytes
        {
            return Err(ScanError::Limit);
        }
        Ok(())
    }
}

pub(crate) fn scan(reader: &mut ImageReader<'_>, limits: ScanLimits) -> Result<Index, ScanError> {
    let (primary, joliet, catalogs) = descriptors(reader)?;
    let primary_susp = detect_susp(reader, &primary)?;
    // Rock Ridge carries POSIX spelling and object types. Prefer it when declared;
    // otherwise the best recognized Unicode descriptor is the effective namespace.
    let (volume, susp) = if primary_susp.extensions.contains(&true) {
        (primary, primary_susp)
    } else if let Some(volume) = joliet {
        let susp = detect_susp(reader, &volume)?;
        (volume, susp)
    } else {
        (primary, primary_susp)
    };
    let mut budget = ParseBudget {
        records: 0,
        names: 0,
        limits,
    };
    let directories = physical_directories(reader, &volume, &susp, &mut budget)?;
    validate_relocations(&directories)?;
    let mut builder = IndexBuilder::new(Format::Iso, limits);
    emit_namespace(reader, &volume, &directories, &mut builder, limits)?;
    emit_boot_images(reader, &volume, &catalogs, &directories, &mut builder)?;
    Ok(builder.finish())
}

fn both16(bytes: &[u8]) -> Result<u16, ScanError> {
    let bytes: &[u8; 4] = bytes.try_into().map_err(|_| ScanError::Invalid)?;
    let little = u16::from_le_bytes([bytes[0], bytes[1]]);
    if little != u16::from_be_bytes([bytes[2], bytes[3]]) {
        return Err(ScanError::Invalid);
    }
    Ok(little)
}

fn both32(bytes: &[u8]) -> Result<u32, ScanError> {
    let bytes: &[u8; 8] = bytes.try_into().map_err(|_| ScanError::Invalid)?;
    let little = u32::from_le_bytes(bytes[..4].try_into().map_err(|_| ScanError::Invalid)?);
    if little != u32::from_be_bytes(bytes[4..].try_into().map_err(|_| ScanError::Invalid)?) {
        return Err(ScanError::Invalid);
    }
    Ok(little)
}

fn little32(bytes: &[u8]) -> Result<u32, ScanError> {
    Ok(u32::from_le_bytes(
        bytes.try_into().map_err(|_| ScanError::Invalid)?,
    ))
}

fn range(volume: &Volume, offset: u64, length: u64) -> Result<(), ScanError> {
    if offset
        .checked_add(length)
        .is_none_or(|end| end > volume.bytes)
    {
        return Err(ScanError::Invalid);
    }
    Ok(())
}

fn descriptors(
    reader: &mut ImageReader<'_>,
) -> Result<(Volume, Option<Volume>, Vec<u32>), ScanError> {
    let mut primary = None;
    let mut joliet = None;
    let mut joliet_level = 0;
    let mut catalogs = Vec::new();
    for sector in 16..272u64 {
        let bytes = reader.read_at(sector * SECTOR, SECTOR as usize)?;
        if &bytes[1..6] != b"CD001" {
            return Err(ScanError::Invalid);
        }
        if bytes[6] != 1 {
            return Err(ScanError::Unsupported);
        }
        match bytes[0] {
            0 => {
                if bytes[7..39].starts_with(b"EL TORITO SPECIFICATION") {
                    let lbn = little32(&bytes[71..75])?;
                    if lbn == 0 {
                        return Err(ScanError::Invalid);
                    }
                    if !catalogs.contains(&lbn) {
                        catalogs.push(lbn);
                    }
                }
            }
            1 => {
                let volume = volume_descriptor(&bytes, reader.len(), false)?;
                if primary.as_ref().is_some_and(|previous: &Volume| {
                    previous.block != volume.block
                        || previous.bytes != volume.bytes
                        || previous.root.key() != volume.root.key()
                }) {
                    return Err(ScanError::Invalid);
                }
                primary = Some(volume);
            }
            2 => {
                let level = match &bytes[88..91] {
                    b"%/@" => 1,
                    b"%/C" => 2,
                    b"%/E" => 3,
                    _ => 0,
                };
                if level > 0 {
                    let volume = volume_descriptor(&bytes, reader.len(), true)?;
                    if level > joliet_level {
                        joliet = Some(volume);
                        joliet_level = level;
                    } else if level == joliet_level
                        && joliet.as_ref().is_some_and(|old: &Volume| {
                            old.block != volume.block
                                || old.bytes != volume.bytes
                                || old.root.key() != volume.root.key()
                        })
                    {
                        return Err(ScanError::Invalid);
                    }
                }
            }
            3 => {
                // The partition describes application-defined bytes, not another
                // hierarchy. Validate its numeric representation, then skip it.
                let _location = both32(&bytes[72..80])?;
                let _length = both32(&bytes[80..88])?;
            }
            255 => return Ok((primary.ok_or(ScanError::Invalid)?, joliet, catalogs)),
            _ => return Err(ScanError::Unsupported),
        }
    }
    Err(ScanError::Limit)
}

fn volume_descriptor(bytes: &[u8], image_size: u64, joliet: bool) -> Result<Volume, ScanError> {
    let blocks = u64::from(both32(&bytes[80..88])?);
    let set_size = both16(&bytes[120..124])?;
    let sequence = both16(&bytes[124..128])?;
    let block = u64::from(both16(&bytes[128..132])?);
    let _path_table_size = both32(&bytes[132..140])?;
    if !matches!(block, 512 | 1024 | 2048) || bytes[881] != 1 {
        return Err(ScanError::Unsupported);
    }
    if set_size == 0 || sequence == 0 || sequence > set_size || blocks == 0 {
        return Err(ScanError::Invalid);
    }
    if set_size != 1 || sequence != 1 {
        return Err(ScanError::Unsupported);
    }
    let length = blocks.checked_mul(block).ok_or(ScanError::Invalid)?;
    if length > image_size || length < 18 * SECTOR {
        return Err(ScanError::Invalid);
    }
    let mut volume = Volume {
        block,
        bytes: length,
        sequence,
        joliet,
        root: FileRecord {
            identifier: Vec::new(),
            lbn: 0,
            xattr: 0,
            length: 0,
            flags: 0,
            time: 0,
            system: Vec::new(),
            rr: RockRidge::default(),
            extents: Vec::new(),
        },
    };
    let root_len = usize::from(bytes[156]);
    if root_len != 34 {
        return Err(ScanError::Invalid);
    }
    volume.root = directory_record(&bytes[156..156 + root_len], &volume)?;
    if !volume.root.directory() || volume.root.identifier != [0] {
        return Err(ScanError::Invalid);
    }
    Ok(volume)
}

fn directory_record(bytes: &[u8], volume: &Volume) -> Result<FileRecord, ScanError> {
    if bytes.len() < 34 || usize::from(bytes[0]) != bytes.len() {
        return Err(ScanError::Invalid);
    }
    let name_len = usize::from(bytes[32]);
    let system_start = 33 + name_len + usize::from(name_len.is_multiple_of(2));
    if name_len == 0
        || system_start > bytes.len()
        || (name_len.is_multiple_of(2) && bytes[33 + name_len] != 0)
    {
        return Err(ScanError::Invalid);
    }
    let lbn = both32(&bytes[2..10])?;
    let length = u64::from(both32(&bytes[10..18])?);
    let sequence = both16(&bytes[28..32])?;
    if sequence != volume.sequence {
        return Err(ScanError::Unsupported);
    }
    if bytes[25] & 0x60 != 0 {
        return Err(ScanError::Invalid);
    }
    if bytes[26] != 0 || bytes[27] != 0 {
        return Err(ScanError::Unsupported);
    }
    if bytes[25] & 2 != 0 && bytes[25] & (4 | 8 | 128) != 0 {
        return Err(ScanError::Invalid);
    }
    if bytes[1] == 0 && bytes[25] & (8 | 16) != 0 {
        return Err(ScanError::Invalid);
    }
    // Record-organized payloads require interpreting the extended-attribute
    // record format. Raw bytes cannot silently stand in for their logical data.
    if bytes[25] & 8 != 0 {
        return Err(ScanError::Unsupported);
    }
    let offset = (u64::from(lbn) + u64::from(bytes[1])) * volume.block;
    range(
        volume,
        u64::from(lbn) * volume.block,
        u64::from(bytes[1]) * volume.block,
    )?;
    range(volume, offset, length)?;
    if bytes[25] & 2 != 0
        && (!offset.is_multiple_of(SECTOR) || length == 0 || !length.is_multiple_of(SECTOR))
    {
        return Err(ScanError::Invalid);
    }
    Ok(FileRecord {
        identifier: bytes[33..33 + name_len].to_vec(),
        lbn,
        xattr: bytes[1],
        length,
        flags: bytes[25],
        time: short_time(&bytes[18..25]),
        system: bytes[system_start..].to_vec(),
        rr: RockRidge::default(),
        extents: if length == 0 || bytes[25] & 2 != 0 {
            Vec::new()
        } else {
            vec![Extent::Data { offset, length }]
        },
    })
}

fn root_dot(reader: &mut ImageReader<'_>, volume: &Volume) -> Result<FileRecord, ScanError> {
    let bytes = reader.read_at(volume.root.data_offset(volume)?, SECTOR as usize)?;
    let length = usize::from(bytes[0]);
    let dot = directory_record(bytes.get(..length).ok_or(ScanError::Invalid)?, volume)?;
    if dot.identifier != [0] || dot.key() != volume.root.key() || !dot.directory() {
        return Err(ScanError::Invalid);
    }
    Ok(dot)
}

fn extended_attributes(
    reader: &mut ImageReader<'_>,
    volume: &Volume,
    record: &mut FileRecord,
) -> Result<(), ScanError> {
    if record.xattr == 0 {
        return Ok(());
    }
    // ECMA-119 §10.5: XAR precedes the file data. Its own fixed-width fields
    // retain both-endian checks, and its modification timestamp is more precise
    // than the directory record's recording timestamp. No permissions are applied.
    let bytes = reader.read_at(u64::from(record.lbn) * volume.block, 250)?;
    let owner = both16(&bytes[..4])?;
    let group = both16(&bytes[4..8])?;
    let record_length = both16(&bytes[80..84])?;
    let application_length = u64::from(both16(&bytes[246..250])?);
    if (owner == 0) != (group == 0)
        || bytes[182..246].iter().any(|byte| *byte != 0)
        || 250 + application_length + u64::from(bytes[181]) > u64::from(record.xattr) * volume.block
    {
        return Err(ScanError::Invalid);
    }
    if bytes[180] != 1 || bytes[78] != 0 {
        return Err(ScanError::Unsupported);
    }
    if record_length != 0 {
        return Err(ScanError::Invalid);
    }
    let modified = long_time(&bytes[27..44]);
    if modified != 0 {
        record.time = modified;
    }
    Ok(())
}

fn detect_susp(reader: &mut ImageReader<'_>, volume: &Volume) -> Result<Susp, ScanError> {
    let dot = root_dot(reader, volume)?;
    let mut susp = Susp::default();
    // SUSP §5.3 permits SP at byte 1, or byte 15 for CD-ROM XA.
    for offset in [0, 14] {
        if dot.system.get(offset..offset + 2) == Some(b"SP") {
            let sp = dot
                .system
                .get(offset..offset + 7)
                .ok_or(ScanError::Invalid)?;
            if sp[2..6] != [7, 1, 0xbe, 0xef] {
                return Err(ScanError::Invalid);
            }
            susp.skip = usize::from(sp[6]);
            susp.root_offset = offset;
            susp.active = true;
            break;
        }
    }
    if susp.active {
        let entries = system_entries(reader, volume, &dot.system[susp.root_offset..])?;
        for entry in entries.iter().filter(|entry| &entry[..2] == b"ER") {
            if entry[3] != 1 || entry.len() < 8 {
                return Err(ScanError::Invalid);
            }
            let id_len = usize::from(entry[4]);
            let length = 8 + id_len + usize::from(entry[5]) + usize::from(entry[6]);
            if length != entry.len() || id_len == 0 {
                return Err(ScanError::Invalid);
            }
            let id = &entry[8..8 + id_len];
            let recognized = matches!(id, b"RRIP_1991A" | b"IEEE_P1282" | b"IEEE_1282");
            if recognized && entry[7] != 1 {
                return Err(ScanError::Unsupported);
            }
            susp.extensions.push(recognized);
            if susp.extensions.len() > 255 {
                return Err(ScanError::Limit);
            }
        }
    }
    Ok(susp)
}

fn system_entries(
    reader: &mut ImageReader<'_>,
    volume: &Volume,
    initial: &[u8],
) -> Result<Vec<Vec<u8>>, ScanError> {
    let mut area = initial.to_vec();
    let mut entries = Vec::new();
    let mut visited = BTreeSet::new();
    let mut total = area.len();
    loop {
        let mut offset = 0usize;
        let mut continuation = None;
        while area.len() - offset >= 4 {
            reader.check()?;
            let length = usize::from(area[offset + 2]);
            if length < 4 || length > area.len() - offset {
                return Err(ScanError::Invalid);
            }
            let entry = &area[offset..offset + length];
            match &entry[..2] {
                b"CE" => {
                    if continuation.is_some() || length != 28 || entry[3] != 1 {
                        return Err(ScanError::Invalid);
                    }
                    let lbn = u64::from(both32(&entry[4..12])?);
                    let start = u64::from(both32(&entry[12..20])?);
                    let count = both32(&entry[20..28])? as usize;
                    let physical = lbn
                        .checked_mul(volume.block)
                        .and_then(|n| n.checked_add(start))
                        .ok_or(ScanError::Invalid)?;
                    range(volume, physical, count as u64)?;
                    if count == 0 || !visited.insert((physical, count)) {
                        return Err(ScanError::Invalid);
                    }
                    continuation = Some((physical, count));
                }
                b"ST" => {
                    if length != 4 || entry[3] != 1 {
                        return Err(ScanError::Invalid);
                    }
                    break;
                }
                _ => entries.push(entry.to_vec()),
            }
            if entries.len() > MAX_CE_BYTES / 4 {
                return Err(ScanError::Limit);
            }
            offset += length;
        }
        let Some((physical, count)) = continuation else {
            break;
        };
        total = total.checked_add(count).ok_or(ScanError::Limit)?;
        if total > MAX_CE_BYTES || visited.len() > MAX_CE_AREAS {
            return Err(ScanError::Limit);
        }
        // CE is processed after all entries in the current area, not at its
        // physical position. This preserves NM/SL concatenation order (§5.1).
        area = reader.read_at(physical, count)?;
    }
    Ok(entries)
}

fn rock_ridge(
    reader: &mut ImageReader<'_>,
    volume: &Volume,
    susp: &Susp,
    record: &FileRecord,
    root: bool,
    name_limit: usize,
) -> Result<RockRidge, ScanError> {
    let mut rr = RockRidge::default();
    if !susp.active {
        return Ok(rr);
    }
    let skip = if root { susp.root_offset } else { susp.skip };
    let entries = system_entries(
        reader,
        volume,
        record.system.get(skip..).unwrap_or_default(),
    )?;
    let mut selected = susp
        .extensions
        .iter()
        .position(|recognized| *recognized)
        .map(|n| n + 1);
    if susp.extensions.len() > 1 {
        selected = None;
    }
    let mut nm_more = false;
    let mut sl_more = false;
    let mut sl_component_more = false;
    let mut sl_bytes = 0usize;
    let mut seen = BTreeSet::new();
    for entry in entries {
        let signature = &entry[..2];
        if matches!(signature, b"SP" | b"PD" | b"ER" | b"ES") {
            if entry[3] != 1 {
                return Err(ScanError::Unsupported);
            }
            match signature {
                b"SP" => {
                    if !root
                        || entry.len() != 7
                        || entry[4..6] != [0xbe, 0xef]
                        || usize::from(entry[6]) != susp.skip
                        || !seen.insert(*b"SP")
                    {
                        return Err(ScanError::Invalid);
                    }
                }
                b"ES" => {
                    if entry.len() != 5
                        || entry[4] == 0
                        || usize::from(entry[4]) > susp.extensions.len()
                    {
                        return Err(ScanError::Invalid);
                    }
                    selected = Some(usize::from(entry[4]));
                }
                b"ER"
                    if !root
                        || entry.len() < 8
                        || entry.len()
                            != 8 + usize::from(entry[4])
                                + usize::from(entry[5])
                                + usize::from(entry[6]) =>
                {
                    return Err(ScanError::Invalid);
                }
                _ => (),
            }
            continue;
        }
        if !selected.is_some_and(|sequence| susp.extensions[sequence - 1]) {
            continue;
        }
        let known = matches!(
            signature,
            b"PX" | b"PN" | b"SL" | b"NM" | b"CL" | b"PL" | b"RE" | b"TF" | b"SF" | b"ZF" | b"RR"
        );
        // SUSP §6.2 requires skipping unrecognized entries. Known encodings that
        // change file bytes are rejected rather than exposing encoded payloads.
        if !known {
            continue;
        }
        if entry[3] != 1 {
            return Err(ScanError::Unsupported);
        }
        if !matches!(signature, b"NM" | b"SL") && !seen.insert([entry[0], entry[1]]) {
            return Err(ScanError::Invalid);
        }
        match signature {
            b"PX" => {
                if !matches!(entry.len(), 36 | 44) {
                    return Err(ScanError::Invalid);
                }
                let fields = entry[4..]
                    .as_chunks::<8>()
                    .0
                    .iter()
                    .map(|field| both32(field))
                    .collect::<Result<Vec<_>, _>>()?;
                if fields[1] == 0 {
                    return Err(ScanError::Invalid);
                }
                rr.px = Some(fields);
            }
            b"PN" => {
                if entry.len() != 20 {
                    return Err(ScanError::Invalid);
                }
                both32(&entry[4..12])?;
                both32(&entry[12..20])?;
            }
            b"NM" => {
                if entry.len() < 5 || (rr.name.is_some() && !nm_more) {
                    return Err(ScanError::Invalid);
                }
                let flags = entry[4];
                if flags & !7 != 0 {
                    return Err(ScanError::Unsupported);
                }
                if (flags & 7).count_ones() > 1 {
                    return Err(ScanError::Invalid);
                }
                if flags & 6 != 0 {
                    if entry.len() != 5
                        || rr.name.is_some()
                        || (flags == 2 && record.identifier != [0])
                        || (flags == 4 && record.identifier != [1])
                    {
                        return Err(ScanError::Invalid);
                    }
                    rr.name = Some(if flags == 2 {
                        b".".to_vec()
                    } else {
                        b"..".to_vec()
                    });
                } else {
                    let name = rr.name.get_or_insert_with(Vec::new);
                    if name
                        .len()
                        .checked_add(entry.len() - 5)
                        .is_none_or(|n| n > name_limit)
                    {
                        return Err(ScanError::Limit);
                    }
                    name.extend_from_slice(&entry[5..]);
                }
                nm_more = flags & 1 != 0;
            }
            b"SL" => {
                if entry.len() < 5 || entry[4] & !1 != 0 || (rr.symlink && !sl_more) {
                    return Err(ScanError::Invalid);
                }
                rr.symlink = true;
                let mut offset = 5;
                while offset < entry.len() {
                    let component = entry.get(offset..offset + 2).ok_or(ScanError::Invalid)?;
                    let flags = component[0];
                    let length = usize::from(component[1]);
                    if flags & !15 != 0 {
                        return Err(ScanError::Unsupported);
                    }
                    if (flags & 15).count_ones() > 1
                        || (flags & 14 != 0 && length != 0)
                        || (sl_component_more && flags & 14 != 0)
                    {
                        return Err(ScanError::Invalid);
                    }
                    offset = offset.checked_add(2 + length).ok_or(ScanError::Invalid)?;
                    if offset > entry.len() {
                        return Err(ScanError::Invalid);
                    }
                    sl_bytes = sl_bytes.checked_add(length + 1).ok_or(ScanError::Limit)?;
                    if sl_bytes > name_limit {
                        return Err(ScanError::Limit);
                    }
                    sl_component_more = flags & 1 != 0;
                }
                sl_more = entry[4] & 1 != 0;
                if sl_component_more && !sl_more {
                    return Err(ScanError::Invalid);
                }
            }
            b"CL" | b"PL" => {
                if entry.len() != 12 {
                    return Err(ScanError::Invalid);
                }
                let lbn = both32(&entry[4..12])?;
                range(volume, u64::from(lbn) * volume.block, SECTOR)?;
                if signature == b"CL" {
                    rr.child = Some(lbn);
                } else {
                    rr.parent = Some(lbn);
                }
            }
            b"RE" => {
                if entry.len() != 4 {
                    return Err(ScanError::Invalid);
                }
                rr.relocated = true;
            }
            b"TF" => {
                if entry.len() < 5 {
                    return Err(ScanError::Invalid);
                }
                let flags = entry[4];
                let width = if flags & 128 != 0 { 17 } else { 7 };
                if entry.len() != 5 + (flags & 127).count_ones() as usize * width {
                    return Err(ScanError::Invalid);
                }
                let mut offset = 5;
                for bit in 0..7 {
                    if flags & (1 << bit) != 0 {
                        let bytes = &entry[offset..offset + width];
                        if bit == 1 {
                            rr.modified = Some(if width == 7 {
                                short_time(bytes)
                            } else {
                                long_time(bytes)
                            });
                        }
                        offset += width;
                    }
                }
            }
            b"SF" | b"ZF" => return Err(ScanError::Unsupported),
            b"RR" if entry.len() != 5 => return Err(ScanError::Invalid),
            _ => (),
        }
    }
    if nm_more || sl_more || sl_component_more {
        return Err(ScanError::Invalid);
    }
    if rr.name.as_ref().is_some_and(Vec::is_empty) {
        return Err(ScanError::Invalid);
    }
    if rr.child.is_some() && (rr.relocated || rr.parent.is_some()) {
        return Err(ScanError::Invalid);
    }
    if rr.parent.is_some() && record.identifier != [1] {
        return Err(ScanError::Invalid);
    }
    if rr.child.is_some() && matches!(record.identifier.as_slice(), [0] | [1]) {
        return Err(ScanError::Invalid);
    }
    if rr.child.is_none() {
        let mode = rr.px.as_ref().map(|px| px[0] & 0o170000);
        if mode.is_some_and(|mode| (mode == 0o040000) != record.directory())
            || (rr.symlink && mode.is_some_and(|mode| mode != 0o120000))
            || (mode == Some(0o120000) && !rr.symlink)
        {
            return Err(ScanError::Invalid);
        }
    }
    Ok(rr)
}

type DirectoryKey = (u32, u8, u64);

fn read_directory(
    reader: &mut ImageReader<'_>,
    volume: &Volume,
    susp: &Susp,
    record: &FileRecord,
    budget: &mut ParseBudget,
    root: bool,
) -> Result<Directory, ScanError> {
    let mut dot = None;
    let mut parent = None;
    let mut children = Vec::new();
    let mut pending: Option<FileRecord> = None;
    let mut physical_records = 0usize;
    let start = record.data_offset(volume)?;
    for sector in 0..record.length / SECTOR {
        let bytes = reader.read_at(start + sector * SECTOR, SECTOR as usize)?;
        let mut offset = 0;
        while offset < bytes.len() {
            let length = usize::from(bytes[offset]);
            if length == 0 {
                if bytes[offset..].iter().any(|byte| *byte != 0) {
                    return Err(ScanError::Invalid);
                }
                break;
            }
            let raw = bytes
                .get(offset..offset + length)
                .ok_or(ScanError::Invalid)?;
            let mut child = directory_record(raw, volume)?;
            extended_attributes(reader, volume, &mut child)?;
            if child.flags & 128 == 0 {
                child.rr = rock_ridge(
                    reader,
                    volume,
                    susp,
                    &child,
                    root && physical_records == 0,
                    budget.limits.name_bytes.min(4096),
                )?;
            } else if susp.active {
                // SUSP §4 assigns semantics to the final section only. Earlier
                // areas still need valid framing and bounded continuations.
                system_entries(
                    reader,
                    volume,
                    child.system.get(susp.skip..).unwrap_or_default(),
                )?;
            }
            physical_records += 1;
            if physical_records > 1_000_000 {
                return Err(ScanError::Limit);
            }
            offset += length;
            if physical_records == 1 {
                if child.identifier != [0]
                    || !child.directory()
                    || child.key() != record.key()
                    || child.rr.child.is_some()
                    || child.rr.relocated
                {
                    return Err(ScanError::Invalid);
                }
                dot = Some(child);
                continue;
            }
            if physical_records == 2 {
                if child.identifier != [1]
                    || !child.directory()
                    || child.rr.child.is_some()
                    || child.rr.relocated
                {
                    return Err(ScanError::Invalid);
                }
                parent = Some(child);
                continue;
            }
            if matches!(child.identifier.as_slice(), [0] | [1]) {
                return Err(ScanError::Invalid);
            }
            let continued = child.flags & 128 != 0;
            if continued && (child.length == 0 || !child.length.is_multiple_of(volume.block)) {
                return Err(ScanError::Invalid);
            }
            if let Some(mut previous) = pending.take() {
                if previous.identifier != child.identifier
                    || previous.flags & 0x6f != child.flags & 0x6f
                    || previous.directory()
                    || child.rr.child.is_some()
                    || child.rr.relocated
                {
                    return Err(ScanError::Invalid);
                }
                previous.length = previous
                    .length
                    .checked_add(child.length)
                    .ok_or(ScanError::Invalid)?;
                previous.extents.append(&mut child.extents);
                if previous.extents.len() > 1_000_000 {
                    return Err(ScanError::Limit);
                }
                // SUSP §4: only the final section's system-use attributes apply.
                previous.rr = child.rr;
                previous.time = child.time;
                child = previous;
            }
            if continued {
                pending = Some(child);
            } else {
                budget.record(&child)?;
                children.push(child);
            }
        }
    }
    if pending.is_some() {
        return Err(ScanError::Invalid);
    }
    Ok(Directory {
        dot: dot.ok_or(ScanError::Invalid)?,
        parent: parent.ok_or(ScanError::Invalid)?,
        children,
    })
}

fn physical_directories(
    reader: &mut ImageReader<'_>,
    volume: &Volume,
    susp: &Susp,
    budget: &mut ParseBudget,
) -> Result<BTreeMap<DirectoryKey, Directory>, ScanError> {
    enum Visit {
        Enter(FileRecord, usize),
        Leave(DirectoryKey),
    }
    let mut tasks = vec![Visit::Enter(volume.root.clone(), 0)];
    let mut active = BTreeSet::new();
    let mut locations = BTreeMap::new();
    let mut directories = BTreeMap::new();
    let mut serials = BTreeMap::<u32, (Vec<u32>, Option<u32>)>::new();
    while let Some(task) = tasks.pop() {
        reader.check()?;
        let (record, depth) = match task {
            Visit::Leave(key) => {
                active.remove(&key);
                continue;
            }
            Visit::Enter(record, depth) => (record, depth),
        };
        if depth > MAX_DEPTH {
            return Err(ScanError::Limit);
        }
        let key = record.key();
        if active.contains(&key) {
            return Err(ScanError::Invalid);
        }
        if directories.contains_key(&key) {
            continue;
        }
        if locations
            .insert(record.lbn, key)
            .is_some_and(|previous| previous != key)
        {
            return Err(ScanError::Invalid);
        }
        let directory = read_directory(reader, volume, susp, &record, budget, depth == 0)?;
        for file in std::iter::once(&directory.dot)
            .chain(std::iter::once(&directory.parent))
            .chain(&directory.children)
        {
            if let Some(px) = &file.rr.px
                && px.len() == 5
                && px[4] != 0
            {
                let metadata = (px.clone(), file.rr.modified);
                if serials
                    .insert(px[4], metadata.clone())
                    .is_some_and(|previous| previous != metadata)
                {
                    return Err(ScanError::Invalid);
                }
            }
        }
        active.insert(key);
        tasks.push(Visit::Leave(key));
        for child in directory
            .children
            .iter()
            .rev()
            .filter(|child| child.directory() && child.rr.child.is_none())
        {
            tasks.push(Visit::Enter(child.clone(), depth + 1));
        }
        directories.insert(key, directory);
    }
    Ok(directories)
}

fn validate_relocations(directories: &BTreeMap<DirectoryKey, Directory>) -> Result<(), ScanError> {
    let mut moved = BTreeMap::new();
    let mut links = BTreeMap::new();
    for (key, directory) in directories {
        for child in &directory.children {
            if child.rr.relocated
                && (!child.directory() || moved.insert(child.lbn, child.key()).is_some())
            {
                return Err(ScanError::Invalid);
            }
            if let Some(target) = child.rr.child
                && links.insert(target, key.0).is_some()
            {
                return Err(ScanError::Invalid);
            }
        }
    }
    if links.len() != moved.len() {
        return Err(ScanError::Invalid);
    }
    for (lbn, key) in &moved {
        let original_parent = links.get(lbn).ok_or(ScanError::Invalid)?;
        let directory = directories.get(key).ok_or(ScanError::Invalid)?;
        if directory.parent.rr.parent.as_ref() != Some(original_parent) {
            return Err(ScanError::Invalid);
        }
    }
    let moved_keys: BTreeSet<_> = moved.values().copied().collect();
    for (key, directory) in directories {
        if directory.parent.rr.parent.is_some() && !moved_keys.contains(key) {
            return Err(ScanError::Invalid);
        }
    }
    Ok(())
}

fn name(record: &FileRecord, volume: &Volume) -> Result<Vec<u8>, ScanError> {
    if let Some(name) = &record.rr.name {
        if volume.joliet {
            return ucs2(name);
        }
        return Ok(name.clone());
    }
    let mut name = if volume.joliet {
        ucs2(&record.identifier)?
    } else {
        record.identifier.clone()
    };
    if !record.directory() {
        if let Some(separator) = name.iter().rposition(|byte| *byte == b';') {
            let version = &name[separator + 1..];
            if !version.is_empty() && version.iter().all(u8::is_ascii_digit) {
                name.truncate(separator);
            }
        }
        if name.last() == Some(&b'.') {
            name.pop();
        }
    }
    if name.is_empty() {
        return Err(ScanError::Invalid);
    }
    Ok(name)
}

fn ucs2(bytes: &[u8]) -> Result<Vec<u8>, ScanError> {
    if !bytes.len().is_multiple_of(2) {
        return Err(ScanError::Invalid);
    }
    let mut text = String::new();
    for pair in bytes.as_chunks::<2>().0 {
        let value = u32::from(u16::from_be_bytes([pair[0], pair[1]]));
        // UCS-2 never contains surrogate pairs. Lossy replacement would change
        // identity and can turn distinct on-disc names into one output name.
        text.push(char::from_u32(value).ok_or(ScanError::Invalid)?);
    }
    Ok(text.into_bytes())
}

fn emit_namespace(
    reader: &ImageReader<'_>,
    volume: &Volume,
    directories: &BTreeMap<DirectoryKey, Directory>,
    builder: &mut IndexBuilder,
    limits: ScanLimits,
) -> Result<(), ScanError> {
    struct Frame {
        key: DirectoryKey,
        path: Vec<u8>,
        next: usize,
        unsafe_component: bool,
    }
    let mut stack = vec![Frame {
        key: volume.root.key(),
        path: Vec::new(),
        next: 0,
        unsafe_component: false,
    }];
    let relocated: BTreeMap<_, _> = directories
        .values()
        .flat_map(|directory| &directory.children)
        .filter(|file| file.rr.relocated)
        .map(|file| (file.lbn, file.key()))
        .collect();
    let mut active = BTreeSet::from([volume.root.key()]);
    while let Some(frame) = stack.last_mut() {
        reader.check()?;
        let directory = directories.get(&frame.key).ok_or(ScanError::Invalid)?;
        let Some(child) = directory.children.get(frame.next) else {
            active.remove(&frame.key);
            stack.pop();
            continue;
        };
        frame.next += 1;
        if child.rr.relocated {
            continue;
        }
        let target = child
            .rr
            .child
            .map(|lbn| relocated.get(&lbn).copied().ok_or(ScanError::Invalid))
            .transpose()?;
        let effective = match target {
            Some(key) => &directories.get(&key).ok_or(ScanError::Invalid)?.dot,
            None => child,
        };
        let is_directory = effective.directory();
        let component = name(child, volume)?;
        let unsafe_component = frame.unsafe_component
            || component.contains(&b'/')
            || component.contains(&0)
            || component == b"."
            || component == b"..";
        let full_len = frame
            .path
            .len()
            .checked_add(component.len())
            .and_then(|n| n.checked_add(usize::from(is_directory)))
            .ok_or(ScanError::Limit)?;
        if full_len > limits.name_bytes.min(4096) {
            return Err(ScanError::Limit);
        }
        let mut path = frame.path.clone();
        path.extend_from_slice(&component);
        if is_directory {
            path.push(b'/');
        }
        let node = Node {
            name: path.clone(),
            directory: is_directory,
            link: effective.link(),
            special: effective.special() || unsafe_component,
            size: if is_directory { 0 } else { effective.length },
            modified_dos: effective.modified(),
            extents: if is_directory || effective.link() || effective.special() {
                Vec::new()
            } else {
                effective.extents.clone()
            },
        };
        if volume.joliet {
            builder.push(node)?;
        } else {
            builder.push_unlabelled(node)?;
        }
        if is_directory {
            let key = target.unwrap_or_else(|| child.key());
            if !active.insert(key) {
                return Err(ScanError::Invalid);
            }
            if stack.len() >= MAX_DEPTH {
                return Err(ScanError::Limit);
            }
            stack.push(Frame {
                key,
                path,
                next: 0,
                unsafe_component,
            });
        }
    }
    Ok(())
}

// El Torito v1.0, Figures 2–6. Catalog entries authorize reading bytes only.
fn emit_boot_images(
    reader: &mut ImageReader<'_>,
    volume: &Volume,
    catalogs: &[u32],
    directories: &BTreeMap<DirectoryKey, Directory>,
    builder: &mut IndexBuilder,
) -> Result<(), ScanError> {
    let mut images = Vec::new();
    let mut file_sizes = BTreeMap::new();
    for file in directories
        .values()
        .flat_map(|directory| &directory.children)
    {
        if !file.directory()
            && !file.link()
            && !file.special()
            && let [Extent::Data { offset, .. }] = file.extents.as_slice()
        {
            let size = file_sizes.entry(*offset).or_insert(0u64);
            *size = (*size).max(file.length);
        }
    }
    for lbn in catalogs {
        let base = u64::from(*lbn) * SECTOR;
        range(volume, base, 64)?;
        let validation = reader.read_at(base, 32)?;
        let checksum = validation
            .as_chunks::<2>()
            .0
            .iter()
            .fold(0u16, |sum, word| {
                sum.wrapping_add(u16::from_le_bytes([word[0], word[1]]))
            });
        if validation[0] != 1
            || validation[2..4] != [0, 0]
            || validation[30..32] != [0x55, 0xaa]
            || checksum != 0
        {
            return Err(ScanError::Invalid);
        }
        let initial = reader.read_at(base + 32, 32)?;
        boot_entry(reader, volume, &initial, true, &file_sizes, &mut images)?;
        let mut slot = 2u64;
        let mut another_header = false;
        loop {
            if slot > 32768 {
                return Err(ScanError::Limit);
            }
            range(volume, base + slot * 32, 32)?;
            let header = reader.read_at(base + slot * 32, 32)?;
            if header.iter().all(|byte| *byte == 0) {
                if another_header {
                    return Err(ScanError::Invalid);
                }
                break;
            }
            if !matches!(header[0], 0x90 | 0x91) {
                return Err(ScanError::Invalid);
            }
            let final_section = header[0] == 0x91;
            let count = u16::from_le_bytes([header[2], header[3]]);
            if count == 0 {
                return Err(ScanError::Invalid);
            }
            slot += 1;
            let end = slot
                .checked_add(u64::from(count))
                .ok_or(ScanError::Invalid)?;
            if end > 32768 {
                return Err(ScanError::Limit);
            }
            while slot < end {
                range(volume, base + slot * 32, 32)?;
                let entry = reader.read_at(base + slot * 32, 32)?;
                boot_entry(reader, volume, &entry, false, &file_sizes, &mut images)?;
                slot += 1;
                let mut continuation = entry[1] & 32 != 0;
                while continuation {
                    if slot >= end {
                        return Err(ScanError::Invalid);
                    }
                    let extension = reader.read_at(base + slot * 32, 32)?;
                    if extension[0] != 0x44 || extension[1] & !32 != 0 {
                        return Err(ScanError::Invalid);
                    }
                    continuation = extension[1] & 32 != 0;
                    slot += 1;
                }
            }
            if final_section {
                break;
            }
            another_header = true;
        }
    }
    if images.is_empty() {
        return Ok(());
    }
    builder.push(Node {
        name: b"[BOOT]/".to_vec(),
        directory: true,
        link: false,
        special: false,
        size: 0,
        modified_dos: 0,
        extents: Vec::new(),
    })?;
    let multiple = images.len() > 1;
    for (index, (label, offset, length)) in images.into_iter().enumerate() {
        let name = if multiple {
            format!("[BOOT]/{}-{label}.img", index + 1)
        } else {
            format!("[BOOT]/{label}.img")
        };
        builder.push(Node {
            name: name.into_bytes(),
            directory: false,
            link: false,
            special: false,
            size: length,
            modified_dos: 0,
            extents: if length == 0 {
                Vec::new()
            } else {
                vec![Extent::Data { offset, length }]
            },
        })?;
    }
    Ok(())
}

type BootImage = (&'static str, u64, u64);

fn boot_entry(
    reader: &mut ImageReader<'_>,
    volume: &Volume,
    entry: &[u8],
    initial: bool,
    file_sizes: &BTreeMap<u64, u64>,
    images: &mut Vec<BootImage>,
) -> Result<(), ScanError> {
    if !matches!(entry[0], 0 | 0x88)
        || entry[5] != 0
        || (initial && (entry[1] & 0xf0 != 0 || entry[12..].iter().any(|byte| *byte != 0)))
        || (!initial && entry[1] & 16 != 0)
    {
        return Err(ScanError::Invalid);
    }
    let lbn = little32(&entry[8..12])?;
    if lbn == 0 && entry[0] == 0 {
        return Ok(());
    }
    let offset = u64::from(lbn) * SECTOR;
    let minimum = u64::from(u16::from_le_bytes([entry[6], entry[7]])) * 512;
    let (label, mut length) = match entry[1] & 15 {
        0 => ("Boot-NoEmul", minimum),
        1 => ("Boot-1.2M", 1200 * 1024),
        2 => ("Boot-1.44M", 1440 * 1024),
        3 => ("Boot-2.88M", 2880 * 1024),
        4 => {
            range(volume, offset, 512)?;
            let mbr = reader.read_at(offset, 512)?;
            if mbr[510..512] != [0x55, 0xaa] {
                return Err(ScanError::Invalid);
            }
            let start = u64::from(little32(&mbr[454..458])?);
            let count = u64::from(little32(&mbr[458..462])?);
            if count == 0 || mbr[462..510].iter().any(|byte| *byte != 0) {
                return Err(ScanError::Invalid);
            }
            (
                "Boot-HardDisk",
                start
                    .checked_add(count)
                    .and_then(|n| n.checked_mul(512))
                    .ok_or(ScanError::Invalid)?,
            )
        }
        _ => return Err(ScanError::Unsupported),
    };
    // No-emulation Sector Count specifies only the initially loaded prefix.
    // A matching directory file supplies an exact, independently bounded size.
    if entry[1] & 15 == 0
        && let Some(size) = file_sizes.get(&offset)
    {
        if *size < minimum {
            return Err(ScanError::Invalid);
        }
        length = *size;
    }
    if length < minimum {
        return Err(ScanError::Invalid);
    }
    range(volume, offset, length)?;
    if images.len() >= 4096 {
        return Err(ScanError::Limit);
    }
    images.push((label, offset, length));
    Ok(())
}

fn short_time(bytes: &[u8]) -> u32 {
    if bytes.len() != 7 || bytes.iter().all(|byte| *byte == 0) {
        return 0;
    }
    dos_time(
        i32::from(bytes[0]) + 1900,
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6] as i8,
    )
}

fn long_time(bytes: &[u8]) -> u32 {
    if bytes.len() != 17 || !bytes[..16].iter().all(u8::is_ascii_digit) {
        return 0;
    }
    let number = |part: &[u8]| {
        part.iter()
            .fold(0u32, |n, byte| n * 10 + u32::from(byte - b'0'))
    };
    dos_time(
        number(&bytes[..4]) as i32,
        number(&bytes[4..6]) as u8,
        number(&bytes[6..8]) as u8,
        number(&bytes[8..10]) as u8,
        number(&bytes[10..12]) as u8,
        number(&bytes[12..14]) as u8,
        bytes[16] as i8,
    )
}

fn month_days(year: i32, month: u8) -> u8 {
    match month {
        2 => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn dos_time(
    mut year: i32,
    mut month: u8,
    mut day: u8,
    hour: u8,
    minute: u8,
    second: u8,
    zone: i8,
) -> u32 {
    if !(1..=9999).contains(&year)
        || !(1..=12).contains(&month)
        || day == 0
        || day > month_days(year, month)
        || hour > 23
        || minute > 59
        || second > 59
        || !(-48..=52).contains(&zone)
    {
        return 0;
    }
    let mut minutes = i32::from(hour) * 60 + i32::from(minute) - i32::from(zone) * 15;
    if minutes < 0 {
        minutes += 1440;
        if day > 1 {
            day -= 1;
        } else {
            if month > 1 {
                month -= 1;
            } else {
                month = 12;
                year -= 1;
            }
            day = month_days(year, month);
        }
    } else if minutes >= 1440 {
        minutes -= 1440;
        if day < month_days(year, month) {
            day += 1;
        } else {
            day = 1;
            if month < 12 {
                month += 1;
            } else {
                month = 1;
                year += 1;
            }
        }
    }
    if !(1980..=2107).contains(&year) {
        return 0;
    }
    ((year as u32 - 1980) << 25)
        | (u32::from(month) << 21)
        | (u32::from(day) << 16)
        | ((minutes as u32 / 60) << 11)
        | ((minutes as u32 % 60) << 5)
        | u32::from(second / 2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use xdzip_codec_api::Cancellation;

    fn put16(bytes: &mut [u8], value: u16) {
        bytes[..2].copy_from_slice(&value.to_le_bytes());
        bytes[2..].copy_from_slice(&value.to_be_bytes());
    }
    fn put32(bytes: &mut [u8], value: u32) {
        bytes[..4].copy_from_slice(&value.to_le_bytes());
        bytes[4..].copy_from_slice(&value.to_be_bytes());
    }
    fn record(identifier: &[u8], lbn: u32, size: u32, flags: u8, system: &[u8]) -> Vec<u8> {
        let length =
            33 + identifier.len() + usize::from(identifier.len().is_multiple_of(2)) + system.len();
        assert!(length <= 255);
        let mut bytes = vec![0; length];
        bytes[0] = length as u8;
        put32(&mut bytes[2..10], lbn);
        put32(&mut bytes[10..18], size);
        bytes[18..25].copy_from_slice(&[126, 10, 25, 12, 34, 56, 0]);
        bytes[25] = flags;
        put16(&mut bytes[28..32], 1);
        bytes[32] = identifier.len() as u8;
        bytes[33..33 + identifier.len()].copy_from_slice(identifier);
        bytes[length - system.len()..].copy_from_slice(system);
        bytes
    }
    fn image(children: &[Vec<u8>], root_system: &[u8]) -> Vec<u8> {
        let mut image = vec![0; 32 * SECTOR as usize];
        let pvd = &mut image[16 * 2048..17 * 2048];
        pvd[0] = 1;
        pvd[1..7].copy_from_slice(b"CD001\x01");
        put32(&mut pvd[80..88], 32);
        put16(&mut pvd[120..124], 1);
        put16(&mut pvd[124..128], 1);
        put16(&mut pvd[128..132], 2048);
        pvd[156..190].copy_from_slice(&record(&[0], 20, 2048, 2, &[]));
        pvd[881] = 1;
        image[17 * 2048] = 255;
        image[17 * 2048 + 1..17 * 2048 + 7].copy_from_slice(b"CD001\x01");
        let mut offset = 20 * 2048;
        for entry in std::iter::once(record(&[0], 20, 2048, 2, root_system))
            .chain(std::iter::once(record(&[1], 20, 2048, 2, &[])))
            .chain(children.iter().cloned())
        {
            image[offset..offset + entry.len()].copy_from_slice(&entry);
            offset += entry.len();
        }
        image
    }
    fn scan_image(bytes: &[u8]) -> Result<Index, ScanError> {
        let mut source = Cursor::new(bytes);
        let control = Cancellation::default();
        let mut reader = ImageReader::new(&mut source, &control, 4 * 1024 * 1024)?;
        crate::parse(&mut reader, Format::Iso, ScanLimits::default())
    }
    fn susp_root() -> Vec<u8> {
        let mut system = vec![b'S', b'P', 7, 1, 0xbe, 0xef, 0];
        system.extend_from_slice(b"ER\x12\x01\x0a\0\0\x01RRIP_1991A");
        system
    }
    fn px(mode: u32, links: u32) -> Vec<u8> {
        let mut bytes = vec![0; 36];
        bytes[..4].copy_from_slice(b"PX\x24\x01");
        for (field, value) in bytes[4..]
            .as_chunks_mut::<8>()
            .0
            .iter_mut()
            .zip([mode, links, 0, 0])
        {
            put32(field, value);
        }
        bytes
    }
    fn nm(name: &[u8]) -> Vec<u8> {
        let mut bytes = vec![b'N', b'M', (name.len() + 5) as u8, 1, 0];
        bytes.extend_from_slice(name);
        bytes
    }
    fn at_directory(
        image: &mut [u8],
        lbn: u32,
        parent: u32,
        dot_system: &[u8],
        parent_system: &[u8],
        children: &[Vec<u8>],
    ) {
        let mut offset = lbn as usize * 2048;
        for entry in std::iter::once(record(&[0], lbn, 2048, 2, dot_system))
            .chain(std::iter::once(record(
                &[1],
                parent,
                2048,
                2,
                parent_system,
            )))
            .chain(children.iter().cloned())
        {
            image[offset..offset + entry.len()].copy_from_slice(&entry);
            offset += entry.len();
        }
    }
    fn relocation(signature: &[u8; 2], lbn: u32) -> Vec<u8> {
        let mut entry = vec![0; 12];
        entry[..2].copy_from_slice(signature);
        entry[2..4].copy_from_slice(&[12, 1]);
        put32(&mut entry[4..12], lbn);
        entry
    }
    fn boot_image() -> Vec<u8> {
        let mut bytes = image(&[record(b"BOOT.IMG;1", 24, 2048, 0, &[])], &[]);
        let terminator = bytes[17 * 2048..18 * 2048].to_vec();
        bytes[18 * 2048..19 * 2048].copy_from_slice(&terminator);
        let boot = &mut bytes[17 * 2048..18 * 2048];
        boot.fill(0);
        boot[1..7].copy_from_slice(b"CD001\x01");
        boot[7..30].copy_from_slice(b"EL TORITO SPECIFICATION");
        boot[71..75].copy_from_slice(&21u32.to_le_bytes());
        let catalog = &mut bytes[21 * 2048..22 * 2048];
        catalog[0] = 1;
        catalog[30..32].copy_from_slice(&[0x55, 0xaa]);
        let sum = catalog[..32]
            .as_chunks::<2>()
            .0
            .iter()
            .fold(0u16, |n, word| n.wrapping_add(u16::from_le_bytes(*word)));
        catalog[28..30].copy_from_slice(&0u16.wrapping_sub(sum).to_le_bytes());
        catalog[32] = 0x88;
        catalog[38..40].copy_from_slice(&4u16.to_le_bytes());
        catalog[40..44].copy_from_slice(&24u32.to_le_bytes());
        bytes
    }

    #[test]
    fn numeric_representations_must_agree() {
        assert_eq!(both16(&[0x34, 0x12, 0x12, 0x34]), Ok(0x1234));
        assert_eq!(both32(&[4, 3, 2, 1, 1, 2, 3, 4]), Ok(0x01020304));
        assert_eq!(both16(&[1, 0, 0, 2]), Err(ScanError::Invalid));
        let mut bytes = image(&[], &[]);
        bytes[16 * 2048 + 84] ^= 1;
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
    }

    #[test]
    fn multi_section_file_retains_every_extent_and_empty_file() {
        let bytes = image(
            &[
                record(b"FILE.BIN;1", 22, 2048, 128, &[]),
                record(b"FILE.BIN;1", 25, 7, 0, &[]),
                record(b"EMPTY.;1", 0, 0, 0, &[]),
            ],
            &[],
        );
        let index = scan_image(&bytes).unwrap();
        assert_eq!(index.records.len(), 2);
        assert_eq!(index.records[0].entry.raw_name, b"FILE.BIN");
        assert_eq!(index.records[0].entry.size, 2055);
        assert_eq!(
            index.records[0].extents,
            [
                Extent::Data {
                    offset: 22 * SECTOR,
                    length: 2048
                },
                Extent::Data {
                    offset: 25 * SECTOR,
                    length: 7
                }
            ]
        );
        assert_eq!(index.records[1].entry.raw_name, b"EMPTY");
        assert!(index.records[1].extents.is_empty());
    }

    #[test]
    fn unterminated_or_inconsistent_sections_fail() {
        for children in [
            vec![record(b"A;1", 22, 2048, 128, &[])],
            vec![
                record(b"A;1", 22, 2048, 128, &[]),
                record(b"B;1", 23, 1, 0, &[]),
            ],
            vec![
                record(b"A;1", 22, 3, 128, &[]),
                record(b"A;1", 23, 1, 0, &[]),
            ],
        ] {
            assert!(matches!(
                scan_image(&image(&children, &[])),
                Err(ScanError::Invalid)
            ));
        }
    }

    #[test]
    fn extended_attribute_blocks_are_not_file_bytes() {
        let mut file = record(b"ATTR.BIN;1", 22, 7, 0, &[]);
        file[1] = 2;
        let mut bytes = image(&[file], &[]);
        bytes[22 * 2048 + 180] = 1;
        bytes[22 * 2048 + 27..22 * 2048 + 44].copy_from_slice(b"2026102501000000\x20");
        let index = scan_image(&bytes).unwrap();
        assert_eq!(
            index.records[0].extents,
            [Extent::Data {
                offset: 24 * SECTOR,
                length: 7
            }]
        );
        assert_eq!(
            index.records[0].entry.modified_dos,
            short_time(&[126, 10, 25, 1, 0, 0, 32])
        );
        bytes[22 * 2048 + 249] = 1;
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
    }

    #[test]
    fn interleaved_and_record_organized_files_are_explicitly_unsupported() {
        let mut interleaved = record(b"A;1", 22, 7, 0, &[]);
        interleaved[26] = 1;
        assert!(matches!(
            scan_image(&image(&[interleaved], &[])),
            Err(ScanError::Unsupported)
        ));
        let mut record_file = record(b"A;1", 22, 7, 8, &[]);
        record_file[1] = 1;
        assert!(matches!(
            scan_image(&image(&[record_file], &[])),
            Err(ScanError::Unsupported)
        ));
    }

    #[test]
    fn directory_records_cannot_cross_sectors_or_hide_nonzero_padding() {
        let mut bytes = image(&[], &[]);
        let start = 20 * 2048;
        bytes[start + 68] = 0;
        bytes[start + 70] = 42;
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
        let mut bytes = image(&[], &[]);
        // Fill the sector with legal-sized records up to byte 2000, then place
        // a record whose declared 60-byte length would cross that sector.
        let fill = record(&[b'X'; 127], 0, 0, 0, &[]);
        let mut offset = start + 68;
        while offset + fill.len() + 60 < start + 2048 {
            bytes[offset..offset + fill.len()].copy_from_slice(&fill);
            offset += fill.len();
        }
        let leftover = start + 2048 - offset;
        if leftover >= 34 {
            bytes[offset] = 255;
            assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
        }
    }

    #[test]
    fn joliet_ucs2_is_big_endian_and_never_lossy() {
        assert_eq!(ucs2(&[0x4e, 0x2d, 0x65, 0x87]).unwrap(), "中文".as_bytes());
        assert_eq!(ucs2(&[0xd8, 0, 0xdc, 0]), Err(ScanError::Invalid));
        assert_eq!(ucs2(&[0]), Err(ScanError::Invalid));
        let mut bytes = image(
            &[record(&[0x4e, 0x2d, 0, b';', 0, b'1'], 22, 3, 0, &[])],
            &[],
        );
        let pvd = bytes[16 * 2048..17 * 2048].to_vec();
        bytes[17 * 2048..18 * 2048].copy_from_slice(&pvd);
        bytes[17 * 2048] = 2;
        bytes[17 * 2048 + 88..17 * 2048 + 91].copy_from_slice(b"%/E");
        bytes[18 * 2048] = 255;
        bytes[18 * 2048 + 1..18 * 2048 + 7].copy_from_slice(b"CD001\x01");
        assert_eq!(
            scan_image(&bytes).unwrap().records[0].entry.raw_name,
            "中".as_bytes()
        );
    }

    #[test]
    fn rr_names_links_and_special_objects_are_preserved_without_output_spans() {
        let mut normal = px(0o100644, 1);
        normal.extend(nm("中文.txt".as_bytes()));
        let mut symlink = px(0o120777, 1);
        symlink.extend(nm(b"link"));
        symlink.extend_from_slice(b"SL\x0a\x01\0\0\x03foo");
        let mut device = px(0o020600, 1);
        device.extend(nm(b"device"));
        let index = scan_image(&image(
            &[
                record(b"NORMAL;1", 22, 3, 0, &normal),
                record(b"LINK;1", 0, 0, 0, &symlink),
                record(b"DEVICE;1", 0, 0, 0, &device),
            ],
            &susp_root(),
        ))
        .unwrap();
        assert_eq!(index.records[0].entry.raw_name, "中文.txt".as_bytes());
        assert!(index.records[1].entry.link);
        assert!(index.records[1].extents.is_empty());
        assert!(index.records[2].entry.warning);
        assert!(index.records[2].extents.is_empty());
    }

    #[test]
    fn rr_ce_follows_area_order_and_detects_cycles() {
        let mut system = nm(b"before");
        system[4] = 1;
        let mut ce = vec![0; 28];
        ce[..4].copy_from_slice(b"CE\x1c\x01");
        put32(&mut ce[4..12], 26);
        put32(&mut ce[12..20], 19);
        let after = nm(b"after");
        put32(&mut ce[20..28], after.len() as u32);
        system.extend_from_slice(&ce);
        let mut bytes = image(&[record(b"A;1", 22, 3, 0, &system)], &susp_root());
        bytes[26 * 2048 + 19..26 * 2048 + 19 + after.len()].copy_from_slice(&after);
        assert_eq!(
            scan_image(&bytes).unwrap().records[0].entry.raw_name,
            b"beforeafter"
        );
        put32(&mut ce[20..28], 28);
        let mut bytes = image(&[record(b"A;1", 22, 3, 0, &ce)], &susp_root());
        bytes[26 * 2048 + 19..26 * 2048 + 19 + 28].copy_from_slice(&ce);
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
    }

    #[test]
    fn rr_continued_names_require_a_final_fragment_and_unknown_entries_are_skipped() {
        let mut unfinished = nm(b"part");
        unfinished[4] = 1;
        assert!(matches!(
            scan_image(&image(
                &[record(b"A;1", 22, 3, 0, &unfinished)],
                &susp_root()
            )),
            Err(ScanError::Invalid)
        ));
        let mut unknown = b"XY\x05\x09\0".to_vec();
        unknown.extend(nm(b"kept"));
        assert_eq!(
            scan_image(&image(&[record(b"A;1", 22, 3, 0, &unknown)], &susp_root()))
                .unwrap()
                .records[0]
                .entry
                .raw_name,
            b"kept"
        );
        assert!(matches!(
            scan_image(&image(
                &[record(b"A;1", 22, 3, 0, b"SF\x04\x01")],
                &susp_root()
            )),
            Err(ScanError::Unsupported)
        ));
    }

    #[test]
    fn embedded_rr_separator_cannot_become_a_new_output_path() {
        let index = scan_image(&image(
            &[record(b"A;1", 22, 3, 0, &nm(b"a/b"))],
            &susp_root(),
        ))
        .unwrap();
        assert_eq!(index.records[0].entry.raw_name, b"a/b");
        assert!(index.records[0].entry.warning);
    }

    #[test]
    fn timestamps_convert_the_declared_offset_and_validate_calendar_days() {
        assert_eq!(
            short_time(&[126, 10, 25, 1, 0, 0, 32]),
            dos_time(2026, 10, 24, 17, 0, 0, 0)
        );
        assert_eq!(short_time(&[126, 2, 30, 0, 0, 0, 0]), 0);
        assert_eq!(
            long_time(b"2026102501000000\x20"),
            short_time(&[126, 10, 25, 1, 0, 0, 32])
        );
    }

    #[test]
    fn metadata_budget_and_cancellation_fail_before_a_partial_index() {
        let bytes = image(&[record(b"A;1", 22, 3, 0, &[])], &[]);
        let mut source = Cursor::new(&bytes);
        let control = Cancellation::default();
        let mut reader = ImageReader::new(&mut source, &control, 4 * 1024 * 1024).unwrap();
        assert!(matches!(
            scan(
                &mut reader,
                ScanLimits {
                    entries: 0,
                    ..ScanLimits::default()
                }
            ),
            Err(ScanError::Limit)
        ));
        control.cancel();
        assert!(matches!(reader.check(), Err(ScanError::Cancelled)));
    }

    #[test]
    fn shared_directory_graph_retains_each_path_but_rejects_cycles() {
        let mut bytes = image(
            &[
                record(b"A", 24, 2048, 2, &[]),
                record(b"B", 24, 2048, 2, &[]),
            ],
            &[],
        );
        at_directory(
            &mut bytes,
            24,
            20,
            &[],
            &[],
            &[record(b"X;1", 22, 3, 0, &[])],
        );
        let index = scan_image(&bytes).unwrap();
        assert_eq!(
            index
                .records
                .iter()
                .map(|record| record.entry.raw_name.as_slice())
                .collect::<Vec<_>>(),
            [b"A/".as_slice(), b"A/X", b"B/", b"B/X"]
        );
        at_directory(
            &mut bytes,
            24,
            20,
            &[],
            &[],
            &[record(b"BACK", 20, 2048, 2, &[])],
        );
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
    }

    #[test]
    fn rr_relocation_resolves_cl_and_validates_pl_and_re() {
        let mut bytes = image(
            &[
                record(b"STORE", 24, 2048, 2, &[]),
                record(b"DEEP;1", 0, 0, 0, &relocation(b"CL", 26)),
            ],
            &susp_root(),
        );
        at_directory(
            &mut bytes,
            24,
            20,
            &[],
            &[],
            &[record(b"MOVED", 26, 2048, 2, b"RE\x04\x01")],
        );
        at_directory(
            &mut bytes,
            26,
            24,
            &px(0o040755, 2),
            &relocation(b"PL", 20),
            &[record(b"FILE;1", 22, 3, 0, &[])],
        );
        let index = scan_image(&bytes).unwrap();
        assert_eq!(
            index
                .records
                .iter()
                .map(|record| record.entry.raw_name.as_slice())
                .collect::<Vec<_>>(),
            [b"STORE/".as_slice(), b"DEEP/", b"DEEP/FILE"]
        );
        at_directory(
            &mut bytes,
            26,
            24,
            &px(0o040755, 2),
            &relocation(b"PL", 24),
            &[record(b"FILE;1", 22, 3, 0, &[])],
        );
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
    }

    #[test]
    fn dangling_relocations_never_silently_discard_directories() {
        let mut bytes = image(
            &[record(b"HIDDEN", 24, 2048, 2, b"RE\x04\x01")],
            &susp_root(),
        );
        at_directory(&mut bytes, 24, 20, &[], &relocation(b"PL", 20), &[]);
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
        let bytes = image(
            &[record(b"DANGLING;1", 0, 0, 0, &relocation(b"CL", 24))],
            &susp_root(),
        );
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
    }

    #[test]
    fn final_section_metadata_overrides_prior_section_names() {
        let bytes = image(
            &[
                record(b"A;1", 22, 2048, 128, &nm(b"earlier")),
                record(b"A;1", 25, 3, 0, &nm(b"final")),
            ],
            &susp_root(),
        );
        assert_eq!(
            scan_image(&bytes).unwrap().records[0].entry.raw_name,
            b"final"
        );
    }

    #[test]
    fn boot_catalog_images_are_bounded_files_and_require_validation_checksum() {
        let mut bytes = boot_image();
        let index = scan_image(&bytes).unwrap();
        assert_eq!(index.records[2].entry.raw_name, b"[BOOT]/Boot-NoEmul.img");
        assert_eq!(
            index.records[2].extents,
            [Extent::Data {
                offset: 24 * SECTOR,
                length: 2048
            }]
        );
        bytes[21 * 2048 + 28] ^= 1;
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
    }

    #[test]
    fn boot_sections_retain_each_image_and_require_the_announced_next_header() {
        let mut bytes = boot_image();
        let initial = bytes[21 * 2048 + 32..21 * 2048 + 64].to_vec();
        bytes[21 * 2048 + 64] = 0x91;
        bytes[21 * 2048 + 66] = 1;
        bytes[21 * 2048 + 96..21 * 2048 + 128].copy_from_slice(&initial);
        let index = scan_image(&bytes).unwrap();
        assert_eq!(index.records[2].entry.raw_name, b"[BOOT]/1-Boot-NoEmul.img");
        assert_eq!(index.records[3].entry.raw_name, b"[BOOT]/2-Boot-NoEmul.img");
        bytes[21 * 2048 + 64] = 0x90;
        assert!(matches!(scan_image(&bytes), Err(ScanError::Invalid)));
    }

    #[test]
    fn bounded_header_and_record_mutations_do_not_panic() {
        let original = image(&[record(b"A;1", 22, 3, 0, &nm(b"original"))], &susp_root());
        let mut seed = 0x17ac_39e1u32;
        for _ in 0..512 {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            let mut bytes = original.clone();
            let regions = [16 * 2048, 17 * 2048, 20 * 2048];
            let offset = regions[seed as usize % regions.len()] + ((seed >> 8) as usize % 1024);
            bytes[offset] ^= (seed >> 24) as u8 | 1;
            let _ = scan_image(&bytes);
        }
    }

    #[test]
    fn public_codec_extracts_legacy_names_with_one_archive_wide_decision() {
        use xdzip_codec_api::MetadataCodec;
        use xdzip_codec_api::names::NameEncoding;
        use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
        struct Sink(Vec<u8>);
        impl DecodedSink for Sink {
            fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
                self.0.extend_from_slice(bytes);
                Ok(())
            }
        }
        let long = b"\xd6\xd0\xce\xc4\xd1\xb9\xcb\xf5\xb0\xfc\xb2\xe2\xca\xd4\xca\xfd\xbe\xdd\xce\xc4\xbc\xfe\xcb\xb5\xc3\xf7\xba\xcd\xca\xb9\xd3\xc3\xb7\xbd\xb7\xa8.txt";
        // GBK for 漏 is also valid UTF-8 for ©. The first name provides the
        // archive sample; resolving this second record independently is wrong.
        let accidental_utf8 = b"\xc2\xa9.txt";
        let mut bytes = image(
            &[
                record(long, 22, 6, 0, &[]),
                record(accidental_utf8, 23, 5, 0, &[]),
            ],
            &[],
        );
        bytes[22 * 2048..22 * 2048 + 6].copy_from_slice(b"legacy");
        bytes[23 * 2048..23 * 2048 + 5].copy_from_slice(b"bytes");
        let codec = crate::OpticalCodec {
            format: Format::Iso,
        };
        let control = Cancellation::default();
        let mut source = Cursor::new(&bytes);
        let rows = codec
            .scan(&mut source, &control, ScanLimits::default())
            .unwrap();
        assert_eq!(rows[0].raw_name, long);
        assert_eq!(
            rows[0].name_bytes(),
            "中文压缩包测试数据文件说明和使用方法.txt".as_bytes()
        );
        assert_eq!(rows[1].raw_name, accidental_utf8);
        assert_eq!(rows[1].name_bytes(), "漏.txt".as_bytes());
        assert!(
            rows.iter()
                .all(|row| !row.warning && matches!(row.name_encoding, NameEncoding::Legacy(_)))
        );
        let mut payload = codec.open_payload(&mut source, &rows, &control).unwrap();
        for (row, expected) in rows.iter().zip([b"legacy".as_slice(), b"bytes"]) {
            let mut sink = Sink(Vec::new());
            payload.decode(row, &mut sink, &control).unwrap();
            assert_eq!(sink.0, expected);
        }
    }

    #[test]
    fn rock_ridge_shift_jis_trail_backslash_is_checked_after_decoding() {
        let encoded = b"\x83\x5c\x83t\x83g\x83E\x83F\x83A\x82\xcc\x90\xdd\x92\xe8\x83t\x83@\x83C\x83\x8b\x82\xc6\x93\xfa\x96{\x8c\xea.txt";
        let index = scan_image(&image(
            &[record(b"FILE;1", 22, 3, 0, &nm(encoded))],
            &susp_root(),
        ))
        .unwrap();
        let row = &index.records[0].entry;
        assert_eq!(row.raw_name, encoded);
        assert_eq!(
            row.name_bytes(),
            "ソフトウェアの設定ファイルと日本語.txt".as_bytes()
        );
        assert!(!row.warning);
    }

    #[test]
    #[ignore = "requires a separately prepared independent ISO fixture"]
    fn inspect_external_fixture() {
        let path = std::env::var_os("XDZIP_ISO_FIXTURE").expect("XDZIP_ISO_FIXTURE");
        let bytes = std::fs::read(path).unwrap();
        let index = scan_image(&bytes).unwrap();
        for record in index.records {
            println!(
                "{:?} size={} directory={} link={} warning={} modified={:08x}",
                record.entry.display_name,
                record.entry.size,
                record.entry.directory,
                record.entry.link,
                record.entry.warning,
                record.entry.modified_dos
            );
        }
    }
}
