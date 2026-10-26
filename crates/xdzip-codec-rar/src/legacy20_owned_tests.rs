use super::*;

const FIRST: &[u8] = b"first.bin";
const SECOND: &[u8] = b"second.bin";
const THIRD: &[u8] = b"third.bin";

fn known_inputs() -> [Vec<u8>; 3] {
    let first: Vec<u8> = (0..65_537usize)
        .map(|i| (31 * i + 7 * (i >> 5) + 13) as u8)
        .collect();
    let mut second = first[173..].to_vec();
    second.extend_from_slice(&first[..173]);
    second.extend_from_slice(&first[..32_780]);
    let third = (0..33_333usize)
        .map(|i| (i * i + 5 * i + (i >> 3)) as u8)
        .collect();
    [first, second, third]
}

fn verify(archive: &[u8], expected_sha: &str, solid: bool) {
    assert!(archive.len() <= 32 * 1024);
    let mut hash = xdzip_crypto::SecretSha256::new();
    hash.update(archive);
    let actual_sha: String = hash.finish().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(actual_sha, expected_sha);

    let control = Cancellation::default();
    let mut source = Cursor::new(archive);
    let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
    assert_eq!(index.entries().len(), 3);
    assert_eq!(index.0.files.len(), 3);
    let inputs = known_inputs();
    for (i, ((row, file), expected)) in index
        .entries()
        .iter()
        .zip(&index.0.files)
        .zip(&inputs)
        .enumerate()
    {
        assert_eq!(row.name_bytes(), [FIRST, SECOND, THIRD][i]);
        assert_eq!(row.id.get(), i as u64 + 1);
        assert_eq!(row.method, 20);
        assert!(row.size_known);
        assert_eq!(row.size, expected.len() as u64);
        let description = &index.0.parts[file.parts[0]].description;
        assert_eq!(description.compression, 5);
        assert_eq!(description.solid, solid && i != 0);
    }

    // A fresh reader for each target also validates bounded solid-prefix replay.
    // No external author executable or filesystem input is needed by this test.
    for (i, expected) in inputs.iter().enumerate() {
        let mut source = Cursor::new(archive);
        let mut reader = RarCodec
            .open_payload(&mut source, index.entries(), &control)
            .unwrap();
        let mut sink = Sink::default();
        reader
            .decode(&index.entries()[i], &mut sink, &control)
            .unwrap();
        assert_eq!(&sink.0, expected);
    }
}

#[test]
fn official_290_plain_members_match_original_mathematical_bytes() {
    verify(
        include_bytes!("../tests/fixtures/legacy20-owned/plain.rar"),
        "88f0d8be35da2a295ef64df74b17faaabfbffcf1d549673a03d38f003794b4f7",
        false,
    );
}

#[test]
fn official_290_solid_members_match_original_mathematical_bytes() {
    verify(
        include_bytes!("../tests/fixtures/legacy20-owned/solid.rar"),
        "5f37763957b2e6c26b4743bb9d2770043cd06f0faf0245ce60fae652dac1a292",
        true,
    );
}

#[test]
fn official_290_forced_audio_solid_matches_original_mathematical_bytes() {
    verify(
        include_bytes!("../tests/fixtures/legacy20-owned/audio-solid.rar"),
        "8e0aa78745399a629da5f23af63b9536f872c1d484ef22e344124c96036d69b8",
        true,
    );
}
