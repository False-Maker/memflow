-- 终端输出日志表（Phase 2 能力的基础存储）
CREATE TABLE IF NOT EXISTS terminal_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Unix 时间戳（秒）
    timestamp INTEGER NOT NULL DEFAULT (strftime('%s','now')),
    -- 终端会话标识（例如窗口句柄或自定义 session id）
    terminal_session_id TEXT,
    -- 捕获时前台应用名（例如 Windows Terminal / PowerShell / cmd 等）
    app_name TEXT,
    -- 捕获时窗口标题
    window_title TEXT,
    -- 原始终端文本（未脱敏，读取时统一走 redact 模块）
    text TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_terminal_logs_timestamp
    ON terminal_logs(timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_terminal_logs_session
    ON terminal_logs(terminal_session_id);

