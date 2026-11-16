use crate::formats::{detect, discover_weak};
use std::io::{Cursor, Write};
use xdzip_codec_api::{Cancellation, Format};
use xdzip_compression::{DeflateWriter, crc32};

fn bh(name: &[u8], body: &[u8]) -> Vec<u8> {
    let mut packed = Vec::new();
    let mut writer = DeflateWriter::new(&mut packed, 6).unwrap();
    writer.write_all(body).unwrap();
    writer.finish().unwrap();
    let mut out = b"BH\x05\x07".to_vec();
    out.extend_from_slice(&(37u16 + name.len() as u16).to_le_bytes());
    out.extend_from_slice(&[0x25, 0xd8, 0x0c, 0, 8]);
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

#[test]
fn runtime_routes_bh_magic_through_weak_discovery() {
    let bytes = bh(b"owned.txt", b"runtime bh");
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    let format = detect(&mut source, &cancel).unwrap();
    assert_eq!(format, Format::Tar);
    let (format, rows) = discover_weak(&mut source, &cancel).unwrap().unwrap();
    assert_eq!(format, Format::Bh);
    assert_eq!(rows[0].name_bytes(), b"owned.txt");
}
