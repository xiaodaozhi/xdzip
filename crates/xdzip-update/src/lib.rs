// crates/xdzip-update/src/lib.rs

//! Qt-independent immutable release selection and publisher authentication.
//!
//! A successful inspection is not authentication. Installation requires a separate
//! successful signature verification using a locally trusted public-key set.

use ed25519_dalek::{Signature, VerifyingKey};
use serde::Deserialize;
use std::{cmp::Ordering, collections::BTreeMap};

/// Fixed publisher. A downloaded manifest cannot redirect trust to another repo.
pub const REPOSITORY: &str = "xiaodaozhi/xdzip";
/// Bound metadata before parsing or signature verification.
pub const MANIFEST_LIMIT: usize = 256 * 1024;
const KEYRING_LIMIT: usize = 64 * 1024;

/// A complete, structurally validated candidate for one installation target.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Inspection {
    /// Structural validation succeeded; this does not authenticate the publisher.
    pub valid: bool,
    /// Stable error identifier, never downloaded text.
    pub error: String,
    /// Candidate is newer and belongs to an allowed progression channel.
    pub newer: bool,
    /// Full semantic product version.
    pub version: String,
    /// Monotonic native build identity.
    pub build: u32,
    /// Stable public release channel.
    pub channel: String,
    /// Fixed GitHub release page.
    pub release_url: String,
    /// Fixed, versioned download URL for this installation.
    pub asset_url: String,
    /// Filename from the validated target contract.
    pub filename: String,
    /// Exact expected package size.
    pub bytes: u64,
    /// Exact lower-case SHA-256 expected for downloaded bytes.
    pub sha256: String,
    /// Optional single-candidate Sparkle feed pinned to this release tag.
    pub appcast_url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
    schema_version: u32,
    product: String,
    version: String,
    channel: String,
    build: u32,
    tag: String,
    commit: String,
    repository: String,
    assets: Vec<Asset>,
    #[serde(default)]
    macos_appcast_url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Asset {
    filename: String,
    platform: String,
    architecture: String,
    package_type: String,
    minimum_system: BTreeMap<String, String>,
    bytes: u64,
    sha256: String,
    url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ProductVersion(u64, u64, u64, u8, u64);

fn version(value: &str, development: bool) -> Option<ProductVersion> {
    let parsed = semver::Version::parse(value).ok()?;
    if !parsed.build.is_empty()
        || [parsed.major, parsed.minor, parsed.patch]
            .iter()
            .any(|v| *v > 65535)
    {
        return None;
    }
    let (rank, number) = if parsed.pre.is_empty() {
        (4, 0)
    } else {
        let (channel, number) = parsed.pre.as_str().split_once('.')?;
        let number = number.parse::<u64>().ok()?;
        if number == 0 || number > 65535 {
            return None;
        }
        let rank = match channel {
            "dev" if development => 0,
            "alpha" => 1,
            "beta" => 2,
            "rc" => 3,
            _ => return None,
        };
        (rank, number)
    };
    Some(ProductVersion(
        parsed.major,
        parsed.minor,
        parsed.patch,
        rank,
        number,
    ))
}

fn channel_rank(channel: &str) -> Option<u8> {
    match channel {
        "development" => Some(0),
        "alpha" => Some(1),
        "beta" => Some(2),
        "rc" => Some(3),
        "stable" => Some(4),
        _ => None,
    }
}

/// Compare supported versions, including development builds; invalid input is 2.
pub fn compare_versions(left: &str, right: &str) -> i32 {
    match (version(left, true), version(right, true)) {
        (Some(left), Some(right)) => match left.cmp(&right) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        },
        _ => 2,
    }
}

/// Prefilter a public GitHub tag without guessing filenames or publication order.
pub fn release_tag_eligible(tag: &str, current: &str, current_channel: &str) -> bool {
    let Some(candidate) = tag.strip_prefix('v').and_then(|v| version(v, false)) else {
        return false;
    };
    let Some(current) = version(current, true) else {
        return false;
    };
    let Some(rank) = channel_rank(current_channel) else {
        return false;
    };
    current.3 == rank && candidate > current && candidate.3 >= rank
}

