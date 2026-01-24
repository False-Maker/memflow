use crate::app_config;
use crate::db;
use crate::recorder;
use crate::window_info;
use device_query::{DeviceQuery, DeviceState, Keycode};
use once_cell::sync::Lazy;
use std::collections::HashSet;
use std::panic::{catch_unwind, AssertUnwindSafe};
use tokio::time::{interval, Duration};

/// 生产力应用白名单（在这些应用间切换不会大幅降低专注度分数）
/// 包含应用名称关键词（不区分大小写匹配）
static PRODUCTIVITY_APPS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let apps = [
        // IDE 和代码编辑器
        "code", "vscode", "visual studio", "cursor", "idea", "intellij", 
        "pycharm", "webstorm", "goland", "rider", "clion", "datagrip",
        "sublime", "atom", "notepad++", "vim", "nvim", "emacs", "neovim",
        "android studio", "xcode", "eclipse",
        // 文档和笔记应用
        "word", "excel", "powerpoint", "onenote", "notion", "obsidian",
        "typora", "marktext", "joplin", "evernote", "roam",
        "acrobat", "pdf", "foxit", "sumatra",
        // 终端和命令行
        "terminal", "iterm", "powershell", "cmd", "wezterm", "alacritty",
        "windows terminal", "hyper", "kitty", "konsole",
        // 开发工具
        "postman", "insomnia", "docker", "datagrip", "dbeaver",
        "sourcetree", "gitkraken", "github", "gitlab",
        // 浏览器（开发/研究用途）
        "chrome", "firefox", "edge", "safari", "brave", "arc",
        // 设计工具
        "figma", "sketch", "photoshop", "illustrator", "affinity",
        // 通讯工具（工作相关）
        "slack", "teams", "discord", "zoom", "meeting",
    ];
    apps.into_iter().collect()
});

/// 判断应用是否为生产力工具
fn is_productivity_app(app_name: &str) -> bool {
    let lower = app_name.to_lowercase();
    PRODUCTIVITY_APPS.iter().any(|&keyword| lower.contains(keyword))
}

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

/// 窗口切换上下文
struct SwitchContext {
    last_app_name: Option<String>,
    last_was_productive: bool,
    /// 生产力应用间的切换（惩罚较低）
    productive_switches: u32,
    /// 切换到非生产力应用（惩罚较高）
    distraction_switches: u32,
}

impl SwitchContext {
    fn new() -> Self {
        Self {
            last_app_name: None,
            last_was_productive: false,
            productive_switches: 0,
            distraction_switches: 0,
        }
    }
    
    fn record_switch(&mut self, new_app_name: &str) {
        let is_productive = is_productivity_app(new_app_name);
        
        if let Some(ref last) = self.last_app_name {
            if last != new_app_name {
                // 发生了窗口切换
                if self.last_was_productive && is_productive {
                    // 从生产力应用切换到另一个生产力应用（低惩罚）
                    self.productive_switches = self.productive_switches.saturating_add(1);
                } else if !is_productive {
                    // 切换到非生产力应用（高惩罚）
                    self.distraction_switches = self.distraction_switches.saturating_add(1);
                } else {
                    // 从非生产力应用切换到生产力应用（低惩罚）
                    self.productive_switches = self.productive_switches.saturating_add(1);
                }
            }
        }
        
        self.last_app_name = Some(new_app_name.to_string());
        self.last_was_productive = is_productive;
    }
    
    fn reset(&mut self) {
        self.productive_switches = 0;
        self.distraction_switches = 0;
        // 保留 last_app_name 和 last_was_productive 以便跨桶继续追踪
    }
    
    /// 计算加权切换次数（生产力切换权重低，分心切换权重高）
    fn weighted_switches(&self) -> f64 {
        (self.productive_switches as f64 * 0.3) + (self.distraction_switches as f64 * 1.0)
    }
    
    /// 获取总切换次数（用于向后兼容）
    fn total_switches(&self) -> u32 {
        self.productive_switches.saturating_add(self.distraction_switches)
    }
}

