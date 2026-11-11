//! Experimental, opt-in, read-only SDK trial. The desktop does not load this host.
mod broker;
mod package;
mod process;
mod sandbox;

use std::path::Path;
use std::sync::atomic::AtomicBool;

/// Stable failures never include peer-controlled diagnostics or private paths.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    /// Invalid package, path or executable digest.
    Package,
    /// The required kernel policy is unavailable; execution was refused.
    Sandbox,
    /// Bounded IPC failed validation.
    Protocol,
    /// Time, memory, byte or metadata budget exceeded.
    Limit,
    /// The caller cancelled; all uncommitted output is discarded.
    Cancelled,
    /// A source changed, an output conflicted, or an I/O primitive failed.
    Io,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Package => "sdk_package",
            Self::Sandbox => "sdk_sandbox_unavailable",
            Self::Protocol => "sdk_protocol",
            Self::Limit => "sdk_limit",
            Self::Cancelled => "sdk_cancelled",
            Self::Io => "sdk_io",
        })
    }
}
impl std::error::Error for Error {}
impl From<xdzip_plugin_protocol::Error> for Error {
    fn from(error: xdzip_plugin_protocol::Error) -> Self {
        match error {
            xdzip_plugin_protocol::Error::Limit => Self::Limit,
            _ => Self::Protocol,
        }
    }
}
impl From<std::io::Error> for Error {
    fn from(_: std::io::Error) -> Self {
        Self::Io
    }
}
impl From<xdzip_io::OutputError> for Error {
    fn from(_: xdzip_io::OutputError) -> Self {
        Self::Io
    }
}

/// Counts measured by the host; no archive content or user pathname is serialized.
#[derive(Debug, serde::Serialize)]
pub struct Report {
    /// Negotiated wire major, independent of the Rust ABI.
    pub protocol_major: u64,
    /// Actual enforced kernel policy, or `trusted-conformance` in unit tests only.
    pub isolation: String,
    /// Number of admitted physical records.
    pub entries: u64,
    /// Actual bytes returned through the source broker.
    pub source_bytes: u64,
    /// Actual committed bytes, zero for scan-only calls.
    pub output_bytes: u64,
    /// Host-computed output digest, empty for scan-only calls.
    pub output_sha256: String,
    /// Whether output publication completed.
    pub committed: bool,
    /// Whether explicit workspace cleanup reported a recoverable leftover.
    pub cleanup_warning: bool,
}

/// Run one native package under the platform gate. Selecting an entry is explicit;
/// no selection means scan-only. The target directory must already exist.
pub fn run(
    manifest: &Path,
    source: &Path,
    selected: Option<(usize, &Path)>,
    cancel: &AtomicBool,
) -> Result<Report, Error> {
    let package = package::Package::load(manifest)?;
    let mut peer = process::Peer::spawn(&package)?;
    broker::run(&mut peer, source, selected, &package.id, cancel)
}

/// Name of the policy available to the trial launcher. Windows is deliberately
/// refused until its AppContainer/Job implementation has native evidence.
pub fn platform_gate() -> &'static str {
    sandbox::policy()
}

#[cfg(test)]
mod conformance;
