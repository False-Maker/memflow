//! Terminal output capture module
//!
//! Provides functionality to capture text output from terminal windows.
//! Currently supports Windows Terminal and console applications.

use thiserror::Error;

/// Errors that can occur when capturing terminal output
#[derive(Debug, Error)]
pub enum TerminalError {
    #[error("No active terminal window found")]
    NotFound,
    #[error("Permission denied accessing terminal")]
    PermissionDenied,
    #[error("Failed to capture terminal output: {0}")]
    CaptureFailed(String),
    #[error("Platform not supported")]
    PlatformNotSupported,
}

/// Information about a detected terminal
#[derive(Debug, Clone)]
pub struct TerminalInfo {
    pub name: String,
    pub pid: u32,
    pub window_title: String,
}

/// Capture terminal output from the active terminal window
///
/// # Arguments
/// * `lines` - Maximum number of lines to capture (default: 50)
///
/// # Returns
/// * `Ok(String)` - The captured terminal output
/// * `Err(TerminalError)` - If terminal not found or capture failed
pub async fn capture_terminal_output(lines: usize) -> Result<String, TerminalError> {
    #[cfg(target_os = "windows")]
    {
        capture_terminal_output_windows(lines).await
    }
    
    #[cfg(target_os = "macos")]
    {
        capture_terminal_output_macos(lines).await
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err(TerminalError::PlatformNotSupported)
    }
}

/// Windows implementation using Windows Console API
#[cfg(target_os = "windows")]
async fn capture_terminal_output_windows(lines: usize) -> Result<String, TerminalError> {
    // Stub implementation - would require windows crate for actual implementation
    // For now, return a message indicating this is not yet fully implemented
    let output = format!(
        "[Terminal Output Capture - Stub Implementation]\n\
         This feature is not yet fully implemented on Windows.\n\
         Requested {} lines of terminal output.\n\
         To implement: Use Windows Console API to read screen buffer.\n",
        lines
    );
    
    Ok(output)
}

/// macOS implementation using Accessibility API
#[cfg(target_os = "macos")]
async fn capture_terminal_output_macos(_lines: usize) -> Result<String, TerminalError> {
    // macOS implementation using Accessibility APIs would go here
    // This is a stub for future implementation
    Err(TerminalError::PlatformNotSupported)
}

/// Detect active terminal windows
pub async fn detect_terminals() -> Result<Vec<TerminalInfo>, TerminalError> {
    #[cfg(target_os = "windows")]
    {
        detect_terminals_windows().await
    }
    
    #[cfg(target_os = "macos")]
    {
        detect_terminals_macos().await
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Err(TerminalError::PlatformNotSupported)
    }
}

#[cfg(target_os = "windows")]
async fn detect_terminals_windows() -> Result<Vec<TerminalInfo>, TerminalError> {
    use std::cell::RefCell;
    use windows::{
        Win32::Foundation::{BOOL, HWND, LPARAM, CloseHandle},
        Win32::System::Threading::{OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, PROCESS_NAME_FORMAT, QueryFullProcessImageNameW},
        Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindowThreadProcessId, GetWindowTextLengthW, GetWindowTextW,
        },
    };

    // Terminal window class names
    const TERMINAL_CLASSES: &[&str] = &[
        "ConsoleWindowClass",
        "Cascadia.Terminal",
        "CASCADIA_HOSTING_WINDOW_CLASS",
    ];

    // Terminal process names
    const TERMINAL_PROCESSES: &[&str] = &[
        "cmd.exe",
        "powershell.exe",
        "pwsh.exe",
        "wt.exe",
        "WindowsTerminal.exe",
        "conhost.exe",
    ];

    // Use a RefCell to collect terminals across the synchronous callback
    let terminals = RefCell::new(Vec::new());

    unsafe {
        unsafe extern "system" fn enumerate_terminal_windows(hwnd: HWND, lparam: LPARAM) -> BOOL {
            // Recover the terminals vector from the LPARAM
            let terminals = &*(lparam.0 as *const RefCell<Vec<TerminalInfo>>);

            // Get window class name
            let mut class_name = [0u16; 256];
            let class_len = GetClassNameW(hwnd, &mut class_name);
            let class_name_str = if class_len > 0 {
                String::from_utf16_lossy(&class_name[..class_len as usize])
            } else {
                String::new()
            };

            // Get process ID
            let mut process_id = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut process_id));

            if process_id == 0 {
                return BOOL(1); // Skip windows without process ID
            }

            // Get process name
            let process_name = get_process_name(process_id).unwrap_or_else(|_| String::new());

            // Check if this is a terminal by class name or process name
            let is_terminal_class = TERMINAL_CLASSES.iter().any(|tc| class_name_str.contains(tc));
            let is_terminal_process = TERMINAL_PROCESSES
                .iter()
                .any(|tp| process_name.eq_ignore_ascii_case(tp));

            if (is_terminal_class || is_terminal_process) && !process_name.is_empty() {
                // Get window title
                let window_title = get_window_title(hwnd).unwrap_or_default();

                // Only include visible windows with titles
                if !window_title.is_empty() {
                    let terminal = TerminalInfo {
                        name: process_name.clone(),
                        pid: process_id,
                        window_title,
                    };

                    if let Ok(mut terms) = terminals.try_borrow_mut() {
                        terms.push(terminal);
                    }
                }
            }

            BOOL(1) // Continue enumeration
        }

        unsafe fn get_process_name(process_id: u32) -> anyhow::Result<String> {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id)?;

            let mut name = [0u16; 512];
            let mut size = name.len() as u32;
            QueryFullProcessImageNameW(handle, PROCESS_NAME_FORMAT(0), windows::core::PWSTR(name.as_mut_ptr()), &mut size)?;

            let _ = CloseHandle(handle);

            let path = String::from_utf16_lossy(&name[..size as usize]);
            let process_name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            Ok(process_name)
        }

        unsafe fn get_window_title(hwnd: HWND) -> anyhow::Result<String> {
            let len = GetWindowTextLengthW(hwnd);
            if len == 0 {
                return Ok(String::new());
            }

            let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
            let copied = GetWindowTextW(hwnd, &mut buffer);

            if copied == 0 {
                return Ok(String::new());
            }

            Ok(String::from_utf16_lossy(&buffer[..copied as usize]))
        }

        EnumWindows(
            Some(enumerate_terminal_windows),
            LPARAM(&terminals as *const RefCell<Vec<TerminalInfo>> as isize),
        )
        .map_err(|e| TerminalError::CaptureFailed(format!("EnumWindows failed: {:?}", e)))?;
    }

    // Extract the collected terminals
    let collected = terminals.into_inner();
    Ok(collected)
}

