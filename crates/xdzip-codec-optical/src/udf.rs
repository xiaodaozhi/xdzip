//! Independent UDF reader, from ECMA-167 (3rd ed., parts 1, 3 and 4) and OSTA
//! UDF 2.60, sections 2.1-2.3 and 6.6. No operating-system filesystem is used.
//!
//! The on-disc logical addresses remain distinct from physical image offsets.
//! A bad primary descriptor sequence may be recovered from its reserve sequence;
//! a successfully selected UDF namespace never falls back to another filesystem.
use std::collections::{BTreeMap, BTreeSet};

use xdzip_codec_api::{Format, ScanError, ScanLimits};

use crate::image::{Extent, ImageReader, Index, IndexBuilder, Node};

const MAX_MAPS: usize = 256;
const MAX_EXTENTS: usize = 262_144;
const MAX_CHAIN: usize = 4096;
const MAX_METADATA: u64 = 64 * 1024 * 1024;

type Result<T> = std::result::Result<T, ScanError>;

fn bytes(data: &[u8], at: usize, len: usize) -> Result<&[u8]> {
    data.get(at..at.checked_add(len).ok_or(ScanError::Invalid)?)
        .ok_or(ScanError::Invalid)
}
fn u16_at(data: &[u8], at: usize) -> Result<u16> {
    Ok(u16::from_le_bytes(
        bytes(data, at, 2)?
            .try_into()
            .map_err(|_| ScanError::Invalid)?,
    ))
}
fn u32_at(data: &[u8], at: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes(data, at, 4)?
            .try_into()
            .map_err(|_| ScanError::Invalid)?,
    ))
}
fn u64_at(data: &[u8], at: usize) -> Result<u64> {
    Ok(u64::from_le_bytes(
        bytes(data, at, 8)?
            .try_into()
            .map_err(|_| ScanError::Invalid)?,
    ))
}
fn add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right).ok_or(ScanError::Invalid)
}
fn mul(left: u64, right: u64) -> Result<u64> {
    left.checked_mul(right).ok_or(ScanError::Invalid)
}
fn to_usize(value: u64) -> Result<usize> {
    usize::try_from(value).map_err(|_| ScanError::Limit)
}

// ECMA-167 3/7.2.6: x^16 + x^12 + x^5 + 1, zero initial register.
fn crc16(data: &[u8]) -> u16 {
    let mut crc = 0u16;
    for &byte in data {
        crc ^= u16::from(byte) << 8;
        for _ in 0..8 {
            crc = if crc & 0x8000 == 0 {
                crc << 1
            } else {
                (crc << 1) ^ 0x1021
            };
        }
    }
    crc
}

fn tag_header(data: &[u8], location: u32) -> Result<u16> {
    let tag = bytes(data, 0, 16)?;
    if !matches!(u16_at(tag, 2)?, 2 | 3) || tag[5] != 0 || u32_at(tag, 12)? != location {
        return Err(ScanError::Invalid);
    }
    let checksum = tag
        .iter()
        .enumerate()
        .filter(|(at, _)| *at != 4)
        .fold(0u8, |sum, (_, byte)| sum.wrapping_add(*byte));
    if checksum != tag[4] {
        return Err(ScanError::Invalid);
    }
    u16_at(tag, 0)
}

fn tag(data: &[u8], location: u32, size: usize, exception: bool) -> Result<u16> {
    let id = tag_header(data, location)?;
    bytes(data, 0, size)?;
    let covered = usize::from(u16_at(data, 10)?);
    let normal = size.checked_sub(16).ok_or(ScanError::Invalid)?.min(65_535);
    if (!exception && covered != normal)
        || covered > normal
        || crc16(bytes(data, 16, covered)?) != u16_at(data, 8)?
    {
        return Err(ScanError::Invalid);
    }
    Ok(id)
}

fn entity_is(data: &[u8], at: usize, identifier: &[u8]) -> Result<bool> {
    let name = bytes(data, at + 1, 23)?;
    Ok(identifier.len() <= name.len()
        && name.starts_with(identifier)
        && name[identifier.len()..].iter().all(|byte| *byte == 0))
}

fn cs0(data: &[u8]) -> Result<String> {
    let (&compression, encoded) = data.split_first().ok_or(ScanError::Invalid)?;
    let name = match compression {
        8 => encoded.iter().map(|byte| char::from(*byte)).collect(),
        16 if encoded.len() % 2 == 0 => {
            let words: Vec<u16> = encoded
                .as_chunks::<2>()
                .0
                .iter()
                .map(|pair| u16::from_be_bytes([pair[0], pair[1]]))
                .collect();
            String::from_utf16(&words).map_err(|_| ScanError::Invalid)?
        }
        _ => return Err(ScanError::Unsupported),
    };
    Ok(name)
}

fn charspec(data: &[u8], at: usize) -> Result<()> {
    let value = bytes(data, at, 64)?;
    let identifier = b"OSTA Compressed Unicode";
    if value[0] != 0
        || !value[1..].starts_with(identifier)
        || value[1 + identifier.len()..].iter().any(|byte| *byte != 0)
    {
        return Err(ScanError::Unsupported);
    }
    Ok(())
}

