use crate::{Index, Record};
use xdzip_codec_api::{
    Cancellation, ScanError, ScanLimits,
    names::{self, NameEncoding, NameResolution, NameSample},
};
/// Fully resolved display and optional effective name, without output authority.
#[derive(Debug, Eq, PartialEq)]
pub struct ResolvedName {
    /// Effective UTF-8 spelling when it differs from the physical fields.
    pub decoded: Option<String>,
    /// Origin of the component-wise resolution, not a confidence guarantee.
    pub encoding: NameEncoding,
    /// Escaped user-facing text; never an output path.
    pub display: String,
    /// Unresolved, ambiguous or suspicious spelling requiring review.
    pub warning: bool,
}
#[cfg(test)]
mod tests;
fn fields(record: &Record) -> impl Iterator<Item = &[u8]> {
    let directory = &record.raw_name[..record.dirname_bytes];
    let directory = directory.strip_suffix(&[255]).unwrap_or(directory);
    let components = directory
        .split(|byte| *byte == 255)
        .filter(move |_| record.dirname_bytes != 0);
    components.chain(std::iter::once(&record.raw_name[record.dirname_bytes..]))
}
fn origin(current: &mut NameEncoding, next: NameEncoding) {
    match (*current, next) {
        (NameEncoding::Unresolved, _) | (_, NameEncoding::Unresolved) => {
            *current = NameEncoding::Unresolved
        }
        (NameEncoding::Legacy(a), NameEncoding::Legacy(b)) if a != b => {
            *current = NameEncoding::Unresolved
        }
        (_, kind @ NameEncoding::Legacy(_)) => *current = kind,
        (NameEncoding::Utf8, NameEncoding::UnmarkedUtf8) => *current = NameEncoding::UnmarkedUtf8,
        _ => {}
    }
}
fn resolve(
    record: &Record,
    resolution: &NameResolution,
    cancel: &Cancellation,
    limit: usize,
) -> Result<ResolvedName, ScanError> {
    let mut text = String::new();
    let mut encoding = NameEncoding::Utf8;
    let mut warning = false;
    let mut items = fields(record).peekable();
    while let Some(raw) = items.next() {
        cancel.check()?;
        let final_field = items.peek().is_none();
        if raw.is_empty() && final_field && record.directory && record.dirname_bytes != 0 {
            break;
        }
        let Some((decoded, kind)) = resolution.decode_unmarked(raw) else {
            encoding = NameEncoding::Unresolved;
            break;
        };
        origin(&mut encoding, kind);
        if encoding == NameEncoding::Unresolved {
            break;
        }
        let normalized = decoded.replace('\\', "/");
        warning |= names::suspicious(&normalized, final_field && record.directory);
        let separator = usize::from(!final_field);
        let size = text
            .len()
            .checked_add(normalized.len())
            .and_then(|n| n.checked_add(separator))
            .ok_or(ScanError::Limit)?;
        if size > limit {
            return Err(ScanError::Limit);
        }
        text.try_reserve(normalized.len() + separator)
            .map_err(|_| ScanError::Limit)?;
        text.push_str(&normalized);
        if !final_field {
            text.push('/');
        }
    }
    if encoding == NameEncoding::Unresolved {
        return Ok(ResolvedName {
            decoded: None,
            encoding,
            display: names::escaped_bytes(&record.raw_name),
            warning: true,
        });
    }
    warning |= names::suspicious(&text, record.directory);
    let display = names::display(&text);
    let decoded = (text.as_bytes() != record.raw_name).then_some(text);
    Ok(ResolvedName {
        decoded,
        encoding,
        display,
        warning,
    })
}
/// Resolve original filename fields independently, then compose and review a shared path.
pub fn resolve_names(
    index: &Index,
    cancel: &Cancellation,
    limits: ScanLimits,
) -> Result<Vec<ResolvedName>, ScanError> {
    cancel.check()?;
    if index.records.len() > limits.entries.min(xdzip_codec_api::MAX_ENTRIES) {
        return Err(ScanError::Limit);
    }
    let mut raw_total = 0usize;
    for record in &index.records {
        cancel.check()?;
        if record.dirname_bytes > record.raw_name.len()
            || (record.dirname_bytes != 0 && record.raw_name[record.dirname_bytes - 1] != 255)
        {
            return Err(ScanError::Invalid);
        }
        raw_total = raw_total
            .checked_add(record.raw_name.capacity())
            .ok_or(ScanError::Limit)?;
        if record.raw_name.len() > limits.name_bytes.min(4096)
            || raw_total > limits.total_name_bytes.min(16 * 1024 * 1024)
        {
            return Err(ScanError::Limit);
        }
    }
    let mut sample = NameSample::default();
    for record in &index.records {
        for raw in fields(record) {
            cancel.check()?;
            sample.observe_bytes(raw);
        }
    }
    let resolution = sample.finish(false);
    let mut result = Vec::new();
    result
        .try_reserve_exact(index.records.len())
        .map_err(|_| ScanError::Limit)?;
    let mut total = 0usize;
    for record in &index.records {
        let name = resolve(record, &resolution, cancel, limits.name_bytes.min(4096))?;
        total = total
            .checked_add(name.decoded.as_ref().map_or(0, String::capacity))
            .and_then(|n| n.checked_add(name.display.capacity()))
            .ok_or(ScanError::Limit)?;
        if total > limits.display_name_bytes.min(16 * 1024 * 1024) {
            return Err(ScanError::Limit);
        }
        result.push(name);
    }
    Ok(result)
}
