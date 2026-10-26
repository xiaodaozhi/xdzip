use super::*;
use crate::tests::Parts;
use xdzip_codec_api::payload::DecodedSink;

const VOLUMES: &[(&[u8], usize, &str)] = &[
    (
        include_bytes!("../tests/fixtures/legacy20-split-owned/split20.rar"),
        32768,
        "cd4d16b5a1b22d963e53da0fbc554d7718d3ae412cb9312c815e971f2af1927a",
    ),
    (
        include_bytes!("../tests/fixtures/legacy20-split-owned/split20.r00"),
        32768,
        "8e0715de01e9da5d90b6f8f9340c2261b4864a18969d89902c570811f82cf49d",
    ),
    (
        include_bytes!("../tests/fixtures/legacy20-split-owned/split20.r01"),
        32768,
        "beb186d57c9c9ff3c64ba6002eddc7825a253491838e445ff3256d9cfda05916",
    ),
    (
        include_bytes!("../tests/fixtures/legacy20-split-owned/split20.r02"),
        32768,
        "712a9f2fb8d94fb0b41ba1200816bf93951c830a0ab2ceba76bac2d97097d19f",
    ),
    (
        include_bytes!("../tests/fixtures/legacy20-split-owned/split20.r03"),
        32768,
        "e91fb6e76c8ad47bdd0b8adee01a2647e5628dcf958be1b62b457a1fc645b032",
    ),
    (
        include_bytes!("../tests/fixtures/legacy20-split-owned/split20.r04"),
        32768,
        "5a001b58bd4c4b25ec4ac3d64d60cbec62e664c4705bd67d77bb3a3059668678",
    ),
    (
        include_bytes!("../tests/fixtures/legacy20-split-owned/split20.r05"),
        4383,
        "c52bdbf0f410a9d75ca84ce3435149a23f9c41d7ff25085f163b46026c93c1c1",
    ),
];

fn sha256(bytes: &[u8]) -> String {
    let mut hash = xdzip_crypto::SecretSha256::new();
    hash.update(bytes);
    hash.finish()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn known_input() -> Vec<u8> {
    let a: Vec<u8> = (0..131_071usize)
        .map(|i| (31 * i + 7 * (i >> 5) + 13) as u8)
        .collect();
    let mut b = Vec::with_capacity(196_640);
    for k in 0_u32..6145 {
        let mut hash = xdzip_crypto::SecretSha256::new();
        hash.update(b"xdzip-rar20-store-middle-v1");
        hash.update(&k.to_le_bytes());
        b.extend_from_slice(hash.finish().as_ref());
    }
    b.truncate(196_613);
    let mut original = Vec::with_capacity(589_826);
    for bytes in [
        a.as_slice(),
        &b[..98_317],
        a.as_slice(),
        &b[98_317..],
        a.as_slice(),
    ] {
        original.extend_from_slice(bytes);
    }
    assert_eq!(original.len(), 589_826);
    assert_eq!(
        sha256(&original),
        "b63fd1d316ba1dcf8a351d0ec895bcf066495cc1ebf1eae1aec7a823c4eb5567"
    );
    original
}

struct KnownBytes {
    expected: Vec<u8>,
    position: usize,
}
impl DecodedSink for KnownBytes {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        assert!(!bytes.is_empty() && bytes.len() <= 8192);
        let end = self.position + bytes.len();
        assert_eq!(bytes, &self.expected[self.position..end]);
        self.position = end;
        Ok(())
    }
}

#[test]
fn official_290_version20_seven_volumes_match_every_owned_byte_and_physical_identity() {
    let mut ordered = xdzip_crypto::SecretSha256::new();
    let volumes: Vec<Vec<u8>> = VOLUMES
        .iter()
        .map(|(bytes, length, expected_sha)| {
            assert_eq!(bytes.len(), *length);
            assert_eq!(sha256(bytes), *expected_sha);
            ordered.update(bytes);
            bytes.to_vec()
        })
        .collect();
    assert_eq!(volumes.len(), 7);
    assert_eq!(volumes.iter().map(Vec::len).sum::<usize>(), 200_991);
    let ordered_sha: String = ordered
        .finish()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    assert_eq!(
        ordered_sha,
        "58534ff4c0cfc3cd893f37b6c92a5e9ffd5a4c63bea68b6c02ee95d03f99ce31"
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
        (1, 1, 7)
    );
    let row = &index.entries()[0];
    assert_eq!(row.id.get(), 1);
    assert!(row.size_known);
    assert_eq!(
        (row.size, row.packed_size, row.method),
        (589_826, 32_707, 20)
    );
    assert_eq!(row.name_bytes(), b"known.bin");
    assert_eq!(index.0.files[0].parts.len(), 7);
    for (i, part_index) in index.0.files[0].parts.iter().enumerate() {
        let part = &index.0.parts[*part_index];
        let flags = u16::from_le_bytes([part.header[3], part.header[4]]);
        assert_eq!(
            flags,
            if i == 0 {
                0x8082
            } else if i == 6 {
                0x8081
            } else {
                0x8083
            }
        );
        assert_eq!(part.volume, i);
        assert_eq!(part.before, i != 0);
        assert_eq!(part.after, i != 6);
        assert_eq!(part.packed, if i == 6 { 4322 } else { 32_707 });
        assert_eq!(
            (part.description.method, part.description.compression),
            (20, 3)
        );
        assert_eq!(part.description.size, Some(589_826));
        assert!(!part.description.solid && !part.description.encrypted);
    }
    let mut sink = KnownBytes {
        expected: known_input(),
        position: 0,
    };
    let mut payload = RarCodec
        .open_payload(&mut source, index.entries(), &control)
        .unwrap();
    payload.decode(row, &mut sink, &control).unwrap();
    assert_eq!(sink.position, sink.expected.len());
    println!(
        "RAR_SPLIT20_RESULT\t7\t589826\tb63fd1d316ba1dcf8a351d0ec895bcf066495cc1ebf1eae1aec7a823c4eb5567\tcrc_verified"
    );
}
