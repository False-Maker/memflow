$ErrorActionPreference = "Stop"

Write-Host "=== RapidOCR Python Setup Script ===" -ForegroundColor Cyan
Write-Host ""

# Check Python installation
Write-Host "Checking Python installation..." -ForegroundColor Yellow
$pythonCmd = $null

# Try python3
try {
    $output = python3 --version 2>&1
    if ($LASTEXITCODE -eq 0) {
        $pythonCmd = "python3"
        Write-Host "Found Python: $output" -ForegroundColor Green
    }
} catch {
    # Continue
}

# Try python
if (-not $pythonCmd) {
    try {
        $output = python --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            $pythonCmd = "python"
            Write-Host "Found Python: $output" -ForegroundColor Green
        }
    } catch {
        # Continue
    }
}

# Try py (Windows)
if (-not $pythonCmd) {
    try {
        $output = py --version 2>&1
        if ($LASTEXITCODE -eq 0) {
            $pythonCmd = "py"
            Write-Host "Found Python: $output" -ForegroundColor Green
        }
    } catch {
        # Continue
    }
}

if (-not $pythonCmd) {
    Write-Host "Python not found!" -ForegroundColor Red
    Write-Host ""
    Write-Host "Please install Python 3.7 or higher:" -ForegroundColor Yellow
    Write-Host "  1. Visit: https://www.python.org/downloads/" -ForegroundColor White
    Write-Host "  2. Download and install Python 3.7+" -ForegroundColor White
    Write-Host "  3. Make sure to check 'Add Python to PATH' during installation" -ForegroundColor White
    Write-Host "  4. Run this script again" -ForegroundColor White
    exit 1
}

Write-Host ""
Write-Host "Installing RapidOCR Python package..." -ForegroundColor Yellow
Write-Host "This may take a few minutes..." -ForegroundColor Yellow
Write-Host ""

# Install rapidocr and runtime
try {
    & $pythonCmd -m pip install rapidocr onnxruntime fastapi uvicorn python-multipart --quiet
    if ($LASTEXITCODE -eq 0) {
        Write-Host "RapidOCR installed successfully!" -ForegroundColor Green
    } else {
        throw "pip install failed"
    }
} catch {
    Write-Host "Failed to install RapidOCR: $_" -ForegroundColor Red
    Write-Host ""
    Write-Host "Try installing manually:" -ForegroundColor Yellow
    Write-Host "  $pythonCmd -m pip install rapidocr onnxruntime fastapi uvicorn python-multipart" -ForegroundColor White
    exit 1
}

Write-Host ""
Write-Host "Verifying installation..." -ForegroundColor Yellow
try {
    & $pythonCmd -c "import onnxruntime; from rapidocr import RapidOCR; print('RapidOCR import successful')" 2>&1
    if ($LASTEXITCODE -eq 0) {
        Write-Host "Verification successful!" -ForegroundColor Green
    } else {
        throw "verification failed"
    }
} catch {
    Write-Host "Verification failed. Please check the installation." -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "Setup complete!" -ForegroundColor Green
Write-Host ""
Write-Host "Next steps:" -ForegroundColor Cyan
Write-Host "  1. The Python wrapper script is at: src-tauri/resources/rapidocr_wrapper.py" -ForegroundColor White
Write-Host "  2. Make sure Python is in your system PATH" -ForegroundColor White
Write-Host "  3. Test with: python src-tauri/resources/rapidocr_wrapper.py <image_path>" -ForegroundColor White

