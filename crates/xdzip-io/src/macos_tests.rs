use crate::tests::{Owned, path};
use crate::{Grant, OutputError};
use std::fs;
use std::io::Write;
use std::path::Path;

// Change only owned fixtures, using the native system tool rather than the
// product's ACL FFI. Do not print paths, account names or OS diagnostics.
fn add_acl(target: &Path, entry: &str) {
    let result = std::process::Command::new("/bin/chmod")
        .args(["+a", entry])
        .arg(target)
        .output()
        .unwrap();
    assert!(result.status.success(), "owned ACL setup failed");
}

fn acl_listing(target: &Path) -> Vec<u8> {
    let result = std::process::Command::new("/bin/ls")
        .arg("-lde")
        .arg(target)
        .output()
        .unwrap();
    assert!(result.status.success(), "owned ACL inspection failed");
    // The first line contains changing directory metadata; subsequent lines
    // contain the native ACL. Keep identity/path text out of failure output.
    result
        .stdout
        .split(|byte| *byte == b'\n')
        .skip(1)
        .flatten()
        .copied()
        .collect()
}

#[test]
fn safe_acl_allows_staging_inherited_output_and_replacement() {
    let account = std::process::Command::new("/usr/bin/id")
        .arg("-un")
        .output()
        .unwrap();
    assert!(account.status.success());
    let account = std::str::from_utf8(&account.stdout).unwrap().trim();
    let owner_rights = "read,write,append,execute,delete,delete_child,readattr,writeattr,readextattr,writeextattr,readsecurity,writesecurity,chown,file_inherit,directory_inherit";
    for entry in [
        None,
        Some("group:everyone allow list,search,readattr,readextattr,readsecurity,file_inherit,directory_inherit".to_owned()),
        Some("group:everyone allow read,file_inherit,directory_inherit,only_inherit".to_owned()),
        Some(format!("user:{account} allow {owner_rights}")),
    ] {
        let root = Owned::new();
        // Standard protection on macOS user directories; this denies a right
        // and does not grant anybody permission to modify the output tree.
        add_acl(&root.0, "group:everyone deny delete");
        if let Some(entry) = entry { add_acl(&root.0, &entry); }
        let acl_before = acl_listing(&root.0);
        let grant = root.grant();
        let mut workspace = grant.workspace(91).unwrap();
        let mut staged = workspace.stage().unwrap();
        staged.write_all(b"verified bytes").unwrap();
        let target = path("nested/result.txt");
        workspace.commit(staged, &target, None).unwrap();
        let existing = grant.existing(&target).unwrap().unwrap();
        let mut staged = workspace.stage().unwrap();
        staged.write_all(b"replacement bytes").unwrap();
        workspace.commit(staged, &target, Some(&existing)).unwrap();
        workspace.cleanup().unwrap();
        assert_eq!(fs::read(root.0.join("nested/result.txt")).unwrap(), b"replacement bytes");
        assert_eq!(grant.leftover_count().unwrap(), 0);
        assert!(Grant::for_creation(&root.0).is_ok());
        assert!(acl_before == acl_listing(&root.0), "original ACL preserved");
    }
}

#[test]
fn outside_mutation_acl_is_rejected_before_any_output() {
    let group = std::process::Command::new("/usr/bin/id")
        .arg("-gn")
        .output()
        .unwrap();
    assert!(group.status.success());
    let group = std::str::from_utf8(&group.stdout).unwrap().trim();
    for principal in [
        "group:everyone".to_owned(),
        "user:nobody".to_owned(),
        format!("group:{group}"),
    ] {
        for permission in [
            "write",
            "add_file",
            "append",
            "add_subdirectory",
            "delete",
            "delete_child",
            "writeattr",
            "writeextattr",
            "writesecurity",
            "chown",
        ] {
            for flags in ["", ",file_inherit,directory_inherit,only_inherit"] {
                let root = Owned::new();
                let source = root.0.join("source.zip");
                fs::write(&source, b"protected old bytes").unwrap();
                add_acl(&root.0, "group:everyone deny delete");
                add_acl(&root.0, &format!("{principal} allow {permission}{flags}"));
                let id = crate::FileIdentity::read(&fs::File::open(&source).unwrap()).unwrap();
                assert!(matches!(
                    Grant::open(&root.0, id),
                    Err(OutputError::UnsafeTarget)
                ));
                assert!(matches!(
                    Grant::for_creation(&root.0),
                    Err(OutputError::UnsafeTarget)
                ));
                assert_eq!(fs::read(&source).unwrap(), b"protected old bytes");
                assert_eq!(fs::read_dir(&root.0).unwrap().count(), 1);
            }
        }
    }
}

