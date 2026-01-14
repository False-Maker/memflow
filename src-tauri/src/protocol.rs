use anyhow::Result;
use tauri::{AppHandle, Manager};

/// 处理 appimg:// 协议请求
/// 格式: appimg://screenshots/filename.png
pub fn handle_appimg_protocol(app_handle: &AppHandle, uri: &str) -> Result<Vec<u8>> {
    // 移除协议前缀
    let path = uri
        .strip_prefix("appimg://")
        .ok_or_else(|| anyhow::anyhow!("无效的协议 URI"))?;

    // 获取截图目录
    let app_data = app_handle
        .path()
        .app_data_dir()
        .map_err(|e| anyhow::anyhow!("无法获取应用数据目录: {}", e))?;

    let screenshots_dir = app_data.join("screenshots");
    let file_path = screenshots_dir.join(path);

    // 验证路径安全性（防止路径遍历攻击）
    if !file_path.starts_with(&screenshots_dir) {
        return Err(anyhow::anyhow!("路径不安全"));
    }

    // 读取文件
    let content = std::fs::read(&file_path)?;

    Ok(content)
}
