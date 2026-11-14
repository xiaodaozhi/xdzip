//! Independent author payloads in known physical ZIP record layouts.
use super::*;
use std::io::Cursor;
use xdzip_codec_api::payload::{DecodedSink, PayloadCodec};
const PHRASE: &[u8] = b"abracadabra abracadabra\n";
struct Bytes(Vec<u8>);
impl DecodedSink for Bytes {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), xdzip_codec_api::payload::DecodeError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/streams")
            .join(name),
    )
    .unwrap()
}
#[test]
fn independent_methods_descriptors_and_physical_duplicates_decode_exactly() {
    let control = Cancellation::default();
    for (method, name) in [
        (9, "deflate64"),
        (12, "bzip2"),
        (14, "lzma"),
        (14, "lzma-no-eos"),
        (95, "xz"),
    ] {
        for descriptor in [false, true] {
            if name == "lzma-no-eos" && descriptor {
                continue;
            }
            let name = format!(
                "zipx-{name}{}.zipx",
                if descriptor { "-descriptor" } else { "" }
            );
            let mut source = Cursor::new(fixture(&name));
            let rows = ZipCodec
                .scan(&mut source, &control, ScanLimits::default())
                .unwrap();
            assert_eq!(rows.len(), 1);
            assert_eq!(rows[0].method, method);
            assert!(rows[0].payload_supported());
            let mut reader = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
            let mut bytes = Bytes(Vec::new());
            reader.decode(&rows[0], &mut bytes, &control).unwrap();
            assert_eq!(bytes.0, PHRASE.repeat(6), "{name}");
        }
    }
    let mut source = Cursor::new(fixture("zipx-mixed.zipx"));
    let rows = ZipCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 5);
    let subset = [rows[1].clone(), rows[4].clone()];
    let mut selected = ZipCodec
        .open_selected_payload(&mut source, &subset, &control)
        .unwrap();
    for (entry, expected) in subset
        .iter()
        .zip([PHRASE.repeat(6), b"distinct physical duplicate\n".to_vec()])
    {
        let mut bytes = Bytes(Vec::new());
        selected.decode(entry, &mut bytes, &control).unwrap();
        assert_eq!(bytes.0, expected);
    }
}
#[test]
fn lzma_flag_sizes_properties_and_all_content_checks_are_mandatory() {
    let control = Cancellation::default();
    for name in ["deflate64", "bzip2", "lzma", "xz"] {
        let original = fixture(&format!("zipx-{name}.zipx"));
        let central = original
            .windows(4)
            .position(|bytes| bytes == b"PK\x01\x02")
            .unwrap();
        for kind in 0..4 {
            let mut bytes = original.clone();
            match kind {
                0 => {
                    bytes[14] ^= 1;
                    bytes[central + 16] ^= 1;
                }
                1 => {
                    bytes[22] ^= 1;
                    bytes[central + 24] ^= 1;
                }
                2 => {
                    bytes[37] ^= 255;
                }
                _ => {
                    bytes[8] ^= 1;
                }
            }
            let mut source = Cursor::new(bytes);
            let rows = ZipCodec
                .scan(&mut source, &control, ScanLimits::default())
                .unwrap();
            let mut reader = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
            assert!(
                reader
                    .decode(&rows[0], &mut Bytes(Vec::new()), &control)
                    .is_err(),
                "{name} {kind}"
            );
        }
        if name == "lzma" {
            let mut bytes = original;
            bytes[6] &= !2;
            bytes[central + 8] &= !2;
            let mut source = Cursor::new(bytes);
            let rows = ZipCodec
                .scan(&mut source, &control, ScanLimits::default())
                .unwrap();
            let mut reader = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
            assert!(
                reader
                    .decode(&rows[0], &mut Bytes(Vec::new()), &control)
                    .is_err()
            );
        }
    }
}
#[test]
fn unsupported_ppmd_is_visible_and_never_decoded_as_another_method() {
    let control = Cancellation::default();
    let mut source = Cursor::new(fixture("zipx-ppmd.zipx"));
    let rows = ZipCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!(rows[0].method, 98);
    assert!(!rows[0].payload_supported());
    let mut reader = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
    assert_eq!(
        reader.decode(&rows[0], &mut Bytes(Vec::new()), &control),
        Err(xdzip_codec_api::payload::DecodeError::Unsupported)
    );
}

#[test]
fn dictionary_admission_fails_before_output_and_preserves_limit_classification() {
    let control = Cancellation::default();
    let mut bytes = fixture("zipx-lzma.zipx");
    // Local header 30, hello.txt 9, SDK header 4, properties byte 1.
    bytes[44..48].fill(255);
    let mut source = Cursor::new(bytes);
    let rows = ZipCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut reader = ZipCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut output = Bytes(Vec::new());
    assert_eq!(
        reader.decode(&rows[0], &mut output, &control),
        Err(xdzip_codec_api::payload::DecodeError::Limit)
    );
    assert!(output.0.is_empty());
}
