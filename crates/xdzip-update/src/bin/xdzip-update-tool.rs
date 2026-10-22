// crates/xdzip-update/src/bin/xdzip-update-tool.rs

//! Publisher-side signing tool. It never creates a production private key.

use base64ct::{Base64, Encoding};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::Deserialize;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{collections::BTreeMap, fs, path::Path, process::ExitCode};
use zeroize::Zeroizing;

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn arguments() -> Result<(String, BTreeMap<String, String>), &'static str> {
    let mut args = std::env::args().skip(1);
    let command = args.next().ok_or("missing_command")?;
    let mut options = BTreeMap::new();
    while let Some(name) = args.next() {
        if !name.starts_with("--")
            || options
                .insert(name, args.next().ok_or("missing_argument")?)
                .is_some()
        {
            return Err("invalid_arguments");
        }
    }
    Ok((command, options))
}

fn bounded_file(path: &str, limit: usize) -> Result<Vec<u8>, &'static str> {
    let metadata = fs::symlink_metadata(path).map_err(|_| "input_unavailable")?;
    if !metadata.is_file() || metadata.len() > limit as u64 {
        return Err("invalid_input_file");
    }
    let bytes = fs::read(path).map_err(|_| "input_unavailable")?;
    if bytes.len() > limit {
        return Err("input_too_large");
    }
    Ok(bytes)
}

fn required<'a>(
    options: &'a BTreeMap<String, String>,
    name: &str,
) -> Result<&'a str, &'static str> {
    options
        .get(name)
        .map(String::as_str)
        .ok_or("missing_argument")
}

fn signing_key(variable: &str) -> Result<SigningKey, &'static str> {
    let source = Zeroizing::new(std::env::var(variable).map_err(|_| "signing_key_not_configured")?);
    if source.len() != 64
        || !source
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
    {
        return Err("invalid_signing_key");
    }
    let mut seed = Zeroizing::new([0; 32]);
    for (index, byte) in seed.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&source[index * 2..index * 2 + 2], 16)
            .map_err(|_| "invalid_signing_key")?;
    }
    Ok(SigningKey::from_bytes(&seed))
}

fn write_new(path: &str, bytes: &[u8]) -> Result<(), &'static str> {
    // A signature contains no secret, but never overwrite an operator's path or a
    // symlink accidentally. Failed publication leaves the original input intact.
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(path))
        .map_err(|_| "output_unavailable")?;
    file.write_all(bytes)
        .and_then(|_| file.sync_all())
        .map_err(|_| "output_write_failed")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SparklePublicKey {
    schema_version: u32,
    public_key: String,
}

fn sparkle_public(path: &str) -> Result<VerifyingKey, &'static str> {
    let config: SparklePublicKey = serde_json::from_slice(&bounded_file(path, 4096)?)
        .map_err(|_| "invalid_sparkle_public_key")?;
    if config.schema_version != 1
        || config.public_key.len() != 64
        || !config
            .public_key
            .bytes()
            .all(|v| v.is_ascii_digit() || (b'a'..=b'f').contains(&v))
    {
        return Err("sparkle_key_not_configured");
    }
    let mut bytes = [0; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&config.public_key[index * 2..index * 2 + 2], 16)
            .map_err(|_| "invalid_sparkle_public_key")?;
    }
    let key = VerifyingKey::from_bytes(&bytes).map_err(|_| "invalid_sparkle_public_key")?;
    if key.is_weak() {
        return Err("invalid_sparkle_public_key");
    }
    Ok(key)
}

fn sparkle_signing_key(path: &str) -> Result<SigningKey, &'static str> {
    let expected = sparkle_public(path)?;
    let key = signing_key("XDZIP_SPARKLE_SIGNING_KEY_HEX")?;
    if key.verifying_key() != expected {
        return Err("sparkle_signing_key_mismatch");
    }
    Ok(key)
}

