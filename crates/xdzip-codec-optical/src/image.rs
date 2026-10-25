//! Shared authority-free image I/O, namespace budgets and physical extents.
use std::io::{self, SeekFrom};
use xdzip_codec_api::names::{NameEncoding, display, escaped_bytes};
use xdzip_codec_api::{Cancellation, Entry, Format, ReadSource, ScanError, ScanLimits};
use xdzip_domain::identity::EntryId;

/// Parsers return exact physical payload spans; only the host can publish output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Extent {
    Data { offset: u64, length: u64 },
    Zero { length: u64 },
    Inline(Vec<u8>),
}

pub(crate) struct Node {
    pub name: Vec<u8>,
    pub directory: bool,
    pub link: bool,
    pub special: bool,
    pub size: u64,
    pub modified_dos: u32,
    pub extents: Vec<Extent>,
}

pub(crate) struct Record {
    pub entry: Entry,
    pub extents: Vec<Extent>,
    unlabelled_name: bool,
}
pub(crate) struct Index {
    pub records: Vec<Record>,
}

pub(crate) struct ImageReader<'a> {
    source: &'a mut dyn ReadSource,
    control: &'a Cancellation,
    remaining: u64,
    length: u64,
}

pub(crate) struct IndexBuilder {
    format: Format,
    limits: ScanLimits,
    records: Vec<Record>,
    names: usize,
    display: usize,
    extents: usize,
    inline: usize,
}

pub(crate) fn source_error(error: io::Error) -> ScanError {
    if let Some(error) = error
        .get_ref()
        .and_then(|error| error.downcast_ref::<ScanError>())
    {
        return *error;
    }
    if error.kind() == io::ErrorKind::UnexpectedEof {
        ScanError::Invalid
    } else {
        ScanError::Io
    }
}

impl<'a> ImageReader<'a> {
    pub(crate) fn new(
        source: &'a mut dyn ReadSource,
        control: &'a Cancellation,
        read_budget: u64,
    ) -> Result<Self, ScanError> {
        control.check()?;
        let length = source.seek(SeekFrom::End(0)).map_err(source_error)?;
        Ok(Self {
            source,
            control,
            remaining: read_budget,
            length,
        })
    }
    pub(crate) fn len(&self) -> u64 {
        self.length
    }
    pub(crate) fn set_budget(&mut self, remaining: u64) {
        self.remaining = remaining;
    }
    pub(crate) fn check(&self) -> Result<(), ScanError> {
        self.control.check()
    }
    pub(crate) fn resolve_names(
        &self,
        index: Index,
        limits: ScanLimits,
    ) -> Result<Index, ScanError> {
        let mut rows = Vec::with_capacity(index.records.len());
        let mut spans = Vec::with_capacity(index.records.len());
        let mut legacy = Vec::with_capacity(index.records.len());
        for record in index.records {
            rows.push(record.entry);
            spans.push(record.extents);
            legacy.push(record.unlabelled_name);
        }
        xdzip_codec_api::names::resolve(&mut rows, &legacy, false, self.control, limits)?;
        // Decoding precedes portable-path admission. In particular, a Shift-JIS
        // trail byte 0x5c must not become an irreversible raw-path warning.
        for row in &mut rows {
            self.control.check()?;
            row.warning |=
                xdzip_domain::path::components(row.name_bytes(), row.directory).is_none();
        }
        Ok(Index {
            records: rows
                .into_iter()
                .zip(spans)
                .zip(legacy)
                .map(|((entry, extents), unlabelled_name)| Record {
                    entry,
                    extents,
                    unlabelled_name,
                })
                .collect(),
        })
    }
    pub(crate) fn read_at(&mut self, offset: u64, size: usize) -> Result<Vec<u8>, ScanError> {
        self.admit(offset, size)?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(size)
            .map_err(|_| ScanError::Limit)?;
        bytes.resize(size, 0);
        self.read_exact_at(offset, &mut bytes)?;
        Ok(bytes)
    }
    fn admit(&self, offset: u64, size: usize) -> Result<(), ScanError> {
        self.check()?;
        if offset
            .checked_add(size as u64)
            .is_none_or(|end| end > self.length)
        {
            return Err(ScanError::Invalid);
        }
        if size as u64 > self.remaining {
            return Err(ScanError::Limit);
        }
        Ok(())
    }
    pub(crate) fn read_exact_at(&mut self, offset: u64, bytes: &mut [u8]) -> Result<(), ScanError> {
        self.admit(offset, bytes.len())?;
        self.source
            .seek(SeekFrom::Start(offset))
            .map_err(source_error)?;
        let mut filled = 0;
        while filled < bytes.len() {
            self.check()?;
            let count = match self.source.read(&mut bytes[filled..]) {
                Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
                result => result.map_err(source_error)?,
            };
            if count == 0 {
                return Err(ScanError::Invalid);
            }
            self.remaining -= count as u64;
            filled += count;
        }
        Ok(())
    }
}

