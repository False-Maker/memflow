use crate::app_config;
use crate::commands::ActivityLog;
use crate::db;
use crate::focus_analytics;
use crate::window_info;
use anyhow::Result;
use image::DynamicImage;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::time::Duration;

static RECORDING: AtomicBool = AtomicBool::new(false);
static LAST_PHASH: once_cell::sync::Lazy<Arc<tokio::sync::Mutex<Option<String>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(tokio::sync::Mutex::new(None)));

// 文本状态追踪（用于智能混合去重）
static LAST_TEXT_HASH: once_cell::sync::Lazy<Arc<tokio::sync::Mutex<Option<u64>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(tokio::sync::Mutex::new(None)));

// 全局 AppHandle 存储（用于 OCR 调用）
static APP_HANDLE: once_cell::sync::Lazy<Arc<tokio::sync::Mutex<Option<AppHandle>>>> =
    once_cell::sync::Lazy::new(|| Arc::new(tokio::sync::Mutex::new(None)));

static HEARTBEAT_SECS: AtomicU64 = AtomicU64::new(60);

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

    // 启动专注度分析（如果已启用）
    focus_analytics::spawn_if_enabled();

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

fn adjust_heartbeat(on_duplicate: bool) {
    let (min, max) = (10_u64, 60_u64);
    let step = 5_u64;
    loop {
        let current = HEARTBEAT_SECS.load(Ordering::Relaxed);
        let next = if on_duplicate {
            (current + step).min(max)
        } else {
            current.saturating_sub(step).max(min)
        };
        if HEARTBEAT_SECS
            .compare_exchange(current, next, Ordering::Relaxed, Ordering::Relaxed)
            .is_ok()
        {
            break;
        }
    }
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
    use crate::win_event::{EventDrivenRecorder, EventLoopConfig, WindowEvent};
    
    log_to_frontend("Starting event-driven recording_loop...").await;

    // 获取配置
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

    // 心跳间隔：使用配置的录制间隔作为最大兜底时间，最小 10 秒，最大 60 秒
    let heartbeat_secs = (config.recording_interval / 1000).max(10).min(60);
    HEARTBEAT_SECS.store(heartbeat_secs, Ordering::Relaxed);
    
    // 初始化事件驱动录制器
    let event_config = EventLoopConfig {
        track_foreground: true,
        track_lifecycle: false,
        track_title_change: false,
        debounce_ms: 100, // 底层事件轮询间隔 100ms
    };
    let mut event_recorder = EventDrivenRecorder::new(event_config);
    let mut event_rx = event_recorder.start();
    
    // 事件防抖计时器：500ms 内的连续事件只处理最后一个
    let mut last_capture_time = std::time::Instant::now();
    let debounce_duration = Duration::from_millis(500);

    log_to_frontend(&format!(
        "Event-driven recording started (heartbeat: {}s, debounce: 500ms)",
        HEARTBEAT_SECS.load(Ordering::Relaxed)
    ))
    .await;

    while RECORDING.load(Ordering::SeqCst) {
        let sleep_secs = HEARTBEAT_SECS.load(Ordering::Relaxed);
        tokio::select! {
            // A. 响应系统事件（窗口切换）
            Some(event) = event_rx.recv() => {
                if let WindowEvent::ForegroundChanged { hwnd: _ } = event {
                    // 防抖检查：500ms 内不重复处理
                    if last_capture_time.elapsed() > debounce_duration {
                        last_capture_time = std::time::Instant::now();
                        tracing::debug!("事件驱动触发录制");
                        match capture_and_save().await {
                            Ok(_) => {}
                            Err(e) => {
                                log_to_frontend(&format!("事件驱动录制失败: {:?}", e)).await;
                            }
                        }
                    } else {
                        tracing::debug!("事件防抖跳过，距上次: {:?}", last_capture_time.elapsed());
                    }
                }
            }
            // B. 兜底心跳（定时采样，防止静止场景漏录）
            _ = tokio::time::sleep(Duration::from_secs(sleep_secs)) => {
                if RECORDING.load(Ordering::SeqCst) {
                    tracing::debug!("心跳触发录制 ({}s)", sleep_secs);
                    match capture_and_save().await {
                        Ok(_) => {}
                        Err(e) => {
                            log_to_frontend(&format!("心跳录制失败: {:?}", e)).await;
                        }
                    }
                    last_capture_time = std::time::Instant::now();
                }
            }
        }
    }
    
    // 停止事件录制器
    event_recorder.stop();
    log_to_frontend("Exited event-driven recording loop").await;
}

