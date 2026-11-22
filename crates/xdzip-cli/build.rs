//! Generate the same VERSION.json and Git identity used by the desktop.
use std::{env, fs, path::PathBuf};

fn write_sfx_messages(root: &std::path::Path) {
    let resources = root.join("resources");
    let registry: serde_json::Value =
        serde_json::from_slice(&fs::read(resources.join("ui-languages.json")).unwrap()).unwrap();
    let mut generated = String::from("const SFX_MESSAGES: &[(&str, &str, &str)] = &[\n");
    for language in registry["languages"].as_array().unwrap() {
        let locale = language["id"].as_str().unwrap();
        let path = resources.join("i18n").join(format!("{locale}.json"));
        println!("cargo:rerun-if-changed={}", path.display());
        if !path.is_file() {
            continue;
        }
        let catalog: serde_json::Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
        for (id, value) in catalog["messages"].as_object().unwrap() {
            if id.starts_with("sfx.") || id == "ui.3ca05aed62cc" {
                let text = value.as_str().unwrap();
                generated.push_str(&format!("    ({locale:?}, {id:?}, {text:?}),\n"));
            }
        }
    }
    generated.push_str("];\n");
    let path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("sfx_messages.rs");
    fs::write(path, generated).unwrap();
}

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rustc-link-arg-bin=xdzip-sfx=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-bin=xdzip-sfx=/MANIFESTINPUT:{}",
            PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap())
                .join("sfx.manifest")
                .display()
        );
        write_sfx_messages(&root);
    }
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
    // Managed builds supply evaluated provenance on every invocation. Cargo
    // fingerprints the value, so a source edit with the same identity is fresh.
    let identity: serde_json::Value = env::var("XDZIP_BUILD_SOURCE")
        .ok()
        .map(|value| serde_json::from_str(&value).expect("invalid build identity"))
        .unwrap_or_default();
    let revision = identity["revision"].as_str().unwrap_or("unknown");
    let dirty = identity["dirty"].as_bool().unwrap_or(false);
    let configuration =
        env::var("XDZIP_CLI_CONFIGURATION").unwrap_or_else(|_| env::var("PROFILE").unwrap());
    for (name, value) in [
        ("XDZIP_DISPLAY_VERSION", version),
        ("XDZIP_CHANNEL", channel.to_owned()),
        ("XDZIP_BUILD", build.to_string()),
        ("XDZIP_REVISION", revision.to_owned()),
        ("XDZIP_DIRTY", dirty.to_string()),
        ("XDZIP_CONFIGURATION", configuration),
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
    println!("cargo:rerun-if-env-changed=XDZIP_BUILD_SOURCE");
    println!("cargo:rerun-if-env-changed=XDZIP_CLI_CONFIGURATION");
    println!(
        "cargo:rerun-if-changed={}",
        root.join("VERSION.json").display()
    );
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        println!("cargo:rerun-if-changed=sfx.manifest");
        println!(
            "cargo:rerun-if-changed={}",
            root.join("resources/ui-languages.json").display()
        );
    }
}
