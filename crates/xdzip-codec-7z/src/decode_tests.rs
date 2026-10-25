use super::*;
use crate::crypto::{AesProperties, AesWriter, KdfBudget};
use std::io::Write;
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
use xdzip_compression::{DeflateWriter, Lzma2Writer};
#[path = "fixture_tests.rs"]
mod fixture_tests;
#[path = "stream_tests.rs"]
mod stream_tests;

#[derive(Clone, Copy)]
struct Checks {
    pack: bool,
    folder: bool,
    streams: bool,
}
const CHECKS: Checks = Checks {
    pack: true,
    folder: true,
    streams: true,
};
fn archive(
    packed: &[u8],
    nodes: &[(&[u8], &[u8], usize)],
    plain: &[u8],
    sizes: &[usize],
    checks: Checks,
) -> Vec<u8> {
    assert_eq!(sizes.iter().sum::<usize>(), plain.len());
    let mut h = vec![1, 4, 6, 0, 1, 9];
    h.extend(number(packed.len() as u64));
    if checks.pack {
        h.extend(digest(&[packed]));
    }
    h.extend([0, 7, 11, 1, 0]);
    h.extend(number(nodes.len() as u64));
    for (method, properties, _) in nodes {
        h.push(method.len() as u8 | if properties.is_empty() { 0 } else { 32 });
        h.extend(*method);
        if !properties.is_empty() {
            h.extend(number(properties.len() as u64));
            h.extend(*properties);
        }
    }
    for node in 1..nodes.len() {
        h.extend(number(node as u64));
        h.extend(number(node as u64 - 1));
    }
    h.push(12);
    for (_, _, size) in nodes {
        h.extend(number(*size as u64));
    }
    if checks.folder {
        h.extend(digest(&[plain]));
    }
    h.extend([0, 8, 13]);
    h.extend(number(sizes.len() as u64));
    if sizes.len() > 1 {
        h.push(9);
        for size in &sizes[..sizes.len() - 1] {
            h.extend(number(*size as u64));
        }
    }
    if checks.streams && (sizes.len() != 1 || !checks.folder) {
        let mut at = 0;
        let values = sizes
            .iter()
            .map(|size| {
                let out = &plain[at..at + size];
                at += size;
                out
            })
            .collect::<Vec<_>>();
        h.extend(digest(&values));
    }
    h.extend([0, 0, 5]);
    h.extend(number(sizes.len() as u64));
    let owned = (0..sizes.len())
        .map(|i| format!("part-{i}"))
        .collect::<Vec<_>>();
    h.extend(names(&owned.iter().map(String::as_str).collect::<Vec<_>>()));
    h.extend([0, 0]);
    image(packed, &h)
}
#[derive(Default)]
struct Meter {
    output: Vec<u8>,
    work: u64,
    credit: u64,
    ordinary: u64,
    precharged: u64,
    fail: bool,
}
impl DecodedSink for Meter {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self.fail {
            return Err(DecodeError::Output);
        }
        self.ordinary += bytes.len() as u64;
        self.output.extend(bytes);
        Ok(())
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.work += bytes;
        self.credit += bytes;
        Ok(())
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if self.fail {
            return Err(DecodeError::Output);
        }
        self.credit = self
            .credit
            .checked_sub(bytes.len() as u64)
            .ok_or(DecodeError::Limit)?;
        self.precharged += bytes.len() as u64;
        self.output.extend(bytes);
        Ok(())
    }
}
fn deflate(plain: &[u8]) -> Vec<u8> {
    let mut writer = DeflateWriter::new(Vec::new(), 6).unwrap();
    writer.write_all(plain).unwrap();
    writer.finish().unwrap()
}
fn lzma2(plain: &[u8]) -> (u8, Vec<u8>) {
    let mut writer = Lzma2Writer::new(Vec::new(), 1, 64 * 1024 * 1024).unwrap();
    let property = writer.property();
    writer.write_all(plain).unwrap();
    (property, writer.finish().unwrap())
}
fn encrypted(plain: &[u8], password: &str) -> Vec<u8> {
    encrypted_with(plain, password, &[63])
}
fn encrypted_with(plain: &[u8], password: &str, properties: &[u8]) -> Vec<u8> {
    let props = AesProperties::parse(properties).unwrap();
    let cancel = Cancellation::default();
    let mut budget = KdfBudget::new(100, 1024 * 1024);
    let mut writer =
        AesWriter::new(Vec::new(), &props, Some(password), &cancel, &mut budget).unwrap();
    writer.write_all(plain).unwrap();
    writer.finish().unwrap()
}

#[test]
fn forward_solid_stream_continues_and_backward_replay_charges_actual_work() {
    let bytes = archive(b"abcdef", &[(&[0], &[], 6)], b"abcdef", &[3, 0, 3], CHECKS);
    struct Observed {
        source: IoCursor<Vec<u8>>,
        reads: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl Read for Observed {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            let n = self.source.read(out)?;
            self.reads.set(self.reads.get() + n);
            Ok(n)
        }
    }
    impl Seek for Observed {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.source.seek(p)
        }
    }
    impl ReadSource for Observed {}
    let rows = scan(&bytes).unwrap().rows;
    let reads = std::rc::Rc::new(std::cell::Cell::new(0));
    let mut source = Observed {
        source: IoCursor::new(bytes),
        reads: reads.clone(),
    };
    let control = Cancellation::default();
    let mut reader = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let before = reads.get();
    let mut meter = Meter::default();
    reader.decode(&rows[0], &mut meter, &control).unwrap();
    assert_eq!(meter.output, b"abc");
    assert_eq!(reads.get() - before, 3);
    assert_eq!(meter.work, 3);
    meter.output.clear();
    reader.decode(&rows[1], &mut meter, &control).unwrap();
    assert!(meter.output.is_empty());
    assert_eq!(reads.get() - before, 3);
    reader.decode(&rows[2], &mut meter, &control).unwrap();
    assert_eq!(meter.output, b"def");
    assert_eq!(reads.get() - before, 6);
    assert_eq!(meter.work, 6);
    meter.output.clear();
    reader.decode(&rows[0], &mut meter, &control).unwrap();
    assert_eq!(meter.output, b"abc");
    assert_eq!(reads.get() - before, 9);
    assert_eq!(meter.work, 9);
    assert_eq!(meter.precharged, 9);
    assert_eq!(meter.ordinary, 0);
}

