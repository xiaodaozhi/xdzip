use super::*;
use std::io::{self, Cursor, Read};
use xdzip_codec_api::payload::DecodedSink;

struct Sink {
    bytes: Vec<u8>,
    reserved: u64,
    work: u64,
    credit: u64,
    aux_reserved: u64,
    aux: u64,
    limit: u64,
    aux_limit: u64,
    fail_after: Option<usize>,
    ordinary_writes: usize,
}
impl Default for Sink {
    fn default() -> Self {
        Self {
            bytes: Vec::new(),
            reserved: 0,
            work: 0,
            credit: 0,
            aux_reserved: 0,
            aux: 0,
            limit: crate::INPUT_LIMIT,
            aux_limit: crate::INPUT_LIMIT,
            fail_after: None,
            ordinary_writes: 0,
        }
    }
}
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.ordinary_writes += 1;
        self.reserve_work(bytes.len() as u64)?;
        self.produced_work(bytes.len() as u64)?;
        self.write_precharged_chunk(bytes)
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.credit = self
            .credit
            .checked_sub(bytes.len() as u64)
            .ok_or(DecodeError::Output)?;
        if let Some(maximum) = self.fail_after {
            let n = bytes.len().min(maximum.saturating_sub(self.bytes.len()));
            self.bytes.extend_from_slice(&bytes[..n]);
            if n != bytes.len() {
                return Err(DecodeError::Output);
            }
        } else {
            self.bytes.extend_from_slice(bytes);
        }
        Ok(())
    }
}
impl WorkOutput for Sink {
    fn reserve_work(&mut self, bytes: u64) -> Result<(), DecodeError> {
        let next = self.reserved.checked_add(bytes).ok_or(DecodeError::Limit)?;
        if self.work.checked_add(next).is_none_or(|n| n > self.limit) {
            return Err(DecodeError::Limit);
        }
        self.reserved = next;
        Ok(())
    }
    fn produced_work(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.reserved = self.reserved.checked_sub(bytes).ok_or(DecodeError::Limit)?;
        self.work += bytes;
        self.credit += bytes;
        Ok(())
    }
    fn reserve_aux(&mut self, bytes: u64) -> Result<(), DecodeError> {
        let next = self
            .aux_reserved
            .checked_add(bytes)
            .ok_or(DecodeError::Limit)?;
        if self
            .aux
            .checked_add(next)
            .is_none_or(|n| n > self.aux_limit)
        {
            return Err(DecodeError::Limit);
        }
        self.aux_reserved = next;
        Ok(())
    }
    fn produced_aux(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.aux_reserved = self
            .aux_reserved
            .checked_sub(bytes)
            .ok_or(DecodeError::Limit)?;
        self.aux += bytes;
        Ok(())
    }
    fn remaining_aux_work(&self) -> u64 {
        self.aux_limit.saturating_sub(self.aux + self.aux_reserved)
    }
}
fn description(size: u64, solid: bool) -> Descriptor {
    Descriptor {
        method: 20,
        compression: 3,
        dictionary: 65536,
        size: Some(size),
        solid,
        ..Descriptor::default()
    }
}
fn unhex(value: &str) -> Vec<u8> {
    assert!(value.len().is_multiple_of(2));
    value
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect()
}
fn fixed_bits(value: &str) -> Vec<u8> {
    // Only packs fixed declarative mathematical bit strings. It accepts no
    // plaintext or Huffman lengths and does not implement a RAR encoder.
    let bits: Vec<_> = value.bytes().filter(|b| !b.is_ascii_whitespace()).collect();
    assert!(bits.iter().all(|&b| b == b'0' || b == b'1'));
    bits.chunks(8)
        .map(|part| {
            part.iter().fold(0u8, |out, &bit| (out << 1) | (bit - b'0')) << (8 - part.len())
        })
        .collect()
}
fn seeded_normal(codec: &mut Lz20, sink: &mut Sink, control: &Cancellation) {
    codec.history.prepare(65536, false, sink, control).unwrap();
    let mut main = [0u8; 298];
    for symbol in [65, 66, 256, 257, 258, 259, 261, 262] {
        main[symbol] = 3;
    }
    codec.tables = Some(Tables::Normal {
        main: Box::new(Huffman::new(&main).unwrap()),
        distance: Box::new(Huffman::new(&[1]).unwrap()),
        repeat: Box::new(Huffman::new(&[1, 1]).unwrap()),
    });
}