fn lower_hex(value: &str, bytes: usize) -> bool {
    value.len() == bytes * 2
        && value
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
}

fn target(
    platform: &str,
    architecture: &str,
    package_type: &str,
) -> Option<(&'static str, BTreeMap<String, String>)> {
    let (suffix, requirements): (&str, &[(&str, &str)]) =
        match (platform, architecture, package_type) {
            ("windows", "x86_64", "zip") => ("windows-x86_64.zip", &[("windows", "11")]),
            ("windows", "x86_64", "exe") => ("windows-x86_64-setup.exe", &[("windows", "11")]),
            ("macos", "arm64", "app.zip") => ("macos-arm64.app.zip", &[("macos", "14.0")]),
            ("macos", "arm64", "dmg") => ("macos-arm64.dmg", &[("macos", "14.0")]),
            ("linux", "x86_64", "tar.gz") => (
                "linux-x86_64.tar.gz",
                &[
                    ("distribution", "ubuntu"),
                    ("version", "24.04"),
                    ("glibc", "2.39"),
                    ("display_server", "X11"),
                ],
            ),
            ("linux", "x86_64", "deb") => (
                "ubuntu24.04-x86_64.deb",
                &[
                    ("distribution", "ubuntu"),
                    ("version", "24.04"),
                    ("glibc", "2.39"),
                    ("display_server", "X11"),
                ],
            ),
            _ => return None,
        };
    Some((
        suffix,
        requirements
            .iter()
            .map(|(k, v)| ((*k).into(), (*v).into()))
            .collect(),
    ))
}

fn manifest(json: &str) -> Result<Manifest, &'static str> {
    if json.len() > MANIFEST_LIMIT {
        return Err("manifest_too_large");
    }
    let manifest: Manifest = serde_json::from_str(json).map_err(|_| "invalid_manifest_json")?;
    if manifest.schema_version != 1 || manifest.product != "xdzip" {
        return Err("unsupported_manifest");
    }
    let parsed = version(&manifest.version, false).ok_or("invalid_release_version")?;
    if channel_rank(&manifest.channel) != Some(parsed.3) {
        return Err("channel_mismatch");
    }
    if !(1..=65535).contains(&manifest.build)
        || manifest.tag != format!("v{}", manifest.version)
        || !lower_hex(&manifest.commit, 20)
        || manifest.repository != REPOSITORY
    {
        return Err("release_identity_mismatch");
    }
    if manifest.assets.len() != 6 {
        return Err("incomplete_release");
    }
    let mut seen = std::collections::BTreeSet::new();
    for asset in &manifest.assets {
        let (suffix, requirements) =
            target(&asset.platform, &asset.architecture, &asset.package_type)
                .ok_or("unsupported_target")?;
        if !seen.insert((&asset.platform, &asset.architecture, &asset.package_type)) {
            return Err("duplicate_target");
        }
        let expected = format!("xdzip-{}-{suffix}", manifest.version);
        if asset.filename != expected
            || asset.minimum_system != requirements
            || asset.bytes == 0
            || !lower_hex(&asset.sha256, 32)
            || asset.url
                != format!(
                    "https://github.com/{REPOSITORY}/releases/download/{}/{expected}",
                    manifest.tag
                )
        {
            return Err("asset_identity_mismatch");
        }
    }
    if !manifest.macos_appcast_url.is_empty()
        && manifest.macos_appcast_url
            != format!(
                "https://github.com/{REPOSITORY}/releases/download/{}/appcast.xml",
                manifest.tag
            )
    {
        return Err("invalid_appcast_url");
    }
    Ok(manifest)
}

