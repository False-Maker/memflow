@echo off
chcp 65001 >nul
REM MemFlow OCR API 服务启动脚本 (批处理版)
REM 双击即可启动 OCR 服务

echo ========================================
echo MemFlow OCR API Service
echo ========================================
echo.

REM 获取脚本所在目录
set SCRIPT_DIR=%~dp0
set PROJECT_DIR=%SCRIPT_DIR%..

REM 检查 Python
python --version >nul 2>&1
if errorlevel 1 (
    echo [X] Python 未安装！
    echo     请从 https://www.python.org/downloads/ 下载安装
    pause
    exit /b 1
)

echo [*] 启动 OCR 服务...
echo     地址: http://127.0.0.1:9003
echo     API 文档: http://127.0.0.1:9003/docs
echo.
echo 按 Ctrl+C 停止服务
echo.

python "%SCRIPT_DIR%ocr_server.py" -ip 127.0.0.1 -p 9003

pause
