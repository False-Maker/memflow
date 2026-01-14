use crate::app_config;
use crate::commands::ActivityLog;
use crate::db;
use crate::ocr;
use crate::window_info;
use anyhow::Result;
use image::DynamicImage;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::Semaphore;
use tokio::time::{interval, Duration};

static RECORDING: AtomicBool = AtomicBool::new(false);
static LAST_PHASH: once_cell::sync::Lazy<Arc<tokio::sync::Mutex<Option<String>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(tokio::sync::Mutex::new(None)));

// 全局 AppHandle 存储（用于 OCR 调用）
static APP_HANDLE: once_cell::sync::Lazy<Arc<tokio::sync::Mutex<Option<AppHandle>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(tokio::sync::Mutex::new(None)));

// OCR 并发控制信号量（限制同时执行的 OCR 任务数量）
// 设置为 2，允许最多 2 个 OCR 任务同时执行
static OCR_SEMAPHORE: once_cell::sync::Lazy<Arc<Semaphore>> =
    once_cell::sync::Lazy::new(|| Arc::new(Semaphore::new(2)));

pub fn init(app_handle: AppHandle) {
    *APP_HANDLE.blocking_lock() = Some(app_handle);
}

pub fn start() -> Result<()> {
    if RECORDING.swap(true, Ordering::SeqCst) {
        return Err(anyhow::anyhow!("录制已在进行中"));
    }

    tokio::spawn(async {
        recording_loop().await;
    });

    Ok(())
}

pub fn stop() -> Result<()> {
    RECORDING.store(false, Ordering::SeqCst);
    Ok(())
}

pub fn is_recording() -> bool {
    RECORDING.load(Ordering::SeqCst)
}

fn normalize_app_name(name: &str) -> String {
    let trimmed = name.trim().trim_matches('"');
    let file_name = std::path::Path::new(trimmed)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(trimmed);
    let mut lower = file_name.to_lowercase();
    if let Some(stripped) = lower.strip_suffix(".exe") {
        lower = stripped.to_string();
    }
    lower
}

async fn log_to_frontend(msg: &str) {
    println!("[DEBUG] {}", msg);
    tracing::info!("{}", msg);
    if let Some(handle) = APP_HANDLE.lock().await.as_ref() {
        use tauri::Emitter; // Tauri 2.0 使用 Emitter trait
        let _ = handle.emit("backend-log", msg);
    }
}

async fn recording_loop() {
    log_to_frontend("Starting recording_loop...").await;

    // 获取配置的录制间隔
    let config = match app_config::get_config().await {
        Ok(c) => {
            log_to_frontend(&format!(
                "Loaded config for recording: interval={}ms",
                c.recording_interval
            ))
            .await;
            c
        }
        Err(e) => {
            log_to_frontend(&format!("获取配置失败，使用默认间隔: {}", e)).await;
            let mut cfg: crate::commands::AppConfig = serde_json::from_str("{}").unwrap();
            cfg.ocr_enabled = true;
            cfg
        }
    };

    let interval_ms = if config.recording_interval < 100 {
        log_to_frontend(&format!(
            "Interval too small ({}), using 1000ms",
            config.recording_interval
        ))
        .await;
        1000
    } else {
        config.recording_interval
    };

    let mut interval_timer = interval(Duration::from_millis(interval_ms));

    log_to_frontend("Entering recording loop while loop...").await;
    while RECORDING.load(Ordering::SeqCst) {
        interval_timer.tick().await;

        // 再次检查标记，因为等待期间可能已停止
        if !RECORDING.load(Ordering::SeqCst) {
            log_to_frontend("Recording stopped during wait").await;
            break;
        }

        match capture_and_save().await {
            Ok(_) => {}
            Err(e) => log_to_frontend(&format!("截图录制失败 details: {:?}", e)).await,
        }
    }
    log_to_frontend("Exited recording loop").await;
}

