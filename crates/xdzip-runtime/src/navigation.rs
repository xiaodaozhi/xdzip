//! Bounded directory navigation derived from the complete immutable metadata snapshot.
use std::collections::BTreeMap;
use xdzip_codec_api::{Cancellation, Entry, ScanError};

/// Maximum navigation nodes; exceeding this disables the tree, never publishes a partial tree.
pub const MAX_DIRECTORIES: usize = 32_768;
/// Aggregate UTF-8 path and label bytes in one navigation snapshot.
pub const MAX_DIRECTORY_BYTES: usize = 4 * 1024 * 1024;
/// Maximum hierarchy depth supported by the navigation view.
pub const MAX_DIRECTORY_DEPTH: usize = 64;

/// A virtual navigation node, never a physical EntryId or an extraction selection.
#[derive(Clone, Debug)]
pub struct Directory {
    /// Session-local navigation identity, distinct from physical records.
    pub id: u64,
    /// Parent identity, or zero for a top-level node.
    pub parent: u64,
    /// Safe leaf label.
    pub name: String,
    /// Safe archive-relative prefix ending with a slash; never a host path.
    pub path: String,
    /// Physical descendants, including duplicates and warned records within this prefix.
    pub count: u64,
}

/// Build complete explicit/implicit navigation or fail before exposing any nodes.
pub fn build(rows: &[Entry], control: &Cancellation) -> Result<Vec<Directory>, ScanError> {
    control.check()?;
    let mut nodes: Vec<Directory> = Vec::new();
    let mut ids = BTreeMap::<Vec<u8>, usize>::new();
    let mut bytes = 0;
    for entry in rows {
        control.check()?;
        if entry.warning || entry.raw_name.contains(&b'\\') {
            continue;
        }
        let Ok(path) = std::str::from_utf8(&entry.raw_name) else {
            continue;
        };
        if path
            .strip_suffix('/')
            .unwrap_or(path)
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == "..")
        {
            continue;
        }
        let mut parent = 0;
        let mut start = 0;
        for (depth, (end, _)) in path.match_indices('/').enumerate() {
            control.check()?;
            if depth >= MAX_DIRECTORY_DEPTH {
                return Err(ScanError::Limit);
            }
            let prefix = &path[..=end];
            let index = match ids.get(prefix.as_bytes()) {
                Some(&index) => index,
                None => {
                    let name = &path[start..end];
                    bytes += prefix.len() + name.len();
                    if nodes.len() >= MAX_DIRECTORIES || bytes > MAX_DIRECTORY_BYTES {
                        return Err(ScanError::Limit);
                    }
                    let index = nodes.len();
                    nodes.push(Directory {
                        id: index as u64 + 1,
                        parent,
                        name: name.into(),
                        path: prefix.into(),
                        count: 0,
                    });
                    ids.insert(prefix.as_bytes().to_vec(), index);
                    index
                }
            };
            parent = nodes[index].id;
            start = end + 1;
        }
    }
    // Count warned descendants too, without decoding their names. No known node
    // exceeds 64 levels, so deeper prefixes cannot match and need not be examined.
    for entry in rows {
        control.check()?;
        for (end, _) in entry
            .raw_name
            .iter()
            .enumerate()
            .filter(|(_, byte)| **byte == b'/')
            .take(MAX_DIRECTORY_DEPTH)
        {
            if end + 1 < entry.raw_name.len()
                && let Some(&index) = ids.get(&entry.raw_name[..=end])
            {
                nodes[index].count += 1;
            }
        }
    }
    Ok(nodes)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(id: u64, name: &str, warning: bool) -> Entry {
        Entry {
            id: id.try_into().unwrap(),
            raw_name: name.as_bytes().to_vec(),
            display_name: name.into(),
            directory: name.ends_with('/'),
            size: 0,
            packed_size: 0,
            method: 0,
            modified_dos: 0,
            encrypted: false,
            warning,
        }
    }
    #[test]
    fn implicit_parents_duplicates_empty_directories_and_unsafe_descendants() {
        let mut rows = vec![
            entry(1, "a/b/file", false),
            entry(2, "a/b/file", false),
            entry(3, "empty/", false),
            entry(4, "a/b/", false),
            entry(5, "a/../bad", true),
            entry(6, "bad//", false),
            entry(7, "./dot/file", false),
            entry(8, "back\\slash/file", false),
            entry(9, "/absolute/file", true),
        ];
        let mut invalid = entry(10, "a/b/", true);
        invalid.raw_name.push(0xff);
        rows.push(invalid);
        let nodes = build(&rows, &Cancellation::default()).unwrap();
        assert_eq!(nodes.len(), 3);
        assert_eq!(
            (&*nodes[0].path, nodes[0].parent, nodes[0].count),
            ("a/", 0, 5)
        );
        assert_eq!(
            (&*nodes[1].path, nodes[1].parent, nodes[1].count),
            ("a/b/", nodes[0].id, 3)
        );
        assert_eq!((&*nodes[2].path, nodes[2].count), ("empty/", 0));
    }
    #[test]
    fn independent_node_byte_and_depth_budgets_never_return_partial_trees() {
        let control = Cancellation::default();
        let mut rows: Vec<_> = (0..MAX_DIRECTORIES)
            .map(|n| entry(n as u64 + 1, &format!("d{n}/f"), false))
            .collect();
        assert_eq!(build(&rows, &control).unwrap().len(), MAX_DIRECTORIES);
        rows.push(entry(3000, "overflow/f", false));
        assert!(matches!(build(&rows, &control), Err(ScanError::Limit)));
        let rows: Vec<_> = (0..MAX_DIRECTORY_BYTES / 8000 + 1)
            .map(|n| entry(n as u64 + 1, &format!("{}-{n}/f", "x".repeat(4000)), false))
            .collect();
        assert!(matches!(build(&rows, &control), Err(ScanError::Limit)));
        assert_eq!(
            build(&[entry(1, &"d/".repeat(64), false)], &control)
                .unwrap()
                .len(),
            64
        );
        assert!(matches!(
            build(&[entry(1, &"d/".repeat(65), false)], &control),
            Err(ScanError::Limit)
        ));
        control.cancel();
        assert!(matches!(build(&[], &control), Err(ScanError::Cancelled)));
    }
}
