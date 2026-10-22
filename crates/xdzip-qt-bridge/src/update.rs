// crates/xdzip-qt-bridge/src/update.rs

//! Separate update ABI. Authentication remains independent of inspection.

// Generated cxx glue validates borrowed buffers and owns returned strings.
#[allow(unsafe_code)]
#[cxx::bridge(namespace = "xdzip::updates")]
mod ffi {
    struct Inspection {
        valid: bool,
        error: String,
        newer: bool,
        version: String,
        build: u32,
        channel: String,
        release_url: String,
        asset_url: String,
        filename: String,
        bytes: u64,
        sha256: String,
        appcast_url: String,
    }
    extern "Rust" {
        fn inspect_release(
            json: &str,
            current: &str,
            current_build: u32,
            current_channel: &str,
            platform: &str,
            architecture: &str,
            package_type: &str,
        ) -> Inspection;
        fn compare_versions(left: &str, right: &str) -> i32;
        fn release_tag_eligible(tag: &str, current: &str, current_channel: &str) -> bool;
        fn verify_manifest(manifest: &[u8], envelope_json: &str, keyring_json: &str) -> String;
    }
}

fn inspect_release(
    json: &str,
    current: &str,
    current_build: u32,
    current_channel: &str,
    platform: &str,
    architecture: &str,
    package_type: &str,
) -> ffi::Inspection {
    let result = xdzip_update::inspect_release(
        json,
        current,
        current_build,
        current_channel,
        platform,
        architecture,
        package_type,
    );
    ffi::Inspection {
        valid: result.valid,
        error: result.error,
        newer: result.newer,
        version: result.version,
        build: result.build,
        channel: result.channel,
        release_url: result.release_url,
        asset_url: result.asset_url,
        filename: result.filename,
        bytes: result.bytes,
        sha256: result.sha256,
        appcast_url: result.appcast_url,
    }
}

fn compare_versions(left: &str, right: &str) -> i32 {
    xdzip_update::compare_versions(left, right)
}
fn release_tag_eligible(tag: &str, current: &str, channel: &str) -> bool {
    xdzip_update::release_tag_eligible(tag, current, channel)
}
fn verify_manifest(bytes: &[u8], envelope: &str, keyring: &str) -> String {
    xdzip_update::verify_manifest(bytes, envelope, keyring)
}
