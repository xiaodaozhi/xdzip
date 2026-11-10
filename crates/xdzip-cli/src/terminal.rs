//! Minimal OS terminal adaptation. Secrets never enter command arguments or events.
#![allow(unsafe_code)]
use crate::output::{Failure, Result};
use std::io::{self, IsTerminal, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use zeroize::{Zeroize, Zeroizing};

static INTERRUPTED: AtomicBool = AtomicBool::new(false);
pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::Relaxed)
}
pub fn interactive() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal() && io::stderr().is_terminal()
}

#[cfg(unix)]
extern "C" fn signal(_: libc::c_int) {
    INTERRUPTED.store(true, Ordering::Relaxed);
}
#[cfg(windows)]
unsafe extern "system" fn signal(kind: u32) -> i32 {
    if kind <= 2 {
        INTERRUPTED.store(true, Ordering::Relaxed);
        1
    } else {
        0
    }
}
pub fn install() -> Result<()> {
    #[cfg(unix)]
    // SAFETY: sigaction is initialized, handler only stores a lock-free atomic.
    // No runtime mutex, allocation, file operation or secret access occurs here.
    unsafe {
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = signal as *const () as usize;
        libc::sigemptyset(&mut action.sa_mask);
        for kind in [libc::SIGINT, libc::SIGTERM] {
            if libc::sigaction(kind, &action, std::ptr::null_mut()) != 0 {
                return Err("signal_handler".into());
            }
        }
    }
    #[cfg(windows)]
    // SAFETY: the process-lifetime handler has the required ABI and only records cancellation.
    unsafe {
        if windows_sys::Win32::System::Console::SetConsoleCtrlHandler(Some(signal), 1) == 0 {
            return Err("signal_handler".into());
        }
    }
    Ok(())
}

struct Echo {
    #[cfg(unix)]
    saved: Option<libc::termios>,
    #[cfg(windows)]
    saved: Option<u32>,
}
impl Echo {
    fn hide(hide: bool) -> Result<Self> {
        #[cfg(unix)]
        {
            let mut saved = None;
            if hide && io::stdin().is_terminal() {
                // SAFETY: fd 0 is borrowed, termios storage is valid. Failed mode
                // changes refuse input; Drop restores the exact original mode.
                unsafe {
                    let mut mode = std::mem::zeroed();
                    if libc::tcgetattr(0, &mut mode) != 0 {
                        return Err("terminal_mode".into());
                    }
                    saved = Some(mode);
                    mode.c_lflag &= !libc::ECHO;
                    if libc::tcsetattr(0, libc::TCSANOW, &mode) != 0 {
                        return Err("terminal_mode".into());
                    }
                }
            }
            Ok(Self { saved })
        }
        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Console::*;
            let _ = hide;
            let mut saved = None;
            if io::stdin().is_terminal() {
                // SAFETY: stdin is borrowed, GetConsoleMode initializes mode on success.
                unsafe {
                    let handle = GetStdHandle(STD_INPUT_HANDLE);
                    let mut mode = 0;
                    if GetConsoleMode(handle, &mut mode) == 0 {
                        return Err("terminal_mode".into());
                    }
                    saved = Some(mode);
                    if SetConsoleMode(handle, mode & !(ENABLE_ECHO_INPUT | ENABLE_LINE_INPUT)) == 0
                    {
                        return Err("terminal_mode".into());
                    }
                }
            }
            Ok(Self { saved })
        }
    }
}
impl Drop for Echo {
    fn drop(&mut self) {
        #[cfg(unix)]
        if let Some(mode) = &self.saved {
            // SAFETY: restores a mode read from this same borrowed terminal.
            unsafe {
                libc::tcsetattr(0, libc::TCSANOW, mode);
            }
        }
        #[cfg(windows)]
        if let Some(mode) = self.saved {
            // SAFETY: restores the previously read process stdin mode.
            unsafe {
                use windows_sys::Win32::System::Console::*;
                SetConsoleMode(GetStdHandle(STD_INPUT_HANDLE), mode);
            }
        }
    }
}
pub fn line(prompt: &str, secret: bool, limit: usize) -> Result<Zeroizing<String>> {
    // Disable echo before publishing the prompt: a fast terminal user (or PTY)
    // can otherwise send secret bytes between the prompt and the mode change.
    let mode = Echo::hide(secret)?;
    if !prompt.is_empty() {
        eprint!("{prompt}");
        io::stderr().flush()?;
    }
    #[cfg(windows)]
    if io::stdin().is_terminal() {
        let result = console_line(secret, limit);
        drop(mode);
        if !prompt.is_empty() {
            eprintln!();
        }
        return result;
    }
    let mut bytes = Zeroizing::new(Vec::new());
    loop {
        if interrupted() {
            return Err("cancelled".into());
        }
        let mut byte = 0u8;
        #[cfg(unix)]
        let count = {
            let mut poll = libc::pollfd {
                fd: 0,
                events: libc::POLLIN,
                revents: 0,
            };
            // SAFETY: one valid pollfd and one writable byte, no retained pointer.
            let ready = unsafe { libc::poll(&mut poll, 1, 100) };
            if ready < 0 && io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
                return Err("input_io".into());
            }
            if ready <= 0 {
                continue;
            }
            unsafe { libc::read(0, (&mut byte as *mut u8).cast(), 1) }
        };
        #[cfg(windows)]
        let count = {
            use windows_sys::Win32::{
                Storage::FileSystem::{FILE_TYPE_PIPE, GetFileType, ReadFile},
                System::{Console::*, Pipes::PeekNamedPipe, Threading::*},
            };
            // SAFETY: borrowed stdin and a live one-byte output buffer; calls are synchronous.
            unsafe {
                let handle = GetStdHandle(STD_INPUT_HANDLE);
                if GetFileType(handle) == FILE_TYPE_PIPE {
                    let mut available = 0;
                    if PeekNamedPipe(
                        handle,
                        std::ptr::null_mut(),
                        0,
                        std::ptr::null_mut(),
                        &mut available,
                        std::ptr::null_mut(),
                    ) == 0
                    {
                        if windows_sys::Win32::Foundation::GetLastError()
                            == windows_sys::Win32::Foundation::ERROR_BROKEN_PIPE
                        {
                            break;
                        }
                        return Err("input_io".into());
                    }
                    if available == 0 {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        continue;
                    }
                }
                if WaitForSingleObject(handle, 100) == 258 {
                    continue;
                }
                let mut read = 0;
                if ReadFile(handle, &mut byte, 1, &mut read, std::ptr::null_mut()) == 0 {
                    return Err("input_io".into());
                }
                read as isize
            }
        };
        if count < 0 {
            if io::Error::last_os_error().kind() == io::ErrorKind::Interrupted {
                continue;
            }
            return Err("input_io".into());
        }
        if count == 0 || byte == b'\n' {
            break;
        }
        if bytes.len() > limit {
            byte.zeroize();
            return Err("input_limit".into());
        }
        bytes.push(byte);
        byte.zeroize();
    }
    drop(mode);
    if secret && !prompt.is_empty() {
        eprintln!();
    }
    if bytes.last() == Some(&b'\r') {
        bytes.pop();
    }
    if bytes.len() > limit {
        return Err("input_limit".into());
    }
    let owned = std::mem::take(&mut *bytes);
    String::from_utf8(owned)
        .map(Zeroizing::new)
        .map_err(|error| {
            let mut bytes = error.into_bytes();
            bytes.zeroize();
            Failure::new("invalid_utf8_input")
        })
}
pub fn password(stdin: bool, allowed: bool, chinese: bool) -> Result<Zeroizing<String>> {
    if !stdin && !allowed {
        return Err("interaction_required".into());
    }
    line(
        if stdin {
            ""
        } else if chinese {
            "密码："
        } else {
            "Password: "
        },
        true,
        1024,
    )
}

