//! Windows UI Automation (UIA) 模块
//!
//! 实现毫秒级、无损文本提取，大幅减少 OCR 依赖。
//! 通过 Windows UI Automation API 直接获取窗口文本结构。
//!
//! 性能熔断机制：使用 TreeWalker 受控遍历，限制深度和超时，
//! 防止在复杂 UI（如 IDE、浏览器）中因遍历过多节点导致主线程挂起。

use anyhow::Result;
use std::time::Instant;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, GetWindowTextW, GetWindowTextLengthW,
};
use windows::Win32::System::Com::{
    CoInitializeEx, CoUninitialize, CoCreateInstance, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationElement, IUIAutomationTreeWalker,
    UIA_TextControlTypeId, UIA_EditControlTypeId, UIA_DocumentControlTypeId,
    UIA_ControlTypePropertyId, UIA_NamePropertyId, UIA_ValueValuePropertyId,
};
use windows::core::BSTR;

/// UIA 遍历性能熔断常量
const MAX_TRAVERSAL_DEPTH: u32 = 5;      // 最大遍历深度
const MAX_TRAVERSAL_TIME_MS: u128 = 200; // 最大遍历时间（毫秒）

/// 获取前台窗口的文本内容
/// 
/// 优先使用 UIA 获取结构化文本，失败时回退到窗口标题
pub fn get_foreground_text() -> Result<Option<String>> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0.is_null() {
            return Ok(None);
        }

        // 尝试获取窗口内的文本内容
        match get_window_text_content(hwnd) {
            Ok(Some(text)) if !text.trim().is_empty() => Ok(Some(text)),
            _ => {
                // 回退到窗口标题
                get_window_title(hwnd)
            }
        }
    }
}

/// 获取指定窗口的标题
pub fn get_window_title(hwnd: HWND) -> Result<Option<String>> {
    unsafe {
        let len = GetWindowTextLengthW(hwnd);
        if len == 0 {
            return Ok(None);
        }

        let mut buffer: Vec<u16> = vec![0; (len + 1) as usize];
        let copied = GetWindowTextW(hwnd, &mut buffer);
        
        if copied == 0 {
            return Ok(None);
        }

        let title = String::from_utf16_lossy(&buffer[..copied as usize]);
        if title.trim().is_empty() {
            Ok(None)
        } else {
            Ok(Some(title))
        }
    }
}

/// 尝试使用 UIA 获取窗口内的文本内容
/// 
/// 性能熔断实现：
/// 1. CoInitializeEx 初始化 COM
/// 2. 创建 IUIAutomation 实例
/// 3. 使用 TreeWalker 手动递归遍历（而非 FindAll）
/// 4. 硬限制：深度 5 层，超时 200ms
/// 5. 释放 COM 资源
pub fn get_window_text_content(hwnd: HWND) -> Result<Option<String>> {
    unsafe {
        // 1. COM 初始化
        let hr = CoInitializeEx(None, COINIT_MULTITHREADED);
        // S_OK (0) 或 S_FALSE (1) 都表示成功（S_FALSE 表示已初始化）
        if hr.is_err() && hr.0 != 1 {
            tracing::debug!("COM 初始化失败: {:?}", hr);
            return Ok(None);
        }
        
        // 使用 scopeguard 确保 CoUninitialize 被调用
        let _guard = scopeguard::guard((), |_| {
            CoUninitialize();
        });

        // 2. 创建 UIA 实例
        let automation: IUIAutomation = match CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER) {
            Ok(a) => a,
            Err(e) => {
                tracing::debug!("创建 UIA 实例失败: {:?}", e);
                return Ok(None);
            }
        };

        // 3. 获取窗口根元素
        let element: IUIAutomationElement = match automation.ElementFromHandle(hwnd) {
            Ok(e) => e,
            Err(e) => {
                tracing::debug!("获取窗口元素失败: {:?}", e);
                return Ok(None);
            }
        };

        // 4. 创建 TreeWalker（使用 ControlViewWalker，性能更好）
        let walker: IUIAutomationTreeWalker = match automation.ControlViewWalker() {
            Ok(w) => w,
            Err(e) => {
                tracing::debug!("创建 TreeWalker 失败: {:?}", e);
                return Ok(None);
            }
        };

        // 5. 使用受控遍历收集文本
        let start_time = Instant::now();
        let mut texts: Vec<String> = Vec::new();
        
        walk_tree(&walker, &element, 0, &start_time, &mut texts);

        if texts.is_empty() {
            Ok(None)
        } else {
            // 限制总长度，防止过大
            let combined = texts.join("\n");
            if combined.len() > 10000 {
                Ok(Some(combined.chars().take(10000).collect()))
            } else {
                Ok(Some(combined))
            }
        }
    }
}

