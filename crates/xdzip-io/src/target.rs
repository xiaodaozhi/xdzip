use crate::{FileIdentity, FileVersion, OutputError, failure, private};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsMaybeDirExt};
use cap_std::fs::{Dir, DirBuilder, OpenOptions};
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use xdzip_domain::path::PortablePath;

/// User-selected directory object. Output paths are resolved only below its held handle.
pub struct Grant {
    dir: Dir,
    path: PathBuf,
    identity: FileIdentity,
    source: Arc<[FileIdentity]>,
    created_directories: Arc<AtomicU64>,
    parent_grant: Option<Arc<Grant>>,
    _temporary_lock: Option<File>,
}
struct Parent {
    dir: Dir,
    identities: Vec<FileIdentity>,
}
/// Current leaf fingerprint for one explicit overwrite decision.
#[derive(Clone, Debug)]
pub struct ExistingFile {
    /// Handle-based identity, length and mtime; never a content CAS guarantee.
    pub version: FileVersion,
    /// The selected archive itself can be renamed around, but cannot be overwritten.
    pub source_protected: bool,
}
impl Grant {
    /// Authorize an existing directory chosen by the host. Broadly writable or linked roots fail.
    pub fn open(path: &Path, source: FileIdentity) -> Result<Arc<Self>, OutputError> {
        Self::open_sources(path, &[source])
    }
    /// Protect every source volume against replacement through an output alias.
    pub fn open_sources(path: &Path, sources: &[FileIdentity]) -> Result<Arc<Self>, OutputError> {
        if sources.is_empty() || sources.len() > 1024 {
            return Err(OutputError::UnsafeTarget);
        }
        Self::authorize(path, Arc::from(sources))
    }
    /// Authorize creation output; the host must protect every frozen input identity.
    pub fn for_creation(path: &Path) -> Result<Arc<Self>, OutputError> {
        Self::authorize(path, Arc::from([]))
    }
    fn authorize(path: &Path, source: Arc<[FileIdentity]>) -> Result<Arc<Self>, OutputError> {
        let m = std::fs::symlink_metadata(path).map_err(failure)?;
        if m.file_type().is_symlink() || !m.is_dir() {
            return Err(OutputError::UnsafeTarget);
        }
        let dir = Dir::open_ambient_dir(path, cap_std::ambient_authority()).map_err(failure)?;
        let file = directory_file(&dir)?;
        private::check(&file, true)?;
        let identity = FileIdentity::read(&file).map_err(failure)?;
        let grant = Arc::new(Self {
            dir,
            path: path.into(),
            identity,
            source,
            created_directories: Arc::new(AtomicU64::new(0)),
            parent_grant: None,
            _temporary_lock: None,
        });
        grant.validate()?;
        Ok(grant)
    }
    /// Human-visible location only; codecs never receive this path.
    pub fn display_path(&self) -> &Path {
        &self.path
    }
    /// Check that the selected pathname still denotes the granted directory object.
    pub fn validate(&self) -> Result<(), OutputError> {
        if let Some(parent) = &self.parent_grant {
            parent.validate()?;
        }
        let m = std::fs::symlink_metadata(&self.path).map_err(|_| OutputError::TargetChanged)?;
        if m.file_type().is_symlink() || !m.is_dir() {
            return Err(OutputError::TargetChanged);
        }
        let current = Dir::open_ambient_dir(&self.path, cap_std::ambient_authority())
            .map_err(|_| OutputError::TargetChanged)?;
        let file = directory_file(&current).map_err(|_| OutputError::TargetChanged)?;
        if FileIdentity::read(&file).map_err(|_| OutputError::TargetChanged)? != self.identity {
            return Err(OutputError::TargetChanged);
        }
        private::check(&file, true)
    }
    fn parent(&self, path: &PortablePath, create: bool) -> Result<Parent, OutputError> {
        self.validate()?;
        let mut dir = self.dir.try_clone().map_err(failure)?;
        let mut identities = Vec::new();
        for part in &path.parts()[..path.parts().len() - 1] {
            if create {
                match dir.create_dir_with(part, &private_builder()) {
                    Ok(()) => {
                        self.created_directories.fetch_add(1, Ordering::Relaxed);
                    }
                    Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
                    Err(e) => return Err(failure(e)),
                }
            }
            let next = dir.open_dir_nofollow(part).map_err(|e| {
                if e.kind() == io::ErrorKind::NotFound {
                    OutputError::TargetChanged
                } else {
                    OutputError::UnsafeTarget
                }
            })?;
            let file = directory_file(&next)?;
            private::check(&file, true)?;
            identities.push(FileIdentity::read(&file).map_err(failure)?);
            dir = next;
        }
        Ok(Parent { dir, identities })
    }
    fn revalidate_parent(&self, path: &PortablePath, parent: &Parent) -> Result<(), OutputError> {
        let current = self.parent(path, false)?;
        if current.identities != parent.identities {
            Err(OutputError::TargetChanged)
        } else {
            Ok(())
        }
    }
    /// Create only verified ordinary parent directories. Existing directories are never replaced.
    pub fn ensure_directory(&self, path: &PortablePath) -> Result<(), OutputError> {
        let parent = self.parent(path, true)?;
        let leaf = path.parts().last().expect("PortablePath is nonempty");
        match parent.dir.create_dir_with(leaf, &private_builder()) {
            Ok(()) => {
                self.created_directories.fetch_add(1, Ordering::Relaxed);
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(failure(e)),
        }
        let child = parent
            .dir
            .open_dir_nofollow(leaf)
            .map_err(|_| OutputError::UnsafeTarget)?;
        private::check(&directory_file(&child)?, true)?;
        self.revalidate_parent(path, &parent)
    }
    /// Number of visible directories actually created through this grant tree.
    pub fn created_directories(&self) -> u64 {
        self.created_directories.load(Ordering::Relaxed)
    }
    /// Derive one named child authority while retaining its authorized parent.
    pub fn subdirectory(self: &Arc<Self>, name: &PortablePath) -> Result<Arc<Grant>, OutputError> {
        if name.parts().len() != 1 {
            return Err(OutputError::UnsafeTarget);
        }
        self.ensure_directory(name)?;
        let leaf = &name.parts()[0];
        let dir = self.dir.open_dir_nofollow(leaf).map_err(failure)?;
        let identity = FileIdentity::read(&directory_file(&dir)?).map_err(failure)?;
        let child = Arc::new(Self {
            dir,
            path: self.path.join(leaf),
            identity,
            source: Arc::clone(&self.source),
            created_directories: Arc::clone(&self.created_directories),
            parent_grant: Some(Arc::clone(self)),
            _temporary_lock: None,
        });
        child.validate()?;
        Ok(child)
    }
    /// Examine an existing ordinary leaf, refusing links, special objects and hard links.
    pub fn existing(&self, path: &PortablePath) -> Result<Option<ExistingFile>, OutputError> {
        let parent = self.parent(path, true)?;
        let result = self.existing_at(&parent, path.parts().last().expect("validated leaf"))?;
        self.revalidate_parent(path, &parent)?;
        Ok(result)
    }
    fn existing_at(
        &self,
        parent: &Parent,
        leaf: &str,
    ) -> Result<Option<ExistingFile>, OutputError> {
        match parent.dir.symlink_metadata(leaf) {
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(failure(e)),
            Ok(m) if m.file_type().is_symlink() || !m.is_file() => {
                return Err(OutputError::UnsafeTarget);
            }
            Ok(_) => {}
        }
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No);
        let file = parent
            .dir
            .open_with(leaf, &options)
            .map_err(|_| OutputError::TargetChanged)?
            .into_std();
        private::check(&file, false)?;
        let version = FileVersion::read(&file).map_err(failure)?;
        Ok(Some(ExistingFile {
            source_protected: self.source.contains(&version.identity),
            version,
        }))
    }
    /// Create a cryptographically random, exclusively created private workspace under this grant.
    pub fn workspace(self: &Arc<Self>, task: u64) -> Result<Workspace, OutputError> {
        self.validate()?;
        let mut random = [0u8; 16];
        getrandom::fill(&mut random).map_err(|_| OutputError::Io)?;
        let nonce = random
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        let name = format!(".xdzip-work-{nonce}");
        self.dir
            .create_dir_with(&name, &private_builder())
            .map_err(failure)?;
        let dir = self.dir.open_dir_nofollow(&name).map_err(failure)?;
        let file = directory_file(&dir)?;
        private::check(&file, true)?;
        let id = FileIdentity::read(&file).map_err(failure)?;
        let mut workspace = Workspace {
            grant: Arc::clone(self),
            dir: Some(Arc::new(dir)),
            identity: id,
            name,
            journal: None,
            journal_identity: None,
            ordinal: 0,
            #[cfg(test)]
            before_publish: None,
        };
        let mut options = private_options();
        options.write(true).create_new(true);
        let journal = workspace
            .dir()
            .open_with("transaction", &options)
            .map_err(failure)?
            .into_std();
        workspace.journal_identity = Some(FileIdentity::read(&journal).map_err(failure)?);
        workspace.journal = Some(journal);
        workspace.record(&format!("xdzip-transaction 1 {task} {nonce}\n"))?;
        Ok(workspace)
    }
    /// Count possible leftovers without deleting or trusting their transaction records.
    pub fn leftover_count(&self) -> Result<usize, OutputError> {
        let mut count = 0;
        for (index, item) in self.dir.entries().map_err(failure)?.enumerate() {
            if index >= 100_000 {
                return Err(OutputError::Limit);
            }
            let item = item.map_err(failure)?;
            if item
                .file_name()
                .to_string_lossy()
                .starts_with(".xdzip-work-")
            {
                count += 1;
            }
        }
        Ok(count)
    }
    /// Allocate a private persistent temporary output. Expired unlocked outputs
    /// are swept only below an authorized application cache, never a user target.
    /// The seven-day grace period accommodates asynchronous external readers.
    pub fn temporary(path: &Path, sources: &[FileIdentity]) -> Result<Arc<Self>, OutputError> {
        let root = Self::open_sources(path, sources)?;
        let now = std::time::SystemTime::now();
        let mut count = 0usize;
        for item in root.dir.entries().map_err(failure)? {
            count += 1;
            if count > 4096 {
                return Err(OutputError::Limit);
            }
            let item = item.map_err(failure)?;
            let name = item.file_name();
            let Some(text) = name.to_str() else {
                continue;
            };
            let Some(hex) = text.strip_prefix("open-") else {
                continue;
            };
            if hex.len() != 32 || !hex.bytes().all(|c| c.is_ascii_hexdigit()) {
                continue;
            }
            let Ok(dir) = root.dir.open_dir_nofollow(&name) else {
                continue;
            };
            if private::check(&directory_file(&dir)?, true).is_err() {
                continue;
            }
            let mut options = private_options();
            options.read(true).write(true);
            let Ok(mut file) = dir.open_with("lease", &options).map(|f| f.into_std()) else {
                continue;
            };
            if private::check(&file, false).is_err()
                || file.metadata().map_err(failure)?.len() != 16
            {
                continue;
            }
            let mut marker = [0u8; 16];
            if file.read_exact(&mut marker).is_err()
                || marker
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
                    != hex
            {
                continue;
            }
            let expired = file
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| now.duration_since(t).ok())
                .is_some_and(|age| age.as_secs() > 7 * 24 * 60 * 60);
            if !expired || file.try_lock().is_err() {
                continue;
            }
            // cap-std's removal stays below this directory and does not traverse
            // symlinks. Private ownership is rechecked before mutating the cache.
            root.validate()?;
            drop(file);
            drop(dir);
            if root.dir.remove_dir_all(&name).is_ok() {
                count -= 1;
            }
        }
        if count >= 256 {
            return Err(OutputError::Limit);
        }
        let mut nonce = [0u8; 16];
        getrandom::fill(&mut nonce).map_err(|_| OutputError::Io)?;
        let name = format!(
            "open-{}",
            nonce.iter().map(|b| format!("{b:02x}")).collect::<String>()
        );
        root.dir
            .create_dir_with(&name, &private_builder())
            .map_err(failure)?;
        let dir = root.dir.open_dir_nofollow(&name).map_err(failure)?;
        #[cfg(windows)]
        {
            use cap_std::fs::OpenOptionsExt;
            use windows_sys::Win32::Storage::FileSystem::{FILE_GENERIC_READ, WRITE_DAC};
            let mut options = OpenOptions::new();
            options
                .read(true)
                .maybe_dir(true)
                .follow(FollowSymlinks::No)
                .access_mode(FILE_GENERIC_READ | WRITE_DAC);
            let handle = dir.open_with(".", &options).map_err(failure)?.into_std();
            private::secure_temporary_directory(&handle)?;
        }
        private::check_confidential(&directory_file(&dir)?, true)?;
        let mut options = private_options();
        options.write(true).read(true).create_new(true);
        let mut lease = dir
            .open_with("lease", &options)
            .map_err(failure)?
            .into_std();
        lease.write_all(&nonce).map_err(failure)?;
        lease.lock().map_err(failure)?;
        let identity = FileIdentity::read(&directory_file(&dir)?).map_err(failure)?;
        let grant = Arc::new(Self {
            dir,
            path: root.path.join(&name),
            identity,
            source: Arc::clone(&root.source),
            created_directories: Arc::new(AtomicU64::new(0)),
            parent_grant: Some(root),
            _temporary_lock: Some(lease),
        });
        let child =
            grant.subdirectory(&PortablePath::parse("payload").expect("fixed portable name"))?;
        child.validate()?;
        private::check_confidential(&directory_file(&child.dir)?, true)?;
        Ok(child)
    }
}
fn directory_file(dir: &Dir) -> Result<File, OutputError> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .maybe_dir(true)
        .follow(FollowSymlinks::No);
    dir.open_with(".", &options)
        .map(|f| f.into_std())
        .map_err(failure)
}
fn private_builder() -> DirBuilder {
    let builder = DirBuilder::new();
    #[cfg(unix)]
    {
        use cap_std::fs::DirBuilderExt;
        let mut builder = builder;
        builder.mode(0o700);
        builder
    }
    #[cfg(not(unix))]
    {
        builder
    }
}
fn private_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    #[cfg(unix)]
    {
        use cap_std::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.follow(FollowSymlinks::No);
    options
}

