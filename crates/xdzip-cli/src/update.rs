//! Explicit, bounded HTTPS updates with exact-byte publisher authentication.
use crate::{
    desktop,
    installation::{self, Context},
    output::{Failure, Response, Result},
    terminal,
};
use clap::ArgMatches;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use ureq::{
    Agent,
    http::Uri,
    tls::{RootCerts, TlsConfig, TlsProvider},
};

const TRUST: &str = include_str!("../../../resources/update-trust.json");
const REPOSITORY: &str = xdzip_update::REPOSITORY;
const PACKAGE_LIMIT: u64 = 2 * 1024u64.pow(3);
#[cfg(test)]
mod tests;

fn uri(value: &str) -> Result<Uri> {
    if value.len() > 8192 || value.contains(['#', '\\']) || value.chars().any(char::is_control) {
        return Err("invalid_url".into());
    }
    let uri: Uri = value.parse().map_err(|_| Failure::new("invalid_url"))?;
    let authority = uri.authority().ok_or("invalid_url")?;
    if uri.scheme_str() != Some("https")
        || authority.as_str().contains('@')
        || uri.port_u16().is_some_and(|port| port != 443)
    {
        return Err("invalid_url".into());
    }
    Ok(uri)
}
fn official(value: &str) -> Result<()> {
    let parsed = uri(value)?;
    if ![
        "api.github.com",
        "github.com",
        "objects.githubusercontent.com",
        "release-assets.githubusercontent.com",
        "github-releases.githubusercontent.com",
    ]
    .contains(&parsed.host().unwrap_or_default())
    {
        return Err("invalid_update_host".into());
    }
    Ok(())
}
struct Transport {
    agent: Agent,
    mirror: Option<String>,
}
impl Transport {
    fn new(matches: &ArgMatches) -> Result<Self> {
        let proxy = if matches.get_flag("no-proxy") {
            None
        } else if let Some(value) = matches.get_one::<String>("proxy") {
            Some(ureq::Proxy::new(value).map_err(|_| Failure::usage("invalid_proxy"))?)
        } else {
            ureq::Proxy::try_from_env()
        };
        if let Some(proxy) = &proxy
            && (proxy.username().is_some()
                || proxy.password().is_some()
                || !["http", "socks5"].contains(&proxy.uri().scheme_str().unwrap_or_default()))
        {
            return Err(Failure::usage("invalid_proxy"));
        }
        let mirror = matches.get_one::<String>("mirror").cloned();
        if let Some(value) = &mirror
            && uri(value)
                .map_err(|_| Failure::usage("invalid_mirror"))?
                .query()
                .is_some()
        {
            return Err(Failure::usage("invalid_mirror"));
        }
        let config = Agent::config_builder()
            .tls_config(
                TlsConfig::builder()
                    .provider(TlsProvider::NativeTls)
                    .root_certs(RootCerts::PlatformVerifier)
                    .build(),
            )
            .https_only(true)
            .http_status_as_error(false)
            .max_redirects(0)
            .proxy(proxy)
            .timeout_global(Some(Duration::from_secs(3600)))
            .timeout_connect(Some(Duration::from_secs(15)))
            .timeout_per_read(Some(Duration::from_secs(30)))
            .user_agent(format!("xdzip/{} CLI", env!("XDZIP_DISPLAY_VERSION")))
            .build();
        Ok(Self {
            agent: config.into(),
            mirror,
        })
    }
    fn response(&self, original: &str, download: bool) -> Result<ureq::http::Response<ureq::Body>> {
        official(original)?;
        let mut address = self.mirror.as_ref().map_or_else(
            || original.to_owned(),
            |mirror| format!("{}/{original}", mirror.trim_end_matches('/')),
        );
        let deadline = Instant::now() + Duration::from_secs(if download { 3600 } else { 30 });
        for _ in 0..=5 {
            if terminal::interrupted() {
                return Err("cancelled".into());
            }
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .ok_or("timeout")?;
            let parsed = uri(&address)?;
            let mirror_host = self
                .mirror
                .as_ref()
                .and_then(|s| uri(s).ok())
                .and_then(|u| u.host().map(str::to_owned));
            if mirror_host.as_deref() != parsed.host() {
                official(&address)?;
            }
            let mut response = self
                .agent
                .get(&address)
                .config()
                .timeout_global(Some(remaining))
                .timeout_per_read(Some(Duration::from_secs(if download { 30 } else { 15 })))
                .build()
                .call()
                .map_err(|error| match error {
                    ureq::Error::Timeout(_) => Failure::new("timeout"),
                    ureq::Error::NativeTls(_) => Failure::new("tls"),
                    _ => Failure::new("network"),
                })?;
            let status = response.status().as_u16();
            if [301, 302, 303, 307, 308].contains(&status) {
                let location = response
                    .headers()
                    .get("location")
                    .and_then(|s| s.to_str().ok())
                    .ok_or("invalid_redirect")?;
                address = if location.starts_with("https://") {
                    location.to_owned()
                } else if location.starts_with("//") {
                    format!("https:{location}")
                } else if location.starts_with('/') {
                    format!("https://{}{location}", parsed.authority().unwrap())
                } else {
                    return Err("invalid_redirect".into());
                };
                continue;
            }
            if status == 429 {
                return Err("rate_limited".into());
            }
            if status == 403 {
                let quota = response
                    .headers()
                    .get("x-ratelimit-remaining")
                    .is_some_and(|s| s == "0");
                let retry = response
                    .headers()
                    .get("retry-after")
                    .and_then(|s| s.to_str().ok())
                    .is_some_and(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()));
                let mut bytes = vec![];
                response
                    .body_mut()
                    .as_reader()
                    .take(8 * 1024 * 1024 + 1)
                    .read_to_end(&mut bytes)?;
                let message = serde_json::from_slice::<Value>(&bytes)
                    .ok()
                    .and_then(|v| v["message"].as_str().map(str::to_ascii_lowercase));
                return Err(
                    if quota || retry || message.is_some_and(|s| s.contains("rate limit")) {
                        "rate_limited"
                    } else {
                        "network"
                    }
                    .into(),
                );
            }
            if status == 404 {
                return Err("not_found".into());
            }
            if status != 200 {
                return Err("network".into());
            }
            return Ok(response);
        }
        Err("redirect_limit".into())
    }
    fn bytes(&self, address: &str, limit: u64) -> Result<Vec<u8>> {
        let mut response = self.response(address, false)?;
        let mut result = vec![];
        response
            .body_mut()
            .as_reader()
            .take(limit + 1)
            .read_to_end(&mut result)?;
        if result.len() as u64 > limit {
            return Err("response_limit".into());
        }
        Ok(result)
    }
}
struct Candidate {
    inspection: xdzip_update::Inspection,
    manifest: Vec<u8>,
    signature: String,
    trusted: bool,
    reason: String,
}
fn platform() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(windows) {
        "windows"
    } else {
        "linux"
    }
}
fn architecture() -> &'static str {
    if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "x86_64"
    }
}
trait Metadata {
    fn fetch(&self, address: &str, limit: u64) -> Result<Vec<u8>>;
}
impl Metadata for Transport {
    fn fetch(&self, address: &str, limit: u64) -> Result<Vec<u8>> {
        self.bytes(address, limit)
    }
}
fn choose(
    transport: &impl Metadata,
    desktop: bool,
    trust: &str,
    package_type: Option<&str>,
) -> Result<Option<Candidate>> {
    let mut releases = vec![];
    for page in 1..=10 {
        let bytes = transport.fetch(
            &format!("https://api.github.com/repos/{REPOSITORY}/releases?per_page=100&page={page}"),
            8 * 1024 * 1024,
        )?;
        let values: Vec<Value> = serde_json::from_slice(&bytes)?;
        let count = values.len();
        releases.extend(values.into_iter().filter(|r| {
            r["draft"] == false
                && xdzip_update::release_tag_eligible(
                    r["tag_name"].as_str().unwrap_or_default(),
                    env!("XDZIP_DISPLAY_VERSION"),
                    env!("XDZIP_CHANNEL"),
                )
        }));
        if count < 100 {
            break;
        }
        if page == 10 {
            return Err("page_limit".into());
        }
    }
    releases.sort_by(|a, b| {
        let compare = xdzip_update::compare_versions(
            a["tag_name"]
                .as_str()
                .unwrap_or_default()
                .trim_start_matches('v'),
            b["tag_name"]
                .as_str()
                .unwrap_or_default()
                .trim_start_matches('v'),
        );
        match compare {
            1 => std::cmp::Ordering::Less,
            -1 => std::cmp::Ordering::Greater,
            _ => std::cmp::Ordering::Equal,
        }
    });
    for release in releases {
        let assets = release["assets"].as_array().ok_or("invalid_response")?;
        let name = if desktop {
            "release.json"
        } else {
            "cli-release.json"
        };
        let tag = release["tag_name"].as_str().ok_or("invalid_response")?;
        let base = format!("https://github.com/{REPOSITORY}/releases/download/{tag}");
        if !assets.iter().any(|a| a["name"] == name) {
            if desktop {
                return Err("missing_manifest".into());
            }
            continue;
        }
        let expected = format!("{base}/{name}");
        if !assets
            .iter()
            .any(|a| a["name"] == name && a["browser_download_url"] == expected)
        {
            return Err("invalid_manifest_url".into());
        }
        let bytes = transport.fetch(&expected, xdzip_update::MANIFEST_LIMIT as u64)?;
        let text =
            std::str::from_utf8(&bytes).map_err(|_| Failure::new("invalid_manifest_json"))?;
        let current_build = env!("XDZIP_BUILD").parse().unwrap();
        let inspection = if desktop {
            xdzip_update::inspect_release(
                text,
                env!("XDZIP_DISPLAY_VERSION"),
                current_build,
                env!("XDZIP_CHANNEL"),
                platform(),
                architecture(),
                package_type.unwrap_or(if cfg!(windows) {
                    "exe"
                } else if cfg!(target_os = "macos") {
                    "app.zip"
                } else {
                    "deb"
                }),
            )
        } else {
            xdzip_update::inspect_cli_release(
                text,
                env!("XDZIP_DISPLAY_VERSION"),
                current_build,
                env!("XDZIP_CHANNEL"),
                platform(),
                architecture(),
            )
        };
        if !inspection.valid {
            return Err(inspection.error.as_str().into());
        }
        if !inspection.newer {
            continue;
        }
        if !assets.iter().any(|a| {
            a["name"] == inspection.filename
                && a["size"].as_u64() == Some(inspection.bytes)
                && a["browser_download_url"] == inspection.asset_url
        }) {
            return Err("asset_identity_mismatch".into());
        }
        let signature_name = format!("{name}.sig");
        let signature_url = format!("{base}/{signature_name}");
        let signature = if assets
            .iter()
            .any(|a| a["name"] == signature_name && a["browser_download_url"] == signature_url)
        {
            String::from_utf8(transport.fetch(&signature_url, 4096)?)
                .map_err(|_| Failure::new("invalid_signature_envelope"))?
        } else {
            String::new()
        };
        let error = if signature.is_empty() {
            "missing_signature".into()
        } else if desktop {
            xdzip_update::verify_manifest(&bytes, &signature, trust)
        } else {
            xdzip_update::verify_cli_manifest(&bytes, &signature, trust)
        };
        let trusted = error.is_empty();
        if !trusted
            && ![
                "missing_signature",
                "unknown_signing_key",
                "signing_key_not_trusted",
                "empty_trust_keyring",
                "untrusted_signing_key",
            ]
            .contains(&error.as_str())
        {
            return Err(error.as_str().into());
        }
        return Ok(Some(Candidate {
            inspection,
            manifest: bytes,
            signature,
            trusted,
            reason: error,
        }));
    }
    Ok(None)
}
fn download(transport: &Transport, candidate: &Candidate, target: &Path) -> Result<PathBuf> {
    if !candidate.trusted {
        return Err("untrusted_update".into());
    }
    let mut response = transport.response(&candidate.inspection.asset_url, true)?;
    receive_package(response.body_mut().as_reader(), candidate, target)
}
fn receive_package(mut source: impl Read, candidate: &Candidate, target: &Path) -> Result<PathBuf> {
    if !candidate.trusted {
        return Err("untrusted_update".into());
    }
    let inspection = &candidate.inspection;
    if inspection.bytes == 0 || inspection.bytes > PACKAGE_LIMIT {
        return Err("package_limit".into());
    }
    let grant = xdzip_io::Grant::for_creation(target).map_err(|_| Failure::new("unsafe_target"))?;
    let name = xdzip_domain::path::PortablePath::parse(&inspection.filename)
        .ok_or("invalid_asset_name")?;
    if grant
        .existing(&name)
        .map_err(|_| Failure::new("unsafe_target"))?
        .is_some()
    {
        return Err("output_exists".into());
    }
    let mut workspace = grant.workspace(0).map_err(|_| Failure::new("output_io"))?;
    let result = (|| {
        let mut stage = workspace.stage().map_err(|_| Failure::new("output_io"))?;
        let mut sha = Sha256::new();
        let mut total = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            if terminal::interrupted() {
                return Err("cancelled".into());
            }
            let n = source.read(&mut buffer)?;
            if n == 0 {
                break;
            }
            total = total
                .checked_add(n as u64)
                .filter(|n| *n <= inspection.bytes)
                .ok_or("package_length_mismatch")?;
            sha.update(&buffer[..n]);
            stage.write_all(&buffer[..n])?;
        }
        if total != inspection.bytes || format!("{:x}", sha.finalize()) != inspection.sha256 {
            return Err("package_hash_mismatch".into());
        }
        let receipt = workspace
            .commit(stage, &name, None)
            .map_err(|_| Failure::new("output_io"))?;
        if receipt.cleanup_warning {
            return Err("cleanup_error".into());
        }
        Ok(target.join(&inspection.filename))
    })();
    let clean = workspace
        .cleanup()
        .map_err(|_| Failure::new("cleanup_error"));
    match result {
        Ok(path) => {
            clean?;
            Ok(path)
        }
        Err(error) => {
            let _ = clean;
            Err(error)
        }
    }
}
fn status(context: &Context) -> Result<Response> {
    match context {
        Context::Managed(root) => {
            let pointer = installation::read_json(&root.join(installation::POINTER))?;
            let receipt = root.join("update-result.json");
            if !installation::exists_leaf(&receipt)? {
                return Ok(Response::success(
                    "update",
                    json!({"identity":pointer["identity"]}),
                    "No update transaction recorded.".into(),
                ));
            }
            let receipt = installation::read_json(&receipt)?;
            let valid = receipt["status"] == "installed"
                && receipt["identity"] == pointer["identity"]
                && installation::current_identity(root)
                    .is_ok_and(|identity| identity == receipt["identity"]);
            let mut response = Response::success(
                "update",
                receipt,
                if valid {
                    "Installed update confirmed."
                } else {
                    "Update requires recovery."
                }
                .into(),
            );
            response.status = if valid { "installed" } else { "recovery" }.into();
            if !valid {
                response.exit = 1;
            }
            Ok(response)
        }
        Context::Desktop(_) => desktop::status(),
        Context::Portable => Err("manual_install".into()),
    }
}
fn activate(root: &Path, archive: &Path, candidate: &Candidate) -> Result<Value> {
    if terminal::interrupted() {
        return Err("cancelled".into());
    }
    let _lock = installation::UpdateLock::acquire(root)?;
    let previous = installation::read_json(&root.join(installation::POINTER))?;
    if previous["identity"]["build"].as_u64().unwrap_or(u64::MAX)
        >= candidate.inspection.build as u64
        || !xdzip_update::release_tag_eligible(
            &format!("v{}", candidate.inspection.version),
            previous["identity"]["version"].as_str().unwrap_or_default(),
            previous["identity"]["channel"].as_str().unwrap_or_default(),
        )
    {
        return Err("non_monotonic_build".into());
    }
    let source = crate::ensure_directory(
        &crate::ensure_directory(&root.join("cache"))?
            .join(format!("unpack-{}", installation::random_id()?)),
    )?;
    let mut runtime = crate::runtime::Runtime::new(false, false, false)?;
    runtime.open(archive.to_str().ok_or("non_utf8_path")?.to_owned())?;
    let result = runtime.task(
        "extract",
        xdzip_runtime::service::Command::Extract {
            session: runtime.session,
            options: xdzip_runtime::extraction::Options {
                target: source.to_str().ok_or("non_utf8_path")?.to_owned(),
                all: true,
                selected: vec![],
                directory: String::new(),
                create_folder: false,
                conflict: xdzip_runtime::extraction::ConflictPolicy::Ask,
                allow_high_ratio: false,
            },
        },
    )?;
    if result.exit != 0 {
        return Err("invalid_update_payload".into());
    }
    let payload = installation::payload(&source)?;
    let manifest: Value = serde_json::from_slice(&candidate.manifest)?;
    if payload.identity["version"] != candidate.inspection.version
        || payload.identity["build"] != candidate.inspection.build
        || payload.identity["dirty"] != false
        || payload.identity["configuration"] != "release"
        || payload.identity["distribution"] != "release"
        || payload.identity["revision"].as_str()
            != manifest["commit"].as_str().and_then(|s| s.get(..12))
    {
        return Err("payload_identity_mismatch".into());
    }
    let (runtime, identity) = installation::install_version(root, &source)?;
    // Cancellation is accepted before the single activation commit. A signal
    // after that point cannot undo a version already selected by new processes.
    if terminal::interrupted() {
        return Err("cancelled".into());
    }
    let prepared = json!({"schema_version":1,"status":"prepared","identity":identity,"previous":previous,"runtime":runtime});
    installation::write_json(&root.join("update-result.json"), &prepared, true)?;
    installation::write_json(
        &root.join(installation::POINTER),
        &json!({"schema_version":1,"runtime":runtime,"identity":identity}),
        true,
    )?;
    let receipt = json!({"schema_version":1,"status":"installed","identity":identity});
    installation::write_json(&root.join("update-result.json"), &receipt, true)?;
    Ok(receipt)
}
pub fn run(matches: &ArgMatches, interactive: bool, chinese: bool) -> Result<Response> {
    let usage = installation::Usage::acquire()?;
    let context = installation::context()?;
    if matches.get_flag("status") {
        return status(&context);
    }
    let transport = Transport::new(matches)?;
    let system_supported = crate::system::update_supported();
    if !matches.get_flag("check") && !system_supported {
        return Err("unsupported_system".into());
    }
    let desktop_install = matches!(context, Context::Desktop(_));
    let desktop_info = if desktop_install {
        Some(desktop::installation_info()?)
    } else {
        None
    };
    if !matches.get_flag("check")
        && !matches.get_flag("download")
        && desktop_info
            .as_ref()
            .is_some_and(|info| info["installed"] != true)
    {
        return Err("manual_install".into());
    }
    let Some(candidate) = choose(
        &transport,
        desktop_install,
        TRUST,
        desktop_info
            .as_ref()
            .and_then(|info| info["package_type"].as_str()),
    )?
    else {
        let mut response = Response::success(
            "update",
            json!({"available":false}),
            if chinese {
                "没有适用于当前安装的新版本。"
            } else {
                "No newer compatible release is available."
            }
            .into(),
        );
        response.status = "up_to_date".into();
        return Ok(response);
    };
    if matches.get_flag("check") {
        let mut response = Response::success(
            "update",
            json!({"available":true,"version":candidate.inspection.version,"build":candidate.inspection.build,"trusted":candidate.trusted,"reason":candidate.reason,"release_url":candidate.inspection.release_url,"target":if desktop_install { "desktop" } else { "cli" }, "system_supported":system_supported}),
            format!("Update available: {}", candidate.inspection.version),
        );
        response.status = "available".into();
        return Ok(response);
    }
    if !candidate.trusted {
        return Err("untrusted_update".into());
    }
    if !matches.get_flag("download") {
        if env!("XDZIP_DIRTY") == "true"
            || env!("XDZIP_CONFIGURATION") != "release"
            || env!("XDZIP_DISTRIBUTION") != "release"
            || env!("XDZIP_REVISION") == "unknown"
        {
            return Err("manual_install".into());
        }
        if matches!(context, Context::Portable) {
            return Err("manual_install".into());
        }
        if desktop_install && !interactive {
            return Err("interaction_required".into());
        }
        if !matches.get_flag("yes") {
            if !interactive {
                return Err("interaction_required".into());
            }
            let answer = terminal::line(
                if chinese {
                    "安装本次更新？[y/N]："
                } else {
                    "Install this update? [y/N]: "
                },
                false,
                8,
            )?;
            if !answer.eq_ignore_ascii_case("y") {
                return Err("cancelled".into());
            }
        }
        if desktop_install {
            drop(usage);
            return desktop::request("update", Some(&candidate.inspection.version));
        }
    }
    let target = if let Some(path) = matches.get_one::<String>("output") {
        crate::ensure_directory(Path::new(path))?
    } else {
        crate::ensure_directory(
            &installation::cache_root()?.join(format!("download-{}", installation::random_id()?)),
        )?
    };
    let archive = download(&transport, &candidate, &target)?;
    installation::atomic(
        &target.join(if desktop_install {
            "release.json"
        } else {
            "cli-release.json"
        }),
        &candidate.manifest,
        false,
        false,
    )?;
    installation::atomic(
        &target.join(if desktop_install {
            "release.json.sig"
        } else {
            "cli-release.json.sig"
        }),
        candidate.signature.as_bytes(),
        false,
        false,
    )?;
    if matches.get_flag("download") {
        let mut response = Response::success(
            "update",
            json!({"path":archive,"version":candidate.inspection.version,"verified":true}),
            format!("Verified download: {}", archive.display()),
        );
        response.status = "downloaded".into();
        return Ok(response);
    }
    if let Context::Managed(root) = context {
        let receipt = activate(&root, &archive, &candidate)?;
        let mut response = Response::success(
            "update",
            receipt,
            "Update installed. The next invocation uses the new version.".into(),
        );
        response.status = "installed".into();
        return Ok(response);
    }
    Err("manual_install".into())
}
