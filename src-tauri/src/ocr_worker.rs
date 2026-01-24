use crate::{app_config, db, ocr};
use once_cell::sync::Lazy;
use std::io::Cursor;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::Notify;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;
use tokio::time::{interval, Duration};

static OCR_NOTIFY: Lazy<Notify> = Lazy::new(Notify::new);

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

pub fn notify_new_task() {
    OCR_NOTIFY.notify_one();
}

async fn run_worker(app_handle: AppHandle) {
    let mut ticker = interval(Duration::from_secs(5));
    tracing::info!("OCR Worker started");

    loop {
        tokio::select! {
            _ = ticker.tick() => {}
            _ = OCR_NOTIFY.notified() => {}
        }

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

        let screenshots_dir = match db::get_screenshots_dir().await {
            Some(dir) => dir,
            None => {
                tracing::error!("无法获取截图目录");
                continue;
            }
        };

        let limiter = Arc::new(Semaphore::new(2));
        let fetch_limit = 10_i64;
        let preprocess_enabled = config.ocr_preprocess_enabled;
        let preprocess_target_width = config.ocr_preprocess_target_width;
        let preprocess_max_pixels = config.ocr_preprocess_max_pixels;

        loop {
            tracing::debug!("Worker fetching pending tasks...");
            let tasks = match db::get_pending_ocr_tasks(fetch_limit).await {
                Ok(t) => t,
                Err(e) => {
                    tracing::error!("Failed to fetch OCR tasks: {}", e);
                    break;
                }
            };

            if tasks.is_empty() {
                break;
            }

            tracing::debug!("Found {} pending OCR tasks", tasks.len());

            let engine = config.ocr_engine.clone();
            let redaction_enabled = config.ocr_redaction_enabled;
            let redaction_level = config.ocr_redaction_level.clone();
            let resource_dir = app_handle.path().resource_dir().ok();

            let mut join_set: JoinSet<()> = JoinSet::new();
            for task in tasks {
                let app_handle = app_handle.clone();
                let engine = engine.clone();
                let redaction_level = redaction_level.clone();
                let resource_dir = resource_dir.clone();
                let screenshots_dir = screenshots_dir.clone();
                let limiter = limiter.clone();
                let preprocess_enabled = preprocess_enabled;
                let preprocess_target_width = preprocess_target_width;
                let preprocess_max_pixels = preprocess_max_pixels;

                join_set.spawn(async move {
                    let _permit = match limiter.acquire().await {
                        Ok(p) => p,
                        Err(_) => return,
                    };

                    tracing::info!("Processing OCR task id={}", task.id);

                    if let Err(e) = db::update_ocr_queue_status(task.id, "processing", None).await {
                        tracing::error!("Failed to mark task {} processing: {}", task.id, e);
                        return;
                    }

                    let mut ocr_config = ocr::OcrConfig::new(&engine)
                        .with_redaction(redaction_enabled)
                        .with_redaction_level(&redaction_level);

                    if let Some(resource_dir) = resource_dir {
                        ocr_config = ocr_config.with_resource_dir(resource_dir);
                    }

                    let full_path = screenshots_dir.join(&task.image_path);
                    let mut input_path = full_path.clone();
                    let mut tmp_path: Option<PathBuf> = None;

                    if preprocess_enabled {
                        let t_preprocess = std::time::Instant::now();
                        let src_path = full_path.clone();
                        let target_width = preprocess_target_width;
                        let max_pixels = preprocess_max_pixels;

                        let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Option<Vec<u8>>> {
                            use image::GenericImageView;
                            let img = image::open(&src_path)?;
                            let (w, h) = img.dimensions();
                            let pixels = w as u64 * h as u64;

                            if w <= target_width && pixels <= max_pixels {
                                return Ok(None);
                            }

                            let new_w = target_width.max(1).min(w);
                            let ratio = new_w as f64 / w.max(1) as f64;
                            let new_h = ((h as f64) * ratio).round().max(1.0) as u32;

                            let resized =
                                img.resize_exact(new_w, new_h, image::imageops::FilterType::Triangle);

                            let mut buf: Vec<u8> = Vec::new();
                            let mut cursor = Cursor::new(&mut buf);
                            resized.write_to(&mut cursor, image::ImageFormat::Png)?;
                            Ok(Some(buf))
                        })
                        .await;

                        if let Ok(Ok(Some(png_bytes))) = result {
                            let nanos = SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap_or_default()
                                .as_nanos();
                            let tmp = screenshots_dir.join(format!("ocr_tmp_{}_{}.png", task.id, nanos));
                            if tokio::fs::write(&tmp, &png_bytes).await.is_ok() {
                                tmp_path = Some(tmp.clone());
                                input_path = tmp;
                                tracing::debug!(
                                    preprocess_ms = t_preprocess.elapsed().as_millis(),
                                    preprocessed_bytes = png_bytes.len(),
                                    "ocr preprocess applied"
                                );
                            }
                        }
                    }

                    let input_path_str = match input_path.to_str() {
                        Some(s) => s,
                        None => {
                            let _ = db::update_ocr_queue_status(
                                task.id,
                                "failed",
                                Some("Invalid image path"),
                            )
                            .await;
                            if let Some(tmp_path) = tmp_path {
                                let _ = tokio::fs::remove_file(tmp_path).await;
                            }
                            return;
                        }
                    };

                    let t_ocr = std::time::Instant::now();
                    let ocr_result = ocr::process_image(input_path_str, ocr_config).await;

                    if let Some(tmp_path) = tmp_path {
                        let _ = tokio::fs::remove_file(tmp_path).await;
                    }

                    match ocr_result {
                        Ok(text) => {
                            let ocr_ms = t_ocr.elapsed().as_millis();
                            let t_db = std::time::Instant::now();
                            if let Err(e) = db::update_activity_ocr(task.activity_id, &text).await {
                                tracing::error!("Failed to update activity OCR: {}", e);
                                let _ = db::update_ocr_queue_status(
                                    task.id,
                                    "pending",
                                    Some(&e.to_string()),
                                )
                                .await;
                                return;
                            }
                            let db_ms = t_db.elapsed().as_millis();

                            let _ = db::update_ocr_queue_status(task.id, "done", None).await;
                            tracing::info!("OCR task {} completed (len: {})", task.id, text.len());
                            tracing::debug!(ocr_ms = ocr_ms, db_ms = db_ms, "ocr_worker timing");

                            let update_data = serde_json::json!({
                                "id": task.activity_id,
                                "ocrText": text
                            });
                            if let Err(e) = app_handle.emit("ocr-updated", &update_data) {
                                tracing::warn!("Failed to emit ocr-updated: {}", e);
                            }
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            tracing::warn!("OCR processing failed for task {}: {}", task.id, err_msg);

                            if task.retry_count >= 3 {
                                let _ =
                                    db::update_ocr_queue_status(task.id, "failed", Some(&err_msg))
                                        .await;
                            } else {
                                let _ =
                                    db::update_ocr_queue_status(task.id, "pending", Some(&err_msg))
                                        .await;
                            }
                        }
                    }
                });
            }

            while join_set.join_next().await.is_some() {}
        }
    }
}
