//! Explicit large-stream checking, separate from the ordinary native fixture batch.
use crate::{Archive, native_backend_tests::check};
use std::path::PathBuf;
use xdzip_codec_api::Format;

#[test]
#[ignore = "requires Rust worker and pinned XDZIP_RAR7_GENUINE_FIXTURE; verifies 4.25GiB with no destination"]
fn full_check_keeps_memory_and_u64_counters() {
    const SIZE: u64 = 4_563_402_752;
    let path = PathBuf::from(
        std::env::var_os("XDZIP_RAR7_GENUINE_FIXTURE").expect("pinned genuine algorithm1 data"),
    );
    let archive = Archive::open(&path, &Default::default()).unwrap();
    assert_eq!(archive.format(), Format::Rar);
    assert_eq!(archive.rows.len(), 1);
    assert_eq!((archive.rows[0].method, archive.rows[0].size), (70, SIZE));
    assert!(archive.rows[0].size_known);
    let checked = check(&archive, false);
    assert_eq!(
        (
            checked.verified_files,
            checked.verified_bytes,
            checked.decoded_bytes
        ),
        (1, SIZE, SIZE)
    );
    assert_eq!(checked.declared_bytes, SIZE);
    assert!(checked.read_bytes > 0);
    assert!(checked.target.is_empty() && checked.open_path.is_empty());
    assert_eq!((checked.written_bytes, checked.committed_bytes), (0, 0));
    println!(
        "RAR7_RUNTIME_RESULT\t{}\t{}\t{}\t{}\t{}\t{}\tno_destination",
        checked.verified_files,
        checked.verified_bytes,
        checked.decoded_bytes,
        checked.read_bytes,
        checked.written_bytes,
        checked.committed_bytes
    );
}
