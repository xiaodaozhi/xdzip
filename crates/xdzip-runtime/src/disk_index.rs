//! Private, session-only metadata and external query sorting. No archive payload is cached.
use crate::navigation::{self, Directory};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BinaryHeap};
use std::io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::sync::Mutex;
use xdzip_codec_api::{Cancellation, Entry, Format, ScanError, names::NameEncoding};
use xdzip_io::{ScratchSpace, Stage};

pub(crate) const HOT_BYTES: usize = 64 * 1024 * 1024;
const SORT_BYTES: usize = 8 * 1024 * 1024;
const DISK_BYTES: u64 = 512 * 1024 * 1024;
const NAV: u64 = 1 << 63;

fn io<T>(result: std::io::Result<T>) -> Result<T, ScanError> {
    result.map_err(|_| ScanError::Io)
}
fn scratch<T>(result: Result<T, xdzip_io::OutputError>) -> Result<T, ScanError> {
    result.map_err(|_| ScanError::Io)
}

pub(crate) struct Builder {
    writer: BufWriter<Stage>,
    offsets: Vec<u64>,
    bytes: u64,
    space: ScratchSpace,
}
impl Builder {
    pub fn new() -> Result<Self, ScanError> {
        let mut space = scratch(ScratchSpace::new())?;
        Ok(Self {
            writer: BufWriter::with_capacity(256 * 1024, scratch(space.file())?),
            offsets: Vec::new(),
            bytes: 0,
            space,
        })
    }
    pub fn push(&mut self, entry: &Entry) -> Result<(), ScanError> {
        if self.offsets.len() >= xdzip_codec_api::MAX_INDEX_ENTRIES
            || entry.id.get() != self.offsets.len() as u64 + 1
        {
            return Err(ScanError::Limit);
        }
        let bytes = encode(entry);
        let next = self
            .bytes
            .checked_add(4 + bytes.len() as u64)
            .filter(|n| *n <= DISK_BYTES)
            .ok_or(ScanError::Limit)?;
        io(self.writer.write_all(&(bytes.len() as u32).to_le_bytes()))?;
        io(self.writer.write_all(&bytes))?;
        self.offsets.push(self.bytes);
        self.bytes = next;
        Ok(())
    }
    pub fn finish(self) -> Result<DiskIndex, ScanError> {
        let stage = self.writer.into_inner().map_err(|_| ScanError::Io)?;
        Ok(DiskIndex {
            data: Mutex::new(stage),
            offsets: self.offsets,
            bytes: self.bytes,
            _space: self.space,
        })
    }
}
pub(crate) struct DiskIndex {
    data: Mutex<Stage>,
    offsets: Vec<u64>,
    bytes: u64,
    _space: ScratchSpace,
}
impl DiskIndex {
    pub fn len(&self) -> usize {
        self.offsets.len()
    }
    pub fn bytes(&self) -> u64 {
        self.bytes
    }
    pub fn get(&self, id: u64) -> Result<Entry, ScanError> {
        let at = id
            .checked_sub(1)
            .and_then(|n| usize::try_from(n).ok())
            .and_then(|n| self.offsets.get(n))
            .ok_or(ScanError::Invalid)?;
        let mut file = self.data.lock().map_err(|_| ScanError::Io)?;
        io(file.seek(SeekFrom::Start(*at)))?;
        let entry = decode(&mut *file)?;
        if entry.id.get() != id {
            return Err(ScanError::Invalid);
        }
        Ok(entry)
    }
    pub fn each(
        &self,
        control: &Cancellation,
        visit: &mut dyn FnMut(&Entry) -> Result<(), ScanError>,
    ) -> Result<(), ScanError> {
        let mut file = self.data.lock().map_err(|_| ScanError::Io)?;
        io(file.seek(SeekFrom::Start(0)))?;
        let mut reader = BufReader::with_capacity(256 * 1024, &mut *file);
        for id in 1..=self.len() as u64 {
            control.check()?;
            let entry = decode(&mut reader)?;
            if entry.id.get() != id {
                return Err(ScanError::Invalid);
            }
            visit(&entry)?;
        }
        Ok(())
    }
    pub fn directories(&self, control: &Cancellation) -> Result<Vec<Directory>, ScanError> {
        navigation::build_from(&mut |visit| self.each(control, visit), control)
    }
    /// Sorting runs retain at most 8 MiB of keys. The final query contains only row tokens.
    pub fn query(
        &self,
        prefix: &[u8],
        text: &str,
        sort: u8,
        directories: &[Directory],
        control: &Cancellation,
    ) -> Result<DiskQuery, ScanError> {
        if text.len() > 1024 || sort > 12 {
            return Err(ScanError::Limit);
        }
        let mut space = scratch(ScratchSpace::new())?;
        let mut runs = Vec::new();
        let mut batch = Vec::new();
        let mut key_bytes = 0;
        let mut disk_bytes = 0;
        let needle = text.to_lowercase();
        let mut virtuals: BTreeMap<&[u8], &Directory> = directories
            .iter()
            .filter(|d| parent(d.path.as_bytes()) == Some(prefix))
            .map(|d| (d.path.as_bytes(), d))
            .collect();
        self.each(control, &mut |entry| {
            if entry.directory && !entry.warning {
                virtuals.remove(entry.name_bytes());
            }
            let name = entry.name_bytes();
            let matches = if text.is_empty() {
                parent(name) == Some(prefix)
            } else {
                name.starts_with(prefix)
                    && (prefix.is_empty() || name != prefix)
                    && entry.display_name.to_lowercase().contains(&needle)
            };
            if matches {
                let key = SortRow::physical(entry, sort);
                key_bytes += key.text.len() + std::mem::size_of::<SortRow>();
                batch.push(key);
                if key_bytes >= SORT_BYTES {
                    flush_run(
                        &mut batch,
                        &mut runs,
                        &mut space,
                        sort,
                        &mut disk_bytes,
                        control,
                    )?;
                    key_bytes = 0;
                }
            }
            Ok(())
        })?;
        if text.is_empty() {
            for node in virtuals.into_values() {
                control.check()?;
                batch.push(SortRow {
                    token: NAV | node.id,
                    text: if sort <= 2 {
                        node.path.clone()
                    } else {
                        String::new()
                    },
                    number: 0,
                    group: 0,
                });
            }
        }
        flush_run(
            &mut batch,
            &mut runs,
            &mut space,
            sort,
            &mut disk_bytes,
            control,
        )?;
        let mut result = BufWriter::with_capacity(256 * 1024, scratch(space.file())?);
        let mut readers: Vec<_> = runs
            .iter_mut()
            .map(|run| {
                io(run.seek(SeekFrom::Start(0)))?;
                Ok(BufReader::with_capacity(64 * 1024, run))
            })
            .collect::<Result<_, ScanError>>()?;
        let mut heap = BinaryHeap::new();
        for (run, reader) in readers.iter_mut().enumerate() {
            if let Some(row) = read_sort(reader)? {
                heap.push(Head { row, run, sort });
            }
        }
        let mut total = 0;
        while let Some(head) = heap.pop() {
            control.check()?;
            io(result.write_all(&head.row.token.to_le_bytes()))?;
            total += 1;
            if total > xdzip_codec_api::MAX_INDEX_ENTRIES {
                return Err(ScanError::Limit);
            }
            if let Some(row) = read_sort(&mut readers[head.run])? {
                heap.push(Head {
                    row,
                    run: head.run,
                    sort,
                });
            }
        }
        drop(readers);
        drop(runs);
        let order = result.into_inner().map_err(|_| ScanError::Io)?;
        Ok(DiskQuery {
            order: Mutex::new(order),
            total,
            _space: space,
        })
    }
}
pub(crate) struct DiskQuery {
    order: Mutex<Stage>,
    pub total: usize,
    _space: ScratchSpace,
}
impl DiskQuery {
    pub fn page(&self, offset: usize) -> Result<Vec<u64>, ScanError> {
        if offset > self.total {
            return Err(ScanError::Limit);
        }
        let mut file = self.order.lock().map_err(|_| ScanError::Io)?;
        io(file.seek(SeekFrom::Start(offset as u64 * 8)))?;
        (offset..self.total.min(offset + 128))
            .map(|_| read_u64(&mut *file))
            .collect()
    }
}
pub(crate) fn navigation_token(token: u64) -> Option<usize> {
    (token & NAV != 0).then(|| ((token & !NAV) - 1) as usize)
}
fn parent(name: &[u8]) -> Option<&[u8]> {
    let name = name.strip_suffix(b"/").unwrap_or(name);
    if name.is_empty() || name.ends_with(b"/") {
        return None;
    }
    Some(
        name.iter()
            .rposition(|b| *b == b'/')
            .map_or(&[][..], |at| &name[..=at]),
    )
}

