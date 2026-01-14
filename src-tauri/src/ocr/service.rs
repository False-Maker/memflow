//! OCR 服务管理模块
//! 负责启动、检查和关闭 OCR API 服务

use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};

/// OCR 服务默认端口
pub const OCR_SERVICE_PORT: u16 = 9003;

/// OCR 服务进程句柄
static OCR_PROCESS: Lazy<Mutex<Option<Child>>> = Lazy::new(|| Mutex::new(None));

/// 启动 OCR 服务
pub fn start_service(app_handle: &AppHandle) -> Result<()> {
    // 检查服务是否已经在运行
    if is_service_running() {
        tracing::info!("OCR 服务已在运行");
        return Ok(());
    }

    // 获取 Python 脚本路径
    let script_path = get_ocr_server_path(app_handle)?;

    tracing::info!("启动 OCR 服务: {}", script_path.display());

    // 查找 Python 解释器
    let python = find_python()?;

    let log_path = app_handle
        .path()
        .app_data_dir()
        .ok()
        .map(|p| p.join("ocr_server.log"));

    let stderr = if let Some(ref log_path) = log_path {
        let _ = std::fs::create_dir_all(log_path.parent().unwrap_or(log_path));
        let _ = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .and_then(|mut f| writeln!(f, "\n--- OCR server start ---"))
            .ok();
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .map(Stdio::from)
            .unwrap_or_else(|_| Stdio::null())
    } else {
        Stdio::null()
    };

    // 启动 Python OCR 服务
    let mut child = Command::new(&python)
        .arg(&script_path)
        .arg("-ip")
        .arg("127.0.0.1")
        .arg("-p")
        .arg(OCR_SERVICE_PORT.to_string())
        .stdout(Stdio::null())
        .stderr(stderr)
        .spawn()
        .with_context(|| {
            format!(
                "启动 OCR 服务失败: {} {}",
                python.display(),
                script_path.display()
            )
        })?;

    let quick_check = wait_for_service_with_child(&mut child, Duration::from_secs(3));

    // 保存进程句柄
    *OCR_PROCESS.lock().unwrap() = Some(child);

    if let Err(e) = quick_check {
        if let Some(log_path) = log_path {
            if let Some(reason) = summarize_log_error(&log_path) {
                tracing::warn!("OCR 服务启动未就绪: {} ({}) (log: {})", e, reason, log_path.display());
            } else {
            tracing::warn!("OCR 服务启动未就绪: {} (log: {})", e, log_path.display());
            }
        } else {
            tracing::warn!("OCR 服务启动未就绪: {}", e);
        }
        return Ok(());
    }

    tracing::info!("OCR 服务启动成功，端口: {}", OCR_SERVICE_PORT);
    Ok(())
}

/// 停止 OCR 服务
pub fn stop_service() {
    let mut process_guard = OCR_PROCESS.lock().unwrap();

    if let Some(mut child) = process_guard.take() {
        tracing::info!("正在停止 OCR 服务...");

        // 尝试优雅关闭
        match child.kill() {
            Ok(_) => {
                let _ = child.wait();
                tracing::info!("OCR 服务已停止");
            }
            Err(e) => {
                tracing::warn!("停止 OCR 服务失败: {}", e);
            }
        }
    }
}

/// 检查 OCR 服务是否在运行
pub fn is_service_running() -> bool {
    // 尝试连接服务
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .no_proxy()
        .build();

    if let Ok(client) = client {
        let urls = [
            format!("http://127.0.0.1:{}/docs", OCR_SERVICE_PORT),
            format!("http://127.0.0.1:{}/", OCR_SERVICE_PORT),
        ];
        for url in urls {
            let response = client.get(&url).send();
            if let Ok(response) = response {
            return response.status().is_success();
        }
        }
    }

    false
}

/// 等待服务启动
fn wait_for_service_with_child(child: &mut Child, timeout: Duration) -> Result<()> {
    let start = std::time::Instant::now();
    let check_interval = Duration::from_millis(500);

    while start.elapsed() < timeout {
        if is_service_running() {
            return Ok(());
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(anyhow::anyhow!("OCR 进程异常退出: {}", status));
        }
        std::thread::sleep(check_interval);
    }

    Err(anyhow::anyhow!("OCR 服务启动超时"))
}

fn summarize_log_error(log_path: &Path) -> Option<String> {
    let content = std::fs::read_to_string(log_path).ok()?;
    let mut last_match: Option<&str> = None;
    for line in content.lines().rev().take(200) {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.contains("ModuleNotFoundError")
            || trimmed.contains("ImportError:")
            || trimmed.contains("Traceback (most recent call last)")
        {
            last_match = Some(trimmed);
            break;
        }
    }
    last_match.map(|s| s.to_string())
}

/// 获取 OCR 服务脚本路径
fn get_ocr_server_path(app_handle: &AppHandle) -> Result<std::path::PathBuf> {
    // 1. 尝试资源目录
    if let Ok(resource_dir) = app_handle.path().resource_dir() {
        let script_path = resource_dir.join("scripts").join("ocr_server.py");
        if script_path.exists() {
            return Ok(script_path);
        }
    }

    // 2. 尝试开发环境路径
    let dev_paths = [
        "scripts/ocr_server.py",
        "../scripts/ocr_server.py",
        "../../scripts/ocr_server.py",
    ];

    for path in &dev_paths {
        let script_path = std::path::PathBuf::from(path);
        if script_path.exists() {
            return Ok(script_path.canonicalize()?);
        }
    }

    // 3. 尝试从当前 exe 目录查找
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let script_path = exe_dir.join("scripts").join("ocr_server.py");
            if script_path.exists() {
                return Ok(script_path);
            }
        }
    }

    Err(anyhow::anyhow!(
        "未找到 OCR 服务脚本 (ocr_server.py)。\n\
        请确保 scripts/ocr_server.py 存在"
    ))
}

/// 查找 Python 解释器
fn find_python() -> Result<std::path::PathBuf> {
    // 1. 检查环境变量
    if let Ok(path) = std::env::var("PYTHON_PATH") {
        let path = std::path::PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    // 2. 尝试常见的 Python 命令
    let python_commands = if cfg!(windows) {
        vec!["python", "python3", "py"]
    } else {
        vec!["python3", "python"]
    };

    for cmd in python_commands {
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                return Ok(std::path::PathBuf::from(cmd));
            }
        }
    }

    Err(anyhow::anyhow!(
        "未找到 Python 解释器。\n\
        请安装 Python 3.8+ 并确保在系统 PATH 中"
    ))
}



