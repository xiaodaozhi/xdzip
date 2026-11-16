use super::*;
use std::io::Cursor;
use xdzip_codec_api::{
    Cancellation, MetadataCodec, ScanLimits,
    payload::{DecodeError, DecodedSink, PayloadCodec},
};
use xdzip_compression::{DeflateWriter, crc32};

fn archive(members: &[(&[u8], &[u8], u8)]) -> Vec<u8> {
    let mut out = b"ALZ\x01\x01\x00\x01\x00".to_vec();
    for (name, body, method) in members {
        let packed = if *method == 0 {
            body.to_vec()
        } else {
            let mut packed = Vec::new();
            let mut writer = DeflateWriter::new(&mut packed, 6).unwrap();
            std::io::Write::write_all(&mut writer, body).unwrap();
            writer.finish().unwrap();
            packed
        };
        out.extend_from_slice(b"BLZ\x01");
        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.push(0x20);
        out.extend_from_slice(&0x12345678u32.to_le_bytes());
        out.push(0x40);
        out.push(0);
        out.push(*method);
        out.push(0);
        out.extend_from_slice(&crc32(body).to_le_bytes());
        out.extend_from_slice(&(packed.len() as u32).to_le_bytes());
        out.extend_from_slice(&(body.len() as u32).to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(&packed);
    }
    out.extend_from_slice(b"CLZ\x02");
    out.extend_from_slice(&[0; 16]);
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
fn scans_and_decodes_store_and_deflate_members() {
    let bytes = archive(&[(b"one.txt", b"hello", 2), (b"two.bin", &[0, 1, 2, 3, 4], 0)]);
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = AlzCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].format, xdzip_codec_api::Format::Alz);
    let mut payload = AlzCodec.open_payload(&mut source, &rows, &cancel).unwrap();
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
    let rows = AlzCodec
        .scan(&mut source, &Cancellation::default(), ScanLimits::default())
        .unwrap();
    assert!(rows[0].warning);
    assert!(!rows[0].payload_supported());
}

#[test]
fn rejects_wrong_header_constants_and_crc() {
    let mut bytes = archive(&[(b"bad", b"body", 2)]);
    bytes[0] ^= 1;
    assert_eq!(
        inspect(
            &mut Cursor::new(bytes),
            &Cancellation::default(),
            ScanLimits::default()
        ),
        Err(xdzip_codec_api::ScanError::Invalid)
    );

    let mut bytes = archive(&[(b"bad", b"body", 2)]);
    let local = 8usize;
    bytes[local + 4 + 9 + 2] ^= 1;
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let rows = AlzCodec
        .scan(&mut source, &cancel, ScanLimits::default())
        .unwrap();
    let mut payload = AlzCodec.open_payload(&mut source, &rows, &cancel).unwrap();
    let mut sink = Sink::default();
    assert_eq!(
        payload.decode(&rows[0], &mut sink, &cancel),
        Err(DecodeError::Integrity)
    );
}
