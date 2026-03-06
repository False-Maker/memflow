$env:ORT_DYLIB_PATH = "D:\Demo\memflow\target\debug\onnxruntime.dll"
$env:RUST_BACKTRACE = "1"

Write-Host "=== Starting MemFlow ===" -ForegroundColor Cyan
Write-Host "ORT_DYLIB_PATH: $env:ORT_DYLIB_PATH"
Write-Host "DLL exists: $(Test-Path $env:ORT_DYLIB_PATH)"
Write-Host ""

$exePath = "D:\Demo\memflow\target\debug\memflow.exe"
$errLog = "D:\temp\memflow_err2.log"
$outLog = "D:\temp\memflow_out2.log"

$proc = Start-Process -FilePath $exePath -PassThru -RedirectStandardError $errLog -RedirectStandardOutput $outLog
Write-Host "Process started with ID: $($proc.Id)"

# Wait a bit for the error to show
Start-Sleep -Seconds 5

if (Test-Path $errLog) {
    $errContent = Get-Content $errLog -Raw
    if ($errContent) {
        Write-Host "=== STDERR ===" -ForegroundColor Red
        Write-Host $errContent
    }
}

if (Test-Path $outLog) {
    $outContent = Get-Content $outLog -Raw
    if ($outContent) {
        Write-Host "=== STDOUT ===" -ForegroundColor Yellow
        Write-Host $outContent
    }
}

Write-Host ""
Write-Host "Process still running: $(-not $proc.HasExited)"
if ($proc.HasExited) {
    Write-Host "Exit code: $($proc.ExitCode)" -ForegroundColor $(if ($proc.ExitCode -eq 0) { "Green" } else { "Red" })
}
