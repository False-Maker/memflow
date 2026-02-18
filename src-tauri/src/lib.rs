pub mod ai;
pub mod app_config;
pub mod chat;
pub mod commands;
pub mod db;
pub mod desktop_context;
pub mod focus_analytics;
pub mod graph;
pub mod ocr;
pub mod ocr_worker;
pub mod performance;
pub mod proactive_context;
pub mod protocol;
pub mod recorder;
pub mod redact;
pub mod scheduler;
pub mod secure_storage;
pub mod system_helpers;
pub mod uia;
pub mod vector_db;
pub mod win_event;
pub mod window_info;

use std::time::Duration;
use tauri::image::Image;
use tauri::menu::{MenuBuilder, MenuItemBuilder};
use tauri::tray::TrayIconBuilder;
use tauri::Emitter;
use tauri::Manager;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tracing_subscriber::prelude::*;

static LOG_GUARD: once_cell::sync::Lazy<
    std::sync::Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>,
> = once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

#[derive(Clone, Copy, PartialEq, Eq)]
enum RunMode {
    Ui,
    TrayOnly,
    Headless,
}

fn parse_run_mode() -> RunMode {
    let mut selected: Option<RunMode> = None;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "--headless" => selected = Some(RunMode::Headless),
            "--tray-only" => selected = Some(RunMode::TrayOnly),
            "--ui" => selected = Some(RunMode::Ui),
            _ => {}
        }
    }
    selected.unwrap_or(RunMode::TrayOnly)
}

fn show_or_create_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("main") {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }

    let window = WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("MemFlow")
        .inner_size(1200.0, 800.0)
        .min_inner_size(800.0, 600.0)
        .resizable(true)
        .build();

    if let Ok(win) = window {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn show_or_create_debug_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("debug") {
        let _ = win.show();
        let _ = win.set_focus();
        return;
    }

    let window =
        WebviewWindowBuilder::new(app, "debug", WebviewUrl::App("index.html?debug=1".into()))
            .title("MemFlow Debug")
            .inner_size(860.0, 680.0)
            .min_inner_size(680.0, 520.0)
            .resizable(true)
            .build();

    if let Ok(win) = window {
        let _ = win.show();
        let _ = win.set_focus();
    }
}

fn mcp_heartbeat_path(app_handle: &AppHandle) -> Option<std::path::PathBuf> {
    let tauri_hb = app_handle
        .path()
        .app_data_dir()
        .map(|d| d.join("mcp_heartbeat.json"))
        .ok();
    let alt_hb = dirs::data_dir().map(|d| d.join("com.memflow.app").join("mcp_heartbeat.json"));
    tauri_hb
        .filter(|p| p.exists())
        .or_else(|| alt_hb.filter(|p| p.exists()))
}

fn mcp_heartbeat_online(app_handle: &AppHandle) -> bool {
    let max_age_secs = 8;
    if let Some(path) = mcp_heartbeat_path(app_handle) {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                let status = val
                    .get("status")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let ts = val.get("ts").and_then(|v| v.as_i64()).unwrap_or(0);
                let now = chrono::Local::now().timestamp();
                let delta = now - ts;
                return status == "online" && delta >= 0 && delta <= max_age_secs;
            }
        }
        if let Ok(meta) = std::fs::metadata(&path) {
            if let Ok(modified) = meta.modified() {
                if let Ok(elapsed) = modified.elapsed() {
                    return elapsed.as_secs() <= max_age_secs as u64;
                }
            }
        }
    }
    false
}

fn status_label(value: bool) -> &'static str {
    if value {
        "Running"
    } else {
        "Stopped"
    }
}

fn mcp_label(app_handle: &AppHandle) -> &'static str {
    if mcp_heartbeat_online(app_handle) {
        "Online"
    } else {
        "Offline"
    }
}

fn format_tray_status(app_handle: &AppHandle) -> String {
    let ocr_running = ocr::service::is_service_running_quiet();
    let recording = recorder::is_recording();
    format!(
        "Status: OCR {} | MCP {} | REC {}",
        status_label(ocr_running),
        mcp_label(app_handle),
        status_label(recording)
    )
}

fn format_status_dialog(app_handle: &AppHandle) -> String {
    let ocr_running = ocr::service::is_service_running();
    let recording = recorder::is_recording();
    format!(
        "OCR: {}\nMCP: {}\nRecorder: {}",
        status_label(ocr_running),
        mcp_label(app_handle),
        status_label(recording)
    )
}

