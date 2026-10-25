use super::*;
use std::io::Cursor;
use xdzip_codec_api::payload::PayloadCodec;

#[test]
fn independent_author_fixtures_decode_every_admitted_method_and_encryption_profile() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut cases = 0;
    let mut expected = std::collections::BTreeMap::new();
    let mut payload = (0..=255).cycle().take(8192).collect::<Vec<u8>>();
    payload.extend(b"\xe8\x01\0\0\0\xe9\xf0\xff\xff\xff".repeat(128));
    payload.extend(b"\0\0\0\xeb".repeat(128));
    payload.extend(b"end");
    expected.insert(b"payload.bin".to_vec(), payload);
    expected.insert(
        b"hello.txt".to_vec(),
        b"independent 7z preparation\n".to_vec(),
    );
    expected.insert(b"empty.txt".to_vec(), Vec::new());
    for path in std::fs::read_dir(directory).unwrap() {
        let path = path.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "7z") {
            continue;
        }
        let name = path.file_name().unwrap().to_str().unwrap();
        let password = name.contains("encrypted").then_some("fixture-only");
        let control = Cancellation::default();
        let mut source = Cursor::new(std::fs::read(&path).unwrap());
        let index = read_index(&mut source, &control, ScanLimits::default(), password)
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(index.rows.len(), expected.len(), "{name}");
        let mut reader = SevenZipCodec
            .open_payload_with_password(&mut source, &index.rows, &control, password)
            .unwrap();
        let mut actual = std::collections::BTreeMap::new();
        for row in &index.rows {
            let mut sink = Meter::default();
            reader
                .decode(row, &mut sink, &control)
                .unwrap_or_else(|error| panic!("{name}: {error}"));
            assert!(
                actual
                    .insert(row.name_bytes().to_vec(), sink.output)
                    .is_none()
            );
        }
        assert_eq!(actual, expected, "{name}");
        cases += 1;
    }
    assert_eq!(cases, 24);
}