fn verify_appcast(bytes: &[u8], public: &VerifyingKey) -> Result<(), &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "invalid_appcast_encoding")?;
    let (content, block) = text
        .rsplit_once("<!-- sparkle-signatures:\n")
        .ok_or("unsigned_appcast")?;
    if content.contains("<!-- sparkle-signatures:") {
        return Err("duplicate_appcast_signature");
    }
    let lines: Vec<_> = block.lines().collect();
    if lines.len() != 3 || lines[2] != "-->" {
        return Err("invalid_appcast_signature");
    }
    let signature = lines[0]
        .strip_prefix("edSignature: ")
        .ok_or("invalid_appcast_signature")?;
    let length: usize = lines[1]
        .strip_prefix("length: ")
        .ok_or("invalid_appcast_signature")?
        .parse()
        .map_err(|_| "invalid_appcast_signature")?;
    if length != content.len() {
        return Err("appcast_length_mismatch");
    }
    let signature = Base64::decode_vec(signature).map_err(|_| "invalid_appcast_signature")?;
    let signature: [u8; 64] = signature
        .try_into()
        .map_err(|_| "invalid_appcast_signature")?;
    public
        .verify_strict(content.as_bytes(), &Signature::from_bytes(&signature))
        .map_err(|_| "appcast_signature_mismatch")
}