async fn run() {
    let device_state = DeviceState::new();
    let mut ticker = interval(Duration::from_secs(1));

    let mut prev_keys: Vec<Keycode> = Vec::new();
    let mut prev_mouse: Option<(i32, i32)> = None;
    let mut switch_ctx = SwitchContext::new();

    let mut key_press_count: u32 = 0;
    let mut mouse_move_distance: f64 = 0.0;
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

            // 使用上下文感知的窗口切换追踪
            if let Ok(info) = window_info::get_foreground_window_info() {
                switch_ctx.record_switch(&info.process_name);
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
        let weighted_switches = switch_ctx.weighted_switches();
        let total_switches = switch_ctx.total_switches().min(i32::MAX as u32) as i32;
        let focus_score = calculate_focus_score_v2(apm, weighted_switches);
        let timestamp = chrono::Utc::now().timestamp();

        tracing::debug!(
            "Focus metrics: APM={}, productive_switches={}, distraction_switches={}, weighted={:.1}, score={:.1}",
            apm,
            switch_ctx.productive_switches,
            switch_ctx.distraction_switches,
            weighted_switches,
            focus_score
        );

        tokio::spawn(async move {
            if let Err(e) = db::insert_focus_metric(timestamp, apm, total_switches, focus_score).await {
                tracing::warn!("Failed to insert focus metric: {}", e);
            }
        });

        key_press_count = 0;
        mouse_move_distance = 0.0;
        switch_ctx.reset();
        seconds_in_bucket = 0;
    }
}

fn calculate_apm(key_press_count: u32, mouse_move_distance: f64) -> i32 {
    let mouse_actions = (mouse_move_distance / 500.0).round().max(0.0) as u32;
    let total = key_press_count.saturating_add(mouse_actions);
    total.min(i32::MAX as u32) as i32
}

/// 旧版评分算法（保留向后兼容）
pub fn calculate_focus_score(apm: i32, window_switch_count: i32) -> f64 {
    let apm = apm.max(0) as f64;
    let window_switch_count = window_switch_count.max(0) as f64;

    let score = apm * 0.8 - window_switch_count * 5.0;
    score.clamp(0.0, 100.0)
}

/// 新版上下文感知评分算法
/// 公式: score = apm_score * 0.6 + stability_score * 0.4
/// - apm_score: 基于每分钟操作数的活跃度评分
/// - stability_score: 基于窗口切换频率的稳定性评分（考虑切换类型）
pub fn calculate_focus_score_v2(apm: i32, weighted_switches: f64) -> f64 {
    let apm = apm.max(0) as f64;
    
    // APM 评分：将 APM 映射到 0-100 分
    // 假设 60 APM 为中等水平 (50分)，120+ APM 为高效 (100分)
    let apm_score = if apm >= 120.0 {
        100.0
    } else if apm >= 60.0 {
        50.0 + (apm - 60.0) * (50.0 / 60.0)
    } else {
        apm * (50.0 / 60.0)
    };
    
    // 稳定性评分：切换次数越少越好
    // 0 次切换 = 100 分
    // 每次加权切换减少一定分数
    let switch_penalty = weighted_switches * 8.0; // 每次加权切换扣 8 分
    let stability_score = (100.0 - switch_penalty).max(0.0);
    
    // 综合评分：活跃度 60% + 稳定性 40%
    let final_score = apm_score * 0.6 + stability_score * 0.4;
    
    final_score.clamp(0.0, 100.0)
}

#[cfg(test)]
mod tests {
    use super::*;

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
    
    #[test]
    fn focus_score_v2_clamps_to_range() {
        let low = calculate_focus_score_v2(0, 100.0);
        assert_eq!(low, 0.0);

        let high = calculate_focus_score_v2(200, 0.0);
        assert_eq!(high, 100.0);
    }
    
    #[test]
    fn focus_score_v2_weighted_switches() {
        // 无切换
        let no_switch = calculate_focus_score_v2(60, 0.0);
        // 生产力切换（权重 0.3）
        let productive = calculate_focus_score_v2(60, 0.3);
        // 分心切换（权重 1.0）
        let distraction = calculate_focus_score_v2(60, 1.0);
        
        assert!(no_switch > productive);
        assert!(productive > distraction);
    }
    
    #[test]
    fn productivity_app_detection() {
        // IDE 应该被识别为生产力应用
        assert!(is_productivity_app("Visual Studio Code"));
        assert!(is_productivity_app("cursor.exe"));
        assert!(is_productivity_app("IntelliJ IDEA"));
        
        // 游戏/娱乐应用不应该被识别
        assert!(!is_productivity_app("Steam"));
        assert!(!is_productivity_app("Spotify"));
        assert!(!is_productivity_app("VLC"));
    }
    
    #[test]
    fn switch_context_tracking() {
        let mut ctx = SwitchContext::new();
        
        // 从 Code 切换到 Chrome（都是生产力应用）
        ctx.record_switch("Code");
        ctx.record_switch("Chrome");
        assert_eq!(ctx.productive_switches, 1);
        assert_eq!(ctx.distraction_switches, 0);
        
        // 从 Chrome 切换到游戏（分心）
        ctx.record_switch("Steam");
        assert_eq!(ctx.distraction_switches, 1);
        
        // 加权切换应该反映不同类型
        let weighted = ctx.weighted_switches();
        assert!(weighted > 0.0);
        assert!(weighted < 2.0); // 1 * 0.3 + 1 * 1.0 = 1.3
    }
}
