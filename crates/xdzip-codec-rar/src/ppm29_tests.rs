use super::*;
use std::io::Cursor;

const BUDGET: u64 = 128 << 20;
const ZERO: &[u8] = &[0xa1, 0, 0, 0, 0, 0, 0];

#[test]
fn ppm29_header_orders_escape_and_real_memory_admission() {
    let control = Cancellation::default();
    for code in 1..32u8 {
        let mut model = Model::default();
        let mut input = Cursor::new([0xe0 | code, 0, 73, 0, 0, 0, 0, 0]);
        assert_eq!(
            model.init(&mut input, &control, BUDGET, u64::MAX),
            Ok(Some(73))
        );
        assert_eq!(
            input.position(),
            7,
            "four range bytes after explicit escape"
        );
        assert_eq!(model.decode_byte(&mut input, &control, u64::MAX), Ok(0));
        assert_eq!(input.position(), 8);
        let actual = model.allocated_bytes();
        let admitted = ppmd_memory_required(1 << 20).unwrap();
        assert!(actual > 1 << 20 && actual <= admitted);
    }
    for flags in [0, 0x21, 0x80, 0xa0, 0xe0] {
        let mut model = Model::default();
        assert_eq!(
            model.init(&mut &ZERO[..], &control, BUDGET, u64::MAX),
            Ok(None)
        );
        model.clear();
        assert_eq!(
            model.init(&mut &[flags, 0, 0, 0, 0, 0][..], &control, BUDGET, u64::MAX),
            Err(DecodeError::Integrity)
        );
    }
    let required = ppmd_memory_required(1 << 20).unwrap();
    let mut model = Model::default();
    assert_eq!(
        model.init(&mut &ZERO[..], &control, required - 1, u64::MAX),
        Err(DecodeError::Limit)
    );
    assert_eq!(
        model.decode_byte(&mut &ZERO[..], &control, u64::MAX),
        Err(DecodeError::Integrity)
    );
    model.clear();
    assert_eq!(
        model.init(&mut &ZERO[..], &control, required, u64::MAX),
        Ok(None)
    );
    let mut maximum = Cursor::new([0xa1, 255, 0, 0, 0, 0]);
    assert_eq!(
        model.init(&mut maximum, &control, 1 << 20, u64::MAX),
        Err(DecodeError::Limit)
    );
    assert_eq!(
        maximum.position(),
        2,
        "reject before reading or allocating large range"
    );
}

#[test]
fn ppm29_keep_preserves_full_statistics_and_reset_does_not_invent_escape() {
    let control = Cancellation::default();
    let mut model = Model::default();
    let mut first = ZERO;
    assert_eq!(model.init(&mut first, &control, BUDGET, u64::MAX), Ok(None));
    assert_eq!(model.decode_byte(&mut first, &control, u64::MAX), Ok(0));
    // A fresh root would choose byte 1 at this cumulative interval. The
    // retained root chooses its updated byte 0, proving actual statistics stay.
    let mut keep = &[0x80, 0, 0xff, 0, 0xff, 0][..];
    assert_eq!(model.init(&mut keep, &control, BUDGET, u64::MAX), Ok(None));
    assert_eq!(model.decode_byte(&mut keep, &control, u64::MAX), Ok(0));
    let mut reset = &[0xa1, 0, 0, 0xff, 0, 0xff, 0][..];
    assert_eq!(model.init(&mut reset, &control, BUDGET, u64::MAX), Ok(None));
    assert_eq!(model.decode_byte(&mut reset, &control, u64::MAX), Ok(1));
    let mut keep_escape = &[0xdf, 71, 0, 0, 0, 0][..];
    assert_eq!(
        model.init(&mut keep_escape, &control, BUDGET, u64::MAX),
        Ok(Some(71))
    );
    assert!(
        keep_escape.is_empty(),
        "keep does not consume memory/order bytes"
    );
    assert_eq!(
        model.init(&mut &[0x80, 0, 0, 0, 0][..], &control, 0, u64::MAX),
        Err(DecodeError::Limit)
    );
    assert_eq!(
        model.init(&mut &ZERO[..], &control, BUDGET, u64::MAX),
        Err(DecodeError::Integrity)
    );
    model.clear();
    assert_eq!(
        model.init(&mut &ZERO[..], &control, BUDGET, u64::MAX),
        Ok(None)
    );
}

