//! Deterministic, owned empty-file ZIP64 records for metadata and transport tests.
#![allow(dead_code)]
fn set16(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_le_bytes());
}
fn set32(bytes: &mut [u8], at: usize, value: u32) {
    bytes[at..at + 4].copy_from_slice(&value.to_le_bytes());
}
fn set64(bytes: &mut [u8], at: usize, value: u64) {
    bytes[at..at + 8].copy_from_slice(&value.to_le_bytes());
}

pub fn archive(count: usize) -> Vec<u8> {
    archive_names(
        &(0..count)
            .map(|n| format!("item-{n:06}.txt"))
            .collect::<Vec<_>>(),
    )
}
pub fn archive_names(names: &[String]) -> Vec<u8> {
    let count = names.len();
    let mut bytes = Vec::new();
    let mut directory = Vec::new();
    for name in names {
        let offset = bytes.len() as u32;
        let date = (46 << 9) | (1 << 5) | 1;
        let mut local = [0; 30];
        set32(&mut local, 0, 0x04034b50);
        set16(&mut local, 4, 20);
        set16(&mut local, 6, 2048);
        set16(&mut local, 12, date);
        set16(&mut local, 26, name.len() as u16);
        bytes.extend(local);
        bytes.extend(name.as_bytes());
        let mut central = [0; 46];
        set32(&mut central, 0, 0x02014b50);
        set16(&mut central, 4, 20);
        set16(&mut central, 6, 20);
        set16(&mut central, 8, 2048);
        set16(&mut central, 14, date);
        set16(&mut central, 28, name.len() as u16);
        set32(&mut central, 42, offset);
        directory.extend(central);
        directory.extend(name.as_bytes());
    }
    let offset = bytes.len() as u64;
    let size = directory.len() as u64;
    bytes.extend(directory);
    let zip64_at = bytes.len() as u64;
    let mut zip64 = [0; 56];
    set32(&mut zip64, 0, 0x06064b50);
    set64(&mut zip64, 4, 44);
    set16(&mut zip64, 12, 45);
    set16(&mut zip64, 14, 45);
    set64(&mut zip64, 24, count as u64);
    set64(&mut zip64, 32, count as u64);
    set64(&mut zip64, 40, size);
    set64(&mut zip64, 48, offset);
    bytes.extend(zip64);
    let mut locator = [0; 20];
    set32(&mut locator, 0, 0x07064b50);
    set64(&mut locator, 8, zip64_at);
    set32(&mut locator, 16, 1);
    bytes.extend(locator);
    let mut end = [0; 22];
    set32(&mut end, 0, 0x06054b50);
    set16(&mut end, 8, u16::MAX);
    set16(&mut end, 10, u16::MAX);
    set32(&mut end, 12, u32::MAX);
    set32(&mut end, 16, u32::MAX);
    bytes.extend(end);
    bytes
}

pub fn classic(count: usize) -> Vec<u8> {
    assert!(count <= u16::MAX as usize);
    let mut bytes = archive(count);
    let footer_at = bytes.len() - 98;
    let size = u64::from_le_bytes(bytes[footer_at + 40..footer_at + 48].try_into().unwrap());
    let offset = u64::from_le_bytes(bytes[footer_at + 48..footer_at + 56].try_into().unwrap());
    bytes.truncate(footer_at);
    let mut end = [0; 22];
    set32(&mut end, 0, 0x06054b50);
    set16(&mut end, 8, count as u16);
    set16(&mut end, 10, count as u16);
    set32(&mut end, 12, size.try_into().unwrap());
    set32(&mut end, 16, offset.try_into().unwrap());
    bytes.extend(end);
    bytes
}

pub fn declared_sizes(unpacked: u64, packed: u64, disk: u32) -> Vec<u8> {
    let original = archive(1);
    let central = original
        .windows(4)
        .position(|w| w == b"PK\x01\x02")
        .unwrap();
    let end = original.len() - 98;
    let mut bytes = original[..end].to_vec();
    set16(&mut bytes, central + 6, 45);
    set32(&mut bytes, central + 20, u32::MAX);
    set32(&mut bytes, central + 24, u32::MAX);
    set16(&mut bytes, central + 30, 32);
    set16(&mut bytes, central + 34, u16::MAX);
    set32(&mut bytes, central + 42, u32::MAX);
    let mut extra = [0; 32];
    set16(&mut extra, 0, 1);
    set16(&mut extra, 2, 28);
    set64(&mut extra, 4, unpacked);
    set64(&mut extra, 12, packed);
    set64(&mut extra, 20, 0);
    set32(&mut extra, 28, disk);
    bytes.extend(extra);
    let mut footer = original[end..].to_vec();
    set64(&mut footer, 40, (bytes.len() - central) as u64);
    set64(&mut footer, 64, bytes.len() as u64);
    bytes.extend(footer);
    bytes
}