#[test]
fn independent_crc_can_complete_before_later_corruption_but_missing_crc_drains() {
    let control = Cancellation::default();
    let late = archive(
        b"abcdeg",
        &[(&[0], &[], 6)],
        b"abcdef",
        &[3, 3],
        Checks {
            folder: false,
            ..CHECKS
        },
    );
    let rows = scan(&late).unwrap().rows;
    let mut source = IoCursor::new(late);
    let mut reader = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let mut meter = Meter::default();
    reader.decode(&rows[0], &mut meter, &control).unwrap();
    assert_eq!(meter.output, b"abc");
    assert_eq!(
        reader.decode(&rows[1], &mut meter, &control),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        reader.decode(&rows[0], &mut meter, &control),
        Err(DecodeError::Integrity)
    );
    let late = archive(
        b"abcdeg",
        &[(&[0], &[], 6)],
        b"abcdef",
        &[3, 3],
        Checks {
            streams: false,
            ..CHECKS
        },
    );
    let good = deflate(b"abcdef");
    let bad_end = archive(
        &good[..good.len() - 1],
        &[(&[4, 1, 8], &[], 6)],
        b"abcdef",
        &[3, 3],
        Checks {
            pack: false,
            folder: false,
            streams: false,
        },
    );
    for bytes in [late, bad_end] {
        let rows = scan(&bytes).unwrap().rows;
        let mut source = IoCursor::new(bytes);
        let mut reader = SevenZipCodec
            .open_payload(&mut source, &rows, &control)
            .unwrap();
        let mut meter = Meter::default();
        assert_eq!(
            reader.decode(&rows[0], &mut meter, &control),
            Err(DecodeError::Integrity)
        );
        assert_eq!(
            reader.decode(&rows[1], &mut meter, &control),
            Err(DecodeError::Integrity)
        );
    }
}