async fn capture_and_save() -> Result<()> {
    let t_total = std::time::Instant::now();
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

    // 2.5 尝试 UIA 获取文本（混合采集管线）
    let t_uia = std::time::Instant::now();
    let uia_text = tokio::task::spawn_blocking(|| -> Result<Option<String>> {
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.0.is_null() {
            return Ok(None);
        }
        crate::uia::get_window_text_content(hwnd)
    })
    .await??;

    let uia_text = uia_text.and_then(|text| {
        if text.trim().is_empty() {
            None
        } else {
            Some(text)
        }
    });

    if let Some(ref text) = uia_text {
        tracing::info!("UIA 提取成功，文本长度: {}", text.len());
    } else {
        tracing::debug!("UIA 未获取到文本，将使用 OCR");
    }

    let uia_ms = t_uia.elapsed().as_millis();

    let t_capture = std::time::Instant::now();
    let (webp_bytes, current_hash) = tokio::task::spawn_blocking(|| -> Result<(Vec<u8>, u64)> {
        let screenshot = capture_screen()?;
        let current_hash = calculate_phash_u64(&screenshot)?;

        let rgba_image = screenshot.to_rgba8();
        let encoder =
            webp::Encoder::from_rgba(&rgba_image, rgba_image.width(), rgba_image.height());
        let webp_memory = encoder.encode(80.0);

        Ok((webp_memory.to_vec(), current_hash))
    })
    .await??;

    let capture_ms = t_capture.elapsed().as_millis();
    let phash_str = format!("{:016x}", current_hash);

    // 5. 计算文本 hash（用于智能混合去重）
    let current_text_hash = uia_text.as_ref().map(|text| {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        hasher.finish()
    });

    // 6. 智能混合去重：只有当画面和文本都没变时，才跳过
    let last_phash = LAST_PHASH.lock().await.clone();
    let last_text_hash = LAST_TEXT_HASH.lock().await.clone();
    
    let visual_changed = if let Some(ref last) = last_phash {
        if let Some(last_hash) = parse_phash(last) {
            let distance = hamming_distance(current_hash, last_hash);
            distance > DEDUP_HAMMING_THRESHOLD
        } else {
            true // 解析失败视为变化
        }
    } else {
        true // 首次运行视为变化
    };

    let text_changed = match (&current_text_hash, &last_text_hash) {
        (Some(curr), Some(last)) => curr != last,
        (Some(_), None) | (None, Some(_)) => true, // 一方有文本一方没有视为变化
        (None, None) => false, // 都没有文本视为未变化
    };

    if !visual_changed && !text_changed {
        // 画面和文本都没变，跳过保存
        tracing::debug!(
            "检测到重复帧：视觉未变，文本未变，跳过保存"
        );
        adjust_heartbeat(true);
        let _ = tokio::spawn(async {
            let _ = db::increment_skipped_stat("duplicate_frame").await;
        });
        return Ok(());
    }
    
    // 有变化时记录原因
    tracing::debug!(
        "检测到变化：visual_changed={}, text_changed={}",
        visual_changed,
        text_changed
    );

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
    let filename = format!("{}_{}.webp", timestamp, phash_short);
    let file_path = screenshots_dir.join(&filename);

    let t_write = std::time::Instant::now();
    tokio::fs::write(&file_path, &webp_bytes).await?;
    let write_ms = t_write.elapsed().as_millis();

    // 7. 保存到数据库
    let t_db = std::time::Instant::now();
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
    let db_ms = t_db.elapsed().as_millis();

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

    // 9. 更新最后 pHash 和文本 hash
    *LAST_PHASH.lock().await = Some(phash_str.clone());
    *LAST_TEXT_HASH.lock().await = current_text_hash;

    // 10. 处理文本：优先使用 UIA 结果，否则触发 OCR
    if let Some(ref text) = uia_text {
        // UIA 成功：直接更新数据库，跳过 OCR
        if let Err(e) = db::update_activity_ocr(activity_id, text).await {
            tracing::error!("更新 UIA 文本失败: {}", e);
        } else {
            tracing::info!("使用 UIA 文本更新数据库成功，跳过 OCR");
            
            // 发送 OCR 更新事件到前端（实际上是 UIA 文本）
            if let Some(app_handle) = APP_HANDLE.lock().await.as_ref() {
                use tauri::Emitter;
                let update_data = serde_json::json!({
                    "id": activity_id,
                    "ocrText": text
                });
                let _ = app_handle.emit("ocr-updated", &update_data);
            }
        }
    } else {
        // UIA 失败：回退到 OCR 流程
        let config = app_config::get_config().await?;
        if config.ocr_enabled {
            tokio::spawn(async move {
                match db::enqueue_ocr_task(activity_id).await {
                    Ok(_) => {
                        crate::ocr_worker::notify_new_task();
                    }
                    Err(e) => {
                        tracing::warn!("OCR 入队失败: {}", e);
                    }
                };
            });
        }
    } // 关闭 else (UIA 失败回退到 OCR) 分支

    tracing::info!(
        "已保存活动记录: {} - {}",
        window_info.process_name,
        window_info.title
    );
    tracing::debug!(
        uia_ms = uia_ms,
        capture_ms = capture_ms,
        write_ms = write_ms,
        db_ms = db_ms,
        total_ms = t_total.elapsed().as_millis(),
        "capture_and_save performance"
    );

    adjust_heartbeat(false);
    Ok(())
}

