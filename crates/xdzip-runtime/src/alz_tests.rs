use crate::formats::{detect, discover_weak};
use std::io::{Cursor, Write};
use xdzip_codec_api::{Cancellation, Format};
use xdzip_compression::{DeflateWriter, crc32};

fn alz(name: &[u8], body: &[u8]) -> Vec<u8> {
    let mut packed = Vec::new();
    let mut writer = DeflateWriter::new(&mut packed, 6).unwrap();
    writer.write_all(body).unwrap();
    writer.finish().unwrap();
    let mut out = b"ALZ\x01\x01\x00\x01\x00".to_vec();
    out.extend_from_slice(b"BLZ\x01");
    out.extend_from_slice(&(name.len() as u16).to_le_bytes());
    out.push(0x20);
    out.extend_from_slice(&0x12345678u32.to_le_bytes());
    out.push(0x40);
    out.push(0);
    out.push(2);
    out.push(0);
    out.extend_from_slice(&crc32(body).to_le_bytes());
    out.extend_from_slice(&(packed.len() as u32).to_le_bytes());
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(name);
    out.extend_from_slice(&packed);
    out.extend_from_slice(b"CLZ\x02");
    out.extend_from_slice(&[0; 16]);
    out
}

#[test]
fn runtime_routes_alz_magic_through_weak_discovery() {
    let bytes = alz(b"owned.txt", b"runtime alz");
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    assert_eq!(detect(&mut source, &cancel).unwrap(), Format::Alz);
    let (format, rows) = discover_weak(&mut source, &cancel).unwrap().unwrap();
    assert_eq!(format, Format::Alz);
    assert_eq!(rows[0].name_bytes(), b"owned.txt");
}