fn domain(data: &[u8], at: usize) -> Result<()> {
    if !entity_is(data, at, b"*OSTA UDF Compliant")? {
        return Err(ScanError::Unsupported);
    }
    let revision = u16_at(data, at + 24)?;
    if !(0x0102..=0x0260).contains(&revision) {
        return Err(ScanError::Unsupported);
    }
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Address {
    partition: u16,
    block: u32,
}
#[derive(Clone, Copy, Debug)]
struct Allocation {
    address: Address,
    length: u64,
    kind: u8,
}

fn long_ad(data: &[u8], at: usize) -> Result<Allocation> {
    let raw = u32_at(data, at)?;
    let length = u64::from(raw & 0x3fff_ffff);
    let allocation = Allocation {
        address: Address {
            block: u32_at(data, at + 4)?,
            partition: u16_at(data, at + 8)?,
        },
        length,
        kind: (raw >> 30) as u8,
    };
    bytes(data, at, 16)?;
    if length == 0
        && (raw != 0 || allocation.address.block != 0 || allocation.address.partition != 0)
    {
        return Err(ScanError::Invalid);
    }
    Ok(allocation)
}

#[derive(Clone, Copy)]
struct Partition {
    start: u32,
    blocks: u32,
}

struct Descriptors {
    primary: Option<Vec<u8>>,
    logical: Option<Vec<u8>>,
    partitions: BTreeMap<u16, Vec<u8>>,
}

fn extent_ad(data: &[u8], at: usize) -> Result<(u32, u32)> {
    let length = u32_at(data, at)?;
    if length & 0xc000_0000 != 0 {
        return Err(ScanError::Invalid);
    }
    Ok((u32_at(data, at + 4)?, length))
}

fn descriptor_size(data: &[u8]) -> Result<usize> {
    let size = match u16_at(data, 0)? {
        6 => 440usize
            .checked_add(to_usize(u64::from(u32_at(data, 264)?))?)
            .ok_or(ScanError::Limit),
        7 => 24usize
            .checked_add(to_usize(mul(u64::from(u32_at(data, 20)?), 8)?)?)
            .ok_or(ScanError::Limit),
        9 => {
            let table = mul(u64::from(u32_at(data, 72)?), 8)?;
            to_usize(add(add(80, table)?, u64::from(u32_at(data, 76)?))?)
        }
        _ => Ok(512),
    }?;
    Ok(size)
}

fn volume_descriptor(reader: &mut ImageReader<'_>, sector: u64, location: u32) -> Result<Vec<u8>> {
    let offset = mul(u64::from(location), sector)?;
    let first = reader.read_at(offset, sector as usize)?;
    tag_header(&first, location)?;
    let size = descriptor_size(&first)?;
    if size as u64 > MAX_METADATA {
        return Err(ScanError::Limit);
    }
    let descriptor = if size > first.len() {
        reader.read_at(offset, size)?
    } else {
        first
    };
    // Variable descriptors may either end at their last table byte (ECMA
    // 3/10.6, 3/10.10) or retain zero-filled space to 512 bytes. Microsoft
    // IMAPI uses the former and other independent authors use the latter.
    let size = if size < 512 && usize::from(u16_at(&descriptor, 10)?) == 496 {
        if bytes(&descriptor, size, 512 - size)?
            .iter()
            .any(|byte| *byte != 0)
        {
            return Err(ScanError::Invalid);
        }
        512
    } else {
        size
    };
    tag(&descriptor, location, size, false)?;
    Ok(descriptor[..size].to_vec())
}

fn prevail(
    slot: &mut Option<Vec<u8>>,
    descriptor: Vec<u8>,
    identity: std::ops::Range<usize>,
) -> Result<()> {
    if let Some(previous) = slot {
        if bytes(previous, identity.start, identity.len())?
            != bytes(&descriptor, identity.start, identity.len())?
        {
            return Err(ScanError::Unsupported);
        }
        let old_sequence = u32_at(previous, 16)?;
        let sequence = u32_at(&descriptor, 16)?;
        if sequence == old_sequence && previous[16..] != descriptor[16..] {
            return Err(ScanError::Invalid);
        }
        if sequence <= old_sequence {
            return Ok(());
        }
    }
    *slot = Some(descriptor);
    Ok(())
}

fn sequence(reader: &mut ImageReader<'_>, sector: u64, start: (u32, u32)) -> Result<Descriptors> {
    let mut result = Descriptors {
        primary: None,
        logical: None,
        partitions: BTreeMap::new(),
    };
    let mut next = Some(start);
    let mut seen = BTreeSet::new();
    let mut count = 0usize;
    while let Some((first, length)) = next.take() {
        if length == 0 || u64::from(length) % sector != 0 || !seen.insert((first, length)) {
            return Err(ScanError::Invalid);
        }
        if add(mul(u64::from(first), sector)?, u64::from(length))? > reader.len() {
            return Err(ScanError::Invalid);
        }
        let mut position = 0u64;
        let mut ended = false;
        while position < u64::from(length) {
            reader.check()?;
            count += 1;
            if count > MAX_CHAIN {
                return Err(ScanError::Limit);
            }
            let location = u32::try_from(add(u64::from(first), position / sector)?)
                .map_err(|_| ScanError::Invalid)?;
            let descriptor = volume_descriptor(reader, sector, location)?;
            let advance = (descriptor.len() as u64).div_ceil(sector) * sector;
            position = add(position, advance)?;
            if position > u64::from(length) {
                return Err(ScanError::Invalid);
            }
            match u16_at(&descriptor, 0)? {
                1 => prevail(&mut result.primary, descriptor, 24..190)?,
                3 => {
                    let extent = extent_ad(&descriptor, 20)?;
                    if extent.1 != 0 {
                        next = Some(extent);
                    }
                    ended = true;
                    break;
                }
                5 => {
                    let number = u16_at(&descriptor, 22)?;
                    if let Some(previous) = result.partitions.get(&number) {
                        let old_sequence = u32_at(previous, 16)?;
                        let sequence = u32_at(&descriptor, 16)?;
                        if sequence == old_sequence && previous[16..] != descriptor[16..] {
                            return Err(ScanError::Invalid);
                        }
                        if sequence <= old_sequence {
                            continue;
                        }
                    }
                    if result.partitions.len() >= MAX_MAPS {
                        return Err(ScanError::Limit);
                    }
                    result.partitions.insert(number, descriptor);
                }
                6 => prevail(&mut result.logical, descriptor, 20..212)?,
                8 => {
                    ended = true;
                    break;
                }
                4 | 7 => {}
                _ => return Err(ScanError::Unsupported),
            }
        }
        if !ended {
            return Err(ScanError::Invalid);
        }
    }
    if result.primary.is_none() || result.logical.is_none() || result.partitions.is_empty() {
        return Err(ScanError::Invalid);
    }
    Ok(result)
}

#[derive(Clone)]
enum Map {
    Physical(Partition),
    Sparable {
        partition: Partition,
        packet: u32,
        relocated: BTreeMap<u32, u32>,
    },
    Virtual {
        base: usize,
        table: Vec<u32>,
    },
    Metadata {
        segments: Vec<Segment>,
        ends: Vec<u64>,
        mirror: Option<Vec<Segment>>,
        mirror_ends: Option<Vec<u64>>,
        size: u64,
    },
    Pending,
}

#[derive(Clone)]
struct Segment {
    extent: Extent,
    // Descriptor tags refer to the address before partition translation.
    location: u32,
    displacement: u64,
}
impl Segment {
    fn length(&self) -> u64 {
        match &self.extent {
            Extent::Data { length, .. } | Extent::Zero { length } => *length,
            Extent::Inline(data) => data.len() as u64,
        }
    }
}

fn segment_ends(segments: &[Segment]) -> Result<Vec<u64>> {
    let mut total = 0;
    segments
        .iter()
        .map(|segment| {
            total = add(total, segment.length())?;
            Ok(total)
        })
        .collect()
}

struct File {
    kind: u8,
    links: u16,
    size: u64,
    modified_dos: u32,
    segments: Vec<Segment>,
}

struct Volume {
    sector: u64,
    maps: Vec<Map>,
    root: Allocation,
    metadata_limit: u64,
}

impl Volume {
    fn discover(reader: &mut ImageReader<'_>, limits: ScanLimits) -> Result<Self> {
        let mut error = ScanError::Invalid;
        // UDF 2.60 2.2.3 also permits sector 512 on unclosed sequential media.
        for sector in [2048u64, 512, 1024, 4096, 8192, 16384, 32768] {
            if reader.len() < mul(257, sector)? {
                continue;
            }
            let last = reader.len() / sector - 1;
            let mut locations = vec![256, last.saturating_sub(256), last, 512];
            let mut seen = BTreeSet::new();
            locations.retain(|location| *location <= last && seen.insert(*location));
            for location in locations {
                let location = u32::try_from(location).map_err(|_| ScanError::Unsupported)?;
                let offset = mul(u64::from(location), sector)?;
                let anchor = reader.read_at(offset, sector as usize)?;
                if tag(&anchor, location, 512, false).is_err() || u16_at(&anchor, 0)? != 2 {
                    continue;
                }
                for extent in [extent_ad(&anchor, 16)?, extent_ad(&anchor, 24)?] {
                    if extent.1 == 0 {
                        continue;
                    }
                    let result = sequence(reader, sector, extent).and_then(|descriptors| {
                        Self::from_descriptors(reader, sector, descriptors, limits)
                    });
                    match result {
                        Ok(volume) => return Ok(volume),
                        Err(ScanError::Invalid) => error = ScanError::Invalid,
                        Err(other) => return Err(other),
                    }
                }
            }
        }
        Err(error)
    }

    fn from_descriptors(
        reader: &mut ImageReader<'_>,
        sector: u64,
        mut descriptors: Descriptors,
        limits: ScanLimits,
    ) -> Result<Self> {
        let primary = descriptors.primary.take().ok_or(ScanError::Invalid)?;
        let logical = descriptors.logical.take().ok_or(ScanError::Invalid)?;
        let volume_number = u16_at(&primary, 56)?;
        if volume_number != 1 || u16_at(&primary, 58)? != 1 {
            return Err(ScanError::Unsupported);
        }
        charspec(&primary, 200)?;
        charspec(&logical, 20)?;
        domain(&logical, 216)?;
        // UDF requires LBS == logical sector size (UDF 2.60, table 2).
        if u64::from(u32_at(&logical, 212)?) != sector {
            return Err(ScanError::Unsupported);
        }
        let mut partitions = BTreeMap::new();
        for (number, descriptor) in descriptors.partitions {
            if !(entity_is(&descriptor, 24, b"+NSR02")? || entity_is(&descriptor, 24, b"+NSR03")?) {
                return Err(ScanError::Unsupported);
            }
            if u32_at(&descriptor, 184)? > 4 {
                return Err(ScanError::Unsupported);
            }
            let partition = Partition {
                start: u32_at(&descriptor, 188)?,
                blocks: u32_at(&descriptor, 192)?,
            };
            if partition.blocks == 0 {
                return Err(ScanError::Invalid);
            }
            // Sequential media can advertise unrecorded capacity beyond EOF. Every
            // referenced recorded block is still checked against the actual image.
            mul(
                add(u64::from(partition.start), u64::from(partition.blocks))?,
                sector,
            )?;
            partitions.insert(number, partition);
        }
        let map_length = to_usize(u64::from(u32_at(&logical, 264)?))?;
        let map_count = to_usize(u64::from(u32_at(&logical, 268)?))?;
        if map_count == 0 || map_count > MAX_MAPS {
            return Err(ScanError::Limit);
        }
        let map_data = bytes(&logical, 440, map_length)?;
        let mut raw_maps = Vec::new();
        let mut position = 0usize;
        for _ in 0..map_count {
            let header = bytes(map_data, position, 2)?;
            let length = usize::from(header[1]);
            let map = bytes(map_data, position, length)?;
            if !matches!((header[0], length), (1, 6) | (2, 64)) {
                return Err(ScanError::Unsupported);
            }
            let number_at = if header[0] == 1 { 2 } else { 36 };
            if u16_at(map, number_at)? != volume_number {
                return Err(ScanError::Unsupported);
            }
            let number = u16_at(map, number_at + 2)?;
            let partition = *partitions.get(&number).ok_or(ScanError::Invalid)?;
            raw_maps.push((number, partition, map.to_vec()));
            position += length;
        }
        if position != map_data.len() {
            return Err(ScanError::Invalid);
        }
        let mut volume = Self {
            sector,
            maps: vec![Map::Pending; map_count],
            root: long_ad(&logical, 248)?,
            metadata_limit: limits.read_bytes.min(MAX_METADATA),
        };
        for (index, (_, partition, map)) in raw_maps.iter().enumerate() {
            volume.maps[index] = if map[0] == 1 {
                Map::Physical(*partition)
            } else if entity_is(map, 4, b"*UDF Sparable Partition")? {
                Self::sparable(reader, sector, *partition, map)?
            } else {
                Map::Pending
            };
        }
        for (index, (number, partition, map)) in raw_maps.iter().enumerate() {
            if !matches!(volume.maps[index], Map::Pending) {
                continue;
            }
            let bases: Vec<usize> = raw_maps
                .iter()
                .enumerate()
                .filter(|(other, (candidate, _, _))| {
                    *other != index
                        && candidate == number
                        && matches!(volume.maps[*other], Map::Physical(_) | Map::Sparable { .. })
                })
                .map(|(other, _)| other)
                .collect();
            if bases.len() != 1 {
                return Err(ScanError::Invalid);
            }
            let base = bases[0];
            if entity_is(map, 4, b"*UDF Virtual Partition")? {
                let table = volume.vat(reader, base, *partition, u16_at(map, 28)?)?;
                volume.maps[index] = Map::Virtual { base, table };
            } else if entity_is(map, 4, b"*UDF Metadata Partition")? {
                let main = u32_at(map, 40)?;
                let mirror = u32_at(map, 44)?;
                let allocation_unit = u32_at(map, 52)?;
                let alignment = u16_at(map, 56)?;
                if allocation_unit == 0 || alignment == 0 {
                    return Err(ScanError::Invalid);
                }
                let load = |reader: &mut ImageReader<'_>, location, expected| -> Result<File> {
                    let file = volume.file(
                        reader,
                        Allocation {
                            address: Address {
                                partition: base as u16,
                                block: location,
                            },
                            length: sector,
                            kind: 0,
                        },
                    )?;
                    if file.kind != expected || file.size % sector != 0 || file.links != 0 {
                        return Err(ScanError::Invalid);
                    }
                    Ok(file)
                };
                let main_file = load(reader, main, 250);
                let mirror_file = if mirror == u32::MAX {
                    None
                } else {
                    match load(reader, mirror, 251) {
                        Ok(file) => Some(file),
                        Err(ScanError::Invalid) => None,
                        Err(error) => return Err(error),
                    }
                };
                let (file, mirror) = match main_file {
                    Ok(file) => {
                        if mirror_file
                            .as_ref()
                            .is_some_and(|mirror| mirror.size != file.size)
                        {
                            return Err(ScanError::Invalid);
                        }
                        (file, mirror_file.map(|mirror| mirror.segments))
                    }
                    Err(ScanError::Invalid) => (mirror_file.ok_or(ScanError::Invalid)?, None),
                    Err(error) => return Err(error),
                };
                volume.maps[index] = Map::Metadata {
                    ends: segment_ends(&file.segments)?,
                    mirror_ends: mirror
                        .as_ref()
                        .map(|segments| segment_ends(segments))
                        .transpose()?,
                    segments: file.segments,
                    mirror,
                    size: file.size,
                };
            } else {
                return Err(ScanError::Unsupported);
            }
        }
        volume.integrity(reader, extent_ad(&logical, 432)?)?;
        volume.root = match volume.file_set(reader, volume.root) {
            Err(ScanError::Invalid) if volume.swap_metadata_mirrors() => {
                volume.file_set(reader, volume.root)?
            }
            result => result?,
        };
        Ok(volume)
    }

    fn swap_metadata_mirrors(&mut self) -> bool {
        let mut replaced = false;
        for map in &mut self.maps {
            if let Map::Metadata {
                segments,
                ends,
                mirror: Some(mirror),
                mirror_ends: Some(mirror_ends),
                ..
            } = map
            {
                std::mem::swap(segments, mirror);
                std::mem::swap(ends, mirror_ends);
                replaced = true;
            }
        }
        replaced
    }

    fn sparable(
        reader: &mut ImageReader<'_>,
        sector: u64,
        partition: Partition,
        map: &[u8],
    ) -> Result<Map> {
        let packet = u32::from(u16_at(map, 40)?);
        let copies = usize::from(map[42]);
        let size = u64::from(u32_at(map, 44)?);
        if packet == 0
            || !(1..=4).contains(&copies)
            || size < 56
            || !partition.start.is_multiple_of(packet)
            || !partition.blocks.is_multiple_of(packet)
        {
            return Err(ScanError::Invalid);
        }
        if size > MAX_METADATA {
            return Err(ScanError::Limit);
        }
        let mut selected: Option<(u32, BTreeMap<u32, u32>)> = None;
        for index in 0..copies {
            let location = u32_at(map, 48 + index * 4)?;
            let parse = |reader: &mut ImageReader<'_>| -> Result<(u32, BTreeMap<u32, u32>)> {
                let table = reader.read_at(mul(u64::from(location), sector)?, to_usize(size)?)?;
                let count = usize::from(u16_at(&table, 48)?);
                let used = 56usize
                    .checked_add(count.checked_mul(8).ok_or(ScanError::Invalid)?)
                    .ok_or(ScanError::Invalid)?;
                if tag(&table, location, used, false)? != 0
                    || !entity_is(&table, 16, b"*UDF Sparing Table")?
                {
                    return Err(ScanError::Invalid);
                }
                let mut relocated = BTreeMap::new();
                let mut previous = None;
                for entry in 0..count {
                    let original = u32_at(&table, 56 + entry * 8)?;
                    let mapped = u32_at(&table, 60 + entry * 8)?;
                    if previous.is_some_and(|old| old > original)
                        || mapped % packet != 0
                        || add(
                            mul(u64::from(mapped), sector)?,
                            mul(u64::from(packet), sector)?,
                        )? > reader.len()
                    {
                        return Err(ScanError::Invalid);
                    }
                    previous = Some(original);
                    if (0xffff_fff1..0xffff_ffff).contains(&original) {
                        return Err(ScanError::Unsupported);
                    }
                    if original >= 0xffff_fff0 {
                        continue;
                    }
                    if original % packet != 0
                        || original >= partition.blocks
                        || relocated.insert(original, mapped).is_some()
                    {
                        return Err(ScanError::Invalid);
                    }
                }
                Ok((u32_at(&table, 52)?, relocated))
            };
            match parse(reader) {
                Ok(candidate) => {
                    if let Some((sequence, previous)) = &selected
                        && *sequence == candidate.0
                        && previous != &candidate.1
                    {
                        return Err(ScanError::Invalid);
                    }
                    if selected.as_ref().is_none_or(|old| old.0 < candidate.0) {
                        selected = Some(candidate);
                    }
                }
                Err(ScanError::Invalid) => {}
                Err(error) => return Err(error),
            }
        }
        let (_, relocated) = selected.ok_or(ScanError::Invalid)?;
        Ok(Map::Sparable {
            partition,
            packet,
            relocated,
        })
    }

    fn map_block(&self, partition: u16, block: u32) -> Result<(u64, u64)> {
        let map = self
            .maps
            .get(usize::from(partition))
            .ok_or(ScanError::Invalid)?;
        match map {
            Map::Physical(partition) => {
                if block >= partition.blocks {
                    return Err(ScanError::Invalid);
                }
                Ok((
                    mul(
                        add(u64::from(partition.start), u64::from(block))?,
                        self.sector,
                    )?,
                    mul(u64::from(partition.blocks - block), self.sector)?,
                ))
            }
            Map::Sparable {
                partition,
                packet,
                relocated,
            } => {
                if block >= partition.blocks {
                    return Err(ScanError::Invalid);
                }
                let original = block / packet * packet;
                let within = block % packet;
                let physical = match relocated.get(&original) {
                    Some(mapped) => add(u64::from(*mapped), u64::from(within))?,
                    None => add(u64::from(partition.start), u64::from(block))?,
                };
                Ok((
                    mul(physical, self.sector)?,
                    mul(
                        u64::from((packet - within).min(partition.blocks - block)),
                        self.sector,
                    )?,
                ))
            }
            Map::Virtual { base, table } => {
                let mapped = *table.get(block as usize).ok_or(ScanError::Invalid)?;
                if mapped == u32::MAX {
                    return Err(ScanError::Invalid);
                }
                let (offset, _) = self.map_block(*base as u16, mapped)?;
                Ok((offset, self.sector))
            }
            Map::Metadata {
                segments,
                ends,
                size,
                ..
            } => {
                let position = mul(u64::from(block), self.sector)?;
                if position >= *size {
                    return Err(ScanError::Invalid);
                }
                let index = ends.partition_point(|end| *end <= position);
                let start = if index == 0 { 0 } else { ends[index - 1] };
                let within = position - start;
                match segments.get(index).ok_or(ScanError::Invalid)?.extent {
                    Extent::Data { offset, length } => Ok((add(offset, within)?, length - within)),
                    _ => Err(ScanError::Invalid),
                }
            }
            Map::Pending => Err(ScanError::Invalid),
        }
    }

    fn mapped_segments(
        &self,
        reader: &ImageReader<'_>,
        address: Address,
        length: u64,
        recorded: bool,
    ) -> Result<Vec<Segment>> {
        let mut result = Vec::new();
        let mut consumed = 0u64;
        while consumed < length {
            reader.check()?;
            if result.len() >= MAX_EXTENTS {
                return Err(ScanError::Limit);
            }
            let block = u32::try_from(add(u64::from(address.block), consumed / self.sector)?)
                .map_err(|_| ScanError::Invalid)?;
            let within = consumed % self.sector;
            let (physical, contiguous) = self.map_block(address.partition, block)?;
            let available = contiguous.checked_sub(within).ok_or(ScanError::Invalid)?;
            let count = available.min(length - consumed);
            if count == 0 {
                return Err(ScanError::Invalid);
            }
            let offset = add(physical, within)?;
            if recorded && add(offset, count)? > reader.len() {
                return Err(ScanError::Invalid);
            }
            result.push(Segment {
                extent: Extent::Data {
                    offset,
                    length: count,
                },
                location: address.block,
                displacement: consumed,
            });
            consumed = add(consumed, count)?;
        }
        Ok(result)
    }

    fn read_mapped(
        &self,
        reader: &mut ImageReader<'_>,
        address: Address,
        length: u64,
    ) -> Result<Vec<u8>> {
        if length > self.metadata_limit {
            return Err(ScanError::Limit);
        }
        let segments = self.mapped_segments(reader, address, length, true)?;
        self.read_segments(reader, &segments, length)
    }

    fn read_segments(
        &self,
        reader: &mut ImageReader<'_>,
        segments: &[Segment],
        size: u64,
    ) -> Result<Vec<u8>> {
        if size > self.metadata_limit {
            return Err(ScanError::Limit);
        }
        let mut data = Vec::new();
        data.try_reserve_exact(to_usize(size)?)
            .map_err(|_| ScanError::Limit)?;
        for segment in segments {
            let length = to_usize(segment.length())?;
            match &segment.extent {
                Extent::Data { offset, .. } => {
                    let begin = data.len();
                    data.resize(begin.checked_add(length).ok_or(ScanError::Limit)?, 0);
                    reader.read_exact_at(*offset, &mut data[begin..])?;
                }
                Extent::Inline(bytes) => data.extend_from_slice(bytes),
                Extent::Zero { .. } => return Err(ScanError::Invalid),
            }
        }
        if data.len() as u64 != size {
            return Err(ScanError::Invalid);
        }
        Ok(data)
    }

    fn vat(
        &self,
        reader: &mut ImageReader<'_>,
        base: usize,
        partition: Partition,
        revision: u16,
    ) -> Result<Vec<u32>> {
        let recorded_blocks = (reader.len() / self.sector)
            .checked_sub(u64::from(partition.start))
            .ok_or(ScanError::Invalid)?;
        let last = recorded_blocks
            .min(u64::from(partition.blocks))
            .checked_sub(1)
            .ok_or(ScanError::Invalid)?;
        // VAT is the last recorded data block; trailing anchors/reserved sectors
        // are skipped in a bounded 256-block recovery window (UDF 2.2.11).
        for delta in 0..=256u64.min(last) {
            reader.check()?;
            let block = u32::try_from(last - delta).map_err(|_| ScanError::Invalid)?;
            let address = Address {
                partition: base as u16,
                block,
            };
            let header = self.read_mapped(reader, address, self.sector)?;
            if !matches!(u16_at(&header, 0)?, 261 | 266) || tag_header(&header, block).is_err() {
                continue;
            }
            let kind = header[27];
            if !(kind == 248 || (revision < 0x0200 && kind == 0)) {
                continue;
            }
            let file = self.file(
                reader,
                Allocation {
                    address,
                    length: self.sector,
                    kind: 0,
                },
            )?;
            let table = self.read_segments(reader, &file.segments, file.size)?;
            let (begin, end) = if revision < 0x0200 {
                if table.len() < 36
                    || !entity_is(&table, table.len() - 36, b"*UDF Virtual Alloc Tbl")?
                {
                    return Err(ScanError::Invalid);
                }
                (0, table.len() - 36)
            } else {
                let header_length = usize::from(u16_at(&table, 0)?);
                let implementation = usize::from(u16_at(&table, 2)?);
                if header_length != 152 + implementation
                    || (implementation != 0 && (implementation < 32 || implementation % 4 != 0))
                {
                    return Err(ScanError::Invalid);
                }
                if u16_at(&table, 144)? > 0x0260 {
                    return Err(ScanError::Unsupported);
                }
                bytes(&table, 0, header_length)?;
                (header_length, table.len())
            };
            if (end - begin) % 4 != 0 {
                return Err(ScanError::Invalid);
            }
            return table[begin..end]
                .as_chunks::<4>()
                .0
                .iter()
                .map(|entry| u32_at(entry, 0))
                .collect();
        }
        Err(ScanError::Invalid)
    }

    fn integrity(&self, reader: &mut ImageReader<'_>, start: (u32, u32)) -> Result<()> {
        if start.1 == 0 {
            return Ok(());
        }
        let mut next = Some(start);
        let mut seen = BTreeSet::new();
        let mut count = 0usize;
        while let Some((location, length)) = next.take() {
            if length == 0
                || u64::from(length) % self.sector != 0
                || !seen.insert((location, length))
            {
                return Err(ScanError::Invalid);
            }
            if add(mul(u64::from(location), self.sector)?, u64::from(length))? > reader.len() {
                return Err(ScanError::Invalid);
            }
            let mut position = 0u64;
            while position < u64::from(length) {
                count += 1;
                if count > MAX_CHAIN {
                    return Err(ScanError::Limit);
                }
                let block = u32::try_from(add(u64::from(location), position / self.sector)?)
                    .map_err(|_| ScanError::Invalid)?;
                let probe = reader.read_at(mul(u64::from(block), self.sector)?, 16)?;
                if probe.iter().all(|byte| *byte == 0) {
                    break;
                }
                let descriptor = volume_descriptor(reader, self.sector, block)?;
                if u16_at(&descriptor, 0)? == 8 {
                    break;
                }
                if u16_at(&descriptor, 0)? != 9 || u32_at(&descriptor, 28)? > 1 {
                    return Err(ScanError::Invalid);
                }
                let partitions = u32_at(&descriptor, 72)? as usize;
                if partitions != self.maps.len() {
                    return Err(ScanError::Invalid);
                }
                let implementation = 80 + partitions * 8;
                if u32_at(&descriptor, 76)? < 46 {
                    return Err(ScanError::Invalid);
                }
                if u16_at(&descriptor, implementation + 40)? > 0x0260 {
                    return Err(ScanError::Unsupported);
                }
                let following = extent_ad(&descriptor, 32)?;
                if following.1 != 0 {
                    next = Some(following);
                    // ECMA-167 3/8.8.2 terminates processing this extent at
                    // the descriptor that identifies its successor.
                    break;
                }
                position = add(
                    position,
                    (descriptor.len() as u64).div_ceil(self.sector) * self.sector,
                )?;
                if position > u64::from(length) {
                    return Err(ScanError::Invalid);
                }
            }
        }
        Ok(())
    }

    fn file_set(&self, reader: &mut ImageReader<'_>, start: Allocation) -> Result<Allocation> {
        let mut next = Some(start);
        let mut seen = BTreeSet::new();
        let mut selected: Option<((u32, u32), Allocation)> = None;
        let mut descriptor_numbers = BTreeMap::<u32, Vec<u8>>::new();
        let mut count = 0usize;
        while let Some(extent) = next.take() {
            if extent.kind != 0
                || extent.length == 0
                || extent.length % self.sector != 0
                || !seen.insert(extent.address)
            {
                return Err(ScanError::Invalid);
            }
            for delta in 0..extent.length / self.sector {
                count += 1;
                if count > MAX_CHAIN {
                    return Err(ScanError::Limit);
                }
                let block = u32::try_from(add(u64::from(extent.address.block), delta)?)
                    .map_err(|_| ScanError::Invalid)?;
                let address = Address {
                    block,
                    ..extent.address
                };
                let descriptor = self.read_mapped(reader, address, self.sector)?;
                // ECMA-167 4/8.3.1 also permits an unrecorded logical block
                // to terminate a file-set descriptor sequence extent.
                if descriptor.iter().all(|byte| *byte == 0) {
                    break;
                }
                let id = tag(&descriptor, block, 512, false)?;
                if id == 8 {
                    break;
                }
                if id != 256 {
                    return Err(ScanError::Invalid);
                }
                charspec(&descriptor, 48)?;
                charspec(&descriptor, 240)?;
                domain(&descriptor, 416)?;
                let stream = long_ad(&descriptor, 464)?;
                if stream.length != 0 {
                    return Err(ScanError::Unsupported);
                }
                let root = long_ad(&descriptor, 400)?;
                let sequence = (u32_at(&descriptor, 40)?, u32_at(&descriptor, 44)?);
                // Descriptor numbers identify identical contents throughout
                // the sequence. The tag changes when recording another copy
                // at a different logical address, so compare its body only.
                let body = bytes(&descriptor, 16, 496)?;
                if let Some(previous) = descriptor_numbers.get(&sequence.1) {
                    if previous.as_slice() != body {
                        return Err(ScanError::Invalid);
                    }
                } else {
                    descriptor_numbers.insert(sequence.1, body.to_vec());
                }
                if selected
                    .as_ref()
                    .is_none_or(|previous| previous.0 < sequence)
                {
                    selected = Some((sequence, root));
                }
                let following = long_ad(&descriptor, 448)?;
                if following.length != 0 {
                    next = Some(following);
                    // A non-null Next Extent terminates this extent. Remaining
                    // allocated blocks are trailing blocks, not descriptors.
                    break;
                }
            }
        }
        selected.map(|(_, root)| root).ok_or(ScanError::Invalid)
    }

    fn file(&self, reader: &mut ImageReader<'_>, reference: Allocation) -> Result<File> {
        if reference.kind != 0 || reference.length < self.sector {
            return Err(ScanError::Invalid);
        }
        let mut address = reference.address;
        let mut icb_length = reference.length;
        let mut seen = BTreeSet::new();
        let (data, address) = loop {
            if !seen.insert(address) {
                return Err(ScanError::Invalid);
            }
            if seen.len() > MAX_CHAIN {
                return Err(ScanError::Limit);
            }
            let data = self.read_mapped(reader, address, self.sector)?;
            let id = tag_header(&data, address.block)?;
            if id == 259 {
                if tag(&data, address.block, 52, false)? != 259 || data[27] != 3 {
                    return Err(ScanError::Invalid);
                }
                let next = long_ad(&data, 36)?;
                if next.kind != 0 || next.length < self.sector {
                    return Err(ScanError::Invalid);
                }
                address = next.address;
                icb_length = next.length;
                continue;
            }
            if !matches!(id, 261 | 266) {
                return Err(ScanError::Invalid);
            }
            let strategy = u16_at(&data, 20)?;
            if strategy == 4 {
                break (data, address);
            }
            if strategy != 4096 {
                return Err(ScanError::Unsupported);
            }
            if icb_length < mul(self.sector, 2)? {
                return Err(ScanError::Invalid);
            }
            if u16_at(&data, 22)? != 1 || u16_at(&data, 24)? != 2 {
                return Err(ScanError::Invalid);
            }
            // UDF 6.6: one direct entry, then one indirect entry in the next block.
            let indirect_address = Address {
                block: address.block.checked_add(1).ok_or(ScanError::Invalid)?,
                ..address
            };
            let indirect = self.read_mapped(reader, indirect_address, self.sector)?;
            if indirect.iter().all(|byte| *byte == 0) {
                break (data, address);
            }
            let indirect_id = tag_header(&indirect, indirect_address.block)?;
            if indirect_id == 260 {
                tag(&indirect, indirect_address.block, 36, false)?;
                break (data, address);
            }
            if tag(&indirect, indirect_address.block, 52, false)? != 259 || indirect[27] != 3 {
                return Err(ScanError::Invalid);
            }
            let next = long_ad(&indirect, 36)?;
            if next.kind != 0 || next.length < self.sector {
                return Err(ScanError::Invalid);
            }
            address = next.address;
            icb_length = next.length;
        };
        let extended = u16_at(&data, 0)? == 266;
        let base = if extended { 216 } else { 176 };
        let ea = u32_at(&data, base - 8)? as usize;
        let ad = u32_at(&data, base - 4)? as usize;
        let ad_start = base.checked_add(ea).ok_or(ScanError::Invalid)?;
        let end = ad_start.checked_add(ad).ok_or(ScanError::Invalid)?;
        if end > data.len() {
            return Err(ScanError::Invalid);
        }
        tag(&data, address.block, end, false)?;
        let flags = u16_at(&data, 34)?;
        if flags & ((1 << 11) | 0xc000) != 0 || flags & 7 > 3 {
            return Err(ScanError::Unsupported);
        }
        if data[50] != 0 || data[51] != 0 || u32_at(&data, 52)? != 0 {
            return Err(ScanError::Unsupported);
        }
        if u16_at(&data, 24)? == 0 {
            return Err(ScanError::Invalid);
        }
        if extended && long_ad(&data, 152)?.length != 0 {
            return Err(ScanError::Unsupported);
        }
        let kind = data[27];
        let size = u64_at(&data, 56)?;
        if extended && u64_at(&data, 64)? < size {
            return Err(ScanError::Invalid);
        }
        let modified_dos = timestamp(&data, if extended { 92 } else { 84 })?;
        let segments = if kind == 249 {
            // Real-time files require media commands and must never be read (UDF 2.3.5.2.1).
            Vec::new()
        } else if flags & 7 == 3 {
            if size != ad as u64 {
                return Err(ScanError::Invalid);
            }
            vec![Segment {
                extent: Extent::Inline(bytes(&data, ad_start, ad)?.to_vec()),
                location: address.block,
                displacement: ad_start as u64,
            }]
        } else {
            self.allocations(
                reader,
                address,
                flags & 7,
                bytes(&data, ad_start, ad)?,
                size,
            )?
        };
        Ok(File {
            kind,
            links: u16_at(&data, 48)?,
            size,
            modified_dos,
            segments,
        })
    }

    fn allocations(
        &self,
        reader: &mut ImageReader<'_>,
        initial: Address,
        mode: u16,
        first: &[u8],
        size: u64,
    ) -> Result<Vec<Segment>> {
        let stride = match mode {
            0 => 8,
            1 => 16,
            2 => 20,
            _ => return Err(ScanError::Unsupported),
        };
        let mut data = first.to_vec();
        let mut address = initial;
        let mut seen = BTreeSet::new();
        let mut output = Vec::new();
        let mut total = 0u64;
        let mut count = 0usize;
        let mut last_partial = false;
        loop {
            if !data.len().is_multiple_of(stride) {
                return Err(ScanError::Invalid);
            }
            let mut continuation = None;
            for (index, raw) in data.chunks_exact(stride).enumerate() {
                reader.check()?;
                count += 1;
                if count > MAX_EXTENTS || output.len() > MAX_EXTENTS {
                    return Err(ScanError::Limit);
                }
                let field = u32_at(raw, 0)?;
                let kind = (field >> 30) as u8;
                let length = u64::from(field & 0x3fff_ffff);
                let block_at = if mode == 2 { 12 } else { 4 };
                let target = Address {
                    block: u32_at(raw, block_at)?,
                    partition: if mode == 0 {
                        address.partition
                    } else {
                        u16_at(raw, block_at + 4)?
                    },
                };
                if length == 0 {
                    if field != 0
                        || target.block != 0
                        || (mode != 0 && target.partition != 0)
                        || data[(index + 1) * stride..].iter().any(|byte| *byte != 0)
                    {
                        return Err(ScanError::Invalid);
                    }
                    break;
                }
                if kind == 3 {
                    if index + 1 != data.len() / stride
                        || length < 24
                        || length > self.sector
                        || !seen.insert(target)
                    {
                        return Err(ScanError::Invalid);
                    }
                    continuation = Some((target, length));
                    break;
                }
                let (recorded, information) = if mode == 2 {
                    let recorded = u64::from(u32_at(raw, 4)?);
                    let information = u64::from(u32_at(raw, 8)?);
                    if recorded > length || information > length {
                        return Err(ScanError::Invalid);
                    }
                    if kind == 0 && recorded != information {
                        return Err(ScanError::Unsupported);
                    }
                    if kind != 0 && recorded != 0 {
                        return Err(ScanError::Invalid);
                    }
                    (recorded, information)
                } else {
                    (if kind == 0 { length } else { 0 }, length)
                };
                let remaining = size.checked_sub(total).ok_or(ScanError::Invalid)?;
                if remaining == 0 {
                    // ECMA-167 4/12.1 permits a tail of preallocated unrecorded extents.
                    if kind != 1 {
                        return Err(ScanError::Invalid);
                    }
                    let spans = self.mapped_segments(reader, target, length, false)?;
                    count = count.checked_add(spans.len()).ok_or(ScanError::Limit)?;
                    if count > MAX_EXTENTS {
                        return Err(ScanError::Limit);
                    }
                    continue;
                }
                if last_partial {
                    return Err(ScanError::Invalid);
                }
                let used = information.min(remaining);
                if information == 0 || (information > remaining && mode == 2) {
                    return Err(ScanError::Invalid);
                }
                if information % self.sector != 0 && information < remaining {
                    last_partial = true;
                }
                if kind == 0 {
                    // Validate the whole allocated extent, including final block padding.
                    let mut mapped = self.mapped_segments(reader, target, recorded, true)?;
                    count = count.checked_add(mapped.len()).ok_or(ScanError::Limit)?;
                    if count > MAX_EXTENTS {
                        return Err(ScanError::Limit);
                    }
                    let mut keep = used;
                    for segment in &mut mapped {
                        if let Extent::Data { length, .. } = &mut segment.extent {
                            *length = (*length).min(keep);
                            keep -= *length;
                        }
                    }
                    mapped.retain(|segment| segment.length() != 0);
                    output.extend(mapped);
                } else {
                    if kind == 1 {
                        let spans = self.mapped_segments(reader, target, length, false)?;
                        count = count.checked_add(spans.len()).ok_or(ScanError::Limit)?;
                        if count > MAX_EXTENTS {
                            return Err(ScanError::Limit);
                        }
                    }
                    output.push(Segment {
                        extent: Extent::Zero { length: used },
                        location: target.block,
                        displacement: 0,
                    });
                }
                total = add(total, used)?;
                if output.len() > MAX_EXTENTS {
                    return Err(ScanError::Limit);
                }
            }
            let Some((target, length)) = continuation else {
                break;
            };
            if seen.len() > MAX_CHAIN {
                return Err(ScanError::Limit);
            }
            let next = self.read_mapped(reader, target, length)?;
            let ad_length = u32_at(&next, 20)? as usize;
            if ad_length == 0 {
                return Err(ScanError::Invalid);
            }
            let end = 24usize.checked_add(ad_length).ok_or(ScanError::Invalid)?;
            if tag(&next, target.block, end, true)? != 258
                || !matches!(usize::from(u16_at(&next, 10)?), 8)
                    && usize::from(u16_at(&next, 10)?) != end - 16
            {
                return Err(ScanError::Invalid);
            }
            data = bytes(&next, 24, ad_length)?.to_vec();
            address = target;
        }
        if total != size {
            return Err(ScanError::Invalid);
        }
        Ok(output)
    }

    fn children(
        &self,
        reader: &mut ImageReader<'_>,
        file: &File,
        limits: ScanLimits,
    ) -> Result<Vec<(String, Allocation, bool)>> {
        let data = self.read_segments(reader, &file.segments, file.size)?;
        let mut position = 0usize;
        let mut segment_index = 0usize;
        let mut segment_start = 0u64;
        let mut result = Vec::new();
        let mut names = 0usize;
        while position < data.len() {
            reader.check()?;
            let header = bytes(&data, position, 38)?;
            let name_length = usize::from(header[19]);
            let implementation_length = usize::from(u16_at(header, 36)?);
            if implementation_length != 0 && implementation_length < 32 {
                return Err(ScanError::Invalid);
            }
            let raw_size = 38usize
                .checked_add(name_length)
                .and_then(|size| size.checked_add(implementation_length))
                .ok_or(ScanError::Invalid)?;
            let length = raw_size.checked_add(3).ok_or(ScanError::Invalid)? & !3;
            let descriptor = bytes(&data, position, length)?;
            while (position as u64)
                >= add(
                    segment_start,
                    file.segments
                        .get(segment_index)
                        .ok_or(ScanError::Invalid)?
                        .length(),
                )?
            {
                segment_start = add(segment_start, file.segments[segment_index].length())?;
                segment_index += 1;
            }
            let segment = file.segments.get(segment_index).ok_or(ScanError::Invalid)?;
            let logical = add(segment.displacement, position as u64 - segment_start)? / self.sector;
            let location = u32::try_from(add(u64::from(segment.location), logical)?)
                .map_err(|_| ScanError::Invalid)?;
            if tag(descriptor, location, length, false)? != 257
                || u16_at(descriptor, 16)? != 1
                || descriptor[raw_size..].iter().any(|byte| *byte != 0)
            {
                return Err(ScanError::Invalid);
            }
            let characteristics = descriptor[18];
            if characteristics & 0xf0 != 0 {
                return Err(ScanError::Unsupported);
            }
            let allocation = long_ad(descriptor, 20)?;
            let directory = characteristics & 2 != 0;
            if characteristics & 8 != 0 {
                if name_length != 0 || !directory {
                    return Err(ScanError::Invalid);
                }
            } else if characteristics & 4 == 0 {
                if name_length < 2 {
                    return Err(ScanError::Invalid);
                }
                let name = cs0(bytes(descriptor, 38 + implementation_length, name_length)?)?;
                names = names.checked_add(name.len()).ok_or(ScanError::Limit)?;
                if result.len() >= limits.entries.min(xdzip_codec_api::MAX_ENTRIES)
                    || name.len() > limits.name_bytes.min(4096)
                    || names > limits.total_name_bytes
                {
                    return Err(ScanError::Limit);
                }
                result.push((name, allocation, directory));
            }
            position = position.checked_add(length).ok_or(ScanError::Invalid)?;
        }
        Ok(result)
    }
}