async fn capture_and_save() -> Result<()> {
    // 0. Check Privacy Mode
    let mut config = match app_config::get_config().await {
        Ok(c) => c,
        Err(_) => return Ok(()),
    };

    let now = chrono::Utc::now().timestamp();
    if config.privacy_mode_enabled {
        if let Some(until) = config.privacy_mode_until {
            if now > until {
                // Auto disable
                config.privacy_mode_enabled = false;
                config.privacy_mode_until = None;
                if let Some(handle) = APP_HANDLE.lock().await.as_ref() {
                    let _ = app_config::update_config(config.clone(), handle.clone()).await;
                }
            } else {
                tracing::debug!("Privacy mode active, skipping recording");
                let _ = tokio::spawn(async {
                    let _ = db::increment_skipped_stat("privacy_mode").await;
                });
                return Ok(());
            }
        } else {
            tracing::debug!("Privacy mode active, skipping recording");
            let _ = tokio::spawn(async {
                let _ = db::increment_skipped_stat("privacy_mode").await;
            });
            return Ok(());
        }
    }

    // 1. 获取前台窗口信息
    let window_info = window_info::get_foreground_window_info()?;

    // Check Blocklist
    if config.blocklist_enabled {
        let blocklist = db::get_blocklist().await?;
        let app_name_norm = normalize_app_name(&window_info.process_name);
        let is_blocked = blocklist
            .iter()
            .any(|b| normalize_app_name(b) == app_name_norm);

        if config.blocklist_mode == "blocklist" {
            if is_blocked {
                tracing::debug!("App {} is in blocklist, skipping", window_info.process_name);
                let _ = tokio::spawn(async {
                    let _ = db::increment_skipped_stat("blocklist").await;
                });
                return Ok(());
            }
        } else if config.blocklist_mode == "allowlist" {
            if !is_blocked {
                tracing::debug!(
                    "App {} is not in allowlist, skipping",
                    window_info.process_name
                );
                let _ = tokio::spawn(async {
                    let _ = db::increment_skipped_stat("allowlist_miss").await;
                });
                return Ok(());
            }
        }
    }

    // 3. 截图
    let screenshot = capture_screen()?;

    // 4. 计算 pHash
    let phash_str = calculate_phash(&screenshot)?;

    // 5. 检查是否与上一帧相同（汉明距离阈值）
    let last_phash = LAST_PHASH.lock().await.clone();
    if let Some(ref last) = last_phash {
        if phash_str == *last {
            // 相同帧，只更新时间戳
            if let Some(_activity_id) = db::find_activity_by_phash(&phash_str).await? {
                tracing::debug!("检测到重复帧，跳过保存: {}", phash_str);
                return Ok(());
            }
        }
    }

    // 6. 保存截图
    let screenshots_dir = db::get_screenshots_dir()
        .await
        .ok_or_else(|| anyhow::anyhow!("截图目录未初始化"))?;

    let timestamp = chrono::Utc::now().timestamp();
    // 使用 pHash 的前 16 个字符作为文件名的一部分
    let phash_short = if phash_str.len() >= 16 {
        &phash_str[..16]
    } else {
        &phash_str
    };
    let filename = format!("{}_{}.png", timestamp, phash_short);
    let file_path = screenshots_dir.join(&filename);

    // 保存为 PNG 格式
    screenshot.save_with_format(&file_path, image::ImageFormat::Png)?;

    // 7. 保存到数据库
    let activity_id = match db::insert_activity(
        timestamp,
        &window_info.process_name,
        &window_info.title,
        &filename,
        Some(&phash_str),
        Some(&window_info.process_path),
    )
    .await
    {
        Ok(id) => id,
        Err(e) => {
            if db::is_database_corrupted(&e) {
                tracing::error!("Database corrupted during insert. Triggering recovery...");
                if let Some(app_handle) = APP_HANDLE.lock().await.clone() {
                    if let Err(recovery_err) = db::force_recovery(app_handle).await {
                        tracing::error!("Recovery failed: {}", recovery_err);
                        return Err(e);
                    }
                    // Recovery succeeded, retry the insert with the new pool
                    tracing::info!("Recovery succeeded, retrying insert...");
                    match db::insert_activity(
                        timestamp,
                        &window_info.process_name,
                        &window_info.title,
                        &filename,
                        Some(&phash_str),
                        Some(&window_info.process_path),
                    )
                    .await
                    {
                        Ok(id) => id,
                        Err(retry_err) => {
                            tracing::error!("Insert retry failed: {}", retry_err);
                            return Err(retry_err);
                        }
                    }
                } else {
                    return Err(e);
                }
            } else {
                return Err(e);
            }
        }
    };

    // 8. 发送新活动事件到前端
    if let Some(app_handle) = APP_HANDLE.lock().await.as_ref() {
        use tauri::Emitter;
        let activity = ActivityLog {
            id: activity_id,
            timestamp,
            app_name: window_info.process_name.clone(),
            window_title: window_info.title.clone(),
            image_path: filename.clone(),
            ocr_text: None, // OCR 是异步的，稍后会更新
            phash: Some(phash_str.clone()),
        };
        if let Err(e) = app_handle.emit("new-activity", &activity) {
            tracing::warn!("发送 new-activity 事件失败: {}", e);
        } else {
            tracing::debug!("已发送 new-activity 事件: id={}", activity_id);
        }
    }

    // 9. 更新最后 pHash
    *LAST_PHASH.lock().await = Some(phash_str.clone());

    // 10. 触发 OCR（异步，不阻塞，带并发控制）
    let config = app_config::get_config().await?;
    if config.ocr_enabled {
        let image_path = filename.clone();
        let ocr_engine = config.ocr_engine.clone();
        let ocr_redaction_enabled = config.ocr_redaction_enabled;
        let ocr_redaction_level = config.ocr_redaction_level.clone();

        // 获取 AppHandle（用于资源目录）
        let app_handle = APP_HANDLE.lock().await.clone();

        // 获取信号量的 Arc 克隆
        let semaphore = OCR_SEMAPHORE.clone();

        tokio::spawn(async move {
            // 获取信号量许可（限制并发 OCR 任务数量）
            let _permit = match semaphore.acquire().await {
                Ok(permit) => permit,
                Err(_) => {
                    tracing::warn!("OCR 信号量已关闭");
                    return;
                }
            };

            // 构建 OCR 配置
            let mut ocr_config = ocr::OcrConfig::new(&ocr_engine)
                .with_redaction(ocr_redaction_enabled)
                .with_redaction_level(ocr_redaction_level);

            // 如果有 AppHandle，设置资源目录
            if let Some(handle) = &app_handle {
                use tauri::Manager;
                if let Ok(resource_dir) = handle.path().resource_dir() {
                    ocr_config = ocr_config.with_resource_dir(resource_dir);
                }
            }

            // 获取完整图片路径
            let screenshots_dir = match db::get_screenshots_dir().await {
                Some(dir) => dir,
                None => {
                    tracing::error!("无法获取截图目录");
                    return;
                }
            };

            let full_path = screenshots_dir.join(&image_path);
            let full_path_str = match full_path.to_str() {
                Some(s) => s,
                None => {
                    tracing::error!("路径转换失败");
                    return;
                }
            };

            // 执行 OCR
            match ocr::process_image(full_path_str, ocr_config).await {
                Ok(text) => {
                    // 更新数据库
                    if let Err(e) = db::update_activity_ocr(activity_id, &text).await {
                        tracing::error!("更新 OCR 文本失败: {}", e);
                    } else {
                        tracing::info!("OCR 识别成功，文本长度: {}", text.len());

                        // 发送 OCR 更新事件到前端
                        if let Some(handle) = &app_handle {
                            use tauri::Emitter;
                            let update_data = serde_json::json!({
                                "id": activity_id,
                                "ocrText": text
                            });
                            if let Err(e) = handle.emit("ocr-updated", &update_data) {
                                tracing::warn!("发送 ocr-updated 事件失败: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("OCR 处理失败: {}", e);
                }
            }
            // _permit 在这里自动释放
        });
    }

    tracing::info!(
        "已保存活动记录: {} - {}",
        window_info.process_name,
        window_info.title
    );

    Ok(())
}

fn capture_screen() -> Result<DynamicImage> {
    // 使用 xcap 截图
    let monitors = xcap::Monitor::all()?;
    let monitor = monitors
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("未找到显示器"))?;

    let image_buffer = monitor.capture_image()?;

    // xcap 返回的是 ImageBuffer<Rgba<u8>, Vec<u8>>
    // 转换为 DynamicImage
    let width = image_buffer.width();
    let height = image_buffer.height();
    let raw_pixels: Vec<u8> = image_buffer
        .pixels()
        .flat_map(|p| [p.0[0], p.0[1], p.0[2], p.0[3]])
        .collect();

    let rgba_image = image::RgbaImage::from_raw(width, height, raw_pixels)
        .ok_or_else(|| anyhow::anyhow!("无法创建图像"))?;

    Ok(DynamicImage::ImageRgba8(rgba_image))
}

fn calculate_phash(image: &DynamicImage) -> Result<String> {
    // 使用简化的哈希实现，避免 img_hash 版本兼容问题
    // TODO: 在未来版本中集成更好的感知哈希算法
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // 将图像转换为灰度并计算简单哈希
    let gray = image::imageops::grayscale(image);
    let resized = image::imageops::resize(&gray, 8, 8, image::imageops::FilterType::Nearest);

    let mut hasher = DefaultHasher::new();
    for pixel in resized.as_raw() {
        pixel.hash(&mut hasher);
    }

    let hash = hasher.finish();
    Ok(format!("{:016x}", hash))
}