#[test]
fn ppm29_carryless_binary_discards_remainder_and_normalization_boundaries() {
    let control = Cancellation::default();
    let mut input = &[][..];
    let mut range = Range {
        low: 0,
        code: 0x90000000,
        width: u32::MAX,
    };
    let unit = range.width >> 14;
    assert!(
        !Binding {
            range: &mut range,
            input: &mut input,
            work: Work {
                control: &control,
                counter: &mut 0,
                start: 0,
                maximum: u64::MAX
            }
        }
        .binary(8192)
        .unwrap()
    );
    assert_eq!(range.low, unit * 8192);
    assert_eq!(range.width, unit * 8192);
    assert_ne!(
        range.width,
        u32::MAX - range.low,
        "7z remainder rule must differ"
    );
    let mut range = Range {
        low: 1,
        code: 1,
        width: 1,
    };
    let mut input = Cursor::new([0; 8]);
    range.normalize(&mut input, &control).unwrap();
    assert_eq!(
        input.position(),
        3,
        "stable-prefix branch precedes BOT correction"
    );
    assert_eq!(range.width, 1 << 24);
    let mut range = Range {
        low: 0,
        code: 0,
        width: 1 << 24,
    };
    range.normalize(&mut input, &control).unwrap();
    assert_eq!(input.position(), 3, "TOP equality does not normalize");
    let mut range = Range {
        low: 0x00fffffe,
        code: 0x00fffffe,
        width: 3,
    };
    let mut input = Cursor::new([0; 8]);
    range.normalize(&mut input, &control).unwrap();
    assert_eq!(input.position(), 2);
    assert_eq!(range.width, 1 << 17);
    let mut range = Range {
        low: 0,
        code: u32::MAX,
        width: u32::MAX,
    };
    assert!(
        Binding {
            range: &mut range,
            input: &mut &[][..],
            work: Work {
                control: &control,
                counter: &mut 0,
                start: 0,
                maximum: u64::MAX
            }
        }
        .count(257)
        .is_err()
    );
}

// libarchive v3.8.2 public fixture, archive SHA256:
// 7005b3c6e169ae559f0183c5f0640fdd201b329cf61838c1ca45725949fe0e06.
// Source: libarchive/test/test_read_format_rar_compress_best.rar.uu.
// Fixed official UnRAR 7.23 independently produced the exact known 20 bytes.
// Upstream notices: tests/fixtures/native-archives/COPYING.txt.
const TEXT: &[u8] = &[
    0xa7, 0x18, 0x73, 0xef, 0x96, 0xd8, 0x4a, 0xce, 0x9d, 0x50, 0x33, 0x72, 0x6a, 0xf6, 0x2f, 0x95,
    0x24, 0x5d, 0x0f, 0x0f, 0xf4, 0xd7, 0x85, 0xbb, 0x00, 0x00, 0xbf, 0x88, 0x67, 0xf6, 0xa9, 0xff,
    0xd4,
];
#[test]
fn ppm29_independent_text_control_bytes_no_sevenzip_end_check() {
    let control = Cancellation::default();
    let mut model = Model::default();
    let mut input = Cursor::new(TEXT);
    assert_eq!(model.init(&mut input, &control, BUDGET, u64::MAX), Ok(None));
    let mut actual = Vec::new();
    for _ in 0..22 {
        actual.push(model.decode_byte(&mut input, &control, u64::MAX).unwrap());
    }
    assert_eq!(&actual[..20], b"test text document\r\n");
    assert_eq!(
        &actual[20..],
        [2, 2],
        "outer escape + file end remain ordinary model bytes"
    );
    assert_eq!(
        input.position(),
        25,
        "outer termination does not consume arbitrary packed tails"
    );
    assert_ne!(
        model.range.as_ref().unwrap().code,
        0,
        "RAR does not require 7z's terminal code zero"
    );
    // Independent fixed UnRAR 7.23 t/p accepted each of these 31 property
    // mutations of the unchanged author payload with exact known plaintext.
    // Only F and m change; no compressed symbols are authored here. This is
    // header/high-order evidence, not a claim of an observed capacity restart.
    for code in 1..32u8 {
        let mut packed = TEXT.to_vec();
        packed[0] = 0xa0 | code;
        packed[1] = 0;
        let mut input = packed.as_slice();
        let mut model = Model::default();
        model.init(&mut input, &control, BUDGET, u64::MAX).unwrap();
        let actual = (0..22)
            .map(|_| model.decode_byte(&mut input, &control, u64::MAX).unwrap())
            .collect::<Vec<_>>();
        assert_eq!(
            &actual[..20],
            b"test text document\r\n",
            "order code {code}"
        );
        assert_eq!(&actual[20..], [2, 2]);
    }
}

