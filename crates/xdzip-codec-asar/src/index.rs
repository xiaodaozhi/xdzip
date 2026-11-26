use crate::json::{self, Value};
use std::io::SeekFrom;
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits, names};
use xdzip_domain::path::extraction_components as components;
const HEADER_LIMIT: u64 = 8 * 1024 * 1024;
const SOURCE_LIMIT: u64 = 24 * 1024 * 1024 * 1024;
const SAFE_INTEGER: u64 = 9_007_199_254_740_991;
fn header_words(bytes: &[u8]) -> Result<[u64; 4], ScanError> {
    let bytes: &[u8; 16] = bytes.try_into().map_err(|_| ScanError::Invalid)?;
    let words: [u64; 4] = std::array::from_fn(|at| {
        u64::from(u32::from_le_bytes(
            bytes[at * 4..at * 4 + 4].try_into().expect("fixed header"),
        ))
    });
    if words[0] != 4 {
        return Err(ScanError::Unsupported);
    }
    let [_, header, payload, json_size] = words;
    if header < 8
        || header != payload + 4
        || header & 3 != 0
        || payload != 4 + ((json_size + 3) & !3)
    {
        return Err(ScanError::Invalid);
    }
    Ok(words)
}
/// Recognize the coherent fixed Pickle envelope only; full schema and bounds remain mandatory.
pub fn recognizes_header(bytes: &[u8]) -> bool {
    header_words(bytes).is_ok()
}
/// Supported optional SHA-256 declaration, without application authentication.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Integrity {
    /// Whole-file digest.
    pub hash: [u8; 32],
    /// Fixed plaintext block size, independent of the reader's buffer size.
    pub block_size: u64,
    /// Ordered digests, including one empty-message digest for an empty file.
    pub blocks: Vec<[u8; 32]>,
}
/// One logical tree record in serialization order, including shared file ranges.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    /// Container-resolved UTF-8 spelling; never a host output grant.
    pub path: String,
    /// Escaped display text, independent of pathname admission.
    pub display: String,
    /// Explicit directory marker.
    pub directory: bool,
    /// Declared link, never followed by this reader.
    pub link: bool,
    /// External unpacked data, never opened by this reader.
    pub external: bool,
    /// Unsafe name or unsupported record profile.
    pub warning: bool,
    /// Declared size, zero for directories and unsupported links.
    pub size: u64,
    /// Authorized absolute stored offset, absent for directories/links/external data.
    pub offset: Option<u64>,
    /// Declarative executable flag, granting no mode restoration or execution.
    pub executable: bool,
    /// Optional corruption checks retained for payload verification.
    pub integrity: Option<Integrity>,
}
/// A completely admitted header, not verification of stored file contents.
pub struct Index {
    /// Serialization-order records; the structural root is omitted.
    pub records: Vec<Record>,
    /// Absolute start of the stored payload area.
    pub data_offset: u64,
    /// Frozen authorized physical span.
    pub length: u64,
    /// Actual source bytes consumed during metadata inspection.
    pub read_work: u64,
}
fn object(value: &Value) -> Result<&[(String, Value)], ScanError> {
    if let Value::Object(fields) = value {
        Ok(fields)
    } else {
        Err(ScanError::Invalid)
    }
}
fn field<'a>(fields: &'a [(String, Value)], name: &str) -> Option<&'a Value> {
    fields
        .iter()
        .find_map(|(key, value)| (key == name).then_some(value))
}
fn string(value: &Value) -> Result<&str, ScanError> {
    if let Value::String(text) = value {
        Ok(text)
    } else {
        Err(ScanError::Invalid)
    }
}
fn integer(value: &Value) -> Result<u64, ScanError> {
    if let Value::Integer(value) = value
        && *value <= SAFE_INTEGER
    {
        return Ok(*value);
    }
    Err(ScanError::Invalid)
}
fn boolean(value: Option<&Value>) -> Result<bool, ScanError> {
    match value {
        None => Ok(false),
        Some(Value::Bool(value)) => Ok(*value),
        _ => Err(ScanError::Invalid),
    }
}
fn allowed(fields: &[(String, Value)], keys: &[&str]) -> Result<(), ScanError> {
    if fields.iter().all(|(key, _)| keys.contains(&key.as_str())) {
        Ok(())
    } else {
        Err(ScanError::Unsupported)
    }
}
fn decimal(text: &str) -> Result<u64, ScanError> {
    if text.is_empty() || text.len() > 20 || !text.bytes().all(|b| b.is_ascii_digit()) {
        return Err(ScanError::Invalid);
    }
    text.bytes().try_fold(0u64, |value, digit| {
        value
            .checked_mul(10)
            .and_then(|n| n.checked_add(u64::from(digit - b'0')))
            .ok_or(ScanError::Invalid)
    })
}
fn digest(value: &Value) -> Result<[u8; 32], ScanError> {
    let text = string(value)?;
    if text.len() != 64 {
        return Err(ScanError::Invalid);
    }
    let mut result = [0; 32];
    for (part, byte) in text
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .zip(result.iter_mut())
    {
        let high = char::from(part[0]).to_digit(16).ok_or(ScanError::Invalid)?;
        let low = char::from(part[1]).to_digit(16).ok_or(ScanError::Invalid)?;
        *byte = (high * 16 + low) as u8;
    }
    Ok(result)
}
fn integrity(
    value: Option<&Value>,
    size: u64,
    hashes: &mut usize,
    cancel: &Cancellation,
) -> Result<Option<Integrity>, ScanError> {
    let Some(value) = value else {
        return Ok(None);
    };
    let fields = object(value)?;
    allowed(fields, &["algorithm", "hash", "blockSize", "blocks"])?;
    if string(field(fields, "algorithm").ok_or(ScanError::Invalid)?)? != "SHA256" {
        return Err(ScanError::Unsupported);
    }
    let hash = digest(field(fields, "hash").ok_or(ScanError::Invalid)?)?;
    let block_size = integer(field(fields, "blockSize").ok_or(ScanError::Invalid)?)?;
    if block_size == 0 {
        return Err(ScanError::Invalid);
    }
    let Some(Value::Array(blocks)) = field(fields, "blocks") else {
        return Err(ScanError::Invalid);
    };
    if blocks.len() as u64 != size.div_ceil(block_size).max(1) {
        return Err(ScanError::Invalid);
    }
    *hashes = hashes.checked_add(blocks.len()).ok_or(ScanError::Limit)?;
    if *hashes > 100_000 {
        return Err(ScanError::Limit);
    }
    let mut output = Vec::with_capacity(blocks.len());
    for value in blocks {
        cancel.check()?;
        output.push(digest(value)?);
    }
    Ok(Some(Integrity {
        hash,
        block_size,
        blocks: output,
    }))
}
struct Tree<'a> {
    records: Vec<Record>,
    raw_names: usize,
    display_names: usize,
    hashes: usize,
    data_offset: u64,
    length: u64,
    limits: ScanLimits,
    cancel: &'a Cancellation,
}
impl Tree<'_> {
    fn walk(
        &mut self,
        files: &Value,
        prefix: &str,
        depth: usize,
        unsafe_parent: bool,
        external_parent: bool,
    ) -> Result<(), ScanError> {
        for (name, value) in object(files)? {
            self.cancel.check()?;
            if depth >= 64 || self.records.len() >= self.limits.entries {
                return Err(ScanError::Limit);
            }
            let fields = object(value)?;
            let directory = field(fields, "files").is_some();
            let link = field(fields, "link").is_some();
            if directory && link {
                return Err(ScanError::Invalid);
            }
            if directory {
                allowed(fields, &["files", "unpacked"])?;
            } else if link {
                allowed(fields, &["link", "unpacked"])?;
                string(field(fields, "link").ok_or(ScanError::Invalid)?)?;
            } else {
                allowed(
                    fields,
                    &["size", "offset", "integrity", "unpacked", "executable"],
                )?;
            }
            let external = external_parent || boolean(field(fields, "unpacked"))?;
            let executable = boolean(field(fields, "executable"))?;
            let length = prefix
                .len()
                .checked_add(name.len())
                .and_then(|n| n.checked_add(usize::from(directory)))
                .ok_or(ScanError::Limit)?;
            self.raw_names = self.raw_names.checked_add(length).ok_or(ScanError::Limit)?;
            if length > self.limits.name_bytes || self.raw_names > self.limits.total_name_bytes {
                return Err(ScanError::Limit);
            }
            let path = format!("{prefix}{name}{}", if directory { "/" } else { "" });
            let display = names::display(&path);
            self.display_names = self
                .display_names
                .checked_add(display.len())
                .ok_or(ScanError::Limit)?;
            if self.display_names > self.limits.display_name_bytes {
                return Err(ScanError::Limit);
            }
            let unsafe_name = unsafe_parent
                || components(name.as_bytes(), false).is_none_or(|parts| parts.len() != 1)
                || components(path.as_bytes(), directory).is_none();
            let (size, offset, check) = if directory || link {
                (0, None, None)
            } else {
                let size = integer(field(fields, "size").ok_or(ScanError::Invalid)?)?;
                let offset = if external {
                    if field(fields, "offset").is_some() {
                        return Err(ScanError::Invalid);
                    }
                    None
                } else {
                    let relative =
                        decimal(string(field(fields, "offset").ok_or(ScanError::Invalid)?)?)?;
                    let start = self
                        .data_offset
                        .checked_add(relative)
                        .ok_or(ScanError::Invalid)?;
                    start
                        .checked_add(size)
                        .filter(|end| *end <= self.length)
                        .ok_or(ScanError::Invalid)?;
                    Some(start)
                };
                let check = integrity(
                    field(fields, "integrity"),
                    size,
                    &mut self.hashes,
                    self.cancel,
                )?;
                (size, offset, check)
            };
            self.records.push(Record {
                path: path.clone(),
                display,
                directory,
                link,
                external,
                warning: unsafe_name || link || external,
                size,
                offset,
                executable,
                integrity: check,
            });
            if directory {
                self.walk(
                    field(fields, "files").ok_or(ScanError::Invalid)?,
                    &path,
                    depth + 1,
                    unsafe_name,
                    external,
                )?;
            }
        }
        Ok(())
    }
    fn stored_ranges(&self) -> Result<(), ScanError> {
        let mut ranges = Vec::new();
        for record in &self.records {
            self.cancel.check()?;
            if let Some(offset) = record.offset
                && record.size != 0
            {
                ranges.push((offset, offset + record.size));
            }
        }
        ranges.sort_unstable();
        let mut position = self.data_offset;
        let mut previous = None;
        for (start, end) in ranges {
            self.cancel.check()?;
            if previous == Some((start, end)) {
                continue;
            }
            if start != position {
                return Err(ScanError::Invalid);
            }
            position = end;
            previous = Some((start, end));
        }
        if position != self.length {
            return Err(ScanError::Invalid);
        }
        Ok(())
    }
}
fn exact(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    mut bytes: &mut [u8],
    work: &mut u64,
    limit: u64,
) -> Result<(), ScanError> {
    while !bytes.is_empty() {
        cancel.check()?;
        let available = limit.checked_sub(*work).ok_or(ScanError::Limit)?;
        if available == 0 {
            return Err(ScanError::Limit);
        }
        let take = bytes.len().min(available.min(usize::MAX as u64) as usize);
        let n = source
            .read(&mut bytes[..take])
            .map_err(|error| ScanError::from_source_error(&error))?;
        if n > take {
            return Err(ScanError::Io);
        }
        if n == 0 {
            return Err(ScanError::Invalid);
        }
        *work += n as u64;
        bytes = &mut bytes[n..];
    }
    Ok(())
}
/// Admit the complete bounded ASAR metadata and all stored ranges before returning any rows.
pub fn inspect(
    source: &mut dyn ReadSource,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    cancel.check()?;
    if source
        .volume_lengths()
        .is_some_and(|volumes| volumes.len() != 1)
    {
        return Err(ScanError::Unsupported);
    }
    let length = source
        .seek(SeekFrom::End(0))
        .map_err(|error| ScanError::from_source_error(&error))?;
    if length > SOURCE_LIMIT {
        return Err(ScanError::Limit);
    }
    if source
        .seek(SeekFrom::Start(0))
        .map_err(|error| ScanError::from_source_error(&error))?
        != 0
    {
        return Err(ScanError::SourceChanged);
    }
    let mut work = 0;
    let mut words = [0; 16];
    exact(source, cancel, &mut words, &mut work, limits.read_bytes)?;
    let [_, header, _, json_size] = header_words(&words)?;
    if json_size > HEADER_LIMIT || header > HEADER_LIMIT + 8 {
        return Err(ScanError::Limit);
    }
    let data_offset = 8 + header;
    if data_offset > length {
        return Err(ScanError::Invalid);
    }
    if data_offset > limits.read_bytes {
        return Err(ScanError::Limit);
    }
    let mut bytes = vec![0; (header - 8) as usize];
    exact(source, cancel, &mut bytes, &mut work, limits.read_bytes)?;
    if bytes[json_size as usize..].iter().any(|byte| *byte != 0) {
        return Err(ScanError::Invalid);
    }
    let root = json::parse(&bytes[..json_size as usize], cancel)?;
    let fields = object(&root)?;
    allowed(fields, &["files"])?;
    let mut tree = Tree {
        records: Vec::new(),
        raw_names: 0,
        display_names: 0,
        hashes: 0,
        data_offset,
        length,
        limits,
        cancel,
    };
    tree.walk(
        field(fields, "files").ok_or(ScanError::Invalid)?,
        "",
        0,
        false,
        false,
    )?;
    tree.stored_ranges()?;
    cancel.check()?;
    Ok(Index {
        records: tree.records,
        data_offset,
        length,
        read_work: work,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn fixture(name: &str) -> Vec<u8> {
        std::fs::read(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../tests/fixtures/asar")
                .join(name),
        )
        .unwrap()
    }
    #[test]
    fn independent_headers_preserve_tree_order_shared_payloads_and_unsupported_records() {
        let cancel = Cancellation::default();
        for (name, count) in [
            ("ordinary.asar", 6),
            ("without-integrity.asar", 6),
            ("shared.asar", 3),
            ("executable.asar", 1),
            ("external.asar", 1),
            ("link.asar", 2),
            ("empty.asar", 0),
        ] {
            let bytes = fixture(name);
            let index = inspect(&mut Cursor::new(&bytes), &cancel, ScanLimits::default()).unwrap();
            assert_eq!(index.records.len(), count);
            assert_eq!(index.length, bytes.len() as u64);
            assert_eq!(index.read_work, index.data_offset);
            if name == "ordinary.asar" || name == "without-integrity.asar" {
                assert_eq!(
                    index
                        .records
                        .iter()
                        .map(|r| r.path.as_str())
                        .collect::<Vec<_>>(),
                    [
                        "assets/",
                        "assets/资料.txt",
                        "empty-dir/",
                        "hello.txt",
                        "empty.bin",
                        "copy.txt"
                    ]
                );
                assert_eq!(index.records[3].offset, index.records[5].offset);
                assert!(index.records.iter().all(|r| !r.warning));
                assert_eq!(
                    index.records[3].integrity.is_some(),
                    name == "ordinary.asar"
                );
            }
            if name == "shared.asar" {
                assert_eq!(index.records[0].offset, index.records[1].offset);
                assert_eq!(index.records[0].integrity.as_ref().unwrap().blocks.len(), 3);
            }
            if name == "executable.asar" {
                assert!(index.records[0].executable && !index.records[0].warning);
            }
            if name == "external.asar" {
                assert!(index.records[0].external && index.records[0].warning);
                assert_eq!(index.records[0].offset, None);
            }
            if name == "link.asar" {
                assert!(index.records[0].link && index.records[0].warning);
            }
        }
    }
    fn wrap(json: &str, body: &[u8]) -> Vec<u8> {
        let padded = (json.len() + 3) & !3;
        let mut bytes = Vec::new();
        for word in [4, 8 + padded as u32, 4 + padded as u32, json.len() as u32] {
            bytes.extend(word.to_le_bytes());
        }
        bytes.extend(json.as_bytes());
        bytes.resize(16 + padded, 0);
        bytes.extend(body);
        bytes
    }
    #[test]
    fn every_truncated_span_padding_and_pickle_word_is_rejected_without_partial_rows() {
        let cancel = Cancellation::default();
        let bytes = fixture("ordinary.asar");
        for end in 0..bytes.len() {
            assert!(
                inspect(
                    &mut Cursor::new(&bytes[..end]),
                    &cancel,
                    ScanLimits::default()
                )
                .is_err(),
                "prefix {end}"
            );
        }
        for at in [0, 4, 8, 12] {
            let mut bad = bytes.clone();
            bad[at] ^= 1;
            assert!(
                inspect(&mut Cursor::new(bad), &cancel, ScanLimits::default()).is_err(),
                "word {at}"
            );
        }
        let mut bad = bytes;
        bad.push(0);
        assert_eq!(
            inspect(&mut Cursor::new(bad), &cancel, ScanLimits::default()).err(),
            Some(ScanError::Invalid)
        );
        let mut bytes = wrap("{\"files\":{}} ", b"");
        bytes[29] = 1;
        assert_eq!(
            inspect(&mut Cursor::new(bytes), &cancel, ScanLimits::default()).err(),
            Some(ScanError::Invalid)
        );
    }
    #[test]
    fn schema_arithmetic_overlaps_unknown_profiles_and_original_components_are_explicit() {
        let cancel = Cancellation::default();
        for (json, body) in [
            (
                r#"{"files":{"x":{"size":1,"offset":"18446744073709551616"}}}"#,
                b"x".as_slice(),
            ),
            (
                r#"{"files":{"x":{"size":9007199254740992,"offset":"0"}}}"#,
                b"x",
            ),
            (
                r#"{"files":{"x":{"size":3,"offset":"0"},"y":{"size":3,"offset":"1"}}}"#,
                b"abcd",
            ),
            (r#"{"files":{"x":{"size":1,"offset":"1"}}}"#, b"xx"),
            (r#"{"files":{"x":{"size":1,"offset":"0"}}}"#, b"xx"),
            (
                r#"{"files":{"x":{"size":1,"offset":"0","unpacked":true}}}"#,
                b"x",
            ),
            (r#"{"files":{"x":{"size":1,"offset":0}}}"#, b"x"),
            (
                r#"{"files":{"x":{"size":1,"offset":"0","executable":1}}}"#,
                b"x",
            ),
            (r#"{"files":{"x":{"files":{},"link":"x"}}}"#, b""),
        ] {
            assert_eq!(
                inspect(
                    &mut Cursor::new(wrap(json, body)),
                    &cancel,
                    ScanLimits::default()
                )
                .err(),
                Some(ScanError::Invalid),
                "{json}"
            );
        }
        assert_eq!(
            inspect(
                &mut Cursor::new(wrap(r#"{"files":{},"unknown":true}"#, b"")),
                &cancel,
                ScanLimits::default()
            )
            .err(),
            Some(ScanError::Unsupported)
        );
        for name in ["/absolute", "..", "one/two", "CON", "back\\\\slash"] {
            let json = format!(r#"{{"files":{{"{name}":{{"size":1,"offset":"0"}}}}}}"#);
            let index = inspect(
                &mut Cursor::new(wrap(&json, b"x")),
                &cancel,
                ScanLimits::default(),
            )
            .unwrap();
            assert!(index.records[0].warning, "{name}");
        }
    }
    #[test]
    fn exact_metadata_work_and_aggregate_index_limits_do_not_expand_per_directory() {
        let cancel = Cancellation::default();
        let bytes = fixture("ordinary.asar");
        let index = inspect(&mut Cursor::new(&bytes), &cancel, ScanLimits::default()).unwrap();
        let limits = ScanLimits {
            entries: 6,
            read_bytes: index.data_offset,
            total_name_bytes: index.records.iter().map(|r| r.path.len()).sum(),
            display_name_bytes: index.records.iter().map(|r| r.display.len()).sum(),
            ..ScanLimits::default()
        };
        inspect(&mut Cursor::new(&bytes), &cancel, limits).unwrap();
        for limits in [
            ScanLimits {
                entries: 5,
                ..limits
            },
            ScanLimits {
                read_bytes: limits.read_bytes - 1,
                ..limits
            },
            ScanLimits {
                total_name_bytes: limits.total_name_bytes - 1,
                ..limits
            },
            ScanLimits {
                display_name_bytes: limits.display_name_bytes - 1,
                ..limits
            },
        ] {
            assert_eq!(
                inspect(&mut Cursor::new(&bytes), &cancel, limits).err(),
                Some(ScanError::Limit)
            );
        }
        cancel.cancel();
        assert_eq!(
            inspect(&mut Cursor::new(&bytes), &cancel, limits).err(),
            Some(ScanError::Cancelled)
        );
    }
}
