//! 定时任务调度器
//! 
//! 负责在应用启动时及每日定时执行清理逻辑。

use tokio::time::{interval, Duration};
use crate::{app_config, db};

/// 调度间隔（24小时）
const CLEANUP_INTERVAL_SECS: u64 = 24 * 60 * 60;

/// 启动自动清理调度器
/// 
/// 在应用启动时立即执行一次清理，之后每 24 小时执行一次。
pub fn spawn_retention_scheduler() {
    tokio::spawn(async {
        // 1. 启动后立即执行一次（延迟 30 秒，等待数据库初始化完成）
        tokio::time::sleep(Duration::from_secs(30)).await;
        run_cleanup().await;

        // 2. 每 24 小时执行一次
        let mut ticker = interval(Duration::from_secs(CLEANUP_INTERVAL_SECS));
        loop {
            ticker.tick().await;
            run_cleanup().await;
        }
    });
}

/// 执行单次清理（按时间和空间）
async fn run_cleanup() {
    match app_config::get_config().await {
        Ok(config) => {
            let days = config.retention_days;
            let max_storage_mb = config.max_storage_gb as f64 * 1024.0;
            
            tracing::info!("🧹 自动清理调度启动：保留最近 {} 天数据，最大存储 {} GB", days, config.max_storage_gb);

            // 1. 首先按时间清理
            match db::cleanup_old_activities(days, false).await {
                Ok(stats) => {
                    tracing::info!(
                        "✅ 时间清理完成: 删除 {} 条活动记录, {} 张截图, 释放 {:.2} MB",
                        stats.deleted_activities,
                        stats.deleted_screenshots,
                        stats.freed_bytes as f64 / 1024.0 / 1024.0
                    );
                }
                Err(e) => {
                    tracing::error!("❌ 时间清理失败: {}", e);
                }
            }

            // 2. 然后检查磁盘空间，如果超过限制则继续清理
            let current_size = calculate_current_storage_size().await;
            if current_size > max_storage_mb {
                tracing::info!(
                    "📊 当前存储 {:.2} MB 超过限制 {} MB，开始按空间清理...",
                    current_size,
                    max_storage_mb
                );
                
                // 逐步删除更早的数据直到空间足够
                let mut cleanup_days = days;
                while calculate_current_storage_size().await > max_storage_mb && cleanup_days > 1 {
                    cleanup_days = cleanup_days.saturating_sub(7); // 每次减少7天
                    if cleanup_days < 1 {
                        break;
                    }
                    
                    tracing::info!("🧹 按空间清理：保留最近 {} 天数据", cleanup_days);
                    match db::cleanup_old_activities(cleanup_days, false).await {
                        Ok(stats) => {
                            tracing::info!(
                                "✅ 空间清理完成: 删除 {} 条活动记录, {} 张截图, 释放 {:.2} MB",
                                stats.deleted_activities,
                                stats.deleted_screenshots,
                                stats.freed_bytes as f64 / 1024.0 / 1024.0
                            );
                        }
                        Err(e) => {
                            tracing::error!("❌ 空间清理失败: {}", e);
                            break;
                        }
                    }
                }
            }
            
            // 3. 执行数据库 VACUUM 回收空间
            if let Ok(pool) = db::get_pool().await {
                if let Err(e) = sqlx::query("VACUUM").execute(&pool).await {
                    tracing::warn!("⚠️ VACUUM 失败: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ 获取配置失败，跳过自动清理: {}", e);
        }
    }
}

/// 计算当前存储大小（MB）
async fn calculate_current_storage_size() -> f64 {
    let mut total: f64 = 0.0;
    
    // 截图目录大小
    if let Some(screenshots_dir) = db::get_screenshots_dir().await {
        if screenshots_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&screenshots_dir) {
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        total += meta.len() as f64 / 1024.0 / 1024.0;
                    }
                }
            }
        }
    }
    
    // 数据库文件大小
    total += db::get_database_size_mb().await;
    
    total
}
