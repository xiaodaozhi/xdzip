use super::*;
use std::io::{Cursor, Read};
use std::path::Path;
use std::process::Command;
use xdzip_compression::crc32;

fn decode(method: &[u8], properties: &[u8], input: &[u8], chunk: usize) -> Vec<u8> {
    let mut filter = ByteFilter::new(method, properties).unwrap();
    let control = Cancellation::default();
    let mut pending = Vec::new();
    let mut output = Vec::new();
    for (index, bytes) in input.chunks(chunk).enumerate() {
        pending.extend_from_slice(bytes);
        let eof = (index + 1) * chunk >= input.len();
        let used = filter.transform(&mut pending, eof, &control).unwrap();
        output.extend_from_slice(&pending[..used]);
        pending.drain(..used);
    }
    if input.is_empty() {
        assert_eq!(filter.transform(&mut [], true, &control).unwrap(), 0);
    }
    assert!(pending.is_empty());
    output
}

#[test]
fn normative_short_math_vectors_and_exact_properties() {
    let cases: &[(&[u8], &[u8], &[u8])] = &[
        (&[3], &[1, 2, 3, 4, 5], &[1, 2, 4, 6, 9]),
        (&[2, 3, 4], &[1, 2, 3, 4, 5, 6], &[4, 3, 2, 1, 5, 6]),
        (&[3, 3, 5, 1], &[0, 0, 0, 0xeb], &[0xfe, 0xff, 0xff, 0xeb]),
        (
            &[3, 3, 7, 1],
            &[0, 0xf0, 0, 0xf8],
            &[0xff, 0xf7, 0xfe, 0xff],
        ),
        (
            &[10],
            &[0, 0, 0, 0x94, 0, 0, 0, 0x94],
            &[0, 0, 0, 0x94, 0xff, 0xff, 0xff, 0x97],
        ),
    ];
    for (method, input, expected) in cases {
        let properties: &[u8] = if *method == [3] { &[1] } else { &[] };
        for chunk in 1..=19 {
            assert_eq!(decode(method, properties, input, chunk), *expected);
        }
    }
    assert!(matches!(
        ByteFilter::new(&[3], &[]),
        Err(DecodeError::Unsupported)
    ));
    assert!(matches!(
        ByteFilter::new(&[3, 3, 5, 1], &[0; 4]),
        Err(DecodeError::Unsupported)
    ));
    assert!(matches!(
        ByteFilter::new(&[10], &[2, 0, 0, 0]),
        Err(DecodeError::Unsupported)
    ));
    assert!(ByteFilter::new(&[11], &[2, 0, 0, 0]).is_ok());
    assert_eq!(decode(&[3], &[255], &[1; 257], 1)[256], 2);
}

#[test]
fn all_short_tails_and_chunk_boundaries_preserve_instruction_state() {
    let methods: &[&[u8]] = &[
        &[3, 3, 1, 3],
        &[3, 3, 2, 5],
        &[3, 3, 4, 1],
        &[3, 3, 5, 1],
        &[3, 3, 7, 1],
        &[3, 3, 8, 5],
        &[10],
        &[11],
        &[2, 3, 2],
        &[2, 3, 4],
    ];
    let mut data: Vec<u8> = (0..4099).map(|n| ((n * 101 + n / 7) % 256) as u8).collect();
    // Deliberately include nested x86 markers and instruction-shaped words.
    data[..25].copy_from_slice(&[
        0xe8, 0xe9, 0, 0, 0, 0xe8, 0, 0, 0, 0xff, 0x0f, 0x85, 0, 0, 0, 0, 0, 0xf0, 0, 0xf8, 0, 0,
        0, 0xeb, 0,
    ]);
    for method in methods {
        for len in 0..=31 {
            let expected = decode(method, &[], &data[..len], len.max(1));
            for chunk in 1..=len.max(1) {
                assert_eq!(decode(method, &[], &data[..len], chunk), expected);
            }
        }
        let expected = decode(method, &[], &data, data.len());
        for chunk in 1..=37 {
            assert_eq!(decode(method, &[], &data, chunk), expected);
        }
    }
}

