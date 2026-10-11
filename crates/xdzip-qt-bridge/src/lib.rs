//! Worker-only read API. Not the full asynchronous command protocol or a plugin ABI.
use std::path::Path;
/// Thread-safe cooperative control; only this shared handle permits concurrent const calls.
#[derive(Default)]
pub struct Cancellation(xdzip_codec_api::Cancellation);
impl Cancellation {
    /// Request stop without waiting for the worker or taking a queue slot.
    pub fn cancel(&self) {
        self.0.cancel();
    }
}
/// Worker-owned read-only session; never concurrently called or destroyed.
pub struct Archive(xdzip_runtime::Archive);

// Only the generated cxx ABI glue may contain unsafe code; handwritten code stays denied.
#[allow(unsafe_code)]
#[cxx::bridge(namespace = "xdzip::reader")]
mod ffi {
    struct Row {
        id: String,
        name: String,
        size: u64,
        packed_size: u64,
        method: u16,
        directory: bool,
        encrypted: bool,
        warning: bool,
    }
    struct BrowsePage {
        total: u64,
        rows: Vec<Row>,
    }
    struct PreflightReport {
        entries: u64,
        declared_bytes: String,
        findings: Vec<String>,
        finding_count: u64,
        unicode_review: bool,
    }
    extern "Rust" {
        type Cancellation;
        type Archive;
        fn new_cancellation() -> Box<Cancellation>;
        fn cancel(self: &Cancellation);
        fn open_archive(path: &str, control: &Cancellation) -> Result<Box<Archive>>;
        fn preflight(
            archive: &Archive,
            selected: &[String],
            control: &Cancellation,
        ) -> Result<PreflightReport>;
        fn count(archive: &Archive) -> u64;
        fn session_id(archive: &Archive) -> String;
        fn browse(
            archive: &Archive,
            directory: &str,
            text: &str,
            offset: u64,
            control: &Cancellation,
        ) -> Result<BrowsePage>;
    }
}
/// Shared cancellation supports concurrent const calls; its Box must stay alive for the worker.
pub fn new_cancellation() -> Box<Cancellation> {
    Box::default()
}
/// Blocking open: never call on the GUI thread. UTF-8 source paths are the current boundary.
pub fn open_archive(
    path: &str,
    control: &Cancellation,
) -> Result<Box<Archive>, xdzip_codec_api::ScanError> {
    xdzip_runtime::Archive::open(Path::new(path), &control.0)
        .map(|archive| Box::new(Archive(archive)))
}
/// Physical record count of a complete immutable snapshot.
pub fn count(archive: &Archive) -> u64 {
    archive.0.len() as u64
}
/// Opaque decimal identity avoids exposing 64-bit IDs through QML floating point.
pub fn session_id(archive: &Archive) -> String {
    archive.0.key().session.get().to_string()
}
/// Bounded display query. Physical identities survive filtering and directory navigation.
pub fn browse(
    archive: &Archive,
    directory: &str,
    text: &str,
    offset: u64,
    control: &Cancellation,
) -> Result<ffi::BrowsePage, xdzip_codec_api::ScanError> {
    let (total, rows) = archive.0.browse(directory, text, offset, 128, &control.0)?;
    Ok(ffi::BrowsePage {
        total: total as u64,
        rows: rows
            .into_iter()
            .map(|entry| ffi::Row {
                id: entry.id.get().to_string(),
                name: entry.display_name.clone(),
                size: entry.size,
                packed_size: entry.packed_size,
                method: entry.method,
                directory: entry.directory,
                encrypted: entry.encrypted,
                warning: entry.warning,
            })
            .collect(),
    })
}

/// Preflight selection IDs against the owned snapshot. No output capability is returned.
pub fn preflight(
    archive: &Archive,
    selected: &[String],
    control: &Cancellation,
) -> Result<ffi::PreflightReport, xdzip_codec_api::ScanError> {
    use xdzip_codec_api::ScanError;
    if selected.len() > 128 {
        return Err(ScanError::Limit);
    }
    let ids = selected
        .iter()
        .map(|id| {
            if id.len() > 20 {
                return Err(ScanError::Invalid);
            }
            id.parse::<u64>()
                .ok()
                .and_then(|n| xdzip_domain::identity::EntryId::try_from(n).ok())
                .ok_or(ScanError::Invalid)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let report = archive.0.preflight(&ids, &control.0)?;
    Ok(ffi::PreflightReport {
        entries: report.entries as u64,
        declared_bytes: report
            .declared_bytes
            .map(|n| n.to_string())
            .unwrap_or_else(|| "unknown".into()),
        findings: report
            .findings
            .iter()
            .map(|f| format!("{}:{}", f.entry.get(), f.reason))
            .collect(),
        finding_count: report.finding_count as u64,
        unicode_review: report.unicode_review,
    })
}