#[test]
fn chronological_recent_matches_retain_duplicates_including_last_repeat() {
    let control = Cancellation::default();
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    seeded_normal(&mut codec, &mut sink, &control);
    // A,B; shortD1; shortD2; chronological secondD1/G0; lastD1; thirdD2/G1.
    let bytes = fixed_bits("000 001 110 00 110 01 100 0 010 101 1");
    codec
        .decode(
            &description(13, true),
            &mut Cursor::new(bytes),
            &mut sink,
            &control,
            0,
        )
        .unwrap();
    assert_eq!(sink.bytes, b"ABBBBBBBBBBBB");
    assert_eq!(codec.recent, [2, 1, 1, 2]);
    assert_eq!((codec.last_distance, codec.last_length), (2, 3));
    assert_eq!(sink.work, 13);
    assert_eq!(sink.credit, 0);
    assert_eq!(sink.ordinary_writes, 0);
}

#[test]
fn slots_keep_legacy_raw_distances_and_maximum_length_bounds() {
    let mut source = Cursor::new([0xff; 16]);
    let mut bits = Bits::new(&mut source);
    assert_eq!(Lz20::distance(47, &mut bits).unwrap(), 1048576);
    assert_eq!(Lz20::distance(34, &mut bits).unwrap(), 196608);
    assert_eq!(Lz20::length(27, &mut bits).unwrap(), 255);
    assert!(Lz20::distance(48, &mut bits).is_err());
    assert!(Lz20::length(28, &mut bits).is_err());
    for slot in 0..4 {
        assert_eq!(
            Lz20::distance(slot, &mut bits).unwrap(),
            u64::from(slot) + 1
        );
    }
}

// Fixed declarative mathematical vectors: no user input or archive compression.
const LITERAL_ABBA: &str = "044000000000000000000000000000000001800000000000000000000000000000000000000000000000000000000000000000000000000006";
const NORMAL_EMPTY: &str = "044000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const NORMAL_RETAIN: &str = "444000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const AUDIO4_RETAIN: &str = "f1100000000000000000800000000000000000000000000000000000000000000000000000000000000040000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000000";
const AUDIO1_RETAIN: &str =
    "c1100000000000000000000000000000000000000000000000000000000000000000000000000000000000";
const SWITCH_NEXT_AUDIO: &str = "81220000000000000000800000000000000000000000000000000000000000000000c00000000000000039822000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0000004000000000000000000481220000000000000000800000000000000000000000000000000000000000000000c00000000000000030";
const SWITCH_AUDIO_FINAL_SAMPLE: &str = "81220000000000000000800000000000000000000000000000000000000000000000c00000000000000039822000000000000000000000000000000000000000000000000000000000000000000000000000000000000c0000004000000000000000000481220000000000000000800000000000000000000000000000000000000000000000c00000000000000030";
const OVERLONG_RUN: &str = "0000000000000000048bffffff80";
const FIRST_REPEAT: &str = "0000000000000000048800";

