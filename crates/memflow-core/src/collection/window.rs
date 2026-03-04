//! Window information module for MemFlow Core
//!
//! Provides cross-platform window information retrieval

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Window information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowInfo {
    /// Process name
    pub process_name: String,
    /// Window title
    pub title: String,
    /// Process path
    pub process_path: Option<String>,
    /// Window handle
    #[cfg(windows)]
    pub hwnd: Option<isize>,
    #[cfg(not(windows))]
    pub hwnd: Option<u64>,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            process_name: "unknown".to_string(),
            title: "unknown".to_string(),
            process_path: None,
            hwnd: None,
        }
    }
}

/// Get the foreground window information
#[cfg(windows)]
pub fn get_foreground_window_info() -> Result<WindowInfo> {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    use windows::Win32::UI::WindowsAndMessaging::GetWindowTextW;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return Ok(WindowInfo::default());
        }

        // Get window title
        let mut title_buf = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buf);
        let title = if title_len > 0 {
            OsString::from_wide(&title_buf[..title_len as usize])
                .to_string_lossy()
                .to_string()
        } else {
            String::new()
        };

        // Get process ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // Get process name and path using a simpler method
        let (process_name, process_path) = if process_id > 0 {
            get_process_name_simple(process_id)
        } else {
            (String::new(), None)
        };

        Ok(WindowInfo {
            process_name,
            title,
            process_path,
            hwnd: Some(hwnd.0 as isize),
        })
    }
}

#[cfg(windows)]
fn get_process_name_simple(process_id: u32) -> (String, Option<String>) {
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW, PROCESS_NAME_WIN32};
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;

    unsafe {
        let handle = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) {
            Ok(h) => h,
            Err(_) => return (String::new(), None),
        };

        // Use QueryFullProcessImageNameW instead of GetModuleFileNameExW
        let mut name_buf = [0u16; 1024];
        let mut size = name_buf.len() as u32;
        
        let success = QueryFullProcessImageNameW(
            handle,
            PROCESS_NAME_WIN32,
            windows::core::PWSTR(name_buf.as_mut_ptr()),
            &mut size,
        );

        if success.is_ok() && size > 0 {
            let full_path = OsString::from_wide(&name_buf[..size as usize])
                .to_string_lossy()
                .to_string();
            
            let process_name = std::path::Path::new(&full_path)
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.to_string())
                .unwrap_or_default();

            (process_name, Some(full_path))
        } else {
            (String::new(), None)
        }
    }
}

#[cfg(not(windows))]
pub fn get_foreground_window_info() -> Result<WindowInfo> {
    Ok(WindowInfo::default())
}

/// Check if a process is a terminal/shell
pub fn is_terminal_process(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.contains("powershell")
        || lower.contains("pwsh")
        || lower.contains("cmd.exe")
        || lower.contains("windows terminal")
        || lower.contains("wt.exe")
        || lower.contains("wezterm")
        || lower.contains("alacritty")
        || lower.contains("terminal")
        || lower.contains("bash")
        || lower.contains("zsh")
        || lower.contains("fish")
        || lower.contains("sh")
        || lower.contains("zoc")
        || lower.contains("conemu")
        || lower.contains("cmder")
        || lower.contains("hyper")
        || lower.contains("iterm")
}

/// Normalize application name
pub fn normalize_app_name(name: &str) -> String {
    let trimmed = name.trim().trim_matches('"');
    let file_name = std::path::Path::new(trimmed)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(trimmed);
    let mut lower = file_name.to_lowercase();
    if let Some(stripped) = lower.strip_suffix(".exe") {
        lower = stripped.to_string();
    }
    lower
}
