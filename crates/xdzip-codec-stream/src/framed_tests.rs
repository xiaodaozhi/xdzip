use super::StreamCodec;
use super::controlled::{ControlledStream, Filter};
use std::io::{Cursor, Read};
use xdzip_codec_api::payload::{DecodeError, DecodedSink, PayloadCodec};
use xdzip_codec_api::{Cancellation, ScanError};
use xdzip_codec_api::{Format, MetadataCodec, ScanLimits};

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/fixtures/streams/");
fn sample(name: &str) -> Vec<u8> {
    std::fs::read(format!("{ROOT}{name}")).unwrap()
}
fn decode(bytes: &[u8], filter: Filter) -> Result<Vec<u8>, ScanError> {
    let cancel = Cancellation::default();
    let mut input = Cursor::new(bytes);
    let mut stream = ControlledStream::new(&mut input, &cancel, filter, 2 << 20, 2 << 20)?;
    let mut out = Vec::new();
    stream
        .read_to_end(&mut out)
        .map_err(|_| stream.failure().unwrap_or(ScanError::Invalid))?;
    Ok(out)
}
fn phrase() -> Vec<u8> {
    b"abracadabra abracadabra\n".repeat(6)
}
#[test]
fn standalone_reference_frames_empty_members_and_concatenation_decode_exactly() {
    for (name, filter, expected) in [
        ("phrase.bz2", Filter::Bzip2, phrase()),
        ("empty.bz2", Filter::Bzip2, vec![]),
        ("multiple.bz2", Filter::Bzip2, phrase()),
        ("phrase.lzma", Filter::Lzma, phrase()),
        ("known-size.lzma", Filter::Lzma, phrase()),
        ("empty.lzma", Filter::Lzma, vec![]),
        ("phrase.lz", Filter::Lzip, phrase()),
        ("empty.lz", Filter::Lzip, vec![]),
        ("multiple.lz", Filter::Lzip, phrase().repeat(2)),
    ] {
        assert_eq!(decode(&sample(name), filter).unwrap(), expected, "{name}");
    }
    let expected: Vec<u8> = (0..320000)
        .map(|i| (((i * 73) ^ (i >> 8)) & 255) as u8)
        .collect();
    assert_eq!(
        decode(&sample("multiblock.bz2"), Filter::Bzip2).unwrap(),
        expected
    );
}
#[test]
fn truncation_and_nonmember_trailing_bytes_reject_all_profiles() {
    for (name, filter) in [
        ("phrase.bz2", Filter::Bzip2),
        ("phrase.lzma", Filter::Lzma),
        ("phrase.lz", Filter::Lzip),
    ] {
        let bytes = sample(name);
        for length in [0, 1, 5, bytes.len() - 1] {
            assert!(decode(&bytes[..length], filter).is_err(), "{name}/{length}");
        }
        let mut bytes = bytes;
        bytes.extend_from_slice(b"garbage");
        assert!(decode(&bytes, filter).is_err(), "{name}");
    }
}
#[test]
fn lzip_checks_crc_output_size_member_size_version_and_dictionary_before_success() {
    let valid = sample("phrase.lz");
    for at in [valid.len() - 20, valid.len() - 16, valid.len() - 8] {
        let mut bytes = valid.clone();
        bytes[at] ^= 1;
        assert!(decode(&bytes, Filter::Lzip).is_err(), "offset {at}");
    }
    let mut bytes = valid.clone();
    bytes[4] = 0;
    assert_eq!(decode(&bytes, Filter::Lzip), Err(ScanError::Unsupported));
    bytes = valid.clone();
    bytes[5] = 0;
    assert_eq!(decode(&bytes, Filter::Lzip), Err(ScanError::Invalid));
    let mut bytes = valid;
    bytes[5] = 29;
    assert_eq!(decode(&bytes, Filter::Lzip), Err(ScanError::Limit));
}
#[test]
fn lzma_enforces_declared_size_dictionary_and_single_stream_only() {
    let known = sample("known-size.lzma");
    for size in [143u64, 145] {
        let mut bytes = known.clone();
        bytes[5..13].copy_from_slice(&size.to_le_bytes());
        assert_eq!(decode(&bytes, Filter::Lzma), Err(ScanError::Invalid));
    }
    let mut bytes = known;
    bytes[1..5].copy_from_slice(&(512u32 << 20).to_le_bytes());
    assert_eq!(decode(&bytes, Filter::Lzma), Err(ScanError::Limit));
    let bytes = sample("phrase.lzma").repeat(2);
    assert_eq!(decode(&bytes, Filter::Lzma), Err(ScanError::Invalid));
}

