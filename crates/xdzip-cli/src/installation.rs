//! User-owned versioned installation and explicit command registration.
use crate::{
    absolute, ensure_directory,
    output::{Failure, Response, Result},
};
use clap::ArgMatches;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::Command,
};
use xdzip_domain::path::PortablePath;

pub const CONTEXT: &str = "xdzip-cli-context.json";
pub const POINTER: &str = "current.json";
pub const PAYLOAD: &str = "cli-payload.json";
pub const REGISTRATION: &str = "xdzip-command.json";

pub fn executable(name: &str) -> String {
    format!("{name}{}", if cfg!(windows) { ".exe" } else { "" })
}
pub fn random_id() -> Result<String> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| Failure::new("random_unavailable"))?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
pub fn exists_leaf(path: &Path) -> Result<bool> {
    match fs::symlink_metadata(path) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(_) => Err("io_error".into()),
    }
}
pub fn read_bytes(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let file = xdzip_io::open_regular_nofollow(path)?;
    if file.metadata()?.len() > limit {
        return Err("file_limit".into());
    }
    let mut bytes = vec![];
    file.take(limit + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > limit {
        return Err("file_limit".into());
    }
    Ok(bytes)
}
pub fn read_json(path: &Path) -> Result<Value> {
    Ok(serde_json::from_slice(&read_bytes(path, 256 * 1024)?)?)
}
pub fn digest(path: &Path) -> Result<String> {
    let mut file = xdzip_io::open_regular_nofollow(path)?;
    let mut sha = Sha256::new();
    let mut bytes = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        sha.update(&bytes[..n]);
    }
    Ok(format!("{:x}", sha.finalize()))
}
pub fn atomic(path: &Path, bytes: &[u8], replace: bool, executable_file: bool) -> Result<()> {
    let grant = xdzip_io::Grant::for_creation(path.parent().ok_or("unsafe_target")?)
        .map_err(|_| Failure::new("unsafe_target"))?;
    let name = PortablePath::parse(
        path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("unsafe_target")?,
    )
    .ok_or("unsafe_target")?;
    let previous = grant
        .existing(&name)
        .map_err(|_| Failure::new("unsafe_target"))?;
    if previous.is_some() && !replace {
        return Err("command_conflict".into());
    }
    let mut workspace = grant.workspace(0).map_err(|_| Failure::new("output_io"))?;
    let result = (|| {
        let mut stage = workspace.stage().map_err(|_| Failure::new("output_io"))?;
        stage.write_all(bytes)?;
        #[cfg(unix)]
        if executable_file {
            stage
                .restore_mode(0o755)
                .map_err(|_| Failure::new("output_io"))?;
        }
        #[cfg(not(unix))]
        let _ = executable_file;
        let receipt = workspace
            .commit(stage, &name, previous.as_ref())
            .map_err(|_| Failure::new("target_changed"))?;
        if receipt.cleanup_warning {
            return Err("cleanup_error".into());
        }
        Ok(())
    })();
    let cleaned = workspace
        .cleanup()
        .map_err(|_| Failure::new("cleanup_error"));
    result.and(cleaned)
}
pub fn write_json(path: &Path, value: &Value, replace: bool) -> Result<()> {
    atomic(path, &serde_json::to_vec(value)?, replace, false)
}
pub fn data_root() -> Result<PathBuf> {
    #[cfg(windows)]
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .ok_or("user_directory_unavailable")?;
    #[cfg(target_os = "macos")]
    let base = std::env::var_os("HOME")
        .map(|p| PathBuf::from(p).join("Library/Application Support"))
        .ok_or("user_directory_unavailable")?;
    #[cfg(all(unix, not(target_os = "macos")))]
    let base = std::env::var_os("XDG_DATA_HOME")
        .filter(|p| Path::new(p).is_absolute())
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|p| PathBuf::from(p).join(".local/share")))
        .ok_or("user_directory_unavailable")?;
    absolute(base.join("xdzip/cli"))
}
pub fn cache_root() -> Result<PathBuf> {
    let root = if let Context::Managed(root) = context()? {
        root
    } else {
        data_root()?
    };
    ensure_directory(&root.join("cache"))
}