#[test]
fn every_node_is_sized_and_finished_but_host_work_counts_only_terminal_output() {
    let plain = b"bounded graph payload\0".repeat(300);
    let (property, packed) = lzma2(&plain);
    let encrypted = encrypted(&packed, "owned-test-password");
    let bytes = archive(
        &encrypted,
        &[
            (&[6, 0xf1, 7, 1], &[63], packed.len()),
            (&[0x21], &[property], plain.len()),
            (&[0], &[], plain.len()),
        ],
        &plain,
        &[7, plain.len() - 7],
        CHECKS,
    );
    let rows = scan(&bytes).unwrap().rows;
    let mut source = IoCursor::new(bytes);
    let control = Cancellation::default();
    let mut reader = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let mut meter = Meter::default();
    reader
        .decode_with_password(&rows[1], &mut meter, &control, Some("owned-test-password"))
        .unwrap();
    assert_eq!(meter.output, &plain[7..]);
    assert_eq!(meter.work, plain.len() as u64);
    meter.output.clear();
    reader
        .decode_with_password(&rows[0], &mut meter, &control, Some("owned-test-password"))
        .unwrap();
    assert_eq!(meter.output, &plain[..7]);
    assert_eq!(meter.work, plain.len() as u64 + 7);
    assert_eq!(
        reader.decode_with_password(&rows[0], &mut meter, &control, Some("different")),
        Err(DecodeError::PasswordOrIntegrity)
    );
}
#[test]
fn no_file_crc_still_requires_whole_folder_crc_and_compression_eof() {
    let plain = b"without per-file crc".repeat(1000);
    let packed = deflate(&plain);
    let bytes = archive(
        &packed,
        &[(&[4, 1, 8], &[], plain.len())],
        &plain,
        &[10, plain.len() - 10],
        Checks {
            streams: false,
            ..CHECKS
        },
    );
    let rows = scan(&bytes).unwrap().rows;
    let mut source = IoCursor::new(bytes);
    let control = Cancellation::default();
    let mut reader = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let mut meter = Meter::default();
    reader.decode(&rows[0], &mut meter, &control).unwrap();
    assert_eq!(meter.output, &plain[..10]);
    assert_eq!(meter.work, plain.len() as u64);
}
struct Input {
    bytes: Vec<u8>,
    reads: usize,
}
impl HeaderSource for Input {
    fn length(&self) -> u64 {
        self.bytes.len() as u64
    }
    fn check(&self) -> Result<(), ScanError> {
        Ok(())
    }
    fn read_exact_at(&mut self, at: u64, out: &mut [u8]) -> Result<(), ScanError> {
        let at = at as usize;
        out.copy_from_slice(
            self.bytes
                .get(at..at + out.len())
                .ok_or(ScanError::Invalid)?,
        );
        self.reads += out.len();
        Ok(())
    }
}
#[test]
fn aggregate_memory_precedes_input_and_actual_reads_consume_task_limits() {
    let bytes = archive(
        b"abcdef",
        &[(&[0], &[], 6), (&[0], &[], 6)],
        b"abcdef",
        &[6],
        CHECKS,
    );
    let index = scan(&bytes).unwrap();
    let control = Cancellation::default();
    for limits in [
        DecodeLimits {
            memory_bytes: 64 * 1024 * 1024 + 11,
            ..DecodeLimits::default()
        },
        DecodeLimits {
            input_bytes: 5,
            ..DecodeLimits::default()
        },
    ] {
        let mut source = Input {
            bytes: bytes.clone(),
            reads: 0,
        };
        let mut decoder = MethodGraphDecoder::new(limits);
        assert_eq!(
            decoder
                .folder(
                    &mut source,
                    &index.streams,
                    0,
                    0,
                    &control,
                    None,
                    &mut |_| Ok(())
                )
                .unwrap_err(),
            DecodeError::Limit
        );
        assert_eq!(source.reads, if limits.input_bytes == 5 { 5 } else { 0 });
    }
    let mut source = Input { bytes, reads: 0 };
    let mut decoder = MethodGraphDecoder::default();
    let mut charged = 0;
    decoder
        .folder(
            &mut source,
            &index.streams,
            0,
            0,
            &control,
            None,
            &mut |n| {
                charged += n;
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(charged, 6);
    assert_eq!(decoder.counters(), (6, 6));
    assert_eq!(decoder.node_work(), 12);
}
#[test]
fn snapshot_identity_output_rejection_cancellation_and_errors_poison_the_session() {
    let bytes = archive(b"abcdef", &[(&[0], &[], 6)], b"abcdef", &[3, 3], CHECKS);
    let mut rows = scan(&bytes).unwrap().rows;
    let control = Cancellation::default();
    rows[0].size += 1;
    assert!(matches!(
        SevenZipCodec.open_payload(&mut IoCursor::new(bytes.clone()), &rows, &control),
        Err(DecodeError::Integrity)
    ));
    rows[0].size -= 1;
    let mut source = IoCursor::new(bytes.clone());
    let mut reader = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let mut meter = Meter {
        fail: true,
        ..Meter::default()
    };
    assert_eq!(
        reader.decode(&rows[0], &mut meter, &control),
        Err(DecodeError::Output)
    );
    meter.fail = false;
    assert_eq!(
        reader.decode(&rows[0], &mut meter, &control),
        Err(DecodeError::Integrity)
    );
    struct Cancel<'a>(&'a Cancellation);
    impl DecodedSink for Cancel<'_> {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            panic!("cancelled before publication")
        }
        fn charge_skipped(&mut self, _: u64) -> Result<(), DecodeError> {
            self.0.cancel();
            Ok(())
        }
    }
    let mut source = IoCursor::new(bytes);
    let cancel = Cancellation::default();
    let mut reader = SevenZipCodec
        .open_payload(&mut source, &rows, &cancel)
        .unwrap();
    assert_eq!(
        reader.decode(&rows[0], &mut Cancel(&cancel), &cancel),
        Err(DecodeError::Cancelled)
    );
    assert_eq!(
        reader.decode(&rows[0], &mut Meter::default(), &control),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn kdf_admission_is_shared_between_nodes_and_never_refunded_after_failure() {
    let plain = [4; 16];
    let first = encrypted(&plain, "test");
    let packed = encrypted_with(&first, "test", &[0]);
    let bytes = archive(
        &packed,
        &[(&[6, 0xf1, 7, 1], &[0], 16), (&[6, 0xf1, 7, 1], &[63], 16)],
        &plain,
        &[16],
        CHECKS,
    );
    let index = scan(&bytes).unwrap();
    let mut source = Input { bytes, reads: 0 };
    let control = Cancellation::default();
    let mut decoder = MethodGraphDecoder::new(DecodeLimits {
        kdf_rounds: 1,
        ..DecodeLimits::default()
    });
    assert_eq!(
        decoder
            .folder(
                &mut source,
                &index.streams,
                0,
                0,
                &control,
                Some("test"),
                &mut |_| Ok(())
            )
            .unwrap_err(),
        DecodeError::Limit
    );
    assert_eq!(decoder.remaining_kdf().0, 0);
    assert_eq!(
        decoder
            .folder(
                &mut source,
                &index.streams,
                0,
                0,
                &control,
                Some("test"),
                &mut |_| Ok(())
            )
            .unwrap_err(),
        DecodeError::Integrity
    );
}

#[test]
fn independent_python_lzma1_node_preserves_eos_and_exact_lengths() {
    // Python stdlib/liblzma encoded this repository-owned phrase; data only.
    let packed = [
        0, 48, 152, 138, 170, 154, 89, 245, 17, 216, 62, 177, 21, 90, 80, 62, 73, 129, 72, 143,
        255, 255, 157, 142, 0, 0,
    ];
    let plain = b"abracadabra abracadabra\n".repeat(6);
    let properties = [0x5d, 0, 0x10, 0, 0];
    let bytes = archive(
        &packed,
        &[(&[3, 1, 1], &properties, plain.len())],
        &plain,
        &[1, plain.len() - 1],
        CHECKS,
    );
    let rows = scan(&bytes).unwrap().rows;
    let mut source = IoCursor::new(bytes);
    let control = Cancellation::default();
    let mut reader = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let mut sink = Meter::default();
    reader.decode(&rows[1], &mut sink, &control).unwrap();
    assert_eq!(sink.output, &plain[1..]);
    assert_eq!(sink.work, plain.len() as u64);
}

#[test]
fn unsupported_methods_special_entries_and_oversized_models_fail_before_work() {
    let control = Cancellation::default();
    for (method, properties, error) in [
        (&[0x7f][..], &[][..], DecodeError::Unsupported),
        (&[4, 1, 8][..], &[1][..], DecodeError::Integrity),
        (
            &[3, 1, 1][..],
            &[0x5d, 255, 255, 255, 255][..],
            DecodeError::Limit,
        ),
        (&[0x21][..], &[40][..], DecodeError::Limit),
        (&[3, 4, 1][..], &[1, 0, 8, 0, 0][..], DecodeError::Integrity),
        (&[3, 4, 1][..], &[6, 0, 0, 0, 128][..], DecodeError::Limit),
    ] {
        let bytes = archive(
            b"abcdef",
            &[(method, properties, 6)],
            b"abcdef",
            &[6],
            CHECKS,
        );
        let rows = scan(&bytes).unwrap().rows;
        let mut source = IoCursor::new(bytes);
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &rows, &control)
            .unwrap();
        let mut sink = Meter::default();
        assert_eq!(payload.decode(&rows[0], &mut sink, &control), Err(error));
        assert_eq!(sink.work, 0);
        assert!(sink.output.is_empty());
    }
    for bytes in [plain(), external_values()] {
        let rows = scan(&bytes).unwrap().rows;
        for row in rows
            .iter()
            .filter(|row| row.directory || row.link || row.warning)
        {
            let mut source = IoCursor::new(bytes.clone());
            let mut payload = SevenZipCodec
                .open_payload(&mut source, &rows, &control)
                .unwrap();
            let mut sink = Meter::default();
            assert_eq!(
                payload.decode(row, &mut sink, &control),
                if row.directory {
                    Ok(())
                } else {
                    Err(DecodeError::Unsupported)
                }
            );
            assert_eq!(sink.work, 0);
            assert!(sink.output.is_empty());
        }
    }
}
fn encoded_metadata(packed: &[u8], decoded: &[u8], method: &[u8], properties: &[u8]) -> Vec<u8> {
    encoded_metadata_after(&[], packed, decoded, method, properties)
}
#[test]
fn ppmd_known_size_nodes_validate_zero_output_range_tail_and_exact_packed_span() {
    let control = Cancellation::default();
    for (packed, plain, valid) in [
        (&[0; 6][..], &[0][..], true),
        (&[0; 5][..], &[][..], true),
        (&[0, 0xff, 0, 0xff, 0, 0][..], &[][..], false),
        (&[0; 7][..], &[0][..], false),
    ] {
        let bytes = archive(
            packed,
            &[(&[3, 4, 1], &[2, 0, 8, 0, 0, 99], plain.len())],
            plain,
            &[plain.len()],
            CHECKS,
        );
        let mut source = IoCursor::new(bytes);
        let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &index.rows, &control)
            .unwrap();
        let mut sink = Meter::default();
        let result = payload.decode(&index.rows[0], &mut sink, &control);
        if valid {
            result.unwrap();
            assert_eq!(sink.output, plain);
        } else {
            assert_eq!(result, Err(DecodeError::Integrity));
            assert_eq!(
                payload.decode(&index.rows[0], &mut sink, &control),
                Err(DecodeError::Integrity)
            );
        }
    }
}
fn encoded_metadata_after(
    prefix: &[u8],
    packed: &[u8],
    decoded: &[u8],
    method: &[u8],
    properties: &[u8],
) -> Vec<u8> {
    let mut header = vec![23, 6];
    header.extend(number(prefix.len() as u64));
    header.extend([1, 9]);
    header.extend(number(packed.len() as u64));
    header.extend(digest(&[packed]));
    header.extend([0, 7, 11, 1, 0, 1]);
    header.push(method.len() as u8 | if properties.is_empty() { 0 } else { 32 });
    header.extend(method);
    if !properties.is_empty() {
        header.extend(number(properties.len() as u64));
        header.extend(properties);
    }
    header.push(12);
    header.extend(number(decoded.len() as u64));
    header.extend(digest(&[decoded]));
    header.extend([0, 0]);
    image(&[prefix, packed].concat(), &header)
}
#[test]
fn metadata_and_payload_nodes_share_nonrefundable_kdf_and_cached_keys() {
    let plain = b"known encrypted payload";
    let packed = encrypted(plain, "same-task");
    let inner = archive(
        &packed,
        &[(&[6, 0xf1, 7, 1], &[63], plain.len())],
        plain,
        &[plain.len()],
        CHECKS,
    );
    let at = 32 + u64::from_le_bytes(inner[12..20].try_into().unwrap()) as usize;
    let header = &inner[at..];
    let encoded = encrypted(header, "same-task");
    let outer = encoded_metadata_after(&packed, &encoded, header, &[6, 0xf1, 7, 1], &[63]);
    let mut decoder = MethodGraphDecoder::new(DecodeLimits {
        kdf_rounds: 1,
        ..DecodeLimits::default()
    });
    let control = Cancellation::default();
    let index = read_index_with_decoder(
        &mut IoCursor::new(outer.clone()),
        &control,
        ScanLimits::default(),
        Some("same-task"),
        &mut decoder,
    )
    .unwrap();
    assert_eq!(index.rows.len(), 1);
    assert_eq!(decoder.remaining_kdf().0, 0);
    let mut source = Input {
        bytes: outer,
        reads: 0,
    };
    assert_eq!(
        decoder
            .folder(
                &mut source,
                &index.streams,
                0,
                0,
                &control,
                Some("same-task"),
                &mut |_| Ok(())
            )
            .unwrap()
            .as_slice(),
        plain
    );
    assert_eq!(decoder.remaining_kdf().0, 0);
    assert_eq!(
        decoder
            .folder(
                &mut source,
                &index.streams,
                0,
                0,
                &control,
                Some("changed-password"),
                &mut |_| Ok(())
            )
            .unwrap_err(),
        DecodeError::Limit
    );
    assert_eq!(decoder.remaining_kdf().0, 0);
}
#[test]
fn many_aes_folders_with_distinct_ivs_share_one_kdf_but_password_changes_do_not() {
    let control = Cancellation::default();
    let plain = b"sixteen-byte-msg";
    let properties = (0..128)
        .map(|n| {
            let mut properties = vec![0x40, 15];
            properties.extend_from_slice(&[n; 16]);
            properties
        })
        .collect::<Vec<_>>();
    let packed = properties
        .iter()
        .map(|properties| encrypted_with(plain, "same-task", properties))
        .collect::<Vec<_>>();
    let mut h = vec![1, 4, 6, 0];
    h.extend(number(packed.len() as u64));
    h.push(9);
    for bytes in &packed {
        h.extend(number(bytes.len() as u64));
    }
    h.extend(digest(
        &packed.iter().map(Vec::as_slice).collect::<Vec<_>>(),
    ));
    h.extend([0, 7, 11]);
    h.extend(number(packed.len() as u64));
    h.push(0);
    for properties in &properties {
        h.extend([1, 0x24, 6, 0xf1, 7, 1]);
        h.extend(number(properties.len() as u64));
        h.extend(properties);
    }
    h.push(12);
    for _ in &packed {
        h.extend(number(plain.len() as u64));
    }
    h.extend(digest(&vec![plain.as_slice(); packed.len()]));
    h.extend([0, 0, 5]);
    h.extend(number(packed.len() as u64));
    let paths = (0..packed.len())
        .map(|n| format!("file-{n}"))
        .collect::<Vec<_>>();
    h.extend(names(&paths.iter().map(String::as_str).collect::<Vec<_>>()));
    h.extend([0, 0]);
    let mut source = IoCursor::new(image(&packed.concat(), &h));
    let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
    assert_eq!(index.streams.folders.len(), 128);
    let mut payload = crate::payload::Payload::open_with_limits(
        &mut source,
        &index.rows,
        &control,
        Some("same-task"),
        DecodeLimits {
            kdf_rounds: 1,
            ..DecodeLimits::default()
        },
    )
    .unwrap();
    let mut sink = Meter::default();
    for row in &index.rows {
        sink.output.clear();
        payload.decode(row, &mut sink, &control).unwrap();
        assert_eq!(sink.output, plain);
    }
    assert_eq!(sink.work, 128 * plain.len() as u64);
    assert_eq!(
        payload.decode_with_password(
            &index.rows[0],
            &mut sink,
            &control,
            Some("changed-password")
        ),
        Err(DecodeError::Limit)
    );
    assert_eq!(
        payload.decode(&index.rows[0], &mut sink, &control),
        Err(DecodeError::Integrity)
    );
}
#[test]
fn compressed_and_encrypted_headers_decode_before_any_index_is_published() {
    let mut decoded = vec![1, 5, 1];
    decoded.extend(property(14, &[0x80]));
    decoded.extend(property(15, &[0x80]));
    decoded.extend(names(&["known-empty"]));
    decoded.extend([0, 0]);
    let compressed = deflate(&decoded);
    let (property, lzma2) = lzma2(&decoded);
    let aes = encrypted(&decoded, "metadata-test");
    for (packed, method, props, password) in [
        (compressed, vec![4, 1, 8], vec![], None),
        (lzma2, vec![0x21], vec![property], None),
        (aes, vec![6, 0xf1, 7, 1], vec![63], Some("metadata-test")),
    ] {
        let bytes = encoded_metadata(&packed, &decoded, &method, &props);
        let mut input = IoCursor::new(bytes.clone());
        let index = read_index(
            &mut input,
            &Cancellation::default(),
            ScanLimits::default(),
            password,
        )
        .unwrap();
        assert_eq!(index.rows.len(), 1);
        assert_eq!(index.rows[0].name_bytes(), b"known-empty");
        assert!(!index.rows[0].directory);
        let mut damaged = bytes;
        damaged[32] ^= 0x20;
        assert!(
            read_index(
                &mut IoCursor::new(damaged),
                &Cancellation::default(),
                ScanLimits::default(),
                password
            )
            .is_err()
        );
    }
}
#[test]
fn encrypted_metadata_distinguishes_ambiguous_password_failure_from_source_and_policy_errors() {
    let mut header = vec![1, 5, 1];
    header.extend(property(14, &[0x80]));
    header.extend(property(15, &[0x80]));
    header.extend(names(&["empty"]));
    header.extend([0, 0]);
    let packed = encrypted(&header, "correct");
    let bytes = encoded_metadata(&packed, &header, &[6, 0xf1, 7, 1], &[63]);
    let control = Cancellation::default();
    assert_eq!(
        read_index(
            &mut IoCursor::new(bytes.clone()),
            &control,
            ScanLimits::default(),
            None
        ),
        Err(ScanError::PasswordRequired)
    );
    assert_eq!(
        read_index(
            &mut IoCursor::new(bytes.clone()),
            &control,
            ScanLimits::default(),
            Some("incorrect")
        ),
        Err(ScanError::PasswordOrIntegrity)
    );
    // Keep the plain descriptor CRC valid so this exercises a damaged CBC
    // stream with the correct password, rather than an outer-header failure.
    let mut damaged = packed.clone();
    damaged[0] ^= 1;
    let damaged = encoded_metadata(&damaged, &header, &[6, 0xf1, 7, 1], &[63]);
    assert_eq!(
        read_index(
            &mut IoCursor::new(damaged),
            &control,
            ScanLimits::default(),
            Some("correct")
        ),
        Err(ScanError::PasswordOrIntegrity)
    );
    struct Failure(ScanError);
    impl HeaderDecoder for Failure {
        fn decode_metadata(
            &mut self,
            _: HeaderPurpose,
            _: &mut dyn HeaderSource,
            _: &Streams,
            _: &mut dyn MetadataSink,
            _: &Cancellation,
            _: Option<&str>,
        ) -> Result<(), ScanError> {
            Err(self.0)
        }
    }
    for error in [
        ScanError::Invalid,
        ScanError::Io,
        ScanError::SourceChanged,
        ScanError::Limit,
        ScanError::Cancelled,
        ScanError::Unsupported,
        ScanError::PasswordRequired,
        ScanError::WrongPassword,
    ] {
        assert_eq!(
            read_index_with_decoder(
                &mut IoCursor::new(bytes.clone()),
                &control,
                ScanLimits::default(),
                Some("correct"),
                &mut Failure(error)
            ),
            Err(if error == ScanError::Invalid {
                ScanError::PasswordOrIntegrity
            } else {
                error
            })
        );
    }
    let mut bad_outer_crc = bytes;
    bad_outer_crc[8] ^= 1;
    assert_eq!(
        read_index(
            &mut IoCursor::new(bad_outer_crc),
            &control,
            ScanLimits::default(),
            Some("correct")
        ),
        Err(ScanError::Invalid)
    );
}

