use super::*;
use crate::filter_author_tests::{Case, Sink, cases, check_payload, digest};
use xdzip_codec_api::ScanLimits;
#[test]
#[ignore = "explicit independent official-author inputs with original bytes; descriptor counts plus complete transform/output success"]
fn modern_independent_author_known_bytes_and_filter_descriptors() {
    for case in cases(50) {
        check_case(&case);
    }
}
fn check_case(case: &Case) -> [u64; 4] {
    check_payload(case);
    let control = Cancellation::default();
    let mut source = std::io::Cursor::new(&case.archive);
    let mut reader =
        crate::source::Reader::new(&mut source, &control, crate::METADATA_LIMIT).unwrap();
    let index = crate::index::parse(&mut reader, ScanLimits::default(), None).unwrap();
    assert_eq!(index.files.len(), 1);
    let file = &index.files[0];
    assert_eq!(file.parts.len(), 1);
    let part = &index.parts[file.parts[0]];
    let mut description = part.description.clone();
    description.size = file.size;
    let packed = reader
        .bytes(
            part.volume,
            part.data_at,
            usize::try_from(part.packed).unwrap(),
        )
        .unwrap();
    let mut lz = Lz::default();
    let mut output = Sink::default();
    lz.decode(
        &description,
        &mut std::io::Cursor::new(packed),
        &mut output,
        &control,
    )
    .unwrap();
    assert_eq!(output.0, case.original);
    println!(
        "FILTER_AUTHOR_RESULT\t{}\t50\t{}\t{}\t{:?}",
        case.label,
        case.original.len(),
        digest(&case.original),
        lz.filter_counts
    );
    if let Some(kind) = case.required {
        assert!(
            lz.filter_counts[kind] > 0,
            "required descriptor with complete successful output {}",
            case.label
        );
    }
    lz.filter_counts
}

#[test]
fn fixed_official_723_delta_matches_original_recipe_and_executes() {
    let original: Vec<_> = (0..524288usize)
        .map(|i| ((i / 4) * (i % 4 + 1) + (i / 1024)) as u8)
        .collect();
    assert_eq!(
        digest(&original),
        "b8ecd227124f8d48f2b8309969c4ba16a2e9d5eb54b629beed907a8433528f0c"
    );
    let archive = include_bytes!("../tests/fixtures/filter-author/modern-delta4.rar").to_vec();
    assert_eq!(
        digest(&archive),
        "080642174193f72c0d4d983d909f144bb79e871bfaae61452bfdddc901796753"
    );
    let case = Case {
        label: "fixed-official723-modern-delta".into(),
        method: 50,
        archive,
        original,
        required: Some(0),
    };
    assert_eq!(check_case(&case), [8, 0, 0, 0]);
}
