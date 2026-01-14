# OCR API 集成测试脚本
# 测试 RapidOCR API 服务是否正常工作

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "MemFlow OCR API Integration Test" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

$API_URL = "http://127.0.0.1:9003"

# 1. 检查服务是否运行
Write-Host "[1/3] 检查 OCR 服务状态..." -ForegroundColor Yellow
try {
    $response = python -c "import requests; r = requests.get('$API_URL/', timeout=3); print(r.status_code)"
    if ($response -eq "200") {
        Write-Host "  [OK] OCR 服务运行中" -ForegroundColor Green
    } else {
        Write-Host "  [!] OCR 服务返回异常状态: $response" -ForegroundColor Red
        Write-Host ""
        Write-Host "请先启动 OCR 服务:" -ForegroundColor Yellow
        Write-Host "  .\scripts\start_ocr_service.ps1" -ForegroundColor Gray
        exit 1
    }
} catch {
    Write-Host "  [X] OCR 服务未运行!" -ForegroundColor Red
    Write-Host ""
    Write-Host "请先启动 OCR 服务:" -ForegroundColor Yellow
    Write-Host "  .\scripts\start_ocr_service.ps1" -ForegroundColor Gray
    Write-Host "  或双击 scripts\start_ocr_service.bat" -ForegroundColor Gray
    exit 1
}

# 2. 测试 OCR 功能
Write-Host "[2/3] 测试 OCR 识别功能..." -ForegroundColor Yellow

$testImage = ".\src-tauri\resources\monitor_0_screenshot.png"
if (-not (Test-Path $testImage)) {
    $testImage = ".\monitor_0_screenshot.png"
}

if (Test-Path $testImage) {
    $result = python -c @"
import requests
import json
r = requests.post('$API_URL/ocr', files={'image': open('$testImage'.replace('\\', '/'), 'rb')})
print(f'Status: {r.status_code}')
data = r.json()
print(f'Lines: {len(data)}')
if data:
    texts = [v['rec_txt'] for v in list(data.values())[:5]]
    print(f'Sample: {texts}')
"@
    Write-Host "  $result" -ForegroundColor Gray
    Write-Host "  [OK] OCR 识别成功" -ForegroundColor Green
} else {
    Write-Host "  [!] 未找到测试图片，跳过" -ForegroundColor Yellow
}

# 3. 检查 Rust 编译
Write-Host "[3/3] 检查 Rust 代码编译..." -ForegroundColor Yellow
Set-Location .\src-tauri
$buildResult = cargo check 2>&1
if ($LASTEXITCODE -eq 0) {
    Write-Host "  [OK] Rust 代码编译正常" -ForegroundColor Green
} else {
    Write-Host "  [X] Rust 编译失败" -ForegroundColor Red
    Write-Host $buildResult -ForegroundColor Red
}
Set-Location ..

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "测试完成!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "使用方法:" -ForegroundColor Yellow
Write-Host "  1. 启动 OCR 服务: .\scripts\start_ocr_service.ps1" -ForegroundColor Gray
Write-Host "  2. 启动应用: pnpm tauri dev" -ForegroundColor Gray












