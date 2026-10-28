use crate::source::Reader;
use std::collections::{BTreeMap, VecDeque};
use xdzip_codec_api::names::NameEncoding;
use xdzip_codec_api::{Entry, Format, ScanError, ScanLimits};
use xdzip_domain::identity::EntryId;

const MAX_BLOCKS: usize = 1_000_000;
const MAX_FOLDERS: usize = 100_000;

pub(crate) fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
pub(crate) fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(crate) enum Method {
    Store,
    Mszip,
    Lzx(u8),
    Quantum(u8, u8),
}
impl Method {
    fn parse(raw: u16) -> Result<Self, ScanError> {
        let bits = ((raw >> 8) & 31) as u8;
        let level = ((raw >> 4) & 15) as u8;
        if raw & 0xe000 != 0 {
            return Err(ScanError::Unsupported);
        }
        match raw & 15 {
            0 if raw == 0 => Ok(Self::Store),
            1 if raw == 1 => Ok(Self::Mszip),
            2 if (10..=21).contains(&bits) && (1..=7).contains(&level) => {
                Ok(Self::Quantum(bits, level))
            }
            3 if (15..=21).contains(&bits) && level == 0 => Ok(Self::Lzx(bits)),
            _ => Err(ScanError::Unsupported),
        }
    }
    pub(crate) fn packed_limit(self) -> u64 {
        match self {
            Self::Store => 32768,
            Self::Mszip => 32780,
            Self::Lzx(_) => 38912,
            // Quantum has no published maximum compressed expansion. Admit a
            // bounded complete frame, independently of its physical split count.
            Self::Quantum(_, _) => 1024 * 1024,
        }
    }
}
pub(crate) struct Fragment {
    pub(crate) volume: usize,
    pub(crate) offset: u64,
    pub(crate) packed: u16,
    pub(crate) unpacked: u16,
    pub(crate) checksum: u32,
    pub(crate) reserve: u8,
}
pub(crate) struct Frame {
    pub(crate) fragments: Vec<Fragment>,
    pub(crate) start: u64,
    pub(crate) unpacked: u16,
}
pub(crate) struct Folder {
    pub(crate) method: Method,
    pub(crate) frames: Vec<Frame>,
    pub(crate) size: u64,
    pub(crate) last_record: Option<usize>,
    pending: Vec<Fragment>,
    packed: u64,
}
pub(crate) struct File {
    pub(crate) folder: usize,
    pub(crate) offset: u64,
    pub(crate) attributes: u16,
}
pub(crate) struct Index {
    pub(crate) rows: Vec<Entry>,
    pub(crate) files: Vec<File>,
    pub(crate) folders: Vec<Folder>,
}
struct LocalFolder {
    start: u64,
    blocks: u16,
    method: Method,
    from: bool,
    to: bool,
}
struct RawFile {
    folder: usize,
    offset: u64,
    size: u64,
    attributes: u16,
    modified: u32,
    name: Vec<u8>,
    from: bool,
    to: bool,
}
#[derive(Eq, PartialEq, Ord, PartialOrd)]
struct Key {
    folder: usize,
    offset: u64,
    size: u64,
    attributes: u16,
    modified: u32,
    name: Vec<u8>,
}
fn add(offset: u64, count: u64) -> Result<u64, ScanError> {
    offset.checked_add(count).ok_or(ScanError::Invalid)
}
fn name_matches(actual: &[u8], expected: &[u8]) -> bool {
    actual.eq_ignore_ascii_case(expected)
}

