use super::{MAX_FILE_BYTES, MAX_TASK_BYTES, Options};
use crate::{Archive, source::Source};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;
use unicode_normalization::UnicodeNormalization;
use xdzip_codec_api::{Cancellation, Entry};
use xdzip_domain::path::PortablePath;

pub(crate) struct Planned {
    pub index: usize,
    pub path: PortablePath,
}
pub(crate) struct Job {
    pub source: Source,
    pub format: xdzip_codec_api::Format,
    pub rows: Arc<Vec<Entry>>,
    pub planned: Vec<Planned>,
    pub options: Options,
    pub session: u64,
}
/// Canonical admission key accounts for common Unicode/case aliases. Exclusive native
/// publication remains authoritative on filesystems with different name semantics.
fn key(value: &str) -> String {
    value.nfd().flat_map(char::to_lowercase).collect()
}
pub(crate) fn prepare(
    archive: &Archive,
    options: &Options,
    cancel: &Cancellation,
) -> Result<Vec<Planned>, &'static str> {
    if !options.bounded() {
        return Err("invalid_options");
    }
    let base = if options.all {
        &[][..]
    } else {
        archive
            .directory_prefix(&options.directory)
            .map_err(|_| "invalid_selection")?
    };
    let mut selected = BTreeSet::new();
    let mut prefixes = Vec::new();
    if !options.all {
        for id in &options.selected {
            cancel.check().map_err(|_| "cancelled")?;
            if id.starts_with('d') {
                prefixes.push(
                    archive
                        .directory_prefix(id)
                        .map_err(|_| "invalid_selection")?,
                );
            } else {
                let number = id.parse::<u64>().map_err(|_| "invalid_selection")?;
                let entry = number
                    .checked_sub(1)
                    .and_then(|n| usize::try_from(n).ok())
                    .and_then(|n| archive.rows.get(n))
                    .filter(|e| e.id.get() == number)
                    .ok_or("invalid_selection")?;
                selected.insert(entry.id);
                if entry.directory {
                    prefixes.push(entry.raw_name.as_slice());
                }
            }
        }
    }
    // Prefix membership never scans 100,000 selectors for every physical row.
    let mut prefix_keys = BTreeSet::new();
    for prefix in prefixes {
        prefix_keys.insert(prefix);
    }
    let mut mapped: BTreeMap<String, (String, bool)> = BTreeMap::new();
    let mut result = Vec::new();
    let mut bytes = 0usize;
    let mut key_bytes = 0usize;
    let mut declared = 0u64;
    for (index, entry) in archive.rows.iter().enumerate() {
        cancel.check().map_err(|_| "cancelled")?;
        let descendant = entry
            .raw_name
            .iter()
            .enumerate()
            .filter(|(_, b)| **b == b'/')
            .any(|(at, _)| prefix_keys.contains(&entry.raw_name[..=at]));
        if !options.all && !selected.contains(&entry.id) && !descendant {
            continue;
        }
        if entry.link {
            return Err("unsupported_link");
        }
        if entry.warning || entry.encrypted || (!entry.directory && !matches!(entry.method, 0 | 8))
        {
            return Err("unsupported_entry");
        }
        let relative = entry
            .raw_name
            .strip_prefix(base)
            .ok_or("invalid_selection")?;
        if relative.is_empty() && entry.directory {
            continue;
        }
        let text = std::str::from_utf8(relative).map_err(|_| "unsafe_name")?;
        let text = if entry.directory {
            text.strip_suffix('/').ok_or("unsafe_name")?
        } else {
            text
        };
        let path = PortablePath::parse(text).ok_or("unsafe_name")?;
        if !entry.directory {
            if entry.size > MAX_FILE_BYTES {
                return Err("file_quota");
            }
            declared = declared
                .checked_add(entry.size)
                .filter(|n| *n <= MAX_TASK_BYTES)
                .ok_or("task_quota")?;
            if entry.format == xdzip_codec_api::Format::Zip
                && !options.allow_high_ratio
                && entry.size > entry.packed_size.saturating_mul(1000)
                && entry.size > 1024 * 1024
            {
                return Err("high_ratio");
            }
        }
        let display = path.display();
        bytes = bytes
            .checked_add(display.len())
            .filter(|n| *n <= 4 * 1024 * 1024)
            .ok_or("resource_limit")?;
        // Resolve case/NFD aliases deterministically, rejecting ambiguous directory spellings
        // and all file/directory collisions before a workspace or visible directory is created.
        let mut canonical_parts = Vec::new();
        for (part_at, part) in path.parts().iter().enumerate() {
            canonical_parts.push(part.clone());
            let spelling = canonical_parts.join("/");
            let canonical = key(&spelling);
            let directory = part_at + 1 < path.parts().len() || entry.directory;
            if let Some((previous, was_dir)) = mapped.get(&canonical) {
                if *was_dir != directory || (directory && previous != &spelling) {
                    return Err("ambiguous_name");
                }
                if !directory {
                    canonical_parts = previous.split('/').map(str::to_owned).collect();
                }
            } else {
                key_bytes = key_bytes
                    .checked_add(canonical.len() + spelling.len())
                    .filter(|n| *n <= 8 * 1024 * 1024)
                    .ok_or("resource_limit")?;
                mapped.insert(canonical, (spelling, directory));
            }
        }
        let canonical_path = canonical_parts.join("/");
        result.push(Planned {
            index,
            path: PortablePath::parse(&canonical_path).ok_or("unsafe_name")?,
        });
    }
    // GZIP has no per-entry compressed sizes. Use the immutable whole-source
    // size for a conservative selected-output ratio gate; work quotas still apply.
    if archive.format == xdzip_codec_api::Format::TarGzip
        && !options.allow_high_ratio
        && declared > 1024 * 1024
        && declared > archive.source.length().saturating_mul(1000)
    {
        return Err("high_ratio");
    }
    Ok(result)
}

impl Job {
    pub(crate) fn retained_bytes(&self) -> usize {
        // Charge the full immutable snapshot conservatively even when Arc shares it.
        self.rows
            .iter()
            .map(|e| {
                std::mem::size_of::<Entry>() + e.raw_name.capacity() + e.display_name.capacity()
            })
            .sum::<usize>()
            + self
                .planned
                .iter()
                .map(|p| {
                    std::mem::size_of::<Planned>()
                        + p.path
                            .parts()
                            .iter()
                            .map(|n| std::mem::size_of::<String>() + n.capacity())
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.options.bytes()
    }
}
