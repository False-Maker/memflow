//! MemFlow Core Daemon
//!
//! A standalone daemon process that handles activity recording,
//! data storage, and provides IPC interface for Desktop UI and MCP.

use anyhow::Result;
use clap::{Parser, Subcommand};
use memflow_core::collection::{ActivityCollector, CollectionConfig};
use memflow_core::ipc::{IpcServer, DEFAULT_IPC_PORT};
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[cfg(target_os = "windows")]
use std::ffi::OsStr;
#[cfg(target_os = "windows")]
use std::os::windows::ffi::OsStrExt;

/// Set DLL search path to include resources directory
/// This ensures we load our bundled onnxruntime.dll instead of system32's old version
/// The DLL should be placed in the `resources` folder next to the exe
#[cfg(target_os = "windows")]
fn init_dll_search_path() {
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

/// MemFlow Core Daemon
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// IPC server port
    #[arg(long, default_value_t = DEFAULT_IPC_PORT)]
    port: u16,

    /// Data directory (default: ~/.memflow)
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Log level
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Run in foreground (don't daemonize)
    #[arg(long, default_value_t = false)]
    foreground: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Start the daemon
    Start,
    /// Stop the daemon
    Stop,
    /// Get daemon status
    Status,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize DLL search path BEFORE any other code runs
    // This ensures onnxruntime.dll is loaded from our bundle, not system32
    init_dll_search_path();

    let args = Args::parse();

    // Initialize logging
    let log_dir = args.data_dir
        .clone()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".memflow"))
        .join("logs");
    
    std::fs::create_dir_all(&log_dir)?;

    let file_appender = tracing_appender::rolling::daily(&log_dir, "memflow-daemon.log");
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(args.log_level.clone()));

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(tracing_subscriber::fmt::layer().with_writer(file_writer).with_ansi(false))
        .init();

    // Log DLL search path after logging is initialized
    log_dll_search_path();

    tracing::info!("MemFlow Core Daemon starting...");
    tracing::info!("IPC port: {}", args.port);
    tracing::info!("Data directory: {:?}", args.data_dir);

    // Handle commands
    match args.command {
        Some(Commands::Start) => {
            start_daemon(args).await?;
        }
        Some(Commands::Stop) => {
            stop_daemon(args.port).await?;
        }
        Some(Commands::Status) => {
            check_status(args.port).await?;
        }
        None => {
            // Default: start daemon
            start_daemon(args).await?;
        }
    }

    Ok(())
}

async fn start_daemon(args: Args) -> Result<()> {
    tracing::info!("Starting MemFlow Core daemon...");

    // Get data directory
    let data_dir = args.data_dir
        .clone()
        .unwrap_or_else(|| dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".memflow"));
    
    // Create screenshots directory
    let screenshots_dir = data_dir.join("screenshots");
    std::fs::create_dir_all(&screenshots_dir)?;
    tracing::info!("Screenshots directory: {:?}", screenshots_dir);

    // Create and start IPC server
    let mut server = IpcServer::new(args.port);
    server.register_default_handlers().await;
    
    // Create ActivityCollector
    let collector = Arc::new(ActivityCollector::new(server.state_manager()));
    
    // Set screenshots directory
    collector.set_screenshots_dir(screenshots_dir);
    
    // Set default config
    let config = CollectionConfig::default();
    collector.update_config(config).await;
    
    // Start collection loop (it will check state before capturing)
    collector.start().await?;
    
    tracing::info!("Activity collector initialized");
    
    // Start the server - handle the error manually
    if let Err(e) = server.start().await {
        tracing::error!("Failed to start IPC server: {}", e);
        return Err(anyhow::anyhow!("Failed to start IPC server: {}", e));
    }

    tracing::info!("MemFlow Core daemon started successfully");
    tracing::info!("IPC server listening on 127.0.0.1:{}", args.port);

    // Keep the daemon running
    // In production, this would be managed by system service (systemd, Windows service, etc.)
    tokio::signal::ctrl_c().await?;

    tracing::info!("Shutting down MemFlow Core daemon...");
    server.stop();

    Ok(())
}

async fn stop_daemon(port: u16) -> Result<()> {
    use memflow_core::ipc_client::IpcClient;

    tracing::info!("Stopping MemFlow Core daemon...");

    let client = IpcClient::with_port(port);
    
    match client.stop_recording().await {
        Ok(_) => {
            tracing::info!("Daemon stopped successfully");
        }
        Err(e) => {
            tracing::warn!("Failed to stop daemon (may not be running): {}", e);
        }
    }

    Ok(())
}

async fn check_status(port: u16) -> Result<()> {
    use memflow_core::ipc_client::IpcClient;

    let client = IpcClient::with_port(port);

    if client.ping().await {
        let status = client.get_status().await?;
        println!("MemFlow Core daemon is running");
        println!("Status: {}", status);
    } else {
        println!("MemFlow Core daemon is not running");
    }

    Ok(())
}
