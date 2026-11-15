use crate::formats::{codec, detect};
use std::io::Cursor;
use xdzip_codec_api::{Cancellation, Format, ScanLimits};

fn fixture() -> Vec<u8> {
    let pe_offset = 0x80usize;
    let section_table = pe_offset + 4 + 20 + 0xe0;
    let overlay = 0x400usize;
    let mut bytes = vec![0u8; overlay + 128];
    bytes[..2].copy_from_slice(b"MZ");
    bytes[0x3c..0x40].copy_from_slice(&(pe_offset as u32).to_le_bytes());
    bytes[pe_offset..pe_offset + 4].copy_from_slice(b"PE\0\0");
    bytes[pe_offset + 4..pe_offset + 6].copy_from_slice(&0x14cu16.to_le_bytes());
    bytes[pe_offset + 6..pe_offset + 8].copy_from_slice(&1u16.to_le_bytes());
    bytes[pe_offset + 20..pe_offset + 22].copy_from_slice(&0xe0u16.to_le_bytes());
    bytes[section_table + 8..section_table + 12].copy_from_slice(&0x200u32.to_le_bytes());
    bytes[section_table + 16..section_table + 20].copy_from_slice(&0x200u32.to_le_bytes());
    bytes[section_table + 20..section_table + 24].copy_from_slice(&0x200u32.to_le_bytes());
    let first = overlay;
    bytes[first..first + 4].copy_from_slice(&0x02u32.to_le_bytes());
    bytes[first + 4..first + 8].copy_from_slice(&0xdead_beefu32.to_le_bytes());
    for (index, word) in [
        u32::from_le_bytes(*b"Null"),
        u32::from_le_bytes(*b"soft"),
        u32::from_le_bytes(*b"Inst"),
    ]
    .into_iter()
    .enumerate()
    {
        let begin = first + 8 + index * 4;
        bytes[begin..begin + 4].copy_from_slice(&word.to_le_bytes());
    }
    bytes[first + 20..first + 24].copy_from_slice(&64u32.to_le_bytes());
    bytes[first + 24..first + 28].copy_from_slice(&128u32.to_le_bytes());
    bytes
}

#[test]
fn runtime_detects_nsis_by_content_without_an_extension() {
    let mut source = Cursor::new(fixture());
    assert_eq!(
        detect(&mut source, &Cancellation::default()).unwrap(),
        Format::Nsis
    );
    assert_eq!(Format::Nsis.descriptor().extensions, [".exe"]);
    assert!(Format::Nsis.descriptor().extensions.len() == 1);
}

#[test]
fn runtime_publishes_metadata_and_no_payload_capability() {
    let mut source = Cursor::new(fixture());
    let rows = codec(Format::Nsis)
        .scan(&mut source, &Cancellation::default(), ScanLimits::default())
        .unwrap();
    assert_eq!(rows[0].display_name, "NSIS/firstheader");
    assert_eq!(rows[0].method, 0x02);
    assert!(!rows[0].payload_supported());
}