#[cfg(windows)]
fn console_line(secret: bool, limit: usize) -> Result<Zeroizing<String>> {
    use windows_sys::Win32::System::{Console::*, Threading::WaitForSingleObject};
    let mut units = Zeroizing::new(Vec::<u16>::new());
    loop {
        if interrupted() {
            return Err("cancelled".into());
        }
        let mut unit = 0u16;
        let mut count = 0;
        // SAFETY: stdin is a borrowed console handle in non-line mode; the one
        // UTF-16 output unit and count remain valid for this synchronous call.
        unsafe {
            let handle = GetStdHandle(STD_INPUT_HANDLE);
            if WaitForSingleObject(handle, 100) == 258 {
                continue;
            }
            if ReadConsoleW(
                handle,
                (&mut unit as *mut u16).cast(),
                1,
                &mut count,
                std::ptr::null_mut(),
            ) == 0
            {
                return Err(if interrupted() {
                    "cancelled"
                } else {
                    "input_io"
                }
                .into());
            }
        }
        if count == 0 || unit == 13 || unit == 10 {
            break;
        }
        if unit == 8 {
            if let Some(previous) = units.pop() {
                if (0xdc00..=0xdfff).contains(&previous) {
                    units.pop();
                }
                if !secret {
                    eprint!("\x08 \x08");
                }
            }
            continue;
        }
        if unit < 32 {
            continue;
        }
        if units.len() > limit {
            unit.zeroize();
            return Err("input_limit".into());
        }
        units.push(unit);
        if !secret && !(0xd800..=0xdbff).contains(&unit) {
            let start = if (0xdc00..=0xdfff).contains(&unit) {
                units.len().saturating_sub(2)
            } else {
                units.len() - 1
            };
            eprint!("{}", String::from_utf16_lossy(&units[start..]));
            io::stderr().flush()?;
        }
        unit.zeroize();
    }
    let text = Zeroizing::new(
        String::from_utf16(&units).map_err(|_| Failure::new("invalid_utf16_input"))?,
    );
    if text.len() > limit {
        return Err("input_limit".into());
    }
    Ok(text)
}
