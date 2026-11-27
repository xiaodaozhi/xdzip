//! Windows owns dialog layout, DPI, fonts and accessibility. Pointers are borrowed only for synchronous native calls.
#![allow(unsafe_code)]
use super::{Control, authorize_target, extract};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, LazyLock, Mutex};
use windows_sys::Win32::Foundation::{HWND, LPARAM, WPARAM};
use windows_sys::Win32::Globalization::{GetUserDefaultUILanguage, LCIDToLocaleName};
use windows_sys::Win32::System::Com::{
    COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize,
};
use windows_sys::Win32::UI::Controls::*;
use windows_sys::Win32::UI::Shell::*;
use windows_sys::Win32::UI::WindowsAndMessaging::{IDCANCEL, SendMessageW};
use xdzip_runtime::extraction::{ConflictPolicy, Snapshot, Status};

include!(concat!(env!("OUT_DIR"), "/sfx_messages.rs"));

fn sfx_locale() -> &'static str {
    static LOCALE: LazyLock<&str> = LazyLock::new(|| {
        let mut tag = [0u16; 85]; // LOCALE_NAME_MAX_LENGTH, including the terminator.
        // A LANGID with the default sort ID is a valid LCID input. The writable
        // buffer is private to the call. A Windows conversion failure uses English.
        let length = unsafe {
            LCIDToLocaleName(
                u32::from(GetUserDefaultUILanguage()),
                tag.as_mut_ptr(),
                tag.len() as i32,
                0,
            )
        };
        if length <= 1 {
            return "en-US";
        }
        let tag = String::from_utf16_lossy(&tag[..length as usize - 1]).to_ascii_lowercase();
        let mut subtags = tag.split('-');
        let primary = subtags.next().unwrap_or("");
        if primary == "zh" {
            return match subtags.next() {
                Some("hant" | "tw" | "hk" | "mo") => "zh-TW",
                _ => "zh-CN",
            };
        }
        SFX_MESSAGES
            .iter()
            .find(|(locale, _, _)| locale.split('-').next() == Some(primary))
            .map_or("en-US", |(locale, _, _)| *locale)
    });
    *LOCALE
}

