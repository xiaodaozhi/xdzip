use super::*;
use sha2::{Digest, Sha256};
use std::{
    io::Read,
    process::{Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

fn fixture(root: &Path) -> std::path::PathBuf {
    let mut archive = b"XDP1".to_vec();
    archive.extend(2u32.to_le_bytes());
    for (name, data) in [
        ("目录/hello.txt", b"owned text\n".to_vec()),
        ("large.bin", vec![0x5a; 500_000]),
    ] {
        archive.extend((name.len() as u16).to_le_bytes());
        archive.extend((data.len() as u32).to_le_bytes());
        archive.extend(name.as_bytes());
        archive.extend(data);
    }
    let path = root.join("owned.xdp");
    std::fs::write(&path, archive).unwrap();
    path
}
fn reference() -> std::path::PathBuf {
    std::env::var_os("XDZIP_SDK_REFERENCE")
        .map(Into::into)
        .expect("run scripts/sdk-trial.py test after building the native peer")
}
fn package(root: &Path) -> std::path::PathBuf {
    let exe = reference();
    let bytes = std::fs::read(exe).unwrap();
    std::fs::write(root.join("peer"), &bytes).unwrap();
    std::fs::write(root.join("LICENSE"), b"owned Apache-2.0 fixture").unwrap();
    let manifest = serde_json::json!({"manifest_version":1,"plugin_id":"org.xdzip.sdk.reference","version":"0.1.0","author":"xdzip","protocol_majors":[1],"targets":[format!("{}-{}", std::env::consts::OS,std::env::consts::ARCH)],"entrypoint":"peer","sha256":crate::package::hex(&Sha256::digest(&bytes)),"formats":["xdp1"],"permissions":{"broker_source_read":true,"broker_output_write":true,"network":false},"resources":{"memory_bytes":536870912,"threads":1},"license_files":["LICENSE"],"source_url":"https://github.com/xiaodaozhi/xdzip"});
    let path = root.join("manifest.json");
    std::fs::write(&path, serde_json::to_vec(&manifest).unwrap()).unwrap();
    path
}
fn python(fault: &str) -> process::Peer {
    let mut command =
        Command::new(std::env::var_os("XDZIP_SDK_PYTHON").unwrap_or_else(|| "python3".into()));
    command.arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/sdk/reference.py"));
    if !fault.is_empty() {
        command.arg(fault);
    }
    process::Peer::trusted(command).unwrap()
}

#[test]
#[ignore = "explicit native SDK conformance entry point"]
fn independent_python_and_native_peers_interoperate_with_the_host() {
    let root = tempfile::tempdir().unwrap();
    let source = fixture(root.path());
    let target = root.path().join("python-output");
    std::fs::create_dir(&target).unwrap();
    let mut peer = python("");
    let report = broker::run(
        &mut peer,
        &source,
        Some((1, &target)),
        "org.xdzip.sdk.reference",
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(report.committed);
    assert_eq!(report.entries, 2);
    assert_eq!(report.output_bytes, 500_000);
    assert_eq!(
        std::fs::read(target.join("large.bin")).unwrap(),
        vec![0x5a; 500_000]
    );
    let mut peer = process::Peer::trusted(Command::new(reference())).unwrap();
    let report = broker::run(
        &mut peer,
        &source,
        None,
        "org.xdzip.sdk.reference",
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.entries, 2);
    assert!(!report.committed);
}

#[test]
#[ignore = "explicit native SDK conformance entry point"]
fn malicious_wire_io_and_incomplete_terminals_never_publish() {
    for fault in [
        "version",
        "required",
        "frame",
        "source",
        "overflow",
        "replay",
        "operation",
        "duplicate",
        "path",
        "incomplete",
        "sink",
        "credit",
        "crash",
        "digest",
        "terminal",
    ] {
        let root = tempfile::tempdir().unwrap();
        let source = fixture(root.path());
        let target = root.path().join("output");
        std::fs::create_dir(&target).unwrap();
        let mut peer = python(fault);
        let result = broker::run(
            &mut peer,
            &source,
            Some((1, &target)),
            "org.xdzip.sdk.reference",
            &AtomicBool::new(false),
        );
        assert!(result.is_err(), "fault accepted: {fault}");
        drop(peer);
        assert_eq!(
            std::fs::read_dir(&target).unwrap().count(),
            0,
            "leftover: {fault}"
        );
    }
}

#[test]
#[ignore = "explicit native SDK conformance entry point"]
fn cancellation_revokes_output_and_kills_an_uncooperative_peer() {
    let root = tempfile::tempdir().unwrap();
    let source = fixture(root.path());
    let target = root.path().join("output");
    std::fs::create_dir(&target).unwrap();
    let cancel = Arc::new(AtomicBool::new(false));
    let set = Arc::clone(&cancel);
    let trigger = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(250));
        set.store(true, Ordering::Release);
    });
    let mut peer = python("hang");
    let start = Instant::now();
    assert_eq!(
        broker::run(
            &mut peer,
            &source,
            Some((1, &target)),
            "org.xdzip.sdk.reference",
            &cancel
        )
        .unwrap_err(),
        Error::Cancelled
    );
    drop(peer);
    trigger.join().unwrap();
    assert!(start.elapsed() < Duration::from_secs(4));
    assert_eq!(std::fs::read_dir(target).unwrap().count(), 0);
}

