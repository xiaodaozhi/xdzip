//! Pure metadata preflight. Never grants output authority or opens a filesystem path.
use std::collections::{BTreeMap, BTreeSet};
use xdzip_codec_api::{Cancellation, Entry, ScanError};
use xdzip_domain::identity::EntryId;

/// One bounded finding associated with a physical record, never an output pathname.
#[derive(Debug)]
pub struct Finding {
    /// Record carrying the finding.
    pub entry: EntryId,
    /// Stable reason for presentation by the host.
    pub reason: &'static str,
}
/// Advisory report; even a report without findings is not permission to write.
#[derive(Debug, Default)]
pub struct Report {
    /// Physical records after directory expansion and ID deduplication.
    pub entries: usize,
    /// Declared total bytes; None means arithmetic overflow, not zero bytes.
    pub declared_bytes: Option<u64>,
    /// Total findings, including those omitted from the bounded detail list.
    pub finding_count: usize,
    /// At most 64 representative findings in physical order.
    pub findings: Vec<Finding>,
    /// Non-ASCII components require platform Unicode-equivalence checks later.
    pub unicode_review: bool,
}
impl Report {
    fn add(&mut self, entry: EntryId, reason: &'static str) {
        self.finding_count += 1;
        if self.findings.len() < 64 {
            self.findings.push(Finding { entry, reason });
        }
    }
}

/// Check a bounded immutable metadata snapshot and an explicit nonempty selection.
/// Names are never repaired. Directories expand over all descendants, independent of UI filters.
pub fn inspect(
    rows: &[Entry],
    selected: &[EntryId],
    control: &Cancellation,
) -> Result<Report, ScanError> {
    control.check()?;
    if rows.len() > xdzip_codec_api::MAX_ENTRIES || selected.is_empty() || selected.len() > 128 {
        return Err(ScanError::Limit);
    }
    let ids: BTreeSet<_> = selected.iter().copied().collect();
    if ids.iter().any(|id| !rows.iter().any(|row| row.id == *id)) {
        return Err(ScanError::Invalid);
    }
    let directories: Vec<_> = rows
        .iter()
        .filter(|row| ids.contains(&row.id) && row.directory)
        .map(|row| row.raw_name.as_slice())
        .collect();
    let mut report = Report {
        declared_bytes: Some(0),
        ..Report::default()
    };
    // (directory, explicitly present): implicit parents can later receive a directory record.
    let mut outputs: BTreeMap<String, (bool, bool, String)> = BTreeMap::new();
    let mut name_budget = 0_usize;
    let mut key_budget = 0_usize;
    for row in rows {
        control.check()?;
        if !ids.contains(&row.id)
            && !directories
                .iter()
                .any(|prefix| prefix.ends_with(b"/") && row.raw_name.starts_with(prefix))
        {
            continue;
        }
        report.entries += 1;
        name_budget = name_budget
            .checked_add(row.raw_name.len())
            .ok_or(ScanError::Limit)?;
        if name_budget > 4 * 1024 * 1024 {
            return Err(ScanError::Limit);
        }
        if !row.directory {
            report.declared_bytes = report.declared_bytes.and_then(|n| n.checked_add(row.size));
            if row.size > 8 * 1024 * 1024 * 1024 {
                report.add(row.id, "file_limit");
            }
            if row.format == xdzip_codec_api::Format::Zip
                && row.size > row.packed_size.saturating_mul(1000)
            {
                report.add(row.id, "high_ratio");
            }
        }
        if row.encrypted {
            report.add(row.id, "encrypted");
        }
        if !matches!(row.method, 0 | 8) {
            report.add(row.id, "method");
        }
        if row.warning {
            report.add(row.id, "metadata");
        }
        let Some(parts) = components(&row.raw_name, row.directory) else {
            report.add(row.id, "path");
            continue;
        };
        report.unicode_review |= parts.iter().any(|part| !part.is_ascii());
        let mut path = String::new();
        let mut original = String::new();
        for (index, part) in parts.iter().enumerate() {
            if !path.is_empty() {
                path.push('/');
                original.push('/');
            }
            // Conservative case comparison only; full filesystem equivalence is a separate gate.
            path.push_str(&part.to_lowercase());
            original.push_str(part);
            let last = index + 1 == parts.len();
            let directory = !last || row.directory;
            match outputs.get_mut(&path) {
                Some((existing_dir, explicit, spelling)) => {
                    if spelling != &original || !*existing_dir || !directory || (last && *explicit)
                    {
                        report.add(row.id, "collision");
                        break;
                    }
                    if last {
                        *explicit = true;
                    }
                }
                None => {
                    key_budget = key_budget
                        .checked_add(path.len() + original.len())
                        .ok_or(ScanError::Limit)?;
                    if key_budget > 8 * 1024 * 1024 {
                        return Err(ScanError::Limit);
                    }
                    outputs.insert(path.clone(), (directory, last, original.clone()));
                }
            }
        }
    }
    if report
        .declared_bytes
        .is_none_or(|n| n > 20 * 1024 * 1024 * 1024)
    {
        report.add(*ids.first().ok_or(ScanError::Invalid)?, "total_limit");
    }
    Ok(report)
}