#[test]
fn ppm29_truncation_root_escape_and_cancellation_poison_until_clear() {
    let control = Cancellation::default();
    for prefix in 0..ZERO.len() {
        let mut input = &ZERO[..prefix];
        let mut model = Model::default();
        let failed = model.init(&mut input, &control, BUDGET, u64::MAX).is_err()
            || model.decode_byte(&mut input, &control, u64::MAX).is_err();
        assert!(failed, "prefix {prefix} must not zero-pad");
        assert_eq!(
            model.decode_byte(&mut &ZERO[..], &control, u64::MAX),
            Err(DecodeError::Integrity)
        );
    }
    let mut input = &[0xa1, 0, 0xff, 0, 0xff, 0, 0][..];
    let mut model = Model::default();
    model.init(&mut input, &control, BUDGET, u64::MAX).unwrap();
    assert_eq!(
        model.decode_byte(&mut input, &control, u64::MAX),
        Err(DecodeError::Integrity)
    );
    let cancelled = Cancellation::default();
    cancelled.cancel();
    let mut model = Model::default();
    let mut input = Cursor::new(ZERO);
    assert_eq!(
        model.init(&mut input, &cancelled, BUDGET, u64::MAX),
        Err(DecodeError::Cancelled)
    );
    assert_eq!(input.position(), 0);
    model.clear();
    model.init(&mut input, &control, BUDGET, u64::MAX).unwrap();
    assert_eq!(
        model.decode_byte(&mut input, &cancelled, u64::MAX),
        Err(DecodeError::Cancelled)
    );
    assert_eq!(
        model.decode_byte(&mut input, &control, u64::MAX),
        Err(DecodeError::Integrity)
    );
}

struct Fault {
    bytes: &'static [u8],
    error: Option<io::Error>,
    before: usize,
}
impl Read for Fault {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if self.before == 0
            && let Some(error) = self.error.take()
        {
            return Err(error);
        }
        let count = self.bytes.len().min(output.len());
        output[..count].copy_from_slice(&self.bytes[..count]);
        self.bytes = &self.bytes[count..];
        self.before = self.before.saturating_sub(count);
        Ok(count)
    }
}

#[test]
fn ppm29_initialization_work_is_monotonic_and_failed_range_keeps_actual_cost() {
    let control = Cancellation::default();
    let mut model = Model::default();
    assert_eq!(
        model.init(&mut &ZERO[..], &control, BUDGET, 0),
        Err(DecodeError::Limit)
    );
    assert_eq!(model.initialization_work(), 0);
    model.clear();
    assert_eq!(
        model.init(&mut &[0xa1, 0][..], &control, BUDGET, u64::MAX),
        Err(DecodeError::Integrity)
    );
    let actual = model.initialization_work();
    assert!(actual > 1 << 20);
    model.clear();
    assert_eq!(
        model.initialization_work(),
        actual,
        "clear cannot refund completed CPU work"
    );
    assert_eq!(
        model.init(&mut &ZERO[..], &control, BUDGET, actual - 1),
        Err(DecodeError::Limit)
    );
    assert_eq!(model.initialization_work(), actual);
    model.clear();
    assert_eq!(
        model.init(&mut &ZERO[..], &control, BUDGET, actual),
        Ok(None)
    );
    assert_eq!(model.initialization_work(), actual * 2);
    assert_eq!(
        model.init(&mut &[0x80, 0, 0, 0, 0][..], &control, BUDGET, 0),
        Ok(None)
    );
    assert_eq!(
        model.initialization_work(),
        actual * 2,
        "retained model and new range allocate no arena"
    );
}

