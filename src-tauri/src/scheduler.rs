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

/// 执行单次清理
async fn run_cleanup() {
    match app_config::get_config().await {
        Ok(config) => {
            let days = config.retention_days;
            tracing::info!("🧹 自动清理调度启动：保留最近 {} 天数据", days);

            match db::cleanup_old_activities(days, false).await {
                Ok(stats) => {
                    tracing::info!(
                        "✅ 自动清理完成: 删除 {} 条活动记录, {} 张截图, 释放 {:.2} MB",
                        stats.deleted_activities,
                        stats.deleted_screenshots,
                        stats.freed_bytes as f64 / 1024.0 / 1024.0
                    );
                }
                Err(e) => {
                    tracing::error!("❌ 自动清理失败: {}", e);
                }
            }
        }
        Err(e) => {
            tracing::warn!("⚠️ 获取配置失败，跳过自动清理: {}", e);
        }
    }
}