/// Private transaction owner. Cleanup targets only known objects with matching identities.
pub struct Workspace {
    grant: Arc<Grant>,
    dir: Option<Arc<Dir>>,
    identity: FileIdentity,
    name: String,
    journal: Option<File>,
    journal_identity: Option<FileIdentity>,
    ordinal: u64,
    #[cfg(test)]
    before_publish: Option<Box<dyn FnOnce()>>,
}
/// Whole-file publication receipt. Post-publication cleanup/log failures cannot undo a commit.
#[derive(Debug)]
pub struct CommitReceipt {
    /// A private staging link or journal update could not be cleaned; content is already published.
    pub cleanup_warning: bool,
}
/// Set publication failure with an explicit identity-safe rollback outcome.
#[derive(Debug)]
pub struct SetCommitError {
    /// Original failure class without paths.
    pub error: OutputError,
    /// Some newly linked parts may remain; existing files were never replaced.
    pub rollback_failed: bool,
}
impl Workspace {
    /// Publish a plan-validated relative symbolic link, exclusively, under a held no-follow
    /// parent. This never opens its target or replaces an existing object. Call only after
    /// all ordinary output has committed; Windows link privileges are not assumed.
    pub fn commit_symlink(
        &mut self,
        path: &PortablePath,
        target: &PortablePath,
    ) -> Result<CommitReceipt, OutputError> {
        #[cfg(not(unix))]
        {
            let _ = (path, target);
            Err(OutputError::Unsupported)
        }
        #[cfg(unix)]
        {
            let parent = self.grant.parent(path, true)?;
            let leaf = path.parts().last().ok_or(OutputError::UnsafeTarget)?;
            self.grant.revalidate_parent(path, &parent)?;
            // symlink itself is an atomic create-if-absent operation, including dangling leaves.
            self.record(&format!("prepared-link {}\n", path.display()))?;
            #[cfg(test)]
            if let Some(change) = self.before_publish.take() {
                change();
            }
            self.grant.revalidate_parent(path, &parent)?;
            parent
                .dir
                .symlink(target.display(), leaf)
                .map_err(failure)?;
            let recorded = self.record(&format!("committed-link {}\n", path.display()));
            Ok(CommitReceipt {
                cleanup_warning: recorded.is_err(),
            })
        }
    }
    /// Exact private directory excluded from a creation input scan, with its identity.
    pub fn input_exclusion(&self) -> (PathBuf, FileIdentity) {
        (self.grant.path.join(&self.name), self.identity)
    }
    fn dir(&self) -> &Dir {
        self.dir.as_ref().expect("workspace already cleaned")
    }
    fn record(&mut self, text: &str) -> Result<(), OutputError> {
        let file = self.journal.as_mut().ok_or(OutputError::Io)?;
        file.write_all(text.as_bytes()).map_err(failure)?;
        file.flush().map_err(failure)
    }
    /// Allocate an ordinary exclusively created temporary file, not an archive-supplied name.
    pub fn stage(&mut self) -> Result<Stage, OutputError> {
        self.ordinal = self.ordinal.checked_add(1).ok_or(OutputError::Limit)?;
        let name = format!("{}.part", self.ordinal);
        let mut options = private_options();
        options.write(true).read(true).create_new(true);
        let file = self
            .dir()
            .open_with(&name, &options)
            .map_err(failure)?
            .into_std();
        let identity = FileIdentity::read(&file).map_err(failure)?;
        Ok(Stage {
            file: Some(file),
            // Closed volume stages share one held directory instead of each
            // retaining a separate directory descriptor (macOS limits are low).
            dir: Arc::clone(self.dir.as_ref().expect("workspace live")),
            identity,
            name,
            published: false,
            sealed: None,
        })
    }
    /// Publish verified data. Caller owns the task's commit permit. No-overwrite uses
    /// atomic hard-link creation; replacement uses same-volume rename, never truncation.
    /// Filesystems lacking the primitive fail without a copy/delete fallback.
    pub fn commit(
        &mut self,
        mut stage: Stage,
        path: &PortablePath,
        overwrite: Option<&ExistingFile>,
    ) -> Result<CommitReceipt, OutputError> {
        stage.finish()?;
        self.validate_stage(&stage)?;
        let parent = self.grant.parent(path, true)?;
        let leaf = path.parts().last().expect("validated leaf");
        self.grant.revalidate_parent(path, &parent)?;
        let existing = self.grant.existing_at(&parent, leaf)?;
        match (existing.as_ref(), overwrite) {
            (None, None) => {}
            (Some(current), Some(expected))
                if !current.source_protected && current.version == expected.version => {}
            (Some(current), _) if current.source_protected => {
                return Err(OutputError::UnsafeTarget);
            }
            (_, Some(_)) => return Err(OutputError::TargetChanged),
            _ => return Err(OutputError::Conflict),
        }
        self.record(&format!("prepared {} {}\n", stage.name, path.display()))?;
        #[cfg(test)]
        if let Some(change) = self.before_publish.take() {
            change();
        }
        self.grant.revalidate_parent(path, &parent)?;
        self.validate_stage(&stage)?;
        if overwrite.is_some() {
            // Identity check is best effort; ordinary rename has no content-CAS
            // guarantee against a same-account editor between this check and commit.
            if self
                .grant
                .existing_at(&parent, leaf)?
                .as_ref()
                .map(|f| &f.version)
                != overwrite.map(|f| &f.version)
            {
                return Err(OutputError::TargetChanged);
            }
            self.dir()
                .rename(&stage.name, &parent.dir, leaf)
                .map_err(failure)?;
            stage.published = true;
        } else {
            self.dir()
                .hard_link(&stage.name, &parent.dir, leaf)
                .map_err(failure)?;
        }
        let clean = stage.remove_known();
        let recorded = self.record(&format!("committed {} {}\n", stage.name, path.display()));
        Ok(CommitReceipt {
            cleanup_warning: clean.is_err() || recorded.is_err(),
        })
    }
    /// Publish a verified flat volume set without replacing existing leaves.
    /// Multiple files cannot appear atomically: check all names first, then roll
    /// back only matching newly linked identities after any publication failure.
    /// The host must already own the task's non-cancellable commit permit.
    pub fn commit_set(
        &mut self,
        mut stages: Vec<Stage>,
        paths: &[PortablePath],
    ) -> Result<CommitReceipt, SetCommitError> {
        let mut linked = Vec::new();
        let result = (|| -> Result<(), OutputError> {
            if stages.is_empty()
                || stages.len() != paths.len()
                || stages.len() > 1024
                || paths.iter().any(|p| p.parts().len() != 1)
                || paths
                    .iter()
                    .map(PortablePath::display)
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    != paths.len()
            {
                return Err(OutputError::Limit);
            }
            self.grant.validate()?;
            for (stage, path) in stages.iter_mut().zip(paths) {
                stage.finish()?;
                self.validate_stage(stage)?;
                if self.grant.existing(path)?.is_some() {
                    return Err(OutputError::Conflict);
                }
                self.record(&format!("prepared {} {}\n", stage.name, path.display()))?;
            }
            #[cfg(test)]
            if let Some(change) = self.before_publish.take() {
                change();
            }
            for (index, (stage, path)) in stages.iter().zip(paths).enumerate() {
                self.grant.validate()?;
                self.validate_stage(stage)?;
                self.dir()
                    .hard_link(&stage.name, &self.grant.dir, &path.parts()[0])
                    .map_err(failure)?;
                linked.push(index);
            }
            Ok(())
        })();
        if let Err(error) = result {
            let mut rollback_failed = false;
            for index in linked.into_iter().rev() {
                rollback_failed |= self
                    .grant
                    .validate()
                    .and_then(|()| {
                        remove_known(
                            &self.grant.dir,
                            &paths[index].parts()[0],
                            stages[index].identity,
                        )
                    })
                    .is_err();
            }
            return Err(SetCommitError {
                error,
                rollback_failed,
            });
        }
        let mut cleanup_warning = false;
        for (stage, path) in stages.iter_mut().zip(paths) {
            cleanup_warning |= stage.remove_known().is_err();
            cleanup_warning |= self
                .record(&format!("committed {} {}\n", stage.name, path.display()))
                .is_err();
        }
        Ok(CommitReceipt { cleanup_warning })
    }
    fn validate_stage(&self, stage: &Stage) -> Result<(), OutputError> {
        if FileIdentity::read(&directory_file(&stage.dir)?).map_err(failure)? != self.identity {
            return Err(OutputError::UnsafeTarget);
        }
        let mut options = private_options();
        options.read(true);
        let file = stage
            .dir
            .open_with(&stage.name, &options)
            .map_err(failure)?
            .into_std();
        let version = FileVersion::read(&file).map_err(failure)?;
        if version.identity != stage.identity || stage.sealed.as_ref() != Some(&version) {
            return Err(OutputError::TargetChanged);
        }
        Ok(())
    }
    /// Finish private cleanup; preserve unknown or replaced objects for explicit recovery.
    pub fn cleanup(mut self) -> Result<(), OutputError> {
        self.clean()
    }
    fn clean(&mut self) -> Result<(), OutputError> {
        if self.dir.is_none() {
            return Ok(());
        }
        self.journal.take();
        // An unremoved staging file or unknown object requires recovery evidence.
        // Keep the journal and never recursively delete an unfamiliar tree.
        for item in self.dir().entries().map_err(failure)? {
            if item.map_err(failure)?.file_name() != "transaction" {
                return Err(OutputError::Io);
            }
        }
        if let Some(identity) = self.journal_identity.take() {
            remove_known(self.dir(), "transaction", identity)?;
        }
        let current = self
            .grant
            .dir
            .open_dir_nofollow(&self.name)
            .map_err(failure)?;
        if FileIdentity::read(&directory_file(&current)?).map_err(failure)? != self.identity {
            return Err(OutputError::TargetChanged);
        }
        // Windows capability directories intentionally omit FILE_SHARE_DELETE.
        // Close both handles before removing this verified empty private directory.
        drop(current);
        let dir = self.dir.take().expect("workspace live");
        let dir = Arc::try_unwrap(dir).map_err(|_| OutputError::Io)?;
        #[cfg(windows)]
        {
            dir.remove_open_dir().map_err(failure)
        }
        #[cfg(not(windows))]
        {
            drop(dir);
            self.grant.dir.remove_dir(&self.name).map_err(failure)
        }
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = self.clean();
    }
}
/// One private ordinary file. Dropping it never removes a published destination.
pub struct Stage {
    file: Option<File>,
    dir: Arc<Dir>,
    identity: FileIdentity,
    name: String,
    published: bool,
    sealed: Option<FileVersion>,
}
impl Stage {
    /// Borrow a read handle for post-write verification. Writing has finished before this call.
    /// A duplicated handle may share its cursor; this method is deliberately sequential.
    pub fn verification_reader(&mut self) -> Result<File, OutputError> {
        use std::io::{Seek, SeekFrom};
        self.flush().map_err(failure)?;
        let mut reader = self
            .file
            .as_ref()
            .ok_or(OutputError::Io)?
            .try_clone()
            .map_err(failure)?;
        reader.seek(SeekFrom::Start(0)).map_err(failure)?;
        Ok(reader)
    }
    /// Restore only ordinary ZIP read/execute bits plus owner read/write. Privileged,
    /// group/other write and file-type bits never reach the OS. Applied by held file handle
    /// before publication; failure leaves the destination untouched.
    pub fn restore_mode(&mut self, mode: u32) -> Result<(), OutputError> {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let file = self.file.as_ref().ok_or(OutputError::Io)?;
            file.set_permissions(std::fs::Permissions::from_mode(0o600 | (mode & 0o155)))
                .map_err(failure)?;
        }
        #[cfg(not(unix))]
        let _ = mode;
        Ok(())
    }
    /// Synchronize verified file data before publication, detecting delayed I/O failures.
    pub fn finish(&mut self) -> Result<(), OutputError> {
        if let Some(mut file) = self.file.take() {
            file.flush().map_err(failure)?;
            file.sync_all().map_err(failure)?;
            if FileIdentity::read(&file).map_err(failure)? != self.identity {
                return Err(OutputError::TargetChanged);
            }
            self.sealed = Some(FileVersion::read(&file).map_err(failure)?);
        }
        Ok(())
    }
    fn remove_known(&mut self) -> Result<(), OutputError> {
        self.file.take();
        if self.published {
            return Ok(());
        }
        remove_known(&self.dir, &self.name, self.identity)?;
        self.published = true;
        Ok(())
    }
}
impl Write for Stage {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("stage_closed"))?
            .write(bytes)
    }
    fn flush(&mut self) -> io::Result<()> {
        self.file
            .as_mut()
            .ok_or_else(|| io::Error::other("stage_closed"))?
            .flush()
    }
}
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = self.remove_known();
    }
}
fn remove_known(dir: &Dir, name: &str, identity: FileIdentity) -> Result<(), OutputError> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    let file = match dir.open_with(name, &options) {
        Ok(file) => file.into_std(),
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(failure(e)),
    };
    if FileIdentity::read(&file).map_err(failure)? != identity {
        return Err(OutputError::TargetChanged);
    }
    dir.remove_file(name).map_err(failure)
}

