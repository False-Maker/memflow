//! Mock runtime context for testing
//!
//! This module provides a mock implementation of the runtime context
//! for use in unit tests.

/// Mock runtime context for testing
#[derive(Debug, Clone)]
pub struct MockContext {
    /// Simulated active window info
    pub active_window: Option<WindowInfo>,
    /// Simulated system info
    pub system_info: SystemInfo,
    /// Read-only mode flag
    pub read_only: bool,
    /// Authorized flag
    pub authorized: bool,
}

/// Window information
#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub app_name: String,
    pub window_title: String,
    pub pid: u32,
}

/// System information
#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub os_version: String,
    pub memory_total: u64,
    pub cpu_count: usize,
    pub hostname: String,
}

impl MockContext {
    /// Create a new mock context with default values
    pub fn new() -> Self {
        Self {
            active_window: Some(WindowInfo {
                app_name: "VSCode".to_string(),
                window_title: "test_project".to_string(),
                pid: 1234,
            }),
            system_info: SystemInfo {
                os_version: "Windows 11".to_string(),
                memory_total: 16_000_000_000,
                cpu_count: 8,
                hostname: "test-machine".to_string(),
            },
            read_only: false,
            authorized: true,
        }
    }

    /// Create a mock context in read-only mode
    pub fn read_only() -> Self {
        let mut ctx = Self::new();
        ctx.read_only = true;
        ctx
    }

    /// Create a mock context that is unauthorized
    pub fn unauthorized() -> Self {
        let mut ctx = Self::new();
        ctx.authorized = false;
        ctx
    }

    /// Get the active window info
    pub fn get_active_window(&self) -> Option<&WindowInfo> {
        self.active_window.as_ref()
    }

    /// Get system info
    pub fn get_system_info(&self) -> &SystemInfo {
        &self.system_info
    }

    /// Check if in read-only mode
    pub fn is_read_only(&self) -> bool {
        self.read_only
    }

    /// Check if authorized
    pub fn is_authorized(&self) -> bool {
        self.authorized
    }

    /// Set active window
    pub fn set_active_window(&mut self, window: Option<WindowInfo>) {
        self.active_window = window;
    }

    /// Simulate terminal output capture
    pub fn get_terminal_output(&self, _lines: usize) -> Result<String, String> {
        if self
            .active_window
            .as_ref()
            .map(|w| w.app_name.contains("Terminal"))
            .unwrap_or(false)
        {
            Ok("mock terminal output\nline 2\nline 3".to_string())
        } else {
            Err("No active terminal".to_string())
        }
    }
}

impl Default for MockContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_context_new() {
        let ctx = MockContext::new();
        assert!(ctx.is_authorized());
        assert!(!ctx.is_read_only());
        assert!(ctx.get_active_window().is_some());
    }

    #[test]
    fn test_mock_context_read_only() {
        let ctx = MockContext::read_only();
        assert!(ctx.is_read_only());
    }

    #[test]
    fn test_mock_context_unauthorized() {
        let ctx = MockContext::unauthorized();
        assert!(!ctx.is_authorized());
    }

    #[test]
    fn test_get_system_info() {
        let ctx = MockContext::new();
        let info = ctx.get_system_info();
        assert_eq!(info.cpu_count, 8);
        assert_eq!(info.memory_total, 16_000_000_000);
    }

    #[test]
    fn test_terminal_output_with_terminal() {
        let mut ctx = MockContext::new();
        ctx.set_active_window(Some(WindowInfo {
            app_name: "Windows Terminal".to_string(),
            window_title: "PowerShell".to_string(),
            pid: 5678,
        }));

        let output = ctx.get_terminal_output(10);
        assert!(output.is_ok());
    }

    #[test]
    fn test_terminal_output_without_terminal() {
        let ctx = MockContext::new();
        let output = ctx.get_terminal_output(10);
        assert!(output.is_err());
    }
}
