use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub disk_usage_mb: f64,
    pub screenshot_count: i64,
    pub activities_count: i64,
}

pub struct PerformanceMonitor;

impl PerformanceMonitor {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_metrics(&self) -> Result<PerformanceMetrics> {
        // 内存使用（简化实现）
        let memory_usage_mb = Self::get_memory_usage().await;

        // CPU 使用（简化实现）
        let cpu_usage_percent = Self::get_cpu_usage().await;

        // 磁盘使用
        let disk_usage_mb = Self::get_disk_usage().await?;

        // 活动统计
        let (screenshot_count, activities_count) = Self::get_activity_stats().await?;

        Ok(PerformanceMetrics {
            memory_usage_mb,
            cpu_usage_percent,
            disk_usage_mb,
            screenshot_count,
            activities_count,
        })
    }

    async fn get_memory_usage() -> f64 {
        #[cfg(windows)]
        {
            use windows::Win32::System::ProcessStatus::*;
            use windows::Win32::System::Threading::GetCurrentProcess;

            unsafe {
                let process = GetCurrentProcess();
                let mut pmc = PROCESS_MEMORY_COUNTERS_EX::default();
                let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;

                if GetProcessMemoryInfo(process, &mut pmc as *mut _ as *mut _, size).is_ok() {
                    pmc.WorkingSetSize as f64 / 1024.0 / 1024.0
                } else {
                    0.0
                }
            }
        }

        #[cfg(not(windows))]
        {
            0.0
        }
    }

    async fn get_cpu_usage() -> f64 {
        // 简化的 CPU 使用率计算
        // 实际应该使用系统 API 获取真实的 CPU 使用率
        0.0
    }

    async fn get_disk_usage() -> Result<f64> {
        use crate::db;

        // 获取截图目录
        let screenshots_dir = db::get_screenshots_dir()
            .await
            .ok_or_else(|| anyhow::anyhow!("截图目录未初始化"))?;

        let mut total_size: u64 = 0;

        if screenshots_dir.exists() {
            for entry in std::fs::read_dir(&screenshots_dir)? {
                let entry = entry?;
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }

        Ok(total_size as f64 / 1024.0 / 1024.0)
    }

    async fn get_activity_stats() -> Result<(i64, i64)> {
        use crate::db;

        let pool = db::get_pool().await?;

        let screenshot_count: i64 =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(DISTINCT image_path) FROM activity_logs")
                .fetch_one(&pool)
                .await?;

        let activities_count: i64 =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM activity_logs")
                .fetch_one(&pool)
                .await?;

        Ok((screenshot_count, activities_count))
    }

    /// 触发垃圾回收
    pub async fn trigger_gc(&self) -> Result<()> {
        use crate::db;

        // 1. 清理过期的活动记录
        let config = crate::app_config::get_config().await?;
        let retention_timestamp =
            chrono::Utc::now().timestamp() - (config.retention_days as i64 * 86400);

        let pool = db::get_pool().await?;

        // 删除过期记录
        sqlx::query("DELETE FROM activity_logs WHERE timestamp < ?")
            .bind(retention_timestamp)
            .execute(&pool)
            .await?;

        // 2. 清理孤立的截图文件
        let screenshots_dir = db::get_screenshots_dir()
            .await
            .ok_or_else(|| anyhow::anyhow!("截图目录未初始化"))?;

        if screenshots_dir.exists() {
            let valid_files: std::collections::HashSet<String> =
                sqlx::query_scalar::<_, String>("SELECT DISTINCT image_path FROM activity_logs")
                    .fetch_all(&pool)
                    .await?
                    .into_iter()
                    .collect();

            for entry in std::fs::read_dir(&screenshots_dir)? {
                let entry = entry?;
                let filename = entry.file_name().to_string_lossy().to_string();

                if !valid_files.contains(&filename) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }

        // 3. 执行 SQLite VACUUM
        sqlx::query("VACUUM").execute(&pool).await?;

        tracing::info!("垃圾回收完成");

        Ok(())
    }
}

impl Default for PerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}
