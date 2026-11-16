use super::*;
use crate::tests::{fixture, scan, with_extension};
#[test]
fn raw_dirname_boundaries_and_decoded_separators_keep_the_same_complete_name() {
    let bytes = with_extension(2, b"folder\xff");
    let index = scan(&bytes).unwrap();
    let rows = resolve_names(&index, &Cancellation::default(), ScanLimits::default()).unwrap();
    assert_eq!(index.records[0].raw_name, b"folder\xffhello.bin");
    assert_eq!(rows[0].decoded.as_deref(), Some("folder/hello.bin"));
    assert_eq!(rows[0].display, "folder/hello.bin");
    assert!(!rows[0].warning);
    let directory = [b"nested\xff".as_slice(), "中文".as_bytes(), &[255]].concat();
    let index = scan(&with_extension(2, &directory)).unwrap();
    let names = resolve_names(&index, &Cancellation::default(), ScanLimits::default()).unwrap();
    assert_eq!(names[0].decoded.as_deref(), Some("nested/中文/hello.bin"));
    assert_eq!(names[0].encoding, NameEncoding::UnmarkedUtf8);
    assert!(!names[0].warning);
    let raw = fixture("lh0-h0.lzh");
    let mut index = scan(&raw).unwrap();
    index.records[0].directory = true;
    index.records[0].raw_name = b"folder\\".to_vec();
    let rows = resolve_names(&index, &Cancellation::default(), ScanLimits::default()).unwrap();
    assert_eq!(rows[0].decoded.as_deref(), Some("folder/"));
    assert!(!rows[0].warning);
}
#[test]
fn independently_encoded_shift_jis_trail_backslash_is_a_character_before_path_conversion() {
    // Python's standard Shift-JIS encoder produced this complete field for
    // ソフトウェアの設定; the first 0x5c is a multibyte trail, not a separator.
    let directory = [
        0x83, 0x5c, 0x83, 0x74, 0x83, 0x67, 0x83, 0x45, 0x83, 0x46, 0x83, 0x41, 0x82, 0xcc, 0x90,
        0xdd, 0x92, 0xe8, 255,
    ];
    let index = scan(&with_extension(2, &directory)).unwrap();
    let rows = resolve_names(&index, &Cancellation::default(), ScanLimits::default()).unwrap();
    assert_eq!(
        rows[0].decoded.as_deref(),
        Some("ソフトウェアの設定/hello.bin")
    );
    assert_eq!(rows[0].encoding, NameEncoding::Legacy("Shift_JIS"));
    assert!(!rows[0].warning);
    assert!(index.records[0].raw_name.starts_with(&[0x83, 0x5c]));
    let mut separated = index.clone();
    separated.records[0].raw_name = b"\x83\xff\\safe.bin".to_vec();
    separated.records[0].dirname_bytes = 2;
    let rows = resolve_names(&separated, &Cancellation::default(), ScanLimits::default()).unwrap();
    assert!(!rows[0].decoded.as_deref().unwrap_or("").contains('ソ'));
}
#[test]
fn original_unsafe_components_and_control_text_do_not_gain_safe_paths_when_joined() {
    for field in [
        b"..\xff".as_slice(),
        b"/absolute\xff",
        b"folder\\..\xff",
        b"folder\xff\xff",
    ] {
        let index = scan(&with_extension(2, field)).unwrap();
        let rows = resolve_names(&index, &Cancellation::default(), ScanLimits::default()).unwrap();
        assert!(rows[0].warning);
    }
    let mut index = scan(&fixture("lh5-h0.lzh")).unwrap();
    index.records[0].raw_name = "dir\u{202e}name/file.txt".as_bytes().to_vec();
    let rows = resolve_names(&index, &Cancellation::default(), ScanLimits::default()).unwrap();
    assert!(rows[0].warning);
    assert!(rows[0].display.contains("\\u{202E}"));
}
#[test]
fn retained_text_raw_field_and_cancellation_limits_apply_without_partial_results() {
    let index = scan(&with_extension(2, b"folder\xff")).unwrap();
    let cancel = Cancellation::default();
    let rows = resolve_names(&index, &cancel, ScanLimits::default()).unwrap();
    let text = rows[0].display.capacity() + rows[0].decoded.as_ref().map_or(0, String::capacity);
    let exact = ScanLimits {
        entries: 1,
        name_bytes: 16,
        total_name_bytes: index.records[0].raw_name.capacity(),
        display_name_bytes: text,
        ..ScanLimits::default()
    };
    resolve_names(&index, &cancel, exact).unwrap();
    for limit in [
        ScanLimits {
            entries: 0,
            ..exact
        },
        ScanLimits {
            name_bytes: 15,
            ..exact
        },
        ScanLimits {
            total_name_bytes: exact.total_name_bytes - 1,
            ..exact
        },
        ScanLimits {
            display_name_bytes: text - 1,
            ..exact
        },
    ] {
        assert_eq!(resolve_names(&index, &cancel, limit), Err(ScanError::Limit));
    }
    let mut invalid = index.clone();
    invalid.records[0].dirname_bytes = usize::MAX;
    assert_eq!(
        resolve_names(&invalid, &cancel, exact),
        Err(ScanError::Invalid)
    );
    invalid.records[0].dirname_bytes = 1;
    assert_eq!(
        resolve_names(&invalid, &cancel, exact),
        Err(ScanError::Invalid)
    );
    cancel.cancel();
    assert_eq!(
        resolve_names(&index, &cancel, exact),
        Err(ScanError::Cancelled)
    );
}