#[test]
fn encrypted_payload_failure_is_ambiguous_but_snapshot_source_and_output_errors_are_not() {
    let plain = b"verified encrypted bytes";
    let packed = encrypted(plain, "correct");
    let bytes = archive(
        &packed,
        &[(&[6, 0xf1, 7, 1], &[63], plain.len())],
        plain,
        &[plain.len()],
        CHECKS,
    );
    let rows = scan(&bytes).unwrap().rows;
    let control = Cancellation::default();
    for (password, failure) in [
        (None, DecodeError::PasswordRequired),
        (Some("incorrect"), DecodeError::PasswordOrIntegrity),
    ] {
        let mut source = IoCursor::new(bytes.clone());
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &rows, &control)
            .unwrap();
        assert_eq!(
            payload.decode_with_password(&rows[0], &mut Meter::default(), &control, password),
            Err(failure)
        );
        assert_eq!(
            payload.decode_with_password(
                &rows[0],
                &mut Meter::default(),
                &control,
                Some("correct")
            ),
            Err(DecodeError::Integrity)
        );
    }
    let mut damaged = packed.clone();
    damaged[0] ^= 1;
    let damaged = archive(
        &damaged,
        &[(&[6, 0xf1, 7, 1], &[63], plain.len())],
        plain,
        &[plain.len()],
        CHECKS,
    );
    let mut source = IoCursor::new(damaged);
    let damaged_rows = read_index(&mut source, &control, ScanLimits::default(), None)
        .unwrap()
        .rows;
    let mut payload = SevenZipCodec
        .open_payload(&mut source, &damaged_rows, &control)
        .unwrap();
    assert_eq!(
        payload.decode_with_password(
            &damaged_rows[0],
            &mut Meter::default(),
            &control,
            Some("correct")
        ),
        Err(DecodeError::PasswordOrIntegrity)
    );
    let mut source = IoCursor::new(bytes.clone());
    let mut payload = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let mut changed = rows[0].clone();
    changed.size += 1;
    assert_eq!(
        payload.decode_with_password(&changed, &mut Meter::default(), &control, Some("correct")),
        Err(DecodeError::Integrity)
    );
    let mut source = IoCursor::new(bytes);
    let mut payload = SevenZipCodec
        .open_payload(&mut source, &rows, &control)
        .unwrap();
    let mut sink = Meter {
        fail: true,
        ..Meter::default()
    };
    assert_eq!(
        payload.decode_with_password(&rows[0], &mut sink, &control, Some("correct")),
        Err(DecodeError::Output)
    );
}
#[test]
fn typed_source_failures_survive_every_node_and_standard_io_retry_boundary() {
    struct Fault {
        source: IoCursor<Vec<u8>>,
        end: u64,
        error: ScanError,
        kind: io::ErrorKind,
        partial: bool,
    }
    impl Read for Fault {
        fn read(&mut self, out: &mut [u8]) -> io::Result<usize> {
            let at = self.source.position();
            if at >= 32 && at < self.end {
                if self.partial {
                    self.partial = false;
                    return self.source.read(&mut out[..1]);
                }
                return Err(io::Error::new(self.kind, self.error));
            }
            self.source.read(out)
        }
    }
    impl Seek for Fault {
        fn seek(&mut self, p: SeekFrom) -> io::Result<u64> {
            self.source.seek(p)
        }
    }
    impl ReadSource for Fault {}
    let plain = b"source failure".repeat(100);
    let deflate = deflate(&plain);
    let (property_value, lzma2) = lzma2(&plain);
    let aes = encrypted(&plain, "test");
    for (packed, method, props, password) in [
        (plain.clone(), vec![0], vec![], None),
        (deflate, vec![4, 1, 8], vec![], None),
        (lzma2, vec![0x21], vec![property_value], None),
        (aes, vec![6, 0xf1, 7, 1], vec![63], Some("test")),
    ] {
        for error in [
            ScanError::Io,
            ScanError::SourceChanged,
            ScanError::Limit,
            ScanError::Cancelled,
        ] {
            for kind in [io::ErrorKind::Other, io::ErrorKind::Interrupted] {
                for partial in [false, true] {
                    let bytes = archive(
                        &packed,
                        &[(&method, &props, plain.len())],
                        &plain,
                        &[plain.len()],
                        CHECKS,
                    );
                    let rows = scan(&bytes).unwrap().rows;
                    let mut source = Fault {
                        source: IoCursor::new(bytes),
                        end: 32 + packed.len() as u64,
                        error,
                        kind,
                        partial,
                    };
                    let control = Cancellation::default();
                    let mut reader = SevenZipCodec
                        .open_payload(&mut source, &rows, &control)
                        .unwrap();
                    let mut sink = Meter::default();
                    assert_eq!(
                        reader.decode_with_password(&rows[0], &mut sink, &control, password),
                        Err(crate::decode::decode_error(error))
                    );
                    assert!(sink.output.is_empty());
                    let mut header = vec![1, 5, 1];
                    header.extend(property(14, &[0x80]));
                    header.extend(property(15, &[0x80]));
                    header.extend(names(&["zero"]));
                    header.extend([0, 0]);
                    let packed = deflate_bytes_for_header(&header);
                    let image = encoded_metadata(&packed, &header, &[4, 1, 8], &[]);
                    let mut source = Fault {
                        source: IoCursor::new(image),
                        end: 32 + packed.len() as u64,
                        error,
                        kind,
                        partial,
                    };
                    assert_eq!(
                        read_index(&mut source, &control, ScanLimits::default(), None),
                        Err(error)
                    );
                }
            }
        }
    }
}
fn deflate_bytes_for_header(bytes: &[u8]) -> Vec<u8> {
    deflate(bytes)
}

