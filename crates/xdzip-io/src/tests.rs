use super::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use xdzip_domain::path::PortablePath;

pub(crate) struct Owned(pub(crate) PathBuf);
impl Owned {
    pub(crate) fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        // macOS clock ticks can repeat across parallel tests. Time is a label,
        // not uniqueness; the process-local counter separates owned fixtures.
        let serial = NEXT.fetch_add(1, Ordering::Relaxed);
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("xdzip-io-{}-{nonce}-{serial}", std::process::id()));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    pub(crate) fn grant(&self) -> std::sync::Arc<Grant> {
        let source = self.0.join("source.zip");
        fs::write(&source, b"source must stay intact").unwrap();
        let identity = FileIdentity::read(&File::open(source).unwrap()).unwrap();
        Grant::open(&self.0, identity).unwrap()
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
pub(crate) fn path(value: &str) -> PortablePath {
    PortablePath::parse(value).unwrap()
}

#[test]
fn all_source_volumes_are_protected_and_temporary_cache_only_sweeps_expired_unlocked_outputs() {
    for _ in 0..16 {
        cache_lease_round_trip();
    }
}
fn cache_lease_round_trip() {
    let root = Owned::new();
    let mut ids = Vec::new();
    for name in ["archive.zip.001", "archive.zip.002"] {
        fs::write(root.0.join(name), b"source").unwrap();
        ids.push(FileIdentity::read(&File::open(root.0.join(name)).unwrap()).unwrap());
    }
    let grant = Grant::open_sources(&root.0, &ids).unwrap();
    for name in ["archive.zip.001", "archive.zip.002"] {
        assert!(
            grant
                .existing(&path(name))
                .unwrap()
                .unwrap()
                .source_protected
        );
    }
    let first = Grant::temporary(&root.0, &ids).unwrap();
    let directory = first.display_path().parent().unwrap().to_path_buf();
    let marker = fs::OpenOptions::new()
        .write(true)
        .open(directory.join("lease"))
        .unwrap();
    marker
        .set_times(
            fs::FileTimes::new()
                .set_modified(std::time::UNIX_EPOCH + std::time::Duration::from_secs(1)),
        )
        .unwrap();
    drop(marker);
    assert!(
        std::time::SystemTime::now()
            .duration_since(
                fs::metadata(directory.join("lease"))
                    .unwrap()
                    .modified()
                    .unwrap()
            )
            .unwrap()
            .as_secs()
            > 7 * 24 * 60 * 60,
        "fixture lease is actually expired"
    );
    let second = Grant::temporary(&root.0, &ids).unwrap();
    assert!(
        directory.exists(),
        "active cache lease must survive age-based cleanup"
    );
    drop(first);
    let foreign = root.0.join(format!("open-{}", "0".repeat(32)));
    fs::create_dir(&foreign).unwrap();
    fs::write(foreign.join("lease"), [1u8; 16]).unwrap();
    let third = Grant::temporary(&root.0, &ids).unwrap();
    assert!(!directory.exists());
    assert!(foreign.exists());
    assert!(second.display_path().is_dir());
    assert!(third.display_path().is_dir());
}

#[test]
fn verified_publication_conflict_and_conditional_replace_keep_old_bytes() {
    let root = Owned::new();
    let grant = root.grant();
    let mut workspace = grant.workspace(1).unwrap();
    let target = path("目录/a.txt");
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"whole verified content").unwrap();
    workspace.commit(staged, &target, None).unwrap();
    let existing = grant.existing(&target).unwrap().unwrap();
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"must not overwrite").unwrap();
    assert!(matches!(
        workspace.commit(staged, &target, None),
        Err(OutputError::Conflict)
    ));
    assert_eq!(
        fs::read(root.0.join("目录/a.txt")).unwrap(),
        b"whole verified content"
    );
    fs::write(root.0.join("目录/a.txt"), b"editor changed this file").unwrap();
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"stale authorization").unwrap();
    assert!(matches!(
        workspace.commit(staged, &target, Some(&existing)),
        Err(OutputError::TargetChanged)
    ));
    assert_eq!(
        fs::read(root.0.join("目录/a.txt")).unwrap(),
        b"editor changed this file"
    );
    let current = grant.existing(&target).unwrap().unwrap();
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"authorized replacement").unwrap();
    workspace.commit(staged, &target, Some(&current)).unwrap();
    assert_eq!(
        fs::read(root.0.join("目录/a.txt")).unwrap(),
        b"authorized replacement"
    );
    workspace.cleanup().unwrap();
    assert_eq!(grant.leftover_count().unwrap(), 0);
}
#[test]
fn dropped_stage_is_unpublished_and_source_cannot_be_replaced() {
    let root = Owned::new();
    let grant = root.grant();
    let mut workspace = grant.workspace(2).unwrap();
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"unverified prefix").unwrap();
    drop(staged);
    assert!(!root.0.join("partial.txt").exists());
    let source = path("source.zip");
    let existing = grant.existing(&source).unwrap().unwrap();
    assert!(existing.source_protected);
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"do not replace source").unwrap();
    assert!(matches!(
        workspace.commit(staged, &source, Some(&existing)),
        Err(OutputError::UnsafeTarget)
    ));
    assert_eq!(
        fs::read(root.0.join("source.zip")).unwrap(),
        b"source must stay intact"
    );
    workspace.cleanup().unwrap();
}
#[test]
fn existing_hard_link_is_refused_and_external_content_is_unchanged() {
    let root = Owned::new();
    let outside = Owned::new();
    fs::write(outside.0.join("important.txt"), b"important original").unwrap();
    fs::hard_link(outside.0.join("important.txt"), root.0.join("linked.txt")).unwrap();
    let grant = root.grant();
    assert!(matches!(
        grant.existing(&path("linked.txt")),
        Err(OutputError::UnsafeTarget)
    ));
    assert_eq!(
        fs::read(outside.0.join("important.txt")).unwrap(),
        b"important original"
    );
}
#[cfg(unix)]
#[test]
fn linked_parent_linked_leaf_and_root_replacement_never_write_outside() {
    use std::os::unix::fs::symlink;
    let root = Owned::new();
    let outside = Owned::new();
    fs::write(outside.0.join("important.txt"), b"outside unchanged").unwrap();
    let grant = root.grant();
    symlink(&outside.0, root.0.join("linked")).unwrap();
    symlink(outside.0.join("important.txt"), root.0.join("leaf.txt")).unwrap();
    assert!(grant.ensure_directory(&path("linked/sub")).is_err());
    assert!(matches!(
        grant.existing(&path("leaf.txt")),
        Err(OutputError::UnsafeTarget)
    ));
    let mut workspace = grant.workspace(3).unwrap();
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"bad replacement").unwrap();
    assert!(
        workspace
            .commit(staged, &path("linked/important.txt"), None)
            .is_err()
    );
    assert_eq!(
        fs::read(outside.0.join("important.txt")).unwrap(),
        b"outside unchanged"
    );
    assert!(!outside.0.join("sub").exists());
    workspace.cleanup().unwrap();
    let old = root.0.with_extension("old");
    fs::rename(&root.0, &old).unwrap();
    symlink(&outside.0, &root.0).unwrap();
    assert_eq!(grant.validate(), Err(OutputError::TargetChanged));
    fs::remove_file(&root.0).unwrap();
    fs::rename(old, &root.0).unwrap();
}
#[cfg(unix)]
#[test]
fn broad_write_permissions_are_refused_before_staging() {
    use std::os::unix::fs::PermissionsExt;
    let root = Owned::new();
    let source = File::create(root.0.join("source")).unwrap();
    fs::set_permissions(&root.0, fs::Permissions::from_mode(0o777)).unwrap();
    assert!(matches!(
        Grant::open(&root.0, FileIdentity::read(&source).unwrap()),
        Err(OutputError::UnsafeTarget)
    ));
}