#[test]
fn read_only_stream_metadata_and_payload_use_observed_sizes_and_profiles() {
    struct Sink(Vec<u8>);
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.0.extend_from_slice(bytes);
            Ok(())
        }
    }
    for (name, format, expected) in [
        ("phrase.bz2", Format::Bzip2, phrase()),
        ("phrase.lzma", Format::Lzma, phrase()),
        ("multiple.lz", Format::Lzip, phrase().repeat(2)),
    ] {
        let codec = StreamCodec { format };
        let cancel = Cancellation::default();
        let mut source = Cursor::new(sample(name));
        let rows = codec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].format, format);
        assert_eq!(rows[0].size, expected.len() as u64);
        assert!(rows[0].payload_supported());
        assert!(!format.descriptor().rewrite);
        let mut reader = codec.open_payload(&mut source, &rows, &cancel).unwrap();
        let mut sink = Sink(vec![]);
        reader.decode(&rows[0], &mut sink, &cancel).unwrap();
        assert_eq!(sink.0, expected);
    }
}

#[test]
fn filtered_payload_rechecks_changed_checksum_or_size_after_metadata() {
    struct Sink;
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
            Ok(())
        }
    }
    for (name, format, at) in [
        ("phrase.bz2", Format::Bzip2, 10),
        ("known-size.lzma", Format::Lzma, 5),
        ("phrase.lz", Format::Lzip, sample("phrase.lz").len() - 20),
    ] {
        let codec = StreamCodec { format };
        let cancel = Cancellation::default();
        let mut source = Cursor::new(sample(name));
        let rows = codec
            .scan(&mut source, &cancel, ScanLimits::default())
            .unwrap();
        source.get_mut()[at] ^= 1;
        let mut reader = codec.open_payload(&mut source, &rows, &cancel).unwrap();
        assert_eq!(
            reader.decode(&rows[0], &mut Sink, &cancel),
            Err(DecodeError::Integrity),
            "{name}"
        );
        assert_eq!(
            reader.decode(&rows[0], &mut Sink, &cancel),
            Err(DecodeError::Integrity)
        );
    }
}

#[test]
fn all_filtered_profiles_enforce_actual_output_limit_and_midstream_cancellation() {
    for (name, filter) in [
        ("phrase.bz2", Filter::Bzip2),
        ("phrase.lzma", Filter::Lzma),
        ("phrase.lz", Filter::Lzip),
    ] {
        for (limit, good) in [(144, true), (143, false)] {
            let cancel = Cancellation::default();
            let mut source = Cursor::new(sample(name));
            let mut stream =
                ControlledStream::new(&mut source, &cancel, filter, 1 << 20, limit).unwrap();
            let mut out = vec![];
            assert_eq!(stream.read_to_end(&mut out).is_ok(), good);
            assert!(out.len() as u64 <= limit);
            if !good {
                assert_eq!(stream.failure(), Some(ScanError::Limit));
            }
        }
        let cancel = Cancellation::default();
        let mut source = Cursor::new(sample(name));
        let mut stream =
            ControlledStream::new(&mut source, &cancel, filter, 1 << 20, 1 << 20).unwrap();
        assert_eq!(stream.read(&mut [0]).unwrap(), 1);
        cancel.cancel();
        assert!(stream.read(&mut [0]).is_err());
        assert_eq!(stream.failure(), Some(ScanError::Cancelled));
    }
}