fn permuted_bcj2(bad_upstream: bool) -> Vec<u8> {
    let expected = [0xe8, 0, 0, 0, 0];
    let main = deflate(&[0xe8]);
    let mut call = deflate(&[0, 0, 0, 5]);
    if bad_upstream {
        call.push(0);
    }
    let jump = deflate(&[]);
    let rc = [0, 0x7f, 0xff, 0xfc, 0];
    // Serialized pack order supplies global inputs 5,3,6,4. BCJ2 is coder0,
    // with input ports0..3, while Deflate coders1..3 use inputs4..6.
    let packs = [
        call.as_slice(),
        rc.as_slice(),
        jump.as_slice(),
        main.as_slice(),
    ];
    let mut h = vec![1, 4, 6, 0, 4, 9];
    for p in packs {
        h.extend(number(p.len() as u64));
    }
    h.extend(digest(&packs));
    h.extend([0, 7, 11, 1, 0, 4, 0x14, 3, 3, 1, 0x1b, 4, 1]);
    for _ in 0..3 {
        h.extend([3, 4, 1, 8]);
    }
    h.extend([0, 1, 1, 2, 2, 3, 5, 3, 6, 4, 12, 5, 1, 4, 0]);
    h.extend(digest(&[&expected]));
    h.extend([0, 0, 5, 1]);
    h.extend(names(&["graph.bin"]));
    h.extend([0, 0]);
    image(&packs.concat(), &h)
}
#[test]
fn bcj2_four_sources_follow_global_ports_and_serialized_pack_order() {
    let control = Cancellation::default();
    for bad in [false, true] {
        let bytes = permuted_bcj2(bad);
        let index = scan(&bytes).unwrap();
        assert_eq!(
            index.streams.folders[0]
                .packed_inputs
                .iter()
                .map(|p| p.input)
                .collect::<Vec<_>>(),
            [5, 3, 6, 4]
        );
        let mut source = IoCursor::new(bytes);
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &index.rows, &control)
            .unwrap();
        let mut sink = Meter::default();
        let result = payload.decode(&index.rows[0], &mut sink, &control);
        if bad {
            assert_eq!(result, Err(DecodeError::Integrity));
            // Bytes remain staged: the upstream's malformed termination still
            // prevents this whole-folder stream from returning success.
            assert!(sink.work <= 5);
        } else {
            result.unwrap();
            assert_eq!(sink.output, [0xe8, 0, 0, 0, 0]);
            assert_eq!(sink.work, 5);
        }
    }
}

