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

#[test]
fn egg_comments_follow_the_physical_entry_into_the_query_page() {
    let mut bytes = egg(b"owned.txt", b"runtime egg");
    let block = bytes
        .windows(4)
        .position(|part| part == [0x13, 0x0c, 0xb5, 0x02])
        .unwrap();
    let comment = "注释".as_bytes();
    let mut header = vec![0x72, 0x36, 0xc6, 0x04, 0];
    header.extend_from_slice(&(comment.len() as u16).to_le_bytes());
    header.extend_from_slice(comment);
    bytes.splice(block - 4..block - 4, header);
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("xdzip-egg-comment-{}-{nonce}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let path = root.join("comment.egg");
    std::fs::write(&path, &bytes).unwrap();
    let control = Cancellation::default();
    let mut archive = crate::Archive::open(&path, &control).unwrap();
    assert!(archive.rows[0].details.comment.is_empty());
    let page = archive
        .query_page(
            crate::query::QueryRequest {
                directory: "",
                text: "",
                sort: 0,
                offset: 0,
                expected: None,
                generation: 1,
            },
            &control,
        )
        .unwrap();
    let crate::query::QueryRow::Physical(row) = &page.rows[0] else {
        panic!("expected physical entry")
    };
    assert_eq!(row.details.comment, comment);
    assert!(row.details.comment_utf8);
    drop(archive);
    std::fs::remove_file(path).unwrap();
    std::fs::remove_dir(root).unwrap();
}
