use crate::envelope::Envelope;
use xdzip_codec_api::{
    Entry, Format, ScanError, ScanLimits,
    names::NameEncoding,
    time::{self, Timestamp},
};
use xdzip_domain::{identity::EntryId, path::components};
pub(crate) enum Origin {
    Member(usize),
    Inner(usize, usize),
}
pub(crate) struct Index {
    pub(crate) rows: Vec<Entry>,
    pub(crate) origins: Vec<Origin>,
    pub(crate) children: [Vec<Entry>; 2],
}
struct Names {
    raw: usize,
    text: usize,
}
impl Names {
    fn admit(
        &mut self,
        raw: usize,
        decoded: usize,
        display: usize,
        limits: ScanLimits,
    ) -> Result<(), ScanError> {
        self.raw = self.raw.checked_add(raw).ok_or(ScanError::Limit)?;
        self.text = self
            .text
            .checked_add(decoded)
            .and_then(|n| n.checked_add(display))
            .ok_or(ScanError::Limit)?;
        if raw > limits.name_bytes
            || decoded > limits.name_bytes
            || self.raw > limits.total_name_bytes
            || self.text > limits.display_name_bytes
        {
            return Err(ScanError::Limit);
        }
        Ok(())
    }
}
pub(crate) fn assemble(
    envelope: &Envelope,
    children: [Vec<Entry>; 2],
    limits: ScanLimits,
) -> Result<Index, ScanError> {
    let count = children
        .iter()
        .try_fold(envelope.members.len(), |count, rows| {
            count.checked_add(rows.len())
        })
        .ok_or(ScanError::Limit)?;
    if count > limits.entries {
        return Err(ScanError::Limit);
    }
    let mut rows = Vec::with_capacity(count);
    let mut origins = Vec::with_capacity(count);
    let mut names = Names { raw: 0, text: 0 };
    for (at, member) in envelope.members.iter().enumerate() {
        let length = 8 + member.name.len();
        names.admit(length, 0, length, limits)?;
        let name = format!("members/{}", member.name);
        let warning = components(member.name.as_bytes(), false).is_none()
            || components(name.as_bytes(), false).is_none();
        rows.push(Entry {
            format: Format::Deb,
            id: EntryId::try_from(rows.len() as u64 + 1).map_err(|_| ScanError::Limit)?,
            raw_name: name.as_bytes().to_vec(),
            decoded_name: None,
            display_name: name,
            name_encoding: NameEncoding::Utf8,
            directory: false,
            link: false,
            unix_mode: 0,
            size: member.size,
            size_known: true,
            packed_size: member.size,
            method: 0,
            modified_dos: time::utc_calendar(Timestamp::UnixSeconds(member.modified))
                .filter(|t| (1980..=2107).contains(&t.year))
                .and_then(|t| t.wrapping_dos())
                .unwrap_or(0),
            encrypted: false,
            password_supported: false,
            warning,
        });
        origins.push(Origin::Member(at));
    }
    for (group, children) in children.iter().enumerate() {
        let prefix = if group == 0 { "control/" } else { "data/" };
        for (at, child) in children.iter().enumerate() {
            let raw = prefix
                .len()
                .checked_add(child.raw_name.len())
                .ok_or(ScanError::Limit)?;
            let decoded = child.decoded_name.as_ref().map_or(Ok(0), |n| {
                prefix.len().checked_add(n.len()).ok_or(ScanError::Limit)
            })?;
            let display = prefix
                .len()
                .checked_add(child.display_name.len())
                .ok_or(ScanError::Limit)?;
            names.admit(raw, decoded, display, limits)?;
            let mut row = child.clone();
            row.raw_name = [prefix.as_bytes(), child.raw_name.as_slice()].concat();
            row.decoded_name = child
                .decoded_name
                .as_ref()
                .map(|name| format!("{prefix}{name}"));
            row.display_name = format!("{prefix}{}", child.display_name);
            row.id = EntryId::try_from(rows.len() as u64 + 1).map_err(|_| ScanError::Limit)?;
            row.format = Format::Deb;
            // Validate the original spelling before prefixing: /absolute must not
            // silently become an apparently relative data/absolute export path.
            row.warning |= components(child.name_bytes(), child.directory).is_none()
                || components(row.name_bytes(), row.directory).is_none();
            rows.push(row);
            origins.push(Origin::Inner(group, at));
        }
    }
    Ok(Index {
        rows,
        origins,
        children,
    })
}
