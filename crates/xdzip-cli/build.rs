//! Generate the same VERSION.json and Git identity used by the desktop.
use std::{env, fs, path::PathBuf, process::Command};

fn git(root: &std::path::Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let value: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("VERSION.json")).unwrap()).unwrap();
    let channel = value["channel"].as_str().unwrap();
    let build = value["build"].as_u64().unwrap();
    let base = value["version"].as_str().unwrap();
    let version = if channel == "stable" {
        base.to_owned()
    } else {
        let suffix = if channel == "development" {
            "dev"
        } else {
            channel
        };
        format!(
            "{base}-{suffix}.{}",
            value["prerelease_number"].as_u64().unwrap_or(build)
        )
    };
    let is_root = git(&root, &["rev-parse", "--show-toplevel"])
        .is_some_and(|p| fs::canonicalize(p).ok() == fs::canonicalize(&root).ok());
    let revision = if is_root {
        git(&root, &["rev-parse", "--short=12", "HEAD"]).unwrap_or("unknown".into())
    } else {
        "unknown".into()
    };
    let dirty = is_root
        && git(
            &root,
            &["status", "--porcelain", "--untracked-files=normal"],
        )
        .is_some_and(|s| !s.is_empty());
    for (name, value) in [
        ("XDZIP_DISPLAY_VERSION", version),
        ("XDZIP_CHANNEL", channel.to_owned()),
        ("XDZIP_BUILD", build.to_string()),
        ("XDZIP_REVISION", revision),
        ("XDZIP_DIRTY", dirty.to_string()),
        ("XDZIP_CONFIGURATION", env::var("PROFILE").unwrap()),
        (
            "XDZIP_DISTRIBUTION",
            if env::var("XDZIP_CLI_RELEASE_BUILD").as_deref() == Ok("1") {
                "release"
            } else {
                "source"
            }
            .to_owned(),
        ),
    ] {
        println!("cargo:rustc-env={name}={value}");
    }
    println!("cargo:rerun-if-env-changed=XDZIP_CLI_RELEASE_BUILD");
    // Re-evaluate tracked/untracked state like the desktop's always-run identity target.
    println!(
        "cargo:rerun-if-changed={}",
        root.join("VERSION.json").display()
    );
    if let Some(files) = git(
        &root,
        &["ls-files", "--cached", "--others", "--exclude-standard"],
    ) {
        for path in files.lines() {
            println!("cargo:rerun-if-changed={}", root.join(path).display());
        }
    }
    if let Some(git_dir) = git(&root, &["rev-parse", "--absolute-git-dir"]) {
        println!("cargo:rerun-if-changed={git_dir}/HEAD");
        println!("cargo:rerun-if-changed={git_dir}/index");
        println!("cargo:rerun-if-changed={git_dir}/refs");
    }
}
