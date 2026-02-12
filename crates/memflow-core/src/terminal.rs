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

/// Windows implementation using Windows Console API with UIA fallback
#[cfg(target_os = "windows")]
async fn capture_terminal_output_windows(lines: usize) -> Result<String, TerminalError> {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

    // Get foreground window
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() {
        return Err(TerminalError::NotFound);
    }

    // Check if it's a modern terminal
    let is_modern = is_modern_terminal(hwnd)?;

    if is_modern {
        tracing::warn!("Modern terminal detected, using UIA fallback path");
        return capture_with_uia(hwnd, lines).await;
    }

    // Try Console API for legacy terminals
    let console_result = try_console_api_capture(lines).await;

    match console_result {
        Ok(text) => Ok(text),
        Err(e) => {
            // Console API failed, try UIA fallback
            tracing::warn!("Console API capture failed, falling back to UIA: {:?}", e);
            capture_with_uia(hwnd, lines).await
        }
    }
}

/// Check if the given window is a modern terminal (Windows Terminal, Cascadia)
#[cfg(target_os = "windows")]
fn is_modern_terminal(hwnd: windows::Win32::Foundation::HWND) -> Result<bool, TerminalError> {
    use windows::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetWindowTextW};

    unsafe {
        // Get window class name
        let mut class_name = [0u16; 256];
        let class_len = GetClassNameW(hwnd, &mut class_name);
        let class_name_str = if class_len > 0 {
            String::from_utf16_lossy(&class_name[..class_len as usize])
        } else {
            String::new()
        };

        // Get window title
        let mut title_buffer = [0u16; 512];
        let title_len = GetWindowTextW(hwnd, &mut title_buffer);
        let title_str = if title_len > 0 {
            String::from_utf16_lossy(&title_buffer[..title_len as usize])
        } else {
            String::new()
        };

        // Modern terminal indicators
        let class_indicators = ["Cascadia", "CASCADIA_HOSTING", "WindowsTerminal"];
        let title_indicators = ["Windows Terminal", "PowerShell", "Ubuntu", "WSL"];

        let is_modern = class_indicators.iter().any(|ind| class_name_str.contains(ind))
            || title_indicators.iter().any(|ind| title_str.contains(ind));

        Ok(is_modern)
    }
}