fn month_days(year: u16, month: u8) -> u8 {
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    }
}

fn timestamp(data: &[u8], at: usize) -> Result<u32> {
    let value = bytes(data, at, 12)?;
    let mut year = u16_at(value, 2)?;
    let mut month = value[4];
    let mut day = value[5];
    let type_zone = u16_at(value, 0)?;
    let zone = ((type_zone as i16) << 4) >> 4;
    if !(1979..=2108).contains(&year)
        || day == 0
        || day > month_days(year, month)
        || value[6] > 23
        || value[7] > 59
        || value[8] > 59
        || value[9..].iter().any(|part| *part > 99)
        || type_zone >> 12 > 1
        || (!(-1440..=1440).contains(&zone) && zone != -2047)
    {
        return Ok(0);
    }
    // OSTA 2.1.4: type 1 is local time and -2047 is unspecified. Follow its
    // recommended UTC interpretation of unspecified zones. Known offsets are
    // normalized to UTC, matching other non-ZIP codec timestamps.
    let offset = if type_zone >> 12 == 1 && zone != -2047 {
        i32::from(zone)
    } else {
        0
    };
    let mut minutes = i32::from(value[6]) * 60 + i32::from(value[7]) - offset;
    if minutes < 0 {
        minutes += 1440;
        if day > 1 {
            day -= 1;
        } else {
            if month > 1 {
                month -= 1;
            } else {
                year -= 1;
                month = 12;
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
                year += 1;
                month = 1;
            }
        }
    }
    if !(1980..=2107).contains(&year) {
        return Ok(0);
    }
    Ok((u32::from(year - 1980) << 25)
        | (u32::from(month) << 21)
        | (u32::from(day) << 16)
        | ((minutes as u32 / 60) << 11)
        | ((minutes as u32 % 60) << 5)
        | u32::from(value[8] / 2))
}

