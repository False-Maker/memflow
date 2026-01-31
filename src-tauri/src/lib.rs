pub mod desktop_context;
pub mod ai;
pub mod app_config;
pub mod chat;
pub mod commands;
pub mod db;
pub mod graph;
pub mod ocr;
pub mod performance;
pub mod protocol;
pub mod recorder;
pub mod secure_storage;
pub mod vector_db;
pub mod window_info;
pub mod focus_analytics;
pub mod ocr_worker;
pub mod proactive_context;
pub mod redact;
pub mod scheduler;
pub mod uia;
pub mod win_event;

use tracing_subscriber::prelude::*;
use tauri::Manager;

static LOG_GUARD: once_cell::sync::Lazy<std::sync::Mutex<Option<tracing_appender::non_blocking::WorkerGuard>>> =
    once_cell::sync::Lazy::new(|| std::sync::Mutex::new(None));

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        ])
        .setup(|app| {
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

            // 初始化配置和数据库
            tauri::async_runtime::spawn(async move {
                if let Err(e) = app_config::init_config(app_handle.clone()).await {
                    tracing::error!("CRITICAL: Config init failed: {:#}", e);
                    tracing::error!("CRITICAL: Config init failed (debug): {:?}", e);
                    eprintln!("CRITICAL: Config init failed: {:#}", e);
                }
                
                // 初始化 Prompts 配置（从资源目录加载）
                let resource_path = app_handle.path().resource_dir().ok();
                if let Err(e) = ai::prompts::init_prompts(resource_path).await {
                    tracing::warn!("Prompts 配置初始化失败，使用默认值: {}", e);
                } else {
                    tracing::info!("Prompts 配置初始化完成");
                }

                tracing::info!("Starting database initialization...");
                if let Err(e) = db::init_db(app_handle.clone()).await {
                    let error_msg = format!("CRITICAL: Database init failed: {}", e);
                    tracing::error!("{}", error_msg);
                    tracing::error!("CRITICAL: Database init failed (debug): {:?}", e);
                    eprintln!("{}", error_msg);

                    let (kind, hint) = db::diagnose_init_error(&e);
                    tracing::error!("Database init failure kind: {:?}. {}", kind, hint);
                    eprintln!("Database init hint: {}", hint);
                     
                    // 记录详细的诊断信息
                    if let Ok(db_path) = db::get_db_path_for_diagnostics(&app_handle) {
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

            Ok(())
        })
        .on_window_event(|_window, event| {
            // 应用退出时停止 OCR 服务
            if let tauri::WindowEvent::Destroyed = event {
                let _ = recorder::stop();
                ocr::service::stop_service();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