#[derive(Clone, Debug)]
pub enum Context {
    Portable,
    Managed(PathBuf),
    Desktop(PathBuf),
}
pub fn context() -> Result<Context> {
    context_for(&std::env::current_exe()?)
}
fn desktop_context_path(bin: &Path) -> Option<PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let helpers = bin.parent()?;
        let contents = helpers.parent()?;
        (bin.file_name()? == "cli"
            && helpers.file_name()? == "Helpers"
            && contents.file_name()? == "Contents")
            .then(|| contents.join("Resources").join(CONTEXT))
    }
    #[cfg(not(target_os = "macos"))]
    {
        Some(bin.join(CONTEXT))
    }
}
pub fn context_for(exe: &Path) -> Result<Context> {
    let bin = exe.parent().ok_or("invalid_installation")?;
    if let Some(context_path) = desktop_context_path(bin)
        && exists_leaf(&context_path)?
    {
        let context = read_json(&context_path)?;
        if context["schema_version"] != 1 || context["kind"] != "desktop" {
            return Err("invalid_installation".into());
        }
        let relative = context["desktop_relative"]
            .as_str()
            .ok_or("invalid_installation")?;
        let expected = if cfg!(windows) {
            "../xdzip.exe"
        } else if cfg!(target_os = "macos") {
            "../../MacOS/xdzip"
        } else {
            "../xdzip"
        };
        if relative != expected {
            return Err("invalid_installation".into());
        }
        let gui = fs::canonicalize(bin.join(relative))?;
        if !fs::symlink_metadata(&gui)?.is_file() {
            return Err("invalid_installation".into());
        }
        return Ok(Context::Desktop(gui));
    }
    // Managed payloads are root/versions/<fixed-leaf>/bin/xdzip.
    if let Some(version) = bin.parent()
        && let Some(versions) = version.parent()
        && versions.file_name().is_some_and(|n| n == "versions")
        && let Some(root) = versions.parent()
    {
        let marker_path = root.join("installation.json");
        if !exists_leaf(&marker_path)? {
            return Ok(Context::Portable);
        }
        let marker = read_json(&marker_path)?;
        if marker["schema_version"] == 1
            && marker["product"] == "xdzip"
            && marker["component"] == "cli"
        {
            xdzip_io::Grant::for_creation(root).map_err(|_| Failure::new("unsafe_installation"))?;
            return Ok(Context::Managed(root.to_owned()));
        }
        return Err("invalid_installation".into());
    }
    Ok(Context::Portable)
}

#[cfg(test)]
mod context_tests {
    use super::*;

