$ErrorActionPreference = "Stop"

Write-Host "=== RapidOCR Setup Guide ===" -ForegroundColor Cyan
Write-Host ""
Write-Host "NOTE: RapidOCR official releases may not include pre-built .exe files." -ForegroundColor Yellow
Write-Host "We recommend using the Python version instead." -ForegroundColor Yellow
Write-Host ""

Write-Host "Recommended: Use Python version" -ForegroundColor Green
Write-Host "  Run: .\scripts\setup_rapidocr_python.ps1" -ForegroundColor White
Write-Host ""

Write-Host "Alternative options:" -ForegroundColor Cyan
Write-Host "  1. Check RapidOCR-json releases:" -ForegroundColor White
Write-Host "     https://github.com/RapidAI/RapidOcrOnnx/releases" -ForegroundColor Gray
Write-Host ""
Write-Host "  2. Compile from source (advanced):" -ForegroundColor White
Write-Host "     See RAPIDOCR_INTEGRATION_STEPS.md for details" -ForegroundColor Gray
Write-Host ""
Write-Host "  3. Use Tesseract OCR instead:" -ForegroundColor White
Write-Host "     choco install tesseract" -ForegroundColor Gray
Write-Host ""

Write-Host "For detailed instructions, see:" -ForegroundColor Cyan
Write-Host "  RAPIDOCR_INTEGRATION_STEPS.md" -ForegroundColor White
