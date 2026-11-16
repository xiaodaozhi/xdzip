use crate::formats::detect;
use std::io::Cursor;
use xdzip_codec_api::{Cancellation, Format};

#[test]
fn runtime_detects_compound_file_signature_without_extension() {
    let mut bytes = vec![0u8; 512];
    bytes[..8].copy_from_slice(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);
    let cancel = Cancellation::default();
    let mut source = Cursor::new(bytes);
    assert_eq!(detect(&mut source, &cancel).unwrap(), Format::Compound);
    assert_eq!(Format::Compound.descriptor().container, "cfb");
    assert_eq!(Format::Compound.descriptor().extensions, [".msi"]);
}
