// crates/xdzip-update/src/tests.rs

use super::*;
use ed25519_dalek::{Signer, SigningKey};
use serde_json::{Value, json};

fn candidate() -> Value {
    let version = "0.1.0-alpha.7";
    let mut assets = Vec::new();
    for (platform, architecture, package_type) in [
        ("windows", "x86_64", "zip"),
        ("windows", "x86_64", "exe"),
        ("macos", "arm64", "app.zip"),
        ("macos", "arm64", "dmg"),
        ("linux", "x86_64", "tar.gz"),
        ("linux", "x86_64", "deb"),
    ] {
        let (suffix, requirements) = target(platform, architecture, package_type).unwrap();
        let filename = format!("xdzip-{version}-{suffix}");
        assets.push(json!({"filename":filename,"platform":platform,"architecture":architecture,"package_type":package_type,
            "minimum_system":requirements,"bytes":12,"sha256":"ab".repeat(32),
            "url":format!("https://github.com/{REPOSITORY}/releases/download/v{version}/{filename}")}));
    }
    json!({"schema_version":1,"product":"xdzip","version":version,"channel":"alpha","build":20,
        "tag":format!("v{version}"),"commit":"a".repeat(40),"repository":REPOSITORY,"assets":assets,
        "macos_appcast_url":format!("https://github.com/{REPOSITORY}/releases/download/v{version}/appcast.xml")})
}

fn inspect(value: &Value) -> Inspection {
    inspect_release(
        &value.to_string(),
        "0.1.0-alpha.6",
        19,
        "alpha",
        "windows",
        "x86_64",
        "exe",
    )
}

#[test]
fn semantic_and_channel_progression() {
    assert_eq!(compare_versions("0.1.0-alpha.9", "0.1.0-alpha.10"), -1);
    assert_eq!(compare_versions("0.1.0-dev.99", "0.1.0-alpha.1"), -1);
    assert_eq!(compare_versions("0.1.0-rc.1", "0.1.0"), -1);
    for invalid in [
        "v0.1.0",
        "0.1.0-alpha.0",
        "0.1.0-alpha.01",
        "0.1.0-alpha.1+metadata",
        "0.1.0-preview.1",
        "65536.0.0",
    ] {
        assert_eq!(compare_versions(invalid, "0.1.0"), 2, "{invalid}");
    }
    for (current, channel, tag, expected) in [
        ("0.1.0-alpha.6", "alpha", "v0.1.0-beta.1", true),
        ("0.1.0-beta.1", "beta", "v0.2.0-alpha.1", false),
        ("0.1.0-rc.1", "rc", "v0.2.0-beta.1", false),
        ("0.1.0", "stable", "v0.2.0-rc.1", false),
        ("0.1.0", "stable", "v0.2.0", true),
        ("0.1.0-dev.99", "development", "v0.1.0-alpha.1", true),
        ("0.1.0-alpha.6", "stable", "v0.1.0", false),
    ] {
        assert_eq!(
            release_tag_eligible(tag, current, channel),
            expected,
            "{current}/{channel}/{tag}"
        );
    }
}

#[test]
fn shared_python_rust_selection_vectors() {
    let cases: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/update/selection-cases.json"
    ))
    .unwrap();
    for row in cases["comparisons"].as_array().unwrap() {
        assert_eq!(
            compare_versions(row[0].as_str().unwrap(), row[1].as_str().unwrap()),
            row[2].as_i64().unwrap() as i32
        );
    }
    for row in cases["eligibility"].as_array().unwrap() {
        assert_eq!(
            release_tag_eligible(
                row[0].as_str().unwrap(),
                row[1].as_str().unwrap(),
                row[2].as_str().unwrap()
            ),
            row[3].as_bool().unwrap()
        );
    }
}

