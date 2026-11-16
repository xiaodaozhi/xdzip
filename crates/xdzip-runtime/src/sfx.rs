//! Owned SFX envelope. The host copies a pinned-location stub; no stub or payload is executed while creating.
use sha2::{Digest, Sha256};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use xdzip_codec_api::{Cancellation, MetadataCodec, ScanLimits};
use xdzip_codec_zip::ZipCodec;

/// Version marker compiled into the independently runnable Windows stub.
pub const STUB_MARKER: &[u8] = b"XDZIP OWNED ZIP SFX STUB VERSION 1";
const MAGIC: &[u8; 16] = b"XDZIP-SFX-V1\0\0\0\0";
const FOOTER: u64 = 64;
const MAX_STUB: u64 = 64 << 20;
const MAX_IMAGE: u64 = 24 << 30;

fn check(cancel: &Cancellation) -> Result<(), &'static str> {
    cancel.check().map_err(|_| "cancelled")
}
fn stub_valid(bytes: &[u8]) -> bool {
    if bytes.len() < 512
        || bytes.len() as u64 > MAX_STUB
        || !bytes.starts_with(b"MZ")
        || !bytes.windows(STUB_MARKER.len()).any(|b| b == STUB_MARKER)
    {
        return false;
    }
    let offset = u32::from_le_bytes(bytes[60..64].try_into().unwrap()) as usize;
    let Some(header) = bytes.get(offset..offset.saturating_add(26)) else {
        return false;
    };
    if &header[..4] != b"PE\0\0" || &header[4..6] != b"\x64\x86" || &header[24..26] != b"\x0b\x02" {
        return false;
    }
    let sections = u16::from_le_bytes(header[6..8].try_into().unwrap()) as usize;
    let optional = u16::from_le_bytes(header[20..22].try_into().unwrap()) as usize;
    if sections == 0 || sections > 96 || optional < 112 {
        return false;
    }
    let start = offset.saturating_add(24).saturating_add(optional);
    let Some(table) = bytes.get(start..start.saturating_add(sections * 40)) else {
        return false;
    };
    for section in table.as_chunks::<40>().0 {
        let size = u32::from_le_bytes(section[16..20].try_into().unwrap()) as u64;
        let at = u32::from_le_bytes(section[20..24].try_into().unwrap()) as u64;
        if size != 0 && (at < (start + sections * 40) as u64 || at + size > bytes.len() as u64) {
            return false;
        }
    }
    bytes.get(bytes.len().saturating_sub(64)..bytes.len().saturating_sub(48)) != Some(MAGIC)
}
/// Read only the product-owned sibling stub, never searching PATH or a user archive.
pub(crate) fn load_stub(cancel: &Cancellation) -> Result<Vec<u8>, &'static str> {
    if !cfg!(all(windows, target_arch = "x86_64")) {
        return Err("unsupported_feature");
    }
    let exe = std::env::current_exe().map_err(|_| "source_io")?;
    let directory = exe.parent().ok_or("source_io")?;
    let mut candidates = vec![directory.join("xdzip-sfx.exe")];
    candidates.push(directory.join("cli/xdzip-sfx.exe"));
    if matches!(
        directory.file_name().and_then(|n| n.to_str()),
        Some("cli" | "deps")
    ) {
        candidates.push(directory.parent().ok_or("source_io")?.join("xdzip-sfx.exe"));
    }
    let path = candidates
        .into_iter()
        .find(|p| p.is_file())
        .ok_or("sfx_stub_missing")?;
    check(cancel)?;
    let mut file = xdzip_io::open_regular_nofollow(&path).map_err(|_| "source_io")?;
    let size = file.metadata().map_err(|_| "source_io")?.len();
    if size > MAX_STUB {
        return Err("resource_limit");
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take(MAX_STUB + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "source_io")?;
    check(cancel)?;
    if size != bytes.len() as u64 || !stub_valid(&bytes) {
        return Err("sfx_stub_invalid");
    }
    Ok(bytes)
}
/// Finalize the EOCD comment while the complete output is still a private stage.
pub(crate) fn finish(
    stage: &mut xdzip_io::Stage,
    prefix: u64,
    cancel: &Cancellation,
) -> Result<u64, &'static str> {
    check(cancel)?;
    let end = stage.seek(SeekFrom::End(0)).map_err(|_| "output_io")?;
    if end < prefix + 22 || end + FOOTER > MAX_IMAGE {
        return Err("resource_limit");
    }
    stage.seek(SeekFrom::End(-2)).map_err(|_| "output_io")?;
    stage
        .write_all(&(FOOTER as u16).to_le_bytes())
        .map_err(|_| "output_io")?;
    stage.flush().map_err(|_| "output_io")?;
    let mut reader = stage.verification_reader().map_err(|_| "output_io")?;
    let digest = hash(&mut reader, end, cancel)?;
    let mut footer = Vec::from(MAGIC.as_slice());
    footer.extend(prefix.to_le_bytes());
    footer.extend((end - prefix).to_le_bytes());
    footer.extend(digest);
    stage.seek(SeekFrom::Start(end)).map_err(|_| "output_io")?;
    stage.write_all(&footer).map_err(|_| "output_io")?;
    stage.flush().map_err(|_| "output_io")?;
    Ok(end + FOOTER)
}
fn hash<R: Read + Seek>(
    source: &mut R,
    length: u64,
    cancel: &Cancellation,
) -> Result<[u8; 32], &'static str> {
    source.seek(SeekFrom::Start(0)).map_err(|_| "source_io")?;
    let mut remaining = length;
    let mut digest = Sha256::new();
    let mut buffer = vec![0; 128 << 10];
    while remaining > 0 {
        check(cancel)?;
        let take = remaining.min(buffer.len() as u64) as usize;
        source
            .read_exact(&mut buffer[..take])
            .map_err(|_| "source_io")?;
        digest.update(&buffer[..take]);
        remaining -= take as u64;
    }
    Ok(digest.finalize().into())
}
/// Validate the envelope and admitted ZIP profile without extracting or running payloads.
pub fn validate(path: &std::path::Path, cancel: &Cancellation) -> Result<(), &'static str> {
    let mut file = xdzip_io::open_regular_nofollow(path).map_err(|_| "source_io")?;
    validate_reader(&mut file, cancel)
}
pub(crate) fn validate_reader<R: Read + Seek>(
    source: &mut R,
    cancel: &Cancellation,
) -> Result<(), &'static str> {
    check(cancel)?;
    let length = source.seek(SeekFrom::End(0)).map_err(|_| "source_io")?;
    if !(512 + 22 + FOOTER..=MAX_IMAGE).contains(&length) {
        return Err("invalid_sfx");
    }
    let mut footer = [0; 64];
    source.seek(SeekFrom::End(-64)).map_err(|_| "source_io")?;
    source.read_exact(&mut footer).map_err(|_| "source_io")?;
    if &footer[..16] != MAGIC {
        return Err("invalid_sfx");
    }
    let prefix = u64::from_le_bytes(footer[16..24].try_into().unwrap());
    let zip = u64::from_le_bytes(footer[24..32].try_into().unwrap());
    if !(512..=MAX_STUB).contains(&prefix)
        || zip < 22
        || prefix.checked_add(zip) != Some(length - FOOTER)
    {
        return Err("invalid_sfx");
    }
    if hash(source, length - FOOTER, cancel)?.as_slice() != &footer[32..] {
        return Err("integrity_error");
    }
    source.seek(SeekFrom::Start(0)).map_err(|_| "source_io")?;
    let mut stub = vec![0; prefix as usize];
    source.read_exact(&mut stub).map_err(|_| "source_io")?;
    if !stub_valid(&stub) {
        return Err("invalid_sfx");
    }
    source
        .seek(SeekFrom::Start(length - FOOTER - 22))
        .map_err(|_| "source_io")?;
    let mut end = [0; 22];
    source.read_exact(&mut end).map_err(|_| "source_io")?;
    if &end[..4] != b"PK\x05\x06" || u16::from_le_bytes(end[20..22].try_into().unwrap()) != 64 {
        return Err("invalid_sfx");
    }
    // The complete ZIP scanner independently validates local and central spans.
    struct Source<'a, T>(&'a mut T);
    impl<T: Read> Read for Source<'_, T> {
        fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
            self.0.read(b)
        }
    }
    impl<T: Seek> Seek for Source<'_, T> {
        fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
            self.0.seek(p)
        }
    }
    impl<T: Read + Seek> xdzip_codec_api::ReadSource for Source<'_, T> {}
    let rows = ZipCodec
        .scan(&mut Source(source), cancel, ScanLimits::default())
        .map_err(|_| "invalid_sfx")?;
    if rows
        .iter()
        .any(|e| e.encrypted || e.link || e.warning || !matches!(e.method, 0 | 8))
    {
        return Err("unsupported_feature");
    }
    Ok(())
}

