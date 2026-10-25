use super::*;
use crate::tests::Parts;
use std::io::Cursor;
use xdzip_codec_api::payload::DecodedSink;

fn sum(bytes: &[u8]) -> u16 {
    bytes
        .iter()
        .fold(0u16, |c, &b| c.wrapping_add(u16::from(b)).rotate_left(1))
}
fn main(flags: u8, extension: &[u8]) -> Vec<u8> {
    let mut out = b"RE~^".to_vec();
    out.extend((7u16 + extension.len() as u16).to_le_bytes());
    out.push(flags);
    out.extend(extension);
    out
}
#[allow(clippy::too_many_arguments)] // Original independent wire fields.
fn file(
    name: &[u8],
    packed: &[u8],
    size: u32,
    checksum: u16,
    flags: u8,
    selector: u8,
    method: u8,
    attr: u8,
    extension: &[u8],
) -> Vec<u8> {
    let mut out = (packed.len() as u32).to_le_bytes().to_vec();
    out.extend(size.to_le_bytes());
    out.extend(checksum.to_le_bytes());
    out.extend((21u16 + name.len() as u16 + extension.len() as u16).to_le_bytes());
    out.extend(0x58210000u32.to_le_bytes());
    out.extend([attr, flags, selector, name.len() as u8, method]);
    out.extend(name);
    out.extend(extension);
    out.extend(packed);
    out
}
#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend(bytes);
        Ok(())
    }
}

#[test]
fn independent_ancient_headers_keep_generation_flags_extra_bytes_and_checksum() {
    // c: 0 -> 2 -> 8 -> 22. No CRC32 or final complement participates.
    assert_eq!(sum(&[1, 2, 3]), 22);
    let mut bytes = main(0x80, &[0xa1, 0xa2]);
    bytes.extend(file(b"same", &[1, 2, 3], 3, 22, 0xf0, 2, 0, 0x20, &[0xfe]));
    bytes.extend(file(b"same", &[4], 1, 8, 0, 255, 0, 0x20, &[]));
    bytes.extend(file(b"empty", &[], 0, 0, 0, 0, 0, 0x10, &[]));
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
    assert_eq!(
        index.entries().iter().map(|e| e.method).collect::<Vec<_>>(),
        [13, 10, 10]
    );
    assert_eq!(
        index
            .entries()
            .iter()
            .map(|e| e.id.get())
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    assert_eq!(
        index.entries()[0].name_bytes(),
        index.entries()[1].name_bytes()
    );
    assert!(!index.entries()[0].directory);
    assert!(!index.entries()[0].encrypted);
    assert!(index.entries()[2].directory);
    assert_eq!(index.0.parts[0].header.len(), 26);
    let mut payload = RarCodec
        .open_payload(&mut source, index.entries(), &control)
        .unwrap();
    for (entry, expected) in index.entries().iter().zip([&[1, 2, 3][..], &[4], &[]]) {
        let mut sink = Sink::default();
        payload.decode(entry, &mut sink, &control).unwrap();
        assert_eq!(sink.0, expected);
    }
}

#[test]
fn ancient_split_ignores_nonfinal_sum_and_hashes_complete_logical_output() {
    let mut first = main(0x81, &[]);
    first.extend(file(b"part", &[1], 3, 0xffff, 2, 2, 0, 0x20, &[]));
    let mut second = main(0x81, &[]);
    second.extend(file(b"part", &[2, 3], 3, 22, 1, 2, 0, 0x20, &[]));
    let control = Cancellation::default();
    let mut source = Parts::new(vec![first.clone(), second.clone()]);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    assert_eq!((rows.len(), rows[0].packed_size, rows[0].size), (1, 1, 3));
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    let mut sink = Sink::default();
    payload.decode(&rows[0], &mut sink, &control).unwrap();
    assert_eq!(sink.0, [1, 2, 3]);
    second[7 + 8] ^= 1;
    let mut source = Parts::new(vec![first, second]);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Sink::default(), &control),
        Err(DecodeError::Integrity)
    );
    assert_eq!(
        payload.decode(&rows[0], &mut Sink::default(), &control),
        Err(DecodeError::Integrity)
    );
}

#[test]
fn ancient_solid_starts_first_logical_file_and_never_promotes_method_ten() {
    let mut bytes = main(0x88, &[]);
    bytes.extend(file(b"first", &[0], 1, 0, 0x10, 2, 3, 0x20, &[]));
    bytes.extend(file(b"second", &[0], 1, 0, 0, 2, 3, 0x20, &[]));
    bytes.extend(file(b"ten", &[0], 1, 0, 0, 255, 3, 0x20, &[]));
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let index = read_index(&mut source, &control, ScanLimits::default(), None).unwrap();
    assert!(!index.0.parts[0].description.solid);
    assert!(index.0.parts[1].description.solid);
    assert!(index.entries()[2].warning);
    let mut payload = RarCodec
        .open_payload(&mut source, index.entries(), &control)
        .unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        payload.decode(&index.entries()[2], &mut sink, &control),
        Err(DecodeError::Unsupported)
    );
    assert!(sink.0.is_empty());
}

#[test]
fn ancient_bounds_and_final_sum_are_checked_without_header_crc() {
    let mut valid = main(0x80, &[]);
    valid.extend(file(b"name", b"abc", 3, sum(b"abc"), 0, 2, 0, 0x20, &[]));
    let control = Cancellation::default();
    for offset in [7 + 10, 7 + 19] {
        let mut bytes = valid.clone();
        bytes[offset] = 0;
        if offset == 7 + 19 {
            bytes[offset] = 255;
        }
        assert_eq!(
            RarCodec.scan(&mut Cursor::new(bytes), &control, ScanLimits::default()),
            Err(ScanError::Invalid)
        );
    }
    let mut truncated = valid.clone();
    truncated.pop();
    assert_eq!(
        RarCodec.scan(&mut Cursor::new(truncated), &control, ScanLimits::default()),
        Err(ScanError::Invalid)
    );
    valid[7 + 8] ^= 1;
    let mut source = Cursor::new(valid);
    let rows = RarCodec
        .scan(&mut source, &control, ScanLimits::default())
        .unwrap();
    let mut payload = RarCodec.open_payload(&mut source, &rows, &control).unwrap();
    assert_eq!(
        payload.decode(&rows[0], &mut Sink::default(), &control),
        Err(DecodeError::Integrity)
    );
}
