# OCR Function Test Script
# Test RapidOCR installation and functionality

$ErrorActionPreference = "Stop"

Write-Host "=== OCR Function Test ===" -ForegroundColor Cyan
Write-Host ""

# Check RapidOCR executable
$rapidocrPaths = @(
    (Join-Path $PSScriptRoot "..\src-tauri\resources\rapidocr.exe"),
    $env:RAPIDOCR_PATH
)

$rapidocrPath = $null
foreach ($path in $rapidocrPaths) {
    if ($path -and (Test-Path $path)) {
        $rapidocrPath = $path
        Write-Host "Found RapidOCR: $path" -ForegroundColor Green
        break
    }
}

if (-not $rapidocrPath) {
    Write-Host "RapidOCR executable not found" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please do one of the following:" -ForegroundColor Yellow
    Write-Host "  1. Run download script: .\scripts\download_rapidocr.ps1" -ForegroundColor White
    Write-Host "  2. Manually download and place at: src-tauri\resources\rapidocr.exe" -ForegroundColor White
    Write-Host "  3. Set environment variable: `$env:RAPIDOCR_PATH = 'C:\path\to\rapidocr.exe'" -ForegroundColor White
    exit 1
}

# Test 1: Check executable file
Write-Host ""
Write-Host "Test 1: Check executable file..." -ForegroundColor Cyan
$fileInfo = Get-Item $rapidocrPath
Write-Host "  File exists" -ForegroundColor Green
Write-Host "  File size: $([math]::Round($fileInfo.Length / 1MB, 2)) MB" -ForegroundColor Green
Write-Host "  File path: $rapidocrPath" -ForegroundColor Green

# Test 2: Check file permissions
Write-Host ""
Write-Host "Test 2: Check file permissions..." -ForegroundColor Cyan
try {
    $acl = Get-Acl $rapidocrPath
    Write-Host "  File permissions OK" -ForegroundColor Green
} catch {
    Write-Host "  Warning: Cannot read file permissions: $_" -ForegroundColor Yellow
}

# Test 3: Command line test
Write-Host ""
Write-Host "Test 3: Command line test..." -ForegroundColor Cyan
Write-Host "  Note: Need test image for full test" -ForegroundColor Yellow
Write-Host "  After creating test image, run:" -ForegroundColor Yellow
Write-Host "    & '$rapidocrPath' test_image.png" -ForegroundColor White

# Test 4: Check app configuration
Write-Host ""
Write-Host "Test 4: Check app configuration..." -ForegroundColor Cyan
$configPath = Join-Path $env:APPDATA "memflow\config.json"
if (Test-Path $configPath) {
    $config = Get-Content $configPath | ConvertFrom-Json
    Write-Host "  Config file exists" -ForegroundColor Green
    if ($config.ocr_engine -eq "rapidocr") {
        Write-Host "  OCR engine set to RapidOCR" -ForegroundColor Green
    } else {
        Write-Host "  OCR engine set to: $($config.ocr_engine)" -ForegroundColor Yellow
        Write-Host "    Please switch to RapidOCR in app settings" -ForegroundColor Yellow
    }
} else {
    Write-Host "  Config file not found (will be created on first run)" -ForegroundColor Yellow
}

# Test 5: Performance test suggestions
Write-Host ""
Write-Host "Test 5: Performance test suggestions" -ForegroundColor Cyan
Write-Host "  1. Start app: pnpm tauri:dev" -ForegroundColor White
Write-Host "  2. Select RapidOCR engine in settings" -ForegroundColor White
Write-Host "  3. Start recording" -ForegroundColor White
Write-Host "  4. Wait a few seconds, check activity records" -ForegroundColor White
Write-Host "  5. Check ocr_text field for content" -ForegroundColor White
Write-Host "  6. View logs: RUST_LOG=debug pnpm tauri:dev" -ForegroundColor White

Write-Host ""
Write-Host "=== Test Complete ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Yellow
Write-Host "  1. Ensure RapidOCR executable is ready" -ForegroundColor White
Write-Host "  2. Start app and test OCR functionality" -ForegroundColor White
Write-Host "  3. Check OCR recognition accuracy" -ForegroundColor White
