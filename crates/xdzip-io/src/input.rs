//! Frozen, bounded, no-follow input trees. Only the host sees filesystem paths.
use crate::{FileIdentity, FileVersion};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt};
use cap_std::fs::{Dir, OpenOptions};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use unicode_normalization::UnicodeNormalization;
use xdzip_domain::path::PortablePath;

/// Stable source failures; paths and OS error text never enter task diagnostics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InputError {
    /// An admitted source cannot be read.
    Io,
    /// A frozen source or directory identity changed.
    Changed,
    /// A link, reparse point or special object was encountered.
    Unsafe,
    /// A name is not valid under the portable path policy.
    Name,
    /// Names collide after canonical Unicode and case folding.
    Collision,
    /// A source admission budget was exceeded.
    Limit,
    /// No source entries remain after excluding the exact creation outputs.
    Empty,
    /// Cooperative cancellation was observed.
    Cancelled,
}
impl InputError {
    /// Runtime diagnostic code.
    pub fn code(self) -> &'static str {
        match self {
            Self::Io => "source_io",
            Self::Changed => "source_changed",
            Self::Unsafe => "unsafe_input",
            Self::Name => "unsafe_name",
            Self::Collision => "input_collision",
            Self::Limit => "resource_limit",
            Self::Empty => "empty_input",
            Self::Cancelled => "cancelled",
        }
    }
}
/// Immutable format-visible information; no ambient filesystem authority.
#[derive(Clone, Debug)]
pub struct InputEntry {
    /// Portable archive-relative name, without a trailing directory slash.
    pub name: String,
    /// Explicit directory, including empty directories.
    pub directory: bool,
    /// Frozen ordinary-file bytes, zero for directories.
    pub size: u64,
    /// Source time, used for ZIP timestamp fields.
    pub modified: SystemTime,
    root: usize,
    relative: PathBuf,
    version: FileVersion,
}
struct Root {
    dir: Dir,
    path: PathBuf,
    identity: FileIdentity,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Child {
    name: String,
    directory: bool,
    identity: FileIdentity,
}
struct Directory {
    root: usize,
    relative: PathBuf,
    identity: FileIdentity,
    children: Vec<Child>,
}
/// Host-owned source authority and frozen directory topology. It retains only root handles.
pub struct InputTree {
    roots: Vec<Root>,
    entries: Vec<InputEntry>,
    directories: Vec<Directory>,
    directory_index: BTreeMap<(usize, PathBuf), usize>,
    excluded: Vec<(PathBuf, Option<FileIdentity>)>,
    names: BTreeSet<String>,
    name_bytes: usize,
    total_bytes: u64,
}
fn id(dir: &Dir) -> Result<FileIdentity, InputError> {
    FileIdentity::read(&dir.try_clone().map_err(|_| InputError::Io)?.into_std_file())
        .map_err(|_| InputError::Io)
}
fn check(stop: &dyn Fn() -> bool) -> Result<(), InputError> {
    if stop() {
        Err(InputError::Cancelled)
    } else {
        Ok(())
    }
}
// Resolve parent aliases (not the selected leaf) so /var and /private/var
// identify the same exclusions on macOS. Leaf links remain visible to nofollow.
fn parent_resolved(path: &Path) -> Result<PathBuf, InputError> {
    let parent = path.parent().ok_or(InputError::Unsafe)?;
    let leaf = path.file_name().ok_or(InputError::Unsafe)?;
    Ok(parent
        .canonicalize()
        .map_err(|_| InputError::Io)?
        .join(leaf))
}
fn file(dir: &Dir, name: &Path) -> Result<File, InputError> {
    let m = dir
        .symlink_metadata(name)
        .map_err(|_| InputError::Changed)?;
    if !m.is_file() || m.file_type().is_symlink() {
        return Err(InputError::Unsafe);
    }
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        // A regular-file-to-FIFO race must not block the worker before classification.
        options.custom_flags(rustix::fs::OFlags::NONBLOCK.bits() as i32);
    }
    let f = dir
        .open_with(name, &options)
        .map_err(|_| InputError::Changed)?
        .into_std();
    if !f.metadata().map_err(|_| InputError::Io)?.is_file() {
        return Err(InputError::Unsafe);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if f.metadata().map_err(|_| InputError::Io)?.file_attributes() & 0x400 != 0 {
            return Err(InputError::Unsafe);
        }
    }
    Ok(f)
}
// Resolve a regular directory entry's actual spelling without granting authority
// through that ambient path. Distinct hard-link names remain distinct paths.
fn regular_resolved(path: &Path) -> Result<PathBuf, InputError> {
    let parent_path = path.parent().ok_or(InputError::Unsafe)?;
    let leaf = path.file_name().ok_or(InputError::Unsafe)?;
    let parent = Dir::open_ambient_dir(parent_path, cap_std::ambient_authority())
        .map_err(|_| InputError::Changed)?;
    let parent_identity = id(&parent)?;
    let source = file(&parent, Path::new(leaf)).map_err(|_| InputError::Changed)?;
    let version = FileVersion::read(&source).map_err(|_| InputError::Io)?;
    let resolved = path.canonicalize().map_err(|_| InputError::Changed)?;
    let resolved_parent = Dir::open_ambient_dir(
        resolved.parent().ok_or(InputError::Changed)?,
        cap_std::ambient_authority(),
    )
    .map_err(|_| InputError::Changed)?;
    let resolved_source = file(
        &resolved_parent,
        Path::new(resolved.file_name().ok_or(InputError::Changed)?),
    )
    .map_err(|_| InputError::Changed)?;
    let current_parent = Dir::open_ambient_dir(parent_path, cap_std::ambient_authority())
        .map_err(|_| InputError::Changed)?;
    let current_source = file(&parent, Path::new(leaf)).map_err(|_| InputError::Changed)?;
    if id(&resolved_parent)? != parent_identity
        || id(&current_parent)? != parent_identity
        || FileVersion::read(&resolved_source).map_err(|_| InputError::Io)? != version
        || FileVersion::read(&current_source).map_err(|_| InputError::Io)? != version
        || FileVersion::read(&source).map_err(|_| InputError::Io)? != version
    {
        return Err(InputError::Changed);
    }
    Ok(resolved)
}
impl InputTree {
    /// Freeze selected inputs and explicitly excluded output/workspace paths.
    /// The workspace exclusion is checked by identity, never by a filename prefix.
    /// Explicit ordinary output files are omitted just like outputs found in a directory;
    /// selecting the private workspace or a non-file output is still rejected.
    pub fn scan(
        paths: &[PathBuf],
        excluded: Vec<(PathBuf, Option<FileIdentity>)>,
        stop: &dyn Fn() -> bool,
    ) -> Result<Self, InputError> {
        if paths.is_empty() || paths.len() > 100_000 {
            return Err(InputError::Limit);
        }
        let excluded = excluded
            .into_iter()
            .map(|(path, identity)| {
                let mut path = parent_resolved(&path)?;
                if identity.is_none() {
                    match std::fs::symlink_metadata(&path) {
                        Ok(m) if m.is_file() && !m.file_type().is_symlink() => {
                            #[cfg(windows)]
                            {
                                use std::os::windows::fs::MetadataExt;
                                if m.file_attributes() & 0x400 != 0 {
                                    return Ok((path, identity));
                                }
                            }
                            path = regular_resolved(&path)?;
                        }
                        Ok(_) => {}
                        Err(e) if e.kind() == io::ErrorKind::NotFound => {}
                        Err(_) => return Err(InputError::Io),
                    }
                }
                Ok((path, identity))
            })
            .collect::<Result<Vec<_>, InputError>>()?;
        let mut selected = BTreeMap::new();
        for p in paths {
            check(stop)?;
            if !p.is_absolute() {
                return Err(InputError::Unsafe);
            }
            let mut p = parent_resolved(p)?;
            let m = std::fs::symlink_metadata(&p).map_err(|_| InputError::Io)?;
            if m.file_type().is_symlink() || (!m.is_file() && !m.is_dir()) {
                return Err(InputError::Unsafe);
            }
            #[cfg(windows)]
            {
                use std::os::windows::fs::MetadataExt;
                if m.file_attributes() & 0x400 != 0 {
                    return Err(InputError::Unsafe);
                }
            }
            if m.is_file()
                && !excluded.iter().any(|(e, _)| e == &p)
                && excluded
                    .iter()
                    .any(|(e, identity)| identity.is_none() && e.parent() == p.parent())
            {
                p = regular_resolved(&p)?;
            }
            if m.is_dir() && excluded.iter().any(|(_, identity)| identity.is_some()) {
                let parent = Dir::open_ambient_dir(
                    p.parent().ok_or(InputError::Unsafe)?,
                    cap_std::ambient_authority(),
                )
                .map_err(|_| InputError::Changed)?;
                let directory = parent
                    .open_dir_nofollow(p.file_name().ok_or(InputError::Unsafe)?)
                    .map_err(|_| InputError::Changed)?;
                let identity = id(&directory)?;
                if excluded
                    .iter()
                    .any(|(_, expected)| *expected == Some(identity))
                {
                    return Err(InputError::Unsafe);
                }
            }
            if let Some((_, identity)) = excluded.iter().find(|(e, _)| e == &p) {
                if identity.is_some() || !m.is_file() {
                    return Err(InputError::Unsafe);
                }
                continue;
            }
            selected.insert(p, m.is_dir());
        }
        let mut tree = Self {
            roots: vec![],
            entries: vec![],
            directories: vec![],
            directory_index: BTreeMap::new(),
            excluded,
            names: BTreeSet::new(),
            name_bytes: 0,
            total_bytes: 0,
        };
        let mut root_ids = BTreeMap::new();
        for (path, directory) in &selected {
            check(stop)?;
            if path
                .ancestors()
                .skip(1)
                .any(|ancestor| selected.get(ancestor) == Some(&true))
            {
                continue;
            }
            let parent = path.parent().ok_or(InputError::Unsafe)?;
            let root = if let Some(index) = root_ids.get(parent) {
                *index
            } else {
                if tree.roots.len() >= 128 {
                    return Err(InputError::Limit);
                }
                let dir = Dir::open_ambient_dir(parent, cap_std::ambient_authority())
                    .map_err(|_| InputError::Io)?;
                let identity = id(&dir)?;
                let index = tree.roots.len();
                tree.roots.push(Root {
                    dir,
                    identity,
                    path: parent.into(),
                });
                root_ids.insert(parent.to_owned(), index);
                index
            };
            let name = path
                .file_name()
                .and_then(|s| s.to_str())
                .ok_or(InputError::Name)?
                .to_owned();
            let relative = PathBuf::from(&name);
            tree.visit(root, relative, name, *directory, stop)?;
        }
        if tree.entries.is_empty() {
            return Err(InputError::Empty);
        }
        tree.validate(stop)?;
        Ok(tree)
    }
    /// Ordered, immutable format-visible records.
    pub fn entries(&self) -> &[InputEntry] {
        &self.entries
    }
    /// Exact total ordinary-file input bytes after scanning.
    pub fn total_bytes(&self) -> u64 {
        self.total_bytes
    }
    /// Whether publication would replace an input object under another name.
    pub fn contains_identity(&self, identity: FileIdentity) -> bool {
        self.entries.iter().any(|e| e.version.identity == identity)
    }
    fn open_dir(&self, root: usize, relative: &Path) -> Result<Dir, InputError> {
        let mut dir = self.roots[root]
            .dir
            .try_clone()
            .map_err(|_| InputError::Io)?;
        let mut prefix = PathBuf::new();
        for part in relative.components() {
            prefix.push(part.as_os_str());
            dir = dir
                .open_dir_nofollow(part.as_os_str())
                .map_err(|_| InputError::Changed)?;
            if let Some(index) = self.directory_index.get(&(root, prefix.clone()))
                && id(&dir)? != self.directories[*index].identity
            {
                return Err(InputError::Changed);
            }
        }
        Ok(dir)
    }
    fn children(
        &self,
        root: usize,
        relative: &Path,
        dir: &Dir,
        stop: &dyn Fn() -> bool,
    ) -> Result<Vec<Child>, InputError> {
        let mut children = Vec::new();
        let absolute = self.roots[root]
            .path
            .join(relative)
            .canonicalize()
            .map_err(|_| InputError::Changed)?;
        let resolved = Dir::open_ambient_dir(&absolute, cap_std::ambient_authority())
            .map_err(|_| InputError::Changed)?;
        if id(&resolved)? != id(dir)? || id(&self.open_dir(root, relative)?)? != id(dir)? {
            return Err(InputError::Changed);
        }
        for item in dir.entries().map_err(|_| InputError::Io)? {
            check(stop)?;
            if children.len() >= 100_000 {
                return Err(InputError::Limit);
            }
            let name = item.map_err(|_| InputError::Io)?.file_name();
            let absolute = absolute.join(&name);
            if let Some((_, expected)) = self.excluded.iter().find(|(p, _)| p == &absolute) {
                if let Some(expected) = expected {
                    let excluded = dir
                        .open_dir_nofollow(&name)
                        .map_err(|_| InputError::Changed)?;
                    if id(&excluded)? != *expected {
                        return Err(InputError::Changed);
                    }
                }
                continue;
            }
            let name = name.to_str().ok_or(InputError::Name)?.to_owned();
            let m = dir
                .symlink_metadata(&name)
                .map_err(|_| InputError::Changed)?;
            if m.file_type().is_symlink() || (!m.is_dir() && !m.is_file()) {
                return Err(InputError::Unsafe);
            }
            let identity = if m.is_dir() {
                id(&dir
                    .open_dir_nofollow(&name)
                    .map_err(|_| InputError::Changed)?)?
            } else {
                FileIdentity::read(&file(dir, Path::new(&name))?).map_err(|_| InputError::Io)?
            };
            children.push(Child {
                name,
                directory: m.is_dir(),
                identity,
            });
        }
        children.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(children)
    }
    fn visit(
        &mut self,
        root: usize,
        relative: PathBuf,
        name: String,
        directory: bool,
        stop: &dyn Fn() -> bool,
    ) -> Result<(), InputError> {
        check(stop)?;
        if self.entries.len() >= 100_000 {
            return Err(InputError::Limit);
        }
        let portable = PortablePath::parse(&name).ok_or(InputError::Name)?;
        if portable.parts().len() > 64 {
            return Err(InputError::Limit);
        }
        self.name_bytes = self
            .name_bytes
            .checked_add(name.len())
            .ok_or(InputError::Limit)?;
        if self.name_bytes > 16 * 1024 * 1024 {
            return Err(InputError::Limit);
        }
        if !self
            .names
            .insert(name.nfc().collect::<String>().to_lowercase())
        {
            return Err(InputError::Collision);
        }
        let parent = self.open_dir(root, relative.parent().ok_or(InputError::Name)?)?;
        let leaf = relative.file_name().ok_or(InputError::Name)?;
        let f = if directory {
            parent
                .open_dir_nofollow(leaf)
                .map_err(|_| InputError::Changed)?
                .into_std_file()
        } else {
            file(&parent, Path::new(leaf))?
        };
        let version = FileVersion::read(&f).map_err(|_| InputError::Io)?;
        let size = if directory { 0 } else { version.length };
        self.total_bytes = self
            .total_bytes
            .checked_add(size)
            .ok_or(InputError::Limit)?;
        if size > 8 * 1024 * 1024 * 1024 || self.total_bytes > 20 * 1024 * 1024 * 1024 {
            return Err(InputError::Limit);
        }
        self.entries.push(InputEntry {
            name: name.clone(),
            directory,
            size,
            modified: version.modified,
            root,
            relative: relative.clone(),
            version,
        });
        if directory {
            let dir = self.open_dir(root, &relative)?;
            let identity = id(&dir)?;
            let children = self.children(root, &relative, &dir, stop)?;
            self.directory_index
                .insert((root, relative.clone()), self.directories.len());
            self.directories.push(Directory {
                root,
                relative: relative.clone(),
                identity,
                children: children.clone(),
            });
            for child in children {
                self.visit(
                    root,
                    relative.join(&child.name),
                    format!("{name}/{}", child.name),
                    child.directory,
                    stop,
                )?;
            }
        }
        Ok(())
    }
    /// Recheck roots, directory membership and all file versions. No recursive rediscovery occurs.
    pub fn validate(&self, stop: &dyn Fn() -> bool) -> Result<(), InputError> {
        for root in &self.roots {
            check(stop)?;
            let current = Dir::open_ambient_dir(&root.path, cap_std::ambient_authority())
                .map_err(|_| InputError::Changed)?;
            if id(&current)? != root.identity {
                return Err(InputError::Changed);
            }
        }
        for node in &self.directories {
            check(stop)?;
            let dir = self.open_dir(node.root, &node.relative)?;
            if id(&dir)? != node.identity
                || self.children(node.root, &node.relative, &dir, stop)? != node.children
            {
                return Err(InputError::Changed);
            }
        }
        for index in 0..self.entries.len() {
            if !self.entries[index].directory {
                check(stop)?;
                let _ = self.open(index)?;
            }
        }
        Ok(())
    }
    /// Open one frozen ordinary file, verifying every directory identity and leaf version.
    pub fn open(&self, index: usize) -> Result<InputReader, InputError> {
        let entry = self.entries.get(index).ok_or(InputError::Name)?;
        if entry.directory {
            return Err(InputError::Unsafe);
        }
        let parent = self.open_dir(entry.root, entry.relative.parent().ok_or(InputError::Name)?)?;
        let f = file(
            &parent,
            Path::new(entry.relative.file_name().ok_or(InputError::Name)?),
        )?;
        if FileVersion::read(&f).map_err(|_| InputError::Io)? != entry.version {
            return Err(InputError::Changed);
        }
        Ok(InputReader {
            file: f,
            expected: entry.version.clone(),
            read: 0,
        })
    }
}
/// Read-only stream that checks the frozen version on EOF.
pub struct InputReader {
    file: File,
    expected: FileVersion,
    read: u64,
}
impl Read for InputReader {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        let n = self.file.read(output)?;
        self.read += n as u64;
        if self.read > self.expected.length
            || (n == 0
                && !output.is_empty()
                && (self.read != self.expected.length
                    || FileVersion::read(&self.file)? != self.expected))
        {
            return Err(io::Error::other("source_changed"));
        }
        Ok(n)
    }
}
