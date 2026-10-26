use crate::source::Budget;
use crate::types::{FileRecord, Index, StreamRef, Streams};
use crate::wire::Cursor;
use xdzip_codec_api::{Cancellation, Entry, Format, MAX_ENTRIES, ScanError, ScanLimits, names};
use xdzip_domain::identity::EntryId;

pub(crate) fn parse<'a>(
    c: &mut Cursor<'a>,
    streams: Streams,
    external: &'a [Vec<u8>],
    budget: &mut Budget,
    limits: ScanLimits,
    control: &Cancellation,
) -> Result<Index, ScanError> {
    let count = c.count(limits.entries.min(MAX_ENTRIES))?;
    budget.claim(count * 6)?;
    let mut empty = vec![false; count];
    let mut empty_files = Vec::new();
    let mut anti = Vec::new();
    let mut names_value = None;
    let (mut ctime, mut atime, mut mtime, mut attributes) = (None, None, None, None);
    let mut seen = [false; 256];
    loop {
        control.check()?;
        let id = c.byte()?;
        if id == 0 {
            break;
        }
        let size = usize::try_from(c.number()?).map_err(|_| ScanError::Limit)?;
        let mut p = Cursor::new(c.take(size)?);
        if id != 25 && seen[usize::from(id)] {
            return Err(ScanError::Invalid);
        }
        seen[usize::from(id)] = true;
        match id {
            14 => {
                empty = p.bits(count)?;
            }
            15 | 16 => {
                let n = empty.iter().filter(|v| **v).count();
                if !seen[14] {
                    return Err(ScanError::Invalid);
                }
                if id == 15 {
                    empty_files = p.bits(n)?;
                } else {
                    anti = p.bits(n)?;
                }
            }
            17 => {
                names_value = Some(read_names(
                    &mut p, count, external, budget, limits, control,
                )?);
            }
            18..=20 => {
                let values = values64(&mut p, count, external, budget)?;
                match id {
                    18 => ctime = Some(values),
                    19 => atime = Some(values),
                    _ => mtime = Some(values),
                }
            }
            21 => {
                budget.claim(count * (std::mem::size_of::<Option<u32>>() + 1))?;
                let defined = p.defined(count)?;
                let mut values = Vec::with_capacity(count);
                let mut outside = external_cursor(&mut p, external)?;
                let value = outside.as_mut().unwrap_or(&mut p);
                for bit in defined {
                    values.push(if bit { Some(value.u32()?) } else { None });
                }
                if let Some(outside) = outside {
                    outside.end()?;
                }
                attributes = Some(values);
            }
            // Dummy is explicitly padding. Its contents cannot affect records.
            25 => {
                if p.take(size)?.iter().any(|byte| *byte != 0) {
                    return Err(ScanError::Invalid);
                }
            }
            _ => return Err(ScanError::Unsupported),
        }
        p.end()?;
    }
    let stream_count = streams
        .folders
        .iter()
        .map(|f| f.substreams.len())
        .sum::<usize>();
    if stream_count != empty.iter().filter(|v| !**v).count() {
        return Err(ScanError::Invalid);
    }
    let mut locations = budget.vec(stream_count)?;
    for (folder, f) in streams.folders.iter().enumerate() {
        control.check()?;
        for stream in 0..f.substreams.len() {
            locations.push(StreamRef { folder, stream });
        }
    }
    if locations.len() != empty.iter().filter(|v| !**v).count() {
        return Err(ScanError::Invalid);
    }
    let mut locations = locations.into_iter();
    let mut index = Index {
        rows: budget.vec(count)?,
        files: budget.vec(count)?,
        streams,
        has_archive_properties: false,
    };
    let mut names_value = match names_value {
        Some(values) => values,
        None if count == 0 => Vec::new(),
        None => return Err(ScanError::Invalid),
    }
    .into_iter();
    let (mut empty_index, mut raw_names, mut displays) = (0usize, 0usize, 0usize);
    for (record, is_empty) in empty.into_iter().enumerate() {
        control.check()?;
        let (data, is_file, is_anti) = if is_empty {
            let result = (
                None,
                empty_files.get(empty_index).copied().unwrap_or(false),
                anti.get(empty_index).copied().unwrap_or(false),
            );
            empty_index += 1;
            result
        } else {
            (
                Some(locations.next().ok_or(ScanError::Invalid)?),
                false,
                false,
            )
        };
        let directory = is_empty && !is_file;
        let size = data.as_ref().map_or(0, |r| {
            index.streams.folders[r.folder].substreams[r.stream].length
        });
        let encrypted = data
            .as_ref()
            .is_some_and(|r| index.streams.folders[r.folder].encrypted());
        let packed_size = data.as_ref().filter(|r| r.stream == 0).map_or(0, |r| {
            index.streams.folders[r.folder]
                .packed_inputs
                .iter()
                .map(|input| index.streams.packs[input.pack].range.length)
                .sum()
        });
        let attrs = attributes.as_ref().and_then(|v| v[record]);
        let mode = attrs.unwrap_or(0) >> 16 & 0o170000;
        let link = mode == 0o120000;
        let special = mode != 0 && mode != 0o100000 && mode != 0o040000 && !link;
        let mut name = names_value.next().ok_or(ScanError::Invalid)?;
        let original_capacity = name.capacity();
        if directory && !name.ends_with('/') {
            name.push('/');
        }
        let display_name = names::display(&name);
        raw_names = raw_names.checked_add(name.len()).ok_or(ScanError::Limit)?;
        displays = displays
            .checked_add(display_name.len())
            .ok_or(ScanError::Limit)?;
        if name.len() > limits.name_bytes
            || raw_names > limits.total_name_bytes
            || displays > limits.display_name_bytes
        {
            return Err(ScanError::Limit);
        }
        budget
            .claim(name.capacity().saturating_sub(original_capacity) + display_name.capacity())?;
        let modified = mtime.as_ref().and_then(|v| v[record]);
        let warning = is_anti
            || special
            || xdzip_domain::path::components(name.as_bytes(), directory).is_none();
        index.rows.push(Entry {
            format: Format::SevenZip,
            id: EntryId::try_from(record as u64 + 1).map_err(|_| ScanError::Limit)?,
            raw_name: name.into_bytes(),
            decoded_name: None,
            name_encoding: names::NameEncoding::Utf8,
            display_name,
            directory,
            link,
            unix_mode: 0,
            size,
            size_known: true,
            packed_size,
            method: 0,
            modified_dos: modified.map_or(0, filetime_dos),
            encrypted,
            password_supported: encrypted,
            warning,
        });
        index.files.push(FileRecord {
            data,
            empty_file: is_file,
            anti: is_anti,
            attributes: attrs,
            creation_time: ctime.as_ref().and_then(|v| v[record]),
            access_time: atime.as_ref().and_then(|v| v[record]),
            modified_time: modified,
        });
    }
    Ok(index)
}
fn external_cursor<'a>(
    p: &mut Cursor<'_>,
    external: &'a [Vec<u8>],
) -> Result<Option<Cursor<'a>>, ScanError> {
    if p.byte()? == 0 {
        return Ok(None);
    }
    let index = usize::try_from(p.number()?).map_err(|_| ScanError::Invalid)?;
    Ok(Some(Cursor::new(
        external.get(index).ok_or(ScanError::Invalid)?,
    )))
}
fn values64<'a>(
    p: &mut Cursor<'a>,
    count: usize,
    external: &'a [Vec<u8>],
    budget: &mut Budget,
) -> Result<Vec<Option<u64>>, ScanError> {
    budget.claim(count * (std::mem::size_of::<Option<u64>>() + 1))?;
    let defined = p.defined(count)?;
    let mut outside = external_cursor(p, external)?;
    let value = outside.as_mut().unwrap_or(p);
    let mut values = Vec::with_capacity(count);
    for bit in defined {
        values.push(if bit { Some(value.u64()?) } else { None });
    }
    if let Some(outside) = outside {
        outside.end()?;
    }
    Ok(values)
}
fn read_names<'a>(
    p: &mut Cursor<'a>,
    count: usize,
    external: &'a [Vec<u8>],
    budget: &mut Budget,
    limits: ScanLimits,
    control: &Cancellation,
) -> Result<Vec<String>, ScanError> {
    let mut outside = external_cursor(p, external)?;
    let value = outside.as_mut().unwrap_or(p);
    let mut names = budget.vec(count)?;
    let mut total = 0usize;
    for _ in 0..count {
        control.check()?;
        let mut units = Vec::new();
        loop {
            let unit =
                u16::from_le_bytes(value.take(2)?.try_into().map_err(|_| ScanError::Invalid)?);
            if unit == 0 {
                break;
            }
            if units.len() >= limits.name_bytes {
                return Err(ScanError::Limit);
            }
            units.push(unit);
        }
        budget.claim(units.capacity() * 2)?;
        let text = String::from_utf16(&units).map_err(|_| ScanError::Invalid)?;
        total = total.checked_add(text.len()).ok_or(ScanError::Limit)?;
        if text.len() > limits.name_bytes || total > limits.total_name_bytes {
            return Err(ScanError::Limit);
        }
        budget.claim(text.capacity())?;
        names.push(text);
    }
    if let Some(outside) = outside {
        outside.end()?;
    }
    Ok(names)
}
fn filetime_dos(ticks: u64) -> u32 {
    xdzip_codec_api::time::filetime_dos_utc(ticks)
}