#[test]
fn fixed_tables_literals_full_baseline_and_actual_aux_work() {
    let control = Cancellation::default();
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    let bytes = unhex(LITERAL_ABBA);
    codec
        .decode(
            &description(4, false),
            &mut Cursor::new(bytes),
            &mut sink,
            &control,
            0,
        )
        .unwrap();
    assert_eq!(sink.bytes, b"ABBA");
    assert_eq!(sink.work, 4);
    assert_eq!(sink.credit, 0);
    assert_eq!(sink.aux, 65536 + 1028 + 19 + 374);
    // The following fixed retained vector starts with a separate sparse old
    // baseline, rather than ABBA's already-complete two-symbol codebook.
    codec.baseline.fill(0);
    codec.baseline[1000] = 7;
    let bytes = unhex(NORMAL_RETAIN);
    codec
        .read_table(&mut Bits::new(&mut Cursor::new(bytes)), &mut sink, &control)
        .unwrap();
    assert_eq!(codec.baseline[1000], 7);
    let bytes = unhex(AUDIO4_RETAIN);
    codec
        .read_table(&mut Bits::new(&mut Cursor::new(bytes)), &mut sink, &control)
        .unwrap();
    assert_eq!(codec.baseline[1000], 7);
    assert!(matches!(codec.tables,Some(Tables::Audio(ref tables)) if tables.len()==4));
    let bytes = unhex(AUDIO1_RETAIN);
    codec
        .read_table(&mut Bits::new(&mut Cursor::new(bytes)), &mut sink, &control)
        .unwrap();
    assert_eq!(codec.baseline[1000], 7);
    let bytes = unhex(NORMAL_EMPTY);
    codec
        .read_table(&mut Bits::new(&mut Cursor::new(bytes)), &mut sink, &control)
        .unwrap();
    assert_eq!(codec.baseline, [0; 1028]);
}

#[test]
fn independent_rar206_encrypted_compressed_ciphertext_matches_all_known_bytes() {
    use crate::legacy_crypto::{LegacyCipher, LegacyKdfBudget, LegacyReader, NarrowPassword};
    // Fixed public Junrar v8.1.0 test data, commit
    // 57091f9ccd43661cf8f12c389917cc24950df707. Original archive
    // rar20-password-compressed.rar has SHA256
    // 9f2791bc468da169f824e29add32220468df33c11cd33995b26014ad3d9425ab.
    // Only its finite original ciphertext is reproduced here. No compressor
    // or third-party decoding implementation was read. Public password and
    // plaintext digest are declared by the upstream extraction test.
    let ciphertext = unhex(
        "e3ff14b7e9b8c0719c6b27b1564fba4112b75766f9020fd979f4e3574ff3ac052ffc1638da4648f1ebee8c40d7cd8bccb77180d911a64a4c5d2103d689f5ba9bccb6d4fb9386d41efc35590ff5563d2c755efec87df6b910d238eb09a2958618",
    );
    let control = Cancellation::default();
    let password = NarrowPassword::ascii("password").unwrap();
    let cipher = LegacyCipher::old20(&password, &control, &mut LegacyKdfBudget::default()).unwrap();
    let mut input = LegacyReader::new(ciphertext.as_slice(), 96, cipher, &control).unwrap();
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    codec
        .decode(&description(87, false), &mut input, &mut sink, &control, 15)
        .unwrap();
    input.finish().unwrap();
    assert_eq!(input.consumed_bytes(), 96);
    assert_eq!(sink.bytes.len(), 87);
    let mut hash = xdzip_crypto::SecretSha256::new();
    hash.update(&sink.bytes);
    let digest: String = hash.finish().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(
        digest,
        "5f512316b5a4d27c6563b83299c5dd7061fc0c4ee6969a4483363b836c2d6951"
    );
    assert_eq!(xdzip_compression::crc32(&sink.bytes), 2524841378);
    assert_eq!(sink.work, 87);
    assert_eq!(sink.credit, 0);
}

