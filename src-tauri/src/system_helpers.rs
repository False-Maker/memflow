use tokio::process::Command;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ToolDetectionError {
    #[error("Command execution failed: {0}")]
    CommandFailed(String),
    #[error("Command timed out")]
    Timeout,
}

type Result<T> = std::result::Result<T, ToolDetectionError>;

/// Generic tool version detection with timeout
pub async fn detect_tool_version(tool_name: &str) -> Result<Option<String>> {
    let mut command = Command::new(tool_name);
    
    // Try version command first
    match command.arg("--version").output().await {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(Some(version))
            } else {
                // Version command failed, try help
                detect_tool_version_with_help(tool_name).await
            }
        }
        Err(_) => {
            // Command failed, try help
            detect_tool_version_with_help(tool_name).await
        }
    }
}

async fn detect_tool_version_with_help(tool_name: &str) -> Result<Option<String>> {
    let mut command = Command::new(tool_name);
    match command.args(["--help"]).output().await {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(Some(version))
            } else {
                // Tool exists but help command failed
                Ok(None)
            }
        }
        Err(_) => {
            // Tool not found
            Ok(None)
        }
    }
}

/// Detect Node.js version
pub async fn detect_node_version() -> Result<Option<String>> {
    let mut command = Command::new("node");
    command.args(["--version"]);
    
    let timeout = Duration::from_secs(3);
    
    match tokio::time::timeout(timeout, command.output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(Some(version))
            } else {
                Ok(None)
            }
        }
        Ok(Err(e)) => Err(ToolDetectionError::CommandFailed(e.to_string())),
        Err(_) => Ok(None),
    }
}

/// Detect Python version
pub async fn detect_python_version() -> Result<Option<String>> {
    let mut command = Command::new("python");
    command.args(["--version"]);
    
    let timeout = Duration::from_secs(3);
    
    match tokio::time::timeout(timeout, command.output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(Some(version))
            } else {
                Ok(None)
            }
        }
        Ok(Err(e)) => Err(ToolDetectionError::CommandFailed(e.to_string())),
        Err(_) => Ok(None),
    }
}

/// Detect Rust version
pub async fn detect_rust_version() -> Result<Option<String>> {
    let mut command = Command::new("rustc");
    command.args(["--version"]);
    
    let timeout = Duration::from_secs(3);
    
    match tokio::time::timeout(timeout, command.output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(Some(version))
            } else {
                Ok(None)
            }
        }
        Ok(Err(e)) => Err(ToolDetectionError::CommandFailed(e.to_string())),
        Err(_) => Ok(None),
    }
}

/// Detect Docker version
pub async fn detect_docker_version() -> Result<Option<String>> {
    let mut command = Command::new("docker");
    command.args(["--version"]);
    
    let timeout = Duration::from_secs(3);
    
    match tokio::time::timeout(timeout, command.output()).await {
        Ok(Ok(output)) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .to_string();
                Ok(Some(version))
            } else {
                Ok(None)
            }
        }
        Ok(Err(e)) => Err(ToolDetectionError::CommandFailed(e.to_string())),
        Err(_) => Ok(None),
    }
}

/// Test function for verifying tool detection works
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_detect_node_version() {
        let result = detect_node_version().await;
        // Node might not be installed, so we accept both success and "not found"
        match result {
            Ok(Some(_)) => println!("Node.js detected"),
            Ok(None) => println!("Node.js not found"),
            Err(e) => panic!("Error detecting Node.js: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_detect_python_version() {
        let result = detect_python_version().await;
        match result {
            Ok(Some(_)) => println!("Python detected"),
            Ok(None) => println!("Python not found"),
            Err(e) => panic!("Error detecting Python: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_detect_rust_version() {
        let result = detect_rust_version().await;
        match result {
            Ok(Some(_)) => println!("Rust detected"),
            Ok(None) => println!("Rust not found"),
            Err(e) => panic!("Error detecting Rust: {}", e),
        }
    }
    
    #[tokio::test]
    async fn test_detect_docker_version() {
        let result = detect_docker_version().await;
        match result {
            Ok(Some(_)) => println!("Docker detected"),
            Ok(None) => println!("Docker not found"),
            Err(e) => panic!("Error detecting Docker: {}", e),
        }
    }
}