/// 全景拼接截图：捕获所有显示器并拼接为一张全景图
fn capture_screen() -> Result<DynamicImage> {
    let monitors = xcap::Monitor::all()?;
    if monitors.is_empty() {
        return Err(anyhow::anyhow!("未找到显示器"));
    }
    
    // 如果只有一个显示器，直接返回（优化单屏场景）
    if monitors.len() == 1 {
        let monitor = &monitors[0];
        let image_buffer = monitor.capture_image()?;
        let width = image_buffer.width();
        let height = image_buffer.height();
        let mut raw_pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
        for p in image_buffer.pixels() {
            raw_pixels.extend_from_slice(&p.0);
        }
        let rgba_image = image::RgbaImage::from_raw(width, height, raw_pixels)
            .ok_or_else(|| anyhow::anyhow!("无法创建图像"))?;
        return Ok(DynamicImage::ImageRgba8(rgba_image));
    }
    
    // 多显示器：计算画布边界（bounding box）
    let min_x = monitors.iter().map(|m| m.x()).min().unwrap_or(0);
    let min_y = monitors.iter().map(|m| m.y()).min().unwrap_or(0);
    let max_x = monitors.iter().map(|m| m.x() + m.width() as i32).max().unwrap_or(1920);
    let max_y = monitors.iter().map(|m| m.y() + m.height() as i32).max().unwrap_or(1080);
    
    let canvas_width = (max_x - min_x) as u32;
    let canvas_height = (max_y - min_y) as u32;
    
    tracing::info!(
        "全景拼接：检测到 {} 个显示器，画布尺寸 {}x{}",
        monitors.len(),
        canvas_width,
        canvas_height
    );
    
    // 使用 std::thread 并行采集所有显示器截图
    // （xcap::Monitor 不支持 Send，所以我们在每个线程中重新获取 monitors）
    let num_monitors = monitors.len();
    drop(monitors); // 释放原始 monitors 以允许线程内重新获取
    
    let captures: Vec<_> = (0..num_monitors)
        .map(|idx| {
            let min_x = min_x;
            let min_y = min_y;
            std::thread::spawn(move || {
                // 每个线程独立获取显示器列表并截取指定索引的显示器
                let monitors = match xcap::Monitor::all() {
                    Ok(m) => m,
                    Err(e) => return Err(anyhow::anyhow!("获取显示器列表失败: {:?}", e)),
                };
                if idx >= monitors.len() {
                    return Err(anyhow::anyhow!("显示器索引越界"));
                }
                let monitor = &monitors[idx];
                let x_offset = (monitor.x() - min_x) as u32;
                let y_offset = (monitor.y() - min_y) as u32;
                
                let image_buffer = monitor.capture_image()?;
                let width = image_buffer.width();
                let height = image_buffer.height();
                let mut raw_pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);
                for p in image_buffer.pixels() {
                    raw_pixels.extend_from_slice(&p.0);
                }
                
                let rgba_image = image::RgbaImage::from_raw(width, height, raw_pixels)
                    .ok_or_else(|| anyhow::anyhow!("无法创建图像"))?;
                
                Ok((rgba_image, x_offset, y_offset))
            })
        })
        .collect();
    
    // 等待所有线程完成并收集结果
    let results: Vec<_> = captures
        .into_iter()
        .map(|handle| handle.join().unwrap_or_else(|_| Err(anyhow::anyhow!("线程 panic"))))
        .collect();
    
    // 创建空白画布（RGBA，黑色背景）
    let mut panorama = image::RgbaImage::new(canvas_width, canvas_height);
    
    // 将每个显示器的截图贴到画布上
    for result in results {
        match result {
            Ok((monitor_img, x_offset, y_offset)) => {
                image::imageops::overlay(&mut panorama, &monitor_img, x_offset as i64, y_offset as i64);
            }
            Err(e) => {
                tracing::warn!("显示器截图失败: {:?}，跳过该显示器", e);
            }
        }
    }
    
    Ok(DynamicImage::ImageRgba8(panorama))
}

