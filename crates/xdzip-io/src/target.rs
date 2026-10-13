use crate::{FileIdentity, FileVersion, OutputError, failure, private};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsMaybeDirExt};
use cap_std::fs::{Dir, DirBuilder, OpenOptions};
use std::fs::File;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use xdzip_domain::path::PortablePath;

/// User-selected directory object. Output paths are resolved only below its held handle.
pub struct Grant {
    dir: Dir,
    path: PathBuf,
    identity: FileIdentity,
    source: FileIdentity,
    created_directories: Arc<AtomicU64>,
    parent_grant: Option<Arc<Grant>>,
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
            source: self.source,
            created_directories: Arc::clone(&self.created_directories),
            parent_grant: Some(Arc::clone(self)),
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
            source_protected: version.identity == self.source,
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
            dir: Some(dir),
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
    dir: Option<Dir>,
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
impl Workspace {
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
            dir: self.dir().try_clone().map_err(failure)?,
            identity,
            name,
            published: false,
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
    dir: Dir,
    identity: FileIdentity,
    name: String,
    published: bool,
}
impl Stage {
    /// Synchronize verified file data before publication, detecting delayed I/O failures.
    pub fn finish(&mut self) -> Result<(), OutputError> {
        if let Some(mut file) = self.file.take() {
            file.flush().map_err(failure)?;
            file.sync_all().map_err(failure)?;
            if FileIdentity::read(&file).map_err(failure)? != self.identity {
                return Err(OutputError::TargetChanged);
            }
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
