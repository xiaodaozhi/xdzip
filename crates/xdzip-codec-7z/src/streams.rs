use crate::graph;
use crate::source::{Budget, Reader};
use crate::types::{ByteRange, Folder, PackStream, Streams, Substream};
use crate::wire::Cursor;
use xdzip_codec_api::{Cancellation, MAX_ENTRIES, ScanError};

pub(crate) fn parse(
    c: &mut Cursor<'_>,
    reader: &Reader<'_>,
    budget: &mut Budget,
    external: &[Vec<u8>],
) -> Result<Streams, ScanError> {
    let mut streams = Streams::default();
    if c.peek() == Some(6) {
        c.byte()?;
        let start = 32u64.checked_add(c.number()?).ok_or(ScanError::Invalid)?;
        let count = c.count(MAX_ENTRIES * 4)?;
        let mut sizes = budget.vec(count)?;
        if c.peek() == Some(9) {
            c.byte()?;
            for _ in 0..count {
                sizes.push(c.number()?);
            }
        } else if count != 0 {
            return Err(ScanError::Invalid);
        }
        budget.claim(count * (std::mem::size_of::<Option<u32>>() + 1))?;
        let digests = if c.peek() == Some(10) {
            c.byte()?;
            c.digests(count)?
        } else {
            vec![None; count]
        };
        c.id(0)?;
        streams.packs = budget.vec(count)?;
        let mut at = start;
        for (size, crc) in sizes.into_iter().zip(digests) {
            reader.range(at, size)?;
            streams.packs.push(PackStream {
                range: ByteRange {
                    offset: at,
                    length: size,
                },
                crc,
            });
            at = at.checked_add(size).ok_or(ScanError::Invalid)?;
        }
    }
    if c.peek() == Some(7) {
        c.byte()?;
        c.id(11)?;
        let count = c.count(MAX_ENTRIES)?;
        let outside = c.byte()?;
        streams.folders = budget.vec(count)?;
        match outside {
            0 => {
                for _ in 0..count {
                    streams
                        .folders
                        .push(graph::folder(c, budget, reader.control)?);
                }
            }
            _ => {
                let index = usize::try_from(c.number()?).map_err(|_| ScanError::Invalid)?;
                let data = external.get(index).ok_or(ScanError::Invalid)?;
                let mut table = Cursor::new(data);
                for _ in 0..count {
                    streams
                        .folders
                        .push(graph::folder(&mut table, budget, reader.control)?);
                }
                table.end()?;
            }
        }
        c.id(12)?;
        for folder in &mut streams.folders {
            let outputs: usize = folder.coders.iter().map(|v| v.outputs).sum();
            folder.output_sizes = budget.vec(outputs)?;
            for _ in 0..outputs {
                folder.output_sizes.push(c.number()?);
            }
        }
        if c.peek() == Some(10) {
            c.byte()?;
            budget.claim(count * (std::mem::size_of::<Option<u32>>() + 1))?;
            for (folder, crc) in streams.folders.iter_mut().zip(c.digests(count)?) {
                folder.crc = crc;
            }
        }
        c.id(0)?;
    }
    substreams(c, &mut streams.folders, budget, reader.control)?;
    c.id(0)?;
    let mut pack = 0usize;
    for folder in &mut streams.folders {
        for input in &mut folder.packed_inputs {
            if pack >= streams.packs.len() {
                return Err(ScanError::Invalid);
            }
            input.pack = pack;
            pack += 1;
        }
    }
    if pack != streams.packs.len() {
        return Err(ScanError::Invalid);
    }
    Ok(streams)
}

fn substreams(
    c: &mut Cursor<'_>,
    folders: &mut [Folder],
    budget: &mut Budget,
    control: &Cancellation,
) -> Result<(), ScanError> {
    let present = c.peek() == Some(8);
    if present {
        c.byte()?;
    }
    let mut counts = budget.vec(folders.len())?;
    counts.resize(folders.len(), 1usize);
    if present && c.peek() == Some(13) {
        c.byte()?;
        for count in &mut counts {
            *count = c.count(MAX_ENTRIES)?;
        }
    }
    let total = counts
        .iter()
        .try_fold(0usize, |total, n| total.checked_add(*n))
        .filter(|n| *n <= MAX_ENTRIES)
        .ok_or(ScanError::Limit)?;
    budget.claim(total * std::mem::size_of::<Substream>())?;
    let sizes_present = present && c.peek() == Some(9);
    if sizes_present {
        c.byte()?;
    }
    for (folder, count) in folders.iter_mut().zip(&counts) {
        control.check()?;
        let mut offset = 0u64;
        folder
            .substreams
            .try_reserve_exact(*count)
            .map_err(|_| ScanError::Limit)?;
        for stream in 0..*count {
            let length = if stream + 1 == *count {
                folder
                    .unpacked_size()
                    .checked_sub(offset)
                    .ok_or(ScanError::Invalid)?
            } else if sizes_present {
                c.number()?
            } else {
                return Err(ScanError::Invalid);
            };
            let end = offset
                .checked_add(length)
                .filter(|n| *n <= folder.unpacked_size())
                .ok_or(ScanError::Invalid)?;
            folder.substreams.push(Substream {
                offset,
                length,
                crc: if *count == 1 { folder.crc } else { None },
            });
            offset = end;
        }
    }
    if present && c.peek() == Some(10) {
        c.byte()?;
        let digest_count: usize = folders
            .iter()
            .filter(|f| f.substreams.len() != 1 || f.crc.is_none())
            .map(|f| f.substreams.len())
            .sum();
        budget.claim(digest_count * (std::mem::size_of::<Option<u32>>() + 1))?;
        let mut digests = c.digests(digest_count)?.into_iter();
        for folder in folders {
            if folder.substreams.len() == 1 && folder.crc.is_some() {
                continue;
            }
            for stream in &mut folder.substreams {
                stream.crc = digests.next().ok_or(ScanError::Invalid)?;
            }
        }
        if digests.next().is_some() {
            return Err(ScanError::Invalid);
        }
    }
    if present {
        c.id(0)?;
    }
    Ok(())
}