#[test]
fn eof_and_cancellation_poison_the_filter() {
    let control = Cancellation::default();
    let mut filter = ByteFilter::new(&[10], &[]).unwrap();
    assert_eq!(
        filter.transform(&mut [1, 2, 3], false, &control).unwrap(),
        0
    );
    assert_eq!(filter.transform(&mut [1, 2, 3], true, &control).unwrap(), 3);
    assert!(matches!(
        filter.transform(&mut [], true, &control),
        Err(DecodeError::Integrity)
    ));
    let mut filter = ByteFilter::new(&[3], &[0]).unwrap();
    control.cancel();
    assert!(matches!(
        filter.transform(&mut [1], false, &control),
        Err(DecodeError::Cancelled)
    ));
    assert!(matches!(
        filter.transform(&mut [1], false, &control),
        Err(DecodeError::Integrity)
    ));
}

fn number(value: u64) -> Vec<u8> {
    for extra in 0..8 {
        if value < 1u64 << (7 * (extra + 1)) {
            let mut bytes = vec![((0xffu16 << (8 - extra)) as u8) | (value >> (8 * extra)) as u8];
            bytes.extend_from_slice(&value.to_le_bytes()[..extra]);
            return bytes;
        }
    }
    [vec![255], value.to_le_bytes().to_vec()].concat()
}
fn archive(method: &[u8], properties: &[u8], inputs: &[&[u8]], expected: &[u8]) -> Vec<u8> {
    let mut streams = vec![6, 0];
    streams.extend(number(inputs.len() as u64));
    streams.push(9);
    for input in inputs {
        streams.extend(number(input.len() as u64));
    }
    streams.extend([10, 1]);
    for input in inputs {
        streams.extend(crc32(input).to_le_bytes());
    }
    streams.extend([0, 7, 11, 1, 0, 1]);
    let flags = method.len() as u8
        | if properties.is_empty() { 0 } else { 0x20 }
        | if inputs.len() == 1 { 0 } else { 0x10 };
    streams.push(flags);
    streams.extend(method);
    if inputs.len() != 1 {
        streams.extend(number(inputs.len() as u64));
        streams.push(1);
    }
    if !properties.is_empty() {
        streams.extend(number(properties.len() as u64));
        streams.extend(properties);
    }
    if inputs.len() != 1 {
        for index in 0..inputs.len() {
            streams.extend(number(index as u64));
        }
    }
    streams.push(12);
    streams.extend(number(expected.len() as u64));
    streams.extend([10, 1]);
    streams.extend(crc32(expected).to_le_bytes());
    streams.extend([0, 0]);
    let mut header = vec![1, 4];
    header.extend(streams);
    header.extend([5, 1, 17]);
    let mut name = vec![0];
    for unit in "payload.bin".encode_utf16().chain([0]) {
        name.extend(unit.to_le_bytes());
    }
    header.extend(number(name.len() as u64));
    header.extend(name);
    header.extend([0, 0]);
    let packed: Vec<u8> = inputs
        .iter()
        .flat_map(|input| input.iter().copied())
        .collect();
    let mut start = (packed.len() as u64).to_le_bytes().to_vec();
    start.extend((header.len() as u64).to_le_bytes());
    start.extend(crc32(&header).to_le_bytes());
    let mut result = b"7z\xbc\xaf\x27\x1c\x00\x04".to_vec();
    result.extend(crc32(&start).to_le_bytes());
    result.extend(start);
    result.extend(packed);
    result.extend(header);
    result
}
fn verify_case(executable: &Path, output: &Path, label: &str, image: &[u8], expected: &[u8]) {
    let file = output.join(format!("{label}.7z"));
    std::fs::write(&file, image).unwrap();
    std::fs::write(output.join(format!("{label}.known")), expected).unwrap();
    let tested = Command::new(executable)
        .args(["t", "-bd", "-bb0"])
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        tested.status.success(),
        "official test failed for {label}: {:?}",
        tested.status.code()
    );
    let extracted = Command::new(executable)
        .args(["x", "-so", "-bd", "-bb0"])
        .arg(&file)
        .arg("payload.bin")
        .output()
        .unwrap();
    assert!(
        extracted.status.success(),
        "official extract failed for {label}: {:?}",
        extracted.status.code()
    );
    assert_eq!(
        extracted.stdout, expected,
        "official known bytes differ for {label}"
    );
}

