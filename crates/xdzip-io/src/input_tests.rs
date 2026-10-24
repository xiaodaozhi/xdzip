// crates/xdzip-io/src/input_tests.rs

use crate::{InputError, InputTree};
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "xdzip-metadata-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        for path in [
            "资料/真空目录",
            "资料/仅元数据/下级/__MACOSX",
            "资料/__MACOSX",
            "资料/下级/_MACOSX",
            "__MACOSX",
            "_MACOSX",
            "仅元数据/下级",
            "空目录",
        ] {
            fs::create_dir_all(root.join(path)).unwrap();
        }
        for path in [
            ".DS_Store",
            "__MACOSX/._root",
            "_MACOSX/._root",
            "资料/.DS_Store",
            "资料/__MACOSX/._file",
            "资料/下级/.DS_Store",
            "资料/下级/_MACOSX/._file",
            "资料/仅元数据/下级/__MACOSX/._file",
            "仅元数据/下级/.DS_Store",
        ] {
            fs::write(root.join(path), b"metadata").unwrap();
        }
        for path in [
            "资料/文件.txt",
            "资料/._keep",
            "资料/.DS_Store.backup",
            "资料/下级/zero",
        ] {
            fs::write(root.join(path), b"").unwrap();
        }
        Self(root)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn names(tree: &InputTree) -> BTreeSet<&str> {
    tree.entries().iter().map(|e| e.name.as_str()).collect()
}
#[test]
fn archive_policy_filters_root_and_nested_metadata_without_deleting_sources() {
    let f = Fixture::new();
    let selected: Vec<_> = [
        ".DS_Store",
        "__MACOSX",
        "__MACOSX/._root",
        "_MACOSX",
        "资料",
    ]
    .iter()
    .map(|p| f.0.join(p))
    .collect();
    let tree = InputTree::scan_for_archive(&selected, vec![], &|| false).unwrap();
    assert_eq!(
        names(&tree),
        BTreeSet::from([
            "资料",
            "资料/真空目录",
            "资料/文件.txt",
            "资料/._keep",
            "资料/.DS_Store.backup",
            "资料/下级",
            "资料/下级/zero"
        ])
    );
    assert!(f.0.join("资料/.DS_Store").is_file());
    assert!(f.0.join("资料/仅元数据/下级/__MACOSX/._file").is_file());
    assert!(f.0.join("__MACOSX/._root").is_file());
    assert_eq!(tree.total_bytes(), 0);
}
#[test]
fn filtered_only_directories_are_empty_but_genuine_empty_inputs_survive() {
    let f = Fixture::new();
    for path in [".DS_Store", "__MACOSX", "_MACOSX", "仅元数据"] {
        assert!(
            matches!(
                InputTree::scan_for_archive(&[f.0.join(path)], vec![], &|| false),
                Err(InputError::Empty)
            ),
            "{path}"
        );
    }
    let tree = InputTree::scan_for_archive(
        &[f.0.join("空目录"), f.0.join("资料/下级/zero")],
        vec![],
        &|| false,
    )
    .unwrap();
    assert_eq!(names(&tree), BTreeSet::from(["空目录", "zero"]));
    assert_eq!(tree.total_bytes(), 0);
    // Reserved directory names are not a wildcard for ordinary user files.
    fs::write(f.0.join("资料/_MACOSX"), b"ordinary file").unwrap();
    let tree = InputTree::scan_for_archive(&[f.0.join("资料/_MACOSX")], vec![], &|| false).unwrap();
    assert_eq!(tree.total_bytes(), 13);
}
#[test]
fn frozen_tree_ignores_metadata_churn_but_rejects_ordinary_membership_changes() {
    let f = Fixture::new();
    let tree = InputTree::scan_for_archive(&[f.0.join("资料")], vec![], &|| false).unwrap();
    fs::remove_file(f.0.join("资料/.DS_Store")).unwrap();
    fs::write(f.0.join("资料/真空目录/.DS_Store"), b"new metadata").unwrap();
    fs::write(f.0.join("资料/__MACOSX/new"), b"new metadata").unwrap();
    tree.validate(&|| false).unwrap();
    fs::write(f.0.join("资料/仅元数据/new.txt"), b"new ordinary input").unwrap();
    assert_eq!(tree.validate(&|| false), Err(InputError::Changed));
    fs::remove_file(f.0.join("资料/仅元数据/new.txt")).unwrap();
    tree.validate(&|| false).unwrap();
    fs::remove_file(f.0.join("资料/文件.txt")).unwrap();
    assert_eq!(tree.validate(&|| false), Err(InputError::Changed));
}
#[test]
fn ordinary_publication_scan_keeps_all_metadata() {
    let f = Fixture::new();
    let tree = InputTree::scan(
        &[f.0.join("资料"), f.0.join(".DS_Store")],
        vec![],
        &|| false,
    )
    .unwrap();
    let names = names(&tree);
    for name in [
        ".DS_Store",
        "资料/.DS_Store",
        "资料/__MACOSX/._file",
        "资料/仅元数据/下级/__MACOSX/._file",
    ] {
        assert!(names.contains(name));
    }
}