#[test]
fn independent_rar20_audio_and_regular_archives_match_published_wav_digest() {
    let audio =
        include_bytes!("../tests/fixtures/legacy20/BoatModernEnglish-audio-text-unpack20.rar");
    let regular =
        include_bytes!("../tests/fixtures/legacy20/BoatModernEnglish-regular-unpack20.rar");
    for (archive, packed, audio_mode) in [(&audio[..], 27902, true), (&regular[..], 42551, false)] {
        let descriptor = Descriptor {
            compression: 5,
            dictionary: 131072,
            ..description(56464, false)
        };
        let control = Cancellation::default();
        let mut codec = Lz20::default();
        let mut sink = Sink::default();
        let mut input = Cursor::new(&archive[73..73 + packed]);
        codec
            .decode(&descriptor, &mut input, &mut sink, &control, 0)
            .unwrap();
        assert_eq!(input.position(), packed as u64);
        assert_eq!(sink.work, 56464);
        assert_eq!(sink.bytes.len(), 56464);
        assert_eq!(sink.credit, 0);
        assert_eq!(xdzip_compression::crc32(&sink.bytes), 2194861449);
        let mut hash = xdzip_crypto::SecretSha256::new();
        hash.update(&sink.bytes);
        let digest: String = hash.finish().iter().map(|b| format!("{b:02x}")).collect();
        assert_eq!(
            digest,
            "0aeeb3c12f01d7089ca83eb321d5a54673ba0e6011231f5bc1fa10102ea3d797"
        );
        assert_eq!(matches!(codec.tables, Some(Tables::Audio(_))), audio_mode);
        if let Some(Tables::Audio(ref trees)) = codec.tables {
            assert_eq!(trees.len(), 2);
        }
    }
}

#[test]
fn meta_length15_is_literal_and_failed_runs_do_not_commit_partial_table() {
    let control = Cancellation::default();
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    // Header00; meta0/meta1 bothlength15; all374 meta-symbol0 words are15bits.
    let mut bytes = vec![0u8; 711];
    bytes[0] = 0x3f;
    bytes[1] = 0xc0;
    codec
        .read_table(&mut Bits::new(&mut Cursor::new(bytes)), &mut sink, &control)
        .unwrap();
    assert_eq!(codec.baseline, [0; 1028]);
    for vector in [OVERLONG_RUN, FIRST_REPEAT] {
        codec.baseline[999] = 7;
        let previous = codec.baseline;
        let bytes = unhex(vector);
        assert_eq!(
            codec.read_table(&mut Bits::new(&mut Cursor::new(bytes)), &mut sink, &control),
            Err(DecodeError::Integrity)
        );
        assert_eq!(codec.baseline, previous);
    }
}

#[test]
fn unaligned_audio_lz_switch_stores_pcm_and_tail_table_continues_next_solid_file() {
    let control = Cancellation::default();
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    let bytes = unhex(SWITCH_NEXT_AUDIO);
    codec
        .decode(
            &description(5, false),
            &mut Cursor::new(bytes),
            &mut sink,
            &control,
            15,
        )
        .unwrap();
    assert_eq!(sink.bytes, b"AAAAA");
    assert_eq!(sink.work, 5);
    assert!(matches!(codec.tables, Some(Tables::Audio(_))));
    sink.bytes.clear();
    codec
        .decode(
            &description(1, true),
            &mut Cursor::new([0]),
            &mut sink,
            &control,
            0,
        )
        .unwrap();
    assert_eq!(sink.bytes, b"A");
    assert_eq!(sink.work, 6);
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    let bytes = unhex(SWITCH_AUDIO_FINAL_SAMPLE);
    codec
        .decode(
            &description(6, false),
            &mut Cursor::new(bytes),
            &mut sink,
            &control,
            0,
        )
        .unwrap();
    assert_eq!(sink.bytes, b"AAAAAA");
}