#[test]
fn acl_changes_are_rechecked_on_root_parents_and_existing_files() {
    let root = Owned::new();
    let grant = root.grant();
    let mut workspace = grant.workspace(92).unwrap();
    grant.ensure_directory(&path("nested")).unwrap();
    let target = path("nested/result.txt");
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"protected bytes").unwrap();
    workspace.commit(staged, &target, None).unwrap();
    let existing = grant.existing(&target).unwrap().unwrap();
    add_acl(
        &root.0.join("nested/result.txt"),
        "group:everyone allow writeextattr",
    );
    assert!(matches!(
        grant.existing(&target),
        Err(OutputError::UnsafeTarget)
    ));
    let mut staged = workspace.stage().unwrap();
    staged.write_all(b"rejected replacement").unwrap();
    assert!(matches!(
        workspace.commit(staged, &target, Some(&existing)),
        Err(OutputError::UnsafeTarget)
    ));
    assert_eq!(
        fs::read(root.0.join("nested/result.txt")).unwrap(),
        b"protected bytes"
    );
    add_acl(&root.0.join("nested"), "group:everyone allow delete_child");
    assert_eq!(
        grant.ensure_directory(&path("nested/child")),
        Err(OutputError::UnsafeTarget)
    );
    assert!(!root.0.join("nested/child").exists());
    workspace.cleanup().unwrap();
    assert_eq!(grant.leftover_count().unwrap(), 0);
    add_acl(
        &root.0,
        "group:everyone allow write,file_inherit,directory_inherit,only_inherit",
    );
    assert_eq!(grant.validate(), Err(OutputError::UnsafeTarget));
    assert!(matches!(
        grant.workspace(93),
        Err(OutputError::UnsafeTarget)
    ));
}

#[test]
#[allow(unsafe_code)]
fn synthetic_uuid_only_trusts_the_current_user_not_a_group_with_the_same_id() {
    use std::os::fd::AsRawFd;
    use std::os::raw::{c_int, c_void};
    use std::ptr::null_mut;
    unsafe extern "C" {
        fn acl_init(count: c_int) -> *mut c_void;
        fn acl_create_entry(acl: *mut *mut c_void, entry: *mut *mut c_void) -> c_int;
        fn acl_set_tag_type(entry: *mut c_void, tag: c_int) -> c_int;
        fn acl_set_qualifier(entry: *mut c_void, qualifier: *const c_void) -> c_int;
        fn acl_set_permset_mask_np(entry: *mut c_void, mask: u64) -> c_int;
        fn acl_set_fd_np(fd: c_int, acl: *mut c_void, kind: c_int) -> c_int;
        fn acl_free(acl: *mut c_void) -> c_int;
    }
    struct Acl(*mut c_void);
    impl Drop for Acl {
        fn drop(&mut self) {
            // SAFETY: the native initializer allocated this ACL; borrowed
            // entries have expired, including when a setup assertion panics.
            unsafe { acl_free(self.0) };
        }
    }
    let uid = rustix::process::geteuid().as_raw();
    for (prefix, id, accepted) in [
        (
            [
                0xff, 0xff, 0xee, 0xee, 0xdd, 0xdd, 0xcc, 0xcc, 0xbb, 0xbb, 0xaa, 0xaa,
            ],
            uid,
            true,
        ),
        (
            [
                0xff, 0xff, 0xee, 0xee, 0xdd, 0xdd, 0xcc, 0xcc, 0xbb, 0xbb, 0xaa, 0xaa,
            ],
            uid.checked_add(1).unwrap(),
            false,
        ),
        (
            [
                0xaa, 0xaa, 0xbb, 0xbb, 0xcc, 0xcc, 0xdd, 0xdd, 0xee, 0xee, 0xff, 0xff,
            ],
            uid,
            false,
        ),
    ] {
        let root = Owned::new();
        let file = fs::File::open(&root.0).unwrap();
        let mut uuid = [0u8; 16];
        uuid[..12].copy_from_slice(&prefix);
        uuid[12..].copy_from_slice(&id.to_be_bytes());
        // chmod resolves names rather than accepting arbitrary synthetic UUIDs.
        // Use native setter APIs solely for this owned fixture. The product only
        // queries ACLs; it never invokes these setters or rewrites user ACLs.
        let pointer = unsafe { acl_init(1) };
        assert!(!pointer.is_null());
        let mut acl = Acl(pointer);
        let mut entry = null_mut();
        // SAFETY: live owned ACL, valid output storage, full UUID and borrowed
        // descriptor. acl_create_entry may replace the allocation owned by Acl.
        unsafe {
            assert_eq!(acl_create_entry(&mut acl.0, &mut entry), 0);
            assert_eq!(acl_set_tag_type(entry, 1), 0);
            assert_eq!(acl_set_qualifier(entry, uuid.as_ptr().cast()), 0);
            assert_eq!(acl_set_permset_mask_np(entry, 1 << 2), 0);
            assert_eq!(acl_set_fd_np(file.as_raw_fd(), acl.0, 0x100), 0);
        }
        if accepted {
            let grant = Grant::for_creation(&root.0).unwrap();
            let mut workspace = grant.workspace(94).unwrap();
            let mut staged = workspace.stage().unwrap();
            staged.write_all(b"owner UUID output").unwrap();
            workspace.commit(staged, &path("result.zip"), None).unwrap();
            workspace.cleanup().unwrap();
            assert_eq!(
                fs::read(root.0.join("result.zip")).unwrap(),
                b"owner UUID output"
            );
        } else {
            assert!(matches!(
                Grant::for_creation(&root.0),
                Err(OutputError::UnsafeTarget)
            ));
            assert_eq!(fs::read_dir(&root.0).unwrap().count(), 0);
        }
    }
}
