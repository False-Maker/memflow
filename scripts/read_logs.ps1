# MemFlow 日志读取脚本

$logDir = "$env:LOCALAPPDATA\com.memflow.app\logs"
$logFile = "$logDir\memflow.log"

Write-Host "=== MemFlow 日志查看工具 ===" -ForegroundColor Cyan
Write-Host ""

# 检查日志目录
if (-not (Test-Path $logDir)) {
    Write-Host "日志目录不存在: $logDir" -ForegroundColor Yellow
    Write-Host "应用可能尚未运行或尚未完成初始化" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "请检查运行应用的终端窗口查看实时日志" -ForegroundColor Green
    exit 0
}

# 检查日志文件
if (-not (Test-Path $logFile)) {
    Write-Host "日志文件不存在: $logFile" -ForegroundColor Yellow
    Write-Host "应用可能刚启动，日志尚未写入" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "请检查运行应用的终端窗口查看实时日志" -ForegroundColor Green
    exit 0
}

# 显示日志文件信息
$logInfo = Get-Item $logFile
Write-Host "日志文件: $logFile" -ForegroundColor Green
Write-Host "文件大小: $([math]::Round($logInfo.Length / 1KB, 2)) KB" -ForegroundColor Green
Write-Host "最后修改: $($logInfo.LastWriteTime)" -ForegroundColor Green
Write-Host ""

# 显示最后 N 行日志
$lines = 50
Write-Host "=== 最后 $lines 行日志 ===" -ForegroundColor Cyan
Write-Host ""
Get-Content $logFile -Tail $lines -Encoding UTF8

Write-Host ""
Write-Host "=== 日志统计 ===" -ForegroundColor Cyan

# 统计错误和警告
$allLogs = Get-Content $logFile -Encoding UTF8
$errorCount = ($allLogs | Select-String -Pattern "ERROR|CRITICAL|error" -CaseSensitive:$false).Count
$warnCount = ($allLogs | Select-String -Pattern "WARN|warning" -CaseSensitive:$false).Count
$infoCount = ($allLogs | Select-String -Pattern "INFO" -CaseSensitive:$false).Count

Write-Host "总日志行数: $($allLogs.Count)" -ForegroundColor White
Write-Host "INFO 级别: $infoCount" -ForegroundColor Green
Write-Host "WARN 级别: $warnCount" -ForegroundColor Yellow
Write-Host "ERROR 级别: $errorCount" -ForegroundColor Red

Write-Host ""
Write-Host "=== 最近的错误日志 ===" -ForegroundColor Cyan
$errors = $allLogs | Select-String -Pattern "ERROR|CRITICAL|error" -CaseSensitive:$false | Select-Object -Last 10
if ($errors) {
    $errors | ForEach-Object { Write-Host $_.Line -ForegroundColor Red }
} else {
    Write-Host "No error logs found" -ForegroundColor Green
}