#[cfg(test)]
mod race_tests {
    use super::*;
    use crate::tests::Owned;
    use std::fs;
    fn link_directory(target: &Path, link: &Path) {
        #[cfg(unix)]
        std::os::unix::fs::symlink(target, link).unwrap();
        #[cfg(windows)]
        {
            // Native junction creation does not need symlink developer mode.
            let result = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(link)
                .arg(target)
                .output()
                .unwrap();
            assert!(
                result.status.success(),
                "native junction fixture creation failed"
            );
        }
    }
    #[test]
    fn parent_replaced_with_link_between_prepare_and_publish_does_not_escape() {
        let root = Owned::new();
        let outside = Owned::new();
        fs::write(outside.0.join("important.txt"), b"external original").unwrap();
        let grant = root.grant();
        let target = PortablePath::parse("safe/important.txt").unwrap();
        fs::create_dir(root.0.join("safe")).unwrap();
        let mut workspace = grant.workspace(9).unwrap();
        let mut stage = workspace.stage().unwrap();
        stage.write_all(b"verified replacement").unwrap();
        let from = root.0.join("safe");
        let moved = root.0.join("old-safe");
        let external = outside.0.clone();
        let blocked = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let seen = Arc::clone(&blocked);
        workspace.before_publish = Some(Box::new(move || {
            match fs::rename(&from, &moved) {
                Ok(()) => link_directory(&external, &from),
                #[cfg(windows)]
                Err(e) if e.raw_os_error() == Some(32) => {
                    seen.store(true, Ordering::Relaxed);
                }
                Err(e) => panic!("parent attack fixture failed: {e}"),
            }
            #[cfg(not(windows))]
            let _ = seen;
        }));
        let commit = workspace.commit(stage, &target, None);
        if blocked.load(Ordering::Relaxed) {
            assert!(commit.is_ok());
            assert_eq!(
                fs::read(root.0.join("safe/important.txt")).unwrap(),
                b"verified replacement"
            );
        } else {
            assert!(commit.is_err());
        }
        assert_eq!(
            fs::read(outside.0.join("important.txt")).unwrap(),
            b"external original"
        );
        assert!(!root.0.join("old-safe/important.txt").exists());
        workspace.cleanup().unwrap();
    }
    #[test]
    fn destination_created_after_prepare_is_never_overwritten_by_no_replace() {
        let root = Owned::new();
        let grant = root.grant();
        let mut workspace = grant.workspace(10).unwrap();
        let mut stage = workspace.stage().unwrap();
        stage.write_all(b"new file").unwrap();
        let target = root.0.join("racing.txt");
        workspace.before_publish = Some(Box::new(move || {
            fs::write(target, b"concurrent editor").unwrap()
        }));
        assert!(matches!(
            workspace.commit(stage, &PortablePath::parse("racing.txt").unwrap(), None),
            Err(OutputError::Conflict)
        ));
        assert_eq!(
            fs::read(root.0.join("racing.txt")).unwrap(),
            b"concurrent editor"
        );
        workspace.cleanup().unwrap();
    }
    #[test]
    fn late_volume_conflict_rolls_back_only_new_parts() {
        let root = Owned::new();
        let grant = root.grant();
        let mut workspace = grant.workspace(12).unwrap();
        let mut first = workspace.stage().unwrap();
        first.write_all(b"first part").unwrap();
        let mut second = workspace.stage().unwrap();
        second.write_all(b"second part").unwrap();
        first.finish().unwrap();
        second.finish().unwrap();
        let concurrent = root.0.join("a.zip.002");
        workspace.before_publish = Some(Box::new(move || {
            std::fs::write(concurrent, b"other editor").unwrap()
        }));
        let result = workspace.commit_set(
            vec![first, second],
            &[
                PortablePath::parse("a.zip.001").unwrap(),
                PortablePath::parse("a.zip.002").unwrap(),
            ],
        );
        let failure = result.unwrap_err();
        assert_eq!(failure.error, OutputError::Conflict);
        assert!(!failure.rollback_failed);
        assert!(!root.0.join("a.zip.001").exists());
        assert_eq!(
            std::fs::read(root.0.join("a.zip.002")).unwrap(),
            b"other editor"
        );
        workspace.cleanup().unwrap();
    }
    #[test]
    fn a_stage_from_another_transaction_is_never_published() {
        let root = Owned::new();
        let grant = root.grant();
        let mut first = grant.workspace(13).unwrap();
        let mut second = grant.workspace(14).unwrap();
        let mut foreign = first.stage().unwrap();
        foreign.write_all(b"foreign data").unwrap();
        let result = second.commit_set(
            vec![foreign],
            &[PortablePath::parse("foreign.zip.001").unwrap()],
        );
        assert_eq!(result.unwrap_err().error, OutputError::UnsafeTarget);
        assert!(!root.0.join("foreign.zip.001").exists());
        first.cleanup().unwrap();
        second.cleanup().unwrap();
    }
    #[test]
    fn unknown_staging_objects_preserve_journal_instead_of_recursive_deletion() {
        let root = Owned::new();
        let grant = root.grant();
        let workspace = grant.workspace(11).unwrap();
        let name = workspace.name.clone();
        fs::write(root.0.join(&name).join("unfamiliar"), b"do not delete").unwrap();
        assert_eq!(workspace.cleanup(), Err(OutputError::Io));
        assert!(root.0.join(&name).join("transaction").is_file());
        assert_eq!(
            fs::read(root.0.join(name).join("unfamiliar")).unwrap(),
            b"do not delete"
        );
    }
}

