use super::*;
use std::io::Cursor;
use xdzip_codec_api::{
    Cancellation, MetadataCodec, ScanLimits,
    payload::{DecodeError, DecodedSink, PayloadCodec},
};
use xdzip_compression::{DeflateWriter, crc32};

const END_SIG: [u8; 4] = [0x22, 0x82, 0xe2, 0x08];
const FILE_SIG: [u8; 4] = [0xe3, 0x90, 0x85, 0x0a];
const FILENAME_SIG: [u8; 4] = [0xac, 0x91, 0x85, 0x0a];
const BLOCK_SIG: [u8; 4] = [0x13, 0x0c, 0xb5, 0x02];

fn archive(members: &[(&[u8], &[u8], u8)]) -> Vec<u8> {
    let mut out = b"EGGA\x00\x01\x01\x00\x00\x00\x00\x00\x00\x00".to_vec();
    out.extend_from_slice(&END_SIG);
    for (id, (name, body, method)) in members.iter().enumerate() {
        let packed = if *method == 0 {
            body.to_vec()
        } else {
            let mut packed = Vec::new();
            let mut writer = DeflateWriter::new(&mut packed, 6).unwrap();
            std::io::Write::write_all(&mut writer, body).unwrap();
            writer.finish().unwrap();
            packed
        };
        out.extend_from_slice(&FILE_SIG);
        out.extend_from_slice(&(id as u32).to_le_bytes());
        out.extend_from_slice(&(body.len() as u64).to_le_bytes());
        out.extend_from_slice(&FILENAME_SIG);
        out.push(0);
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(&END_SIG);
        out.extend_from_slice(&BLOCK_SIG);
        out.push(*method);
        out.push(0);
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(&(packed.len() as u32).to_le_bytes());
        out.extend_from_slice(&crc32(body).to_le_bytes());
        out.extend_from_slice(&END_SIG);
        out.extend_from_slice(&packed);
    }
    out.extend_from_slice(&END_SIG);
    out
}

#[test]
fn windows_info_exposes_egg_specific_attributes() {
    let mut bytes = archive(&[(b"file.txt", b"data", 0)]);
    let block = bytes.windows(4).position(|part| part == BLOCK_SIG).unwrap();
    let mut info = vec![0x0b, 0x95, 0x86, 0x2c, 0, 9, 0];
    info.extend_from_slice(&[0; 8]);
    info.push(3);
    bytes.splice(block - END_SIG.len()..block - END_SIG.len(), info);
    let rows = EggCodec
        .scan(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default(),
        )
        .unwrap();
    assert_eq!(rows[0].details.attributes.as_deref(), Some("__HR"));
}

#[test]
fn comments_are_read_lazily_with_bounded_output() {
    let mut bytes = archive(&[(b"file.txt", b"data", 0)]);
    let block = bytes.windows(4).position(|part| part == BLOCK_SIG).unwrap();
    let mut comments = Vec::new();
    for text in ["中文注释".as_bytes(), b"second".as_slice()] {
        comments.extend_from_slice(&[0x72, 0x36, 0xc6, 0x04, 0]);
        comments.extend_from_slice(&(text.len() as u16).to_le_bytes());
        comments.extend_from_slice(text);
    }
    bytes.splice(block - END_SIG.len()..block - END_SIG.len(), comments);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let index = inspect(&mut source, &control, ScanLimits::default()).unwrap();
    let region = index.records[0].comment_region.unwrap();
    let (comment, truncated) = read_comment(&mut source, &control, region).unwrap();
    assert_eq!(comment, "中文注释\nsecond".as_bytes());
    assert!(!truncated);
    let mut malformed = source.into_inner();
    let last_byte = region.1 as usize - 1;
    malformed[last_byte] ^= 1;
    assert!(read_comment(&mut Cursor::new(malformed), &control, region).is_err());
}

#[test]
fn oversized_comment_is_shortened_without_rejecting_the_archive() {
    let mut bytes = archive(&[(b"file.txt", b"data", 0)]);
    let block = bytes.windows(4).position(|part| part == BLOCK_SIG).unwrap();
    let mut comment = vec![0x72, 0x36, 0xc6, 0x04, 1];
    comment.extend_from_slice(&65_536u32.to_le_bytes());
    comment.extend(std::iter::repeat_n(b'x', 65_536));
    bytes.splice(block - END_SIG.len()..block - END_SIG.len(), comment);
    let control = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let index = inspect(&mut source, &control, ScanLimits::default()).unwrap();
    let (comment, truncated) = read_comment(
        &mut source,
        &control,
        index.records[0].comment_region.unwrap(),
    )
    .unwrap();
    assert_eq!(comment.len(), 65_535);
    assert!(truncated);
}

#[derive(Default)]
struct Sink(Vec<u8>);
impl DecodedSink for Sink {
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<(), DecodeError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

#[test]
fn scans_and_decodes_store_and_deflate_members() {
    let bytes = archive(&[(b"one.txt", b"hello", 1), (b"two.bin", &[0, 1, 2, 3, 4], 0)]);
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = EggCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].format, xdzip_codec_api::Format::Egg);
    let mut payload = EggCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    payload.decode(&rows[0], &mut sink, &cancel).unwrap();
    assert_eq!(sink.0, b"hello");
    sink.0.clear();
    payload.decode(&rows[1], &mut sink, &cancel).unwrap();
    assert_eq!(sink.0, [0, 1, 2, 3, 4]);
}

#[test]
fn unknown_method_is_visible_but_not_extractable() {
    let mut source = Cursor::new(archive(&[(b"unknown", b"body", 7)]));
    let rows = EggCodec
        .scan(&mut source, &Cancellation::default(), ScanLimits::default())
        .unwrap();
    assert!(rows[0].warning);
    assert!(!rows[0].payload_supported());
}

#[test]
fn rejects_wrong_magic_and_crc() {
    let mut bytes = archive(&[(b"bad", b"body", 1)]);
    bytes[0] ^= 1;
    assert_eq!(
        inspect(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default()
        ),
        Err(xdzip_codec_api::ScanError::Invalid)
    );
    let mut bytes = archive(&[(b"bad", b"body", 1)]);
    let block_crc = bytes
        .windows(4)
        .position(|window| window == BLOCK_SIG)
        .unwrap()
        + 4
        + 10;
    bytes[block_crc] ^= 1;
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = EggCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut payload = EggCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
}
