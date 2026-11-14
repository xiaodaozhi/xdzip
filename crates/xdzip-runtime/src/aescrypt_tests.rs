//! Password admission and actual host publication for independently encoded AES Crypt inputs.
use super::*;
use crate::extraction::{ConflictPolicy, Control, Options, Password, Status};
use std::path::PathBuf;
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "xdzip-aes-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/aescrypt")
        .join(name)
}
fn password(profile: &str) -> Password {
    Password::new(match profile {
        "unicode" => "owned-資料-🔑".into(),
        "long" => "p".repeat(128) + "資料-🔑",
        _ => "owned-aes-fixture".into(),
    })
    .unwrap()
}
fn options(target: &Path) -> Options {
    std::fs::create_dir_all(target).unwrap();
    Options {
        target: target.to_string_lossy().into_owned(),
        all: true,
        selected: vec![],
        directory: String::new(),
        create_folder: false,
        shell_layout: 0,
        conflict: ConflictPolicy::Overwrite,
        allow_high_ratio: false,
    }
}
#[test]
fn all_versions_require_admission_then_use_retained_password_for_check_and_selected_export() {
    let cancel = Cancellation::default();
    for version in 0..4 {
        for (profile, expected) in [
            ("phrase", b"abracadabra abracadabra\n".repeat(6)),
            ("empty", vec![]),
            ("tail17", (0..17).collect()),
            ("unicode", b"abracadabra abracadabra\n".repeat(6)),
        ] {
            let scratch = Scratch::new();
            let path = fixture(&format!("v{version}-{profile}.aes"));
            assert!(matches!(
                Archive::open(&path, &cancel),
                Err(ScanError::PasswordRequired)
            ));
            assert!(matches!(
                Archive::open_with_password(
                    &path,
                    &cancel,
                    Some(Password::new("incorrect".into()).unwrap())
                ),
                Err(ScanError::WrongPassword | ScanError::PasswordOrIntegrity)
            ));
            let archive =
                Archive::open_with_password(&path, &cancel, Some(password(profile))).unwrap();
            assert_eq!(archive.format, xdzip_codec_api::Format::AesCrypt);
            assert!(!archive.update_allowed);
            assert_eq!(archive.rows.len(), 1);
            let row = &archive.rows[0];
            assert!(
                row.size_known
                    && row.encrypted
                    && row.password_supported
                    && row.payload_supported()
            );
            assert_eq!(row.size, expected.len() as u64);
            assert_eq!(row.unix_mode, 0);
            assert_eq!(row.packed_size, std::fs::metadata(&path).unwrap().len());
            let mut result = None;
            checking::run(
                archive.check_job(&cancel).unwrap(),
                1,
                &Control::default(),
                &mut |s| result = Some(s),
            );
            let result = result.unwrap();
            assert_eq!(result.status, Status::Succeeded, "{}", result.error);
            assert_eq!(result.verified_bytes, expected.len() as u64);
            let mut selected = options(&scratch.0);
            selected.all = false;
            selected.selected = vec![row.id.get().to_string()];
            let mut result = None;
            extraction::run(
                archive.extraction_job(selected, &cancel).unwrap(),
                2,
                &Control::default(),
                &mut |s| result = Some(s),
            );
            let result = result.unwrap();
            assert_eq!(result.status, Status::Succeeded, "{}", result.error);
            assert_eq!(result.committed_files, 1);
            assert_eq!(
                std::fs::read(
                    scratch
                        .0
                        .join(String::from_utf8_lossy(row.name_bytes()).as_ref())
                )
                .unwrap(),
                expected
            );
        }
    }
}
#[test]
fn extensions_have_no_path_authority_and_suffixes_do_not_select_the_format() {
    let scratch = Scratch::new();
    let cancel = Cancellation::default();
    for name in ["elsewhere.zip", "safe.AES", ".aes", "..aes"] {
        let source = scratch.0.join(name);
        std::fs::copy(fixture("v3-extensions.aes"), &source).unwrap();
        let archive =
            Archive::open_with_password(&source, &cancel, Some(password("ascii"))).unwrap();
        assert_eq!(archive.format, xdzip_codec_api::Format::AesCrypt);
        assert_eq!(
            archive.rows[0].name_bytes(),
            if name == "safe.AES" {
                b"safe".as_slice()
            } else {
                b"content"
            }
        );
        assert!(!archive.rows[0].warning);
    }
    let source = scratch.0.join("owned.txt.aes");
    std::fs::copy(fixture("v3-phrase.aes"), &source).unwrap();
    let archive = Archive::open_with_password(&source, &cancel, Some(password("ascii"))).unwrap();
    let preview = archive.preview(1, None, &cancel).unwrap();
    assert_eq!(
        preview.bytes.as_slice(),
        b"abracadabra abracadabra\n".repeat(6)
    );
}
#[test]
fn authenticated_aes_with_tar_marker_and_tar_member_named_aes_do_not_collide() {
    let cancel = Cancellation::default();
    let scratch = Scratch::new();
    let mut aes = std::fs::read(fixture("v3-phrase.aes")).unwrap();
    // Unauthenticated extension bytes may contain a TAR signature at its usual offset.
    let mut extension = vec![0; 512];
    extension[0] = b'X';
    extension[250..255].copy_from_slice(b"ustar");
    let mut framed = aes[..5].to_vec();
    framed.extend(512u16.to_be_bytes());
    framed.extend(extension);
    framed.extend(&aes[5..]);
    let path = scratch.0.join("cipher.tar");
    std::fs::write(&path, &framed).unwrap();
    assert!(xdzip_codec_tar::recognizes_header(&framed[..512]));
    assert_eq!(
        Archive::open_with_password(&path, &cancel, Some(password("ascii")))
            .unwrap()
            .format,
        xdzip_codec_api::Format::AesCrypt
    );
    let at = framed.len() - 1;
    framed[at] ^= 1;
    std::fs::write(&path, framed).unwrap();
    assert!(matches!(
        Archive::open_with_password(&path, &cancel, Some(password("ascii"))),
        Err(ScanError::Invalid)
    ));
    let mut tar = vec![0; 1536];
    tar[..3].copy_from_slice(b"AES");
    tar[100..108].copy_from_slice(b"0000644\0");
    tar[108..116].copy_from_slice(b"0000000\0");
    tar[116..124].copy_from_slice(b"0000000\0");
    tar[124..136].copy_from_slice(b"00000000000\0");
    tar[136..148].copy_from_slice(b"00000000000\0");
    tar[148..156].fill(b' ');
    tar[156] = b'0';
    tar[257..263].copy_from_slice(b"ustar\0");
    tar[263..265].copy_from_slice(b"00");
    let checksum: u32 = tar[..512].iter().map(|b| u32::from(*b)).sum();
    tar[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
    std::fs::write(&path, tar).unwrap();
    let archive = Archive::open(&path, &cancel).unwrap();
    assert_eq!(archive.format, xdzip_codec_api::Format::Tar);
    assert_eq!(archive.rows[0].name_bytes(), b"AES");
    aes[3] = 4;
    std::fs::write(&path, aes).unwrap();
    assert!(matches!(
        Archive::open(&path, &cancel),
        Err(ScanError::Unsupported)
    ));
}
#[test]
fn corrupt_cancelled_and_changed_sources_preserve_old_output() {
    let cancel = Cancellation::default();
    let scratch = Scratch::new();
    let source = scratch.0.join("owned.aes");
    let mut bytes = std::fs::read(fixture("v3-tail17.aes")).unwrap();
    let at = bytes.len() - 1;
    bytes[at] ^= 1;
    std::fs::write(&source, bytes).unwrap();
    assert!(matches!(
        Archive::open_with_password(&source, &cancel, Some(password("ascii"))),
        Err(ScanError::Invalid)
    ));
    std::fs::copy(fixture("v3-tail17.aes"), &source).unwrap();
    let archive = Archive::open_with_password(&source, &cancel, Some(password("ascii"))).unwrap();
    let target = scratch.0.join("output");
    let opts = options(&target);
    std::fs::write(target.join("owned"), b"keep").unwrap();
    let control = Control::default();
    assert!(control.cancel());
    let mut result = None;
    extraction::run(
        archive.extraction_job(opts.clone(), &cancel).unwrap(),
        1,
        &control,
        &mut |s| result = Some(s),
    );
    assert_eq!(result.unwrap().status, Status::Cancelled);
    let job = archive.extraction_job(opts, &cancel).unwrap();
    std::fs::write(&source, b"changed").unwrap();
    let mut result = None;
    extraction::run(job, 2, &Control::default(), &mut |s| result = Some(s));
    let result = result.unwrap();
    assert_eq!(result.status, Status::Failed);
    assert_eq!(result.error, "source_changed");
    assert_eq!(std::fs::read(target.join("owned")).unwrap(), b"keep");
    assert_eq!(std::fs::read_dir(target).unwrap().count(), 1);
}
