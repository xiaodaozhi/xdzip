//! LZH level-2 creation over frozen input indices and host-owned private output.
use std::io::{self, Read, Write};
use xdzip_codec_api::{
    Cancellation, ScanError,
    encode::{EncodeEntry, EncodeError, EncodeInput, EncodeOutput, EncodeProgress, EncodeSettings},
};
use xdzip_compression::{Crc16, LzhWriter, crc16};
use xdzip_domain::path::PortablePath;
const MAX_BYTES: u64 = 24 * 1024 * 1024 * 1024;
const MAX_WORK: u64 = 96 * 1024 * 1024 * 1024;
#[derive(Debug)]
struct Fault(EncodeError);
impl std::fmt::Display for Fault {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}
impl std::error::Error for Fault {}
fn encoded(error: io::Error) -> EncodeError {
    if let Some(fault) = error.get_ref().and_then(|e| e.downcast_ref::<Fault>()) {
        return fault.0;
    }
    match ScanError::from_source_error(&error) {
        ScanError::Cancelled => EncodeError::Cancelled,
        ScanError::Limit => EncodeError::Limit,
        _ if error.kind() == io::ErrorKind::OutOfMemory => EncodeError::Limit,
        _ if error.kind() == io::ErrorKind::InvalidData => EncodeError::Invalid,
        _ => EncodeError::Output,
    }
}
fn check(cancel: &Cancellation) -> Result<(), EncodeError> {
    cancel.check().map_err(|_| EncodeError::Cancelled)
}
struct Output<'a> {
    host: &'a mut dyn EncodeOutput,
    cancel: &'a Cancellation,
    position: u64,
    member_end: Option<u64>,
}
impl Write for Output<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        check(self.cancel).map_err(|e| io::Error::other(Fault(e)))?;
        let count = bytes.len().min(8192);
        let end = self
            .position
            .checked_add(count as u64)
            .filter(|n| *n <= self.member_end.unwrap_or(MAX_BYTES).min(MAX_BYTES))
            .ok_or_else(|| io::Error::other(Fault(EncodeError::Limit)))?;
        let n = self.host.write(&bytes[..count]).map_err(|e| {
            if e.kind() == io::ErrorKind::Interrupted && e.get_ref().is_none() {
                e
            } else {
                io::Error::other(Fault(if self.cancel.check().is_err() {
                    EncodeError::Cancelled
                } else {
                    EncodeError::Output
                }))
            }
        })?;
        if n > count {
            return Err(io::Error::other(Fault(EncodeError::Output)));
        }
        self.position = end - (count - n) as u64;
        Ok(n)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl Output<'_> {
    fn patch(&mut self, at: u64, bytes: &[u8]) -> Result<(), EncodeError> {
        check(self.cancel)?;
        if at
            .checked_add(bytes.len() as u64)
            .is_none_or(|n| n > self.position)
        {
            return Err(EncodeError::Invalid);
        }
        self.host.write_at(at, bytes)?;
        // EncodeOutput's default file implementations leave the cursor after a patch.
        // A zero-byte patch restores the sequential endpoint without granting a seek path.
        self.host.write_at(self.position, &[])?;
        check(self.cancel)
    }
}
fn extension(header: &mut Vec<u8>, kind: u8, data: &[u8], next: usize) {
    header.push(kind);
    header.extend_from_slice(data);
    header.extend_from_slice(&(next as u16).to_le_bytes());
}
fn name_parts(name: &str, directory: bool) -> Result<(&[u8], Vec<u8>), EncodeError> {
    let (directory, filename) = if directory {
        (name, "")
    } else {
        name.rsplit_once('/').unwrap_or(("", name))
    };
    let mut dir = Vec::new();
    dir.try_reserve_exact(directory.len() + usize::from(!directory.is_empty()))
        .map_err(|_| EncodeError::Limit)?;
    if !directory.is_empty() {
        dir.extend(directory.bytes().map(|b| if b == b'/' { 255 } else { b }));
        dir.push(255);
    }
    if filename.len() + dir.len() > 4096 {
        return Err(EncodeError::Limit);
    }
    Ok((filename.as_bytes(), dir))
}

fn seal_length(header: &mut Vec<u8>) -> Result<(), EncodeError> {
    // A zero low size byte would be read as an archive terminator.
    if header.len() % 256 == 0 {
        header.push(0);
    }
    let length = u16::try_from(header.len()).map_err(|_| EncodeError::Limit)?;
    header[..2].copy_from_slice(&length.to_le_bytes());
    Ok(())
}