/// 计算感知哈希并返回 u64 值（便于计算 Hamming 距离）
fn calculate_phash_u64(image: &DynamicImage) -> Result<u64> {
    // 使用差分哈希 (dHash) 算法:
    // 1. 缩放为 9x8（宽度多 1 像素用于比较）
    // 2. 转为灰度
    // 3. 比较每行相邻像素，生成 64 位哈希
    let gray = image::imageops::grayscale(image);
    let resized = image::imageops::resize(&gray, 9, 8, image::imageops::FilterType::Lanczos3);

    let mut hash: u64 = 0;
    let mut bit_index = 0;

    for y in 0..8 {
        for x in 0..8 {
            let left = resized.get_pixel(x, y).0[0];
            let right = resized.get_pixel(x + 1, y).0[0];
            if left > right {
                hash |= 1 << bit_index;
            }
            bit_index += 1;
        }
    }

    Ok(hash)
}

/// 计算两个哈希值之间的 Hamming 距离
/// Hamming 距离 = 不同位的数量
fn hamming_distance(hash1: u64, hash2: u64) -> u32 {
    (hash1 ^ hash2).count_ones()
}

/// 从十六进制字符串解析 u64 哈希值
fn parse_phash(phash_str: &str) -> Option<u64> {
    u64::from_str_radix(phash_str, 16).ok()
}

