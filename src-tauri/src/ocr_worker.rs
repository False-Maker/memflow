use crate::{app_config, db, ocr, desktop_context::TauriContext, vector_db};
use memflow_core::context::RuntimeContext;
use memflow_core::ocr_enhance;
use once_cell::sync::Lazy;
use sqlx::Row;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
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
            // 使用统一的 RuntimeContext 获取资源目录
            let resource_dir = {
                let ctx = TauriContext::new(app_handle.clone());
                Some(ctx.resource_dir())
            };

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

                        let result = tokio::task::spawn_blocking(move || {
                            ocr_enhance::preprocess_terminal_image(
                                &src_path,
                                target_width,
                                max_pixels,
                            )
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
                            // 保留原始 OCR 输出，便于与增强后文本做质量评估（例如 CER/WER）。
                            let raw_text = text;
                            let mut processed_text = raw_text.clone();

                            let is_code = ocr_enhance::is_likely_code(&processed_text);
                            let detected_lang = ocr_enhance::detect_language(&processed_text);

                            if is_code {
                                processed_text =
                                    ocr_enhance::postprocess_terminal_text(&processed_text);
                            }

                            // 使用增强前后的文本做一次“自对比”质量评估：
                            // - reference: 增强/后处理后的文本
                            // - hypothesis: 原始 OCR 文本
                            // 这可以粗略反映后处理对文本结构的修正幅度，便于后续回归对比。
                            let quality =
                                ocr_enhance::evaluate_ocr_quality(&processed_text, &raw_text);

                            let ocr_ms = t_ocr.elapsed().as_millis();
                            let t_db = std::time::Instant::now();
                            if let Err(e) =
                                db::update_activity_ocr(task.activity_id, &processed_text).await
                            {
                                tracing::error!("Failed to update activity OCR: {}", e);
                                let _ = db::update_ocr_queue_status(
                                    task.id,
                                    "pending",
                                    Some(&e.to_string()),
                                )
                                .await;
                                return;
                            }

                            // 质量指标单独更新，失败不会影响主写入路径。
                            if let Err(e) = db::update_activity_ocr_quality(
                                task.activity_id,
                                quality.cer,
                                quality.wer,
                                quality.score,
                            )
                            .await
                            {
                                tracing::debug!(
                                    "Failed to update OCR quality metrics for activity {}: {}",
                                    task.activity_id,
                                    e
                                );
                            }

                            let db_ms = t_db.elapsed().as_millis();

                            let _ = db::update_ocr_queue_status(task.id, "done", None).await;
                            tracing::info!(
                                "OCR task {} completed (len: {}, is_code: {}, lang: {:?})",
                                task.id,
                                processed_text.len(),
                                is_code,
                                detected_lang
                            );
                            tracing::debug!(ocr_ms = ocr_ms, db_ms = db_ms, "ocr_worker timing");

                            let update_data = serde_json::json!({
                                "id": task.activity_id,
                                "ocrText": processed_text
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

/// Spawn the vectorization background worker
pub fn spawn_vectorize_worker(app_handle: AppHandle) {
    tracing::info!("Spawning vectorize worker");
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(2)).await;
        run_vectorize_worker(app_handle).await;
    });
}

/// Run vectorization once at startup to convert historical data
/// This will regenerate ALL embeddings using the local Chinese model (BGE-small-zh-v1.5)
/// regardless of whether they already have vectors, to ensure consistency.
pub fn run_startup_vectorize() {
    tracing::info!("Scheduling startup vectorization");
    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(Duration::from_secs(5)).await;
        
        let config = match app_config::get_config().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Startup vectorize: failed to get config: {}", e);
                return;
            }
        };

        if !config.vectorize_enabled {
            tracing::info!("Startup vectorize: disabled in config");
            return;
        }

        // 获取所有有 OCR 文本的活动（不只 pending），用于重新生成向量
        let pool = match memflow_core::db::get_pool().await {
            Ok(p) => p,
            Err(e) => {
                tracing::error!("Startup vectorize: failed to get DB pool: {}", e);
                return;
            }
        };

        let rows = match sqlx::query(
            "SELECT id, ocr_text FROM activity_logs WHERE ocr_text IS NOT NULL AND ocr_text != ''"
        )
        .fetch_all(&pool)
        .await
        {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Startup vectorize: failed to get activities: {}", e);
                return;
            }
        };

        if rows.is_empty() {
            tracing::info!("Startup vectorize: no activities with OCR text");
            return;
        }

        tracing::info!("Startup vectorize: found {} activities to vectorize", rows.len());

        let mut success_count = 0;
        let mut fail_count = 0;

        for row in rows {
            let activity_id: i64 = row.get(0);
            let ocr_text: String = row.get(1);

            if ocr_text.is_empty() {
                continue;
            }

            // 生成新向量
            match vector_db::generate_embedding(&ocr_text).await {
                Ok(embedding) => {
                    // 先删除旧向量
                    sqlx::query("DELETE FROM vector_embeddings WHERE activity_id = ?")
                        .bind(activity_id)
                        .execute(&pool)
                        .await
                        .ok();

                    // 插入新向量
                    match memflow_core::vector_db::insert_embedding(activity_id, embedding).await {
                        Ok(_) => {
                            success_count += 1;
                        }
                        Err(e) => {
                            tracing::warn!("Startup vectorize: insert failed for {}: {}", activity_id, e);
                            fail_count += 1;
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Startup vectorize: generate failed for {}: {}", activity_id, e);
                    fail_count += 1;
                }
            }
        }

        tracing::info!(
            "Startup vectorize completed: {}/{} succeeded, {} failed",
            success_count,
            success_count + fail_count,
            fail_count
        );
    });
}

