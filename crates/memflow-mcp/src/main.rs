use anyhow::Result;
use clap::Parser;
use std::io;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

/// Set DLL search path to include resources directory
/// This ensures we load our bundled onnxruntime.dll instead of system32's old version
/// The DLL should be placed in the `resources` folder next to the exe
#[cfg(target_os = "windows")]
fn init_dll_search_path() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    // Get the resources directory (exe/resources)
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resources_dir = exe_dir.join("resources");
            
            // Convert path to wide string for Windows API
            let wide_path: Vec<u16> = OsStr::new(&resources_dir)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            // SetDllDirectoryW - adds the path to DLL search order
            // This takes precedence over system32
            unsafe {
                windows_sys::Win32::System::LibraryLoader::SetDllDirectoryW(wide_path.as_ptr());
            }
        }
    }
}

/// Log DLL search path after logging is initialized
#[cfg(target_os = "windows")]
fn log_dll_search_path() {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let resources_dir = exe_dir.join("resources");
            tracing::info!("DLL search path set to: {:?}", resources_dir);
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn init_dll_search_path() {
    // No-op on non-Windows
}

#[cfg(not(target_os = "windows"))]
fn log_dll_search_path() {
    // No-op on non-Windows
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

mod context;
mod protocol;
mod server;
mod tools;

use context::McpContext;
use memflow_core::context::RuntimeContext;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize DLL search path BEFORE any other code runs
    // This ensures onnxruntime.dll is loaded from our bundle, not system32
    init_dll_search_path();

    // 🛑 关键修复：强制日志输出到 Stderr，绝对不能污染 Stdout！
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,sqlx=warn".into()); // 默认设为 info，方便调试 DLL 加载

    fmt()
        .with_env_filter(env_filter)
        .with_ansi(false) // <--- 禁止 ANSI 颜色，防止某些客户端解析出错
        .with_writer(io::stderr) // <--- 就是这一行！把日志赶到 Stderr 去
        .init();

    // Log DLL search path after logging is initialized
    log_dll_search_path();

    let _args = Args::parse();
    
    // Initialize context
    let ctx = McpContext::new();
    info!("memflow-mcp server starting...");
    info!("Resource dir: {:?}", ctx.resource_dir());

    // P3-4: Check Core health before starting server
    // If Core is unavailable, we'll log a warning but continue in degraded mode
    // so users can at least get tool schema information
    match ctx.check_core_health().await {
        Ok(_) => {
            info!("Core health check passed - running in full mode");
        }
        Err(e) => {
            // Log the error but don't exit - allow MCP to run in degraded mode
            // This lets at least schema queries work even if Core is down
            eprintln!("WARNING: Core unavailable - running in degraded mode. Error: {}", e);
        }
    }

    // Database initialization is handled lazily by memflow-core when first used.
    server::run_server(ctx).await
}
