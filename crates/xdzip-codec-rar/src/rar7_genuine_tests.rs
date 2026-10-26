//! Build-only public original-input witness; no author program or plaintext file.
use super::*;
use std::fmt::Write;
use std::io::{Cursor, Read};
use xdzip_codec_api::payload::DecodedSink;
use xdzip_compression::Crc32;

const SIZE: u64 = 4_563_402_752;
const DICTIONARY: u64 = 6 * 1024 * 1024 * 1024;
const ARCHIVE_BYTES: usize = 183_863;
const ARCHIVE_SHA: &str = "d915d38bd81ae0eeb8a72f0a6b31fb0474a53c5bb85982752e3b12569421416d";
const ORIGINAL_SHA: &str = "3c404d434bf9533da6f4b3f1b813094c81fe71c39e878b9d376e8eb54bb84b2c";
const ORIGINAL_CRC: u32 = 0x8e9b8bf4;

fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}

struct ZeroSink {
    hash: xdzip_crypto::SecretSha256,
    crc: Crc32,
    bytes: u64,
    work: u64,
    auxiliary: u64,
    credit: u64,
    maximum_chunk: usize,
}
impl DecodedSink for ZeroSink {
    fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
        Err(DecodeError::Output)
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if bytes.is_empty() || bytes.len() > 8192 || bytes.iter().any(|byte| *byte != 0) {
            return Err(DecodeError::Integrity);
        }
        let count = bytes.len() as u64;
        self.credit = self.credit.checked_sub(count).ok_or(DecodeError::Output)?;
        self.bytes = self
            .bytes
            .checked_add(count)
            .filter(|total| *total <= SIZE)
            .ok_or(DecodeError::Limit)?;
        self.maximum_chunk = self.maximum_chunk.max(bytes.len());
        self.hash.update(bytes);
        self.crc.update(bytes);
        Ok(())
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        if bytes == 0 || bytes > 8192 {
            return Err(DecodeError::Integrity);
        }
        self.work = self
            .work
            .checked_add(bytes)
            .filter(|total| *total <= SIZE)
            .ok_or(DecodeError::Limit)?;
        if self
            .work
            .checked_add(self.auxiliary)
            .is_none_or(|total| total > INPUT_LIMIT)
        {
            return Err(DecodeError::Limit);
        }
        self.credit = self
            .credit
            .checked_add(bytes)
            .filter(|total| *total <= SIZE)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        if bytes == 0 || bytes > 65536 {
            return Err(DecodeError::Integrity);
        }
        self.auxiliary = self
            .auxiliary
            .checked_add(bytes)
            .filter(|total| *total <= INPUT_LIMIT)
            .ok_or(DecodeError::Limit)?;
        if self
            .work
            .checked_add(self.auxiliary)
            .is_none_or(|total| total > INPUT_LIMIT)
        {
            return Err(DecodeError::Limit);
        }
        Ok(())
    }
}

#[test]
#[ignore = "explicit release-mode genuine algorithm1 4.25GiB full zero/SHA/CRC/work test; build-only fixture"]
#[allow(clippy::assertions_on_constants)]
fn public_official_723_algorithm1_full_stream() {
    // Debug discovery compiles the ignored test; an explicit run must meet the
    // release-mode workload gate.
    assert!(
        !cfg!(debug_assertions),
        "explicit witness requires --release"
    );
    let path = std::env::var_os("XDZIP_RAR7_GENUINE_FIXTURE")
        .expect("prepare the pinned build-only genuine algorithm1 fixture");
    let file = std::fs::File::open(path).unwrap();
    assert_eq!(file.metadata().unwrap().len(), ARCHIVE_BYTES as u64);
    let mut archive = Vec::new();
    file.take(ARCHIVE_BYTES as u64 + 1)
        .read_to_end(&mut archive)
        .unwrap();
    assert_eq!(archive.len(), ARCHIVE_BYTES);
    let mut archive_hash = xdzip_crypto::SecretSha256::new();
    archive_hash.update(&archive);
    assert_eq!(hex(archive_hash.finish().as_ref()), ARCHIVE_SHA);

    let control = Cancellation::default();
    let mut source = Cursor::new(archive.as_slice());
    let snapshot = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut reader = source::Reader::new(&mut source, &control, METADATA_LIMIT).unwrap();
    let index = index::parse(&mut reader, ScanLimits::default(), None).unwrap();
    assert_eq!(index.rows, snapshot);
    assert_eq!(index.files.len(), 1);
    assert_eq!(index.rows.len(), 1);
    let row = index.rows[0].clone();
    assert!(row.size_known && !row.warning && !row.directory && !row.link);
    assert_eq!((row.size, row.packed_size, row.method), (SIZE, 183_701, 70));
    assert_eq!(row.name_bytes(), b"zeros.bin");
    assert_eq!(index.files[0].parts.len(), 1);
    let description = &index.parts[index.files[0].parts[0]].description;
    assert_eq!((description.declared, description.effective), (1, 1));
    assert_eq!((description.method, description.compression), (70, 1));
    assert_eq!(description.dictionary, DICTIONARY);
    assert_eq!(description.size, Some(SIZE));
    assert_eq!(description.crc, Some(ORIGINAL_CRC));
    assert!(!description.solid && !description.encrypted);

    // This is the same scan -> admitted Reader -> Payload constructor path used
    // by open_payload. Keeping its concrete type exposes test-only instance
    // memory statistics without altering the public payload trait or globals.
    reader.set_limit(INPUT_LIMIT);
    let mut payload = payload::Payload::new(reader, index);
    let mut sink = ZeroSink {
        hash: xdzip_crypto::SecretSha256::new(),
        crc: Crc32::new(),
        bytes: 0,
        work: 0,
        auxiliary: 0,
        credit: 0,
        maximum_chunk: 0,
    };
    payload.decode(&row, &mut sink, &control).unwrap();
    let original_sha = hex(sink.hash.finish().as_ref());
    assert_eq!(original_sha, ORIGINAL_SHA);
    assert_eq!(sink.crc.checksum(), ORIGINAL_CRC);
    assert_eq!((sink.bytes, sink.work, sink.credit), (SIZE, SIZE, 0));
    assert!(sink.auxiliary > 0 && sink.work + sink.auxiliary <= INPUT_LIMIT);
    assert!((1..=8192).contains(&sink.maximum_chunk));
    let (logical, allocated, raw) = payload
        .paged_history_stats()
        .expect("the actual decoded stream must retain paged history");
    assert_eq!(logical, DICTIONARY);
    assert!(allocated > 0 && allocated <= 512 * 1024 * 1024);
    assert!(raw <= allocated);
    // History retains capacities, so its allocation counter is the peak. This
    // includes its conservative allowance; it is not a process RSS measurement.
    println!(
        "RAR7_GENUINE_RESULT\t{}\t{}\t{:08x}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\tall_zero\tsize_crc_verified",
        sink.bytes,
        original_sha,
        sink.crc.checksum(),
        sink.work,
        sink.auxiliary,
        sink.maximum_chunk,
        sink.credit,
        logical,
        allocated,
        raw,
        ARCHIVE_SHA
    );
}
