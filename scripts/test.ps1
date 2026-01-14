# MemFlow Test Script

Write-Host "=== MemFlow Test Script ===" -ForegroundColor Cyan
Write-Host ""

# Check dependencies
Write-Host "1. Checking dependencies..." -ForegroundColor Yellow
if (-not (Get-Command pnpm -ErrorAction SilentlyContinue)) {
    Write-Host "X pnpm not installed" -ForegroundColor Red
    exit 1
}
Write-Host "V pnpm installed" -ForegroundColor Green

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "X cargo not installed" -ForegroundColor Red
    exit 1
}
Write-Host "V cargo installed" -ForegroundColor Green

# Check Rust Version
$rustVersion = (rustc --version).Split(' ')[1]
Write-Host "V Rust Version: $rustVersion" -ForegroundColor Green

# Check Node Version
$nodeVersion = node --version
Write-Host "V Node Version: $nodeVersion" -ForegroundColor Green

Write-Host ""
Write-Host "2. Checking Rust dependencies..." -ForegroundColor Yellow

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$distPath = Join-Path $repoRoot "dist"

if (-not (Test-Path $distPath)) {
    Write-Host ""
    Write-Host "2.1 Frontend dist not found, building frontend..." -ForegroundColor Yellow
    Push-Location $repoRoot
    pnpm install
    if ($LASTEXITCODE -ne 0) {
        Write-Host "X pnpm install failed" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    pnpm build
    if ($LASTEXITCODE -ne 0) {
        Write-Host "X pnpm build failed" -ForegroundColor Red
        Pop-Location
        exit 1
    }
    Pop-Location
    Write-Host "V Frontend build completed" -ForegroundColor Green
}

Push-Location src-tauri
cargo check
if ($LASTEXITCODE -ne 0) {
    Write-Host "X Rust Check Failed" -ForegroundColor Red
    Pop-Location
    exit 1
}
cargo test
if ($LASTEXITCODE -ne 0) {
    Write-Host "X Rust Test Failed" -ForegroundColor Red
    Pop-Location
    exit 1
}
Pop-Location
Write-Host "V Rust Check and Test Passed" -ForegroundColor Green

Write-Host ""
Write-Host "V All Checks Completed!" -ForegroundColor Green