#[test]
fn truncation_quota_and_partial_output_failures_keep_only_real_work_and_poison() {
    let control = Cancellation::default();
    let bytes = unhex(LITERAL_ABBA);
    for length in 0..bytes.len() {
        let mut codec = Lz20::default();
        let mut sink = Sink::default();
        assert!(
            codec
                .decode(
                    &description(4, false),
                    &mut Cursor::new(&bytes[..length]),
                    &mut sink,
                    &control,
                    0
                )
                .is_err()
        );
        assert_eq!(sink.work, 0);
        assert!(sink.bytes.is_empty());
        assert_eq!(
            codec.decode(
                &description(4, false),
                &mut Cursor::new(&bytes),
                &mut sink,
                &control,
                0
            ),
            Err(DecodeError::Integrity)
        );
    }
    let mut codec = Lz20::default();
    let mut sink = Sink {
        limit: 3,
        ..Sink::default()
    };
    assert_eq!(
        codec.decode(
            &description(4, false),
            &mut Cursor::new(&bytes),
            &mut sink,
            &control,
            0
        ),
        Err(DecodeError::Limit)
    );
    assert_eq!(sink.work, 3);
    assert!(sink.bytes.is_empty());
    let mut codec = Lz20::default();
    let mut sink = Sink {
        fail_after: Some(2),
        ..Sink::default()
    };
    assert_eq!(
        codec.decode(
            &description(4, false),
            &mut Cursor::new(&bytes),
            &mut sink,
            &control,
            0
        ),
        Err(DecodeError::Output)
    );
    assert_eq!(sink.work, 4);
    assert_eq!(sink.bytes, b"AB");
    assert_eq!(sink.credit, 0);
}

#[test]
fn bad_match_size_history_and_missing_last_never_deliver_pending_output() {
    let control = Cancellation::default();
    for (size, pattern, actual) in [(1, "010", 0), (2, "000 110 00", 1), (2, "000 110 01", 1)] {
        let mut codec = Lz20::default();
        let mut sink = Sink::default();
        seeded_normal(&mut codec, &mut sink, &control);
        let result = codec.decode(
            &description(size, true),
            &mut Cursor::new(fixed_bits(pattern)),
            &mut sink,
            &control,
            0,
        );
        assert_eq!(result, Err(DecodeError::Integrity));
        assert_eq!(sink.work, actual);
        assert!(sink.bytes.is_empty());
        assert_eq!(sink.credit, actual);
    }
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    assert_eq!(
        codec.decode(
            &description(1, true),
            &mut Cursor::new([0]),
            &mut sink,
            &control,
            0
        ),
        Err(DecodeError::Unsupported)
    );
    assert_eq!(sink.aux, 0);
}

#[test]
fn typed_errors_during_content_and_tail_survive_and_bare_interrupted_retries() {
    use xdzip_codec_api::ScanError;
    struct Fault {
        bytes: Cursor<Vec<u8>>,
        error: ScanError,
        bare: bool,
    }
    impl Read for Fault {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if self.bare {
                self.bare = false;
                return Err(io::ErrorKind::Interrupted.into());
            }
            if self.bytes.position() < self.bytes.get_ref().len() as u64 {
                self.bytes.read(output)
            } else {
                Err(io::Error::new(io::ErrorKind::Interrupted, self.error))
            }
        }
    }
    let control = Cancellation::default();
    for error in [
        ScanError::SourceChanged,
        ScanError::Cancelled,
        ScanError::Limit,
        ScanError::Io,
    ] {
        for length in [0, 10, unhex(LITERAL_ABBA).len()] {
            let mut input = Fault {
                bytes: Cursor::new(unhex(LITERAL_ABBA)[..length].to_vec()),
                error,
                bare: true,
            };
            let mut codec = Lz20::default();
            let mut sink = Sink::default();
            assert_eq!(
                codec.decode(&description(4, false), &mut input, &mut sink, &control, 0),
                Err(crate::decode_error(error))
            );
            assert_eq!(
                sink.work,
                if length == unhex(LITERAL_ABBA).len() {
                    4
                } else {
                    0
                }
            );
            assert!(sink.bytes.is_empty());
            assert_eq!(
                codec.decode(
                    &description(4, false),
                    &mut Cursor::new(unhex(LITERAL_ABBA)),
                    &mut sink,
                    &control,
                    0
                ),
                Err(DecodeError::Integrity)
            );
        }
    }
}

