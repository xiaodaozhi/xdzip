//! CLI broker I/O retains the same native directory/ACL and commit checks as tasks.
use std::{
    io::{Read, Write},
    path::Path,
};
use xdzip_domain::path::PortablePath;

fn authorize(
    directory: &Path,
    root: &Path,
    request: bool,
) -> Option<std::sync::Arc<xdzip_io::Grant>> {
    if !directory.is_absolute() || !root.is_absolute() {
        return None;
    }
    if request {
        let leaf = directory.file_name()?.to_str()?.strip_prefix("desktop-")?;
        if leaf.len() != 32
            || !leaf.bytes().all(|b| b.is_ascii_hexdigit())
            || directory.parent()?.canonicalize().ok()? != root.canonicalize().ok()?
        {
            return None;
        }
    } else if directory.canonicalize().ok()? != root.canonicalize().ok()? {
        return None;
    }
    xdzip_io::Grant::for_creation(directory).ok()
}
pub(super) fn cli_request_read(directory: &str, root: &str) -> String {
    let result = (|| {
        let path = Path::new(directory);
        let grant = authorize(path, Path::new(root), true)?;
        let file = xdzip_io::open_regular_nofollow(&path.join("request.json")).ok()?;
        let mut bytes = vec![];
        file.take(16385).read_to_end(&mut bytes).ok()?;
        if bytes.len() > 16384 {
            return None;
        }
        grant.validate().ok()?;
        String::from_utf8(bytes).ok()
    })();
    result.unwrap_or_default()
}
fn write(directory: &str, root: &str, name: &str, result: &str, request: bool) -> bool {
    let written = (|| {
        if result.len() > 16384 {
            return None;
        }
        let grant = authorize(Path::new(directory), Path::new(root), request)?;
        let name = PortablePath::parse(name)?;
        let existing = grant.existing(&name).ok()?;
        if request && existing.is_some() {
            return None;
        }
        let mut workspace = grant.workspace(0).ok()?;
        let result = (|| {
            let mut stage = workspace.stage().ok()?;
            stage.write_all(result.as_bytes()).ok()?;
            let receipt = workspace.commit(stage, &name, existing.as_ref()).ok()?;
            (!receipt.cleanup_warning).then_some(())
        })();
        let cleanup = workspace.cleanup().ok();
        result.and(cleanup)
    })();
    written.is_some()
}
pub(super) fn cli_request_write(directory: &str, root: &str, result: &str) -> bool {
    write(directory, root, "result.json", result, true)
}
pub(super) fn cli_status_write(root: &str, result: &str) -> bool {
    write(root, root, "desktop-update-status.json", result, false)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broker_bounds_owned_request_and_reply_paths() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../build/test-tmp")
            .join(format!(
                "cli-broker-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
        std::fs::create_dir_all(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let directory = root.join(format!("desktop-{}", "a".repeat(32)));
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("request.json"), b"{\"schema_version\":1}").unwrap();
        let path = directory.to_str().unwrap();
        let base = root.to_str().unwrap();
        assert_eq!(cli_request_read(path, base), "{\"schema_version\":1}");
        assert!(cli_request_read(path, root.parent().unwrap().to_str().unwrap()).is_empty());
        assert!(cli_request_write(path, base, "{\"status\":\"handed_off\"}"));
        assert!(!cli_request_write(path, base, "{\"status\":\"installed\"}"));
        assert_eq!(
            std::fs::read_to_string(directory.join("result.json")).unwrap(),
            "{\"status\":\"handed_off\"}"
        );
        std::fs::write(directory.join("request.json"), vec![b'x'; 16385]).unwrap();
        assert!(cli_request_read(path, base).is_empty());
        assert!(!cli_request_write(path, base, &"x".repeat(16385)));
        assert!(cli_status_write(base, "{\"status\":\"recovery\"}"));
        #[cfg(unix)]
        {
            std::fs::remove_file(directory.join("request.json")).unwrap();
            std::os::unix::fs::symlink(
                root.join("desktop-update-status.json"),
                directory.join("request.json"),
            )
            .unwrap();
            assert!(cli_request_read(path, base).is_empty());
        }
        std::fs::remove_dir_all(root).unwrap();
    }
}
