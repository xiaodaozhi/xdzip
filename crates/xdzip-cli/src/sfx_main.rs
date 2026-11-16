//! Independently runnable, project-owned ZIP self-extractor.
#![cfg_attr(windows, windows_subsystem = "windows")]
fn main() {
    std::process::exit(xdzip_cli::run_sfx());
}
