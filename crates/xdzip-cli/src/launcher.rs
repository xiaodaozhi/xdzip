//! Stable console entry; the versioned payload may change while old tasks finish.
use std::{path::PathBuf, process::Command};
fn launch() -> Result<i32, Box<dyn std::error::Error>> {
    let exe = std::env::current_exe()?;
    let parent = exe.parent().ok_or("invalid_registration")?;
    // Reuse the same native target authorization as setup. The registration is
    // user-owned and cannot be selected by an archive, environment or argument.
    xdzip_io::Grant::for_creation(parent).map_err(|_| "unsafe_registration")?;
    let file = xdzip_io::open_regular_nofollow(&parent.join("xdzip-command.json"))?;
    use std::io::Read;
    let mut bytes = vec![];
    file.take(16385).read_to_end(&mut bytes)?;
    if bytes.len() > 16384 {
        return Err("invalid_registration".into());
    }
    let record: serde_json::Value = serde_json::from_slice(&bytes)?;
    if record["schema_version"] != 1 || record["product"] != "xdzip" {
        return Err("invalid_registration".into());
    }
    let target = if record["kind"] == "managed" {
        let root = PathBuf::from(record["root"].as_str().ok_or("invalid_registration")?);
        if !root.is_absolute() {
            return Err("invalid_registration".into());
        }
        xdzip_io::Grant::for_creation(&root).map_err(|_| "unsafe_registration")?;
        let pointer = xdzip_io::open_regular_nofollow(&root.join("current.json"))?;
        let mut bytes = vec![];
        pointer.take(16385).read_to_end(&mut bytes)?;
        if bytes.len() > 16384 {
            return Err("invalid_registration".into());
        }
        let pointer: serde_json::Value = serde_json::from_slice(&bytes)?;
        let runtime = pointer["runtime"].as_str().ok_or("invalid_registration")?;
        if runtime.is_empty()
            || !runtime
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        {
            return Err("invalid_registration".into());
        }
        root.join("versions")
            .join(runtime)
            .join("bin")
            .join(if cfg!(windows) { "xdzip.exe" } else { "xdzip" })
    } else if record["kind"] == "desktop" {
        PathBuf::from(
            record["executable"]
                .as_str()
                .ok_or("invalid_registration")?,
        )
    } else {
        return Err("invalid_registration".into());
    };
    if !target.is_absolute() || !std::fs::symlink_metadata(&target)?.is_file() {
        return Err("invalid_registration".into());
    }
    // No shell or argument rewriting; Windows waits and propagates the real
    // console child's status, including when stdout/stderr are redirected.
    let mut command = Command::new(target);
    command.args(std::env::args_os().skip(1));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec().into())
    }
    #[cfg(windows)]
    {
        wait_console(command)
    }
}
fn main() {
    match launch() {
        Ok(code) => std::process::exit(code),
        Err(_) => {
            eprintln!("xdzip: invalid or unavailable command registration");
            std::process::exit(3);
        }
    }
}

#[cfg(windows)]
static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
#[cfg(windows)]
#[allow(unsafe_code)]
unsafe extern "system" fn console_signal(kind: u32) -> i32 {
    if kind <= 2 {
        INTERRUPTED.store(true, std::sync::atomic::Ordering::Relaxed);
        1
    } else {
        0
    }
}
#[cfg(windows)]
#[allow(unsafe_code)]
fn wait_console(mut command: Command) -> Result<i32, Box<dyn std::error::Error>> {
    use std::os::windows::process::CommandExt;
    use windows_sys::Win32::System::Console::*;
    // The stable launcher stays alive until the payload finishes cleanup. A
    // separate process group lets the parent forward Ctrl+C as Ctrl+Break to
    // exactly its child; no unrelated console process receives cancellation.
    unsafe {
        if SetConsoleCtrlHandler(Some(console_signal), 1) == 0 {
            return Err("console_handler".into());
        }
    }
    command.creation_flags(0x00000200);
    let mut child = command.spawn()?;
    loop {
        if INTERRUPTED.swap(false, std::sync::atomic::Ordering::Relaxed) {
            // SAFETY: captured child PID is the process-group ID created above.
            unsafe {
                GenerateConsoleCtrlEvent(CTRL_BREAK_EVENT, child.id());
            }
        }
        if let Some(status) = child.try_wait()? {
            return Ok(status.code().unwrap_or(130));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