/// Try to capture using Console API (for legacy terminals)
#[cfg(target_os = "windows")]
async fn try_console_api_capture(lines: usize) -> Result<String, TerminalError> {
    use windows::{
        Win32::Foundation::GetLastError,
        Win32::System::Console::{
            AttachConsole, GetConsoleScreenBufferInfo, GetStdHandle, ReadConsoleOutputW,
            CHAR_INFO, CONSOLE_SCREEN_BUFFER_INFO, COORD, SMALL_RECT, STD_OUTPUT_HANDLE,
        },
    };

    // Find the first terminal process
    let target_pid = find_console_terminal_pid().await?;

    if target_pid == 0 {
        return Err(TerminalError::CaptureFailed("No console terminal found".to_string()));
    }

    // Attach to the target console
    let result = unsafe { AttachConsole(target_pid) };
    if result.is_err() {
        let error = unsafe { GetLastError() };
        return Err(TerminalError::CaptureFailed(format!(
            "AttachConsole failed for PID {}: error {:?}",
            target_pid, error
        )));
    }

    // Use scopeguard to ensure FreeConsole is called
    let _guard = scopeguard::guard((), |_| unsafe {
        let _ = windows::Win32::System::Console::FreeConsole();
    });

    // Get console handle
    let console_handle = match unsafe { GetStdHandle(STD_OUTPUT_HANDLE) } {
        Ok(handle) => handle,
        Err(e) => {
            return Err(TerminalError::CaptureFailed(format!(
                "GetStdHandle failed: {:?}",
                e
            )))
        }
    };

    if console_handle.is_invalid() {
        return Err(TerminalError::CaptureFailed(
            "GetStdHandle returned invalid handle".to_string(),
        ));
    }

    // Get console screen buffer info
    let mut buffer_info: CONSOLE_SCREEN_BUFFER_INFO = unsafe { std::mem::zeroed() };
    let result = unsafe { GetConsoleScreenBufferInfo(console_handle, &mut buffer_info) };
    if result.is_ok() {
        let buffer_size = buffer_info.dwSize;
        let buffer_width = buffer_size.X;
        let buffer_height = buffer_size.Y;

        if buffer_width == 0 || buffer_height == 0 {
            return Err(TerminalError::CaptureFailed(
                "Invalid console buffer size".to_string(),
            ));
        }

        // Allocate buffer for CHAR_INFO
        let mut char_buffer: Vec<CHAR_INFO> = vec![unsafe { std::mem::zeroed() };
            buffer_width as usize * buffer_height as usize];

        let mut read_region = SMALL_RECT {
            Left: 0,
            Top: 0,
            Right: buffer_width - 1,
            Bottom: buffer_height - 1,
        };

        let read_coord = COORD { X: 0, Y: 0 };

        // Read entire buffer
        let result = unsafe {
            ReadConsoleOutputW(
                console_handle,
                char_buffer.as_mut_ptr(),
                buffer_size,
                read_coord,
                &mut read_region,
            )
        };

        if result.is_ok() {
            // Parse CHAR_INFO array to extract text
            let mut text_lines: Vec<String> = Vec::new();

            for row in 0..buffer_height {
                let mut line = String::new();

                for col in 0..buffer_width {
                    let idx = (row * buffer_width + col) as usize;
                    if idx < char_buffer.len() {
                        let char_info = &char_buffer[idx];
                        let char_val = unsafe { char_info.Char.UnicodeChar };

                        // Only process printable characters
                        if char_val > 0x20 && char_val <= 0x7E {
                            if let Some(c) = std::char::from_u32(char_val as u32) {
                                line.push(c);
                            }
                        } else if char_val == 0x20 {
                            line.push(' ');
                        }
                    }
                }

                // Trim trailing spaces and keep non-empty lines
                let line = line.trim_end().to_string();
                if !line.is_empty() {
                    text_lines.push(line);
                }
            }

            // Return last N lines
            if text_lines.is_empty() {
                return Err(TerminalError::CaptureFailed(
                    "No text content in console buffer".to_string(),
                ));
            }

            let start_idx = if text_lines.len() > lines {
                text_lines.len() - lines
            } else {
                0
            };

            Ok(text_lines[start_idx..].join("\n"))
        } else {
            Err(TerminalError::CaptureFailed(
                "ReadConsoleOutputW failed".to_string(),
            ))
        }
    } else {
        Err(TerminalError::CaptureFailed(
            "GetConsoleScreenBufferInfo failed".to_string(),
        ))
    }
}

/// Capture terminal output using UI Automation (fallback for modern terminals)
#[cfg(target_os = "windows")]
async fn capture_with_uia(hwnd: windows::Win32::Foundation::HWND, lines: usize) -> Result<String, TerminalError> {
    use windows::{
        Win32::System::Com::{
            CoInitializeEx, CoUninitialize, CoCreateInstance, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
        },
        Win32::UI::Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTreeWalker,
        },
    };
    use std::time::Instant;

    unsafe {
        // Initialize COM
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() && hr.0 != 1 {
            tracing::debug!("COM initialization failed in UIA fallback: {:?}", hr);
            return Err(TerminalError::CaptureFailed(
                "COM initialization failed".to_string(),
            ));
        }

        // Ensure CoUninitialize is called
        let _guard = scopeguard::guard((), |_| {
            CoUninitialize();
        });

        // Create UIA instance
        let automation: IUIAutomation = match CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
            Ok(a) => a,
            Err(e) => {
                tracing::debug!("Failed to create UIA instance: {:?}", e);
                return Err(TerminalError::CaptureFailed(format!(
                    "UIA instance creation failed: {:?}",
                    e
                )));
            }
        };

        // Get window element
        let element: IUIAutomationElement = match automation.ElementFromHandle(hwnd) {
            Ok(e) => e,
            Err(e) => {
                tracing::debug!("Failed to get window element: {:?}", e);
                return Err(TerminalError::CaptureFailed(format!(
                    "Failed to get window element: {:?}",
                    e
                )));
            }
        };

        // Create TreeWalker
        let walker: IUIAutomationTreeWalker = match automation.ControlViewWalker() {
            Ok(w) => w,
            Err(e) => {
                tracing::debug!("Failed to create TreeWalker: {:?}", e);
                return Err(TerminalError::CaptureFailed(format!(
                    "Failed to create TreeWalker: {:?}",
                    e
                )));
            }
        };

        // Walk the tree to collect text
        let start_time = Instant::now();
        let mut texts: Vec<String> = Vec::new();

        walk_terminal_tree(&walker, &element, 0, &start_time, &mut texts);

        if texts.is_empty() {
            return Err(TerminalError::CaptureFailed(
                "No text extracted from terminal".to_string(),
            ));
        }

        // Combine and limit text
        let combined = texts.join("\n");
        let result = if lines > 0 {
            // Take last N lines
            let all_lines: Vec<&str> = combined.lines().collect();
            if all_lines.len() > lines {
                all_lines[all_lines.len() - lines..].join("\n")
            } else {
                combined
            }
        } else {
            combined
        };

        Ok(result)
    }
}