#[test]
fn cancellation_aux_denial_and_non_solid_reset_preserve_budget_boundaries() {
    let bytes = unhex(LITERAL_ABBA);
    let control = Cancellation::default();
    control.cancel();
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    assert_eq!(
        codec.decode(
            &description(4, false),
            &mut Cursor::new(&bytes),
            &mut sink,
            &control,
            0
        ),
        Err(DecodeError::Cancelled)
    );
    assert_eq!((sink.work, sink.aux, sink.credit), (0, 0, 0));
    for aux_limit in [65535, 65536] {
        let control = Cancellation::default();
        let mut codec = Lz20::default();
        let mut sink = Sink {
            aux_limit,
            ..Sink::default()
        };
        assert_eq!(
            codec.decode(
                &description(4, false),
                &mut Cursor::new(&bytes),
                &mut sink,
                &control,
                0
            ),
            Err(DecodeError::Limit)
        );
        assert_eq!((sink.work, sink.credit), (0, 0));
        assert_eq!(sink.aux, if aux_limit < 65536 { 0 } else { 65536 });
        assert!(codec.tables.is_none());
        assert_eq!(codec.baseline, [0; 1028]);
    }
    let control = Cancellation::default();
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    for method in [20, 26] {
        let mut descriptor = description(4, false);
        descriptor.method = method;
        codec
            .decode(
                &descriptor,
                &mut Cursor::new(&bytes),
                &mut sink,
                &control,
                0,
            )
            .unwrap();
    }
    assert_eq!(sink.bytes, b"ABBAABBA");
    assert_eq!(sink.work, 8);
    assert_eq!(sink.aux, 65536 + 2 * (1028 + 19 + 374));
    codec.reset();
    assert!(codec.tables.is_none());
    assert_eq!(codec.recent, [0; 4]);
    assert!(
        codec
            .memory_required(&Descriptor {
                dictionary: 2 * 1024 * 1024,
                ..description(1, false)
            })
            .is_err()
    );
}

#[test]
fn known_size_tail_is_bounded_drain_without_fabricated_eof_and_bad_table_is_fatal() {
    let control = Cancellation::default();
    // Other tail symbols are observations, not extra output or PCM updates.
    let mut bytes = unhex(LITERAL_ABBA);
    bytes.extend([0u8; 64]);
    let mut source = Cursor::new(bytes);
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    codec
        .decode(&description(4, false), &mut source, &mut sink, &control, 0)
        .unwrap();
    assert_eq!(source.position(), source.get_ref().len() as u64);
    assert_eq!(sink.bytes, b"ABBA");
    let mut codec = Lz20::default();
    let mut sink = Sink::default();
    seeded_normal(&mut codec, &mut sink, &control);
    // A table marker after N must parse fully, and cannot publish pending A.
    let mut main = [0u8; 298];
    main[65] = 1;
    main[269] = 1;
    codec.tables = Some(Tables::Normal {
        main: Box::new(Huffman::new(&main).unwrap()),
        distance: Box::new(Huffman::new(&[1]).unwrap()),
        repeat: Box::new(Huffman::new(&[1]).unwrap()),
    });
    let bytes = fixed_bits("01 00000000 00000000 00000000 00000000 00000000");
    assert_eq!(
        codec.decode(
            &description(1, true),
            &mut Cursor::new(bytes),
            &mut sink,
            &control,
            0
        ),
        Err(DecodeError::Integrity)
    );
    assert_eq!(sink.work, 1);
    assert!(sink.bytes.is_empty());
    assert!(matches!(codec.tables, Some(Tables::Normal { .. })));
}