    struct Desktop {
        root: PathBuf,
        bin: PathBuf,
        gui: PathBuf,
        marker: PathBuf,
    }
    impl Desktop {
        fn new() -> Self {
            let root = Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../build/test-tmp")
                .join(format!("cli-context-{}", random_id().unwrap()));
            let (bin, gui, marker) = if cfg!(target_os = "macos") {
                let contents = root.join("Owned.app/Contents");
                (
                    contents.join("Helpers/cli"),
                    contents.join("MacOS/xdzip"),
                    contents.join("Resources").join(CONTEXT),
                )
            } else {
                (
                    root.join("cli"),
                    root.join(executable("xdzip")),
                    root.join("cli").join(CONTEXT),
                )
            };
            fs::create_dir_all(&bin).unwrap();
            fs::create_dir_all(gui.parent().unwrap()).unwrap();
            fs::create_dir_all(marker.parent().unwrap()).unwrap();
            fs::write(&gui, b"owned GUI target").unwrap();
            Self {
                root,
                bin,
                gui,
                marker,
            }
        }
        fn write(&self) {
            let relative = if cfg!(target_os = "macos") {
                "../../MacOS/xdzip"
            } else if cfg!(windows) {
                "../xdzip.exe"
            } else {
                "../xdzip"
            };
            fs::write(
                &self.marker,
                serde_json::to_vec(&json!({
                    "schema_version": 1, "kind": "desktop", "desktop_relative": relative
                }))
                .unwrap(),
            )
            .unwrap();
        }
        fn context(&self) -> Result<Context> {
            context_for(&self.bin.join(executable("xdzip")))
        }
    }
    impl Drop for Desktop {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.root).unwrap();
        }
    }

    #[test]
    fn desktop_context_resolves_only_the_fixed_target_and_identity() {
        let fixture = Desktop::new();
        assert!(matches!(fixture.context().unwrap(), Context::Portable));
        fixture.write();
        assert!(matches!(fixture.context().unwrap(), Context::Desktop(path)
            if path == fs::canonicalize(&fixture.gui).unwrap()));
        for value in [
            json!({"schema_version": 2, "kind": "desktop", "desktop_relative": "../../MacOS/xdzip"}),
            json!({"schema_version": 1, "kind": "desktop", "desktop_relative": "../foreign"}),
        ] {
            fs::write(&fixture.marker, serde_json::to_vec(&value).unwrap()).unwrap();
            assert_eq!(fixture.context().unwrap_err().code, "invalid_installation");
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_context_requires_resources_and_the_helpers_bundle_layout() {
        let fixture = Desktop::new();
        fixture.write();
        fs::rename(&fixture.marker, fixture.bin.join(CONTEXT)).unwrap();
        assert!(matches!(fixture.context().unwrap(), Context::Portable));
        fixture.write();
        let contents = fixture.bin.parent().unwrap().parent().unwrap();
        let moved = contents.with_file_name("Other");
        fs::rename(contents, &moved).unwrap();
        assert!(matches!(
            context_for(&moved.join("Helpers/cli/xdzip")).unwrap(),
            Context::Portable
        ));
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadManifest {
    pub schema_version: u32,
    pub product: String,
    pub identity: Value,
    pub files: Vec<PayloadFile>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PayloadFile {
    pub path: String,
    pub sha256: String,
    pub executable: bool,
}
pub fn payload(path: &Path) -> Result<PayloadManifest> {
    let manifest: PayloadManifest =
        serde_json::from_slice(&read_bytes(&path.join(PAYLOAD), 256 * 1024)?)?;
    if manifest.schema_version != 1
        || manifest.product != "xdzip"
        || manifest.identity["product"] != "xdzip"
        || manifest.identity["component"] != "cli"
        || manifest.identity["platform"] != std::env::consts::OS
        || manifest.identity["architecture"] != std::env::consts::ARCH
        || manifest.files.len() > 2048
        || manifest.files.len() < 3
    {
        return Err("invalid_payload".into());
    }
    let mut names = std::collections::BTreeSet::new();
    let mut bytes = 0u64;
    for file in &manifest.files {
        if PortablePath::parse(&file.path).is_none()
            || !names.insert(&file.path)
            || file.path == PAYLOAD
        {
            return Err("invalid_payload".into());
        }
        let source = path.join(&file.path);
        bytes = bytes
            .checked_add(fs::symlink_metadata(&source)?.len())
            .filter(|n| *n <= 2 * 1024u64.pow(3))
            .ok_or("payload_limit")?;
        if digest(&source)? != file.sha256 {
            return Err("payload_hash_mismatch".into());
        }
    }
    for required in [
        format!("bin/{}", executable("xdzip")),
        format!("bin/{}", executable("xdzip-native-codec")),
        format!("bin/{}", executable("xdzip-launcher")),
    ] {
        if !names.contains(&required) {
            return Err("incomplete_payload".into());
        }
    }
    let mut pending = vec![path.to_owned()];
    let mut actual = std::collections::BTreeSet::new();
    let mut objects = 0;
    while let Some(directory) = pending.pop() {
        for item in fs::read_dir(directory)? {
            let item = item?;
            objects += 1;
            if objects > 4096 {
                return Err("payload_limit".into());
            }
            let kind = item.file_type()?;
            if kind.is_dir() {
                pending.push(item.path());
            } else if kind.is_file() {
                actual.insert(
                    item.path()
                        .strip_prefix(path)
                        .map_err(|_| Failure::new("invalid_payload"))?
                        .to_string_lossy()
                        .replace('\\', "/"),
                );
            } else {
                return Err("invalid_payload".into());
            }
        }
    }
    let expected: std::collections::BTreeSet<String> = manifest
        .files
        .iter()
        .map(|file| file.path.clone())
        .chain(std::iter::once(PAYLOAD.into()))
        .collect();
    if actual != expected {
        return Err("invalid_payload".into());
    }
    Ok(manifest)
}
pub fn install_version(root: &Path, source: &Path) -> Result<(String, Value)> {
    let manifest = payload(source)?;
    let revision = manifest.identity["revision"]
        .as_str()
        .ok_or("invalid_payload")?;
    if revision.len() != 12 || !revision.bytes().all(|c| c.is_ascii_hexdigit()) {
        return Err("invalid_payload".into());
    }
    let build = manifest.identity["build"]
        .as_u64()
        .ok_or("invalid_payload")?;
    let runtime = format!("b{build}-{revision}-{}", random_id()?);
    let versions = ensure_directory(&root.join("versions"))?;
    let destination = ensure_directory(&versions.join(&runtime))?;
    // Destination is a fresh, owner-only tree. Every write still uses held
    // parent capabilities, and every payload file is checked before activation.
    for file in &manifest.files {
        if crate::terminal::interrupted() {
            return Err("cancelled".into());
        }
        let output = destination.join(&file.path);
        ensure_directory(output.parent().ok_or("invalid_payload")?)?;
        let bytes = read_bytes(&source.join(&file.path), 128 * 1024 * 1024)?;
        if format!("{:x}", Sha256::digest(&bytes)) != file.sha256 {
            return Err("source_changed".into());
        }
        atomic(&output, &bytes, false, file.executable)?;
    }
    atomic(
        &destination.join(PAYLOAD),
        &serde_json::to_vec(&manifest)?,
        false,
        false,
    )?;
    let actual = self_check(&destination, &manifest.identity)?;
    Ok((runtime, actual))
}

fn self_check(directory: &Path, expected: &Value) -> Result<Value> {
    let identity = Command::new(directory.join("bin").join(executable("xdzip")))
        .arg("--version-json")
        .output()?;
    if !identity.status.success() || identity.stdout.len() > 16384 {
        return Err("self_check_failed".into());
    }
    let actual: Value = serde_json::from_slice(&identity.stdout)?;
    for key in [
        "product",
        "component",
        "version",
        "build",
        "revision",
        "dirty",
        "channel",
        "configuration",
        "distribution",
        "platform",
        "architecture",
    ] {
        if actual[key] != expected[key] {
            return Err("self_check_failed".into());
        }
    }
    Ok(actual)
}
pub fn current_identity(root: &Path) -> Result<Value> {
    let pointer = read_json(&root.join(POINTER))?;
    let name = pointer["runtime"].as_str().ok_or("invalid_installation")?;
    if name.is_empty() || !name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err("invalid_installation".into());
    }
    let directory = root.join("versions").join(name);
    let manifest = payload(&directory)?;
    if pointer["identity"] != manifest.identity {
        return Err("invalid_installation".into());
    }
    self_check(&directory, &manifest.identity)
}

#[cfg(windows)]
fn retired_records(directory: &Path) -> Result<Vec<Value>> {
    let index = directory.join("retired.json");
    if !exists_leaf(&index)? {
        return Ok(vec![]);
    }
    let record = read_json(&index)?;
    if record["schema_version"] != 1 || record["product"] != "xdzip" {
        return Err("command_conflict".into());
    }
    let files = record["files"].as_array().ok_or("command_conflict")?;
    if files.len() > 64 {
        return Err("resource_limit".into());
    }
    let mut names = std::collections::BTreeSet::new();
    for file in files {
        let name = file["file"].as_str().ok_or("command_conflict")?;
        let nonce = name.strip_suffix(".exe").ok_or("command_conflict")?;
        let sha = file["sha256"].as_str().ok_or("command_conflict")?;
        if nonce.len() != 32
            || !nonce.bytes().all(|b| b.is_ascii_hexdigit())
            || sha.len() != 64
            || !sha.bytes().all(|b| b.is_ascii_hexdigit())
            || !names.insert(name.to_ascii_lowercase())
        {
            return Err("command_conflict".into());
        }
    }
    Ok(files.clone())
}
#[cfg(windows)]
fn cleanup_retired(bin: &Path) -> Result<()> {
    let directory = bin.join(".xdzip-retired");
    if !exists_leaf(&directory)? {
        return Ok(());
    }
    xdzip_io::Grant::for_creation(&directory).map_err(|_| Failure::new("unsafe_target"))?;
    let files = retired_records(&directory)?;
    let mut retained = vec![];
    for record in &files {
        let path = directory.join(record["file"].as_str().unwrap());
        if !exists_leaf(&path)? {
            continue;
        }
        if record["sha256"].as_str() != Some(&digest(&path)?) {
            return Err("command_conflict".into());
        }
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => {
                retained.push(record.clone())
            }
            Err(error) => return Err(error.into()),
        }
    }
    if retained != files {
        write_json(
            &directory.join("retired.json"),
            &json!({"schema_version":1,"product":"xdzip","files":retained}),
            true,
        )?;
    }
    Ok(())
}
#[cfg(windows)]
fn retire_launcher(bin: &Path, entry: &Path, sha: &Value) -> Result<PathBuf> {
    let directory = ensure_directory(&bin.join(".xdzip-retired"))?;
    cleanup_retired(bin)?;
    let mut files = retired_records(&directory)?;
    if files.len() >= 64 {
        return Err("resource_limit".into());
    }
    let name = format!("{}.exe", random_id()?);
    let target = directory.join(&name);
    if exists_leaf(&target)? {
        return Err("command_conflict".into());
    }
    files.push(json!({"file":name,"sha256":sha}));
    // Windows retains the waiting launcher's image. Only the hash-checked owned
    // entry is moved within the authorized command directory. Commit its bounded
    // receipt first: a failed rename preserves the live registration, and the
    // next explicit setup can discard the receipt for an absent retired file.
    write_json(
        &directory.join("retired.json"),
        &json!({"schema_version":1,"product":"xdzip","files":files}),
        true,
    )?;
    fs::rename(entry, &target)?;
    Ok(target)
}

pub fn setup(matches: &ArgMatches) -> Result<Response> {
    let root = if let Some(path) = matches.get_one::<String>("install-dir") {
        absolute(path)?
    } else {
        data_root()?
    };
    let bin = if let Some(path) = matches.get_one::<String>("bin-dir") {
        absolute(path)?
    } else {
        #[cfg(windows)]
        let path = root.join("bin");
        #[cfg(unix)]
        let path = std::env::var_os("HOME")
            .map(|p| PathBuf::from(p).join(".local/bin"))
            .ok_or("user_directory_unavailable")?;
        path
    };
    let entry = bin.join(executable("xdzip"));
    let registration = bin.join(REGISTRATION);
    if matches.get_flag("remove") {
        let own = read_json(&registration)?;
        if own["product"] != "xdzip" || own["launcher_sha256"].as_str() != Some(&digest(&entry)?) {
            return Err("command_conflict".into());
        }
        xdzip_io::Grant::for_creation(&bin).map_err(|_| Failure::new("unsafe_target"))?;
        // Only exact, previously hashed registration files are removed. Payload
        // versions and user data are deliberately retained for active processes.
        let retained: Option<PathBuf> = match fs::remove_file(&entry) {
            Ok(()) => None,
            Err(error) => {
                #[cfg(windows)]
                {
                    if error.kind() != std::io::ErrorKind::PermissionDenied {
                        return Err(error.into());
                    }
                    Some(retire_launcher(&bin, &entry, &own["launcher_sha256"])?)
                }
                #[cfg(not(windows))]
                {
                    return Err(error.into());
                }
            }
        };
        fs::remove_file(&registration)?;
        return Ok(Response::success(
            "setup",
            json!({"removed":true,"retained_launcher":retained}),
            "Command registration removed.".into(),
        ));
    }
    if exists_leaf(&entry)? || exists_leaf(&registration)? {
        return Err("command_conflict".into());
    }
    ensure_directory(&bin)?;
    #[cfg(windows)]
    cleanup_retired(&bin)?;
    let exe = std::env::current_exe()?;
    let context = context()?;
    let mut record = json!({"schema_version":1,"product":"xdzip"});
    let launcher;
    match context {
        Context::Desktop(gui) => {
            record["kind"] = json!("desktop");
            record["executable"] = json!(exe);
            launcher = exe.parent().unwrap().join(executable("xdzip-launcher"));
            let state = ensure_directory(&data_root()?)?;
            write_json(
                &state.join("desktop.json"),
                &json!({"schema_version":1,"executable":gui}),
                true,
            )?;
        }
        Context::Managed(ref installed) => {
            record["kind"] = json!("managed");
            record["root"] = json!(installed);
            launcher = exe.parent().unwrap().join(executable("xdzip-launcher"));
        }
        Context::Portable => {
            ensure_directory(&root)?;
            if exists_leaf(&root.join("installation.json"))? {
                return Err("command_conflict".into());
            }
            let source = exe
                .parent()
                .and_then(Path::parent)
                .ok_or("invalid_payload")?;
            let (runtime, identity) = install_version(&root, source)?;
            write_json(
                &root.join("installation.json"),
                &json!({"schema_version":1,"product":"xdzip","component":"cli","install_id":random_id()?}),
                false,
            )?;
            write_json(
                &root.join(POINTER),
                &json!({"schema_version":1,"runtime":runtime,"identity":identity}),
                false,
            )?;
            record["kind"] = json!("managed");
            record["root"] = json!(root);
            launcher = source.join("bin").join(executable("xdzip-launcher"));
        }
    }
    let bytes = read_bytes(&launcher, 128 * 1024 * 1024)?;
    record["launcher_sha256"] = json!(format!("{:x}", Sha256::digest(&bytes)));
    // Sidecar is committed first; an interrupted registration cannot expose a
    // launcher that resolves an unknown or partially written runtime pointer.
    write_json(&registration, &record, false)?;
    atomic(&entry, &bytes, false, true)?;
    let on_path = std::env::var_os("PATH")
        .is_some_and(|paths| std::env::split_paths(&paths).any(|p| p == bin));
    Ok(Response::success(
        "setup",
        json!({"entry":entry,"on_path":on_path,"kind":record["kind"]}),
        format!(
            "Command registered: {}\n{}",
            entry.display(),
            if on_path {
                ""
            } else {
                "Add this directory to your user PATH, then open a new terminal."
            }
        ),
    ))
}

/// Exclusive transaction lock, automatically released on crash without stale PID files.
pub struct UpdateLock {
    _file: fs::File,
}
impl UpdateLock {
    pub fn acquire(root: &Path) -> Result<Self> {
        xdzip_io::Grant::for_creation(root).map_err(|_| Failure::new("unsafe_installation"))?;
        let mut options = fs::OpenOptions::new();
        options.read(true).write(true).create(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
        }
        #[cfg(windows)]
        {
            use std::os::windows::fs::OpenOptionsExt;
            options.custom_flags(0x00200000);
        }
        let file = options.open(root.join("update.lock"))?;
        #[cfg(windows)]
        {
            use std::os::windows::fs::MetadataExt;
            if file.metadata()?.file_attributes() & 0x400 != 0 {
                return Err("unsafe_installation".into());
            }
        }
        file.try_lock()
            .map_err(|_| Failure::new("installation_busy"))?;
        Ok(Self { _file: file })
    }
}

/// Platform installation occupancy. Handles live until all Rust tasks have joined.
pub struct Usage {
    #[cfg(unix)]
    _file: Option<fs::File>,
    #[cfg(windows)]
    handle: windows_sys::Win32::Foundation::HANDLE,
}
impl Usage {
    pub fn acquire() -> Result<Self> {
        #[cfg(unix)]
        {
            use std::os::{fd::AsRawFd, unix::fs::OpenOptionsExt};
            let mut file = None;
            if let Context::Desktop(_) = context()? {
                #[cfg(target_os = "macos")]
                let gate = || -> Result<()> {
                    let record = data_root()?.join("desktop-update-status.json");
                    if record.try_exists()? && read_json(&record)?["install_gate"] == true {
                        return Err("installation_busy".into());
                    }
                    Ok(())
                };
                #[cfg(target_os = "macos")]
                gate()?;
                let lock = if cfg!(target_os = "linux")
                    && std::env::current_exe()?.starts_with("/opt/xdzip/versions")
                {
                    PathBuf::from("/run/xdzip/usage.lock")
                } else {
                    ensure_directory(&data_root()?)?.join("desktop-usage.lock")
                };
                let handle = fs::OpenOptions::new()
                    .read(true)
                    .write(!lock.starts_with("/run/"))
                    .create(!lock.starts_with("/run/"))
                    .mode(0o600)
                    .custom_flags(libc::O_NOFOLLOW)
                    .open(lock)?;
                #[allow(unsafe_code)]
                // SAFETY: flock borrows a live file descriptor. Nonblocking admission
                // fails during installation; closing the file releases occupancy.
                if unsafe { libc::flock(handle.as_raw_fd(), libc::LOCK_SH | libc::LOCK_NB) } != 0 {
                    return Err("installation_busy".into());
                }
                #[cfg(target_os = "macos")]
                gate()?;
                file = Some(handle);
            }
            Ok(Self { _file: file })
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::{Foundation::*, System::Threading::*};
            let mut handle: HANDLE = std::ptr::null_mut();
            if matches!(context()?, Context::Desktop(_)) {
                #[allow(unsafe_code)]
                // SAFETY: fixed terminated mutex names, no borrowed data survives calls.
                unsafe {
                    let gate: Vec<u16> = "Local\\xdzip.desktop.updating\0".encode_utf16().collect();
                    let running: Vec<u16> =
                        "Local\\xdzip.desktop.running\0".encode_utf16().collect();
                    for pass in 0..2 {
                        let update = OpenMutexW(SYNCHRONIZATION_SYNCHRONIZE, 0, gate.as_ptr());
                        if !update.is_null() {
                            CloseHandle(update);
                            if !handle.is_null() {
                                CloseHandle(handle);
                            }
                            return Err("installation_busy".into());
                        }
                        if GetLastError() != ERROR_FILE_NOT_FOUND {
                            return Err("installation_busy".into());
                        }
                        if pass == 0 {
                            handle = CreateMutexW(std::ptr::null(), 0, running.as_ptr());
                            if handle.is_null() {
                                return Err("installation_busy".into());
                            }
                        }
                    }
                }
            }
            Ok(Self { handle })
        }
    }
}
#[cfg(windows)]
impl Drop for Usage {
    fn drop(&mut self) {
        #[allow(unsafe_code)]
        // SAFETY: owns this optional mutex handle, released exactly once.
        unsafe {
            if !self.handle.is_null() {
                windows_sys::Win32::Foundation::CloseHandle(self.handle);
            }
        }
    }
}