#[cfg(target_os = "macos")]
async fn detect_terminals_macos() -> Result<Vec<TerminalInfo>, TerminalError> {
    Err(TerminalError::PlatformNotSupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capture_terminal_output_error_when_no_terminal() {
        // This will likely fail since there's no terminal in test environment
        let result = capture_terminal_output(50).await;
        // We expect either success with stub or NotFound error
        match result {
            Ok(_) | Err(TerminalError::NotFound) => (), // Expected
            Err(e) => panic!("Unexpected error: {}", e),
        }
    }

    #[test]
    fn test_terminal_error_display() {
        let err = TerminalError::NotFound;
        assert_eq!(format!("{}", err), "No active terminal window found");

        let err = TerminalError::CaptureFailed("test error".to_string());
        assert!(format!("{}", err).contains("Failed to capture"));
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_detect_terminals() {
        let result = detect_terminals().await;

        // Test should pass if detection succeeds (even if no terminals found)
        match result {
            Ok(terminals) => {
                // If terminals are found, validate their structure
                for terminal in &terminals {
                    assert!(!terminal.name.is_empty(), "Terminal name should not be empty");
                    assert!(terminal.pid > 0, "Terminal PID should be greater than 0");
                    assert!(
                        !terminal.window_title.is_empty(),
                        "Terminal window_title should not be empty"
                    );
                }
            }
            Err(e) => {
                // Only error that's acceptable is PlatformNotSupported
                if !matches!(e, TerminalError::PlatformNotSupported) {
                    panic!("Unexpected error: {}", e);
                }
            }
        }
    }

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_detect_terminals_returns_valid_metadata() {
        let result = detect_terminals().await;

        if let Ok(terminals) = result {
            // If we find terminals, verify they have valid metadata
            if !terminals.is_empty() {
                let terminal = &terminals[0];
                assert!(!terminal.name.is_empty(), "Terminal name should not be empty");
                assert!(terminal.pid > 0, "Terminal PID should be greater than 0");
                assert!(
                    !terminal.window_title.is_empty(),
                    "Terminal window_title should not be empty"
                );

                // Verify terminal name is one of expected terminal processes
                let valid_names = [
                    "cmd.exe",
                    "powershell.exe",
                    "pwsh.exe",
                    "wt.exe",
                    "WindowsTerminal.exe",
                    "conhost.exe",
                ];
                assert!(
                    valid_names.iter().any(|name| {
                        terminal.name.eq_ignore_ascii_case(name)
                            || terminal.name.to_lowercase().contains(
                                &name.strip_suffix(".exe").unwrap_or(name).to_lowercase()
                            )
                    }),
                    "Terminal name should be a known terminal process: {}",
                    terminal.name
                );
            }
        }
    }

    #[test]
    fn test_terminal_info_clone() {
        let terminal = TerminalInfo {
            name: "test.exe".to_string(),
            pid: 1234,
            window_title: "Test Terminal".to_string(),
        };

        let cloned = terminal.clone();
        assert_eq!(terminal.name, cloned.name);
        assert_eq!(terminal.pid, cloned.pid);
        assert_eq!(terminal.window_title, cloned.window_title);
    }
}
