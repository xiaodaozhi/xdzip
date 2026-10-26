use super::*;
use crate::tests::Parts;
use std::fmt::Write;

const SIZE: u64 = 2 * 1024 * 1024 * 1024 + 1;
const WORK_LIMIT: u64 = 20 * 1024 * 1024 * 1024;
const ARCHIVE_SHA: &str = "e2b067cfcc7aff4f8ad9e8b9cef3755dec65d9109bbcccf328c679a8546d2148";
const ORIGINAL_SHA: &str = "d1ea9e8a1dd77c140253106db1be9331430622ee141e57a1ccdf31bab1d5a13a";

const SPLIT_VOLUMES: &[(&[u8], usize, &str)] = &[
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.rar"),
        65536,
        "fb52ea700c4f82edf6ada0a2940d6af46c60f3102b3c7bdbe198d146679f1bcb",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r00"),
        65536,
        "eaaf7678c9ddb3265af58d01b3ef51ff9e6f062e3d72c618ca9a07888c70d199",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r01"),
        65536,
        "bf09e3b9a28d2f6647681019dca3e58ee866d35e601cb09130332db6d25c7948",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r02"),
        65536,
        "aa7de01a03d203d484261067fc03ea6a3c1440fc44bc9614f8554c44ba15e485",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r03"),
        65536,
        "147baec3c33de7448e0b20ddece21b80c1b46a5ec027a29d1ee8d468612c247e",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r04"),
        65536,
        "d81af52a46fde38b515473e31ba888b88c042484c748d3f23a44e1d37539b125",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r05"),
        65536,
        "e5b5d995838cd7042792446c9f48a7644f036d2f3de25d8569deeb5d2eca6542",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r06"),
        65536,
        "772ff1ec28afb80ad64a6a8b782df7a37c492e7a34187020112a9b5dc55d9c9a",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r07"),
        65536,
        "918c0974f63a428a0c96a0ebe1dec324184c3aeef75bc78d2afd6295fe91d451",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r08"),
        65536,
        "a6cff1b1cbe6f89146fc6de0b2afbc72da4a564129523c606a55c7dde0031336",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r09"),
        65536,
        "c07897f0b077b51b10ece65afa7c648c457f97834e21c97a6f4fc5185402260d",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r10"),
        65536,
        "86696a0b55a9acd7dc11b5e76472c72ce8af917c00a0973b0d0c9582b917b52d",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r11"),
        65536,
        "54a974848808ae311657fd415d466e4216394c2c0e505c26bcdf2cf86c19aa2b",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r12"),
        65536,
        "0273b3f4f8303d53babd53f4c42c11834f88309e756d417779c4b48aaf3d3c00",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r13"),
        65536,
        "a768cde22c2ca0b163f2c8b7e94bddf428a3623ee6c5f0d51e5201e7314c3e79",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r14"),
        65536,
        "c1899e897896f16a4f9182fc84220dcdd34379d2182971b1b7b95a4cf5986315",
    ),
    (
        include_bytes!("../tests/fixtures/legacy26-split-owned/split26.r15"),
        15266,
        "5247270581cf2116fa3f3fd9d0cc074f5a4b8769f5c6e122e9e195ba55260953",
    ),
];

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

fn stream_known_bytes(
    source: &mut dyn ReadSource,
    index: &RarIndex,
    control: &Cancellation,
) -> (u64, u64, u64, usize) {
    let mut payload = RarCodec
        .open_payload(source, index.entries(), control)
        .unwrap();
    let mut sink = BoundedSink {
        hash: xdzip_crypto::SecretSha256::new(),
        bytes: 0,
        work: 0,
        credit: 0,
        auxiliary: 0,
        maximum_chunk: 0,
    };
    payload
        .decode(&index.entries()[0], &mut sink, control)
        .unwrap();
    assert_eq!(hex(sink.hash.finish().as_ref()), ORIGINAL_SHA);
    assert_eq!(sink.bytes, SIZE);
    assert_eq!(sink.work, SIZE);
    assert_eq!(sink.credit, 0);
    assert!(sink.auxiliary > 0);
    assert!(sink.maximum_chunk <= 8192);
    (sink.bytes, sink.work, sink.auxiliary, sink.maximum_chunk)
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
    let (bytes, work, auxiliary, maximum_chunk) = stream_known_bytes(&mut source, &index, &control);
    println!(
        "RAR_LARGE_RESULT\t26\t{}\t{}\t{}\t{}\t{}\t{}\tcrc_verified",
        bytes, ORIGINAL_SHA, ARCHIVE_SHA, work, auxiliary, maximum_chunk
    );
}

#[test]
#[ignore = "explicit release-mode 17-volume 2GiB+1 full SHA/CRC/credit regression; no large plaintext"]
fn official_290_version26_split_large_streaming_sha_crc_and_bounded_sink() {
    let mut ordered_hash = xdzip_crypto::SecretSha256::new();
    let volumes: Vec<Vec<u8>> = SPLIT_VOLUMES
        .iter()
        .map(|(bytes, length, expected_sha)| {
            assert_eq!(bytes.len(), *length);
            let mut hash = xdzip_crypto::SecretSha256::new();
            hash.update(bytes);
            assert_eq!(hex(hash.finish().as_ref()), *expected_sha);
            ordered_hash.update(bytes);
            bytes.to_vec()
        })
        .collect();
    assert_eq!(volumes.len(), 17);
    assert_eq!(volumes.iter().map(Vec::len).sum::<usize>(), 1_063_842);
    let ordered_sha = hex(ordered_hash.finish().as_ref());
    assert_eq!(
        ordered_sha,
        "7319c547fdbfaaad63b136e5e305db5f9e73fb19d822990c79cc346508e0bbbd"
    );
    let control = Cancellation::default();
    let mut source = Parts::new(volumes);
    let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
    assert_eq!(
        (
            index.entries().len(),
            index.0.files.len(),
            index.0.parts.len()
        ),
        (1, 1, 17)
    );
    let row = &index.entries()[0];
    assert!(row.size_known);
    assert_eq!((row.method, row.size, row.packed_size), (26, SIZE, 65_412));
    assert_eq!(
        row.name_bytes(),
        b"GitHub/xdzip/build/rar-backend/build/rar26-owned/large-owned.bin"
    );
    assert_eq!(index.0.files[0].parts.len(), 17);
    for (i, part_index) in index.0.files[0].parts.iter().enumerate() {
        let part = &index.0.parts[*part_index];
        let flags = u16::from_le_bytes([part.header[3], part.header[4]]);
        assert_eq!(
            flags,
            if i == 0 {
                0x8102
            } else if i == 16 {
                0x8101
            } else {
                0x8103
            }
        );
        assert_eq!(part.volume, i);
        assert_eq!(part.before, i != 0);
        assert_eq!(part.after, i != 16);
        assert_eq!(part.packed, if i == 16 { 15_142 } else { 65_412 });
        assert_eq!(
            (part.description.method, part.description.compression),
            (26, 1)
        );
        assert_eq!(part.description.size, Some(SIZE));
        assert!(!part.description.solid && !part.description.encrypted);
        assert!(part.description.crc.is_some());
    }
    let (bytes, work, auxiliary, maximum_chunk) = stream_known_bytes(&mut source, &index, &control);
    println!(
        "RAR_LARGE_SPLIT_RESULT\t26\t17\t{}\t{}\t{}\t{}\t{}\t{}\tcrc_verified",
        bytes, ORIGINAL_SHA, ordered_sha, work, auxiliary, maximum_chunk
    );
}
