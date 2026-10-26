//! Explicit independent-author experiments. No RAR compressor is implemented.
use crate::*;
use std::io::{Cursor, Read};
use std::path::{Component, Path};
use xdzip_codec_api::payload::DecodedSink;

pub(crate) struct Case {
    pub label: String,
    pub method: u16,
    pub archive: Vec<u8>,
    pub original: Vec<u8>,
    pub required: Option<usize>,
}
pub(crate) fn digest(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut hash = xdzip_crypto::SecretSha256::new();
    hash.update(bytes);
    let mut result = String::new();
    for byte in hash.finish().iter() {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}
fn read(path: &Path, cap: u64) -> Vec<u8> {
    let file = std::fs::File::open(path).unwrap();
    assert!(file.metadata().unwrap().len() <= cap);
    let mut bytes = Vec::new();
    file.take(cap + 1).read_to_end(&mut bytes).unwrap();
    assert!(bytes.len() as u64 <= cap);
    bytes
}
pub(crate) fn cases(method: u16) -> Vec<Case> {
    let directory =
        std::env::var_os("XDZIP_RAR_FILTER_AUTHOR_FIXTURES").expect("explicit fixture directory");
    let directory = Path::new(&directory);
    let text = String::from_utf8(read(&directory.join("cases.tsv"), 65536)).unwrap();
    let mut cases = Vec::new();
    let mut total = 0usize;
    for line in text
        .lines()
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
    {
        let f: Vec<_> = line.split('\t').collect();
        assert_eq!(f.len(), 7);
        let actual_method = f[1].parse::<u16>().unwrap();
        if actual_method != method {
            continue;
        }
        for file in [f[2], f[4]] {
            assert!(
                Path::new(file)
                    .components()
                    .all(|c| matches!(c, Component::Normal(_)))
            );
        }
        let cap = (64usize * 1024 * 1024 - total).min(8 * 1024 * 1024) as u64;
        let archive = read(&directory.join(f[2]), cap);
        total += archive.len();
        let cap = (64usize * 1024 * 1024 - total).min(8 * 1024 * 1024) as u64;
        let original = read(&directory.join(f[4]), cap);
        total += original.len();
        assert_eq!(digest(&archive), f[3], "archive identity {}", f[0]);
        assert_eq!(digest(&original), f[5], "original identity {}", f[0]);
        let required = if f[6] == "observe" {
            None
        } else {
            Some(f[6].parse().unwrap())
        };
        cases.push(Case {
            label: f[0].to_owned(),
            method,
            archive,
            original,
            required,
        });
    }
    assert!(!cases.is_empty());
    assert!(cases.len() <= 128);
    cases
}

#[test]
fn fixed_official_591_itanium_matches_original_bundle_bytes_and_executes() {
    // Original ISA-like test bytes, never executed. Only the official author
    // produces the RAR bitstream. No RAR encoder is implemented here.
    let mut original = Vec::with_capacity(524288);
    for i in 0..32768u32 {
        let mut bundle = 22u128;
        for slot in 0..3u32 {
            let start = 5 + 41 * slot;
            bundle |= 5u128 << (start + 37);
            bundle |= u128::from(0x4000u32.wrapping_sub(i).wrapping_add(slot * 4) & 0xfffff)
                << (start + 13);
        }
        original.extend_from_slice(&bundle.to_le_bytes());
    }
    assert_eq!(
        digest(&original),
        "d6d8cd0cc8833fda7b5f4cc89ff77bd32b358a3b9e6e53f00e0e095d647cf4cb"
    );
    let archive = include_bytes!("../tests/fixtures/filter-author/rar29-itanium.rar").to_vec();
    assert_eq!(
        digest(&archive),
        "2c573d132e16f1e10ace7306f1b3adde21f51bcfa8587991eea6f64de187d298"
    );
    let case = Case {
        label: "fixed-official591-itanium".into(),
        method: 29,
        archive,
        original,
        required: Some(2),
    };
    crate::filter29_transform::take_executed();
    check_payload(&case);
    let counts = crate::filter29_transform::take_executed();
    assert_eq!(counts, [0, 0, 8, 0, 0, 0]);
    println!(
        "FILTER_AUTHOR_RESULT\t{}\t29\t{}\t{}\t{counts:?}",
        case.label,
        case.original.len(),
        digest(&case.original)
    );
}
#[test]
fn fixed_official_591_e8e9_matches_original_branches_and_executes() {
    // These original branch-like bytes are data only and are never executed.
    let mut original = Vec::with_capacity(2048);
    for i in 0..256 {
        let position = i32::try_from(original.len()).unwrap();
        original.push(if i % 8 == 0 { 0xe8 } else { 0xe9 });
        original.extend_from_slice(&(0x10000i32 - position - 5).to_le_bytes());
        original.extend_from_slice(&[0x90; 3]);
    }
    assert_eq!(
        digest(&original),
        "6de2e3e6e612b27a70138e34c512a2799c16f709a663583d5d50c36fe3f884f3"
    );
    let archive = include_bytes!("../tests/fixtures/filter-author/rar29-e8e9-auto.rar").to_vec();
    assert_eq!(
        digest(&archive),
        "df693c44b99e9eeebd261f1cb55760047efe5add584c22f050bfb58258ef842a"
    );
    let case = Case {
        label: "fixed-official591-e8e9".into(),
        method: 29,
        archive,
        original,
        required: Some(1),
    };
    crate::filter29_transform::take_executed();
    check_payload(&case);
    let counts = crate::filter29_transform::take_executed();
    assert_eq!(counts, [0, 1, 0, 0, 0, 0]);
    println!(
        "FILTER_AUTHOR_RESULT\t{}\t29\t{}\t{}\t{counts:?}",
        case.label,
        case.original.len(),
        digest(&case.original)
    );
}

#[derive(Default)]
pub(crate) struct Sink(pub Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, b: &[u8]) -> Result<(), DecodeError> {
        if self
            .0
            .len()
            .checked_add(b.len())
            .is_none_or(|n| n > 8 * 1024 * 1024)
        {
            return Err(DecodeError::Limit);
        }
        self.0.extend_from_slice(b);
        Ok(())
    }
}
impl crate::algorithm::WorkOutput for Sink {}
pub(crate) fn check_payload(case: &Case) {
    let control = Cancellation::default();
    let mut source = Cursor::new(&case.archive);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].method, case.method);
    assert_eq!(rows[0].size, case.original.len() as u64);
    let mut reader = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    reader.decode(&rows[0], &mut sink, &control).unwrap();
    assert_eq!(sink.0, case.original, "complete original {}", case.label);
}
#[test]
#[ignore = "explicit independent official-author inputs and original byte identities; no compressor"]
fn legacy_independent_author_known_bytes_and_actual_transform() {
    for case in cases(29) {
        crate::filter29_transform::take_executed();
        check_payload(&case);
        let counts = crate::filter29_transform::take_executed();
        println!(
            "FILTER_AUTHOR_RESULT\t{}\t29\t{}\t{}\t{counts:?}",
            case.label,
            case.original.len(),
            digest(&case.original)
        );
        if let Some(kind) = case.required {
            assert!(counts[kind] > 0, "required actual transform {}", case.label);
        }
    }
}