fn header(entry: &EncodeEntry) -> Result<Vec<u8>, EncodeError> {
    let time = u32::try_from(entry.modified_unix).map_err(|_| EncodeError::Invalid)?;
    let size = u32::try_from(entry.size).map_err(|_| EncodeError::Limit)?;
    let (filename, dir) = name_parts(&entry.name, entry.directory)?;
    let length = 26 + 5 + filename.len() + 3 + if dir.is_empty() { 0 } else { dir.len() + 3 };
    let mut header = Vec::new();
    header
        .try_reserve_exact(length + 1)
        .map_err(|_| EncodeError::Limit)?;
    header.resize(26, 0);
    header[2..7].copy_from_slice(if entry.directory { b"-lhd-" } else { b"-lh5-" });
    header[11..15].copy_from_slice(&size.to_le_bytes());
    header[15..19].copy_from_slice(&time.to_le_bytes());
    header[19] = if entry.directory { 0x10 } else { 0x20 };
    header[20] = 2;
    header[23] = b'U';
    header[24..26].copy_from_slice(&5u16.to_le_bytes());
    extension(&mut header, 0, &[0, 0], filename.len() + 3);
    extension(
        &mut header,
        1,
        filename,
        if dir.is_empty() { 0 } else { dir.len() + 3 },
    );
    if !dir.is_empty() {
        extension(&mut header, 2, &dir, 0);
    }
    seal_length(&mut header)?;
    Ok(header)
}
pub(crate) fn rename_header(
    original: &[u8],
    name: &str,
    directory: bool,
) -> Result<Vec<u8>, EncodeError> {
    let name = if directory {
        name.strip_suffix('/').ok_or(EncodeError::Invalid)?
    } else {
        name
    };
    PortablePath::parse(name).ok_or(EncodeError::Invalid)?;
    let (filename, dirname) = name_parts(name, directory)?;
    let mut header = Vec::new();
    header
        .try_reserve_exact(original.len() + filename.len() + dirname.len() + 16)
        .map_err(|_| EncodeError::Limit)?;
    header.extend_from_slice(original.get(..24).ok_or(EncodeError::Invalid)?);
    header.extend_from_slice(&[0; 2]);
    // Regenerate the filename and dirname, preserve independent metadata from
    // the validated original, and recalculate the common header CRC.
    let mut extra = Vec::new();
    let mut common = [0; 3];
    let mut common_size = 2;
    let mut at = 24;
    loop {
        let word = original.get(at..at + 2).ok_or(EncodeError::Invalid)?;
        let size = usize::from(u16::from_le_bytes([word[0], word[1]]));
        if size == 0 {
            break;
        }
        if size < 3 {
            return Err(EncodeError::Invalid);
        }
        let bytes = original
            .get(at + 2..at + size)
            .ok_or(EncodeError::Invalid)?;
        match bytes[0] {
            0 => {
                if !(3..=4).contains(&bytes.len()) {
                    return Err(EncodeError::Invalid);
                }
                common_size = bytes.len() - 1;
                if common_size == 3 {
                    common[2] = bytes[3];
                }
            }
            1 | 2 => {}
            kind => extra.push((kind, &bytes[1..])),
        }
        at += size;
    }
    let mut parts = vec![(0, &common[..common_size]), (1, filename)];
    if !dirname.is_empty() {
        parts.push((2, dirname.as_slice()));
    }
    parts.extend(extra);
    header[24..26].copy_from_slice(&((common_size + 3) as u16).to_le_bytes());
    for (index, &(kind, data)) in parts.iter().enumerate() {
        extension(
            &mut header,
            kind,
            data,
            parts.get(index + 1).map_or(0, |part| part.1.len() + 3),
        );
    }
    seal_length(&mut header)?;
    let checksum = crc16(&header);
    header[27..29].copy_from_slice(&checksum.to_le_bytes());
    Ok(header)
}

