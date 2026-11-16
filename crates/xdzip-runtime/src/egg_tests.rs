use crate::formats::{detect, discover_weak};
use std::io::{Cursor, Write};
use xdzip_codec_api::{Cancellation, Format};
use xdzip_compression::{DeflateWriter, crc32};

fn egg(name: &[u8], body: &[u8]) -> Vec<u8> {
    let mut packed = Vec::new();
    let mut writer = DeflateWriter::new(&mut packed, 6).unwrap();
    writer.write_all(body).unwrap();
    writer.finish().unwrap();
    let end = [0x22, 0x82, 0xe2, 0x08];
    let mut out = b"EGGA\x00\x01\x01\x00\x00\x00\x00\x00\x00\x00".to_vec();
    out.extend_from_slice(&end);
    out.extend_from_slice(&[0xe3, 0x90, 0x85, 0x0a]);
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&(body.len() as u64).to_le_bytes());
    out.extend_from_slice(&[0xac, 0x91, 0x85, 0x0a, 0, name.len() as u8, 0]);
    out.extend_from_slice(name);
    out.extend_from_slice(&end);
    out.extend_from_slice(&[0x13, 0x0c, 0xb5, 0x02, 1, 0]);
    out.extend_from_slice(&(body.len() as u32).to_le_bytes());
    out.extend_from_slice(&(packed.len() as u32).to_le_bytes());
    out.extend_from_slice(&crc32(body).to_le_bytes());
    out.extend_from_slice(&end);
    out.extend_from_slice(&packed);
    out.extend_from_slice(&end);
    out
}

#[test]
fn runtime_routes_egg_magic_through_weak_discovery() {
    let bytes = egg(b"owned.txt", b"runtime egg");
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    assert_eq!(detect(&mut source, &cancel).unwrap(), Format::Egg);
    let (format, rows) = discover_weak(&mut source, &cancel).unwrap().unwrap();
    assert_eq!(format, Format::Egg);
    assert_eq!(rows[0].name_bytes(), b"owned.txt");
}