fn namespace(reader: &mut ImageReader<'_>, volume: &Volume, limits: ScanLimits) -> Result<Index> {
    let root = volume.file(reader, volume.root)?;
    if root.kind != 4 || root.links == 0 {
        return Err(ScanError::Invalid);
    }
    enum Work {
        Visit {
            name: String,
            reference: Allocation,
            directory: bool,
            unsafe_name: bool,
        },
        Leave(Address),
    }
    let mut builder = IndexBuilder::new(Format::Udf, limits);
    let mut active = BTreeSet::new();
    active.insert(volume.root.address);
    let children = volume.children(reader, &root, limits)?;
    let mut work: Vec<Work> = children
        .into_iter()
        .rev()
        .map(|(name, reference, directory)| {
            let unsafe_name = name.contains(['/', '\\']);
            Work::Visit {
                name,
                reference,
                directory,
                unsafe_name,
            }
        })
        .collect();
    let mut queued_names = work
        .iter()
        .map(|item| match item {
            Work::Visit { name, .. } => name.len(),
            _ => 0,
        })
        .sum::<usize>();
    while let Some(item) = work.pop() {
        reader.check()?;
        let Work::Visit {
            mut name,
            reference,
            directory,
            unsafe_name,
        } = item
        else {
            if let Work::Leave(address) = item {
                active.remove(&address);
            }
            continue;
        };
        queued_names -= name.len();
        if name.len() > limits.name_bytes.min(4096) {
            return Err(ScanError::Limit);
        }
        let file = volume.file(reader, reference)?;
        if matches!(file.kind, 4 | 5 | 12) && file.links == 0 {
            return Err(ScanError::Invalid);
        }
        if (file.kind == 4) != directory {
            return Err(ScanError::Invalid);
        }
        if directory && !active.insert(reference.address) {
            return Err(ScanError::Invalid);
        }
        let link = file.kind == 12 || (!directory && file.links > 1);
        let special = unsafe_name || !matches!(file.kind, 4 | 5 | 12);
        if directory {
            name.push('/');
        }
        builder.push(Node {
            name: name.as_bytes().to_vec(),
            directory,
            link,
            special,
            size: if directory { 0 } else { file.size },
            modified_dos: file.modified_dos,
            extents: if directory || link || special {
                Vec::new()
            } else {
                file.segments
                    .iter()
                    .map(|part| part.extent.clone())
                    .collect()
            },
        })?;
        if directory {
            if active.len() > limits.name_bytes.min(4096) {
                return Err(ScanError::Limit);
            }
            work.push(Work::Leave(reference.address));
            for (child, reference, directory) in
                volume.children(reader, &file, limits)?.into_iter().rev()
            {
                let full_length = name
                    .len()
                    .checked_add(child.len())
                    .ok_or(ScanError::Limit)?;
                queued_names = queued_names
                    .checked_add(full_length)
                    .ok_or(ScanError::Limit)?;
                if full_length > limits.name_bytes.min(4096)
                    || queued_names > limits.total_name_bytes
                    || work.len() >= limits.entries.min(xdzip_codec_api::MAX_ENTRIES) * 2
                {
                    return Err(ScanError::Limit);
                }
                let unsafe_name = unsafe_name || child.contains(['/', '\\']);
                work.push(Work::Visit {
                    name: format!("{name}{child}"),
                    reference,
                    directory,
                    unsafe_name,
                });
            }
        }
    }
    Ok(builder.finish())
}