#[test]
#[ignore = "explicit fixed baseline and fixed official reference executable"]
fn independent_baseline_payload_graphs_match_reference_and_known_hashes() {
    use sha2::{Digest, Sha256};
    let folder =
        std::path::PathBuf::from(std::env::var_os("XDZIP_7Z_BASELINE").expect("fixed baseline"));
    let reference = std::env::var_os("XDZIP_7Z_REFERENCE").expect("fixed author");
    let expected = std::collections::BTreeMap::from([
        (
            "empty.txt",
            (
                0,
                "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            ),
        ),
        (
            "hello.txt",
            (
                27,
                "688eabc3477808e4ae377f11b671c7c9a398d59e9a54f233c32deead7983f720",
            ),
        ),
        (
            "payload.bin",
            (
                9987,
                "85b8e769ad1d97a77cd555775c916b49c17df9aba30d5b9658f29330ff373a1b",
            ),
        ),
    ]);
    let mut cases = 0;
    for path in std::fs::read_dir(&folder)
        .unwrap()
        .map(|entry| entry.unwrap().path())
    {
        let name = path.file_stem().unwrap().to_string_lossy();
        if path.extension().is_none_or(|e| e != "7z") || name.contains("encrypted") {
            continue;
        }
        assert!(std::fs::metadata(&path).unwrap().len() <= 8 * 1024 * 1024);
        let bytes = std::fs::read(&path).unwrap();
        let mut source = IoCursor::new(bytes);
        let control = Cancellation::default();
        let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &index.rows, &control)
            .unwrap();
        let mut sink = Meter::default();
        for row in &index.rows {
            sink.output.clear();
            payload
                .decode(row, &mut sink, &control)
                .unwrap_or_else(|error| panic!("{name}: {error}"));
            let filename = std::str::from_utf8(row.name_bytes()).unwrap();
            let (size, hash) = expected.get(filename).unwrap();
            assert_eq!(sink.output.len(), *size);
            assert_eq!(format!("{:x}", Sha256::digest(&sink.output)), *hash);
            let result = std::process::Command::new(&reference)
                .args(["e", "-so", "-bd", "-y"])
                .arg(&path)
                .arg(filename)
                .output()
                .unwrap();
            assert!(result.status.success());
            assert_eq!(sink.output, result.stdout);
        }
        cases += 1;
    }
    assert_eq!(cases, 21);
}