fn start_mcp_if_needed(app_handle: &AppHandle) {
    if mcp_heartbeat_online(app_handle) {
        return;
    }
    let exe_path = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            tracing::warn!("Failed to resolve current exe: {}", e);
            return;
        }
    };
    let exe_dir = match exe_path.parent() {
        Some(dir) => dir.to_path_buf(),
        None => {
            tracing::warn!("Failed to resolve current exe dir");
            return;
        }
    };
    let resource_dir = app_handle.path().resource_dir().ok();
    let mut candidates = vec![exe_dir.join("memflow-mcp.exe"), exe_dir.join("memflow-mcp")];
    if let Some(dir) = resource_dir {
        candidates.push(dir.join("memflow-mcp.exe"));
        candidates.push(dir.join("memflow-mcp"));
    }
    let candidate = candidates.into_iter().find(|p| p.exists());
    let Some(path) = candidate else {
        tracing::warn!("memflow-mcp binary not found near app executable");
        return;
    };
    let _ = std::process::Command::new(path)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    run_with_mode(parse_run_mode());
}

pub fn run_headless() {
    run_with_mode(RunMode::Headless);
}

fn run_with_mode(run_mode: RunMode) {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::start_recording,
            commands::stop_recording,
            commands::get_activities,
            commands::get_activity_by_id,
            commands::get_config,
            commands::update_config,
            commands::set_privacy_mode,
            commands::search_activities,
            commands::get_blocklist,
            commands::add_blocklist_item,
            commands::remove_blocklist_item,
            commands::clear_blocklist,
            commands::get_stats,
            commands::get_activity_heatmap_stats,
            commands::get_app_usage_stats,
            commands::get_hourly_activity_stats,
            commands::get_focus_metrics,
            commands::get_image_path,
            commands::get_graph_data,
            commands::rebuild_graph,
            commands::get_performance_metrics,
            commands::trigger_gc,
            commands::ai_chat,
            commands::ai_chat_stream,
            commands::test_chat_connection,
            commands::test_embedding_connection,
            commands::save_api_key,
            commands::get_api_key,
            commands::delete_api_key,
            // 对话历史相关命令
            commands::create_chat_session,
            commands::save_chat_message,
            commands::update_session_title,
            commands::get_chat_sessions,
            commands::get_chat_messages,
            commands::delete_chat_session,
            commands::clear_all_chat_history,
            // 反馈相关命令
            commands::rate_message,
            commands::submit_feedback,
            commands::get_user_feedbacks,
            // 智能代理（自动化提案/执行/审计）
            commands::agent_propose_automation,
            commands::agent_execute_automation,
            commands::agent_list_executions,
            commands::agent_cancel_execution,
            commands::run_retention_cleanup,
            commands::get_recording_stats,
            commands::get_ocr_queue_stats,
            commands::get_system_status,
        ])
        .setup(move |app| {
            let mut filter = tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

            if let Ok(directive) = "reqwest::blocking::client=warn".parse() {
                filter = filter.add_directive(directive);
            }

            let log_dir = app
                .path()
                .app_data_dir()
                .map(|d| d.join("logs"))
                .unwrap_or_else(|_| std::env::temp_dir().join("memflow-logs"));

            let _ = std::fs::create_dir_all(&log_dir);
            tracing::info!("Logging directory: {}", log_dir.display());
            let file_appender = tracing_appender::rolling::daily(&log_dir, "memflow.log");
            let (file_writer, guard) = tracing_appender::non_blocking(file_appender);
            *LOG_GUARD.lock().unwrap() = Some(guard);

            let _ = tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_ansi(false)
                        .with_writer(file_writer),
                )
                .try_init();

            let app_handle = app.handle().clone();
            let mcp_handle = app_handle.clone();
            tauri::async_runtime::spawn_blocking(move || {
                start_mcp_if_needed(&mcp_handle);
            });

            let ocr_handle = app_handle.clone();
            tauri::async_runtime::spawn_blocking(move || {
                tracing::info!("Starting OCR service...");
                if let Err(e) = ocr::service::start_service(&ocr_handle) {
                    tracing::warn!("OCR service failed to start: {}", e);
                    eprintln!("WARNING: OCR service failed to start: {}", e);
                }
            });

            // 初始化录制器（传递 AppHandle）
            recorder::init(app_handle.clone());

            // 初始化后台 OCR Worker
            tracing::info!("Calling ocr_worker::spawn_ocr_worker...");
            ocr_worker::spawn_ocr_worker(app_handle.clone());
            tracing::info!("ocr_worker::spawn_ocr_worker returned.");

            let config_handle = app_handle.clone();
            // 初始化配置和数据库
            tauri::async_runtime::spawn(async move {
                if let Err(e) = app_config::init_config(config_handle.clone()).await {
                    tracing::error!("CRITICAL: Config init failed: {:#}", e);
                    tracing::error!("CRITICAL: Config init failed (debug): {:?}", e);
                    eprintln!("CRITICAL: Config init failed: {:#}", e);
                }

                // 初始化 Prompts 配置（从资源目录加载）
                let resource_path = config_handle.path().resource_dir().ok();
                if let Err(e) = ai::prompts::init_prompts(resource_path).await {
                    tracing::warn!("Prompts 配置初始化失败，使用默认值: {}", e);
                } else {
                    tracing::info!("Prompts 配置初始化完成");
                }

                tracing::info!("Starting database initialization...");
                if let Err(e) = db::init_db(config_handle.clone()).await {
                    let error_msg = format!("CRITICAL: Database init failed: {}", e);
                    tracing::error!("{}", error_msg);
                    tracing::error!("CRITICAL: Database init failed (debug): {:?}", e);
                    eprintln!("{}", error_msg);

                    let (kind, hint) = db::diagnose_init_error(&e);
                    tracing::error!("Database init failure kind: {:?}. {}", kind, hint);
                    eprintln!("Database init hint: {}", hint);

                    // 记录详细的诊断信息
                    if let Ok(db_path) = db::get_db_path_for_diagnostics(&config_handle) {
                        tracing::error!(
                            "诊断信息 - 数据库路径: {}, 请检查文件权限和是否被其他进程占用",
                            db_path.display()
                        );
                    }
                } else {
                    tracing::info!("Database initialization completed successfully.");
                    // 启动自动清理调度器 (等待数据库初始化完成后)
                    scheduler::spawn_retention_scheduler();
                }
            });

        let status = MenuItemBuilder::with_id("status", "Status: Initializing")
                .enabled(true)
                .build(app)?;
            let show = MenuItemBuilder::with_id("show", "Open Dashboard").build(app)?;
            let quit = MenuItemBuilder::with_id("quit", "Quit Memflow").build(app)?;
            
            let show_main = MenuItemBuilder::with_id("show-main", "显示主窗口").build(app)?;
            let start_recording = MenuItemBuilder::with_id("start-recording", "开始录制").build(app)?;
            let stop_recording = MenuItemBuilder::with_id("stop-recording", "停止录制").build(app)?;
            let settings = MenuItemBuilder::with_id("settings", "设置").build(app)?;

            let menu = MenuBuilder::new(app)
                .items(&[&show_main, &start_recording, &stop_recording, &settings])
                .separator()
                .items(&[&status, &show])
                .separator()
                .items(&[&quit])
                .build()?;

          let icon = Image::from_bytes(include_bytes!("../icons/icon.png"))?;
            let tray = TrayIconBuilder::new()
                .menu(&menu)
                .icon(icon)
                .tooltip("MemFlow")
                .on_menu_event(|app, event| match event.id().as_ref() {
                    "status" => {
                        let message = format_status_dialog(app);
                        app.dialog()
                            .message(message)
                            .title("Memflow Status")
                            .kind(MessageDialogKind::Info)
                            .buttons(MessageDialogButtons::Ok)
                            .show(|_| {});
                    }
                    "show" | "show-main" => show_or_create_main_window(&app),
                    "start-recording" => {
                        let _ = app.emit("start_recording", ());
                    }
                    "stop-recording" => {
                        let _ = app.emit("stop_recording", ());
                    }
                    "settings" => {
                        // Open settings - trigger settings modal or command
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.emit("open-settings", {});
                        }
                    }
                    "quit" => std::process::exit(0),
                    _ => {}
                })
                .build(app)?;

            let status_handle = status.clone();
            let status_app = app_handle.clone();
            let _ = status_handle.set_text(format_tray_status(&status_app));
            tauri::async_runtime::spawn(async move {
                loop {
                    let _ = status_handle.set_text(format_tray_status(&status_app));
                    tokio::time::sleep(Duration::from_secs(5)).await;
                }
            });

            if run_mode == RunMode::Headless {
                tracing::info!("Starting in Headless Mode...");
            } else {
                show_or_create_main_window(&app_handle);
            }

            Ok(())
        })
      .on_window_event(move |window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                if run_mode != RunMode::Ui {
                    let _ = window.hide();
                    return;
                }
            }
            if let tauri::WindowEvent::Destroyed = event {
                if run_mode == RunMode::Ui {
                    let _ = recorder::stop();
                    ocr::service::stop_service();
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