#[test]
#[ignore = "explicit native SDK conformance entry point"]
fn native_kernel_gate_and_owned_permission_attacks() {
    let root = tempfile::tempdir().unwrap();
    let manifest = package(root.path());
    let source = fixture(root.path());
    if platform_gate() == "disabled" {
        assert_eq!(
            run(&manifest, &source, None, &AtomicBool::new(false)).unwrap_err(),
            Error::Sandbox
        );
        return;
    }
    let target = root.path().join("output");
    std::fs::create_dir(&target).unwrap();
    let report = run(
        &manifest,
        &source,
        Some((0, &target)),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(report.output_bytes, 11);
    assert_eq!(
        std::fs::read(target.join("目录/hello.txt")).unwrap(),
        b"owned text\n"
    );
    let package = package::Package::load(&manifest).unwrap();
    let read = root.path().join("forbidden-read");
    let write = root.path().join("forbidden-write");
    std::fs::write(&read, b"private owned marker").unwrap();
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    // Positive control makes each denial meaningful: the same owned resources
    // and self-exec are accessible before kernel policy installation.
    let control = Command::new(reference())
        .arg("--sandbox-audit")
        .arg(&read)
        .arg(&write)
        .arg(listener.local_addr().unwrap().port().to_string())
        .output()
        .unwrap();
    assert_eq!(control.status.code(), Some(1));
    let admitted: serde_json::Value = serde_json::from_slice(&control.stdout).unwrap();
    for field in [
        "read_denied",
        "write_denied",
        "network_denied",
        "descendant_denied",
        "fork_denied",
    ] {
        assert_eq!(admitted[field], false);
    }
    assert_eq!(std::fs::read(&write).unwrap(), b"forbidden");
    std::fs::remove_file(&write).unwrap();
    assert!(listener.accept().is_ok());
    let mut command = sandbox::command(&package.executable).unwrap();
    command
        .arg("--sandbox-audit")
        .arg(&read)
        .arg(&write)
        .arg(listener.local_addr().unwrap().port().to_string())
        .env_clear();
    sandbox::configure(&mut command).unwrap();
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if start.elapsed() > Duration::from_secs(3) {
            let _ = child.kill();
            let _ = child.wait();
            panic!("kernel audit timed out");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    let mut output = String::new();
    child
        .stdout
        .take()
        .unwrap()
        .read_to_string(&mut output)
        .unwrap();
    let audit: serde_json::Value = serde_json::from_str(&output).unwrap();
    for field in [
        "read_denied",
        "write_denied",
        "network_denied",
        "descendant_denied",
        "fork_denied",
    ] {
        assert_eq!(audit[field], true, "{field}");
    }
    assert!(!write.exists());
    assert_eq!(std::fs::read(read).unwrap(), b"private owned marker");
    assert!(listener.accept().is_err());
}

#[test]
#[ignore = "explicit native SDK conformance entry point"]
fn changed_source_and_conflicting_targets_cannot_publish() {
    let root = tempfile::tempdir().unwrap();
    let source = fixture(root.path());
    let target = root.path().join("output");
    std::fs::create_dir(&target).unwrap();
    let changed = source.clone();
    let editor = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(250));
        std::fs::write(changed, b"changed").unwrap();
    });
    let mut peer = python("pause");
    assert_eq!(
        broker::run(
            &mut peer,
            &source,
            Some((1, &target)),
            "org.xdzip.sdk.reference",
            &AtomicBool::new(false)
        )
        .unwrap_err(),
        Error::Io
    );
    drop(peer);
    editor.join().unwrap();
    assert_eq!(std::fs::read_dir(&target).unwrap().count(), 0);
    let source = fixture(root.path());
    std::fs::write(target.join("large.bin"), b"retain").unwrap();
    let mut peer = python("");
    assert_eq!(
        broker::run(
            &mut peer,
            &source,
            Some((1, &target)),
            "org.xdzip.sdk.reference",
            &AtomicBool::new(false)
        )
        .unwrap_err(),
        Error::Io
    );
    drop(peer);
    assert_eq!(std::fs::read(target.join("large.bin")).unwrap(), b"retain");
    assert_eq!(std::fs::read_dir(target).unwrap().count(), 1);
}

#[test]
#[ignore = "explicit native SDK conformance entry point"]
fn strict_manifest_and_digest_admission_reject_escaping_packages() {
    let root = tempfile::tempdir().unwrap();
    let path = package(root.path());
    let original: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    for pointer in [
        "/entrypoint",
        "/permissions/network",
        "/permissions/unknown",
        "/sha256",
    ] {
        let mut value = original.clone();
        match pointer {
            "/entrypoint" => value["entrypoint"] = "../peer".into(),
            "/permissions/network" => value["permissions"]["network"] = true.into(),
            "/permissions/unknown" => value["permissions"]["unknown"] = true.into(),
            _ => value["sha256"] = "0".repeat(64).into(),
        }
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(
            matches!(package::Package::load(&path), Err(Error::Package)),
            "{pointer}"
        );
    }
    #[cfg(unix)]
    {
        let outside = tempfile::tempdir().unwrap();
        std::fs::copy(reference(), outside.path().join("peer")).unwrap();
        std::os::unix::fs::symlink(outside.path(), root.path().join("alias")).unwrap();
        let mut value = original.clone();
        value["entrypoint"] = "alias/peer".into();
        std::fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        assert!(matches!(package::Package::load(&path), Err(Error::Package)));
    }
}