impl IndexBuilder {
    pub(crate) fn new(format: Format, limits: ScanLimits) -> Self {
        Self {
            format,
            limits,
            records: Vec::new(),
            names: 0,
            display: 0,
            extents: 0,
            inline: 0,
        }
    }
    pub(crate) fn push(&mut self, node: Node) -> Result<(), ScanError> {
        self.push_with_origin(node, false)
    }
    pub(crate) fn push_unlabelled(&mut self, node: Node) -> Result<(), ScanError> {
        self.push_with_origin(node, true)
    }
    fn push_with_origin(&mut self, node: Node, unlabelled_name: bool) -> Result<(), ScanError> {
        if self.records.len() >= self.limits.entries.min(xdzip_codec_api::MAX_ENTRIES)
            || node.name.len() > self.limits.name_bytes.min(4096)
        {
            return Err(ScanError::Limit);
        }
        self.names = self
            .names
            .checked_add(node.name.len())
            .ok_or(ScanError::Limit)?;
        self.extents = self
            .extents
            .checked_add(node.extents.len())
            .ok_or(ScanError::Limit)?;
        if self.names > self.limits.total_name_bytes || self.extents > 1_000_000 {
            return Err(ScanError::Limit);
        }
        let mut total = 0u64;
        for extent in &node.extents {
            let length = match extent {
                Extent::Data { length, .. } | Extent::Zero { length } => *length,
                Extent::Inline(bytes) => {
                    self.inline = self
                        .inline
                        .checked_add(bytes.len())
                        .ok_or(ScanError::Limit)?;
                    bytes.len() as u64
                }
            };
            total = total.checked_add(length).ok_or(ScanError::Invalid)?;
        }
        if self.inline > 64 * 1024 * 1024 {
            return Err(ScanError::Limit);
        }
        if !node.link && !node.special && total != node.size {
            return Err(ScanError::Invalid);
        }
        if node.directory && node.size != 0 {
            return Err(ScanError::Invalid);
        }
        let text = std::str::from_utf8(&node.name);
        let valid_utf8 = text.is_ok();
        let display_name = if unlabelled_name {
            String::new()
        } else {
            text.map(display)
                .unwrap_or_else(|_| escaped_bytes(&node.name))
        };
        self.display = self
            .display
            .checked_add(display_name.len())
            .ok_or(ScanError::Limit)?;
        if self.display > self.limits.display_name_bytes {
            return Err(ScanError::Limit);
        }
        let warning = node.special
            || (!unlabelled_name
                && xdzip_domain::path::components(&node.name, node.directory).is_none());
        let entry = Entry {
            format: self.format,
            id: EntryId::try_from(self.records.len() as u64 + 1).map_err(|_| ScanError::Limit)?,
            raw_name: node.name,
            decoded_name: None,
            name_encoding: if valid_utf8 || unlabelled_name {
                NameEncoding::Utf8
            } else {
                NameEncoding::Unresolved
            },
            display_name,
            directory: node.directory,
            link: node.link,
            unix_mode: 0,
            size: node.size,
            packed_size: node.size,
            method: 0,
            modified_dos: node.modified_dos,
            encrypted: false,
            password_supported: false,
            warning,
        };
        self.records.push(Record {
            entry,
            extents: node.extents,
            unlabelled_name,
        });
        Ok(())
    }
    pub(crate) fn finish(self) -> Index {
        Index {
            records: self.records,
        }
    }
}