#[test]
#[ignore = "explicit public fixture directory, full streaming output digests, no large plaintext"]
fn ppm29_independent_public_archive_streaming_digests() {
    use xdzip_codec_api::{
        ScanLimits,
        payload::{DecodedSink, PayloadCodec},
    };
    use xdzip_crypto::SecretSha256;
    struct Sink {
        hash: SecretSha256,
        bytes: u64,
        maximum: u64,
        work: u64,
    }
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            assert!(bytes.len() <= 65536);
            self.bytes += bytes.len() as u64;
            if self.bytes > self.maximum {
                return Err(DecodeError::Limit);
            }
            self.hash.update(bytes);
            Ok(())
        }
        fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
            self.work += bytes;
            if self.work > 20 * 1024 * 1024 * 1024 {
                return Err(DecodeError::Limit);
            }
            Ok(())
        }
    }
    let directory = std::path::PathBuf::from(
        std::env::var_os("XDZIP_RAR_PPM_FIXTURES").expect("fixture directory"),
    );
    for archive in [
        "test_read_format_rar_compress_best.rar",
        "test_read_format_rar_ppmd_lzss_conversion.rar",
    ] {
        let mut input = std::fs::File::open(directory.join(archive)).unwrap();
        let control = Cancellation::default();
        let index = crate::read_index(&mut input, &control, ScanLimits::default(), None).unwrap();
        let mut payload = crate::RarCodec
            .open_payload(&mut input, index.entries(), &control)
            .unwrap();
        for row in index
            .entries()
            .iter()
            .filter(|row| !row.directory && !row.link)
        {
            let mut sink = Sink {
                hash: SecretSha256::new(),
                bytes: 0,
                maximum: row.size,
                work: 0,
            };
            let result = payload.decode(row, &mut sink, &control);
            let digest: String = sink
                .hash
                .finish()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            eprintln!(
                "rar29_public {archive} {} bytes={} work={} sha256={digest} result={result:?}",
                row.display_name, sink.bytes, sink.work
            );
            result.unwrap();
            let expected = match row.size {
                20 => "5a5f16e01faf8adf92eb4499a2d3e93010c4b41dbb7f698f4a8466d9f58e6dd2",
                20111 => "ee16390e87152d7dec632ae2b37566da6135966e46b5166e6f40a1c827fa61ad",
                241647978 => "a0b8a5130c56577e61620f3f51ae29478a4c5251f1ed575cec73455fc9b485ea",
                other => panic!("unexpected fixture member size {other}"),
            };
            assert_eq!(sink.bytes, row.size);
            assert_eq!(digest, expected);
        }
    }
}
#[test]
fn ppm29_typed_and_bare_interruptions_do_not_disappear_or_restart_model() {
    let control = Cancellation::default();
    for offset in [0, 6] {
        for marker in [
            DecodeError::SourceIo,
            DecodeError::Limit,
            DecodeError::Cancelled,
            DecodeError::Output,
        ] {
            let mut input = Fault {
                bytes: ZERO,
                error: Some(io::Error::new(io::ErrorKind::Interrupted, marker)),
                before: offset,
            };
            let mut model = Model::default();
            let result = model
                .init(&mut input, &control, BUDGET, u64::MAX)
                .and_then(|_| {
                    model
                        .decode_byte(&mut input, &control, u64::MAX)
                        .map(|_| None::<u8>)
                });
            assert_eq!(result, Err(marker));
            assert_eq!(
                model.decode_byte(&mut input, &control, u64::MAX),
                Err(DecodeError::Integrity)
            );
        }
        let mut input = Fault {
            bytes: ZERO,
            error: Some(io::ErrorKind::Interrupted.into()),
            before: offset,
        };
        let mut model = Model::default();
        model.init(&mut input, &control, BUDGET, u64::MAX).unwrap();
        assert_eq!(model.decode_byte(&mut input, &control, u64::MAX), Ok(0));
    }
    for kind in [io::ErrorKind::InvalidData, io::ErrorKind::OutOfMemory] {
        let mut input = Fault {
            bytes: ZERO,
            error: Some(kind.into()),
            before: 0,
        };
        let mut model = Model::default();
        assert_eq!(
            model.init(&mut input, &control, BUDGET, u64::MAX),
            Err(DecodeError::SourceIo)
        );
    }
}
