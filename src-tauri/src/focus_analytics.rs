use crate::app_config;
use crate::db;
use crate::recorder;
use crate::window_info;
use device_query::{DeviceQuery, DeviceState, Keycode};
use std::panic::{catch_unwind, AssertUnwindSafe};
use tokio::time::{interval, Duration};

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

async fn run() {
    let device_state = DeviceState::new();
    let mut ticker = interval(Duration::from_secs(1));

    let mut prev_keys: Vec<Keycode> = Vec::new();
    let mut prev_mouse: Option<(i32, i32)> = None;
    let mut last_process_path: Option<String> = None;

    let mut key_press_count: u32 = 0;
    let mut mouse_move_distance: f64 = 0.0;
    let mut window_switch_count: u32 = 0;
    let mut seconds_in_bucket: u32 = 0;

    while recorder::is_recording() {
        ticker.tick().await;

        if !recorder::is_recording() {
            break;
        }

        let enabled = app_config::get_config()
            .await
            .map(|c| c.enable_focus_analytics)
            .unwrap_or(false);
        if !enabled {
            break;
        }

        let tick_result = catch_unwind(AssertUnwindSafe(|| {
            let keys_now = device_state.get_keys();
            let new_presses = keys_now
                .iter()
                .filter(|k| !prev_keys.contains(k))
                .count() as u32;
            prev_keys = keys_now;
            key_press_count = key_press_count.saturating_add(new_presses);

            let mouse = device_state.get_mouse();
            let (x, y) = mouse.coords;
            if let Some((px, py)) = prev_mouse {
                let dx = (x - px) as f64;
                let dy = (y - py) as f64;
                mouse_move_distance += (dx * dx + dy * dy).sqrt();
            }
            prev_mouse = Some((x, y));

            if let Ok(info) = window_info::get_foreground_window_info() {
                match &last_process_path {
                    Some(last) if last != &info.process_path => {
                        window_switch_count = window_switch_count.saturating_add(1);
                        last_process_path = Some(info.process_path);
                    }
                    None => {
                        last_process_path = Some(info.process_path);
                    }
                    _ => {}
                }
            }
        }));

        if tick_result.is_err() {
            tracing::warn!("Focus analytics tick panicked and was recovered");
        }

        seconds_in_bucket += 1;
        if seconds_in_bucket < 60 {
            continue;
        }

        let apm = calculate_apm(key_press_count, mouse_move_distance);
        let switches = window_switch_count.min(i32::MAX as u32) as i32;
        let focus_score = calculate_focus_score(apm, switches);
        let timestamp = chrono::Utc::now().timestamp();

        tokio::spawn(async move {
            if let Err(e) = db::insert_focus_metric(timestamp, apm, switches, focus_score).await {
                tracing::warn!("Failed to insert focus metric: {}", e);
            }
        });

        key_press_count = 0;
        mouse_move_distance = 0.0;
        window_switch_count = 0;
        seconds_in_bucket = 0;
    }
}

fn calculate_apm(key_press_count: u32, mouse_move_distance: f64) -> i32 {
    let mouse_actions = (mouse_move_distance / 500.0).round().max(0.0) as u32;
    let total = key_press_count.saturating_add(mouse_actions);
    total.min(i32::MAX as u32) as i32
}

pub fn calculate_focus_score(apm: i32, window_switch_count: i32) -> f64 {
    let apm = apm.max(0) as f64;
    let window_switch_count = window_switch_count.max(0) as f64;

    let score = apm * 0.8 - window_switch_count * 5.0;
    score.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::calculate_focus_score;

    #[test]
    fn focus_score_clamps_to_range() {
        let low = calculate_focus_score(0, 999);
        assert_eq!(low, 0.0);

        let high = calculate_focus_score(999, 0);
        assert_eq!(high, 100.0);
    }

    #[test]
    fn focus_score_decreases_with_switches() {
        let a = calculate_focus_score(60, 0);
        let b = calculate_focus_score(60, 5);
        assert!(a > b);
    }
}
