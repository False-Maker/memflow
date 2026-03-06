use anyhow::Result;

/// 桌面端封装：复用 memflow-core 提供的系统环境探测逻辑。
///
/// 该函数不访问任何远程服务，仅基于本地 OS / PATH 信息收集：
/// - 基础信息：OS、架构、用户、主机名、Shell、目录、CPU 核心数
/// - 常见开发工具版本（git / rustc / cargo / node / npm / pnpm / python 等）
/// - 一小撮常见本地端口是否被占用（3000 / 5173 / 8000 / 8080 等）
pub async fn get_system_environment_report() -> Result<String> {
    Ok(memflow_core::system_env::get_system_environment_report().await)
}

