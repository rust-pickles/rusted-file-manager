@echo off
REM Double-click to install RUSTED from the files in this folder.
setlocal
cd /d "%~dp0"

where powershell >nul 2>nul
if errorlevel 1 (
    echo [ERROR] PowerShell was not found.
    pause
    exit /b 1
)

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0install.ps1"

if errorlevel 1 (
    echo.
    echo [FAILED] Installation did not complete. Scroll up for the reason.
    pause
    exit /b 1
)

echo.
pause
endlocal
