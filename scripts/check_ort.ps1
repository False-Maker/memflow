$headers = @{'User-Agent' = 'Mozilla/5.0'}
$response = Invoke-RestMethod -Uri 'https://api.github.com/repos/microsoft/onnxruntime/releases/tags/v1.24.1' -Headers $headers
$response.assets | Where-Object { $_.name -like '*win-x64*' } | Select-Object name, browser_download_url
