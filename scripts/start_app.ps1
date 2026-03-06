$exePath = "D:\Demo\memflow\target\debug\memflow.exe"
Write-Host "Starting MemFlow..." -ForegroundColor Cyan
$proc = Start-Process -FilePath $exePath -PassThru
Write-Host "Process ID: $($proc.Id)"
Write-Host "Started: $(-not $proc.HasExited)"

Start-Sleep -Seconds 5

if (-not $proc.HasExited) {
    Write-Host "Application is running!" -ForegroundColor Green
} else {
    Write-Host "Application exited with code: $($proc.ExitCode)" -ForegroundColor Red
}
