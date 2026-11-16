use crate::index::{self, Index};
use std::io::SeekFrom;
use xdzip_codec_api::{
    Cancellation, ReadSource, ScanError, ScanLimits,
    payload::{DecodeError, DecodedSink},
};
use xdzip_crypto::SecretSha256;
const WORK_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
/// Task-private work ceilings; they cannot relax the backend's hard maximums.
#[derive(Clone, Copy)]
pub struct Limits {
    /// Actual source input including metadata and repeated ranges.
    pub input: u64,
    /// Actual delivered file bytes, including repeated records.
    pub decoded: u64,
    /// Actual SHA-256 compression-block bytes, including padding.
    pub hash_work: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            input: WORK_LIMIT,
            decoded: WORK_LIMIT,
            hash_work: WORK_LIMIT,
        }
    }
}
fn decode_error(error: ScanError) -> DecodeError {
    match error {
        ScanError::Cancelled => DecodeError::Cancelled,
        ScanError::Limit => DecodeError::Limit,
        ScanError::Io | ScanError::SourceChanged => DecodeError::SourceIo,
        ScanError::Unsupported => DecodeError::Unsupported,
        _ => DecodeError::Integrity,
    }
}
struct Hash {
    value: SecretSha256,
    bytes: u64,
}
impl Hash {
    fn new() -> Self {
        Self {
            value: SecretSha256::new(),
            bytes: 0,
        }
    }
    fn reserve(work: u64, available: &mut u64) -> Result<(), DecodeError> {
        *available = available.checked_sub(work).ok_or(DecodeError::Limit)?;
        Ok(())
    }
    fn update(
        &mut self,
        bytes: &[u8],
        available: &mut u64,
        sink: &mut dyn DecodedSink,
    ) -> Result<(), DecodeError> {
        let next = self
            .bytes
            .checked_add(bytes.len() as u64)
            .ok_or(DecodeError::Limit)?;
        let work = (next / 64 - self.bytes / 64) * 64;
        Self::reserve(work, available)?;
        self.value.update(bytes);
        self.bytes = next;
        if work != 0 {
            sink.charge_auxiliary(work)?;
        }
        Ok(())
    }
    fn finish(
        self,
        available: &mut u64,
        sink: &mut dyn DecodedSink,
    ) -> Result<[u8; 32], DecodeError> {
        let remainder = self.bytes % 64;
        let work = if remainder <= 55 { 64 } else { 128 };
        Self::reserve(work, available)?;
        let digest = self.value.finish();
        sink.charge_auxiliary(work)?;
        Ok(*digest)
    }
}
/// One completely indexed authorized source. It never opens a path or follows a link.
pub struct Reader<'a> {
    source: &'a mut dyn ReadSource,
    cancel: &'a Cancellation,
    index: Index,
    limits: Limits,
    failure: Option<DecodeError>,
}
impl<'a> Reader<'a> {
    /// Parse metadata once and retain bounded locators for the entire task.
    pub fn open(
        source: &'a mut dyn ReadSource,
        cancel: &'a Cancellation,
    ) -> Result<Self, DecodeError> {
        Self::open_with_limits(source, cancel, Limits::default())
    }
    /// Open with stricter private quotas, preserving ordinary host source authority.
    pub fn open_with_limits(
        source: &'a mut dyn ReadSource,
        cancel: &'a Cancellation,
        mut limits: Limits,
    ) -> Result<Self, DecodeError> {
        limits.input = limits.input.min(WORK_LIMIT);
        limits.decoded = limits.decoded.min(WORK_LIMIT);
        limits.hash_work = limits.hash_work.min(WORK_LIMIT);
        let scan = ScanLimits {
            read_bytes: limits.input.min(ScanLimits::default().read_bytes),
            ..ScanLimits::default()
        };
        let index = index::inspect(source, cancel, scan).map_err(decode_error)?;
        limits.input = limits
            .input
            .checked_sub(index.read_work)
            .ok_or(DecodeError::Limit)?;
        Ok(Self {
            source,
            cancel,
            index,
            limits,
            failure: None,
        })
    }
    /// Immutable metadata for the parent's exact snapshot projection.
    pub fn index(&self) -> &Index {
        &self.index
    }
    /// Decode an admitted serialization-order record. The product adapter additionally
    /// binds its opaque physical identity to the exact immutable Entry metadata.
    pub fn decode_record(
        &mut self,
        at: usize,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        if let Some(error) = self.failure {
            return Err(error);
        }
        let result = self.deliver(at, sink, cancel);
        if let Err(error) = result {
            self.failure = Some(error);
        }
        result
    }
    fn deliver(
        &mut self,
        at: usize,
        sink: &mut dyn DecodedSink,
        cancel: &Cancellation,
    ) -> Result<(), DecodeError> {
        self.cancel.check().map_err(decode_error)?;
        cancel.check().map_err(decode_error)?;
        let record = self.index.records.get(at).ok_or(DecodeError::Integrity)?;
        if record.warning || record.link || record.external {
            return Err(DecodeError::Unsupported);
        }
        if record.directory {
            return Ok(());
        }
        let offset = record.offset.ok_or(DecodeError::Integrity)?;
        if self
            .source
            .seek(SeekFrom::Start(offset))
            .map_err(|error| decode_error(ScanError::from_source_error(&error)))?
            != offset
        {
            return Err(DecodeError::Integrity);
        }
        let mut whole = Hash::new();
        let mut block = Hash::new();
        let mut block_bytes = 0;
        let mut block_at = 0;
        let mut remaining = record.size;
        let mut buffer = [0; 64 * 1024];
        while remaining != 0 {
            self.cancel.check().map_err(decode_error)?;
            cancel.check().map_err(decode_error)?;
            let available = self.limits.input.min(self.limits.decoded);
            if available == 0 {
                return Err(DecodeError::Limit);
            }
            let mut take = remaining.min(available).min(buffer.len() as u64);
            if let Some(check) = &record.integrity {
                take = take.min(check.block_size - block_bytes);
            }
            let n = self
                .source
                .read(&mut buffer[..take as usize])
                .map_err(|error| decode_error(ScanError::from_source_error(&error)))?;
            if n > take as usize {
                return Err(DecodeError::SourceIo);
            }
            if n == 0 {
                return Err(DecodeError::Integrity);
            }
            self.limits.input -= n as u64;
            self.limits.decoded -= n as u64;
            remaining -= n as u64;
            if let Some(check) = &record.integrity {
                whole.update(&buffer[..n], &mut self.limits.hash_work, sink)?;
                block.update(&buffer[..n], &mut self.limits.hash_work, sink)?;
                block_bytes += n as u64;
                if block_bytes == check.block_size {
                    let actual = block.finish(&mut self.limits.hash_work, sink)?;
                    if check.blocks.get(block_at) != Some(&actual) {
                        return Err(DecodeError::Integrity);
                    }
                    block_at += 1;
                    block_bytes = 0;
                    block = Hash::new();
                }
            }
            sink.write_chunk(&buffer[..n])?;
        }
        if let Some(check) = &record.integrity {
            if block_bytes != 0 || record.size == 0 {
                let actual = block.finish(&mut self.limits.hash_work, sink)?;
                if check.blocks.get(block_at) != Some(&actual) {
                    return Err(DecodeError::Integrity);
                }
                block_at += 1;
            }
            if block_at != check.blocks.len()
                || whole.finish(&mut self.limits.hash_work, sink)? != check.hash
            {
                return Err(DecodeError::Integrity);
            }
        }
        self.cancel.check().map_err(decode_error)?;
        cancel.check().map_err(decode_error)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/asar")
                .join(name),
        )
        .unwrap()
    }
    #[derive(Default)]
    struct Sink {
        bytes: Vec<u8>,
        work: u64,
    }
    impl DecodedSink for Sink {
        fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
            self.bytes.extend(bytes);
            Ok(())
        }
        fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
            self.work += bytes;
            Ok(())
        }
    }
    #[test]
    fn independent_stored_files_shared_ranges_and_empty_hashes_decode_exactly() {
        let cancel = Cancellation::default();
        for name in [
            "ordinary.asar",
            "without-integrity.asar",
            "shared.asar",
            "executable.asar",
        ] {
            let mut source = Cursor::new(fixture(name));
            let mut reader = Reader::open(&mut source, &cancel).unwrap();
            for at in 0..reader.index.records.len() {
                let record = reader.index.records[at].clone();
                let mut sink = Sink::default();
                reader.decode_record(at, &mut sink, &cancel).unwrap();
                assert_eq!(sink.bytes.len() as u64, record.size);
                if record.directory || record.size == 0 {
                    assert!(sink.bytes.is_empty());
                } else if record.path.contains("资料") {
                    assert_eq!(sink.bytes, "owned UTF-8 ASAR content\n资料\n".as_bytes());
                } else {
                    assert_eq!(sink.bytes, b"abracadabra abracadabra\n".repeat(6));
                }
                assert_eq!(sink.work != 0, record.integrity.is_some());
            }
        }
    }
    #[test]
    fn real_sha_compression_work_and_repeated_range_quotas_are_exact_and_terminal() {
        let cancel = Cancellation::default();
        let bytes = fixture("shared.asar");
        let index =
            index::inspect(&mut Cursor::new(&bytes), &cancel, ScanLimits::default()).unwrap();
        // Whole SHA: three blocks. Two full 64-byte blocks plus one short block
        // need five block hashes including their padding: 8 * 64 bytes total.
        let exact = Limits {
            input: index.read_work + 144,
            decoded: 144,
            hash_work: 512,
        };
        let mut source = Cursor::new(&bytes);
        let mut reader = Reader::open_with_limits(&mut source, &cancel, exact).unwrap();
        let mut sink = Sink::default();
        reader.decode_record(0, &mut sink, &cancel).unwrap();
        assert_eq!(sink.work, 512);
        assert_eq!(
            reader.decode_record(1, &mut Sink::default(), &cancel),
            Err(DecodeError::Limit)
        );
        assert_eq!(
            reader.decode_record(2, &mut Sink::default(), &cancel),
            Err(DecodeError::Limit)
        );
        for limits in [
            Limits {
                input: exact.input - 1,
                ..exact
            },
            Limits {
                decoded: 143,
                ..exact
            },
            Limits {
                hash_work: 511,
                ..exact
            },
        ] {
            let mut source = Cursor::new(&bytes);
            let mut reader = Reader::open_with_limits(&mut source, &cancel, limits).unwrap();
            assert_eq!(
                reader.decode_record(0, &mut Sink::default(), &cancel),
                Err(DecodeError::Limit)
            );
        }
    }
    #[test]
    fn corrupted_whole_or_block_digest_or_data_never_returns_payload_success() {
        let cancel = Cancellation::default();
        let bytes = fixture("shared.asar");
        let index =
            index::inspect(&mut Cursor::new(&bytes), &cancel, ScanLimits::default()).unwrap();
        let mut variants = Vec::new();
        let mut data = bytes.clone();
        data[index.data_offset as usize] ^= 1;
        variants.push(data);
        for marker in [b"\"hash\":\"".as_slice(), b"\"blocks\":[\""] {
            let mut bad = bytes.clone();
            let at = bad
                .windows(marker.len())
                .position(|part| part == marker)
                .unwrap()
                + marker.len();
            bad[at] = if bad[at] == b'0' { b'1' } else { b'0' };
            variants.push(bad);
        }
        for bad in variants {
            let mut source = Cursor::new(bad);
            let mut reader = Reader::open(&mut source, &cancel).unwrap();
            assert_eq!(
                reader.decode_record(0, &mut Sink::default(), &cancel),
                Err(DecodeError::Integrity)
            );
            assert_eq!(
                reader.decode_record(2, &mut Sink::default(), &cancel),
                Err(DecodeError::Integrity)
            );
        }
    }
    #[test]
    fn unsupported_external_and_link_records_never_read_their_payload_or_resume() {
        let cancel = Cancellation::default();
        for name in ["external.asar", "link.asar"] {
            let mut source = Cursor::new(fixture(name));
            let mut reader = Reader::open(&mut source, &cancel).unwrap();
            let mut sink = Sink::default();
            assert_eq!(
                reader.decode_record(0, &mut sink, &cancel),
                Err(DecodeError::Unsupported)
            );
            assert!(sink.bytes.is_empty() && sink.work == 0);
            assert_eq!(
                reader.decode_record(0, &mut sink, &cancel),
                Err(DecodeError::Unsupported)
            );
        }
    }
    #[test]
    fn output_failure_and_cancellation_poison_readers_without_refunding_work() {
        struct Refuse;
        impl DecodedSink for Refuse {
            fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
                Err(DecodeError::Output)
            }
        }
        let cancel = Cancellation::default();
        let mut source = Cursor::new(fixture("shared.asar"));
        let mut reader = Reader::open(&mut source, &cancel).unwrap();
        assert_eq!(
            reader.decode_record(0, &mut Refuse, &cancel),
            Err(DecodeError::Output)
        );
        assert_eq!(
            reader.decode_record(2, &mut Sink::default(), &cancel),
            Err(DecodeError::Output)
        );
        drop(reader);
        let mut reader = Reader::open(&mut source, &cancel).unwrap();
        cancel.cancel();
        assert_eq!(
            reader.decode_record(0, &mut Sink::default(), &Cancellation::default()),
            Err(DecodeError::Cancelled)
        );
        assert_eq!(
            reader.decode_record(2, &mut Sink::default(), &Cancellation::default()),
            Err(DecodeError::Cancelled)
        );
    }
}