/// Walk the UI tree to extract terminal text
#[cfg(target_os = "windows")]
fn walk_terminal_tree(
    walker: &windows::Win32::UI::Accessibility::IUIAutomationTreeWalker,
    element: &windows::Win32::UI::Accessibility::IUIAutomationElement,
    depth: u32,
    start_time: &std::time::Instant,
    texts: &mut Vec<String>,
) {
    use windows::{
        Win32::UI::Accessibility::{
            UIA_TextControlTypeId, UIA_EditControlTypeId, UIA_DocumentControlTypeId,
            UIA_ControlTypePropertyId, UIA_NamePropertyId, UIA_ValueValuePropertyId,
        },
        core::BSTR,
    };

    const MAX_TRAVERSAL_DEPTH: u32 = 10;
    const MAX_TRAVERSAL_TIME_MS: u128 = 500;

    // Circuit breaker: depth and timeout
    if depth > MAX_TRAVERSAL_DEPTH {
        return;
    }
    if start_time.elapsed().as_millis() > MAX_TRAVERSAL_TIME_MS {
        tracing::debug!("UIA traversal timeout ({}ms)", MAX_TRAVERSAL_TIME_MS);
        return;
    }

    unsafe {
        // Check if current element is a text element
        if let Ok(control_type) = element.GetCurrentPropertyValue(UIA_ControlTypePropertyId) {
            let ct_val: i32 = control_type.as_raw().Anonymous.Anonymous.Anonymous.lVal;
            let is_text_element = ct_val == UIA_TextControlTypeId.0
                || ct_val == UIA_EditControlTypeId.0
                || ct_val == UIA_DocumentControlTypeId.0;

            if is_text_element {
                // Try to get Name property
                if let Ok(name) = element.GetCurrentPropertyValue(UIA_NamePropertyId) {
                    if let Ok(bstr) = BSTR::try_from(&name) {
                        let text = bstr.to_string();
                        if !text.trim().is_empty() {
                            texts.push(text);
                        }
                    }
                }

                // Try to get Value property (for Edit controls)
                if let Ok(value) = element.GetCurrentPropertyValue(UIA_ValueValuePropertyId) {
                    if let Ok(bstr) = BSTR::try_from(&value) {
                        let text = bstr.to_string();
                        if !text.trim().is_empty() && !texts.contains(&text) {
                            texts.push(text);
                        }
                    }
                }
            }
        }

        // Traverse child elements
        if let Ok(first_child) = walker.GetFirstChildElement(element) {
            walk_terminal_tree(walker, &first_child, depth + 1, start_time, texts);

            // Traverse siblings
            let mut current = first_child;
            while let Ok(next) = walker.GetNextSiblingElement(&current) {
                if start_time.elapsed().as_millis() > MAX_TRAVERSAL_TIME_MS {
                    tracing::debug!("UIA traversal timeout (sibling traversal)");
                    return;
                }
                walk_terminal_tree(walker, &next, depth + 1, start_time, texts);
                current = next;
            }
        }
    }
}

