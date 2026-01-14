# MemFlow 开发启动脚本

Write-Host "=== MemFlow 开发环境启动 ===" -ForegroundColor Cyan
Write-Host ""

# 检查环境
Write-Host "检查环境..." -ForegroundColor Yellow
if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    Write-Host "❌ pnpm 未安装，请先安装 pnpm" -ForegroundColor Red
    exit 1
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "❌ cargo 未安装，请先安装 Rust" -ForegroundColor Red
    exit 1
}

Write-Host "✅ 环境检查通过" -ForegroundColor Green
Write-Host ""

# 设置日志级别
$env:RUST_LOG = "info"
$env:RUST_BACKTRACE = "1"

# 启动开发服务器
Write-Host "启动开发服务器..." -ForegroundColor Yellow
Write-Host "提示: 按 Ctrl+C 停止服务器" -ForegroundColor Gray
Write-Host ""

pnpm tauri:dev