#[test]
#[ignore = "requires the separately pinned official 7-Zip reference executable"]
fn official_filters_agree_on_owned_encoded_instruction_vectors() {
    let executable = std::env::var_os("XDZIP_7Z_REFERENCE").expect("pinned reference required");
    let output = std::env::var_os("XDZIP_7Z_FILTER_OUTPUT").expect("owned evidence path required");
    let output = Path::new(&output);
    std::fs::create_dir_all(output).unwrap();
    let methods: &[(&str, &[u8], &[u8])] = &[
        ("x86", &[3, 3, 1, 3], &[]),
        ("ppc", &[3, 3, 2, 5], &[]),
        ("ia64", &[3, 3, 4, 1], &[]),
        ("arm", &[3, 3, 5, 1], &[]),
        ("thumb", &[3, 3, 7, 1], &[]),
        ("sparc", &[3, 3, 8, 5], &[]),
        ("arm64", &[10], &[]),
        ("riscv", &[11], &[]),
        ("arm64-wrap", &[10], &[0xc0, 255, 255, 255]),
        ("riscv-wrap", &[11], &[0xc0, 255, 255, 255]),
        ("delta-1", &[3], &[0]),
        ("delta-2", &[3], &[1]),
        ("delta-256", &[3], &[255]),
        ("swap2", &[2, 3, 2], &[]),
        ("swap4", &[2, 3, 4], &[]),
    ];
    let mut input = Vec::new();
    let mut random = 0x8755a31du32;
    for index in 0..8193 {
        random ^= random << 13;
        random ^= random >> 17;
        random ^= random << 5;
        input.push((random >> (index % 4 * 8)) as u8);
    }
    let instruction_words: [u32; 20] = [
        0xeb000000, 0x94000000, 0x90000000, 0xf0000000, 0xff0000e8, 0x0000e9e8, 0x00e8e9e8,
        0x01e8e9e8, 0xffe8e9e8, 0x48000001, 0x40000000, 0x7fc00000, 0xf800f000, 0x000000ef,
        0x000002ef, 0x00000317, 0x00830313, 0x00003117, 0x28003117, 0x00108093,
    ];
    for (index, slot) in input.as_chunks_mut::<4>().0.iter_mut().enumerate() {
        if index % 3 != 0 {
            slot.copy_from_slice(&instruction_words[index % instruction_words.len()].to_le_bytes());
        }
    }
    // IA64 templates with eligible 41-bit branch slots and varied immediates.
    for index in 0..32 {
        let mut bundle = 0x16u128;
        for slot in 0..3 {
            let immediate = ((index * 17021 + slot * 917) & 0x1fffff) as u128;
            let instruction = 5u128 << 37 | (immediate & 0xfffff) << 13 | (immediate >> 20) << 36;
            bundle |= instruction << (5 + slot * 41);
        }
        input[index * 16..index * 16 + 16].copy_from_slice(&bundle.to_le_bytes());
    }
    for (label, method, properties) in methods {
        let expected = decode(method, properties, &input, 1);
        verify_case(
            Path::new(&executable),
            output,
            label,
            &archive(method, properties, &[&input], &expected),
            &expected,
        );
    }
    std::fs::write(
        output.join("single-input-passed.txt"),
        "15 official test/extract known-byte cases\n",
    )
    .unwrap();
}