#[cfg(windows)]
#[test]
fn windows_read_only_acl_allows_output_but_mutation_does_not() {
    // Modify only newly owned fixtures through the system ACL tool, not product
    // FFI. Native filesystem checks exercise effective and inherited grants.
    fn grant_access(root: &Owned, sid: &str, rights: &str) {
        let tool =
            PathBuf::from(std::env::var_os("SystemRoot").unwrap()).join("System32/icacls.exe");
        let result = std::process::Command::new(tool)
            .arg(&root.0)
            .arg("/grant:r")
            .arg(format!("*{sid}:(OI)(CI){rights}"))
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "owned ACL setup: {:?}",
            result.status.code()
        );
    }
    for rights in ["(RA,RC,S)", "(RD)", "(R)", "(RX)", "(GR,GE)"] {
        let root = Owned::new();
        grant_access(&root, "S-1-1-0", rights);
        // OWNER RIGHTS is a dynamic reference to the already trusted object owner.
        // Exercise inheritance on real stage files and newly created output parents.
        if rights == "(GR,GE)" {
            grant_access(&root, "S-1-3-4", "(F)");
        }
        let grant = root.grant();
        let mut workspace = grant.workspace(90).unwrap();
        let mut staged = workspace.stage().unwrap();
        staged.write_all(b"verified private bytes").unwrap();
        workspace
            .commit(staged, &path("nested/private.txt"), None)
            .unwrap();
        workspace.cleanup().unwrap();
        assert_eq!(
            fs::read(root.0.join("nested/private.txt")).unwrap(),
            b"verified private bytes"
        );
        let id = FileIdentity::read(&File::open(root.0.join("source.zip")).unwrap()).unwrap();
        let temporary = Grant::temporary(&root.0, &[id]).unwrap();
        assert!(temporary.display_path().is_dir());
        drop(grant);
    }
    // Read-only sharing does not authorize mutation. Both effective and
    // inherited write/delete/ACL rights and generic write/all must still
    // reject before creating a staging directory or touching any existing bytes.
    for rights in [
        "(W)", "(D)", "(WDAC)", "(WO)", "(DC)", "(GW)", "(F)", "(IO)(W)", "(IO)(F)",
    ] {
        let root = Owned::new();
        let source = root.0.join("source.zip");
        fs::write(&source, b"protected old bytes").unwrap();
        let id = FileIdentity::read(&File::open(&source).unwrap()).unwrap();
        grant_access(&root, "S-1-3-4", "(F)");
        grant_access(&root, "S-1-1-0", rights);
        assert!(matches!(
            Grant::open(&root.0, id),
            Err(OutputError::UnsafeTarget)
        ));
        assert_eq!(fs::read(&source).unwrap(), b"protected old bytes");
        assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
    }
}