/// Inspect one complete manifest for a known local installation target.
#[allow(clippy::too_many_arguments)]
pub fn inspect_release(
    json: &str,
    current: &str,
    current_build: u32,
    current_channel: &str,
    platform: &str,
    architecture: &str,
    package_type: &str,
) -> Inspection {
    let result = (|| {
        let parsed = manifest(json)?;
        let current_version = version(current, true).ok_or("invalid_current_version")?;
        if channel_rank(current_channel) != Some(current_version.3)
            || !(1..=65535).contains(&current_build)
        {
            return Err("invalid_current_identity");
        }
        let asset = parsed
            .assets
            .iter()
            .find(|asset| {
                asset.platform == platform
                    && asset.architecture == architecture
                    && asset.package_type == package_type
            })
            .ok_or("unsupported_installation")?;
        let newer = release_tag_eligible(&parsed.tag, current, current_channel);
        if newer && parsed.build <= current_build {
            return Err("non_monotonic_build");
        }
        Ok(Inspection {
            valid: true,
            error: String::new(),
            newer,
            version: parsed.version.clone(),
            build: parsed.build,
            channel: parsed.channel.clone(),
            release_url: format!(
                "https://github.com/{REPOSITORY}/releases/tag/{}",
                parsed.tag
            ),
            asset_url: asset.url.clone(),
            filename: asset.filename.clone(),
            bytes: asset.bytes,
            sha256: asset.sha256.clone(),
            appcast_url: parsed.macos_appcast_url.clone(),
        })
    })();
    result.unwrap_or_else(|error: &str| Inspection {
        error: error.into(),
        ..Inspection::default()
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SignatureEnvelope {
    schema_version: u32,
    algorithm: String,
    key_id: String,
    signature: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Keyring {
    schema_version: u32,
    keys: Vec<PublicKey>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicKey {
    key_id: String,
    public_key: String,
}

fn decode_hex<const N: usize>(value: &str) -> Result<[u8; N], &'static str> {
    if !lower_hex(value, N) {
        return Err("invalid_hex");
    }
    let mut result = [0; N];
    for (index, byte) in result.iter_mut().enumerate() {
        *byte =
            u8::from_str_radix(&value[index * 2..index * 2 + 2], 16).map_err(|_| "invalid_hex")?;
    }
    Ok(result)
}

fn valid_key_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|c| c.is_ascii_alphanumeric() || c == b'-' || c == b'_')
}

/// Authenticate the exact original manifest bytes with a locally trusted keyring.
///
/// An empty return means success; all failures are stable nonempty error codes.
/// An empty production keyring deliberately disables authenticated installation.
pub fn verify_manifest(bytes: &[u8], envelope_json: &str, keyring_json: &str) -> String {
    let result = (|| {
        if bytes.len() > MANIFEST_LIMIT {
            return Err("manifest_too_large");
        }
        let json = std::str::from_utf8(bytes).map_err(|_| "invalid_manifest_json")?;
        manifest(json)?;
        if envelope_json.len() > 4096 || keyring_json.len() > KEYRING_LIMIT {
            return Err("signature_metadata_too_large");
        }
        let envelope: SignatureEnvelope =
            serde_json::from_str(envelope_json).map_err(|_| "invalid_signature_envelope")?;
        let keyring: Keyring =
            serde_json::from_str(keyring_json).map_err(|_| "invalid_trust_keyring")?;
        if envelope.schema_version != 1
            || envelope.algorithm != "ed25519"
            || !valid_key_id(&envelope.key_id)
        {
            return Err("unsupported_signature");
        }
        if keyring.schema_version != 1 || keyring.keys.len() > 16 {
            return Err("invalid_trust_keyring");
        }
        let mut seen = std::collections::BTreeSet::new();
        let mut selected = None;
        for key in &keyring.keys {
            if !valid_key_id(&key.key_id) || !seen.insert(&key.key_id) {
                return Err("invalid_trust_keyring");
            }
            let public = decode_hex::<32>(&key.public_key).map_err(|_| "invalid_trust_keyring")?;
            let public = VerifyingKey::from_bytes(&public).map_err(|_| "invalid_trust_keyring")?;
            if public.is_weak() {
                return Err("invalid_trust_keyring");
            }
            if key.key_id == envelope.key_id {
                selected = Some(public);
            }
        }
        let public = selected.ok_or("untrusted_signing_key")?;
        let signature = Signature::from_bytes(
            &decode_hex::<64>(&envelope.signature).map_err(|_| "invalid_signature")?,
        );
        public
            .verify_strict(bytes, &signature)
            .map_err(|_| "signature_mismatch")
    })();
    result.err().unwrap_or("").into()
}

#[cfg(test)]
mod tests;
