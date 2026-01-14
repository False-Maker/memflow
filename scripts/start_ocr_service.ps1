# MemFlow OCR API 服务启动脚本
# 启动基于 RapidOCR (OpenVINO) 的 OCR 服务

param(
    [int]$Port = 9003,
    [string]$Host = "127.0.0.1"
)

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "MemFlow OCR API Service" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# 检查依赖
Write-Host "[1/3] 检查依赖..." -ForegroundColor Yellow

# 检查 rapidocr
$checkRapidocr = python -c "from rapidocr import RapidOCR" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [!] rapidocr 未安装，正在安装..." -ForegroundColor Yellow
    pip install rapidocr --quiet
}
Write-Host "  [OK] rapidocr" -ForegroundColor Green

# 检查 openvino
$checkOpenvino = python -c "import openvino" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [!] openvino 未安装，正在安装..." -ForegroundColor Yellow
    pip install openvino --quiet
}
Write-Host "  [OK] openvino" -ForegroundColor Green

# 检查 fastapi + uvicorn
$checkFastapi = python -c "import fastapi; import uvicorn" 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "  [!] fastapi/uvicorn 未安装，正在安装..." -ForegroundColor Yellow
    pip install fastapi uvicorn python-multipart --quiet
}
Write-Host "  [OK] fastapi + uvicorn" -ForegroundColor Green

Write-Host ""
Write-Host "[2/3] 检查配置文件..." -ForegroundColor Yellow

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectDir = Split-Path -Parent $scriptDir
$configPath = Join-Path $projectDir "default_rapidocr.yaml"

if (Test-Path $configPath) {
    Write-Host "  [OK] 配置文件: $configPath" -ForegroundColor Green
} else {
    Write-Host "  [!] 配置文件不存在，生成默认配置..." -ForegroundColor Yellow
    Set-Location $projectDir
    rapidocr config
    
    # 修改为使用 openvino
    $content = Get-Content $configPath -Raw
    $content = $content -replace 'engine_type: "onnxruntime"', 'engine_type: "openvino"'
    Set-Content $configPath $content
    Write-Host "  [OK] 已生成并配置为 OpenVINO" -ForegroundColor Green
}

Write-Host ""
Write-Host "[3/3] 启动 OCR 服务..." -ForegroundColor Yellow
Write-Host "  地址: http://${Host}:${Port}" -ForegroundColor Gray
Write-Host "  API 文档: http://${Host}:${Port}/docs" -ForegroundColor Gray
Write-Host ""
Write-Host "按 Ctrl+C 停止服务" -ForegroundColor DarkGray
Write-Host ""

# 启动服务
$ocrServer = Join-Path $scriptDir "ocr_server.py"
python $ocrServer -ip $Host -p $Port