fn sfx_text(key: &str) -> String {
    let locale = sfx_locale();
    SFX_MESSAGES
        .iter()
        .find(|(candidate, id, _)| *candidate == locale && *id == key)
        .or_else(|| {
            SFX_MESSAGES
                .iter()
                .find(|(candidate, id, _)| *candidate == "en-US" && *id == key)
        })
        .map_or(key, |(_, _, text)| *text)
        .to_owned()
}
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
fn message(heading: &str, content: &str) {
    let title = wide("xdzip SFX");
    let heading = wide(heading);
    let content = wide(content);
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        pszWindowTitle: title.as_ptr(),
        pszMainInstruction: heading.as_ptr(),
        pszContent: content.as_ptr(),
        dwCommonButtons: TDCBF_CLOSE_BUTTON,
        dwFlags: TDF_SIZE_TO_CONTENT,
        ..Default::default()
    };
    // All UTF-16 buffers and config outlive the modal call. Failure has no archive side effects.
    unsafe {
        TaskDialogIndirect(
            &config,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        );
    }
}
fn target() -> Option<PathBuf> {
    let title = wide(&sfx_text("sfx.target"));
    let mut display = vec![0u16; 32768];
    let info = BROWSEINFOW {
        lpszTitle: title.as_ptr(),
        pszDisplayName: display.as_mut_ptr(),
        ulFlags: BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE | BIF_EDITBOX,
        ..Default::default()
    };
    // COM is initialized by show. The returned PIDL is freed exactly once, including conversion failure.
    unsafe {
        let pidl = SHBrowseForFolderW(&info);
        if pidl.is_null() {
            return None;
        }
        let mut path = vec![0u16; 32768];
        let success = SHGetPathFromIDListEx(pidl, path.as_mut_ptr(), path.len() as u32, 0) != 0;
        CoTaskMemFree(pidl.cast());
        if !success {
            return None;
        }
        path.truncate(path.iter().position(|c| *c == 0)?);
        Some(PathBuf::from(String::from_utf16(&path).ok()?))
    }
}
fn conflict(target: &Path) -> Option<ConflictPolicy> {
    let heading = wide(&sfx_text("sfx.conflict"));
    let content = wide(&target.to_string_lossy());
    let title = wide("xdzip SFX");
    let labels = [
        wide(&sfx_text("sfx.skip")),
        wide(&sfx_text("sfx.rename")),
        wide(&sfx_text("sfx.overwrite")),
    ];
    let buttons: Vec<_> = labels
        .iter()
        .enumerate()
        .map(|(i, label)| TASKDIALOG_BUTTON {
            nButtonID: 100 + i as i32,
            pszButtonText: label.as_ptr(),
        })
        .collect();
    let note = wide(&sfx_text("sfx.noRun"));
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        pszWindowTitle: title.as_ptr(),
        pszMainInstruction: heading.as_ptr(),
        pszContent: content.as_ptr(),
        pszFooter: note.as_ptr(),
        cButtons: buttons.len() as u32,
        pButtons: buttons.as_ptr(),
        nDefaultButton: 100,
        dwCommonButtons: TDCBF_CANCEL_BUTTON,
        dwFlags: TDF_ALLOW_DIALOG_CANCELLATION | TDF_SIZE_TO_CONTENT,
        ..Default::default()
    };
    let mut button = 0;
    // Native call borrows stable config, button and UTF-16 storage until it returns.
    let hr = unsafe {
        TaskDialogIndirect(
            &config,
            &mut button,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if hr < 0 {
        return None;
    }
    match button {
        100 => Some(ConflictPolicy::Skip),
        101 => Some(ConflictPolicy::Rename),
        102 => Some(ConflictPolicy::Overwrite),
        _ => None,
    }
}
#[derive(Default)]
struct State {
    control: Arc<Control>,
    ready: (Mutex<bool>, Condvar),
    latest: Mutex<Option<Snapshot>>,
    result: Mutex<Option<Result<Snapshot, &'static str>>>,
}
unsafe extern "system" fn callback(
    window: HWND,
    event: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
    data: isize,
) -> i32 {
    // TaskDialog runs synchronously on show's thread; the Arc remains live until callback delivery ends.
    let state = unsafe { &*(data as *const State) };
    if event == TDN_CREATED as u32 {
        *state.ready.0.lock().unwrap_or_else(|e| e.into_inner()) = true;
        state.ready.1.notify_one();
    }
    let finished = state
        .result
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .is_some();
    if event == TDN_BUTTON_CLICKED as u32 && wparam == IDCANCEL as usize && !finished {
        state.control.stop();
        return 1; // S_FALSE keeps the dialog until staging cleanup has finished.
    }
    if event == TDN_TIMER as u32 || event == TDN_CREATED as u32 {
        if finished {
            // Synchronously request the existing Cancel button; callback now permits closing.
            unsafe {
                SendMessageW(window, TDM_CLICK_BUTTON as u32, IDCANCEL as usize, 0);
            }
        } else if let Some(snapshot) = state
            .latest
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .as_ref()
        {
            let percent = if snapshot.declared_bytes > 0 {
                snapshot
                    .written_bytes
                    .saturating_mul(100)
                    .checked_div(snapshot.declared_bytes)
                    .unwrap_or(0)
                    .min(100)
            } else {
                0
            };
            let summary = wide(&format!(
                "{}\n{} / {}",
                snapshot.current, snapshot.committed_files, snapshot.files
            ));
            // SendMessage consumes the borrowed string before returning, and this is the owner UI thread.
            unsafe {
                SendMessageW(window, TDM_SET_PROGRESS_BAR_POS as u32, percent as usize, 0);
                SendMessageW(
                    window,
                    TDM_SET_ELEMENT_TEXT as u32,
                    TDE_CONTENT as usize,
                    summary.as_ptr() as isize,
                );
            }
        }
    }
    0
}
pub(super) fn show(source: &Path) -> i32 {
    // A balanced STA lifetime owns folder selection; native failure stops before extraction.
    let hr = unsafe { CoInitializeEx(std::ptr::null(), COINIT_APARTMENTTHREADED as u32) };
    if hr < 0 {
        return 1;
    }
    struct Com;
    impl Drop for Com {
        fn drop(&mut self) {
            unsafe {
                CoUninitialize();
            }
        }
    }
    let _com = Com;
    let Some(target) = target() else {
        return 0;
    };
    let target_identity = match authorize_target(&target) {
        Ok(identity) => identity,
        Err(code) => {
            message(&sfx_text("sfx.failed"), &failure_text(code));
            return 1;
        }
    };
    let Some(policy) = conflict(&target) else {
        return 0;
    };
    let state = Arc::new(State::default());
    let worker_state = state.clone();
    let source = source.to_path_buf();
    let worker = std::thread::spawn(move || {
        // No hashing or extraction starts until the progress dialog can receive Cancel.
        let mut ready = worker_state
            .ready
            .0
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        while !*ready {
            ready = worker_state
                .ready
                .1
                .wait(ready)
                .unwrap_or_else(|e| e.into_inner());
        }
        drop(ready);
        let result = extract(
            &source,
            Some((target.as_path(), target_identity)),
            policy,
            &worker_state.control,
            |snapshot| {
                *worker_state
                    .latest
                    .lock()
                    .unwrap_or_else(|e| e.into_inner()) = Some(snapshot);
            },
        );
        *worker_state
            .result
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(result);
    });
    let title = wide("xdzip SFX");
    let heading = wide(&sfx_text("sfx.extracting"));
    let content = wide(&sfx_text("sfx.noRun"));
    let config = TASKDIALOGCONFIG {
        cbSize: std::mem::size_of::<TASKDIALOGCONFIG>() as u32,
        pszWindowTitle: title.as_ptr(),
        pszMainInstruction: heading.as_ptr(),
        pszContent: content.as_ptr(),
        dwCommonButtons: TDCBF_CANCEL_BUTTON,
        dwFlags: TDF_ALLOW_DIALOG_CANCELLATION
            | TDF_CALLBACK_TIMER
            | TDF_SHOW_PROGRESS_BAR
            | TDF_SIZE_TO_CONTENT,
        pfCallback: Some(callback),
        lpCallbackData: Arc::as_ptr(&state) as isize,
        ..Default::default()
    };
    // State and every pointer remain live through the synchronous dialog, then the worker is joined.
    let hr = unsafe {
        TaskDialogIndirect(
            &config,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    if hr < 0 {
        state.control.stop();
        *state.ready.0.lock().unwrap_or_else(|e| e.into_inner()) = true;
        state.ready.1.notify_one();
    }
    if worker.join().is_err() {
        message(&sfx_text("sfx.failed"), &sfx_text("sfx.invalid"));
        return 1;
    }
    let result = state
        .result
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .take();
    match result {
        Some(Ok(snapshot)) => {
            let heading = match snapshot.status {
                Status::Succeeded => sfx_text("sfx.completed"),
                Status::Cancelled => sfx_text("sfx.cancelled"),
                _ => sfx_text("sfx.failed"),
            };
            let mut summary = sfx_text("sfx.result")
                .replace("%1", &snapshot.committed_files.to_string())
                .replace("%2", &snapshot.skipped_files.to_string());
            if snapshot.status == Status::Succeeded
                || snapshot.committed_files > 0
                || snapshot.created_directories > 0
            {
                summary.push('\n');
                summary.push_str(&snapshot.target);
            }
            if snapshot.status == Status::Failed {
                summary.push('\n');
                summary.push_str(&failure_text(&snapshot.error));
            }
            if snapshot.cleanup_warning || snapshot.leftovers > 0 || snapshot.outcome_uncertain {
                summary.push('\n');
                summary.push_str(&sfx_text("ui.3ca05aed62cc"));
            }
            message(&heading, &summary);
            i32::from(snapshot.status != Status::Succeeded)
        }
        Some(Err("cancelled")) => {
            message(
                &sfx_text("sfx.cancelled"),
                &sfx_text("sfx.result").replace("%1", "0").replace("%2", "0"),
            );
            1
        }
        failure => {
            let code = failure.and_then(Result::err).unwrap_or_default();
            message(&sfx_text("sfx.failed"), &failure_text(code));
            1
        }
    }
}

fn failure_text(code: &str) -> String {
    sfx_text(match code {
        "destination_access_denied" => "desktop.destinationAccessDenied",
        "destination_missing" => "desktop.destinationMissing",
        "destination_type" => "desktop.destinationType",
        "target_changed" => "ui.8c5b6e8cc7b3",
        "unsupported_filesystem" => "ui.74b624993d20",
        "unsafe_target" => "ui.123399c0fb8b",
        _ => "sfx.invalid",
    })
}