#[derive(Eq, PartialEq)]
struct SortRow {
    token: u64,
    text: String,
    number: u64,
    group: u8,
}
impl SortRow {
    fn physical(e: &Entry, sort: u8) -> Self {
        Self {
            token: e.id.get(),
            text: match sort {
                1 | 2 => e.display_name.clone(),
                9 | 10 => crate::entry_type(e),
                _ => String::new(),
            },
            number: match sort {
                3 | 4 => e.size,
                5 | 6 => e.packed_size,
                7 | 8 => u64::from(e.method),
                11 | 12 => u64::from(e.modified_dos),
                _ => 0,
            },
            group: if matches!(sort, 3 | 4) {
                u8::from(!e.size_known)
            } else if matches!(sort, 9 | 10) {
                u8::from(!e.directory)
            } else {
                0
            },
        }
    }
    fn compare(&self, other: &Self, sort: u8) -> Ordering {
        let primary = if matches!(sort, 1 | 2 | 9 | 10) {
            self.text.cmp(&other.text)
        } else {
            self.number.cmp(&other.number)
        };
        let primary = if sort != 0 && sort.is_multiple_of(2) {
            primary.reverse()
        } else {
            primary
        };
        self.group
            .cmp(&other.group)
            .then(primary)
            .then(self.token.cmp(&other.token))
    }
}
#[derive(Eq, PartialEq)]
struct Head {
    row: SortRow,
    run: usize,
    sort: u8,
}
impl Ord for Head {
    fn cmp(&self, other: &Self) -> Ordering {
        self.row
            .compare(&other.row, self.sort)
            .reverse()
            .then(other.run.cmp(&self.run))
    }
}
impl PartialOrd for Head {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
fn flush_run(
    batch: &mut Vec<SortRow>,
    runs: &mut Vec<Stage>,
    space: &mut ScratchSpace,
    sort: u8,
    bytes: &mut u64,
    control: &Cancellation,
) -> Result<(), ScanError> {
    if batch.is_empty() {
        return Ok(());
    }
    if runs.len() >= 128 {
        return Err(ScanError::Limit);
    }
    control.check()?;
    batch.sort_unstable_by(|a, b| a.compare(b, sort));
    control.check()?;
    let mut file = BufWriter::with_capacity(256 * 1024, scratch(space.file())?);
    for row in batch.drain(..) {
        *bytes = bytes
            .checked_add(21 + row.text.len() as u64)
            .filter(|n| *n <= DISK_BYTES - 8 * xdzip_codec_api::MAX_INDEX_ENTRIES as u64)
            .ok_or(ScanError::Limit)?;
        io(file.write_all(&row.token.to_le_bytes()))?;
        io(file.write_all(&[row.group]))?;
        io(file.write_all(&row.number.to_le_bytes()))?;
        io(file.write_all(&(row.text.len() as u32).to_le_bytes()))?;
        io(file.write_all(row.text.as_bytes()))?;
    }
    runs.push(file.into_inner().map_err(|_| ScanError::Io)?);
    Ok(())
}
fn read_sort(reader: &mut impl Read) -> Result<Option<SortRow>, ScanError> {
    let mut first = [0];
    if io(reader.read(&mut first))? == 0 {
        return Ok(None);
    }
    let mut token = [0; 8];
    token[0] = first[0];
    io(reader.read_exact(&mut token[1..]))?;
    let mut group = [0];
    io(reader.read_exact(&mut group))?;
    let number = read_u64(reader)?;
    let mut len = [0; 4];
    io(reader.read_exact(&mut len))?;
    let len = u32::from_le_bytes(len) as usize;
    if len > 32 * 1024 {
        return Err(ScanError::Invalid);
    }
    let mut text = vec![0; len];
    io(reader.read_exact(&mut text))?;
    Ok(Some(SortRow {
        token: u64::from_le_bytes(token),
        group: group[0],
        number,
        text: String::from_utf8(text).map_err(|_| ScanError::Invalid)?,
    }))
}
fn read_u64(r: &mut impl Read) -> Result<u64, ScanError> {
    let mut b = [0; 8];
    io(r.read_exact(&mut b))?;
    Ok(u64::from_le_bytes(b))
}
fn encode(e: &Entry) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&e.id.get().to_le_bytes());
    out.extend_from_slice(&e.size.to_le_bytes());
    out.extend_from_slice(&e.packed_size.to_le_bytes());
    out.extend_from_slice(&e.unix_mode.to_le_bytes());
    out.extend_from_slice(&e.modified_dos.to_le_bytes());
    out.extend_from_slice(&e.method.to_le_bytes());
    out.push(
        u8::from(e.directory)
            | u8::from(e.link) << 1
            | u8::from(e.size_known) << 2
            | u8::from(e.encrypted) << 3
            | u8::from(e.password_supported) << 4
            | u8::from(e.warning) << 5
            | u8::from(e.decoded_name.is_some()) << 6,
    );
    out.push(match e.name_encoding {
        NameEncoding::Utf8 => 0,
        NameEncoding::UnmarkedUtf8 => 1,
        NameEncoding::UnicodePath => 2,
        NameEncoding::Legacy(_) => 3,
        NameEncoding::Unresolved => 4,
    });
    for bytes in [
        e.raw_name.as_slice(),
        e.decoded_name.as_deref().unwrap_or_default().as_bytes(),
        e.display_name.as_bytes(),
        match e.name_encoding {
            NameEncoding::Legacy(label) => label.as_bytes(),
            _ => b"",
        },
    ] {
        out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
        out.extend_from_slice(bytes);
    }
    out
}
fn decode(reader: &mut impl Read) -> Result<Entry, ScanError> {
    let mut len = [0; 4];
    io(reader.read_exact(&mut len))?;
    let len = u32::from_le_bytes(len) as usize;
    if !(52..=64 * 1024).contains(&len) {
        return Err(ScanError::Invalid);
    }
    let mut bytes = vec![0; len];
    io(reader.read_exact(&mut bytes))?;
    let mut r = std::io::Cursor::new(bytes);
    let id = read_u64(&mut r)?
        .try_into()
        .map_err(|_| ScanError::Invalid)?;
    let size = read_u64(&mut r)?;
    let packed_size = read_u64(&mut r)?;
    let mut fixed = [0; 12];
    io(r.read_exact(&mut fixed))?;
    let unix_mode = u32::from_le_bytes(fixed[..4].try_into().unwrap());
    let modified_dos = u32::from_le_bytes(fixed[4..8].try_into().unwrap());
    let method = u16::from_le_bytes(fixed[8..10].try_into().unwrap());
    let flags = fixed[10];
    let encoding = fixed[11];
    if flags & !127 != 0 {
        return Err(ScanError::Invalid);
    }
    let mut fields = Vec::new();
    for max in [4096, 4096, 32 * 1024, 64] {
        let mut field_len = [0; 4];
        io(r.read_exact(&mut field_len))?;
        let n = u32::from_le_bytes(field_len) as usize;
        if n > max || r.position() + n as u64 > r.get_ref().len() as u64 {
            return Err(ScanError::Invalid);
        }
        let mut value = vec![0; n];
        io(r.read_exact(&mut value))?;
        fields.push(value);
    }
    if r.position() != r.get_ref().len() as u64 {
        return Err(ScanError::Invalid);
    }
    let label = std::str::from_utf8(&fields[3]).map_err(|_| ScanError::Invalid)?;
    let name_encoding = match encoding {
        0 => NameEncoding::Utf8,
        1 => NameEncoding::UnmarkedUtf8,
        2 => NameEncoding::UnicodePath,
        3 => NameEncoding::Legacy(
            xdzip_codec_api::names::encoding_label(label).ok_or(ScanError::Invalid)?,
        ),
        4 => NameEncoding::Unresolved,
        _ => return Err(ScanError::Invalid),
    };
    let display_name =
        String::from_utf8(std::mem::take(&mut fields[2])).map_err(|_| ScanError::Invalid)?;
    let decoded_name = if flags & 64 == 0 {
        None
    } else {
        Some(String::from_utf8(std::mem::take(&mut fields[1])).map_err(|_| ScanError::Invalid)?)
    };
    Ok(Entry {
        format: Format::Zip,
        id,
        size,
        packed_size,
        unix_mode,
        modified_dos,
        method,
        raw_name: std::mem::take(&mut fields[0]),
        decoded_name,
        display_name,
        name_encoding,
        directory: flags & 1 != 0,
        link: flags & 2 != 0,
        size_known: flags & 4 != 0,
        encrypted: flags & 8 != 0,
        password_supported: flags & 16 != 0,
        warning: flags & 32 != 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Archive,
        query::{QueryRequest, QueryRow},
    };
    #[test]
    fn disk_queries_match_memory_for_all_orders_and_scopes() {
        let control = Cancellation::default();
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/zip/normal.zip");
        let memory = Archive::open(&path, &control).unwrap();
        let mut archive = Archive::open(&path, &control).unwrap();
        let mut builder = Builder::new().unwrap();
        for row in archive.rows.iter() {
            builder.push(row).unwrap();
        }
        archive.disk = Some(builder.finish().unwrap());
        archive.rows = std::sync::Arc::new(Vec::new());
        for directory in ["", "d1"] {
            for text in ["", "notes", "说明", "missing"] {
                for sort in 0..=12 {
                    let expected = memory
                        .listing_sorted(directory, text, 0, 128, sort, &control)
                        .unwrap();
                    let page = archive
                        .query_page(
                            QueryRequest {
                                directory,
                                text,
                                sort,
                                offset: 0,
                                expected: None,
                                generation: archive.next_query + 1,
                            },
                            &control,
                        )
                        .unwrap();
                    assert_eq!(page.total, expected.0);
                    assert_eq!(
                        page.rows.iter().map(QueryRow::path).collect::<Vec<_>>(),
                        expected
                            .1
                            .iter()
                            .map(crate::listing::ListingRow::path)
                            .collect::<Vec<_>>(),
                        "{directory}/{text}/{sort}"
                    );
                }
            }
        }
        assert!(archive.index_disk_bytes() > 0);
        assert_eq!(
            archive
                .catalog(2, 2, &control)
                .unwrap()
                .iter()
                .map(|e| e.id)
                .collect::<Vec<_>>(),
            [3, 4]
        );
        assert_eq!(archive.entry(2).unwrap(), memory.entry(2).unwrap());
        control.cancel();
        assert!(matches!(
            archive
                .disk
                .as_ref()
                .unwrap()
                .query(b"", "", 0, &[], &control),
            Err(ScanError::Cancelled)
        ));
    }
    #[test]
    fn metadata_decoder_rejects_truncation_and_oversize_before_allocating() {
        assert!(decode(&mut std::io::Cursor::new(u32::MAX.to_le_bytes())).is_err());
        assert!(decode(&mut std::io::Cursor::new([52, 0, 0, 0, 1])).is_err());
    }
}
