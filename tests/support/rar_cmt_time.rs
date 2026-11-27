//! Build a small CMT regression from the pinned, author-produced encrypted
//! RAR4 fixture. Only metadata changes; the encrypted payload and its CRC are
//! preserved. The plaintext of b.txt is established by protection_tests.

pub fn encrypted_comment(subdata: &[u8]) -> Vec<u8> {
    encrypted_comment_with_time(subdata, None)
}

pub fn encrypted_comment_with_time(subdata: &[u8], time: Option<&[u8]>) -> Vec<u8> {
    let source = include_bytes!("../fixtures/native-archives/test_read_format_rar4_encrypted.rar");
    let parent = &source[20..80];
    let original = &source[80..162];
    let header_len = usize::from(u16::from_le_bytes(original[5..7].try_into().unwrap()));
    let flags = u16::from_le_bytes(original[3..5].try_into().unwrap());
    let name_len = usize::from(u16::from_le_bytes(original[26..28].try_into().unwrap()));
    assert_eq!(
        (header_len, flags, &original[32..32 + name_len]),
        (50, 0x9424, b"b.txt".as_slice())
    );
    let mut header = original[..32].to_vec();
    header[2] = 0x7a;
    header[3..5].copy_from_slice(&(flags | 0x10).to_le_bytes());
    header[26..28].copy_from_slice(&3u16.to_le_bytes());
    header[28..32].copy_from_slice(&0u32.to_le_bytes());
    header.extend(b"CMT");
    header.extend(subdata);
    let salt_at = 32 + name_len;
    header.extend(&original[salt_at..salt_at + 8]);
    header.extend(time.unwrap_or(&original[salt_at + 8..header_len]));
    let new_size = u16::try_from(header.len()).unwrap();
    header[5..7].copy_from_slice(&new_size.to_le_bytes());
    let crc = xdzip_compression::crc32(&header[2..]) as u16;
    header[..2].copy_from_slice(&crc.to_le_bytes());
    let mut archive = source[..20].to_vec();
    archive.extend(parent);
    archive.extend(header);
    archive.extend(&original[header_len..]);
    archive.extend(&source[304..311]);
    archive
}