/// 去重阈值：Hamming 距离 <= 此值认为是相似帧
/// 0 = 完全相同（最严格，与原逻辑一致）
/// 5 = 允许少量差异（推荐值，可检测鼠标移动、小动画等）
/// 10 = 允许较多差异（更激进的去重）
const DEDUP_HAMMING_THRESHOLD: u32 = 5;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hamming_distance_identical() {
        // 完全相同的哈希，距离为 0
        assert_eq!(hamming_distance(0x0000000000000000, 0x0000000000000000), 0);
        assert_eq!(hamming_distance(0xFFFFFFFFFFFFFFFF, 0xFFFFFFFFFFFFFFFF), 0);
        assert_eq!(hamming_distance(0x123456789ABCDEF0, 0x123456789ABCDEF0), 0);
    }

    #[test]
    fn test_hamming_distance_one_bit_diff() {
        // 只有 1 位不同
        assert_eq!(hamming_distance(0x0000000000000000, 0x0000000000000001), 1);
        assert_eq!(hamming_distance(0x8000000000000000, 0x0000000000000000), 1);
    }

    #[test]
    fn test_hamming_distance_multiple_bits() {
        // 多位不同
        assert_eq!(hamming_distance(0x0000000000000000, 0x0000000000000003), 2); // 2 bits
        assert_eq!(hamming_distance(0x0000000000000000, 0x000000000000000F), 4); // 4 bits
        assert_eq!(hamming_distance(0x0000000000000000, 0x00000000000000FF), 8); // 8 bits
    }

    #[test]
    fn test_hamming_distance_opposite() {
        // 完全相反，64 位都不同
        assert_eq!(hamming_distance(0x0000000000000000, 0xFFFFFFFFFFFFFFFF), 64);
    }

    #[test]
    fn test_parse_phash_valid() {
        assert_eq!(parse_phash("0000000000000000"), Some(0));
        assert_eq!(parse_phash("ffffffffffffffff"), Some(0xFFFFFFFFFFFFFFFF));
        assert_eq!(parse_phash("123456789abcdef0"), Some(0x123456789ABCDEF0));
    }

    #[test]
    fn test_parse_phash_invalid() {
        assert_eq!(parse_phash("not_a_hash"), None);
        assert_eq!(parse_phash(""), None);
        assert_eq!(parse_phash("gggg"), None); // invalid hex
    }

    #[test]
    fn test_dedup_threshold() {
        // 测试阈值逻辑
        let hash1: u64 = 0x0000000000000000;
        let hash2: u64 = 0x000000000000001F; // 5 bits different

        let distance = hamming_distance(hash1, hash2);
        assert_eq!(distance, 5);
        
        // 距离 = 阈值，应该认为是相似的
        assert!(distance <= DEDUP_HAMMING_THRESHOLD);

        // 6 bits different - should exceed threshold
        let hash3: u64 = 0x000000000000003F; // 6 bits different
        let distance3 = hamming_distance(hash1, hash3);
        assert_eq!(distance3, 6);
        assert!(distance3 > DEDUP_HAMMING_THRESHOLD);
    }

    #[test]
    fn test_calculate_phash_deterministic() {
        // 创建一个简单的测试图像
        let img = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_fn(100, 100, |x, y| {
                if (x + y) % 2 == 0 {
                    image::Rgba([255, 255, 255, 255])
                } else {
                    image::Rgba([0, 0, 0, 255])
                }
            })
        );

        // 计算两次哈希，应该相同
        let hash1 = calculate_phash_u64(&img).unwrap();
        let hash2 = calculate_phash_u64(&img).unwrap();
        assert_eq!(hash1, hash2, "pHash should be deterministic");
    }

    #[test]
    fn test_calculate_phash_different_images() {
        // 纯白图像
        let white = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_fn(100, 100, |_, _| image::Rgba([255, 255, 255, 255]))
        );
        
        // 纯黑图像
        let black = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_fn(100, 100, |_, _| image::Rgba([0, 0, 0, 255]))
        );

        let hash_white = calculate_phash_u64(&white).unwrap();
        let hash_black = calculate_phash_u64(&black).unwrap();
        
        // 纯色图像的 dHash 都是 0（因为相邻像素相同）
        // 但它们应该相等（都是纯色）
        assert_eq!(hash_white, hash_black, "Solid color images should have same dHash (all 0)");
    }

    #[test]
    fn test_calculate_phash_gradient() {
        // 水平渐变（从左到右变亮）
        let gradient_lr = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_fn(100, 100, |x, _| {
                let v = (x * 255 / 99) as u8;
                image::Rgba([v, v, v, 255])
            })
        );

        // 水平渐变（从右到左变亮）- 反向
        let gradient_rl = image::DynamicImage::ImageRgba8(
            image::RgbaImage::from_fn(100, 100, |x, _| {
                let v = ((99 - x) * 255 / 99) as u8;
                image::Rgba([v, v, v, 255])
            })
        );

        let hash_lr = calculate_phash_u64(&gradient_lr).unwrap();
        let hash_rl = calculate_phash_u64(&gradient_rl).unwrap();

        // 反向渐变应该有很大的 Hamming 距离
        let distance = hamming_distance(hash_lr, hash_rl);
        assert!(distance > 32, "Opposite gradients should have high Hamming distance: {}", distance);
    }

    #[test]
    #[ignore]
    fn stress_phash_and_webp() {
        let img = image::DynamicImage::ImageRgba8(image::RgbaImage::from_fn(1920, 1080, |x, y| {
            let v = ((x ^ y) & 0xFF) as u8;
            image::Rgba([v, v, v, 255])
        }));

        let iterations = 10_u32;
        let mut phash_total_us: u128 = 0;
        let mut webp_total_us: u128 = 0;

        for _ in 0..iterations {
            let t_phash = std::time::Instant::now();
            let hash = calculate_phash_u64(&img).unwrap();
            phash_total_us += t_phash.elapsed().as_micros();

            let rgba = img.to_rgba8();
            let t_webp = std::time::Instant::now();
            let encoder = webp::Encoder::from_rgba(&rgba, rgba.width(), rgba.height());
            let webp_mem = encoder.encode(80.0);
            webp_total_us += t_webp.elapsed().as_micros();

            std::hint::black_box(hash);
            std::hint::black_box(webp_mem.len());
        }

        println!(
            "stress_phash_and_webp: iterations={}, phash_avg_ms={:.2}, webp_avg_ms={:.2}",
            iterations,
            phash_total_us as f64 / iterations as f64 / 1000.0,
            webp_total_us as f64 / iterations as f64 / 1000.0
        );
    }
}
