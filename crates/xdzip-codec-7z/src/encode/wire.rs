//! Direct serialization of the published 7z wire grammar, with bounded metadata.
use super::{EncodeEntry, EncodeError, FileProperties, META_LIMIT, WrittenFolder, check};
use xdzip_codec_api::Cancellation;
use xdzip_compression::{Crc32, crc32};
struct Buffer {
    bytes: Vec<u8>,
}
impl Buffer {
    fn new(capacity: usize) -> Result<Self, EncodeError> {
        if capacity > META_LIMIT {
            return Err(EncodeError::Limit);
        }
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(capacity)
            .map_err(|_| EncodeError::Limit)?;
        Ok(Self { bytes })
    }
    fn add(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        let size = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or(EncodeError::Limit)?;
        if size > META_LIMIT {
            return Err(EncodeError::Limit);
        }
        if size > self.bytes.capacity() {
            self.bytes
                .try_reserve_exact(size - self.bytes.len())
                .map_err(|_| EncodeError::Limit)?;
        }
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }
    fn byte(&mut self, byte: u8) -> Result<(), EncodeError> {
        self.add(&[byte])
    }
    fn number(&mut self, number: u64) -> Result<(), EncodeError> {
        for extra in 0..8 {
            if number < 1u64 << (7 * (extra + 1)) {
                let prefix = if extra == 0 { 0 } else { 0xffu8 << (8 - extra) };
                self.byte(prefix | (number >> (8 * extra)) as u8)?;
                return self.add(&number.to_le_bytes()[..extra]);
            }
        }
        self.byte(255)?;
        self.add(&number.to_le_bytes())
    }
    fn bits(
        &mut self,
        count: usize,
        mut predicate: impl FnMut(usize) -> bool,
    ) -> Result<(), EncodeError> {
        for start in (0..count).step_by(8) {
            let mut byte = 0;
            for bit in 0..8 {
                if start + bit < count && predicate(start + bit) {
                    byte |= 0x80 >> bit;
                }
            }
            self.byte(byte)?;
        }
        Ok(())
    }
    fn property(&mut self, id: u8, data: Buffer) -> Result<(), EncodeError> {
        self.byte(id)?;
        self.number(data.bytes.len() as u64)?;
        self.add(&data.bytes)
    }
}
fn coder(target: &mut Buffer, id: &[u8], properties: &[u8]) -> Result<(), EncodeError> {
    target.byte(id.len() as u8 | if properties.is_empty() { 0 } else { 32 })?;
    target.add(id)?;
    if !properties.is_empty() {
        target.number(properties.len() as u64)?;
        target.add(properties)?;
    }
    Ok(())
}
fn streams(
    target: &mut Buffer,
    folders: &[WrittenFolder],
    indices: &[usize],
    entries: &[EncodeEntry],
    crcs: &[u32],
    compressed: bool,
    control: &Cancellation,
) -> Result<(), EncodeError> {
    target.byte(6)?;
    target.number(0)?;
    target.number(folders.len() as u64)?;
    target.byte(9)?;
    for folder in folders {
        check(control)?;
        target.number(folder.packed)?;
    }
    target.add(&[10, 1])?;
    for folder in folders {
        check(control)?;
        target.add(&folder.packed_crc.to_le_bytes())?;
    }
    target.add(&[0, 7, 11])?;
    target.number(folders.len() as u64)?;
    target.byte(0)?;
    for folder in folders {
        check(control)?;
        target.number(if folder.aes.is_some() { 2 } else { 1 })?;
        if let Some(properties) = &folder.aes {
            coder(target, &[6, 0xf1, 7, 1], properties)?;
        }
        coder(
            target,
            if compressed { &[0x21] } else { &[0] },
            if compressed { &[22] } else { &[] },
        )?;
        if folder.aes.is_some() {
            target.number(1)?;
            target.number(0)?;
        }
    }
    target.byte(12)?;
    for folder in folders {
        check(control)?;
        if folder.aes.is_some() {
            target.number(folder.intermediate)?;
        }
        target.number(folder.unpacked)?;
    }
    target.add(&[10, 1])?;
    for folder in folders {
        target.add(&folder.crc.to_le_bytes())?;
    }
    target.add(&[0, 8, 13])?;
    for folder in folders {
        target.number(folder.count as u64)?;
    }
    if folders.iter().any(|folder| folder.count > 1) {
        target.byte(9)?;
        for folder in folders {
            check(control)?;
            for &index in &indices[folder.first..folder.first + folder.count - 1] {
                target.number(entries[index].size)?;
            }
        }
        target.add(&[10, 1])?;
        for folder in folders {
            check(control)?;
            if folder.count > 1 {
                for &index in &indices[folder.first..folder.first + folder.count] {
                    target.add(&crcs[index].to_le_bytes())?;
                }
            }
        }
    }
    target.add(&[0, 0])
}
fn times(
    target: &mut Buffer,
    id: u8,
    entries: &[EncodeEntry],
    properties: Option<&[FileProperties]>,
    get: impl Fn(FileProperties) -> Option<u64>,
    control: &Cancellation,
) -> Result<(), EncodeError> {
    let value = |index| {
        get(properties.map_or_else(
            || FileProperties::new(&entries[index]),
            |values| values[index],
        ))
    };
    if !(0..entries.len()).any(|index| value(index).is_some()) {
        return Ok(());
    }
    let mut data = Buffer::new(2 + entries.len().div_ceil(8) + entries.len() * 8)?;
    let defined = (0..entries.len()).all(|index| value(index).is_some());
    data.byte(u8::from(defined))?;
    if !defined {
        data.bits(entries.len(), |index| value(index).is_some())?;
    }
    data.byte(0)?;
    for index in 0..entries.len() {
        check(control)?;
        if let Some(value) = value(index) {
            data.add(&value.to_le_bytes())?;
        }
    }
    target.property(id, data)
}
pub(super) fn header(
    entries: &[EncodeEntry],
    properties: Option<&[FileProperties]>,
    folders: &[WrittenFolder],
    indices: &[usize],
    crcs: &[u32],
    compressed: bool,
    control: &Cancellation,
) -> Result<Vec<u8>, EncodeError> {
    let name_bytes = entries.iter().try_fold(1usize, |sum, entry| {
        sum.checked_add((entry.name.encode_utf16().count() + 1) * 2)
            .ok_or(EncodeError::Limit)
    })?;
    let capacity = name_bytes
        .checked_add(entries.len() * 128)
        .and_then(|v| v.checked_add(folders.len() * 96 + 256))
        .ok_or(EncodeError::Limit)?;
    let mut target = Buffer::new(capacity.min(META_LIMIT))?;
    target.byte(1)?;
    if !folders.is_empty() {
        target.byte(4)?;
        streams(
            &mut target,
            folders,
            indices,
            entries,
            crcs,
            compressed,
            control,
        )?;
    }
    target.byte(5)?;
    target.number(entries.len() as u64)?;
    let empty = entries.iter().filter(|entry| entry.size == 0).count();
    if empty != 0 {
        let mut data = Buffer::new(entries.len().div_ceil(8))?;
        data.bits(entries.len(), |index| entries[index].size == 0)?;
        target.property(14, data)?;
        let mut data = Buffer::new(empty.div_ceil(8))?;
        let mut iterator = entries.iter().filter(|entry| entry.size == 0);
        data.bits(empty, |_| {
            iterator.next().is_some_and(|entry| !entry.directory)
        })?;
        target.property(15, data)?;
    }
    let mut names = Buffer::new(name_bytes)?;
    names.byte(0)?;
    for entry in entries {
        check(control)?;
        for unit in entry.name.encode_utf16() {
            names.add(&unit.to_le_bytes())?;
        }
        names.add(&[0, 0])?;
    }
    target.property(17, names)?;
    times(
        &mut target,
        18,
        entries,
        properties,
        |p| p.creation_time,
        control,
    )?;
    times(
        &mut target,
        19,
        entries,
        properties,
        |p| p.access_time,
        control,
    )?;
    times(
        &mut target,
        20,
        entries,
        properties,
        |p| p.modified_time,
        control,
    )?;
    let value = |index| {
        properties
            .map_or_else(
                || FileProperties::new(&entries[index]),
                |values| values[index],
            )
            .attributes
    };
    if (0..entries.len()).any(|index| value(index).is_some()) {
        let mut data = Buffer::new(2 + entries.len().div_ceil(8) + entries.len() * 4)?;
        let defined = (0..entries.len()).all(|index| value(index).is_some());
        data.byte(u8::from(defined))?;
        if !defined {
            data.bits(entries.len(), |index| value(index).is_some())?;
        }
        data.byte(0)?;
        for index in 0..entries.len() {
            check(control)?;
            if let Some(value) = value(index) {
                data.add(&value.to_le_bytes())?;
            }
        }
        target.property(21, data)?;
    }
    target.add(&[0, 0])?;
    check(control)?;
    Ok(target.bytes)
}
pub(super) fn encoded_header(
    offset: u64,
    packed: u64,
    packed_crc: u32,
    unpacked: u64,
    raw_crc: u32,
    properties: &[u8],
) -> Result<Vec<u8>, EncodeError> {
    let mut target = Buffer::new(192)?;
    target.add(&[23, 6])?;
    target.number(offset)?;
    target.number(1)?;
    target.byte(9)?;
    target.number(packed)?;
    target.add(&[10, 1])?;
    target.add(&packed_crc.to_le_bytes())?;
    target.add(&[0, 7, 11, 1, 0, 1])?;
    coder(&mut target, &[6, 0xf1, 7, 1], properties)?;
    target.byte(12)?;
    target.number(unpacked)?;
    target.add(&[10, 1])?;
    target.add(&raw_crc.to_le_bytes())?;
    target.add(&[0, 0])?;
    Ok(target.bytes)
}
pub(super) fn crc(bytes: &[u8], control: &Cancellation) -> Result<u32, EncodeError> {
    let mut crc = Crc32::new();
    for block in bytes.chunks(32768) {
        check(control)?;
        crc.update(block);
    }
    check(control)?;
    Ok(crc.checksum())
}
pub(super) fn signature(offset: u64, size: u64, crc: u32) -> [u8; 32] {
    let mut signature = [0; 32];
    signature[..8].copy_from_slice(b"7z\xbc\xaf\x27\x1c\0\x04");
    signature[12..20].copy_from_slice(&offset.to_le_bytes());
    signature[20..28].copy_from_slice(&size.to_le_bytes());
    signature[28..].copy_from_slice(&crc.to_le_bytes());
    let start_crc = crc32(&signature[12..]);
    signature[8..12].copy_from_slice(&start_crc.to_le_bytes());
    signature
}
