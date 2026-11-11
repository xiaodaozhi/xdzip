// All native policy installation is confined to the post-fork child. Errors abort
// exec, never fall back to an unrestricted third-party process.
#![allow(unsafe_code)]
use crate::Error;
use std::{path::Path, process::Command};

pub(crate) fn policy() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos-seatbelt-trial"
    } else if cfg!(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    )) {
        "linux-landlock-seccomp-trial"
    } else {
        "disabled"
    }
}

pub(crate) fn command(executable: &Path) -> Result<Command, Error> {
    #[cfg(target_os = "macos")]
    {
        let quoted = executable
            .to_str()
            .filter(|s| !s.contains(['\0', '\n', '\r']))
            .ok_or(Error::Package)?
            .replace('\\', "\\\\")
            .replace('"', "\\\"");
        let profile = format!(
            "(version 1)(deny default)(allow process-exec (literal \"{quoted}\"))(allow sysctl-read)(allow file-read-metadata (literal \"/var\"))(allow file-read* (literal \"/\") (literal \"{quoted}\") (subpath \"/System/Library\") (subpath \"/System/Volumes/Preboot/Cryptexes/OS/System/Library\") (subpath \"/usr/lib\") (literal \"/dev/null\"))(allow file-map-executable (literal \"{quoted}\") (subpath \"/System/Library\") (subpath \"/System/Volumes/Preboot/Cryptexes/OS/System/Library\") (subpath \"/usr/lib\"))"
        );
        let mut command = Command::new("/usr/bin/sandbox-exec");
        command.args(["-p", &profile, "--"]).arg(executable);
        Ok(command)
    }
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    {
        Ok(Command::new(executable))
    }
    #[cfg(not(any(
        target_os = "macos",
        all(
            target_os = "linux",
            any(target_arch = "x86_64", target_arch = "aarch64")
        )
    )))]
    {
        let _ = executable;
        Err(Error::Sandbox)
    }
}

#[cfg(unix)]
pub(crate) fn configure(command: &mut Command) -> Result<(), Error> {
    use std::os::unix::process::CommandExt;
    // Snapshot the descriptor bound before fork. No allocation, locks or Rust
    // logging occur in pre_exec; only native syscalls and prebuilt values.
    let mut limits = libc::rlimit {
        rlim_cur: 0,
        rlim_max: 0,
    };
    // SAFETY: valid initialized storage for the documented getrlimit output.
    if unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut limits) } != 0
        || limits.rlim_cur > 1_048_576
    {
        return Err(Error::Sandbox);
    }
    let fd_bound = limits.rlim_cur as i32;
    #[cfg(all(
        target_os = "linux",
        any(target_arch = "x86_64", target_arch = "aarch64")
    ))]
    let executable = command.get_program().to_owned();
    // SAFETY: the closure uses only async-signal-safe native operations; any
    // failed restriction returns an error to std's exec error pipe.
    unsafe {
        command.pre_exec(move || {
            for fd in 3..fd_bound {
                let flags = libc::fcntl(fd, libc::F_GETFD);
                if flags < 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::EBADF)
                {
                    return Err(std::io::Error::last_os_error());
                }
                if flags >= 0 && libc::fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            for (resource, value) in [
                (libc::RLIMIT_CPU, 3),
                (libc::RLIMIT_CORE, 0),
                (libc::RLIMIT_FSIZE, 0),
                (libc::RLIMIT_NOFILE, 32),
            ] {
                let limit = libc::rlimit {
                    rlim_cur: value,
                    rlim_max: value,
                };
                if libc::setrlimit(resource, &limit) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            #[cfg(all(
                target_os = "linux",
                any(target_arch = "x86_64", target_arch = "aarch64")
            ))]
            {
                let limit = libc::rlimit {
                    rlim_cur: 512 * 1024 * 1024,
                    rlim_max: 512 * 1024 * 1024,
                };
                if libc::setrlimit(libc::RLIMIT_AS, &limit) != 0 {
                    return Err(std::io::Error::last_os_error());
                }
                linux::install(&executable)?;
            }
            Ok(())
        });
    }
    Ok(())
}
#[cfg(not(unix))]
pub(crate) fn configure(_: &mut Command) -> Result<(), Error> {
    Err(Error::Sandbox)
}

