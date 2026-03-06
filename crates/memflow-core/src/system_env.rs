use serde::{Deserialize, Serialize};
use std::env;
use std::net::TcpListener;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

/// Basic information about a developer tool installed on the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolVersion {
    pub name: String,
    pub command: String,
    pub version: Option<String>,
}

/// Simple port usage status for a few common local development ports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortStatus {
    pub port: u16,
    pub in_use: bool,
}

/// Collected system environment information suitable for both Tauri and MCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemEnvironment {
    pub os: String,
    pub arch: String,
    pub username: Option<String>,
    pub hostname: Option<String>,
    pub shell: Option<String>,
    pub home_dir: Option<String>,
    pub current_dir: Option<String>,
    pub logical_cpus: usize,
    pub tools: Vec<ToolVersion>,
    pub ports: Vec<PortStatus>,
}

/// Collect a snapshot of the local system environment.
///
/// This function is intentionally best-effort:
/// - It never fails the caller; any individual probe failure is recorded as None.
/// - It does not contact any remote services.
pub async fn collect_system_environment() -> SystemEnvironment {
    let os = env::consts::OS.to_string();
    let arch = env::consts::ARCH.to_string();

    let username = env::var("USERNAME")
        .or_else(|_| env::var("USER"))
        .ok();

    let hostname = env::var("COMPUTERNAME")
        .or_else(|_| env::var("HOSTNAME"))
        .ok();

    let shell = env::var("SHELL")
        .or_else(|_| env::var("COMSPEC"))
        .ok();

    let home_dir = dirs::home_dir().and_then(|p| p.to_str().map(|s| s.to_string()));
    let current_dir = env::current_dir()
        .ok()
        .and_then(|p| p.to_str().map(|s| s.to_string()));

    let logical_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    let tools = collect_tool_versions().await;
    let ports = detect_common_ports();

    SystemEnvironment {
        os,
        arch,
        username,
        hostname,
        shell,
        home_dir,
        current_dir,
        logical_cpus,
        tools,
        ports,
    }
}

/// Render the collected system environment into a human-readable text report
/// suitable for returning from MCP / Tauri commands.
pub async fn get_system_environment_report() -> String {
    let env = collect_system_environment().await;

    let mut text = String::new();
    text.push_str("系统环境概览（本地收集，不访问任何远程服务）：\n\n");

    text.push_str("[基础信息]\n");
    text.push_str(&format!("- OS：{}\n", env.os));
    text.push_str(&format!("- 架构：{}\n", env.arch));
    if let Some(user) = env.username {
        text.push_str(&format!("- 当前用户：{}\n", user));
    }
    if let Some(host) = env.hostname {
        text.push_str(&format!("- 主机名：{}\n", host));
    }
    if let Some(shell) = env.shell {
        text.push_str(&format!("- Shell / 终端：{}\n", shell));
    }
    if let Some(dir) = env.current_dir {
        text.push_str(&format!("- 当前工作目录：{}\n", dir));
    }
    if let Some(home) = env.home_dir {
        text.push_str(&format!("- 用户目录：{}\n", home));
    }
    text.push_str(&format!("- 逻辑 CPU 核心数：{}\n", env.logical_cpus));
    text.push_str("\n");

    text.push_str("[开发工具版本]\n");
    if env.tools.is_empty() {
        text.push_str("- 未能检测到常见开发工具（可能未安装，或当前 PATH 不可用）。\n");
    } else {
        for tool in env.tools {
            let ver = tool
                .version
                .unwrap_or_else(|| "未找到或调用失败".to_string());
            text.push_str(&format!("- {} ({}): {}\n", tool.name, tool.command, ver));
        }
    }
    text.push_str("\n");

    text.push_str("[常见本地端口占用（仅检测是否被占用）]\n");
    if env.ports.is_empty() {
        text.push_str("- 未检测任何端口。\n");
    } else {
        for p in env.ports {
            let status = if p.in_use { "占用" } else { "空闲" };
            text.push_str(&format!("- 端口 {}：{}\n", p.port, status));
        }
    }

    text
}

async fn collect_tool_versions() -> Vec<ToolVersion> {
    let candidates = vec![
        ("Git", "git", &["--version"][..]),
        ("Rustc", "rustc", &["--version"][..]),
        ("Cargo", "cargo", &["--version"][..]),
        ("Node.js", "node", &["-v"][..]),
        ("npm", "npm", &["-v"][..]),
        ("pnpm", "pnpm", &["-v"][..]),
        ("Python", "python", &["--version"][..]),
        ("Python (3)", "python3", &["--version"][..]),
    ];

    let mut tools = Vec::with_capacity(candidates.len());

    for (name, cmd, args) in candidates {
        let version = get_command_version(cmd, args).await;
        tools.push(ToolVersion {
            name: name.to_string(),
            command: cmd.to_string(),
            version,
        });
    }

    tools
}

async fn get_command_version(cmd: &str, args: &[&str]) -> Option<String> {
    let mut command = Command::new(cmd);
    command.args(args);

    let fut = command.output();

    let output = match timeout(Duration::from_secs(3), fut).await {
        Ok(Ok(out)) => out,
        _ => return None,
    };

    if !output.status.success() {
        return None;
    }

    let mut stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if stdout.is_empty() && !stderr.is_empty() {
        stdout = stderr;
    }

    if stdout.is_empty() {
        None
    } else {
        Some(stdout)
    }
}

fn detect_common_ports() -> Vec<PortStatus> {
    // A small set of common dev / local service ports
    let ports = [3000_u16, 5173, 5174, 8000, 8001, 8080, 9000];

    ports
        .iter()
        .map(|port| PortStatus {
            port: *port,
            in_use: is_port_in_use(*port),
        })
        .collect()
}

fn is_port_in_use(port: u16) -> bool {
    match TcpListener::bind(("127.0.0.1", port)) {
        Ok(listener) => {
            // Port is free; drop the listener immediately.
            drop(listener);
            false
        }
        Err(e) => e.kind() == std::io::ErrorKind::AddrInUse,
    }
}

