$ErrorActionPreference = "Stop"
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$targetDir = Join-Path $PSScriptRoot "..\src-tauri\resources"
$targetFile = Join-Path $targetDir "rapidocr.exe"

if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Path $targetDir -Force
}

$downloadUrl = "https://github.com/RapidAI/RapidOCR/releases/latest/download/rapidocr_windows_x64.exe"

Write-Host "Downloading to $targetFile..."
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $targetFile -UseBasicParsing
    Write-Host "Download complete."
} catch {
    Write-Host "Download failed: $_"
    exit 1
}