#[cfg(all(test, unix))]
mod link_tests {
    use super::*;
    use crate::tests::Owned;
    use std::fs;
    #[test]
    fn link_publication_is_exclusive_and_never_follows_an_existing_leaf() {
        let root = Owned::new();
        let outside = Owned::new();
        let sentinel = outside.0.join("sentinel");
        fs::write(&sentinel, b"unchanged").unwrap();
        let grant = root.grant();
        let mut work = grant.workspace(1).unwrap();
        std::os::unix::fs::symlink(&sentinel, root.0.join("link")).unwrap();
        assert!(
            work.commit_symlink(
                &PortablePath::parse("link").unwrap(),
                &PortablePath::parse("safe").unwrap()
            )
            .is_err()
        );
        assert_eq!(fs::read(sentinel).unwrap(), b"unchanged");
        work.cleanup().unwrap();
    }
    #[test]
    fn parent_replacement_cannot_publish_link_into_an_external_directory() {
        let root = Owned::new();
        let outside = Owned::new();
        fs::create_dir(root.0.join("safe")).unwrap();
        let grant = root.grant();
        let mut work = grant.workspace(1).unwrap();
        let base = root.0.clone();
        let external = outside.0.clone();
        work.before_publish = Some(Box::new(move || {
            fs::rename(base.join("safe"), base.join("old-safe")).unwrap();
            std::os::unix::fs::symlink(external, base.join("safe")).unwrap();
        }));
        assert!(
            work.commit_symlink(
                &PortablePath::parse("safe/link").unwrap(),
                &PortablePath::parse("file").unwrap()
            )
            .is_err()
        );
        assert_eq!(fs::read_dir(&outside.0).unwrap().count(), 0);
        assert_eq!(fs::read_dir(root.0.join("old-safe")).unwrap().count(), 0);
        work.cleanup().unwrap();
    }
}