fn run() -> Result<(), String> {
    let (command, options) = arguments()?;
    match command.as_str() {
        "sparkle-public-key" => {
            if options.len() != 1 {
                return Err("invalid_arguments".into());
            }
            let key = signing_key("XDZIP_SPARKLE_SIGNING_KEY_HEX")?;
            let bytes = serde_json::to_vec_pretty(&json!({"schema_version":1,
                "public_key":hex(key.verifying_key().as_bytes())}))
            .map_err(|_| "serialization_failed")?;
            write_new(required(&options, "--output")?, &bytes)?;
        }
        "sign-archive" => {
            if options.len() != 5 {
                return Err("invalid_arguments".into());
            }
            let archive = required(&options, "--archive")?;
            let version = required(&options, "--version")?;
            let build: u32 = required(&options, "--build")?
                .parse()
                .map_err(|_| "invalid_build")?;
            if !(1..=65535).contains(&build)
                || xdzip_update::compare_versions(version, version) != 0
                || version.contains("-dev.")
            {
                return Err("invalid_release_identity".into());
            }
            let expected = format!("xdzip-{version}-macos-arm64.app.zip");
            if Path::new(archive).file_name().and_then(|v| v.to_str()) != Some(&expected) {
                return Err("invalid_archive_name".into());
            }
            let bytes = bounded_file(archive, 2 * 1024 * 1024 * 1024)?;
            let key = sparkle_signing_key(required(&options, "--public-key")?)?;
            let signature = key.sign(&bytes);
            key.verifying_key()
                .verify_strict(&bytes, &signature)
                .map_err(|_| "signature_self_check_failed")?;
            let metadata = serde_json::to_vec_pretty(
                &json!({"schema_version":1,"version":version,"build":build,
                "filename":expected,"bytes":bytes.len(),"sha256":hex(&Sha256::digest(&bytes)),
                "ed_signature":Base64::encode_string(&signature.to_bytes())}),
            )
            .map_err(|_| "serialization_failed")?;
            write_new(required(&options, "--output")?, &metadata)?;
        }
        "sign-appcast" => {
            if options.len() != 3 {
                return Err("invalid_arguments".into());
            }
            let mut bytes = bounded_file(
                required(&options, "--appcast")?,
                xdzip_update::MANIFEST_LIMIT,
            )?;
            let text = std::str::from_utf8(&bytes).map_err(|_| "invalid_appcast_encoding")?;
            if !text.starts_with("<?xml") || text.contains("<!-- sparkle-signatures:") {
                return Err("invalid_unsigned_appcast".into());
            }
            let key = sparkle_signing_key(required(&options, "--public-key")?)?;
            let signature = Base64::encode_string(&key.sign(&bytes).to_bytes());
            let block = format!(
                "<!-- sparkle-signatures:\nedSignature: {signature}\nlength: {}\n-->\n",
                bytes.len()
            );
            bytes.extend_from_slice(block.as_bytes());
            verify_appcast(&bytes, &key.verifying_key())?;
            write_new(required(&options, "--output")?, &bytes)?;
        }
        "verify-appcast" => {
            if options.len() != 2 {
                return Err("invalid_arguments".into());
            }
            let bytes = bounded_file(
                required(&options, "--appcast")?,
                xdzip_update::MANIFEST_LIMIT + 512,
            )?;
            verify_appcast(
                &bytes,
                &sparkle_public(required(&options, "--public-key")?)?,
            )?;
        }
        "verify-archive" => {
            if options.len() != 3 {
                return Err("invalid_arguments".into());
            }
            let bytes = bounded_file(required(&options, "--archive")?, 2 * 1024 * 1024 * 1024)?;
            let signature = Base64::decode_vec(required(&options, "--signature")?)
                .map_err(|_| "invalid_archive_signature")?;
            let signature: [u8; 64] = signature
                .try_into()
                .map_err(|_| "invalid_archive_signature")?;
            sparkle_public(required(&options, "--public-key")?)?
                .verify_strict(&bytes, &Signature::from_bytes(&signature))
                .map_err(|_| "archive_signature_mismatch")?;
        }
        "sign" => {
            if options.len() != 4 {
                return Err("invalid_arguments".into());
            }
            let bytes = bounded_file(
                required(&options, "--manifest")?,
                xdzip_update::MANIFEST_LIMIT,
            )?;
            let keyring = bounded_file(required(&options, "--keyring")?, 64 * 1024)?;
            let keyring = std::str::from_utf8(&keyring).map_err(|_| "invalid_trust_keyring")?;
            let key = signing_key("XDZIP_UPDATE_SIGNING_KEY_HEX")?;
            let envelope = serde_json::to_vec_pretty(&json!({"schema_version":1,"algorithm":"ed25519",
                "key_id":required(&options,"--key-id")?,"signature":hex(&key.sign(&bytes).to_bytes())})).map_err(|_| "serialization_failed")?;
            let error = xdzip_update::verify_manifest(
                &bytes,
                std::str::from_utf8(&envelope).map_err(|_| "serialization_failed")?,
                keyring,
            );
            if !error.is_empty() {
                return Err(error);
            }
            write_new(required(&options, "--output")?, &envelope)?;
        }
        "verify" => {
            if options.len() != 3 {
                return Err("invalid_arguments".into());
            }
            let bytes = bounded_file(
                required(&options, "--manifest")?,
                xdzip_update::MANIFEST_LIMIT,
            )?;
            let envelope = bounded_file(required(&options, "--signature")?, 4096)?;
            let keyring = bounded_file(required(&options, "--keyring")?, 64 * 1024)?;
            let error = xdzip_update::verify_manifest(
                &bytes,
                std::str::from_utf8(&envelope).map_err(|_| "invalid_signature_envelope")?,
                std::str::from_utf8(&keyring).map_err(|_| "invalid_trust_keyring")?,
            );
            if !error.is_empty() {
                return Err(error);
            }
        }
        "public-key" => {
            if options.len() != 2 {
                return Err("invalid_arguments".into());
            }
            let key = signing_key("XDZIP_UPDATE_SIGNING_KEY_HEX")?;
            let key_id = required(&options, "--key-id")?;
            if key_id.is_empty()
                || key_id.len() > 64
                || !key_id
                    .bytes()
                    .all(|v| v.is_ascii_alphanumeric() || v == b'-' || v == b'_')
            {
                return Err("invalid_key_id".into());
            }
            let bytes =
                serde_json::to_vec_pretty(&json!({"schema_version":1,"keys":[{"key_id":key_id,
                "public_key":hex(key.verifying_key().as_bytes())}]}))
                .map_err(|_| "serialization_failed")?;
            write_new(required(&options, "--output")?, &bytes)?;
        }
        _ => return Err("unsupported_command".into()),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("xdzip update signing failed: {error}");
            ExitCode::FAILURE
        }
    }
}
