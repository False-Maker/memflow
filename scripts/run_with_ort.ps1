# 设置环境变量后启动 MemFlow
$env:ORT_DYLIB_PATH = "D:\Demo\memflow\src-tauri\resources\onnxruntime.dll"
Write-Host "ORT_DYLIB_PATH = $env:ORT_DYLIB_PATH"
Write-Host ""

# 检查 DLL
$dllPath = $env:ORT_DYLIB_PATH
if (Test-Path $dllPath) {
    $version = (Get-Item $dllPath).VersionInfo.FileVersion
    Write-Host "DLL Version: $version" -ForegroundColor Green
} else {
    Write-Host "DLL not found at: $dllPath" -ForegroundColor Red
    exit 1
}

# 启动应用
$exePath = "D:\Demo\memflow\target\debug\memflow.exe"
if (Test-Path $exePath) {
    Write-Host ""
    Write-Host "Starting MemFlow..." -ForegroundColor Cyan
    & $exePath
} else {
    Write-Host "EXE not found at: $exePath" -ForegroundColor Red
}
