//! Private deterministic signing keys and native payload fixtures never enter products.
use super::*;
use ed25519_dalek::{Signer, SigningKey};
use std::{collections::BTreeMap, fs, process::Command};

struct MetadataFixture {
    files: BTreeMap<String, Vec<u8>>,
    failure: Option<&'static str>,
}
impl Metadata for MetadataFixture {
    fn fetch(&self, address: &str, limit: u64) -> Result<Vec<u8>> {
        if let Some(error) = self.failure {
            return Err(error.into());
        }
        let bytes = self.files.get(address).ok_or("not_found")?.clone();
        if bytes.len() as u64 > limit {
            return Err("response_limit".into());
        }
        Ok(bytes)
    }
}
fn signed(bytes: &[u8]) -> (MetadataFixture, String) {
    let key = SigningKey::from_bytes(&[63; 32]);
    let hex = |data: &[u8]| data.iter().map(|b| format!("{b:02x}")).collect::<String>();
    let version = "65535.65535.65535";
    let build = env!("XDZIP_BUILD").parse::<u32>().unwrap() + 1;
    let mut assets = vec![];
    for (platform, arch, kind, minimum) in [
        ("windows", "x86_64", "zip", json!({"windows":"11"})),
        ("macos", "arm64", "tar.gz", json!({"macos":"14.0"})),
        (
            "linux",
            "x86_64",
            "tar.gz",
            json!({"distribution":"ubuntu","version":"24.04","glibc":"2.39"}),
        ),
    ] {
        let name = format!("xdzip-cli-{version}-{platform}-{arch}.{kind}");
        assets.push(json!({"filename":name,"platform":platform,"architecture":arch,"package_type":kind,"minimum_system":minimum,"bytes":bytes.len(),"sha256":format!("{:x}",Sha256::digest(bytes)),"url":format!("https://github.com/{REPOSITORY}/releases/download/v{version}/{name}")}));
    }
    let manifest = json!({"schema_version":1,"product":"xdzip","component":"cli","version":version,"channel":"stable","build":build,"tag":format!("v{version}"),"commit":"ab".repeat(20),"repository":REPOSITORY,"assets":assets});
    let original = serde_json::to_vec(&manifest).unwrap();
    let signature = json!({"schema_version":1,"algorithm":"ed25519","key_id":"private-cli-fixture","signature":hex(&key.sign(&original).to_bytes())});
    let trust = json!({"schema_version":1,"keys":[{"key_id":"private-cli-fixture","public_key":hex(key.verifying_key().as_bytes())}]}).to_string();
    let base = format!("https://github.com/{REPOSITORY}/releases/download/v{version}");
    let mut api_assets: Vec<Value> = manifest["assets"].as_array().unwrap().iter().map(|asset|json!({"name":asset["filename"],"size":asset["bytes"],"browser_download_url":asset["url"]})).collect();
    for name in ["cli-release.json", "cli-release.json.sig"] {
        api_assets.push(json!({"name":name,"browser_download_url":format!("{base}/{name}")}));
    }
    let api = serde_json::to_vec(
        &json!([{"draft":false,"tag_name":format!("v{version}"),"assets":api_assets}]),
    )
    .unwrap();
    (
        MetadataFixture {
            files: BTreeMap::from([
                (
                    format!(
                        "https://api.github.com/repos/{REPOSITORY}/releases?per_page=100&page=1"
                    ),
                    api,
                ),
                (format!("{base}/cli-release.json"), original),
                (
                    format!("{base}/cli-release.json.sig"),
                    serde_json::to_vec(&signature).unwrap(),
                ),
            ]),
            failure: None,
        },
        trust,
    )
}
fn temporary() -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../build/test-tmp")
        .join(format!("cli-update-{}", installation::random_id().unwrap()));
    fs::create_dir_all(&path).unwrap();
    #[cfg(windows)]
    {
        let user = Command::new("whoami")
            .args(["/user", "/fo", "csv", "/nh"])
            .output()
            .unwrap();
        assert!(user.status.success());
        let sid = String::from_utf8_lossy(&user.stdout)
            .trim()
            .rsplit(',')
            .next()
            .unwrap()
            .trim_matches('"')
            .to_owned();
        assert!(sid.starts_with("S-1-"));
        assert!(
            Command::new("icacls")
                .arg(&path)
                .args(["/inheritance:r", "/grant:r"])
                .arg(format!("*{sid}:(OI)(CI)F"))
                .stdout(std::process::Stdio::null())
                .status()
                .unwrap()
                .success()
        );
    }
    crate::ensure_directory(&path.canonicalize().unwrap()).unwrap()
}
#[test]
fn exact_signed_selection_network_errors_and_host_validation() {
    let (mut fixture, trust) = signed(b"owned package");
    let candidate = choose(&fixture, false, &trust, None).unwrap().unwrap();
    assert!(candidate.trusted);
    for address in [
        "http://github.com/a",
        "https://github.com.evil.test/a",
        "https://user:pass@github.com/a",
        "https://github.com/a#fragment",
        "https://github.com:444/a",
    ] {
        assert!(official(address).is_err());
    }
    assert!(official("https://release-assets.githubusercontent.com/owned").is_ok());
    for failure in ["network", "rate_limited", "timeout", "tls"] {
        fixture.failure = Some(failure);
        assert_eq!(
            choose(&fixture, false, &trust, None).err().unwrap().code,
            failure
        );
    }
    fixture.failure = None;
    let manifest = fixture
        .files
        .iter_mut()
        .find(|(name, _)| name.ends_with("/cli-release.json"))
        .unwrap()
        .1;
    manifest.push(b' ');
    assert_eq!(
        choose(&fixture, false, &trust, None).err().unwrap().code,
        "signature_mismatch"
    );
}
#[test]
fn verified_download_commits_only_exact_bytes_and_preserves_existing_files() {
    let root = temporary();
    let (fixture, trust) = signed(b"owned package");
    let candidate = choose(&fixture, false, &trust, None).unwrap().unwrap();
    assert!(receive_package(&b"tampered"[..], &candidate, &root).is_err());
    assert!(!root.join(&candidate.inspection.filename).exists());
    let path = receive_package(&b"owned package"[..], &candidate, &root).unwrap();
    assert_eq!(fs::read(&path).unwrap(), b"owned package");
    assert_eq!(
        receive_package(&b"owned package"[..], &candidate, &root)
            .err()
            .unwrap()
            .code,
        "output_exists"
    );
    assert_eq!(fs::read(&path).unwrap(), b"owned package");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn native_payload_install_switches_pointer_after_validation_and_keeps_old_runtime() {
    let root = temporary();
    let source = crate::ensure_directory(&root.join("source")).unwrap();
    let bin = crate::ensure_directory(&source.join("bin")).unwrap();
    let identity = json!({"product":"xdzip","component":"cli","version":"65535.65535.65535","build":env!("XDZIP_BUILD").parse::<u32>().unwrap()+1,"revision":"ab".repeat(6),"dirty":false,"channel":"stable","configuration":"release","platform":std::env::consts::OS,"architecture":std::env::consts::ARCH});
    let cpp = root.join("fixture.cpp");
    fs::write(
        &cpp,
        format!(
            "#include <cstdio>\nint main() {{ std::puts(R\"JSON({identity})JSON\"); return 0; }}\n"
        ),
    )
    .unwrap();
    let binary = bin.join(installation::executable("xdzip"));
    let compile_status = if cfg!(windows) {
        Command::new("cl")
            .args(["/nologo", "/EHsc"])
            .arg(&cpp)
            .arg(format!("/Fe:{}", binary.display()))
            .current_dir(&root)
            .stdout(std::process::Stdio::null())
            .status()
    } else {
        Command::new("c++")
            .arg(&cpp)
            .arg("-o")
            .arg(&binary)
            .current_dir(&root)
            .status()
    }
    .unwrap();
    assert!(compile_status.success());
    for name in ["xdzip-native-codec", "xdzip-launcher"] {
        fs::copy(&binary, bin.join(installation::executable(name))).unwrap();
    }
    let files:Vec<Value> = ["xdzip","xdzip-native-codec","xdzip-launcher"].iter().map(|name| { let path=format!("bin/{}",installation::executable(name)); json!({"sha256":installation::digest(&source.join(&path)).unwrap(),"path":path,"executable":true}) }).collect();
    installation::write_json(
        &source.join(installation::PAYLOAD),
        &json!({"schema_version":1,"product":"xdzip","identity":identity,"files":files}),
        false,
    )
    .unwrap();
    let archive = root.join("owned.tar.gz");
    let mut runtime = crate::runtime::Runtime::new(false, false, false).unwrap();
    let response = runtime
        .task(
            "create",
            xdzip_runtime::service::Command::Create(xdzip_runtime::creation::Options {
                inputs: vec![
                    bin.to_str().unwrap().into(),
                    source.join(installation::PAYLOAD).to_str().unwrap().into(),
                ],
                target: root.to_str().unwrap().into(),
                name: "owned.tar.gz".into(),
                method: 8,
                level: 1,
                conflict: xdzip_runtime::extraction::ConflictPolicy::Ask,
                volume_bytes: 0,
                solid: false,
                encrypt_headers: false,
                password: None,
            }),
        )
        .unwrap();
    assert_eq!(response.exit, 0);
    drop(runtime);
    let bytes = fs::read(&archive).unwrap();
    let (fixture, trust) = signed(&bytes);
    let candidate = choose(&fixture, false, &trust, None).unwrap().unwrap();
    let managed = crate::ensure_directory(&root.join("managed")).unwrap();
    let old = crate::ensure_directory(&managed.join("versions/old")).unwrap();
    fs::write(old.join("keep"), b"active old runtime").unwrap();
    installation::write_json(
        &managed.join(installation::POINTER),
        &json!({"schema_version":1,"runtime":"old","identity":crate::output::identity()}),
        false,
    )
    .unwrap();
    // The transport is mocked; actual archive creation, secure extraction,
    // native self-check, filesystem commit, receipt and pointer switch execute.
    let received = receive_package(
        &bytes[..],
        &candidate,
        &crate::ensure_directory(&root.join("download")).unwrap(),
    )
    .unwrap();
    let result = activate(&managed, &received, &candidate).unwrap();
    assert_eq!(result["status"], "installed");
    assert_eq!(
        installation::read_json(&managed.join(installation::POINTER)).unwrap()["identity"],
        identity
    );
    assert_eq!(fs::read(old.join("keep")).unwrap(), b"active old runtime");
    assert_eq!(
        activate(&managed, &received, &candidate)
            .err()
            .unwrap()
            .code,
        "non_monotonic_build"
    );
    assert_eq!(
        status(&Context::Managed(managed.clone())).unwrap().status,
        "installed"
    );
    let pointer = installation::read_json(&managed.join(installation::POINTER)).unwrap();
    let worker = managed
        .join("versions")
        .join(pointer["runtime"].as_str().unwrap())
        .join("bin")
        .join(installation::executable("xdzip-native-codec"));
    fs::write(worker, b"changed after installation").unwrap();
    assert_eq!(
        status(&Context::Managed(managed)).unwrap().status,
        "recovery"
    );
    fs::remove_dir_all(root).unwrap();
}
