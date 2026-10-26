use super::*;
use std::fmt::Write;

const SIZE: u64 = 2 * 1024 * 1024 * 1024 + 1;
const WORK_LIMIT: u64 = 20 * 1024 * 1024 * 1024;
const ARCHIVE_SHA: &str = "e2b067cfcc7aff4f8ad9e8b9cef3755dec65d9109bbcccf328c679a8546d2148";
const ORIGINAL_SHA: &str = "d1ea9e8a1dd77c140253106db1be9331430622ee141e57a1ccdf31bab1d5a13a";

fn hex(bytes: &[u8]) -> String {
    let mut text = String::new();
    for byte in bytes {
        write!(text, "{byte:02x}").unwrap();
    }
    text
}

struct BoundedSink {
    hash: xdzip_crypto::SecretSha256,
    bytes: u64,
    work: u64,
    credit: u64,
    auxiliary: u64,
    maximum_chunk: usize,
}
impl DecodedSink for BoundedSink {
    fn write_chunk(&mut self, _: &[u8]) -> Result<(), DecodeError> {
        Err(DecodeError::Output)
    }
    fn write_precharged_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        if bytes.is_empty() || bytes.len() > 8192 {
            return Err(DecodeError::Limit);
        }
        let count = bytes.len() as u64;
        self.credit = self.credit.checked_sub(count).ok_or(DecodeError::Output)?;
        self.bytes = self
            .bytes
            .checked_add(count)
            .filter(|n| *n <= SIZE)
            .ok_or(DecodeError::Limit)?;
        self.maximum_chunk = self.maximum_chunk.max(bytes.len());
        self.hash.update(bytes);
        Ok(())
    }
    fn charge_skipped(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.work = self
            .work
            .checked_add(bytes)
            .filter(|n| *n <= WORK_LIMIT)
            .ok_or(DecodeError::Limit)?;
        self.credit = self.credit.checked_add(bytes).ok_or(DecodeError::Limit)?;
        Ok(())
    }
    fn charge_auxiliary(&mut self, bytes: u64) -> Result<(), DecodeError> {
        self.auxiliary = self
            .auxiliary
            .checked_add(bytes)
            .filter(|n| *n <= WORK_LIMIT)
            .ok_or(DecodeError::Limit)?;
        Ok(())
    }
}

#[test]
#[ignore = "explicit release-mode 2GiB+1 streamed SHA/CRC/credit regression; no large plaintext allocation"]
fn official_290_version26_large_streaming_sha_crc_and_bounded_sink() {
    let archive = include_bytes!("../tests/fixtures/legacy26-owned/large26.rar");
    assert_eq!(archive.len(), 1_061_809);
    let mut hash = xdzip_crypto::SecretSha256::new();
    hash.update(archive);
    assert_eq!(hex(hash.finish().as_ref()), ARCHIVE_SHA);
    let control = Cancellation::default();
    let mut source = Cursor::new(archive.as_slice());
    let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
    assert_eq!(index.entries().len(), 1);
    let row = &index.entries()[0];
    assert!(row.size_known);
    assert_eq!(row.size, SIZE);
    assert_eq!(row.method, 26);
    assert_eq!(row.name_bytes(), b"large-owned.bin");
    let description = &index.0.parts[index.0.files[0].parts[0]].description;
    assert_eq!(description.compression, 1);
    assert!(description.crc.is_some());
    let mut payload = RarCodec
        .open_payload(&mut source, index.entries(), &control)
        .unwrap();
    let mut sink = BoundedSink {
        hash: xdzip_crypto::SecretSha256::new(),
        bytes: 0,
        work: 0,
        credit: 0,
        auxiliary: 0,
        maximum_chunk: 0,
    };
    payload.decode(row, &mut sink, &control).unwrap();
    let actual_sha = hex(sink.hash.finish().as_ref());
    assert_eq!(actual_sha, ORIGINAL_SHA);
    assert_eq!(sink.bytes, SIZE);
    assert_eq!(sink.work, SIZE);
    assert_eq!(sink.credit, 0);
    assert!(sink.auxiliary > 0);
    assert!(sink.maximum_chunk <= 8192);
    println!(
        "RAR_LARGE_RESULT\t26\t{}\t{}\t{}\t{}\t{}\t{}\tcrc_verified",
        sink.bytes, actual_sha, ARCHIVE_SHA, sink.work, sink.auxiliary, sink.maximum_chunk
    );
}