fn finalize(header: &mut [u8], packed: u64, crc: u16) -> Result<(), EncodeError> {
    header[7..11].copy_from_slice(
        &u32::try_from(packed)
            .map_err(|_| EncodeError::Limit)?
            .to_le_bytes(),
    );
    header[21..23].copy_from_slice(&crc.to_le_bytes());
    header[27..29].fill(0);
    let checksum = crc16(header);
    header[27..29].copy_from_slice(&checksum.to_le_bytes());
    Ok(())
}
pub(crate) fn encode(
    input: &dyn EncodeInput,
    output: &mut dyn EncodeOutput,
    settings: EncodeSettings<'_>,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
) -> Result<(), EncodeError> {
    append(input, output, 0, settings, cancel, progress)
}
pub(crate) fn append(
    input: &dyn EncodeInput,
    output: &mut dyn EncodeOutput,
    initial_position: u64,
    settings: EncodeSettings<'_>,
    cancel: &Cancellation,
    progress: &mut dyn FnMut(EncodeProgress<'_>),
) -> Result<(), EncodeError> {
    if settings.method != "lh5"
        || ![1, 6, 9].contains(&settings.level)
        || settings.password.is_some()
        || settings.solid
        || settings.encrypt_headers
    {
        return Err(EncodeError::Unsupported);
    }
    check(cancel)?;
    if input.entries().len() > 100000 {
        return Err(EncodeError::Limit);
    }
    let mut total_size = 0u64;
    let mut names = 0usize;
    let mut headers = 0usize;
    for entry in input.entries() {
        check(cancel)?;
        if PortablePath::parse(&entry.name).is_none() || (entry.directory && entry.size != 0) {
            return Err(EncodeError::Invalid);
        }
        names = names
            .checked_add(entry.name.len())
            .and_then(|n| n.checked_add(usize::from(entry.directory)))
            .filter(|n| *n <= 16 * 1024 * 1024)
            .ok_or(EncodeError::Limit)?;
        total_size = total_size
            .checked_add(entry.size)
            .filter(|n| *n <= MAX_BYTES)
            .ok_or(EncodeError::Limit)?;
        headers = headers
            .checked_add(header(entry)?.len())
            .filter(|n| *n <= 64 * 1024 * 1024)
            .ok_or(EncodeError::Limit)?;
    }
    let mut output = Output {
        host: output,
        cancel,
        position: initial_position,
        member_end: None,
    };
    let mut input_bytes = 0u64;
    let mut work = MAX_WORK;
    let mut bytes = [0; 8192];
    for (index, entry) in input.entries().iter().enumerate() {
        check(cancel)?;
        let mut header = header(entry)?;
        let at = output.position;
        if entry.directory {
            finalize(&mut header, 0, 0)?;
            output.write_all(&header).map_err(encoded)?;
            continue;
        }
        let mut source = input.open(index)?;
        // Invalid placeholder metadata remains private; the host publishes only a verified archive.
        finalize(&mut header, 0, 0)?;
        output.write_all(&header).map_err(encoded)?;
        let begin = output.position;
        output.member_end = Some(begin + u64::from(u32::MAX));
        let mut writer = LzhWriter::with_poll(&mut output, settings.level, 1 << 20, work, || {
            cancel.check().map_err(io::Error::other)
        })
        .map_err(encoded)?;
        let mut read = 0u64;
        let mut crc = Crc16::new();
        loop {
            check(cancel)?;
            let available = (MAX_BYTES - input_bytes).min(bytes.len() as u64) as usize;
            if available == 0 {
                return Err(EncodeError::Limit);
            }
            let n = match source.read(&mut bytes[..available]) {
                Err(e) if e.kind() == io::ErrorKind::Interrupted && e.get_ref().is_none() => {
                    continue;
                }
                Err(e) => {
                    check(cancel)?;
                    return Err(match ScanError::from_source_error(&e) {
                        ScanError::Cancelled => EncodeError::Cancelled,
                        ScanError::Limit => EncodeError::Limit,
                        _ => EncodeError::Source,
                    });
                }
                Ok(n) if n <= available => n,
                Ok(_) => return Err(EncodeError::Source),
            };
            if n == 0 {
                break;
            }
            input_bytes += n as u64;
            read = read
                .checked_add(n as u64)
                .filter(|n| *n <= entry.size)
                .ok_or(EncodeError::Source)?;
            // CRC covers the actual admitted source bytes once, bounded by the 24 GiB input cap.
            crc.update(&bytes[..n]);
            writer.write_all(&bytes[..n]).map_err(encoded)?;
            progress(EncodeProgress {
                name: &entry.name,
                input_bytes,
            });
        }
        if read != entry.size {
            return Err(EncodeError::Source);
        }
        let (output_ref, remaining) = writer.finish_with_work().map_err(encoded)?;
        work = remaining;
        let packed = output_ref.position - begin;
        output_ref.member_end = None;
        finalize(&mut header, packed, crc.checksum())?;
        output.patch(at, &header)?;
    }
    check(cancel)?;
    output.write_all(&[0]).map_err(encoded)?;
    Ok(())
}
#[cfg(test)]
mod tests;
