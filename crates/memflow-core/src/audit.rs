//! Security Audit Module
//!
//! Provides audit logging for MCP tool calls with configurable redaction rules.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{info, warn};

/// Audit log entry for a tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub tool_name: String,
    pub params_summary: String,
    pub result_status: String,
    pub duration_ms: u64,
    pub client_info: Option<String>,
}

/// Redaction rule for sensitive data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedactionRule {
    pub name: String,
    pub pattern: String,
    pub replacement: String,
}

/// Audit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_path: PathBuf,
    pub max_file_size_mb: u64,
    pub retention_days: u32,
    pub redaction_rules: Vec<RedactionRule>,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            log_path: Self::default_log_path(),
            max_file_size_mb: 100,
            retention_days: 30,
            redaction_rules: Self::default_redaction_rules(),
        }
    }
}

impl AuditConfig {
    fn default_log_path() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("memflow")
            .join("audit.log")
    }

    fn default_redaction_rules() -> Vec<RedactionRule> {
        vec![
            RedactionRule {
                name: "api_key".to_string(),
                pattern: r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*["']?[a-zA-Z0-9_-]{16,}["']?"#
                    .to_string(),
                replacement: r#"$1: [REDACTED]"#.to_string(),
            },
            RedactionRule {
                name: "token".to_string(),
                pattern:
                    r#"(?i)(token|auth_token|access_token)\s*[:=]\s*["']?[a-zA-Z0-9_-]{8,}["']?"#
                        .to_string(),
                replacement: r#"$1: [REDACTED]"#.to_string(),
            },
            RedactionRule {
                name: "password".to_string(),
                pattern: r#"(?i)(password|passwd|pwd)\s*[:=]\s*["']?[^"'\s]+["']?"#.to_string(),
                replacement: r#"$1: [REDACTED]"#.to_string(),
            },
            RedactionRule {
                name: "email".to_string(),
                pattern: r#"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}"#.to_string(),
                replacement: r#"[EMAIL REDACTED]"#.to_string(),
            },
            RedactionRule {
                name: "ip_address".to_string(),
                pattern: r#"\b(?:\d{1,3}\.){3}\d{1,3}\b"#.to_string(),
                replacement: r#"[IP REDACTED]"#.to_string(),
            },
            RedactionRule {
                name: "path".to_string(),
                pattern: r#"(?i)(/[^/\s]+)+/(?:[^/\s]+)"#.to_string(),
                replacement: r#"[PATH REDACTED]"#.to_string(),
            },
        ]
    }
}

/// Audit logger
pub struct AuditLogger {
    config: AuditConfig,
    entries: Mutex<Vec<AuditEntry>>,
}

impl AuditLogger {
    /// Create a new audit logger with default config
    pub fn new() -> Self {
        Self::with_config(AuditConfig::default())
    }

    /// Create a new audit logger with custom config
    pub fn with_config(config: AuditConfig) -> Self {
        // Ensure log directory exists
        if let Some(parent) = config.log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        Self {
            config,
            entries: Mutex::new(Vec::new()),
        }
    }