pub(crate) fn scan(reader: &mut ImageReader<'_>, limits: ScanLimits) -> Result<Index> {
    let mut volume = Volume::discover(reader, limits)?;
    match namespace(reader, &volume, limits) {
        Err(ScanError::Invalid) => {
            // Retry the complete namespace from its metadata mirror with the
            // same cancellation and cumulative input budget.
            if volume.swap_metadata_mirrors() {
                namespace(reader, &volume, limits)
            } else {
                Err(ScanError::Invalid)
            }
        }
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use xdzip_codec_api::Cancellation;

    const BLOCK: usize = 2048;

    fn put16(data: &mut [u8], at: usize, value: u16) {
        data[at..at + 2].copy_from_slice(&value.to_le_bytes());
    }
    fn put32(data: &mut [u8], at: usize, value: u32) {
        data[at..at + 4].copy_from_slice(&value.to_le_bytes());
    }
    fn put64(data: &mut [u8], at: usize, value: u64) {
        data[at..at + 8].copy_from_slice(&value.to_le_bytes());
    }

    fn seal(data: &mut [u8], id: u16, location: u32, size: usize) {
        data[..16].fill(0);
        put16(data, 0, id);
        put16(data, 2, 3);
        put16(data, 6, 1);
        put16(data, 10, (size - 16) as u16);
        put32(data, 12, location);
        put16(data, 8, crc16(&data[16..size]));
        data[4] = data[..16]
            .iter()
            .fold(0u8, |sum, byte| sum.wrapping_add(*byte));
    }

    fn set_charspec(data: &mut [u8], at: usize) {
        data[at + 1..at + 24].copy_from_slice(b"OSTA Compressed Unicode");
    }
    fn set_entity(data: &mut [u8], at: usize, name: &[u8]) {
        data[at + 1..at + 1 + name.len()].copy_from_slice(name);
        put16(data, at + 24, 0x0260);
    }
    fn set_dstring(data: &mut [u8], at: usize, width: usize, name: &[u8]) {
        data[at] = 8;
        data[at + 1..at + 1 + name.len()].copy_from_slice(name);
        data[at + width - 1] = (name.len() + 1) as u8;
    }
    fn set_long(data: &mut [u8], at: usize, size: u32, block: u32, partition: u16) {
        put32(data, at, size);
        put32(data, at + 4, block);
        put16(data, at + 8, partition);
    }

    fn file_entry(
        kind: u8,
        mode: u16,
        size: u64,
        allocation: &[u8],
        extended: bool,
        block: u32,
    ) -> Vec<u8> {
        let mut data = vec![0u8; BLOCK];
        let base = if extended { 216 } else { 176 };
        put16(&mut data, 20, 4);
        put16(&mut data, 24, 1);
        data[27] = kind;
        put16(&mut data, 34, mode);
        put16(&mut data, 48, 1);
        put64(&mut data, 56, size);
        if extended {
            put64(&mut data, 64, size);
        }
        put32(&mut data, base - 4, allocation.len() as u32);
        data[base..base + allocation.len()].copy_from_slice(allocation);
        seal(
            &mut data,
            if extended { 266 } else { 261 },
            block,
            base + allocation.len(),
        );
        data
    }

    fn fid(name: &str, target: u32, directory: bool, block: u32) -> Vec<u8> {
        let mut encoded = vec![16];
        for word in name.encode_utf16() {
            encoded.extend_from_slice(&word.to_be_bytes());
        }
        let size = (38 + encoded.len() + 3) & !3;
        let mut data = vec![0; size];
        put16(&mut data, 16, 1);
        data[18] = if directory { 2 } else { 0 };
        data[19] = encoded.len() as u8;
        set_long(&mut data, 20, BLOCK as u32, target, 0);
        data[38..38 + encoded.len()].copy_from_slice(&encoded);
        seal(&mut data, 257, block, size);
        data
    }

    fn simple_image() -> Vec<u8> {
        let mut image = vec![0u8; BLOCK * 600];
        for (block, identifier) in [(16, b"BEA01"), (17, b"NSR03"), (18, b"TEA01")] {
            image[block * BLOCK + 1..block * BLOCK + 6].copy_from_slice(identifier);
            image[block * BLOCK + 6] = 1;
        }
        let mut pvd = vec![0; BLOCK];
        put32(&mut pvd, 16, 1);
        set_dstring(&mut pvd, 24, 32, b"XDZIP_TEST");
        put16(&mut pvd, 56, 1);
        put16(&mut pvd, 58, 1);
        put16(&mut pvd, 60, 2);
        put16(&mut pvd, 62, 3);
        put32(&mut pvd, 64, 1);
        put32(&mut pvd, 68, 1);
        set_dstring(&mut pvd, 72, 128, b"XDZIP_TEST");
        set_charspec(&mut pvd, 200);
        set_charspec(&mut pvd, 264);
        seal(&mut pvd, 1, 32, 512);
        image[32 * BLOCK..33 * BLOCK].copy_from_slice(&pvd);
        let mut pd = vec![0; BLOCK];
        put32(&mut pd, 16, 2);
        put16(&mut pd, 20, 1);
        set_entity(&mut pd, 24, b"+NSR03");
        put32(&mut pd, 184, 1);
        put32(&mut pd, 188, 300);
        put32(&mut pd, 192, 250);
        seal(&mut pd, 5, 33, 512);
        image[33 * BLOCK..34 * BLOCK].copy_from_slice(&pd);
        let mut lvd = vec![0; BLOCK];
        put32(&mut lvd, 16, 3);
        set_charspec(&mut lvd, 20);
        set_dstring(&mut lvd, 84, 128, b"XDZIP_TEST");
        put32(&mut lvd, 212, BLOCK as u32);
        set_entity(&mut lvd, 216, b"*OSTA UDF Compliant");
        set_long(&mut lvd, 248, (2 * BLOCK) as u32, 0, 0);
        put32(&mut lvd, 264, 6);
        put32(&mut lvd, 268, 1);
        lvd[440..446].copy_from_slice(&[1, 6, 1, 0, 0, 0]);
        seal(&mut lvd, 6, 34, 512);
        image[34 * BLOCK..35 * BLOCK].copy_from_slice(&lvd);
        let mut terminator = vec![0; BLOCK];
        seal(&mut terminator, 8, 35, 512);
        image[35 * BLOCK..36 * BLOCK].copy_from_slice(&terminator);
        let mut anchor = vec![0; BLOCK];
        put32(&mut anchor, 16, (16 * BLOCK) as u32);
        put32(&mut anchor, 20, 32);
        seal(&mut anchor, 2, 256, 512);
        image[256 * BLOCK..257 * BLOCK].copy_from_slice(&anchor);
        let mut fsd = vec![0; BLOCK];
        put16(&mut fsd, 28, 3);
        put16(&mut fsd, 30, 3);
        put32(&mut fsd, 32, 1);
        put32(&mut fsd, 36, 1);
        set_charspec(&mut fsd, 48);
        set_dstring(&mut fsd, 112, 128, b"XDZIP_TEST");
        set_charspec(&mut fsd, 240);
        set_dstring(&mut fsd, 304, 32, b"XDZIP_TEST");
        set_entity(&mut fsd, 416, b"*OSTA UDF Compliant");
        set_long(&mut fsd, 400, BLOCK as u32, 2, 0);
        seal(&mut fsd, 256, 0, 512);
        image[300 * BLOCK..301 * BLOCK].copy_from_slice(&fsd);
        seal(&mut terminator, 8, 1, 512);
        image[301 * BLOCK..302 * BLOCK].copy_from_slice(&terminator);
        let directory = fid("文档.txt", 3, false, 2);
        let root = file_entry(4, 3, directory.len() as u64, &directory, false, 2);
        image[302 * BLOCK..303 * BLOCK].copy_from_slice(&root);
        let file = file_entry(5, 3, 5, b"hello", true, 3);
        image[303 * BLOCK..304 * BLOCK].copy_from_slice(&file);
        image
    }

    fn scan_bytes(image: Vec<u8>) -> Result<Index> {
        let mut source = Cursor::new(image);
        let cancellation = Cancellation::default();
        let limits = ScanLimits::default();
        let mut reader = ImageReader::new(&mut source, &cancellation, limits.read_bytes)?;
        scan(&mut reader, limits)
    }

    fn physical_volume() -> Volume {
        Volume {
            sector: BLOCK as u64,
            maps: vec![Map::Physical(Partition {
                start: 0,
                blocks: 100,
            })],
            root: Allocation {
                address: Address {
                    partition: 0,
                    block: 0,
                },
                length: BLOCK as u64,
                kind: 0,
            },
            metadata_limit: MAX_METADATA,
        }
    }

    fn add_type2_map(image: &mut [u8], identifier: &[u8], virtual_namespace: bool) {
        let logical = &mut image[34 * BLOCK..35 * BLOCK];
        put32(logical, 264, 70);
        put32(logical, 268, 2);
        let map = &mut logical[446..510];
        map[0] = 2;
        map[1] = 64;
        set_entity(map, 4, identifier);
        put16(map, 36, 1);
        if virtual_namespace {
            put16(logical, 256, 1);
        }
        seal(logical, 6, 34, 512);
    }

    fn namespace_partition(image: &mut [u8], physical: usize, partition: u16) {
        let fsd = &mut image[physical * BLOCK..(physical + 1) * BLOCK];
        put16(fsd, 408, partition);
        seal(fsd, 256, 0, 512);
        let root = &mut image[(physical + 2) * BLOCK..(physical + 3) * BLOCK];
        put16(root, 176 + 28, partition);
        let fid_length = u32_at(root, 172).unwrap() as usize;
        seal(&mut root[176..176 + fid_length], 257, 2, fid_length);
        seal(root, 261, 2, 176 + fid_length);
    }

    #[test]
    fn metadata_files_and_mirrors_support_full_namespace_recovery() {
        let mut image = simple_image();
        add_type2_map(&mut image, b"*UDF Metadata Partition", true);
        let logical = &mut image[34 * BLOCK..35 * BLOCK];
        put32(logical, 446 + 40, 4);
        put32(logical, 446 + 44, 5);
        put32(logical, 446 + 48, u32::MAX);
        put32(logical, 446 + 52, 32);
        put16(logical, 446 + 56, 1);
        logical[446 + 58] = 1;
        seal(logical, 6, 34, 512);
        namespace_partition(&mut image, 300, 1);
        let original = image[300 * BLOCK..304 * BLOCK].to_vec();
        image[332 * BLOCK..336 * BLOCK].copy_from_slice(&original);
        image[364 * BLOCK..368 * BLOCK].copy_from_slice(&original);
        for (location, kind, data) in [(4, 250, 32), (5, 251, 64)] {
            let mut allocation = [0; 8];
            put32(&mut allocation, 0, (32 * BLOCK) as u32);
            put32(&mut allocation, 4, data);
            let mut file = file_entry(kind, 0, (32 * BLOCK) as u64, &allocation, false, location);
            put16(&mut file, 48, 0);
            seal(&mut file, 261, location, 184);
            image[(300 + location as usize) * BLOCK..(301 + location as usize) * BLOCK]
                .copy_from_slice(&file);
        }
        let index = scan_bytes(image.clone()).unwrap();
        assert_eq!(
            index.records[0].extents,
            vec![Extent::Inline(b"hello".to_vec())]
        );
        // Bad root FE is recovered from the independently allocated mirror.
        image[334 * BLOCK + 56] ^= 1;
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        // Bad FSD is also recovered before locating the namespace root.
        image[332 * BLOCK + 400] ^= 1;
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        image[364 * BLOCK + 400] ^= 1;
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
    }

    #[test]
    fn virtual_allocation_tables_150_and_260_select_logical_namespace() {
        for revision in [0x0150, 0x0260] {
            let mut image = simple_image();
            add_type2_map(&mut image, b"*UDF Virtual Partition", true);
            let logical = &mut image[34 * BLOCK..35 * BLOCK];
            put16(logical, 446 + 28, revision);
            seal(logical, 6, 34, 512);
            namespace_partition(&mut image, 300, 1);
            let mut vat = if revision == 0x0150 {
                vec![0; 16 + 36]
            } else {
                vec![0; 152 + 16]
            };
            let table_at = if revision == 0x0150 { 0 } else { 152 };
            for value in 0..4u32 {
                put32(&mut vat, table_at + value as usize * 4, value);
            }
            if revision == 0x0150 {
                set_entity(&mut vat, 16, b"*UDF Virtual Alloc Tbl");
                put32(&mut vat, 48, u32::MAX);
            } else {
                put16(&mut vat, 0, 152);
                put32(&mut vat, 132, u32::MAX);
                put16(&mut vat, 144, 0x0200);
            }
            let file = file_entry(
                if revision == 0x0150 { 0 } else { 248 },
                3,
                vat.len() as u64,
                &vat,
                false,
                249,
            );
            image[549 * BLOCK..550 * BLOCK].copy_from_slice(&file);
            assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
            let location = 549 * BLOCK + 176 + table_at + 2 * 4;
            put32(&mut image, location, u32::MAX);
            let file = &mut image[549 * BLOCK..550 * BLOCK];
            seal(file, 261, 249, 176 + vat.len());
            assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
        }
    }

    #[test]
    fn sparing_table_remaps_packets_and_validates_redundancy() {
        let mut image = simple_image();
        let original = image[300 * BLOCK..304 * BLOCK].to_vec();
        image[416 * BLOCK..420 * BLOCK].copy_from_slice(&original);
        let pd = &mut image[33 * BLOCK..34 * BLOCK];
        put32(pd, 188, 320);
        put32(pd, 192, 256);
        seal(pd, 5, 33, 512);
        let logical = &mut image[34 * BLOCK..35 * BLOCK];
        put32(logical, 264, 64);
        logical[440..].fill(0);
        let map = &mut logical[440..504];
        map[0] = 2;
        map[1] = 64;
        set_entity(map, 4, b"*UDF Sparable Partition");
        put16(map, 36, 1);
        put16(map, 40, 32);
        map[42] = 2;
        put32(map, 44, BLOCK as u32);
        put32(map, 48, 64);
        put32(map, 52, 80);
        seal(logical, 6, 34, 512);
        let mut table = vec![0; BLOCK];
        set_entity(&mut table, 16, b"*UDF Sparing Table");
        put16(&mut table, 48, 1);
        put32(&mut table, 52, 1);
        put32(&mut table, 60, 416);
        seal(&mut table, 0, 64, 64);
        image[64 * BLOCK..65 * BLOCK].copy_from_slice(&table);
        seal(&mut table, 0, 80, 64);
        image[80 * BLOCK..81 * BLOCK].copy_from_slice(&table);
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        image[64 * BLOCK + 60] ^= 1;
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        image[80 * BLOCK + 60] ^= 1;
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
    }

    #[test]
    fn volume_descriptor_continuation_and_cycle_are_bounded() {
        let mut image = simple_image();
        let mut logical = image[34 * BLOCK..35 * BLOCK].to_vec();
        seal(&mut logical, 6, 48, 512);
        image[48 * BLOCK..49 * BLOCK].copy_from_slice(&logical);
        let mut terminator = vec![0; BLOCK];
        seal(&mut terminator, 8, 49, 512);
        image[49 * BLOCK..50 * BLOCK].copy_from_slice(&terminator);
        let pointer = &mut image[34 * BLOCK..35 * BLOCK];
        pointer.fill(0);
        put32(pointer, 16, 3);
        put32(pointer, 20, (16 * BLOCK) as u32);
        put32(pointer, 24, 48);
        seal(pointer, 3, 34, 512);
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        let pointer = &mut image[48 * BLOCK..49 * BLOCK];
        pointer.fill(0);
        put32(pointer, 20, (16 * BLOCK) as u32);
        put32(pointer, 24, 32);
        seal(pointer, 3, 48, 512);
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
    }

    #[test]
    fn ecma_crc_vector_and_descriptor_integrity() {
        assert_eq!(crc16(&[0x70, 0x6a, 0x77]), 0x3299);
        let mut descriptor = vec![0; 512];
        seal(&mut descriptor, 2, 256, 512);
        assert_eq!(tag(&descriptor, 256, 512, false), Ok(2));
        assert_eq!(tag(&descriptor, 257, 512, false), Err(ScanError::Invalid));
        descriptor[100] ^= 1;
        assert_eq!(tag(&descriptor, 256, 512, false), Err(ScanError::Invalid));
        descriptor[100] ^= 1;
        descriptor[4] ^= 1;
        assert_eq!(tag(&descriptor, 256, 512, false), Err(ScanError::Invalid));
    }

    #[test]
    fn cs0_has_exact_unicode_and_rejects_malformed_encodings() {
        assert_eq!(cs0(&[8, b'a', 0xe9]).unwrap(), "aé");
        assert_eq!(cs0(&[16, 0x65, 0x87, 0x68, 0x63]).unwrap(), "文档");
        assert_eq!(cs0(&[16, 0xd8, 0]), Err(ScanError::Invalid));
        assert_eq!(cs0(&[16, 0]), Err(ScanError::Unsupported));
        assert_eq!(cs0(&[254, 1]), Err(ScanError::Unsupported));
    }

    #[test]
    fn scans_complete_volume_and_embedded_efe() {
        let index = scan_bytes(simple_image()).unwrap();
        assert_eq!(index.records.len(), 1);
        assert_eq!(index.records[0].entry.raw_name, "文档.txt".as_bytes());
        assert_eq!(index.records[0].entry.size, 5);
        assert_eq!(
            index.records[0].extents,
            vec![Extent::Inline(b"hello".to_vec())]
        );
    }

    #[test]
    fn variable_lvd_and_lvid_crc_cover_the_actual_table_end() {
        let mut image = simple_image();
        let logical = &mut image[34 * BLOCK..35 * BLOCK];
        put32(logical, 432, BLOCK as u32);
        put32(logical, 436, 64);
        // A one-map LVD ends at 446, not necessarily at a padded 512.
        seal(logical, 6, 34, 446);
        let integrity = &mut image[64 * BLOCK..65 * BLOCK];
        put32(integrity, 28, 1);
        put32(integrity, 72, 1);
        put32(integrity, 76, 46);
        put16(integrity, 80 + 8 + 40, 0x0102);
        // LVID: fixed 80 bytes + two Uint32 table values + 46-byte use area.
        seal(integrity, 9, 64, 134);
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        image[64 * BLOCK + 80] ^= 1;
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
    }

    #[test]
    fn repeated_file_set_descriptor_numbers_must_have_identical_bodies() {
        let mut image = simple_image();
        let mut duplicate = image[300 * BLOCK..301 * BLOCK].to_vec();
        seal(&mut duplicate, 256, 1, 512);
        image[301 * BLOCK..302 * BLOCK].copy_from_slice(&duplicate);
        // Redundant copies at different tag locations are valid.
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        // An alternative root is independently valid, but the same descriptor
        // number cannot silently keep the first namespace and hide the second.
        let alternative = file_entry(4, 3, 0, &[], false, 4);
        image[304 * BLOCK..305 * BLOCK].copy_from_slice(&alternative);
        set_long(&mut duplicate, 400, BLOCK as u32, 4, 0);
        seal(&mut duplicate, 256, 1, 512);
        image[301 * BLOCK..302 * BLOCK].copy_from_slice(&duplicate);
        assert!(matches!(scan_bytes(image.clone()), Err(ScanError::Invalid)));
        // The number is shared by the sequence, including different file sets.
        set_long(&mut duplicate, 400, BLOCK as u32, 2, 0);
        put32(&mut duplicate, 40, 1);
        seal(&mut duplicate, 256, 1, 512);
        image[301 * BLOCK..302 * BLOCK].copy_from_slice(&duplicate);
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
    }

    #[test]
    fn file_set_continuations_terminate_the_current_extent_before_trailing_blocks() {
        let mut image = simple_image();
        let mut first = image[300 * BLOCK..301 * BLOCK].to_vec();
        let mut following = first.clone();
        set_long(&mut first, 448, (2 * BLOCK) as u32, 20, 0);
        seal(&mut first, 256, 8, 512);
        put32(&mut following, 44, 1);
        seal(&mut following, 256, 20, 512);
        image[308 * BLOCK..309 * BLOCK].copy_from_slice(&first);
        // The remaining three blocks in the first extent are unrecorded.
        image[320 * BLOCK..321 * BLOCK].copy_from_slice(&following);
        let mut terminator = vec![0; BLOCK];
        seal(&mut terminator, 8, 21, 512);
        image[321 * BLOCK..322 * BLOCK].copy_from_slice(&terminator);
        let logical = &mut image[34 * BLOCK..35 * BLOCK];
        set_long(logical, 248, (4 * BLOCK) as u32, 8, 0);
        seal(logical, 6, 34, 512);
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        // Continuations still retain the same cycle rejection.
        set_long(&mut following, 448, (4 * BLOCK) as u32, 8, 0);
        seal(&mut following, 256, 20, 512);
        image[320 * BLOCK..321 * BLOCK].copy_from_slice(&following);
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
        let mut image = simple_image();
        image[301 * BLOCK..302 * BLOCK].fill(0);
        assert_eq!(scan_bytes(image).unwrap().records.len(), 1);
    }

    #[test]
    fn integrity_continuation_does_not_parse_trailing_stale_descriptors() {
        let mut image = simple_image();
        let logical = &mut image[34 * BLOCK..35 * BLOCK];
        put32(logical, 432, (4 * BLOCK) as u32);
        put32(logical, 436, 64);
        seal(logical, 6, 34, 512);
        let mut integrity = vec![0; BLOCK];
        put32(&mut integrity, 28, 1);
        put32(&mut integrity, 72, 1);
        put32(&mut integrity, 76, 46);
        put16(&mut integrity, 80 + 8 + 40, 0x0102);
        seal(&mut integrity, 9, 80, 134);
        image[80 * BLOCK..81 * BLOCK].copy_from_slice(&integrity);
        put32(&mut integrity, 32, (2 * BLOCK) as u32);
        put32(&mut integrity, 36, 80);
        seal(&mut integrity, 9, 64, 134);
        image[64 * BLOCK..65 * BLOCK].copy_from_slice(&integrity);
        // Nonzero stale data is legal after the continuation descriptor and
        // cannot replace its selected next extent or cause tag validation.
        image[65 * BLOCK..66 * BLOCK].fill(0x5a);
        assert_eq!(scan_bytes(image.clone()).unwrap().records.len(), 1);
        image[80 * BLOCK + 80] ^= 1;
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
    }

    #[test]
    fn multi_block_directory_and_multi_extent_file_retain_every_record() {
        let mut image = simple_image();
        let mut directory = Vec::new();
        for entry in 0..200 {
            let location = 10 + directory.len() as u32 / BLOCK as u32;
            directory.extend(fid(&format!("file{entry:03}"), 3, false, location));
        }
        let mut allocation = [0; 8];
        put32(&mut allocation, 0, directory.len() as u32);
        put32(&mut allocation, 4, 10);
        let root = file_entry(4, 0, directory.len() as u64, &allocation, false, 2);
        image[302 * BLOCK..303 * BLOCK].copy_from_slice(&root);
        image[310 * BLOCK..310 * BLOCK + directory.len()].copy_from_slice(&directory);
        let mut descriptors = [0; 32];
        set_long(&mut descriptors, 0, BLOCK as u32, 30, 0);
        set_long(&mut descriptors, 16, 5, 40, 0);
        let file = file_entry(5, 1, BLOCK as u64 + 5, &descriptors, false, 3);
        image[303 * BLOCK..304 * BLOCK].copy_from_slice(&file);
        image[330 * BLOCK..331 * BLOCK].fill(b'A');
        image[340 * BLOCK..340 * BLOCK + 5].copy_from_slice(b"hello");
        let index = scan_bytes(image).unwrap();
        assert_eq!(index.records.len(), 200);
        assert_eq!(index.records[199].entry.raw_name, b"file199");
        assert_eq!(
            index.records[0].extents,
            vec![
                Extent::Data {
                    offset: (330 * BLOCK) as u64,
                    length: BLOCK as u64
                },
                Extent::Data {
                    offset: (340 * BLOCK) as u64,
                    length: 5
                },
            ]
        );
    }

    #[test]
    fn icb_strategy_4096_chooses_latest_direct_entry_and_detects_cycles() {
        let mut image = simple_image();
        let root = &mut image[302 * BLOCK..303 * BLOCK];
        put32(root, 176 + 20, (2 * BLOCK) as u32);
        let fid_length = u32_at(root, 172).unwrap() as usize;
        seal(&mut root[176..176 + fid_length], 257, 2, fid_length);
        seal(root, 261, 2, 176 + fid_length);
        let initial = &mut image[303 * BLOCK..304 * BLOCK];
        put16(initial, 20, 4096);
        put16(initial, 22, 1);
        put16(initial, 24, 2);
        seal(initial, 266, 3, 221);
        let mut indirect = vec![0; BLOCK];
        put16(&mut indirect, 20, 4096);
        put16(&mut indirect, 22, 1);
        put16(&mut indirect, 24, 2);
        indirect[27] = 3;
        set_long(&mut indirect, 36, BLOCK as u32, 5, 0);
        seal(&mut indirect, 259, 4, 52);
        image[304 * BLOCK..305 * BLOCK].copy_from_slice(&indirect);
        let latest = file_entry(5, 3, 5, b"world", false, 5);
        image[305 * BLOCK..306 * BLOCK].copy_from_slice(&latest);
        assert_eq!(
            scan_bytes(image.clone()).unwrap().records[0].extents,
            vec![Extent::Inline(b"world".to_vec())]
        );
        set_long(&mut indirect, 36, (2 * BLOCK) as u32, 3, 0);
        seal(&mut indirect, 259, 4, 52);
        image[304 * BLOCK..305 * BLOCK].copy_from_slice(&indirect);
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
    }

    #[test]
    fn resource_limits_cancellation_and_required_unknown_features_fail_explicitly() {
        let image = simple_image();
        let mut source = Cursor::new(image.clone());
        let cancellation = Cancellation::default();
        let mut reader = ImageReader::new(&mut source, &cancellation, MAX_METADATA).unwrap();
        let limits = ScanLimits {
            name_bytes: 1,
            ..ScanLimits::default()
        };
        assert!(matches!(scan(&mut reader, limits), Err(ScanError::Limit)));
        cancellation.cancel();
        assert!(matches!(
            scan(&mut reader, ScanLimits::default()),
            Err(ScanError::Cancelled)
        ));
        let mut image = image;
        let file = &mut image[303 * BLOCK..304 * BLOCK];
        put16(file, 34, 3 | (1 << 11));
        seal(file, 266, 3, 221);
        assert!(matches!(scan_bytes(image), Err(ScanError::Unsupported)));
        let mut image = simple_image();
        let file = &mut image[303 * BLOCK..304 * BLOCK];
        set_long(file, 152, BLOCK as u32, 20, 0);
        seal(file, 266, 3, 221);
        assert!(matches!(scan_bytes(image), Err(ScanError::Unsupported)));
    }

    #[test]
    fn backup_anchor_and_reserve_sequence_recover_independently() {
        let mut image = simple_image();
        let mut anchor = image[256 * BLOCK..257 * BLOCK].to_vec();
        seal(&mut anchor, 2, 599, 512);
        image[599 * BLOCK..600 * BLOCK].copy_from_slice(&anchor);
        image[256 * BLOCK + 4] ^= 1;
        assert_eq!(scan_bytes(image).unwrap().records.len(), 1);
        let mut image = simple_image();
        for (from, to) in [(32, 48), (33, 49), (34, 50), (35, 51)] {
            let mut descriptor = image[from * BLOCK..(from + 1) * BLOCK].to_vec();
            let id = u16_at(&descriptor, 0).unwrap();
            seal(&mut descriptor, id, to as u32, 512);
            image[to * BLOCK..(to + 1) * BLOCK].copy_from_slice(&descriptor);
        }
        let anchor = &mut image[256 * BLOCK..257 * BLOCK];
        put32(anchor, 24, (16 * BLOCK) as u32);
        put32(anchor, 28, 48);
        seal(anchor, 2, 256, 512);
        image[32 * BLOCK + 200] ^= 1;
        assert_eq!(scan_bytes(image).unwrap().records.len(), 1);
    }

    #[test]
    fn sparse_short_long_and_extended_allocation_descriptors() {
        let mut image = vec![0; 100 * BLOCK];
        image[10 * BLOCK..10 * BLOCK + 3].copy_from_slice(b"abc");
        let control = Cancellation::default();
        let volume = physical_volume();
        for mode in [0, 1, 2] {
            let stride = [8, 16, 20][mode as usize];
            let mut descriptors = vec![0; stride * 2];
            put32(&mut descriptors, 0, 3);
            put32(&mut descriptors, stride, 0x8000_0000 | 7);
            if mode == 2 {
                put32(&mut descriptors, 4, 3);
                put32(&mut descriptors, 8, 3);
                put32(&mut descriptors, 12, 10);
                put32(&mut descriptors, stride + 8, 7);
            } else {
                put32(&mut descriptors, 4, 10);
            }
            let mut source = Cursor::new(image.clone());
            let mut reader = ImageReader::new(&mut source, &control, MAX_METADATA).unwrap();
            // The first extent must occupy complete blocks before another body extent.
            assert!(
                volume
                    .allocations(
                        &mut reader,
                        Address {
                            partition: 0,
                            block: 1
                        },
                        mode,
                        &descriptors,
                        10
                    )
                    .is_err()
            );
            let first_length = BLOCK as u32;
            put32(&mut descriptors, 0, first_length);
            if mode == 2 {
                put32(&mut descriptors, 4, first_length);
                put32(&mut descriptors, 8, first_length);
            }
            let parts = volume
                .allocations(
                    &mut reader,
                    Address {
                        partition: 0,
                        block: 1,
                    },
                    mode,
                    &descriptors,
                    BLOCK as u64 + 7,
                )
                .unwrap();
            assert_eq!(
                parts[0].extent,
                Extent::Data {
                    offset: (10 * BLOCK) as u64,
                    length: BLOCK as u64
                }
            );
            assert_eq!(parts[1].extent, Extent::Zero { length: 7 });
        }
    }

    #[test]
    fn allocation_continuation_validates_crc_and_cycle() {
        let mut image = vec![0; 100 * BLOCK];
        let mut next = vec![0; BLOCK];
        put32(&mut next, 20, 8);
        put32(&mut next, 24, 5);
        put32(&mut next, 28, 10);
        seal(&mut next, 258, 2, 32);
        image[2 * BLOCK..3 * BLOCK].copy_from_slice(&next);
        let mut initial = vec![0; 8];
        put32(&mut initial, 0, 0xc000_0000 | BLOCK as u32);
        put32(&mut initial, 4, 2);
        let control = Cancellation::default();
        let volume = physical_volume();
        let mut source = Cursor::new(image.clone());
        let mut reader = ImageReader::new(&mut source, &control, MAX_METADATA).unwrap();
        let parts = volume
            .allocations(
                &mut reader,
                Address {
                    partition: 0,
                    block: 1,
                },
                0,
                &initial,
                5,
            )
            .unwrap();
        assert_eq!(parts[0].length(), 5);
        image[2 * BLOCK + 24..2 * BLOCK + 32].copy_from_slice(&initial);
        seal(&mut image[2 * BLOCK..3 * BLOCK], 258, 2, 32);
        let mut source = Cursor::new(image);
        let mut reader = ImageReader::new(&mut source, &control, MAX_METADATA).unwrap();
        assert!(matches!(
            volume.allocations(
                &mut reader,
                Address {
                    partition: 0,
                    block: 1
                },
                0,
                &initial,
                5
            ),
            Err(ScanError::Invalid)
        ));
    }

    #[test]
    fn malformed_directory_cycle_and_truncated_image_fail() {
        let mut image = simple_image();
        let directory = fid("cycle", 2, true, 2);
        let root = file_entry(4, 3, directory.len() as u64, &directory, false, 2);
        image[302 * BLOCK..303 * BLOCK].copy_from_slice(&root);
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
        let mut image = simple_image();
        image.truncate(303 * BLOCK + 200);
        assert!(matches!(scan_bytes(image), Err(ScanError::Invalid)));
    }

    #[test]
    fn partition_translation_keeps_virtual_and_metadata_addresses_separate() {
        let mut volume = physical_volume();
        volume.maps.push(Map::Virtual {
            base: 0,
            table: vec![10, 20, u32::MAX],
        });
        assert_eq!(
            volume.map_block(1, 0).unwrap(),
            ((10 * BLOCK) as u64, BLOCK as u64)
        );
        assert_eq!(
            volume.map_block(1, 1).unwrap(),
            ((20 * BLOCK) as u64, BLOCK as u64)
        );
        assert_eq!(volume.map_block(1, 2), Err(ScanError::Invalid));
        volume.maps.push(Map::Metadata {
            segments: vec![
                Segment {
                    extent: Extent::Data {
                        offset: (30 * BLOCK) as u64,
                        length: BLOCK as u64,
                    },
                    location: 30,
                    displacement: 0,
                },
                Segment {
                    extent: Extent::Data {
                        offset: (40 * BLOCK) as u64,
                        length: BLOCK as u64,
                    },
                    location: 40,
                    displacement: 0,
                },
            ],
            ends: vec![BLOCK as u64, (2 * BLOCK) as u64],
            mirror: None,
            mirror_ends: None,
            size: (2 * BLOCK) as u64,
        });
        assert_eq!(
            volume.map_block(2, 1).unwrap(),
            ((40 * BLOCK) as u64, BLOCK as u64)
        );
        volume.maps.push(Map::Sparable {
            partition: Partition {
                start: 100,
                blocks: 100,
            },
            packet: 32,
            relocated: BTreeMap::from([(32, 500)]),
        });
        assert_eq!(
            volume.map_block(3, 33).unwrap(),
            ((501 * BLOCK) as u64, (31 * BLOCK) as u64)
        );
        assert_eq!(
            volume.map_block(3, 1).unwrap(),
            ((101 * BLOCK) as u64, (31 * BLOCK) as u64)
        );
    }

    #[test]
    fn modified_timestamp_normalizes_known_zones_and_checks_calendar() {
        let mut value = [0u8; 12];
        put16(&mut value, 2, 2024);
        value[4..9].copy_from_slice(&[2, 29, 23, 59, 58]);
        assert_ne!(timestamp(&value, 0).unwrap(), 0);
        put16(&mut value, 2, 2023);
        assert_eq!(timestamp(&value, 0).unwrap(), 0);
        put16(&mut value, 2, 2024);
        put16(&mut value, 0, 0x1000 | 480);
        value[4..9].copy_from_slice(&[3, 1, 2, 0, 0]);
        let utc = timestamp(&value, 0).unwrap();
        put16(&mut value, 0, 0x1000);
        value[4..9].copy_from_slice(&[2, 29, 18, 0, 0]);
        assert_eq!(timestamp(&value, 0).unwrap(), utc);
        put16(&mut value, 0, 0x1801);
        assert_eq!(timestamp(&value, 0).unwrap(), utc);
    }

    #[test]
    #[ignore = "requires a separately prepared legal external fixture path"]
    fn inspect_external_fixture() {
        let path = std::env::var_os("XDZIP_UDF_IMAGE").expect("XDZIP_UDF_IMAGE");
        let image = std::fs::read(path).unwrap();
        let index = scan_bytes(image).unwrap();
        for record in index.records {
            println!(
                "{} {} {}",
                record.entry.display_name, record.entry.size, record.entry.link
            );
        }
    }

    #[test]
    #[ignore = "exports an independently encoded format vector to a local build artifact"]
    fn export_specification_image() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../build/optical-specs");
        std::fs::create_dir_all(&root).unwrap();
        std::fs::write(root.join("udf-specification-vector.iso"), simple_image()).unwrap();
    }
}
