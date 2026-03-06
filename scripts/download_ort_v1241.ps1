$headers = @{'User-Agent' = 'Mozilla/5.0'}
$response = Invoke-RestMethod -Uri 'https://api.github.com/repos/microsoft/onnxruntime/releases/tags/v1.24.1' -Headers $headers
$asset = $response.assets | Where-Object { $_.name -eq 'onnxruntime-win-x64-1.24.1.zip' } | Select-Object -First 1

if ($asset) {
    Write-Host "Download URL: $($asset.browser_download_url)"
    $downloadUrl = $asset.browser_download_url
} else {
    Write-Host "Asset not found"
    exit 1
}

$tempFile = "C:\temp\onnxruntime-win-x64-1.24.1.zip"
if (-not (Test-Path "C:\temp")) {
    New-Item -ItemType Directory -Path "C:\temp" -Force | Out-Null
}

Write-Host "Downloading ONNX Runtime v1.24.1..."
Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing -Headers $headers

Write-Host "Extracting..."
$extractDir = "C:\temp\onnxruntime-extract"
if (Test-Path $extractDir) {
    Remove-Item $extractDir -Recurse -Force
}
Expand-Archive -Path $tempFile -DestinationPath $extractDir -Force

$sourceDll = Get-ChildItem -Path $extractDir -Filter "onnxruntime.dll" -Recurse | Select-Object -First 1
if ($sourceDll) {
    Write-Host "Found: $($sourceDll.FullName)"
    $targetDll = "D:\Demo\memflow\src-tauri\resources\onnxruntime.dll"
    Copy-Item -Path $sourceDll.FullName -Destination $targetDll -Force
    Write-Host "Copied to: $targetDll"
    
    $version = (Get-Item $targetDll).VersionInfo.FileVersion
    Write-Host "Version: $version"
} else {
    Write-Host "DLL not found in archive"
}