#[test]
#[ignore = "explicit PPMd author raw/properties/known payloads with separately verified profiles"]
fn independent_ppmd_known_size_profiles_complete_and_end_profiles_fail_in_sevenzip_graphs() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_7Z_PPMD_OUTPUT").expect("owned PPMd profiles"),
    );
    let control = Cancellation::default();
    let (mut known, mut end) = (0, 0);
    for entry in std::fs::read_dir(&directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_none_or(|extension| extension != "raw") {
            continue;
        }
        let plain_path = path.with_extension("plain");
        let props_path = path.with_extension("props");
        for path in [&path, &plain_path, &props_path] {
            assert!(std::fs::metadata(path).unwrap().len() <= 8 * 1024 * 1024);
        }
        let packed = std::fs::read(&path).unwrap();
        let plain = std::fs::read(plain_path).unwrap();
        let props = std::fs::read(props_path).unwrap();
        let is_end = std::fs::read_to_string(path.with_extension("profile"))
            .is_ok_and(|profile| profile.trim() == "end");
        let bytes = archive(
            &packed,
            &[(&[3, 4, 1], &props, plain.len())],
            &plain,
            &[plain.len()],
            CHECKS,
        );
        let mut source = IoCursor::new(bytes);
        let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &index.rows, &control)
            .unwrap();
        let mut sink = Meter::default();
        let result = payload.decode(&index.rows[0], &mut sink, &control);
        if is_end {
            assert_eq!(result, Err(DecodeError::Integrity), "{}", path.display());
            end += 1;
        } else {
            result.unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            assert_eq!(sink.output, plain, "{}", path.display());
            assert_eq!(sink.work, plain.len() as u64);
            known += 1;
        }
    }
    assert_eq!((known, end), (36, 3));
}

#[test]
#[ignore = "explicit owned Deflate64 math images independently read by the official tool"]
fn independently_verified_deflate64_vectors_run_through_the_payload_graph() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_7Z_DEFLATE64_OUTPUT").expect("owned Deflate64 vectors"),
    );
    let control = Cancellation::default();
    for case in 0..7 {
        let path = directory.join(format!("{case}.7z"));
        let known = path.with_extension("known");
        assert!(std::fs::metadata(&path).unwrap().len() <= 1024 * 1024);
        assert!(std::fs::metadata(&known).unwrap().len() <= 1024 * 1024);
        let expected = std::fs::read(known).unwrap();
        let mut source = IoCursor::new(std::fs::read(path).unwrap());
        let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
        assert_eq!(index.rows.len(), 1);
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &index.rows, &control)
            .unwrap();
        let mut sink = Meter::default();
        payload.decode(&index.rows[0], &mut sink, &control).unwrap();
        assert_eq!(sink.output, expected);
        assert_eq!(sink.work, expected.len() as u64);
    }
}