pub(crate) fn check_memory(pid: u32, isolation: &str) -> Result<(), Error> {
    if isolation == "trusted-conformance" {
        return Ok(());
    }
    #[cfg(target_os = "macos")]
    {
        #[repr(C)]
        #[derive(Default)]
        struct TaskInfo {
            counters: [u64; 6],
            fields: [i32; 12],
        }
        unsafe extern "C" {
            fn proc_pidinfo(
                pid: i32,
                flavor: i32,
                arg: u64,
                buffer: *mut std::ffi::c_void,
                buffersize: i32,
            ) -> i32;
        }
        let mut info = TaskInfo::default();
        let size = std::mem::size_of::<TaskInfo>() as i32;
        // SAFETY: PROC_PIDTASKINFO=4 writes this exact SDK-declared 96-byte
        // layout. PID belongs to our unreaped child; failure invalidates the run.
        if unsafe { proc_pidinfo(pid as i32, 4, 0, (&mut info as *mut TaskInfo).cast(), size) }
            != size
        {
            return Err(Error::Sandbox);
        }
        if info.counters[1] > 512 * 1024 * 1024 {
            return Err(Error::Limit);
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = pid;
    Ok(())
}

#[cfg(all(
    target_os = "linux",
    any(target_arch = "x86_64", target_arch = "aarch64")
))]
mod linux {
    use std::ffi::{CStr, OsStr};
    use std::os::unix::ffi::OsStrExt;
    // Linux UAPI ABI v3. Every known v3 filesystem right is handled; no write
    // rule is installed. Newer ABI rights are not silently claimed as enforced.
    #[repr(C)]
    struct Ruleset {
        handled_access_fs: u64,
    }
    #[repr(C, packed)]
    struct PathRule {
        allowed_access: u64,
        parent_fd: i32,
    }
    const READ_FILE: u64 = 1 << 2;
    const READ_DIR: u64 = 1 << 3;
    const EXECUTE: u64 = 1;
    fn failed() -> std::io::Error {
        std::io::Error::from_raw_os_error(libc::ENOTSUP)
    }
    pub unsafe fn install(executable: &OsStr) -> std::io::Result<()> {
        // SAFETY: syscall signatures/layouts follow linux/landlock.h. All fds
        // are local child-owned handles and are closed before returning.
        unsafe {
            let abi = libc::syscall(
                libc::SYS_landlock_create_ruleset,
                std::ptr::null::<u8>(),
                0,
                1,
            );
            if abi < 3 {
                return Err(failed());
            }
            let attr = Ruleset {
                handled_access_fs: (1 << 15) - 1,
            };
            let fd = libc::syscall(
                libc::SYS_landlock_create_ruleset,
                &attr,
                std::mem::size_of::<Ruleset>(),
                0,
            ) as i32;
            if fd < 0 {
                return Err(failed());
            }
            // Fixed preallocated executable spelling; no CString allocation in pre_exec.
            let raw = executable.as_bytes();
            let mut path = [0u8; 4096];
            if raw.len() >= path.len() || raw.contains(&0) {
                libc::close(fd);
                return Err(failed());
            }
            path[..raw.len()].copy_from_slice(raw);
            let program = CStr::from_bytes_until_nul(&path).map_err(|_| failed())?;
            let result = (|| {
                add(fd, program, READ_FILE | EXECUTE)?;
                for path in [c"/usr/lib", c"/lib", c"/lib64", c"/etc/ld.so.cache"] {
                    if libc::access(path.as_ptr(), libc::F_OK) == 0 {
                        add(fd, path, READ_FILE | READ_DIR | EXECUTE)?;
                    }
                }
                if libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) != 0
                    || libc::syscall(libc::SYS_landlock_restrict_self, fd, 0) != 0
                {
                    return Err(failed());
                }
                seccomp()
            })();
            libc::close(fd);
            result
        }
    }
    unsafe fn add(ruleset: i32, path: &CStr, allowed: u64) -> std::io::Result<()> {
        // SAFETY: fixed NUL-terminated paths; O_PATH handles identify the rule
        // object. A file must not receive directory-only READ_DIR permission.
        unsafe {
            let fd = libc::open(path.as_ptr(), libc::O_PATH | libc::O_CLOEXEC);
            if fd < 0 {
                return Err(failed());
            }
            let mut status = std::mem::zeroed::<libc::stat>();
            if libc::fstat(fd, &mut status) != 0 {
                libc::close(fd);
                return Err(failed());
            }
            let allowed = if status.st_mode & libc::S_IFMT == libc::S_IFDIR {
                allowed
            } else {
                allowed & !READ_DIR
            };
            let rule = PathRule {
                allowed_access: allowed,
                parent_fd: fd,
            };
            let result = libc::syscall(libc::SYS_landlock_add_rule, ruleset, 1, &rule, 0);
            libc::close(fd);
            if result == 0 { Ok(()) } else { Err(failed()) }
        }
    }
    unsafe fn seccomp() -> std::io::Result<()> {
        const KILL: u32 = 0x80000000;
        const DENY: u32 = 0x00050000 | libc::EPERM as u32;
        const ALLOW: u32 = 0x7fff0000;
        #[cfg(target_arch = "x86_64")]
        const ARCH: u32 = 0xc000003e;
        #[cfg(target_arch = "aarch64")]
        const ARCH: u32 = 0xc00000b7;
        // Deny all syscalls except the small single-threaded reader runtime.
        // This also denies io_uring, ptrace, sockets and every process creation
        // syscall, including future ones, rather than maintaining a denylist.
        let allowed = [
            libc::SYS_read,
            libc::SYS_write,
            libc::SYS_close,
            libc::SYS_fstat,
            libc::SYS_lseek,
            libc::SYS_mmap,
            libc::SYS_mprotect,
            libc::SYS_munmap,
            libc::SYS_brk,
            libc::SYS_rt_sigaction,
            libc::SYS_rt_sigprocmask,
            libc::SYS_rt_sigreturn,
            libc::SYS_ioctl,
            libc::SYS_pread64,
            libc::SYS_exit,
            libc::SYS_exit_group,
            libc::SYS_futex,
            libc::SYS_getpid,
            libc::SYS_gettid,
            libc::SYS_sigaltstack,
            libc::SYS_clock_gettime,
            libc::SYS_sched_getaffinity,
            libc::SYS_set_tid_address,
            libc::SYS_set_robust_list,
            libc::SYS_prlimit64,
            libc::SYS_getrandom,
            libc::SYS_rseq,
            libc::SYS_openat,
            libc::SYS_newfstatat,
            libc::SYS_readlinkat,
            libc::SYS_fcntl,
            libc::SYS_faccessat,
            libc::SYS_execve,
        ];
        let mut filter = [libc::sock_filter {
            code: 0,
            jt: 0,
            jf: 0,
            k: 0,
        }; 128];
        let mut used = 0;
        let mut push = |code, jt, jf, k| {
            filter[used] = libc::sock_filter { code, jt, jf, k };
            used += 1;
        };
        push(0x20, 0, 0, 4);
        push(0x15, 1, 0, ARCH);
        push(0x06, 0, 0, KILL);
        push(0x20, 0, 0, 0);
        for call in allowed {
            push(0x15, 0, 1, call as u32);
            push(0x06, 0, 0, ALLOW);
        }
        #[cfg(target_arch = "x86_64")]
        for call in [libc::SYS_arch_prctl, libc::SYS_readlink, libc::SYS_access] {
            push(0x15, 0, 1, call as u32);
            push(0x06, 0, 0, ALLOW);
        }
        push(0x06, 0, 0, DENY);
        let program = libc::sock_fprog {
            len: used as u16,
            filter: filter.as_mut_ptr(),
        };
        // SAFETY: BPF storage lives through this syscall. Kernel copies and
        // verifies it atomically; any failure prevents exec.
        if unsafe { libc::prctl(libc::PR_SET_SECCOMP, 2, &program) } != 0 {
            Err(failed())
        } else {
            Ok(())
        }
    }
}