/// 受控递归遍历 UI 树
/// 
/// 性能熔断：
/// - 深度限制：超过 MAX_TRAVERSAL_DEPTH 层立即截断
/// - 时间限制：超过 MAX_TRAVERSAL_TIME_MS 毫秒立即截断
fn walk_tree(
    walker: &IUIAutomationTreeWalker,
    element: &IUIAutomationElement,
    depth: u32,
    start_time: &Instant,
    texts: &mut Vec<String>,
) {
    // 熔断检查：超深或超时，立即返回
    if depth > MAX_TRAVERSAL_DEPTH {
        return;
    }
    if start_time.elapsed().as_millis() > MAX_TRAVERSAL_TIME_MS {
        tracing::debug!("UIA 遍历超时熔断 ({}ms)", MAX_TRAVERSAL_TIME_MS);
        return;
    }

    unsafe {
        // 检查当前元素是否是文本类型
        if let Ok(control_type) = element.GetCurrentPropertyValue(UIA_ControlTypePropertyId) {
            // VARIANT 内部结构需要通过 Anonymous 字段访问
            let ct_val: i32 = control_type.as_raw().Anonymous.Anonymous.Anonymous.lVal;
            let is_text_element = ct_val == UIA_TextControlTypeId.0
                || ct_val == UIA_EditControlTypeId.0
                || ct_val == UIA_DocumentControlTypeId.0;

            if is_text_element {
                // 尝试获取 Name 属性
                if let Ok(name) = element.GetCurrentPropertyValue(UIA_NamePropertyId) {
                    if let Ok(bstr) = BSTR::try_from(&name) {
                        let text = bstr.to_string();
                        if !text.trim().is_empty() {
                            texts.push(text);
                        }
                    }
                }

                // 尝试获取 Value 属性（用于 Edit 控件）
                if let Ok(value) = element.GetCurrentPropertyValue(UIA_ValueValuePropertyId) {
                    if let Ok(bstr) = BSTR::try_from(&value) {
                        let text = bstr.to_string();
                        if !text.trim().is_empty() && !texts.contains(&text) {
                            texts.push(text);
                        }
                    }
                }
            }
        }

        // 遍历子元素：FirstChild -> NextSibling
        if let Ok(first_child) = walker.GetFirstChildElement(element) {
            walk_tree(walker, &first_child, depth + 1, start_time, texts);

            // 遍历兄弟元素
            let mut current = first_child;
            while let Ok(next) = walker.GetNextSiblingElement(&current) {
                // 再次检查超时
                if start_time.elapsed().as_millis() > MAX_TRAVERSAL_TIME_MS {
                    tracing::debug!("UIA 遍历超时熔断 (兄弟遍历)");
                    return;
                }
                walk_tree(walker, &next, depth + 1, start_time, texts);
                current = next;
            }
        }
    }
}


/// 获取前台窗口句柄
pub fn get_foreground_hwnd() -> Option<isize> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            None
        } else {
            Some(hwnd.0 as isize)
        }
    }
}

/// 检查 UIA 是否可用
pub fn is_uia_available() -> bool {
    // 检查 Windows 版本和 UIA 可用性
    // Windows 7+ 默认支持 UIA
    cfg!(target_os = "windows")
}

/// UIA 文本提取选项
#[derive(Debug, Clone)]
pub struct UiaOptions {
    /// 是否包含隐藏元素
    pub include_hidden: bool,
    /// 最大文本长度
    pub max_length: usize,
    /// 超时时间（毫秒）
    pub timeout_ms: u32,
}

impl Default for UiaOptions {
    fn default() -> Self {
        Self {
            include_hidden: false,
            max_length: 10000,
            timeout_ms: 1000,
        }
    }
}

/// 使用 UIA 提取窗口文本（带选项）
pub fn get_foreground_text_with_options(_options: UiaOptions) -> Result<Option<String>> {
    // 当前实现忽略选项，直接调用基础版本
    get_foreground_text()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_uia_available() {
        // 在 Windows 上应该返回 true
        #[cfg(target_os = "windows")]
        assert!(is_uia_available());
    }

    #[test]
    fn test_get_foreground_hwnd() {
        // 测试获取前台窗口（可能为 None 如果在无头环境运行）
        let hwnd = get_foreground_hwnd();
        // 不断言具体值，因为测试环境可能没有前台窗口
        println!("Foreground HWND: {:?}", hwnd);
    }

    #[test]
    fn test_default_options() {
        let options = UiaOptions::default();
        assert!(!options.include_hidden);
        assert_eq!(options.max_length, 10000);
        assert_eq!(options.timeout_ms, 1000);
    }
}
