//! Focus analytics core logic - pure scoring algorithms
//!
//! This module contains the pure, Tauri-independent scoring algorithms
//! for focus analytics. The runtime loop and spawning logic remain in src-tauri.

use once_cell::sync::Lazy;
use std::collections::HashSet;

/// Productivity apps whitelist (switches between these apps have lower penalty)
/// Contains app name keywords (case-insensitive matching)
static PRODUCTIVITY_APPS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let apps = [
        // IDE and code editors
        "code", "vscode", "visual studio", "cursor", "idea", "intellij", 
        "pycharm", "webstorm", "goland", "rider", "clion", "datagrip",
        "sublime", "atom", "notepad++", "vim", "nvim", "emacs", "neovim",
        "android studio", "xcode", "eclipse",
        // Document and note apps
        "word", "excel", "powerpoint", "onenote", "notion", "obsidian",
        "typora", "marktext", "joplin", "evernote", "roam",
        "acrobat", "pdf", "foxit", "sumatra",
        // Terminal and CLI
        "terminal", "iterm", "powershell", "cmd", "wezterm", "alacritty",
        "windows terminal", "hyper", "kitty", "konsole",
        // Dev tools
        "postman", "insomnia", "docker", "datagrip", "dbeaver",
        "sourcetree", "gitkraken", "github", "gitlab",
        // Browsers (dev/research use)
        "chrome", "firefox", "edge", "safari", "brave", "arc",
        // Design tools
        "figma", "sketch", "photoshop", "illustrator", "affinity",
        // Communication tools (work-related)
        "slack", "teams", "discord", "zoom", "meeting",
    ];
    apps.into_iter().collect()
});

/// Check if an app is a productivity tool
pub fn is_productivity_app(app_name: &str) -> bool {
    let lower = app_name.to_lowercase();
    PRODUCTIVITY_APPS.iter().any(|&keyword| lower.contains(keyword))
}

/// Window switch context for tracking focus patterns
#[derive(Debug, Clone)]
pub struct SwitchContext {
    pub last_app_name: Option<String>,
    pub last_was_productive: bool,
    /// Switches between productivity apps (low penalty)
    pub productive_switches: u32,
    /// Switches to non-productivity apps (high penalty)
    pub distraction_switches: u32,
}

impl Default for SwitchContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SwitchContext {
    pub fn new() -> Self {
        Self {
            last_app_name: None,
            last_was_productive: false,
            productive_switches: 0,
            distraction_switches: 0,
        }
    }
    
    pub fn record_switch(&mut self, new_app_name: &str) {
        let is_productive = is_productivity_app(new_app_name);
        
        if let Some(ref last) = self.last_app_name {
            if last != new_app_name {
                if self.last_was_productive && is_productive {
                    self.productive_switches = self.productive_switches.saturating_add(1);
                } else if !is_productive {
                    self.distraction_switches = self.distraction_switches.saturating_add(1);
                } else {
                    self.productive_switches = self.productive_switches.saturating_add(1);
                }
            }
        }
        
        self.last_app_name = Some(new_app_name.to_string());
        self.last_was_productive = is_productive;
    }
    
    pub fn reset(&mut self) {
        self.productive_switches = 0;
        self.distraction_switches = 0;
    }
    
    /// Calculate weighted switch count
    pub fn weighted_switches(&self) -> f64 {
        (self.productive_switches as f64 * 0.3) + (self.distraction_switches as f64 * 1.0)
    }
    
    /// Get total switches (for backward compatibility)
    pub fn total_switches(&self) -> u32 {
        self.productive_switches.saturating_add(self.distraction_switches)
    }
}

/// Calculate APM (actions per minute) from key presses and mouse movement
pub fn calculate_apm(key_press_count: u32, mouse_move_distance: f64) -> i32 {
    let mouse_actions = (mouse_move_distance / 500.0).round().max(0.0) as u32;
    let total = key_press_count.saturating_add(mouse_actions);
    total.min(i32::MAX as u32) as i32
}

/// Legacy scoring algorithm (kept for backward compatibility)
pub fn calculate_focus_score(apm: i32, window_switch_count: i32) -> f64 {
    let apm = apm.max(0) as f64;
    let window_switch_count = window_switch_count.max(0) as f64;

    let score = apm * 0.8 - window_switch_count * 5.0;
    score.clamp(0.0, 100.0)
}

/// Context-aware scoring algorithm (v2)
/// Formula: score = apm_score * 0.6 + stability_score * 0.4
pub fn calculate_focus_score_v2(apm: i32, weighted_switches: f64) -> f64 {
    let apm = apm.max(0) as f64;
    
    let apm_score = if apm >= 120.0 {
        100.0
    } else if apm >= 60.0 {
        50.0 + (apm - 60.0) * (50.0 / 60.0)
    } else {
        apm * (50.0 / 60.0)
    };
    
    let switch_penalty = weighted_switches * 8.0;
    let stability_score = (100.0 - switch_penalty).max(0.0);
    
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
        let no_switch = calculate_focus_score_v2(60, 0.0);
        let productive = calculate_focus_score_v2(60, 0.3);
        let distraction = calculate_focus_score_v2(60, 1.0);
        
        assert!(no_switch > productive);
        assert!(productive > distraction);
    }
    
    #[test]
    fn productivity_app_detection() {
        assert!(is_productivity_app("Visual Studio Code"));
        assert!(is_productivity_app("cursor.exe"));
        assert!(is_productivity_app("IntelliJ IDEA"));
        
        assert!(!is_productivity_app("Steam"));
        assert!(!is_productivity_app("Spotify"));
        assert!(!is_productivity_app("VLC"));
    }
    
    #[test]
    fn switch_context_tracking() {
        let mut ctx = SwitchContext::new();
        
        ctx.record_switch("Code");
        ctx.record_switch("Chrome");
        assert_eq!(ctx.productive_switches, 1);
        assert_eq!(ctx.distraction_switches, 0);
        
        ctx.record_switch("Steam");
        assert_eq!(ctx.distraction_switches, 1);
        
        let weighted = ctx.weighted_switches();
        assert!(weighted > 0.0);
        assert!(weighted < 2.0);
    }
}
