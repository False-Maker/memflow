//! UIA (UI Automation) text extraction for MemFlow Core
//!
//! This module provides full UIA text extraction for activity records.
//! Uses Windows UI Automation API for high-performance text extraction.

use anyhow::Result;
use std::time::Instant;

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowTextLengthW,
};

#[cfg(windows)]
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, CoCreateInstance, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};

#[cfg(windows)]
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTreeWalker,
    UIA_TextControlTypeId, UIA_EditControlTypeId, UIA_DocumentControlTypeId,
    UIA_ControlTypePropertyId, UIA_NamePropertyId, UIA_ValueValuePropertyId,
};

#[cfg(windows)]
use windows::core::BSTR;

/// UIA 遍历性能熔断常量
const MAX_TRAVERSAL_DEPTH: u32 = 5;
const MAX_TRAVERSAL_TIME_MS: u128 = 200;

/// UIA text extraction result
#[derive(Debug, Clone)]
pub struct UiaTextResult {
    /// Extracted text content
    pub text: Option<String>,
    /// Whether UIA was available
    pub available: bool,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Extract text from the foreground window using UIA
#[cfg(windows)]
pub fn extract_uia_text(hwnd: isize) -> Result<UiaTextResult> {
    let start = Instant::now();

    let hwnd = HWND(hwnd as *mut std::ffi::c_void);

    // Try to get window text content using UIA
    match get_window_text_content(hwnd) {
        Ok(Some(text)) => {
            Ok(UiaTextResult {
                text: Some(text),
                available: true,
                processing_time_ms: start.elapsed().as_millis() as u64,
            })
        }
        Ok(None) => {
            // Fallback to window title
            match get_window_title(hwnd) {
                Ok(title) => Ok(UiaTextResult {
                    text: title,
                    available: true,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                }),
                Err(_) => Ok(UiaTextResult {
                    text: None,
                    available: false,
                    processing_time_ms: start.elapsed().as_millis() as u64,
                }),
            }
        }
        Err(_) => Ok(UiaTextResult {
            text: None,
            available: false,
            processing_time_ms: start.elapsed().as_millis() as u64,
        }),
    }
}

#[cfg(not(windows))]
pub fn extract_uia_text(_hwnd: isize) -> Result<UiaTextResult> {
    Ok(UiaTextResult {
        text: None,
        available: false,
        processing_time_ms: 0,
    })
}

/// Get foreground window's text using UIA
#[cfg(windows)]
pub fn get_foreground_text() -> Result<Option<String>> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return Ok(None);
        }

        match get_window_text_content(hwnd) {
            Ok(Some(text)) if !text.trim().is_empty() => Ok(Some(text)),
            _ => get_window_title(hwnd),
        }
    }
}

#[cfg(not(windows))]
pub fn get_foreground_text() -> Result<Option<String>> {
    Ok(None)
}

/// Get window title
#[cfg(windows)]
pub fn get_window_title(hwnd: HWND) -> Result<Option<String>> {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return Ok(None);
        }

        let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
        let copied = GetWindowTextW(hwnd, &mut buffer);

        if copied == 0 {
            return Ok(None);
        }

        let title = String::from_utf16_lossy(&buffer[..copied as usize]);
        if title.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(title))
        }
    }
}

#[cfg(not(windows))]
pub fn get_window_title(_hwnd: isize) -> Result<Option<String>> {
    Ok(None)
}

/// Get window text content using UIA
#[cfg(windows)]
pub fn get_window_text_content(hwnd: HWND) -> Result<Option<String>> {
    unsafe {
        // COM initialization
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        if hr.is_err() && hr.0 != 1 {
            return Ok(None);
        }

        let _guard = scopeguard::guard((), |_| {
            CoUninitialize();
        });

        // Create UIA instance
        let automation: IUIAutomation = match CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
            Ok(a) => a,
            Err(_) => return Ok(None),
        };

        // Get window element
        let element: IUIAutomationElement = match automation.ElementFromHandle(hwnd) {
            Ok(e) => e,
            Err(_) => return Ok(None),
        };

        // Create TreeWalker
        let walker: IUIAutomationTreeWalker = match automation.ControlViewWalker() {
            Ok(w) => w,
            Err(_) => return Ok(None),
        };

        // Traverse tree
        let start_time = Instant::now();
        let mut texts: Vec<String> = Vec::new();

        walk_tree(&walker, &element, 0, &start_time, &mut texts);

        if texts.is_empty() {
            Ok(None)
        } else {
            let combined = texts.join("\n");
            if combined.len() > 10000 {
                Ok(Some(combined.chars().take(10000).collect()))
            } else {
                Ok(Some(combined))
            }
        }
    }
}

#[cfg(not(windows))]
pub fn get_window_text_content(_hwnd: isize) -> Result<Option<String>> {
    Ok(None)
}

/// Traverse UI tree with performance limits
#[cfg(windows)]
fn walk_tree(
    walker: &IUIAutomationTreeWalker,
    element: &IUIAutomationElement,
    depth: u32,
    start_time: &Instant,
    texts: &mut Vec<String>,
) {
    // Check limits
    if depth > MAX_TRAVERSAL_DEPTH {
        return;
    }
    if start_time.elapsed().as_millis() > MAX_TRAVERSAL_TIME_MS {
        return;
    }

    unsafe {
        // Check if element is text type
        if let Ok(control_type) = element.GetCurrentPropertyValue(UIA_ControlTypePropertyId) {
            let ct_val: i32 = control_type.as_raw().Anonymous.Anonymous.Anonymous.lVal;
            let is_text_element = ct_val == UIA_TextControlTypeId.0
                || ct_val == UIA_EditControlTypeId.0
                || ct_val == UIA_DocumentControlTypeId.0;

            if is_text_element {
                // Get Name property
                if let Ok(name) = element.GetCurrentPropertyValue(UIA_NamePropertyId) {
                    if let Ok(bstr) = BSTR::try_from(&name) {
                        let text = bstr.to_string();
                        if !text.trim().is_empty() {
                            texts.push(text);
                        }
                    }
                }

                // Get Value property (for Edit controls)
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

        // Traverse children
        if let Ok(first_child) = walker.GetFirstChildElement(element) {
            walk_tree(walker, &first_child, depth + 1, start_time, texts);

            let mut current = first_child;
            while let Ok(next) = walker.GetNextSiblingElement(&current) {
                if start_time.elapsed().as_millis() > MAX_TRAVERSAL_TIME_MS {
                    return;
                }
                walk_tree(walker, &next, depth + 1, start_time, texts);
                current = next;
            }
        }
    }
}

/// Get foreground window handle
#[cfg(windows)]
pub fn get_foreground_hwnd() -> Option<isize> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            None
        } else {
            Some(hwnd.0 as isize)
        }
    }
}

#[cfg(not(windows))]
pub fn get_foreground_hwnd() -> Option<isize> {
    None
}

/// Check if UIA is available on this system
pub fn is_uia_available() -> bool {
    #[cfg(windows)]
    {
        // Try to initialize COM to check if UIA is available
        unsafe {
            let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
            if hr.is_err() && hr.0 != 1 {
                return false;
            }
            if hr.0 == 1 {
                // Already initialized, don't uninitialize
                return true;
            }
            CoUninitialize();
            true
        }
    }

    #[cfg(not(windows))]
    {
        false
    }
}
