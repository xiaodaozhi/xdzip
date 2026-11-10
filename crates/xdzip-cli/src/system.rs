//! Fixed host-system checks for native update targets, independent of Qt.
pub fn update_supported() -> bool {
    if !matches!(std::env::consts::ARCH, "x86_64" | "aarch64") {
        return false;
    }
    #[cfg(target_os = "macos")]
    {
        if std::env::consts::ARCH != "aarch64" {
            return false;
        }
        let output = std::process::Command::new("/usr/bin/sw_vers")
            .arg("-productVersion")
            .output();
        output.is_ok_and(|value| {
            value.status.success()
                && value.stdout.len() <= 256
                && std::str::from_utf8(&value.stdout)
                    .ok()
                    .and_then(|text| text.trim().split('.').next()?.parse::<u32>().ok())
                    .is_some_and(|major| major >= 14)
        })
    }
    #[cfg(all(target_os = "linux", target_env = "gnu"))]
    {
        if std::env::consts::ARCH != "x86_64" {
            return false;
        }
        let text = std::fs::read_to_string("/etc/os-release").unwrap_or_default();
        if text.len() > 16384 {
            return false;
        }
        let field = |name: &str| {
            text.lines().find_map(|line| {
                line.strip_prefix(&format!("{name}="))
                    .map(|value| value.trim_matches('"'))
            })
        };
        if field("ID") != Some("ubuntu") || field("VERSION_ID") != Some("24.04") {
            return false;
        }
        #[allow(unsafe_code)]
        // SAFETY: glibc returns a process-lifetime, NUL-terminated version string.
        // It is inspected only, without retaining mutable pointers or callbacks.
        let version = unsafe { std::ffi::CStr::from_ptr(libc::gnu_get_libc_version()) };
        let mut parts = version.to_bytes().split(|b| *b == b'.');
        let major = parts
            .next()
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|v| v.parse::<u32>().ok());
        let minor = parts
            .next()
            .and_then(|v| std::str::from_utf8(v).ok())
            .and_then(|v| v.parse::<u32>().ok());
        matches!((major,minor),(Some(major),Some(minor)) if major > 2 || major == 2 && minor >= 39)
    }
    #[cfg(windows)]
    {
        windows_supported()
    }
    #[cfg(not(any(
        target_os = "macos",
        all(target_os = "linux", target_env = "gnu"),
        windows
    )))]
    {
        false
    }
}

#[cfg(windows)]
#[allow(unsafe_code)]
fn windows_supported() -> bool {
    if std::env::consts::ARCH != "x86_64" {
        return false;
    }
    #[repr(C)]
    struct Version {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform: u32,
        description: [u16; 128],
        service_major: u16,
        service_minor: u16,
        suite: u16,
        product: u8,
        reserved: u8,
    }
    #[link(name = "ntdll")]
    unsafe extern "system" {
        fn RtlGetVersion(version: *mut Version) -> i32;
    }
    // SAFETY: fully initialized OSVERSIONINFOEXW-compatible storage includes its
    // exact size. A failed OS query rejects installation rather than guessing.
    unsafe {
        let mut value: Version = std::mem::zeroed();
        value.size = std::mem::size_of::<Version>() as u32;
        RtlGetVersion(&mut value) == 0
            && value.major >= 10
            && value.build >= 22000
            && value.product == 1
    }
}