#[test]
fn complete_candidate_and_native_build_are_bound() {
    let value = candidate();
    let result = inspect(&value);
    assert!(result.valid && result.newer);
    assert!(result.filename.ends_with("-setup.exe"));
    assert!(result.appcast_url.ends_with("/v0.1.0-alpha.7/appcast.xml"));
    assert!(
        !inspect_release(
            &value.to_string(),
            "0.1.0-alpha.7",
            20,
            "alpha",
            "windows",
            "x86_64",
            "exe"
        )
        .newer
    );
    assert_eq!(
        inspect_release(
            &value.to_string(),
            "0.1.0-alpha.6",
            20,
            "alpha",
            "windows",
            "x86_64",
            "exe"
        )
        .error,
        "non_monotonic_build"
    );
    assert_eq!(
        inspect_release(
            &value.to_string(),
            "0.1.0-alpha.6",
            19,
            "alpha",
            "windows",
            "arm64",
            "exe"
        )
        .error,
        "unsupported_installation"
    );
}

#[test]
fn metadata_mutations_fail_closed() {
    let original = candidate();
    let mut mutations = Vec::new();
    let mut v = original.clone();
    v["assets"].as_array_mut().unwrap().pop();
    mutations.push(v);
    let mut v = original.clone();
    v["repository"] = json!("attacker/xdzip");
    mutations.push(v);
    let mut v = original.clone();
    v["assets"][0]["url"] = json!("https://example.com/package");
    mutations.push(v);
    let mut v = original.clone();
    v["assets"][0]["bytes"] = json!(true);
    mutations.push(v);
    let mut v = original.clone();
    v["assets"][0]["minimum_system"]["windows"] = json!("10");
    mutations.push(v);
    let mut v = original.clone();
    v["assets"][0]["sha256"] = json!("AB".repeat(32));
    mutations.push(v);
    let mut v = original.clone();
    v["assets"][1] = v["assets"][0].clone();
    mutations.push(v);
    let mut v = original.clone();
    v["channel"] = json!("beta");
    mutations.push(v);
    let mut v = original.clone();
    v["macos_appcast_url"] =
        json!("https://github.com/xiaodaozhi/xdzip/releases/latest/download/appcast.xml");
    mutations.push(v);
    for v in mutations {
        assert!(!inspect(&v).valid, "{v}");
    }
    let duplicate = original
        .to_string()
        .replacen("\"build\":20", "\"build\":20,\"build\":21", 1);
    assert!(
        !inspect_release(
            &duplicate,
            "0.1.0-alpha.6",
            19,
            "alpha",
            "windows",
            "x86_64",
            "exe"
        )
        .valid
    );
    assert!(
        !inspect_release(
            &" ".repeat(MANIFEST_LIMIT + 1),
            "0.1.0",
            1,
            "stable",
            "windows",
            "x86_64",
            "exe"
        )
        .valid
    );
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

#[test]
fn publisher_signature_authenticates_exact_bytes_and_local_key() {
    // This deterministic key exists only inside unit tests, never production resources.
    let key = SigningKey::from_bytes(&[42; 32]);
    let bytes = candidate().to_string().into_bytes();
    let envelope = json!({"schema_version":1,"algorithm":"ed25519","key_id":"fixture-only",
        "signature":hex(&key.sign(&bytes).to_bytes())})
    .to_string();
    let keyring = json!({"schema_version":1,"keys":[{"key_id":"fixture-only","public_key":hex(key.verifying_key().as_bytes())}]}).to_string();
    assert_eq!(verify_manifest(&bytes, &envelope, &keyring), "");
    let mut changed = bytes.clone();
    changed.push(b' ');
    assert_eq!(
        verify_manifest(&changed, &envelope, &keyring),
        "signature_mismatch"
    );
    assert_eq!(
        verify_manifest(&bytes, &envelope, "{\"schema_version\":1,\"keys\":[]}"),
        "untrusted_signing_key"
    );
    let mut duplicate: Value = serde_json::from_str(&keyring).unwrap();
    let first = duplicate["keys"][0].clone();
    duplicate["keys"].as_array_mut().unwrap().push(first);
    assert_eq!(
        verify_manifest(&bytes, &envelope, &duplicate.to_string()),
        "invalid_trust_keyring"
    );
    assert!(!verify_manifest(&bytes, &envelope.replace("ed25519", "other"), &keyring).is_empty());
    let weak =
        json!({"schema_version":1,"keys":[{"key_id":"fixture-only","public_key":"00".repeat(32)}]});
    assert_eq!(
        verify_manifest(&bytes, &envelope, &weak.to_string()),
        "invalid_trust_keyring"
    );
}
