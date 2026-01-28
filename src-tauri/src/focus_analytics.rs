//! Focus analytics module - Tauri wrapper for memflow-core focus_analytics
//!
//! Re-exports pure scoring functions from memflow_core::focus_analytics
//! and provides Tauri-specific runtime loop and spawn logic.

// Re-export pure scoring functions from memflow-core
pub use memflow_core::focus_analytics::*;

use crate::app_config;
use crate::db;
use crate::recorder;
use crate::window_info;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::panic::{catch_unwind, AssertUnwindSafe};
use tokio::time::{interval, Duration};

/// Spawn focus analytics if enabled in config
pub fn spawn_if_enabled() {
    tokio::spawn(async {
        let enabled = app_config::get_config()
            .await
            .map(|c| c.enable_focus_analytics)
            .unwrap_or(false);
        if !enabled {
            return;
        }

        run().await;
    });
}

/// Main focus analytics loop - Tauri-specific
async fn run() {
    let mut interval = interval(Duration::from_secs(60));
    let mut last_key_count: usize = 0;
    let mut last_mouse_pos: (i32, i32) = (0, 0);
    let mut total_key_presses: u32 = 0;
    let mut mouse_move_distance: f64 = 0.0;
    let mut switch_context = SwitchContext::new();

    let device_state = DeviceState::new();

    loop {
        interval.tick().await;

        if !recorder::is_recording() {
            tracing::debug!("录制已停止，专注度分析暂停");
            break;
        }

        let enabled = app_config::get_config()
            .await
            .map(|c| c.enable_focus_analytics)
            .unwrap_or(false);
        if !enabled {
            break;
        }

        // Safely capture keyboard metrics
        let (key_diff, new_key_count) = {
            let keys_result = catch_unwind(AssertUnwindSafe(|| device_state.get_keys()));
            match keys_result {
                Ok(keys) => {
                    let current_count = keys.iter().filter(|k| **k != Keycode::Key0).count();
                    let diff = if current_count > last_key_count {
                        (current_count - last_key_count) as u32
                    } else {
                        0
                    };
                    (diff, current_count)
                }
                Err(_) => {
                    tracing::warn!("获取键盘状态时出错");
                    (0, last_key_count)
                }
            }
        };

        total_key_presses = total_key_presses.saturating_add(key_diff);
        last_key_count = new_key_count;

        // Safely capture mouse metrics
        let current_mouse = {
            let mouse_result = catch_unwind(AssertUnwindSafe(|| device_state.get_mouse()));
            match mouse_result {
                Ok(mouse) => mouse.coords,
                Err(_) => {
                    tracing::warn!("获取鼠标状态时出错");
                    last_mouse_pos
                }
            }
        };

        let dx = (current_mouse.0 - last_mouse_pos.0) as f64;
        let dy = (current_mouse.1 - last_mouse_pos.1) as f64;
        mouse_move_distance += (dx * dx + dy * dy).sqrt();
        last_mouse_pos = current_mouse;

        // Get current window info
        if let Ok(info) = window_info::get_foreground_window_info() {
            switch_context.record_switch(&info.process_name);
        }

        // Calculate metrics
        let apm = calculate_apm(total_key_presses, mouse_move_distance);
        let weighted_switches = switch_context.weighted_switches();
        let focus_score = calculate_focus_score_v2(apm, weighted_switches);

        tracing::debug!(
            "专注度分析 (v2): APM={}, weighted_switches={:.2}, score={:.1}",
            apm,
            weighted_switches,
            focus_score
        );

        let timestamp = chrono::Utc::now().timestamp();
        if let Err(e) = db::insert_focus_metric(
            timestamp,
            apm,
            switch_context.total_switches() as i32,
            focus_score,
        )
        .await
        {
            tracing::warn!("更新专注度数据失败: {}", e);
        }

        // Reset counters for next interval
        total_key_presses = 0;
        mouse_move_distance = 0.0;
        switch_context.reset();
    }
}
