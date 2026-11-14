use crate::{
    framing::{self, Header},
    keys::{self, Cbc, Mac, Material},
    work::{self, Work},
};
use xdzip_codec_api::{Cancellation, ReadSource, ScanError, ScanLimits, names};
use zeroize::Zeroizing;
/// Authenticated payload size and safe virtual basename, without external-file authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metadata {
    /// Authorized basename policy; the host still validates the destination.
    pub name: String,
    /// Observed authenticated size for v3, or the legacy unauthenticated modulo convention.
    pub size: u64,
    /// Complete physical encrypted envelope size.
    pub packed_size: u64,
    /// Actual AES Crypt stream version, independent of application versions.
    pub version: u8,
}
pub(crate) struct Admitted {
    pub metadata: Metadata,
    pub header: Header,
    pub keys: Material,
}
pub(crate) fn padding_size(block: &[u8; 16], cipher: u64) -> Result<u64, ScanError> {
    let count = usize::from(block[15]);
    let mut mismatch = u8::from(!(1..=16).contains(&count));
    let start = 16usize.saturating_sub(count);
    for (at, byte) in block.iter().enumerate() {
        let selected = u8::from(at >= start).wrapping_neg();
        mismatch |= (*byte ^ block[15]) & selected;
    }
    if mismatch != 0 {
        return Err(ScanError::Invalid);
    }
    cipher.checked_sub(count as u64).ok_or(ScanError::Invalid)
}
pub(crate) fn authenticate(
    source: &mut dyn ReadSource,
    header: &Header,
    keys: &Material,
    cancel: &Cancellation,
    work: &mut Work,
) -> Result<u64, ScanError> {
    cancel.check()?;
    work::seek(source, header.start)?;
    let mut mac = Mac::new(&keys.key, work)?;
    let mut left = header.cipher;
    let mut buffer = [0; 65536];
    let mut previous = Zeroizing::new(*keys.iv);
    let mut last = Zeroizing::new(*keys.iv);
    while left != 0 {
        let take = left.min(buffer.len() as u64) as usize;
        work::read(source, &mut buffer[..take], cancel, work)?;
        mac.update(&buffer[..take], work)?;
        if take >= 32 {
            previous.copy_from_slice(&buffer[take - 32..take - 16]);
        } else {
            previous.copy_from_slice(last.as_ref());
        }
        last.copy_from_slice(&buffer[take - 16..take]);
        left -= take as u64;
    }
    if !constant_time_eq::constant_time_eq_32(
        mac.finish(work)?.as_ref().try_into().expect("MAC width"),
        &header.data_tag,
    ) {
        return Err(if header.version == 0 {
            ScanError::PasswordOrIntegrity
        } else {
            ScanError::Invalid
        });
    }
    if let Some(size) = header.legacy_size {
        return Ok(size);
    }
    Cbc::new(&keys.key, *previous).decrypt(last.as_mut(), cancel, work)?;
    padding_size(&last, header.cipher)
}
pub(crate) fn inspect(
    source: &mut dyn ReadSource,
    password: Option<&str>,
    cancel: &Cancellation,
    limits: ScanLimits,
    work: &mut Work,
) -> Result<Admitted, ScanError> {
    cancel.check()?;
    work.left.input = work.left.input.min(limits.read_bytes);
    if limits.entries == 0 {
        return Err(ScanError::Limit);
    }
    let name = names::single_file_name(source.source_name_hint(), &[".aes"]);
    let display = names::display(&name);
    if name.len() > limits.name_bytes
        || name.len() > limits.total_name_bytes
        || display.len() > limits.display_name_bytes
    {
        return Err(ScanError::Limit);
    }
    let header = framing::parse(source, cancel, work)?;
    let password = password.ok_or(ScanError::PasswordRequired)?;
    let keys = keys::unwrap(&header, password, cancel, work)?;
    let size = authenticate(source, &header, &keys, cancel, work)?;
    Ok(Admitted {
        metadata: Metadata {
            name,
            size,
            packed_size: header.length,
            version: header.version,
        },
        header,
        keys,
    })
}