/// CAB fields have no ABI padding. Offsets stay relative to their authorized
/// physical cabinet until the bounded source translates them for a read.
pub(crate) fn parse(reader: &mut Reader<'_>, limits: ScanLimits) -> Result<Index, ScanError> {
    let mut index = Index {
        rows: Vec::new(),
        files: Vec::new(),
        folders: Vec::new(),
    };
    let mut set_id = None;
    let mut active: BTreeMap<Key, VecDeque<usize>> = BTreeMap::new();
    let mut origins = Vec::new();
    let mut volume_indices = BTreeMap::new();
    for (number, volume) in reader.volumes.iter().enumerate() {
        if volume_indices
            .insert(volume.name.to_ascii_lowercase(), number)
            .is_some()
        {
            return Err(ScanError::Invalid);
        }
    }
    let mut previous_folder = None;
    let mut physical_files = 0usize;
    let mut names = 0usize;
    let mut blocks = 0usize;
    for volume in 0..reader.volumes.len() {
        reader.check()?;
        let header = reader.bytes(volume, 0, 36)?;
        let cabinet_size = u64::from(u32_at(&header, 8));
        if &header[..4] != b"MSCF"
            || header[24..26] != [3, 1]
            || [4, 12, 20].iter().any(|at| u32_at(&header, *at) != 0)
            || cabinet_size < 36
            || cabinet_size > reader.volumes[volume].length
        {
            return Err(ScanError::Invalid);
        }
        reader.volumes[volume].length = cabinet_size;
        let folders_count = usize::from(u16_at(&header, 26));
        let files_count = usize::from(u16_at(&header, 28));
        if files_count
            > limits
                .entries
                .min(xdzip_codec_api::MAX_ENTRIES)
                .saturating_sub(physical_files)
        {
            return Err(ScanError::Limit);
        }
        let flags = u16_at(&header, 30);
        let current_set = u16_at(&header, 32);
        if usize::from(u16_at(&header, 34)) != volume
            || set_id.is_some_and(|set| set != current_set)
        {
            return Err(ScanError::Invalid);
        }
        set_id = Some(current_set);
        if (flags & 1 != 0) != (volume > 0)
            || (flags & 2 != 0) != (volume + 1 < reader.volumes.len())
        {
            return Err(ScanError::MissingVolume);
        }
        let mut cursor = 36u64;
        let (folder_reserve, data_reserve) = if flags & 4 != 0 {
            let reserve = reader.bytes(volume, cursor, 4)?;
            let header_reserve = u16_at(&reserve, 0);
            if header_reserve > 60000 {
                return Err(ScanError::Invalid);
            }
            cursor = add(cursor, 4 + u64::from(header_reserve))?;
            reader.range(volume, cursor, 0)?;
            (reserve[2], reserve[3])
        } else {
            (0, 0)
        };
        let previous = if flags & 1 != 0 {
            let name = reader.string(volume, &mut cursor, 4096)?;
            let _label = reader.string(volume, &mut cursor, 4096)?;
            let number = *volume_indices
                .get(&name.to_ascii_lowercase())
                .ok_or(ScanError::Invalid)?;
            if number >= volume {
                return Err(ScanError::Invalid);
            }
            Some(number)
        } else {
            None
        };
        if flags & 2 != 0 {
            let name = reader.string(volume, &mut cursor, 4096)?;
            let _label = reader.string(volume, &mut cursor, 4096)?;
            let expected = &reader
                .volumes
                .get(volume + 1)
                .ok_or(ScanError::MissingVolume)?
                .name;
            if !name_matches(&name, expected) {
                return Err(ScanError::Invalid);
            }
        }
        let mut local = Vec::with_capacity(folders_count);
        let table_end = add(
            cursor,
            (folders_count as u64) * (8 + u64::from(folder_reserve)),
        )?;
        reader.range(volume, table_end, 0)?;
        for _ in 0..folders_count {
            let raw = reader.bytes(volume, cursor, 8)?;
            local.push(LocalFolder {
                start: u64::from(u32_at(&raw, 0)),
                blocks: u16_at(&raw, 4),
                method: Method::parse(u16_at(&raw, 6))?,
                from: false,
                to: false,
            });
            cursor = add(cursor, 8 + u64::from(folder_reserve))?;
        }
        let files_start = u64::from(u32_at(&header, 16));
        if files_start < table_end {
            return Err(ScanError::Invalid);
        }
        cursor = files_start;
        let mut raw_files = Vec::with_capacity(files_count);
        for _ in 0..files_count {
            reader.check()?;
            physical_files = physical_files.checked_add(1).ok_or(ScanError::Limit)?;
            if physical_files > limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
                return Err(ScanError::Limit);
            }
            let raw = reader.bytes(volume, cursor, 16)?;
            cursor = add(cursor, 16)?;
            let name = reader.string(volume, &mut cursor, limits.name_bytes.min(4096))?;
            names = names.checked_add(name.len()).ok_or(ScanError::Limit)?;
            if names > limits.total_name_bytes {
                return Err(ScanError::Limit);
            }
            let value = u16_at(&raw, 8);
            let from = matches!(value, 0xfffd | 0xffff);
            let to = matches!(value, 0xfffe | 0xffff);
            let folder = match value {
                0xfffd => 0,
                0xfffe => folders_count.checked_sub(1).ok_or(ScanError::Invalid)?,
                0xffff if folders_count == 1 => 0,
                0xffff => return Err(ScanError::Invalid),
                value => usize::from(value),
            };
            let fragment = local.get_mut(folder).ok_or(ScanError::Invalid)?;
            fragment.from |= from;
            fragment.to |= to;
            raw_files.push(RawFile {
                folder,
                offset: u64::from(u32_at(&raw, 4)),
                size: u64::from(u32_at(&raw, 0)),
                attributes: u16_at(&raw, 14),
                modified: (u32::from(u16_at(&raw, 10)) << 16) | u32::from(u16_at(&raw, 12)),
                name,
                from,
                to,
            });
        }
        reader.range(volume, cursor, 0)?;
        let files_end = cursor;
        let mut mapping = Vec::with_capacity(local.len());
        let mut intervals = vec![(0, table_end), (files_start, files_end)];
        for (position, fragment) in local.iter().enumerate() {
            if (fragment.from && position != 0) || (fragment.to && position + 1 != local.len()) {
                return Err(ScanError::Invalid);
            }
            let logical = if fragment.from {
                previous_folder.take().ok_or(ScanError::Invalid)?
            } else {
                if position == 0 && previous_folder.is_some() {
                    return Err(ScanError::Invalid);
                }
                if index.folders.len() >= MAX_FOLDERS {
                    return Err(ScanError::Limit);
                }
                index.folders.push(Folder {
                    method: fragment.method,
                    frames: Vec::new(),
                    size: 0,
                    last_record: None,
                    pending: Vec::new(),
                    packed: 0,
                });
                index.folders.len() - 1
            };
            let folder = &mut index.folders[logical];
            if folder.method != fragment.method {
                return Err(ScanError::Invalid);
            }
            mapping.push(logical);
            let mut cursor = fragment.start;
            for number in 0..fragment.blocks {
                blocks = blocks.checked_add(1).ok_or(ScanError::Limit)?;
                if blocks > MAX_BLOCKS {
                    return Err(ScanError::Limit);
                }
                let raw = reader.bytes(volume, cursor, 8)?;
                let packed = u16_at(&raw, 4);
                let unpacked = u16_at(&raw, 6);
                if unpacked > 32768 {
                    return Err(ScanError::Invalid);
                }
                let end = add(cursor, 8 + u64::from(data_reserve) + u64::from(packed))?;
                reader.range(volume, cursor, end - cursor)?;
                if unpacked == 0 && (number + 1 != fragment.blocks || !fragment.to) {
                    return Err(ScanError::Invalid);
                }
                folder.packed = folder
                    .packed
                    .checked_add(u64::from(packed))
                    .ok_or(ScanError::Limit)?;
                if folder.packed > folder.method.packed_limit() {
                    return Err(if matches!(folder.method, Method::Quantum(_, _)) {
                        ScanError::Limit
                    } else {
                        ScanError::Invalid
                    });
                }
                folder.pending.push(Fragment {
                    volume,
                    offset: cursor,
                    packed,
                    unpacked,
                    checksum: u32_at(&raw, 0),
                    reserve: data_reserve,
                });
                if unpacked != 0 {
                    if folder
                        .frames
                        .last()
                        .is_some_and(|frame| frame.unpacked != 32768)
                        || (folder.method == Method::Store && folder.packed != u64::from(unpacked))
                    {
                        return Err(ScanError::Invalid);
                    }
                    let start = folder.size;
                    folder.size = add(start, u64::from(unpacked))?;
                    folder.frames.push(Frame {
                        fragments: std::mem::take(&mut folder.pending),
                        start,
                        unpacked,
                    });
                    folder.packed = 0;
                }
                cursor = end;
            }
            if fragment.blocks != 0 {
                intervals.push((fragment.start, cursor));
            }
            if fragment.to {
                previous_folder = Some(logical);
            } else if !folder.pending.is_empty() {
                return Err(ScanError::Invalid);
            }
        }
        if local.is_empty() && previous_folder.is_some() {
            return Err(ScanError::Invalid);
        }
        intervals.sort_unstable();
        if intervals.windows(2).any(|pair| pair[0].1 > pair[1].0) {
            return Err(ScanError::Invalid);
        }
        let mut next = BTreeMap::<Key, VecDeque<usize>>::new();
        let mut previous_is_origin = false;
        for raw in raw_files {
            let folder = *mapping.get(raw.folder).ok_or(ScanError::Invalid)?;
            let key = Key {
                folder,
                offset: raw.offset,
                size: raw.size,
                attributes: raw.attributes,
                modified: raw.modified,
                name: raw.name.clone(),
            };
            let record = if raw.from {
                let record = active
                    .get_mut(&key)
                    .and_then(VecDeque::pop_front)
                    .ok_or(ScanError::Invalid)?;
                previous_is_origin |= previous == origins.get(record).copied();
                record
            } else {
                let declared = raw.attributes & 0x80 != 0;
                let name_encoding = if declared && std::str::from_utf8(&raw.name).is_err() {
                    NameEncoding::Unresolved
                } else {
                    NameEncoding::Utf8
                };
                let record = index.rows.len();
                index.rows.push(Entry {
                    format: Format::Cab,
                    id: EntryId::try_from(record as u64 + 1).map_err(|_| ScanError::Limit)?,
                    raw_name: raw.name,
                    decoded_name: None,
                    name_encoding,
                    display_name: String::new(),
                    directory: false,
                    link: false,
                    unix_mode: 0,
                    size: raw.size,
                    size_known: true,
                    packed_size: 0,
                    method: 0,
                    modified_dos: raw.modified,
                    encrypted: false,
                    password_supported: false,
                    warning: false,
                });
                index.files.push(File {
                    folder,
                    offset: raw.offset,
                    attributes: raw.attributes,
                });
                origins.push(volume);
                record
            };
            if raw.to {
                next.entry(key).or_default().push_back(record);
            }
        }
        if active.values().any(|items| !items.is_empty()) {
            return Err(ScanError::Invalid);
        }
        // Some producers use the adjacent cabinet; MS-CAB also permits the
        // earlier cabinet where a continued file first appeared.
        if previous.is_some_and(|number| number + 1 != volume && !previous_is_origin) {
            return Err(ScanError::Invalid);
        }
        active = next;
    }
    if previous_folder.is_some() || !active.is_empty() {
        return Err(ScanError::MissingVolume);
    }
    for (record, (file, row)) in index.files.iter().zip(&index.rows).enumerate() {
        let folder = &mut index.folders[file.folder];
        if add(file.offset, row.size)? > folder.size {
            return Err(ScanError::Invalid);
        }
        folder.last_record = Some(record);
    }
    if index
        .folders
        .iter()
        .any(|folder| !folder.pending.is_empty())
    {
        return Err(ScanError::Invalid);
    }
    if index
        .folders
        .iter()
        .any(|folder| folder.size != 0 && folder.last_record.is_none())
    {
        return Err(ScanError::Invalid);
    }
    let legacy: Vec<_> = index
        .files
        .iter()
        .map(|file| file.attributes & 0x80 == 0)
        .collect();
    xdzip_codec_api::names::resolve_cab(&mut index.rows, &legacy, reader.control, limits)?;
    for row in &mut index.rows {
        reader.check()?;
        row.warning |= xdzip_domain::path::components(row.name_bytes(), false).is_none();
    }
    Ok(index)
}
