// crates/xdzip-runtime/src/source_cab.rs

//! CAB adjacency is resolved by the host before a decoder sees any stream.
use crate::source::SourceVersion;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};
use xdzip_codec_api::{Cancellation, ScanError};

pub(crate) struct Cabinet {
    pub path: PathBuf,
    pub name: Vec<u8>,
    pub version: SourceVersion,
}
struct Header {
    set: u16,
    index: u16,
    prev: Option<Vec<u8>>,
    next: Option<Vec<u8>>,
}
fn leaf(raw: &[u8]) -> Result<String, ScanError> {
    let name = xdzip_codec_api::names::decode_volume_name(raw).ok_or(ScanError::Invalid)?;
    if xdzip_domain::path::PortablePath::parse(&name).is_none_or(|path| path.parts().len() != 1) {
        return Err(ScanError::Invalid);
    }
    Ok(name.to_lowercase())
}
fn zstring(file: &mut File, control: &Cancellation) -> Result<Vec<u8>, ScanError> {
    let mut result = Vec::new();
    for _ in 0..=4096 {
        control.check()?;
        let mut byte = [0];
        file.read_exact(&mut byte).map_err(|_| ScanError::Invalid)?;
        if byte[0] == 0 {
            return Ok(result);
        }
        if result.len() == 4096 {
            return Err(ScanError::Limit);
        }
        result.push(byte[0]);
    }
    Err(ScanError::Limit)
}
fn header(file: &mut File, control: &Cancellation) -> Result<Header, ScanError> {
    file.seek(SeekFrom::Start(0)).map_err(|_| ScanError::Io)?;
    let mut bytes = [0; 36];
    file.read_exact(&mut bytes)
        .map_err(|_| ScanError::Invalid)?;
    let u16_at = |at| u16::from_le_bytes([bytes[at], bytes[at + 1]]);
    let size = u32::from_le_bytes(bytes[8..12].try_into().map_err(|_| ScanError::Invalid)?) as u64;
    let flags = u16_at(30);
    if &bytes[..4] != b"MSCF"
        || bytes[24..26] != [3, 1]
        || flags & !7 != 0
        || size < 36
        || size > file.metadata().map_err(|_| ScanError::Io)?.len()
    {
        return Err(ScanError::Invalid);
    }
    if flags & 4 != 0 {
        let mut reserve = [0; 4];
        file.read_exact(&mut reserve)
            .map_err(|_| ScanError::Invalid)?;
        let count = u16::from_le_bytes([reserve[0], reserve[1]]) as u64;
        if file
            .stream_position()
            .map_err(|_| ScanError::Io)?
            .saturating_add(count)
            > size
        {
            return Err(ScanError::Invalid);
        }
        file.seek(SeekFrom::Current(count as i64))
            .map_err(|_| ScanError::Io)?;
    }
    let mut adjacent = |enabled: bool| -> Result<Option<Vec<u8>>, ScanError> {
        if !enabled {
            return Ok(None);
        }
        let name = zstring(file, control)?;
        leaf(&name)?;
        zstring(file, control)?; // Disk label is data, never a host path.
        Ok(Some(name))
    };
    let prev = adjacent(flags & 1 != 0)?;
    let next = adjacent(flags & 2 != 0)?;
    if file.stream_position().map_err(|_| ScanError::Io)? > size {
        return Err(ScanError::Invalid);
    }
    let index = u16_at(34);
    if index >= 1024 {
        return Err(ScanError::Limit);
    }
    if (index == 0) != prev.is_none() {
        return Err(ScanError::Invalid);
    }
    Ok(Header {
        set: u16_at(32),
        index,
        prev,
        next,
    })
}
fn load(path: &Path, control: &Cancellation) -> Result<(Header, SourceVersion), ScanError> {
    control.check()?;
    let metadata = std::fs::symlink_metadata(path).map_err(|_| ScanError::MissingVolume)?;
    if !metadata.is_file() || metadata.file_type().is_symlink() {
        return Err(ScanError::Io);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x400 != 0 {
            return Err(ScanError::Io);
        }
    }
    let mut file = xdzip_io::open_regular_readonly(path).map_err(|_| ScanError::Io)?;
    let before = SourceVersion::read(&file)?;
    let header = header(&mut file, control)?;
    if SourceVersion::read(&file)? != before {
        return Err(ScanError::SourceChanged);
    }
    Ok((header, before))
}
fn filename(path: &Path) -> Result<String, ScanError> {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(str::to_lowercase)
        .ok_or(ScanError::Invalid)
}
pub(crate) fn discover(path: &Path, control: &Cancellation) -> Result<Vec<Cabinet>, ScanError> {
    let (selected, version) = load(path, control)?;
    if selected.prev.is_none() && selected.next.is_none() {
        return Ok(vec![Cabinet {
            path: path.into(),
            name: path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or(ScanError::Invalid)?
                .as_bytes()
                .to_vec(),
            version,
        }]);
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut directory = BTreeMap::<String, Option<PathBuf>>::new();
    let mut directory_name_bytes = 0usize;
    for (count, item) in std::fs::read_dir(parent)
        .map_err(|_| ScanError::Io)?
        .enumerate()
    {
        control.check()?;
        if count >= 100_000 {
            return Err(ScanError::Limit);
        }
        let item = item.map_err(|_| ScanError::Io)?;
        if let Some(name) = item.file_name().to_str() {
            directory_name_bytes = directory_name_bytes
                .checked_add(name.len().saturating_mul(4))
                .filter(|bytes| *bytes <= 16 * 1024 * 1024)
                .ok_or(ScanError::Limit)?;
            directory
                .entry(name.to_lowercase())
                .and_modify(|path| *path = None)
                .or_insert(Some(PathBuf::from(item.file_name())));
        }
    }
    let resolve = |raw: &[u8]| -> Result<PathBuf, ScanError> {
        let name = directory
            .get(&leaf(raw)?)
            .ok_or(ScanError::MissingVolume)?
            .clone()
            .ok_or(ScanError::Invalid)?;
        Ok(parent.join(name))
    };
    let mut first = path.to_path_buf();
    let mut current = selected;
    let mut seen = BTreeSet::new();
    while let Some(prev) = &current.prev {
        control.check()?;
        if !seen.insert(filename(&first)?) {
            return Err(ScanError::Invalid);
        }
        let previous = resolve(prev)?;
        let (before, _) = load(&previous, control)?;
        if before.set != current.set
            || before.index.checked_add(1) != Some(current.index)
            || before.next.as_ref().map(|name| leaf(name)).transpose()? != Some(filename(&first)?)
        {
            return Err(ScanError::Invalid);
        }
        first = previous;
        current = before;
    }
    let mut result: Vec<Cabinet> = Vec::new();
    let mut next = first;
    let mut raw_name = next
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ScanError::Invalid)?
        .as_bytes()
        .to_vec();
    let set = current.set;
    let mut selected_seen = false;
    loop {
        control.check()?;
        if result.len() >= 1024 {
            return Err(ScanError::Limit);
        }
        let (current, version) = load(&next, control)?;
        if current.set != set
            || current.index as usize != result.len()
            || result
                .iter()
                .any(|part| part.version.same_identity(&version))
        {
            return Err(ScanError::Invalid);
        }
        if let Some(prev) = &current.prev {
            if filename(&resolve(prev)?)?
                != filename(&result.last().ok_or(ScanError::Invalid)?.path)?
            {
                return Err(ScanError::Invalid);
            }
            if result.len() == 1 {
                result[0].name = prev.clone();
            } else if !prev.eq_ignore_ascii_case(&result.last().ok_or(ScanError::Invalid)?.name) {
                return Err(ScanError::Invalid);
            }
        }
        selected_seen |= filename(&next)? == filename(path)?;
        result.push(Cabinet {
            path: next,
            name: raw_name,
            version,
        });
        let Some(name) = current.next else {
            break;
        };
        next = resolve(&name)?;
        raw_name = name;
    }
    if !selected_seen {
        return Err(ScanError::Invalid);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::Source;
    use xdzip_codec_api::ReadSource;
    struct Fixture(PathBuf);
    impl Fixture {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "xdzip-cab-source-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir(&path).unwrap();
            Self(path)
        }
        fn write(&self, name: &str, set: u16, index: u16, prev: Option<&str>, next: Option<&str>) {
            let mut bytes = vec![0; 36];
            bytes[..4].copy_from_slice(b"MSCF");
            bytes[24..26].copy_from_slice(&[3, 1]);
            bytes[30..32].copy_from_slice(
                &(u16::from(prev.is_some()) | u16::from(next.is_some()) << 1).to_le_bytes(),
            );
            bytes[32..34].copy_from_slice(&set.to_le_bytes());
            bytes[34..36].copy_from_slice(&index.to_le_bytes());
            for name in [prev, next].into_iter().flatten() {
                bytes.extend_from_slice(name.as_bytes());
                bytes.extend_from_slice(&[0, 0]);
            }
            let size = bytes.len() as u32;
            bytes[8..12].copy_from_slice(&size.to_le_bytes());
            std::fs::write(self.0.join(name), bytes).unwrap();
        }
        fn open(&self, name: &str) -> Result<Source, ScanError> {
            Source::open(&self.0.join(name), &Default::default())
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn arbitrary_cabinet_names_open_from_any_part_and_protect_all_parts() {
        let fixture = Fixture::new();
        fixture.write("开始.bin", 42, 0, None, Some("middle.any"));
        fixture.write("middle.any", 42, 1, Some("开始.bin"), Some("last.xyz"));
        fixture.write("last.xyz", 42, 2, Some("middle.any"), None);
        for entry in ["开始.bin", "middle.any", "last.xyz"] {
            let mut source = fixture.open(entry).unwrap();
            assert!(source.is_split());
            assert_eq!(
                source.volume_names().unwrap(),
                ["开始.bin", "middle.any", "last.xyz"].map(|name| name.as_bytes().to_vec())
            );
            assert_eq!(source.identities().unwrap().len(), 3);
            let mut bytes = [0; 4];
            source.read_exact(&mut bytes).unwrap();
            assert_eq!(&bytes, b"MSCF");
            source.fork().unwrap().validate().unwrap();
        }
        let source = fixture.open("middle.any").unwrap();
        fixture.write("last.xyz", 43, 2, Some("middle.any"), None);
        assert_eq!(source.validate(), Err(ScanError::SourceChanged));
        assert!(matches!(source.fork(), Err(ScanError::SourceChanged)));
    }
    #[test]
    fn cabinet_chain_missing_cycles_mismatches_and_escaping_references_fail_closed() {
        for next in [
            "../outside.cab",
            "nested/file.cab",
            "nested\\file.cab",
            "C:outside.cab",
            ".",
            "",
            "\u{202e}cab",
        ] {
            let fixture = Fixture::new();
            fixture.write("first.cab", 1, 0, None, Some(next));
            assert!(fixture.open("first.cab").is_err(), "{next}");
        }
        let fixture = Fixture::new();
        fixture.write("first.cab", 1, 0, None, Some("second.cab"));
        assert!(matches!(
            fixture.open("first.cab"),
            Err(ScanError::MissingVolume)
        ));
        fixture.write("second.cab", 2, 1, Some("first.cab"), None);
        assert!(matches!(fixture.open("first.cab"), Err(ScanError::Invalid)));
        fixture.write("second.cab", 1, 2, Some("first.cab"), None);
        assert!(matches!(
            fixture.open("second.cab"),
            Err(ScanError::Invalid)
        ));
        fixture.write("second.cab", 1, 1, Some("first.cab"), Some("first.cab"));
        assert!(matches!(fixture.open("first.cab"), Err(ScanError::Invalid)));
        fixture.write("second.cab", 1, 1024, Some("first.cab"), None);
        assert!(matches!(fixture.open("second.cab"), Err(ScanError::Limit)));
        let control = Cancellation::default();
        control.cancel();
        assert!(matches!(
            Source::open(&fixture.0.join("first.cab"), &control),
            Err(ScanError::Cancelled)
        ));
    }
    #[cfg(unix)]
    #[test]
    fn cabinet_links_and_identity_aliases_are_rejected() {
        let fixture = Fixture::new();
        fixture.write("first.cab", 1, 0, None, Some("second.cab"));
        fixture.write("real.cab", 1, 1, Some("first.cab"), None);
        std::os::unix::fs::symlink(fixture.0.join("real.cab"), fixture.0.join("second.cab"))
            .unwrap();
        assert!(fixture.open("first.cab").is_err());
        assert!(fixture.open("second.cab").is_err());
        std::fs::remove_file(fixture.0.join("second.cab")).unwrap();
        std::fs::hard_link(fixture.0.join("first.cab"), fixture.0.join("second.cab")).unwrap();
        assert!(matches!(fixture.open("first.cab"), Err(ScanError::Invalid)));
    }
}