/// Background worker that periodically converts OCR text to vector embeddings
async fn run_vectorize_worker(_app_handle: AppHandle) {
    tracing::info!("Vectorize worker started");

    loop {
        // Get config for interval
        let interval_secs = match app_config::get_config().await {
            Ok(c) => {
                if !c.vectorize_enabled {
                    // If disabled, wait longer before checking again
                    Duration::from_secs(60)
                } else {
                    Duration::from_secs(c.vectorize_interval)
                }
            }
            Err(_) => {
                tracing::warn!("Vectorize worker: failed to get config, using default interval");
                Duration::from_secs(300)
            }
        };

        tokio::time::sleep(interval_secs).await;

        let config = match app_config::get_config().await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Vectorize worker: failed to get config: {}", e);
                continue;
            }
        };

        if !config.vectorize_enabled {
            continue;
        }

        // Get pending vectorization tasks
        let pending = match db::get_pending_vectorize_tasks(config.vectorize_batch_size).await {
            Ok(ids) => ids,
            Err(e) => {
                tracing::error!("Failed to get pending vectorize tasks: {}", e);
                continue;
            }
        };

        if pending.is_empty() {
            continue;
        }

        tracing::info!("Found {} pending vectorization tasks", pending.len());

        let mut success_count = 0;
        let mut fail_count = 0;

        for activity_id in pending {
            // Get activity OCR text
            let activity = match db::get_activity_by_id(activity_id).await {
                Ok(a) => a,
                Err(e) => {
                    tracing::warn!("Failed to get activity {}: {}", activity_id, e);
                    fail_count += 1;
                    continue;
                }
            };

            let text = activity.ocr_text.unwrap_or_default();
            if text.is_empty() {
                tracing::debug!("Activity {} has no OCR text, skipping", activity_id);
                continue;
            }

            // Generate embedding
            match vector_db::generate_embedding(&text).await {
                Ok(embedding) => {
                    // Store in database
                    if let Err(e) = memflow_core::vector_db::insert_embedding(activity_id, embedding).await {
                        tracing::warn!("Failed to insert embedding for activity {}: {}", activity_id, e);
                        fail_count += 1;
                    } else {
                        success_count += 1;
                        tracing::debug!("Vectorized activity {}", activity_id);
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to generate embedding for activity {}: {}", activity_id, e);
                    fail_count += 1;
                }
            }
        }

        if success_count > 0 || fail_count > 0 {
            tracing::info!(
                "Vectorization completed: {}/{} succeeded, {} failed",
                success_count,
                success_count + fail_count,
                fail_count
            );
        }
    }
}
