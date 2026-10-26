use super::ReadSource;

#[test]
fn portable_sources_do_not_invent_optional_os_password_projection() {
    let source = std::io::Cursor::new(Vec::<u8>::new());
    let mut wide = [0u16; 512];
    wide[..3].copy_from_slice(&[b'A' as u16, 0, b'B' as u16]);
    let mut wide_output = [0xa5u16; 512];
    let mut narrow_output = [0xa5u8; 512];
    assert_eq!(source.oem_to_ansi_wide(&wide, &mut wide_output), Ok(None));
    assert_eq!(source.wide_to_ansi(&wide, &mut narrow_output), Ok(None));
    assert_eq!(
        source.wide_to_local_prefix(&[65, 0, 66], &mut narrow_output),
        Ok(None)
    );
    assert!(wide_output.iter().all(|value| *value == 0xa5));
    assert!(narrow_output.iter().all(|value| *value == 0xa5));
}
