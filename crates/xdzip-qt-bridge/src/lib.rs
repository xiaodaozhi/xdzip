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
    extern "Rust" {
        type Cancellation;
        type Archive;
        fn new_cancellation() -> Box<Cancellation>;
        fn cancel(self: &Cancellation);
        fn open_archive(path: &str, control: &Cancellation) -> Result<Box<Archive>>;
        fn count(archive: &Archive) -> u64;
        fn session_id(archive: &Archive) -> String;
        fn page(archive: &Archive, offset: u64, limit: u32) -> Result<Vec<Row>>;
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
/// Copy one bounded display page. Raw names remain in Rust and are never output paths.
pub fn page(
    archive: &Archive,
    offset: u64,
    limit: u32,
) -> Result<Vec<ffi::Row>, xdzip_codec_api::ScanError> {
    Ok(archive
        .0
        .page(offset, u64::from(limit))?
        .iter()
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
        .collect())
}
