# MemFlow ONNX Runtime 下载脚本
# 
# ort 2.0.0-rc.11 需要 onnxruntime.dll 1.24.1 版本
# 此脚本下载并放置 DLL 到正确的位置

$ErrorActionPreference = "Stop"

# ONNX Runtime 1.24.1 下载 URL (Windows x64)
$downloadUrl = "https://github.com/microsoft/onnxruntime/releases/download/v1.24.1/onnxruntime-win-x64-1.24.1.zip"
$tempFile = Join-Path $env:TEMP "onnxruntime-win-x64-1.24.1.zip"

# 目标目录
$targetDir = Join-Path $PSScriptRoot "src-tauri\resources"
$targetDll = Join-Path $targetDir "onnxruntime.dll"

Write-Host "=== MemFlow ONNX Runtime 下载工具 ===" -ForegroundColor Cyan
Write-Host ""

# 创建目标目录
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force | Out-Null
}

# 检查 DLL 是否已存在
if (Test-Path $targetDll) {
    Write-Host "✅ onnxruntime.dll 已存在于 resources 目录" -ForegroundColor Green
    $existingVersion = (Get-Item $targetDll).VersionInfo.FileVersion
    Write-Host "   当前版本: $existingVersion" -ForegroundColor Gray
    Write-Host ""
    $response = Read-Host "是否重新下载? (y/N)"
    if ($response -ne "y" -and $response -ne "Y") {
        Write-Host "跳过下载" -ForegroundColor Yellow
        exit 0
    }
}

Write-Host "下载 ONNX Runtime 1.24.1..." -ForegroundColor Yellow
Write-Host "   URL: $downloadUrl" -ForegroundColor Gray
Write-Host ""

try {
    # 下载文件
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
    
    # 解压
    Write-Host "解压文件..." -ForegroundColor Yellow
    $extractDir = Join-Path $env:TEMP "onnxruntime-extract"
    if (Test-Path $extractDir) {
        Remove-Item $extractDir -Recurse -Force
    }
    Expand-Archive -Path $tempFile -DestinationPath $extractDir -Force
    
    # 查找 DLL - 在解压目录中递归搜索
    $sourceDll = Get-ChildItem -Path $extractDir -Filter "onnxruntime.dll" -Recurse | Select-Object -First 1
    
    if (-not $sourceDll) {
        throw "解压后未找到 onnxruntime.dll"
    }
    
    Write-Host "   找到 DLL: $($sourceDll.FullName)" -ForegroundColor Gray
    
    # 复制到目标目录
    Copy-Item -Path $sourceDll.FullName -Destination $targetDll -Force
    
    # 清理临时文件
    if (Test-Path $tempFile) {
        Remove-Item $tempFile -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $extractDir) {
        Remove-Item $extractDir -Recurse -Force -ErrorAction SilentlyContinue
    }
    
    Write-Host ""
    Write-Host "✅ ONNX Runtime 安装成功!" -ForegroundColor Green
    Write-Host "   DLL 位置: $targetDll" -ForegroundColor Gray
    
    $version = (Get-Item $targetDll).VersionInfo.FileVersion
    Write-Host "   版本: $version" -ForegroundColor Gray
    Write-Host ""
    Write-Host "提示: 如果 memflow-mcp 仍然无法加载，请将 DLL 复制到:" -ForegroundColor Yellow
    Write-Host "   1. memflow-mcp 可执行文件同级目录" -ForegroundColor Gray
    Write-Host "   2. 或者系统 PATH 目录中" -ForegroundColor Gray
    
} catch {
    Write-Host ""
    Write-Host "❌ 下载失败: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "手动下载方法:" -ForegroundColor Yellow
    Write-Host "   1. 访问: https://github.com/microsoft/onnxruntime/releases/tag/v1.24.1" -ForegroundColor Gray
    Write-Host "   2. 下载 onnxruntime-win-x64-1.24.1.zip" -ForegroundColor Gray
    Write-Host "   3. 解压后找到 onnxruntime.dll" -ForegroundColor Gray
    Write-Host "   4. 复制到 src-tauri\resources\ 目录" -ForegroundColor Gray
    exit 1
}
