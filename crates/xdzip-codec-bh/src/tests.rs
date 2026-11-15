use super::*;
use std::io::Cursor;
use xdzip_codec_api::{
    Cancellation, MetadataCodec, ScanLimits,
    payload::{DecodeError, DecodedSink, PayloadCodec},
};
use xdzip_compression::{DeflateWriter, crc32};

fn member(name: &[u8], body: &[u8], method: u8) -> Vec<u8> {
    let mut packed = Vec::new();
    let mut writer = DeflateWriter::new(&mut packed, 6).unwrap();
    std::io::Write::write_all(&mut writer, body).unwrap();
    writer.finish().unwrap();
    let mut out = Vec::new();
    out.extend_from_slice(b"BH\x05\x07");
    out.extend_from_slice(&(37u16 + name.len() as u16).to_le_bytes());
    out.extend_from_slice(&[0x25, 0xd8, 0x0c, 0]);
    out.push(method);
    out.extend_from_slice(&0x12345678u32.to_le_bytes());
    out.extend_from_slice(&(packed.len() as u32).to_le_bytes());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&crc32(body).to_le_bytes());
    out.push(0x20);
    out.extend_from_slice(&[0; 5]);
    out.extend_from_slice(&(name.len() as u32).to_le_bytes());
    out.extend_from_slice(name);
    out.extend_from_slice(&packed);
    out
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
fn scans_and_decodes_multiple_members_with_exact_crc() {
    let mut bytes = member(b"one.txt", b"hello", 8);
    bytes.extend(member(b"two.bin", &[0, 1, 2, 3, 4], 8));
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = BhCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].format, xdzip_codec_api::Format::Bh);
    let mut payload = BhCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    payload.decode(&rows[0], &mut sink, &cancel).unwrap();
    assert_eq!(sink.0, b"hello");
    sink.0.clear();
    payload.decode(&rows[1], &mut sink, &cancel).unwrap();
    assert_eq!(sink.0, [0, 1, 2, 3, 4]);
}

#[test]
fn unknown_method_is_visible_but_not_extractable() {
    let mut source = Cursor::new(member(b"unknown", b"body", 7));
    let rows = BhCodec
        .scan(&mut source, &Cancellation::default(), ScanLimits::default())
        .unwrap();
    assert!(rows[0].warning);
    assert!(!rows[0].payload_supported());
}

#[test]
fn rejects_wrong_header_constants_and_crc() {
    let mut bytes = member(b"bad", b"body", 8);
    bytes[6] ^= 1;
    assert_eq!(
        inspect(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default()
        ),
        Err(xdzip_codec_api::ScanError::Invalid)
    );
    let mut bytes = member(b"bad", b"body", 8);
    let crc_at = 23;
    bytes[crc_at] ^= 1;
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = BhCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut payload = BhCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
}