#[test]
#[ignore = "requires the separately pinned official 7-Zip reference executable"]
fn official_bcj2_agrees_on_owned_four_port_and_normalization_vectors() {
    let executable = std::env::var_os("XDZIP_7Z_REFERENCE").expect("pinned reference required");
    let output = std::env::var_os("XDZIP_7Z_FILTER_OUTPUT").expect("owned evidence path required");
    let output = Path::new(&output);
    std::fs::create_dir_all(output).unwrap();
    let cases: [([&[u8]; 4], &[u8]); 4] = [
        (
            [&[0xe8], &[0, 0, 0, 5], &[], &[0, 0x7f, 0xff, 0xfc, 0]],
            &[0xe8, 0, 0, 0, 0],
        ),
        (
            [&[0xe9], &[], &[0, 0, 0, 5], &[0, 0x7f, 0xff, 0xfc, 0]],
            &[0xe9, 0, 0, 0, 0],
        ),
        (
            [&[0x0f, 0x80], &[], &[0, 0, 0, 6], &[0, 0x7f, 0xff, 0xfc, 0]],
            &[0x0f, 0x80, 0, 0, 0, 0],
        ),
        (
            [
                &[0xe8, 0x80],
                &[0x0f, 0, 0, 5],
                &[],
                &[0, 0x7f, 0xff, 0xfc, 0],
            ],
            &[0xe8, 0, 0, 0, 0x0f, 0x80],
        ),
    ];
    for (index, (inputs, expected)) in cases.iter().enumerate() {
        let control = Cancellation::default();
        let mut reader = crate::bcj2::Bcj2Reader::new(
            inputs.map(Cursor::new),
            inputs.map(|n| n.len() as u64),
            expected.len() as u64,
            &control,
        )
        .unwrap();
        let mut actual = Vec::new();
        reader.read_to_end(&mut actual).unwrap();
        reader.finish().unwrap();
        assert_eq!(&actual, expected);
        verify_case(
            Path::new(&executable),
            output,
            &format!("bcj2-{index}"),
            &archive(&[3, 3, 1, 0x1b], &[], inputs, expected),
            expected,
        );
    }
    let main: Vec<u8> = (0..32769)
        .map(|n| [0xe8, 0xe9, 0x0f, 0x85, 0, 255, 0xe8][n % 7])
        .collect();
    let mut probabilities = [1024u32; 258];
    let mut range = u32::MAX;
    let mut normalizations = 0;
    let mut previous = 0;
    for &byte in &main {
        let context = if byte == 0xe8 {
            Some(2 + usize::from(previous))
        } else if byte == 0xe9 {
            Some(1)
        } else if previous == 0x0f && byte & 0xf0 == 0x80 {
            Some(0)
        } else {
            None
        };
        if let Some(context) = context {
            let p = probabilities[context];
            range = (range >> 11) * p;
            probabilities[context] += (2048 - p) >> 5;
            if range < 1 << 24 {
                range <<= 8;
                normalizations += 1;
            }
        }
        previous = byte;
    }
    let rc = vec![0; 5 + normalizations];
    let inputs: [&[u8]; 4] = [&main, &[], &[], &rc];
    let control = Cancellation::default();
    let mut reader = crate::bcj2::Bcj2Reader::new(
        inputs.map(Cursor::new),
        inputs.map(|n| n.len() as u64),
        main.len() as u64,
        &control,
    )
    .unwrap();
    let mut actual = Vec::new();
    reader.read_to_end(&mut actual).unwrap();
    reader.finish().unwrap();
    assert_eq!(actual, main);
    verify_case(
        Path::new(&executable),
        output,
        "bcj2-normalization",
        &archive(&[3, 3, 1, 0x1b], &[], &inputs, &main),
        &main,
    );
    std::fs::write(
        output.join("bcj2-passed.txt"),
        "5 official test/extract known-byte cases\n",
    )
    .unwrap();
}