fn components(raw: &[u8], directory: bool) -> Option<Vec<&str>> {
    xdzip_domain::path::components(raw, directory)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn entry(id: u64, name: &str) -> Entry {
        Entry {
            format: xdzip_codec_api::Format::Zip,
            id: EntryId::try_from(id).unwrap(),
            raw_name: name.as_bytes().to_vec(),
            display_name: name.into(),
            directory: name.ends_with('/'),
            size: 1,
            packed_size: 1,
            method: 0,
            modified_dos: 0,
            encrypted: false,
            warning: false,
        }
    }
    fn run(rows: &[Entry]) -> Report {
        inspect(
            rows,
            &rows.iter().map(|r| r.id).collect::<Vec<_>>(),
            &Cancellation::default(),
        )
        .unwrap()
    }
    #[test]
    fn portable_paths_reject_escape_device_and_ambiguous_names() {
        for name in [
            "../x",
            "/x",
            "C:/x",
            "a\\b",
            "a//b",
            "./x",
            "NUL.txt",
            "com1",
            "LPT².log",
            "a. ",
            "a:",
            "x\0y",
            "x\u{202e}y",
            "a/./b",
        ] {
            assert!(
                run(&[entry(1, name)])
                    .findings
                    .iter()
                    .any(|f| f.reason == "path"),
                "{name:?}"
            );
        }
        let report = run(&[entry(1, "旅行/说明.txt")]);
        assert_eq!(report.finding_count, 0);
        assert!(report.unicode_review);
    }
    #[test]
    fn collisions_are_order_independent_and_not_last_writer_wins() {
        for names in [
            ["a", "a/b"],
            ["a/b", "a"],
            ["a", "A"],
            ["a/b", "A/c"],
            ["a/", "a/"],
            ["notes.txt", "notes.txt"],
        ] {
            assert!(
                run(&[entry(1, names[0]), entry(2, names[1])])
                    .findings
                    .iter()
                    .any(|f| f.reason == "collision")
            );
        }
        assert_eq!(run(&[entry(1, "a/b"), entry(2, "a/")]).finding_count, 0);
    }
    #[test]
    fn directory_selection_expands_once_and_rejects_foreign_ids() {
        let rows = [entry(1, "a/"), entry(2, "a/b"), entry(3, "ab/c")];
        let control = Cancellation::default();
        let report = inspect(&rows, &[rows[0].id, rows[1].id], &control).unwrap();
        assert_eq!(report.entries, 2);
        assert_eq!(report.declared_bytes, Some(1));
        assert!(inspect(&rows, &[EntryId::try_from(99).unwrap()], &control).is_err());
        control.cancel();
        assert!(matches!(
            inspect(&rows, &[rows[0].id], &control),
            Err(ScanError::Cancelled)
        ));
    }
    #[test]
    fn oversized_names_depth_and_collision_index_fail_closed() {
        for name in ["x".repeat(256), vec!["x"; 65].join("/"), "x".repeat(4097)] {
            assert!(
                run(&[entry(1, &name)])
                    .findings
                    .iter()
                    .any(|f| f.reason == "path")
            );
        }
        let rows: Vec<_> = (1..=100)
            .map(|n| {
                let name = format!("{n:060}/{}", vec!["x".repeat(60); 63].join("/"));
                entry(n, &name)
            })
            .collect();
        let ids: Vec<_> = rows.iter().map(|row| row.id).collect();
        assert!(matches!(
            inspect(&rows, &ids, &Cancellation::default()),
            Err(ScanError::Limit)
        ));
        assert!(inspect(&rows, &[], &Cancellation::default()).is_err());
        let mut invalid = entry(1, "x");
        invalid.raw_name = vec![255];
        assert!(run(&[invalid]).findings.iter().any(|f| f.reason == "path"));
    }
    #[test]
    fn limits_overflow_and_bounded_findings_do_not_approve_output() {
        let mut rows: Vec<_> = (1..=100).map(|n| entry(n, "../bad")).collect();
        rows[0].size = u64::MAX;
        rows[0].encrypted = true;
        rows[0].method = 99;
        let report = run(&rows);
        assert_eq!(report.declared_bytes, None);
        assert_eq!(report.findings.len(), 64);
        assert!(report.finding_count > report.findings.len());
        assert!(report.findings.iter().any(|f| f.reason == "encrypted"));
    }
}
