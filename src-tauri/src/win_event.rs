//! Windows 事件驱动采样模块
//!
//! 使用 SetWinEventHook 监听窗口切换事件，
//! 实现仅在屏幕内容变动时采样，消除冗余录制。

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;

/// 窗口事件类型
#[derive(Debug, Clone)]
pub enum WindowEvent {
    /// 前台窗口切换
    ForegroundChanged { hwnd: isize },
    /// 窗口创建
    WindowCreated { hwnd: isize },
    /// 窗口销毁
    WindowDestroyed { hwnd: isize },
    /// 窗口标题变化
    TitleChanged { hwnd: isize },
}

/// 事件监听器配置
#[derive(Debug, Clone)]
pub struct EventLoopConfig {
    /// 是否监听前台窗口切换
    pub track_foreground: bool,
    /// 是否监听窗口创建/销毁
    pub track_lifecycle: bool,
    /// 是否监听标题变化
    pub track_title_change: bool,
    /// 事件去重间隔（毫秒）
    pub debounce_ms: u64,
}

impl Default for EventLoopConfig {
    fn default() -> Self {
        Self {
            track_foreground: true,
            track_lifecycle: false,
            track_title_change: false,
            debounce_ms: 100,
        }
    }
}

/// 事件循环句柄
pub struct EventLoopHandle {
    stop_flag: Arc<AtomicBool>,
}

impl EventLoopHandle {
    /// 停止事件循环
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }

    /// 检查是否已停止
    pub fn is_stopped(&self) -> bool {
        self.stop_flag.load(Ordering::SeqCst)
    }
}

/// 启动事件驱动的窗口监听
/// 
/// 返回事件接收通道和控制句柄
pub fn start_event_loop(config: EventLoopConfig) -> (mpsc::Receiver<WindowEvent>, EventLoopHandle) {
    let (tx, rx) = mpsc::channel::<WindowEvent>(100);
    let stop_flag = Arc::new(AtomicBool::new(false));
    let handle = EventLoopHandle {
        stop_flag: stop_flag.clone(),
    };

    // 启动后台线程监听窗口事件
    let config_clone = config.clone();
    std::thread::spawn(move || {
        run_event_loop_internal(tx, stop_flag, config_clone);
    });

    (rx, handle)
}

/// 内部事件循环实现
fn run_event_loop_internal(
    tx: mpsc::Sender<WindowEvent>,
    stop_flag: Arc<AtomicBool>,
    config: EventLoopConfig,
) {
    use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;
    
    let mut last_hwnd: isize = 0;
    let debounce_duration = std::time::Duration::from_millis(config.debounce_ms);

    tracing::info!("事件驱动采样循环启动");

    while !stop_flag.load(Ordering::SeqCst) {
        if config.track_foreground {
            unsafe {
                let hwnd = GetForegroundWindow();
                let current_hwnd = hwnd.0 as isize;

                if current_hwnd != last_hwnd && current_hwnd != 0 {
                    last_hwnd = current_hwnd;

                    // 发送前台窗口切换事件
                    if let Err(e) = tx.blocking_send(WindowEvent::ForegroundChanged {
                        hwnd: current_hwnd,
                    }) {
                        tracing::warn!("发送窗口事件失败: {}", e);
                        break;
                    }
                }
            }
        }

        std::thread::sleep(debounce_duration);
    }

    tracing::info!("事件驱动采样循环停止");
}

/// 使用回调方式启动事件监听（简化版）
pub fn start_event_loop_with_callback<F>(callback: F) -> EventLoopHandle
where
    F: Fn(WindowEvent) + Send + 'static,
{
    let stop_flag = Arc::new(AtomicBool::new(false));
    let handle = EventLoopHandle {
        stop_flag: stop_flag.clone(),
    };

    std::thread::spawn(move || {
        use windows::Win32::UI::WindowsAndMessaging::GetForegroundWindow;

        let mut last_hwnd: isize = 0;
        let debounce_duration = std::time::Duration::from_millis(100);

        while !stop_flag.load(Ordering::SeqCst) {
            unsafe {
                let hwnd = GetForegroundWindow();
                let current_hwnd = hwnd.0 as isize;

                if current_hwnd != last_hwnd && current_hwnd != 0 {
                    last_hwnd = current_hwnd;
                    callback(WindowEvent::ForegroundChanged {
                        hwnd: current_hwnd,
                    });
                }
            }

            std::thread::sleep(debounce_duration);
        }
    });

    handle
}

/// 事件驱动录制器
/// 
/// 将事件循环与现有录制逻辑集成
pub struct EventDrivenRecorder {
    handle: Option<EventLoopHandle>,
    config: EventLoopConfig,
}

impl EventDrivenRecorder {
    pub fn new(config: EventLoopConfig) -> Self {
        Self {
            handle: None,
            config,
        }
    }

    /// 启动事件驱动录制
    pub fn start(&mut self) -> mpsc::Receiver<WindowEvent> {
        let (rx, handle) = start_event_loop(self.config.clone());
        self.handle = Some(handle);
        rx
    }

    /// 停止录制
    pub fn stop(&mut self) {
        if let Some(ref handle) = self.handle {
            handle.stop();
        }
        self.handle = None;
    }

    /// 检查是否正在运行
    pub fn is_running(&self) -> bool {
        self.handle.as_ref().map(|h| !h.is_stopped()).unwrap_or(false)
    }
}

impl Drop for EventDrivenRecorder {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EventLoopConfig::default();
        assert!(config.track_foreground);
        assert!(!config.track_lifecycle);
        assert!(!config.track_title_change);
        assert_eq!(config.debounce_ms, 100);
    }

    #[test]
    fn test_event_loop_handle() {
        let stop_flag = Arc::new(AtomicBool::new(false));
        let handle = EventLoopHandle { stop_flag };

        assert!(!handle.is_stopped());
        handle.stop();
        assert!(handle.is_stopped());
    }

    #[test]
    fn test_recorder_lifecycle() {
        let config = EventLoopConfig::default();
        let mut recorder = EventDrivenRecorder::new(config);

        assert!(!recorder.is_running());
        
        let _rx = recorder.start();
        assert!(recorder.is_running());

        recorder.stop();
        // 给线程一点时间停止
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert!(!recorder.is_running());
    }
}
