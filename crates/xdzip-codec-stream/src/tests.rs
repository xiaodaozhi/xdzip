use super::*;
use std::io::{Cursor, Seek, SeekFrom, Write};

fn reference(data: &[u8]) -> Vec<u8> {
    let mut writer = flate2::GzBuilder::new()
        .filename("../../untrusted-name")
        .write(Vec::new(), flate2::Compression::default());
    writer.write_all(data).unwrap();
    writer.finish().unwrap()
}
#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}
#[test]
fn independent_members_and_empty_stream_have_observed_sizes() {
    for parts in [
        vec![b"".as_slice()],
        vec![b"hello".as_slice(), b"", b" world"],
    ] {
        let encoded: Vec<_> = parts.iter().flat_map(|part| reference(part)).collect();
        let expected = parts.concat();
        let cancel = Cancellation::default();
        let mut source = Cursor::new(encoded);
        let rows = GzipCodec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows[0].size, expected.len() as u64);
        assert_eq!(rows[0].raw_name, b"content");
        assert!(rows[0].payload_supported());
        let mut reader = GzipCodec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink::default();
        reader.decode(&rows[0], &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, expected);
        assert_eq!(
            reader.decode(&rows[0], &mut sink, &cancel),
            Err(DecodeError::Integrity)
        );
    }
}
#[test]
fn crc_isize_truncation_and_trailing_bytes_never_publish_metadata() {
    let valid = reference(b"validated payload");
    let mut broken = Vec::new();
    for at in [valid.len() - 8, valid.len() - 4] {
        let mut bytes = valid.clone();
        bytes[at] ^= 1;
        broken.push(bytes);
    }
    for length in [0, 1, 9, valid.len() - 1] {
        broken.push(valid[..length].to_vec());
    }
    let mut garbage = valid.clone();
    garbage.extend_from_slice(b"garbage");
    broken.push(garbage);
    let mut member = valid.clone();
    member.extend_from_slice(&valid[..valid.len() - 1]);
    broken.push(member);
    for bytes in broken {
        assert_eq!(
            GzipCodec.scan(
                &mut Cursor::new(bytes),
                &Cancellation::default(),
                ScanLimits::default()
            ),
            Err(ScanError::Invalid)
        );
    }
}
struct Named {
    bytes: Cursor<Vec<u8>>,
    name: &'static str,
}
impl Read for Named {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        self.bytes.read(out)
    }
}
impl Seek for Named {
    fn seek(&mut self, at: SeekFrom) -> std::io::Result<u64> {
        self.bytes.seek(at)
    }
}
impl ReadSource for Named {
    fn source_name_hint(&self) -> Option<&str> {
        Some(self.name)
    }
}
#[test]
fn virtual_names_use_only_safe_host_basename_and_escape_display() {
    for (name, expected) in [
        ("report.TXT.GZ", "report.TXT"),
        (".gz", "content"),
        ("../bad.gz", "content"),
        ("..gz", "content"),
        ("C:bad.gz", "content"),
        ("plain", "content"),
        ("文档.gz", "文档"),
        ("a\u{202e}.gz", "a\u{202e}"),
    ] {
        let rows = GzipCodec
            .scan(
                &mut Named {
                    bytes: Cursor::new(reference(b"x")),
                    name,
                },
                &Cancellation::default(),
                ScanLimits::default(),
            )
            .unwrap();
        assert_eq!(rows[0].name_bytes(), expected.as_bytes());
        assert!(!rows[0].display_name.contains('\u{202e}'));
    }
}
#[test]
fn output_and_input_budgets_distinguish_exact_eof_from_overflow() {
    let cancel = Cancellation::default();
    for (limit, good) in [(3, true), (2, false), (0, false)] {
        let mut source = Cursor::new(reference(b"abc"));
        let mut stream =
            ControlledStream::new(&mut source, &cancel, Filter::Gzip, 1024, limit).unwrap();
        let mut out = Vec::new();
        assert_eq!(stream.read_to_end(&mut out).is_ok(), good);
        assert!(out.len() as u64 <= limit);
        if !good {
            assert_eq!(stream.failure(), Some(ScanError::Limit));
        }
    }
    let mut source = Cursor::new(b"abcdef");
    assert!(matches!(
        ControlledStream::new(&mut source, &cancel, Filter::Identity, 3, 6),
        Err(ScanError::Limit)
    ));
    let mut probe = ControlledStream::probe(&mut source, &cancel, Filter::Identity, 3, 6).unwrap();
    let mut out = Vec::new();
    assert!(probe.read_to_end(&mut out).is_err());
    assert_eq!(out, b"abc");
    assert_eq!(probe.failure(), Some(ScanError::Limit));
}
#[test]
fn cancellation_names_and_record_admission_are_enforced() {
    let cancel = Cancellation::default();
    for limits in [
        ScanLimits {
            entries: 0,
            ..Default::default()
        },
        ScanLimits {
            name_bytes: 6,
            ..Default::default()
        },
        ScanLimits {
            display_name_bytes: 6,
            ..Default::default()
        },
        ScanLimits {
            total_name_bytes: 6,
            ..Default::default()
        },
        ScanLimits {
            read_bytes: 1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            GzipCodec.scan(&mut Cursor::new(reference(b"x")), &cancel, limits),
            Err(ScanError::Limit)
        );
    }
    cancel.cancel();
    assert_eq!(
        GzipCodec.scan(
            &mut Cursor::new(reference(b"x")),
            &cancel,
            ScanLimits::default()
        ),
        Err(ScanError::Cancelled)
    );
}
#[test]
fn payload_rechecks_integrity_snapshot_and_sink_failures() {
    struct Rejected;
    impl DecodedSink for Rejected {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Err(DecodeError::Output)
        }
    }
    let cancel = Cancellation::default();
    let mut source = Cursor::new(reference(b"abc"));
    let rows = GzipCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut reader = GzipCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    assert_eq!(
        reader.decode(&rows[0], &mut Rejected, &cancel),
        Err(DecodeError::Output)
    );
    assert_eq!(
        reader.decode(&rows[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
    drop(reader);
    let len = source.get_ref().len();
    source.get_mut()[len - 8] ^= 1;
    let mut reader = GzipCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    assert_eq!(
        reader.decode(&rows[0], &mut Sink::default(), &cancel),
        Err(DecodeError::Integrity)
    );
    drop(reader);
    let mut rows = rows;
    rows[0].directory = true;
    assert!(matches!(
        GzipCodec.open_payload(&mut source, &rows, &cancel),
        Err(DecodeError::Integrity)
    ));
}
struct Input {
    entries: Vec<EncodeEntry>,
    bytes: Vec<u8>,
}
impl EncodeInput for Input {
    fn entries(&self) -> &[EncodeEntry] {
        &self.entries
    }
    fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
        Ok(Box::new(Cursor::new(&self.bytes)))
    }
}
fn input(bytes: &[u8]) -> Input {
    Input {
        entries: vec![EncodeEntry {
            name: "original.txt".into(),
            directory: false,
            size: bytes.len() as u64,
            modified_unix: 0,
        }],
        bytes: bytes.to_vec(),
    }
}
fn settings(level: u8) -> EncodeSettings<'static> {
    EncodeSettings {
        method: "deflate",
        level,
        solid: false,
        encrypt_headers: false,
        password: None,
    }
}
#[test]
fn encoded_streams_are_readable_by_independent_reference_at_all_levels() {
    for bytes in [vec![], b"independently decoded ".repeat(10000)] {
        for level in [0, 1, 6, 9] {
            let mut encoded = Vec::new();
            let mut work = 0;
            GzipCodec
                .encode(
                    &input(&bytes),
                    &mut encoded,
                    settings(level),
                    &Cancellation::default(),
                    &mut |p| work = p.input_bytes,
                )
                .unwrap();
            let mut decoded = Vec::new();
            flate2::read::MultiGzDecoder::new(encoded.as_slice())
                .read_to_end(&mut decoded)
                .unwrap();
            assert_eq!(decoded, bytes);
            assert_eq!(work, bytes.len() as u64);
        }
    }
}
#[test]
fn multiple_files_directories_and_unsupported_options_fail_before_output() {
    let mut directory = input(b"");
    directory.entries[0].directory = true;
    let mut multiple = input(b"x");
    multiple.entries.push(multiple.entries[0].clone());
    for source in [
        directory,
        multiple,
        Input {
            entries: vec![],
            bytes: vec![],
        },
    ] {
        let mut out = Vec::new();
        assert_eq!(
            GzipCodec.encode(
                &source,
                &mut out,
                settings(6),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Unsupported)
        );
        assert!(out.is_empty());
    }
    for option in [
        EncodeSettings {
            solid: true,
            ..settings(6)
        },
        EncodeSettings {
            password: Some("secret"),
            ..settings(6)
        },
        EncodeSettings {
            method: "store",
            ..settings(6)
        },
        settings(2),
    ] {
        let mut out = Vec::new();
        assert_eq!(
            GzipCodec.encode(
                &input(b"x"),
                &mut out,
                option,
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Unsupported)
        );
        assert!(out.is_empty());
    }
}
#[test]
fn creation_detects_input_growth_shrinkage_and_cancellation() {
    for size in [2, 4] {
        let mut source = input(b"abc");
        source.entries[0].size = size;
        assert_eq!(
            GzipCodec.encode(
                &source,
                &mut Vec::new(),
                settings(6),
                &Cancellation::default(),
                &mut |_| {}
            ),
            Err(EncodeError::Source)
        );
    }
    let cancel = Cancellation::default();
    cancel.cancel();
    let mut out = Vec::new();
    assert_eq!(
        GzipCodec.encode(&input(b"abc"), &mut out, settings(6), &cancel, &mut |_| {}),
        Err(EncodeError::Cancelled)
    );
    assert!(out.is_empty());
}