#[test]
fn crashed_private_stage_is_reported_and_never_auto_removed() {
    const VARIABLE: &str = "XDZIP_OWNED_CRASH_TARGET";
    if let Some(path) = std::env::var_os(VARIABLE) {
        let root = std::path::PathBuf::from(path);
        let source = fs::File::open(root.join("source.zip")).unwrap();
        let grant = Grant::open(&root, FileIdentity::read(&source).unwrap()).unwrap();
        let mut workspace = grant.workspace(70).unwrap();
        let mut stage = workspace.stage().unwrap();
        stage.write_all(b"uncommitted crash bytes").unwrap();
        // Exit without running RAII destructors; this owned child is the crash fixture.
        std::process::exit(73);
    }
    let root = Owned::new();
    fs::write(root.0.join("source.zip"), b"source must stay intact").unwrap();
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "tests::crashed_private_stage_is_reported_and_never_auto_removed",
        ])
        .env(VARIABLE, &root.0)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(73));
    let grant = root.grant();
    assert_eq!(grant.leftover_count().unwrap(), 1);
    let crashed = fs::read_dir(&root.0)
        .unwrap()
        .filter_map(Result::ok)
        .find(|e| e.file_name().to_string_lossy().starts_with(".xdzip-work-"))
        .unwrap()
        .path();
    assert!(crashed.join("transaction").is_file());
    assert_eq!(
        fs::read(crashed.join("1.part")).unwrap(),
        b"uncommitted crash bytes"
    );
    // Starting and cleaning a new transaction preserves the previous one.
    grant.workspace(71).unwrap().cleanup().unwrap();
    assert_eq!(grant.leftover_count().unwrap(), 1);
    assert!(!root.0.join("result.txt").exists());
}