    /// Load config from file
    pub fn load_config(path: &PathBuf) -> Option<AuditConfig> {
        match fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => Some(config),
                Err(e) => {
                    warn!("Failed to parse audit config: {}", e);
                    None
                }
            },
            Err(_) => None,
        }
    }

    /// Log a tool call
    pub fn log_tool_call(
        &self,
        tool_name: &str,
        params: &str,
        result_status: &str,
        duration_ms: u64,
        client_info: Option<String>,
    ) {
        if !self.config.enabled {
            return;
        }

        let redacted_params = self.redact_sensitive_data(params);
        let params_summary = self.summarize_params(&redacted_params);

        let entry = AuditEntry {
            timestamp: Utc::now(),
            tool_name: tool_name.to_string(),
            params_summary,
            result_status: result_status.to_string(),
            duration_ms,
            client_info,
        };

        // Add to in-memory buffer
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(entry.clone());

            // Flush to disk if buffer reaches 10 entries
            if entries.len() >= 10 {
                let entries_to_flush: Vec<AuditEntry> = entries.drain(..).collect();
                drop(entries);
                if let Err(e) = self.flush_to_disk(&entries_to_flush) {
                    warn!("Failed to flush audit log: {}", e);
                }
            }
        }

        info!(
            "Audit: {} - {} - {}ms",
            tool_name, result_status, duration_ms
        );
    }

    /// Redact sensitive data from params
    fn redact_sensitive_data(&self, params: &str) -> String {
        let mut result = params.to_string();

        for rule in &self.config.redaction_rules {
            if let Ok(regex) = regex::Regex::new(&rule.pattern) {
                result = regex.replace_all(&result, &rule.replacement).to_string();
            }
        }

        result
    }

    /// Summarize params for logging (truncate if too long)
    fn summarize_params(&self, params: &str) -> String {
        const MAX_LEN: usize = 500;
        if params.len() <= MAX_LEN {
            params.to_string()
        } else {
            format!("{}... [truncated]", &params[..MAX_LEN])
        }
    }

    /// Flush entries to disk
    fn flush_to_disk(&self, entries: &[AuditEntry]) -> std::io::Result<()> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.config.log_path)?;

        for entry in entries {
            let line = serde_json::to_string(entry)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
            writeln!(file, "{}", line)?;
        }

        Ok(())
    }

    /// Force flush all pending entries
    pub fn flush(&self) -> std::io::Result<()> {
        if let Ok(mut entries) = self.entries.lock() {
            if !entries.is_empty() {
                let entries_to_flush: Vec<AuditEntry> = entries.drain(..).collect();
                drop(entries);
                self.flush_to_disk(&entries_to_flush)?;
            }
        }
        Ok(())
    }

    /// Clean up old audit logs
    pub fn cleanup_old_logs(&self) -> std::io::Result<()> {
        // Implementation for log rotation and cleanup
        // This would check file age and size, and rotate or delete as needed
        Ok(())
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

/// Global audit logger instance
static AUDIT_LOGGER: std::sync::OnceLock<AuditLogger> = std::sync::OnceLock::new();

/// Initialize the global audit logger
pub fn init_audit_logger(config: Option<AuditConfig>) {
    let logger = config.map(AuditLogger::with_config).unwrap_or_default();
    let _ = AUDIT_LOGGER.set(logger);
}

/// Log a tool call using the global logger
pub fn log_tool_call(tool_name: &str, params: &str, result_status: &str, duration_ms: u64) {
    if let Some(logger) = AUDIT_LOGGER.get() {
        logger.log_tool_call(tool_name, params, result_status, duration_ms, None);
    }
}

/// Flush the global audit logger
pub fn flush_audit_log() {
    if let Some(logger) = AUDIT_LOGGER.get() {
        if let Err(e) = logger.flush() {
            warn!("Failed to flush audit log: {}", e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction_rules() {
        let logger = AuditLogger::new();

        let input =
            r#"{"api_key": "sk-1234567890abcdef", "email": "test@example.com", "normal": "data"}"#;
        let redacted = logger.redact_sensitive_data(input);

        assert!(!redacted.contains("sk-1234567890abcdef"));
        assert!(!redacted.contains("test@example.com"));
        assert!(redacted.contains("[REDACTED]") || redacted.contains("[EMAIL REDACTED]"));
        assert!(redacted.contains("normal"));
        assert!(redacted.contains("data"));
    }

    #[test]
    fn test_summarize_params() {
        let logger = AuditLogger::new();

        let short = "short params";
        assert_eq!(logger.summarize_params(short), short);

        let long = "a".repeat(600);
        let summarized = logger.summarize_params(&long);
        assert!(summarized.len() < 600);
        assert!(summarized.contains("truncated"));
    }

    #[test]
    fn test_audit_entry_serialization() {
        let entry = AuditEntry {
            timestamp: Utc::now(),
            tool_name: "search_memory".to_string(),
            params_summary: "{\"query\": \"test\"}".to_string(),
            result_status: "success".to_string(),
            duration_ms: 100,
            client_info: None,
        };

        let json = serde_json::to_string(&entry).unwrap();
        assert!(json.contains("search_memory"));
        assert!(json.contains("success"));
    }
}
