// crates/xdzip-codec-tar/src/names_tests.rs

use super::*;
use std::io::Cursor;
#[path = "../../../tests/support/legacy_names.rs"]
mod fixture;
fn scan(bytes: &[u8]) -> Vec<Entry> {
    TarCodec { gzip: false }
        .scan(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap()
}
fn pax(key: &[u8], value: &[u8]) -> Vec<u8> {
    let mut length = key.len() + value.len() + 4;
    loop {
        let prefix = format!("{length} ");
        let mut record = prefix.into_bytes();
        record.extend(key);
        record.push(b'=');
        record.extend(value);
        record.push(b'\n');
        if record.len() == length {
            return record;
        }
        length = record.len();
    }
}
fn extended(name: &[u8], pax_header: bool, binary: bool) -> Vec<u8> {
    extended_extra(name, pax_header, binary, &[])
}
fn extended_extra(name: &[u8], pax_header: bool, binary: bool, extra: &[u8]) -> Vec<u8> {
    let mut builder = tar::Builder::new(Vec::new());
    let mut data = if pax_header {
        pax(b"path", name)
    } else {
        let mut data = name.to_vec();
        data.push(0);
        data
    };
    if binary {
        data.extend(pax(b"hdrcharset", b"BINARY"));
    }
    data.extend(extra);
    let mut header = tar::Header::new_ustar();
    header.set_entry_type(if pax_header {
        tar::EntryType::XHeader
    } else {
        tar::EntryType::GNULongName
    });
    header.set_path("extended").unwrap();
    header.set_size(data.len() as u64);
    header.set_cksum();
    builder.append(&header, Cursor::new(data)).unwrap();
    let mut header = tar::Header::new_ustar();
    header.set_entry_type(tar::EntryType::Regular);
    header.set_path("fallback.txt").unwrap();
    header.set_size(fixture::CONTENT.len() as u64);
    header.set_cksum();
    builder.append(&header, fixture::CONTENT).unwrap();
    builder.into_inner().unwrap()
}
#[test]
fn tar_and_gnu_names_detect_legacy_while_pax_requires_declared_utf8() {
    for (encoding, sample) in fixture::samples() {
        if matches!(encoding, "CP437" | "UnicodePath") {
            continue;
        }
        let names: Vec<_> = sample.iter().map(|(raw, _)| *raw).collect();
        let bytes = fixture::tar(&names);
        let rows = scan(&bytes);
        for (row, (raw, expected)) in rows.iter().zip(&sample) {
            assert_eq!(row.raw_name, *raw);
            assert_eq!(row.display_name, *expected, "{encoding}");
            assert!(!row.warning);
        }
    }
    let raw = b"\xd6\xd0\xce\xc4.txt";
    for (pax_header, binary, safe) in [
        (false, false, true),
        (true, false, false),
        (true, true, true),
    ] {
        let rows = scan(&extended(raw, pax_header, binary));
        assert_eq!(rows[0].raw_name, raw);
        assert_eq!(!rows[0].warning, safe);
        if safe {
            assert_eq!(rows[0].display_name, "中文.txt");
        } else {
            assert!(rows[0].display_name.starts_with("\\xD6"));
        }
    }
    let rows = scan(&extended("名前.txt".as_bytes(), true, false));
    assert_eq!(rows[0].display_name, "名前.txt");
    assert!(!rows[0].warning);
}

#[test]
fn rewritten_binary_pax_path_declares_utf8_and_preserves_payload() {
    use xdzip_codec_api::encode::*;
    struct Empty;
    impl EncodeInput for Empty {
        fn entries(&self) -> &[EncodeEntry] {
            &[]
        }
        fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            Err(EncodeError::Invalid)
        }
    }
    let original = extended(b"\xd6\xd0\xce\xc4.txt", true, true);
    let mut output = Cursor::new(Vec::new());
    TarCodec { gzip: false }
        .rewrite(
            &mut Cursor::new(original),
            RewriteInput {
                keep: &[true],
                renames: &[Some("中文.txt".into())],
                additions: &Empty,
                settings: EncodeSettings {
                    solid: false,
                    encrypt_headers: false,
                    method: "store",
                    level: 0,
                    password: None,
                },
            },
            &mut output,
            &Cancellation::default(),
            &mut |_| {},
        )
        .unwrap();
    let rows = scan(output.get_ref());
    assert_eq!(rows[0].raw_name, "中文.txt".as_bytes());
    assert_eq!(
        rows[0].name_encoding,
        xdzip_codec_api::names::NameEncoding::Utf8
    );
    assert!(!output.get_ref().windows(10).any(|w| w == b"hdrcharset"));
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend(bytes);
            Ok(())
        }
    }
    let cancel = Cancellation::default();
    let mut source = Cursor::new(output.get_ref());
    let mut reader = TarCodec { gzip: false }
        .open_payload(&mut source, &rows, &cancel)
        .unwrap();
    let mut sink = Sink(vec![]);
    reader.decode(&rows[0], &mut sink, &cancel).unwrap();
    assert_eq!(sink.0, fixture::CONTENT);
    let original = extended_extra(
        b"\xd6\xd0\xce\xc4.txt",
        true,
        true,
        &pax(b"comment", b"\xff"),
    );
    let mut refused = Cursor::new(Vec::new());
    assert_eq!(
        TarCodec { gzip: false }.rewrite(
            &mut Cursor::new(original),
            RewriteInput {
                keep: &[true],
                renames: &[Some("中文.txt".into())],
                additions: &Empty,
                settings: EncodeSettings {
                    solid: false,
                    encrypt_headers: false,
                    method: "store",
                    level: 0,
                    password: None
                }
            },
            &mut refused,
            &cancel,
            &mut |_| {}
        ),
        Err(EncodeError::Unsupported)
    );
}