/// Default extraction destination shown to the user; it does not create or authorize a directory.
pub fn suggested_target(executable: &std::path::Path) -> PathBuf {
    let name = executable
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive");
    let stem = name
        .strip_suffix(".sfx.exe")
        .or_else(|| name.strip_suffix(".exe"))
        .unwrap_or(name);
    executable
        .parent()
        .unwrap_or(std::path::Path::new("."))
        .join(stem)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use xdzip_codec_api::encode::{
        CreationCodec, EncodeEntry, EncodeError, EncodeInput, EncodeSettings,
    };
    struct Input(Vec<EncodeEntry>);
    impl EncodeInput for Input {
        fn entries(&self) -> &[EncodeEntry] {
            &self.0
        }
        fn open(&self, _: usize) -> Result<Box<dyn Read + '_>, EncodeError> {
            Ok(Box::new(Cursor::new(b"owned payload")))
        }
    }
    fn frame() -> Vec<u8> {
        let mut stub = vec![0u8; 512];
        stub[..2].copy_from_slice(b"MZ");
        stub[60..64].copy_from_slice(&64u32.to_le_bytes());
        stub[64..68].copy_from_slice(b"PE\0\0");
        stub[68..70].copy_from_slice(&0x8664u16.to_le_bytes());
        stub[70..72].copy_from_slice(&1u16.to_le_bytes());
        stub[84..86].copy_from_slice(&112u16.to_le_bytes());
        stub[88..90].copy_from_slice(&0x20bu16.to_le_bytes());
        stub[216..220].copy_from_slice(&256u32.to_le_bytes());
        stub[220..224].copy_from_slice(&256u32.to_le_bytes());
        stub[260..260 + STUB_MARKER.len()].copy_from_slice(STUB_MARKER);
        assert!(stub_valid(&stub));
        let input = Input(vec![EncodeEntry {
            name: "file.txt".into(),
            directory: false,
            size: 13,
            modified_unix: 0,
        }]);
        let mut zip = vec![];
        ZipCodec
            .encode(
                &input,
                &mut zip,
                EncodeSettings {
                    method: "deflate",
                    level: 6,
                    solid: false,
                    encrypt_headers: false,
                    password: None,
                },
                &Cancellation::default(),
                &mut |_| {},
            )
            .unwrap();
        let len = zip.len();
        zip[len - 2..].copy_from_slice(&64u16.to_le_bytes());
        let mut bytes = stub;
        bytes.extend(zip);
        let digest: [u8; 32] = Sha256::digest(&bytes).into();
        bytes.extend(MAGIC);
        bytes.extend(512u64.to_le_bytes());
        bytes.extend((len as u64).to_le_bytes());
        bytes.extend(digest);
        bytes
    }
    #[test]
    fn envelope_preserves_zip_readability_and_rejects_damage_and_truncation() {
        let bytes = frame();
        validate_reader(&mut Cursor::new(&bytes), &Cancellation::default()).unwrap();
        let mut changed = bytes.clone();
        changed[530] ^= 1;
        assert_eq!(
            validate_reader(&mut Cursor::new(changed), &Cancellation::default()),
            Err("integrity_error")
        );
        for size in [0, 511, bytes.len() - 1] {
            assert!(
                validate_reader(&mut Cursor::new(&bytes[..size]), &Cancellation::default())
                    .is_err()
            );
        }
        let mut wrong = bytes.clone();
        let at = wrong.len() - 64;
        wrong[at + 16..at + 24].copy_from_slice(&513u64.to_le_bytes());
        assert_eq!(
            validate_reader(&mut Cursor::new(wrong), &Cancellation::default()),
            Err("invalid_sfx")
        );
        let cancelled = Cancellation::default();
        cancelled.cancel();
        assert_eq!(
            validate_reader(&mut Cursor::new(bytes), &cancelled),
            Err("cancelled")
        );
    }
    #[test]
    fn stub_and_suffix_do_not_accept_generic_executables() {
        assert!(!stub_valid(&vec![0; 512]));
        assert!(!xdzip_codec_api::registry::is_sfx_creation("program.exe"));
        assert!(xdzip_codec_api::registry::is_sfx_creation(
            "Archive.SFX.EXE"
        ));
        assert_eq!(
            suggested_target(std::path::Path::new("folder/owned.sfx.exe")),
            std::path::Path::new("folder/owned")
        );
    }
}
