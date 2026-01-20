use crate::{app_config, db, ocr};
use tauri::{AppHandle, Emitter, Manager};
use tokio::time::{interval, Duration};

pub fn spawn_ocr_worker(app_handle: AppHandle) {
    tracing::info!("Inside spawn_ocr_worker (sync)");
    tauri::async_runtime::spawn(async move {
        tracing::info!("Inside spawn_ocr_worker (async task block start)");
        // Wait for a moment to ensure app is valid
        tokio::time::sleep(Duration::from_secs(2)).await;
        tracing::info!("OCR worker starting run_worker...");
        run_worker(app_handle).await;
    });
}

async fn run_worker(app_handle: AppHandle) {
    let mut ticker = interval(Duration::from_secs(5));
    tracing::info!("OCR Worker started");

    loop {
        ticker.tick().await;

        let config = match app_config::get_config().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("OCR Worker failed to get config: {}", e);
                continue;
            }
        };

        if !config.ocr_enabled {
            continue;
        }

        tracing::debug!("Worker fetching pending tasks...");
        // Fetch pending tasks
        let tasks = match db::get_pending_ocr_tasks(5).await {
            Ok(t) => t,
            Err(e) => {
                tracing::error!("Failed to fetch OCR tasks: {}", e);
                continue;
            }
        };

        if tasks.is_empty() {
             // tracing::debug!("No pending tasks"); // Reduce noise
             continue;
        }

        tracing::debug!("Found {} pending OCR tasks", tasks.len());

        // Process tasks
        for task in tasks {
            tracing::info!("Processing OCR task id={}", task.id);
            // Mark as processing
            if let Err(e) = db::update_ocr_queue_status(task.id, "processing", None).await {
                tracing::error!("Failed to mark task {} processing: {}", task.id, e);
                continue;
            }

            // Prepare OCR config
            tracing::debug!("Preparing OCR config for task {}", task.id);
            let mut ocr_config = ocr::OcrConfig::new(&config.ocr_engine)
                .with_redaction(config.ocr_redaction_enabled)
                .with_redaction_level(&config.ocr_redaction_level);

            if let Ok(resource_dir) = app_handle.path().resource_dir() {
                ocr_config = ocr_config.with_resource_dir(resource_dir);
            }

            // Get full path (need screenshots dir)
            let screenshots_dir = match db::get_screenshots_dir().await {
                Some(dir) => dir,
                None => {
                    tracing::error!("无法获取截图目录");
                    // If screenshots dir is missing, we probably can't do anything.
                    // Retry later.
                    let _ = db::update_ocr_queue_status(
                        task.id,
                        "pending",
                        Some("Screenshots directory not found"),
                    )
                    .await;
                    continue;
                }
            };

            let full_path = screenshots_dir.join(&task.image_path);
            let full_path_str = match full_path.to_str() {
                Some(s) => s,
                None => {
                    let _ = db::update_ocr_queue_status(
                        task.id,
                        "failed",
                        Some("Invalid image path"),
                    )
                    .await;
                    continue;
                }
            };

            // Run OCR
            match ocr::process_image(full_path_str, ocr_config).await {
                Ok(text) => {
                    // Update activity log
                    if let Err(e) = db::update_activity_ocr(task.activity_id, &text).await {
                        tracing::error!("Failed to update activity OCR: {}", e);
                        // DB update failed, retry
                        let _ = db::update_ocr_queue_status(task.id, "pending", Some(&e.to_string())).await;
                    } else {
                        // Mark done
                        let _ = db::update_ocr_queue_status(task.id, "done", None).await;

                        tracing::info!("OCR task {} completed (len: {})", task.id, text.len());

                        // Emit event
                        let update_data = serde_json::json!({
                            "id": task.activity_id,
                            "ocrText": text
                        });
                        if let Err(e) = app_handle.emit("ocr-updated", &update_data) {
                            tracing::warn!("Failed to emit ocr-updated: {}", e);
                        }
                    }
                }
                Err(e) => {
                    let err_msg = e.to_string();
                    tracing::warn!("OCR processing failed for task {}: {}", task.id, err_msg);

                    if task.retry_count >= 3 {
                        let _ = db::update_ocr_queue_status(task.id, "failed", Some(&err_msg))
                            .await;
                    } else {
                        let _ = db::update_ocr_queue_status(task.id, "pending", Some(&err_msg))
                            .await;
                    }
                }
            }
        }
    }
}
