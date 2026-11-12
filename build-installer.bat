@echo off
REM ============================================================================
REM  RUSTED - build the Windows installer
REM
REM  Double-click this file (or run it from a terminal) to produce:
REM    dist-installer\RUSTED_<version>_x64-setup.exe   <- the installer
REM    target\release\rusted-shell.exe                 <- portable copy
REM
REM  Pass --portable to skip the installer and build only the exe.
REM ============================================================================
setlocal

cd /d "%~dp0"

echo.
echo  RUSTED installer build
echo  ----------------------

where powershell >nul 2>nul
if errorlevel 1 (
    echo [ERROR] PowerShell was not found. This script needs it to build.
    pause
    exit /b 1
)

if /I "%~1"=="--portable" (
    powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\build-installer.ps1" -Portable
) else (
    powershell -NoProfile -ExecutionPolicy Bypass -File "scripts\build-installer.ps1"
)

if errorlevel 1 (
    echo.
    echo [FAILED] The build did not complete. Scroll up for the first error.
    pause
    exit /b 1
)

echo.
echo  Done. The installer is in the dist-installer folder.
echo.
pause
endlocal