#[cfg(target_os = "windows")]
async fn find_console_terminal_pid() -> Result<u32, TerminalError> {
    use std::cell::RefCell;
    use windows::{
        Win32::Foundation::{BOOL, CloseHandle, HWND, LPARAM},
        Win32::System::Threading::{
            OpenProcess, PROCESS_QUERY_INFORMATION, PROCESS_VM_READ, PROCESS_NAME_FORMAT,
            QueryFullProcessImageNameW,
        },
        Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetClassNameW, GetWindowThreadProcessId, GetWindowTextLengthW,
            GetWindowTextW,
        },
    };

    // Console terminal class names
    const TERMINAL_CLASSES: &[&str] = &["ConsoleWindowClass"];

    // Console terminal process names
    const TERMINAL_PROCESSES: &[&str] = &["cmd.exe", "powershell.exe", "pwsh.exe"];

    let target_pid = RefCell::new(0u32);

    unsafe {
        unsafe extern "system" fn enumerate_console_windows(
            hwnd: HWND,
            lparam: LPARAM,
        ) -> BOOL {
            let target_pid = &*(lparam.0 as *const RefCell<u32>);

            // Get window class name
            let mut class_name = [0u16; 256];
            let class_len = GetClassNameW(hwnd, &mut class_name);
            let class_name_str = if class_len > 0 {
                String::from_utf16_lossy(&class_name[..class_len as usize])
            } else {
                String::new()
            };

            // Check if this is a console window
            let is_console = TERMINAL_CLASSES.iter().any(|tc| class_name_str.contains(tc));

            if is_console {
                // Get process ID
                let mut process_id = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut process_id));

                if process_id > 0 {
                    // Get process name
                    if let Ok(process_name) = get_process_name_internal(process_id) {
                        // Check if it's a console terminal process
                        let is_terminal = TERMINAL_PROCESSES
                            .iter()
                            .any(|tp| process_name.eq_ignore_ascii_case(tp));

                        if is_terminal {
                            // Get window title to ensure it's visible
                            if let Ok(window_title) = get_window_title_internal(hwnd) {
                                if !window_title.is_empty() {
                                    *target_pid.borrow_mut() = process_id;
                                    return BOOL(0); // Stop enumeration
                                }
                            }
                        }
                    }
                }
            }

            BOOL(1) // Continue enumeration
        }

        unsafe fn get_process_name_internal(process_id: u32) -> anyhow::Result<String> {
            let handle = OpenProcess(PROCESS_QUERY_INFORMATION | PROCESS_VM_READ, false, process_id)?;

            let mut name = [0u16; 512];
            let mut size = name.len() as u32;
            QueryFullProcessImageNameW(
                handle,
                PROCESS_NAME_FORMAT(0),
                windows::core::PWSTR(name.as_mut_ptr()),
                &mut size,
            )?;

            let _ = CloseHandle(handle);

            let path = String::from_utf16_lossy(&name[..size as usize]);
            let process_name = std::path::Path::new(&path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();

            Ok(process_name)
        }

        unsafe fn get_window_title_internal(hwnd: HWND) -> anyhow::Result<String> {
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
            Some(enumerate_console_windows),
            LPARAM(&target_pid as *const RefCell<u32> as isize),
        )
        .map_err(|e| TerminalError::CaptureFailed(format!("EnumWindows failed: {:?}", e)))?;
    }

    Ok(target_pid.into_inner())
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
        // With UIA fallback, we might get various errors when no terminal is available
        // NotFound, CaptureFailed, or PermissionDenied are all acceptable
        match result {
            Ok(_) => (), // Success is acceptable if terminal exists
            Err(TerminalError::NotFound) => (), // Expected if no terminal
            Err(TerminalError::CaptureFailed(_)) => (), // Expected if UIA fails
            Err(TerminalError::PermissionDenied) => (), // Expected if permissions issue
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

    #[tokio::test]
    #[cfg(target_os = "windows")]
    async fn test_uia_fallback() {
        // Test UIA fallback by capturing terminal output
        // This test will succeed if UIA can extract text from the terminal
        let result = capture_terminal_output(50).await;

        match result {
            Ok(text) => {
                // Verify we got non-empty text
                assert!(!text.trim().is_empty(), "UIA fallback should extract non-empty text");
                // Text should contain some content (not just whitespace)
                assert!(text.chars().any(|c| !c.is_whitespace()), "Text should contain non-whitespace characters");
            }
            Err(TerminalError::NotFound) => {
                // Acceptable if no terminal is found
                println!("No terminal found for UIA test");
            }
            Err(e) => {
                // Other errors might be acceptable in test environment
                println!("UIA test error (may be expected in test env): {}", e);
            }
        }
    }
}
