#[cfg(windows)]
use windows::{
    core::*, Win32::Foundation::*, Win32::System::Threading::*, Win32::UI::WindowsAndMessaging::*,
};

#[cfg(windows)]
pub struct WindowInfo {
    pub title: String,
    pub process_name: String,
    pub process_path: String,
}

#[cfg(windows)]
pub fn get_foreground_window_info() -> anyhow::Result<WindowInfo> {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.0 == std::ptr::null_mut() {
            return Err(anyhow::anyhow!("无法获取前台窗口"));
        }

        // 获取窗口标题
        let mut title = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut title);
        let title_str = if len > 0 {
            String::from_utf16_lossy(&title[..len as usize])
        } else {
            "无标题".to_string()
        };

        // 获取进程 ID
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        // 获取进程名称和路径
        let (process_name, process_path) = get_process_info(process_id)?;

        Ok(WindowInfo {
            title: title_str,
            process_name,
            process_path,
        })
    }
}

#[cfg(windows)]
fn get_process_info(process_id: u32) -> anyhow::Result<(String, String)> {
    unsafe {
        let handle = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        )
        .map_err(|e| anyhow::anyhow!("无法打开进程: {:?}", e))?;

        let mut name = [0u16; 512];
        let mut size = name.len() as u32;

        // 在 windows 0.58+ 中，需要使用 PWSTR
        let pwstr = PWSTR(name.as_mut_ptr());
        QueryFullProcessImageNameW(handle, PROCESS_NAME_WIN32, pwstr, &mut size)
            .map_err(|e| anyhow::anyhow!("无法查询进程名称: {:?}", e))?;

        let _ = CloseHandle(handle);

        let path = String::from_utf16_lossy(&name[..size as usize]);
        let process_name = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知")
            .to_string();

        Ok((process_name, path))
    }
}

#[cfg(not(windows))]
pub fn get_foreground_window_info() -> anyhow::Result<WindowInfo> {
    Ok(WindowInfo {
        title: "不支持的平台".to_string(),
        process_name: "unknown".to_string(),
        process_path: String::new(),
    })
}
