use super::*;
use std::io::{Cursor, Read, Seek, SeekFrom};
use xdzip_codec_api::{
    Cancellation, Format, MetadataCodec, ReadSource, ScanError, ScanLimits,
    payload::{DecodeError, DecodedSink, PayloadCodec},
};
fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/uu")
            .join(name),
    )
    .unwrap()
}
fn payload() -> Vec<u8> {
    let mut bytes = b"Owned UU/XXE payload.\n".repeat(17);
    bytes.extend(0..=255);
    bytes
}
#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend(bytes);
        Ok(())
    }
}
fn inspect(bytes: &[u8]) -> Result<crate::text::Index, ScanError> {
    crate::text::inspect(
        &mut Cursor::new(bytes),
        &Cancellation::default(),
        ScanLimits::default(),
    )
}
#[test]
fn independent_owned_encoders_match_every_exported_byte_and_actual_alphabet() {
    let cancel = Cancellation::default();
    for (name, expected) in [
        ("ordinary.uue", payload()),
        ("ordinary.xxe", payload()),
        ("spaces.uu", payload()),
        ("base64.uue", payload()),
        ("empty.uue", vec![]),
        ("empty.xxe", vec![]),
        ("tails.uue", (0..46).collect()),
        ("tails.xxe", (0..46).collect()),
    ] {
        let mut source = Cursor::new(fixture(name));
        assert!(candidate(&mut source, &cancel).unwrap());
        let format = UuCodec::detect(&mut source, &cancel).unwrap();
        assert_eq!(
            format,
            if name.ends_with("xxe") {
                Format::Xxe
            } else {
                Format::Uu
            }
        );
        let codec = UuCodec { format };
        let rows = codec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].raw_name,
            if name == "base64.uue" {
                b"data.bin".as_slice()
            } else {
                b"hello.bin".as_slice()
            }
        );
        assert_eq!(rows[0].unix_mode, 0);
        assert_eq!(rows[0].size, expected.len() as u64);
        let mut reader = codec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink::default();
        reader.decode(&rows[0], &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, expected, "{name}");
    }
}
#[test]
fn mixed_members_keep_duplicate_physical_names_and_separate_payloads() {
    let cancel = Cancellation::default();
    let mut source = Cursor::new(fixture("multi.uue"));
    let codec = UuCodec { format: Format::Uu };
    let rows = codec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].raw_name, rows[1].raw_name);
    assert_ne!(rows[0].id, rows[1].id);
    assert_eq!(
        rows.iter().map(|row| row.method).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let mut reader = codec.open_payload(&mut source, &rows, &cancel).unwrap();
    for (at, expected) in [(2, payload()), (0, payload()), (1, (0..46).collect())] {
        let mut sink = Sink::default();
        reader.decode(&rows[at], &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, expected);
    }
}
#[test]
fn strict_line_lengths_alphabets_modes_and_trailers_reject_repairs() {
    for bytes in [
        b"begin 644 a\n!00\n`\nend\n".as_slice(),
        b"begin 644 a\n!00000\n`\nend\n",
        b"begin 644 a\n!0~00\n`\nend\n",
        b"begin 644 a\n!0000\nend\n",
        b"begin 644 a\n`\nended\n",
        b"begin 888 a\n`\nend\n",
        b"begin 644 \n`\nend\n",
        b"begin 644 a\n+\nEND\n",
        b"begin-base64 644 a\nTQ==\nTQ==\n====\n",
        b"begin-base64 644 a\nTR==\n====\n",
        b"begin-base64 644 a\nTQ==\nend\n",
    ] {
        assert_eq!(inspect(bytes).err(), Some(ScanError::Invalid), "{bytes:?}");
    }
    assert_eq!(
        inspect(b"begin-encoded 644 name\n`\nend\n").err(),
        Some(ScanError::Unsupported)
    );
    assert_eq!(
        inspect(b"no envelope\n").err(),
        Some(ScanError::Unsupported)
    );
}
#[test]
fn counted_widths_zero_aliases_and_historical_unused_padding_are_preserved() {
    use crate::line::{Encoding, counted, identify};
    for encoding in [Encoding::Uu, Encoding::Xx] {
        let alphabet: &[u8] = if encoding == Encoding::Uu {
            b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_"
        } else {
            b"+-0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        };
        assert_eq!(alphabet.len(), 64);
        for size in 0usize..64 {
            let mut line = vec![alphabet[size]];
            line.extend(vec![alphabet[63]; size.div_ceil(3) * 4]);
            assert_eq!(identify(&line), Ok(encoding));
            assert_eq!(counted(&line, encoding).unwrap(), vec![255; size]);
        }
    }
    assert!(counted(b"`", Encoding::Uu).unwrap().is_empty());
    assert_eq!(crate::line::base64(b"TWE="), Ok((b"Ma".to_vec(), true)));
    assert_eq!(crate::line::base64(b"TWFu"), Ok((b"Man".to_vec(), false)));
    assert!(inspect(b"begin-base64 0 empty\n====").is_ok());
}
#[test]
fn metadata_limits_are_exact_and_cover_transport_and_names() {
    let cancel = Cancellation::default();
    let bytes = fixture("multi.uue");
    let mut limits = ScanLimits {
        read_bytes: bytes.len() as u64,
        ..ScanLimits::default()
    };
    assert_eq!(
        crate::text::inspect(&mut Cursor::new(&bytes), &cancel, limits)
            .unwrap()
            .work,
        bytes.len() as u64
    );
    limits.read_bytes -= 1;
    assert_eq!(
        crate::text::inspect(&mut Cursor::new(&bytes), &cancel, limits).err(),
        Some(ScanError::Limit)
    );
    for limits in [
        ScanLimits {
            entries: 2,
            ..ScanLimits::default()
        },
        ScanLimits {
            name_bytes: 8,
            ..ScanLimits::default()
        },
        ScanLimits {
            total_name_bytes: 17,
            ..ScanLimits::default()
        },
        ScanLimits {
            display_name_bytes: 8,
            ..ScanLimits::default()
        },
    ] {
        assert_eq!(
            UuCodec { format: Format::Uu }
                .scan(&mut Cursor::new(&bytes), &cancel, limits)
                .err(),
            Some(ScanError::Limit)
        );
    }
    let mut long = vec![b'x'; 8193];
    long.push(b'\n');
    assert_eq!(inspect(&long).err(), Some(ScanError::Limit));
    let mut prefix = b"p\n".repeat(32768);
    prefix.extend(fixture("empty.uue"));
    assert_eq!(inspect(&prefix).err(), Some(ScanError::Limit));
    assert!(!candidate(&mut Cursor::new(prefix), &cancel).unwrap());
}
#[test]
fn raw_unsafe_and_unicode_names_remain_host_validated_without_permissions() {
    let cancel = Cancellation::default();
    for name in [
        "../outside",
        "/absolute",
        "C:\\outside",
        "資料.bin",
        "a\u{202e}b",
    ] {
        let bytes = format!("begin 7777 {name}\n`\nend\n");
        let rows = UuCodec { format: Format::Uu }
            .scan(
                &mut Cursor::new(bytes.as_bytes()),
                &cancel,
                ScanLimits::default(),
            )
            .unwrap();
        assert_eq!(rows[0].raw_name, name.as_bytes());
        assert_eq!(rows[0].unix_mode, 0);
        assert_eq!(rows[0].warning, name != "資料.bin");
    }
}
#[test]
fn changed_snapshots_and_sink_failures_poison_payload_readers() {
    let cancel = Cancellation::default();
    let codec = UuCodec { format: Format::Uu };
    let mut source = Cursor::new(fixture("ordinary.uue"));
    let rows = codec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut forged = rows.clone();
    forged[0].size += 1;
    assert_eq!(
        codec.open_payload(&mut source, &forged, &cancel).err(),
        Some(DecodeError::Integrity)
    );
    let mut reader = codec.open_payload(&mut source, &rows, &cancel).unwrap();
    assert_eq!(
        reader.decode(&forged[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        reader.decode(&rows[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
    drop(reader);
    struct Failing;
    impl DecodedSink for Failing {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Err(DecodeError::Output)
        }
    }
    let mut reader = codec.open_payload(&mut source, &rows, &cancel).unwrap();
    assert_eq!(
        reader.decode(&rows[0], &mut Failing, &cancel),
        Err(DecodeError::Output)
    );
    assert_eq!(
        reader.decode(&rows[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Output)
    );
}
#[test]
fn short_reads_and_typed_terminal_io_cancellation_are_not_retried() {
    struct Source {
        data: Cursor<Vec<u8>>,
        failure: Option<ScanError>,
    }
    impl Read for Source {
        fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
            if let Some(error) = self.failure {
                return Err(std::io::Error::new(std::io::ErrorKind::Interrupted, error));
            }
            self.data.read(&mut bytes[..1])
        }
    }
    impl Seek for Source {
        fn seek(&mut self, at: SeekFrom) -> std::io::Result<u64> {
            self.data.seek(at)
        }
    }
    impl ReadSource for Source {}
    let cancel = Cancellation::default();
    let mut source = Source {
        data: Cursor::new(fixture("ordinary.xxe")),
        failure: None,
    };
    assert_eq!(UuCodec::detect(&mut source, &cancel), Ok(Format::Xxe));
    for error in [ScanError::Cancelled, ScanError::Limit, ScanError::Io] {
        source.failure = Some(error);
        assert_eq!(UuCodec::detect(&mut source, &cancel), Err(error));
        assert_eq!(candidate(&mut source, &cancel), Err(error));
        assert_eq!(source.data.position(), 0);
    }
    cancel.cancel();
    assert_eq!(inspect_with_cancel(&cancel), Err(ScanError::Cancelled));
}
fn inspect_with_cancel(cancel: &Cancellation) -> Result<(), ScanError> {
    crate::text::inspect(
        &mut Cursor::new(fixture("ordinary.uue")),
        cancel,
        ScanLimits::default(),
    )
    .map(|_| ())
}