#[test]
#[ignore = "explicit self-owned independently verified filter input directory"]
fn independently_verified_filter_vectors_run_through_metadata_and_payload() {
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_7Z_FILTER_OUTPUT").expect("owned vectors"),
    );
    let mut count = 0;
    let control = Cancellation::default();
    for path in std::fs::read_dir(&directory)
        .unwrap()
        .map(|e| e.unwrap().path())
    {
        if path.extension().is_none_or(|e| e != "7z") {
            continue;
        }
        let known = path.with_extension("known");
        assert!(std::fs::metadata(&path).unwrap().len() <= 1024 * 1024);
        assert!(std::fs::metadata(&known).unwrap().len() <= 1024 * 1024);
        let bytes = std::fs::read(path).unwrap();
        let expected = std::fs::read(known).unwrap();
        let mut source = IoCursor::new(bytes);
        let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
        assert_eq!(index.rows.len(), 1);
        let mut payload = SevenZipCodec
            .open_payload(&mut source, &index.rows, &control)
            .unwrap();
        let mut sink = Meter::default();
        payload.decode(&index.rows[0], &mut sink, &control).unwrap();
        assert_eq!(sink.output, expected);
        assert_eq!(sink.work, expected.len() as u64);
        count += 1;
    }
    assert_eq!(count, 20);
}

#[test]
#[ignore = "explicit official author, only newly generated repository-owned inputs"]
fn official_author_graphs_match_all_known_bytes_through_the_payload_trait() {
    let author = std::env::var_os("XDZIP_7Z_REFERENCE").expect("explicit fixed author");
    let parent =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../build/graph-official");
    std::fs::create_dir_all(&parent).unwrap();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = parent.join(format!("{}-{unique}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    std::fs::create_dir(root.join("nested")).unwrap();
    let known = std::collections::BTreeMap::from([
        (
            "first.bin",
            (0..16391)
                .map(|i| ((i * 17 + i / 251) & 255) as u8)
                .collect::<Vec<_>>(),
        ),
        (
            "nested/second.bin",
            (0..98321)
                .map(|i| b"solid-metadata-known\0"[i % 21])
                .collect::<Vec<_>>(),
        ),
        ("zero.bin", Vec::new()),
    ]);
    for (name, bytes) in &known {
        std::fs::write(root.join(name), bytes).unwrap();
    }
    let mut cases = vec![
        ("copy", "Copy", false, false),
        ("deflate", "Deflate", false, false),
        ("bzip2", "BZip2", false, false),
        ("lzma", "LZMA:d=1m", false, false),
        ("lzma2", "LZMA2:d=1m", false, false),
        ("compressed-header", "LZMA2:d=1m", true, false),
        ("copy-aes", "Copy", false, true),
        ("lzma2-aes", "LZMA2:d=1m", false, true),
        ("encrypted-header", "LZMA2:d=1m", true, true),
    ];
    let mut report = String::from("case\tarchive_sha256\tknown_files\tinput_source\tpassed\n");
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!("{:x}", Sha256::digest(std::fs::read(&author).unwrap())),
        "7ddf0a3bc451e166679296be630d6f0573eb386159c04184761c43020c484291"
    );
    let mut payloads = String::from("name\tbytes\tsha256\n");
    for (name, bytes) in &known {
        payloads.push_str(&format!(
            "{name}\t{}\t{:x}\n",
            bytes.len(),
            Sha256::digest(bytes)
        ));
    }
    std::fs::write(root.join("known-payloads.tsv"), payloads).unwrap();
    for (case, method, header, encryption) in cases.drain(..) {
        let path = root.join(format!("{case}.7z"));
        let mut command = std::process::Command::new(&author);
        command
            .current_dir(&root)
            .args(["a", "-t7z", "-bd", "-y", "-mmt=off", "-ms=on"]);
        command.arg(format!("-m0={method}"));
        command.arg(if header { "-mhc=on" } else { "-mhc=off" });
        // Synthetic fixture secret, never a user credential or a report field.
        let password = encryption.then_some("owned-graph-fixture-password");
        if let Some(password) = password {
            command.arg(format!("-p{password}"));
            command.arg(if header { "-mhe=on" } else { "-mhe=off" });
        }
        let output = command
            .arg(&path)
            .args(["first.bin", "nested/second.bin", "zero.bin"])
            .output()
            .unwrap();
        assert!(output.status.success(), "official author: {case}");
        std::fs::write(
            root.join(format!("{case}.author.log")),
            [output.stdout, output.stderr].concat(),
        )
        .unwrap();
        let bytes = std::fs::read(&path).unwrap();
        let digest = format!("{:x}", Sha256::digest(&bytes));
        let mut source = IoCursor::new(bytes);
        let control = Cancellation::default();
        let index = read_index(&mut source, &control, ScanLimits::default(), password)
            .unwrap_or_else(|error| panic!("scan {case}: {error}"));
        assert_eq!(index.rows.len(), 3);
        let mut payload = SevenZipCodec
            .open_payload_with_password(&mut source, &index.rows, &control, password)
            .unwrap();
        let mut sink = Meter::default();
        for row in &index.rows {
            sink.output.clear();
            payload
                .decode_with_password(row, &mut sink, &control, password)
                .unwrap_or_else(|error| panic!("payload {case}: {error}"));
            let name = std::str::from_utf8(row.name_bytes()).unwrap();
            assert_eq!(&sink.output, known.get(name).unwrap(), "{case}/{name}");
        }
        assert_eq!(
            sink.work,
            known.values().map(|v| v.len() as u64).sum::<u64>()
        );
        report.push_str(&format!("{case}\t{digest}\t3\tofficial_author\ttrue\n"));
    }
    let path = root.join("permuted-bcj2.7z");
    let bytes = permuted_bcj2(false);
    std::fs::write(&path, &bytes).unwrap();
    let tested = std::process::Command::new(&author)
        .args(["t", "-bd", "-y"])
        .arg(&path)
        .output()
        .unwrap();
    assert!(tested.status.success());
    let decoded = std::process::Command::new(&author)
        .args(["e", "-so", "-bd", "-y"])
        .arg(&path)
        .arg("graph.bin")
        .output()
        .unwrap();
    assert!(decoded.status.success());
    assert_eq!(decoded.stdout, [0xe8, 0, 0, 0, 0]);
    std::fs::write(
        root.join("permuted-bcj2.reference.log"),
        [tested.stdout, tested.stderr].concat(),
    )
    .unwrap();
    report.push_str(&format!(
        "permuted-bcj2\t{:x}\t1\towned_wire_official_readback\ttrue\n",
        Sha256::digest(bytes)
    ));
    std::fs::write(
        parent.join("last-run.txt"),
        root.file_name().unwrap().to_string_lossy().as_bytes(),
    )
    .unwrap();
    std::fs::write(root.join("verification.tsv"), report).unwrap();
}
