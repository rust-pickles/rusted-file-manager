<#
.SYNOPSIS
    Installs RUSTED for the current user (or all users) from a built installer
    or a portable executable.

.DESCRIPTION
    Two modes:

    1. With an installer present (dist-installer\RUSTED_*_x64-setup.exe) it runs
       it. The NSIS installer places the app under Program Files and registers it
       in Windows' Add/Remove Programs.

    2. With only the portable executable it installs by hand into
       "$env:ProgramFiles\RUSTED" and creates Start Menu and Desktop shortcuts,
       which is what "like a proper software" means in practice.

.PARAMETER Machine
    Install for every user instead of just you. Requires an elevated shell.

.PARAMETER Uninstall
    Remove an installation made by this script.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File install.ps1
#>
[CmdletBinding()]
param(
    [switch]$Machine,
    [switch]$Uninstall
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$productName = 'RUSTED'
$exeName = 'rusted-shell.exe'

function Test-Admin {
    $identity = [Security.Principal.WindowsIdentity]::GetCurrent()
    $principal = New-Object Security.Principal.WindowsPrincipal($identity)
    return $principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
}

# Program Files for "all users", or the per-user equivalent otherwise.
if ($Machine) {
    if (-not (Test-Admin)) {
        throw "-Machine installs into Program Files and needs an elevated shell. Right-click PowerShell, choose 'Run as administrator', and try again."
    }
    $installDir = Join-Path $env:ProgramFiles $productName
    $startMenu = Join-Path $env:ProgramData "Microsoft\Windows\Start Menu\Programs"
} else {
    $installDir = Join-Path $env:LOCALAPPDATA $productName
    $startMenu = Join-Path $env:APPDATA "Microsoft\Windows\Start Menu\Programs"
}
$installedExe = Join-Path $installDir $exeName
$shortcut = Join-Path $startMenu "$productName.lnk"
$desktopShortcut = Join-Path ([Environment]::GetFolderPath('Desktop')) "$productName.lnk"

if ($Uninstall) {
    Write-Host "Removing $productName from $installDir"
    foreach ($file in @($shortcut, $desktopShortcut)) {
        if (Test-Path $file) { Remove-Item $file -Force; Write-Host "  removed $file" }
    }
    if (Test-Path $installDir) {
        Remove-Item $installDir -Recurse -Force
        Write-Host "  removed $installDir"
    }
    Write-Host "Uninstalled. User data in %LOCALAPPDATA%\Rusted\data was left in place."
    exit 0
}

# Prefer the real installer: it registers an uninstaller with Windows.
$setup = Get-ChildItem -Path (Join-Path $repoRoot 'dist-installer') -Filter '*setup.exe' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

if ($setup) {
    Write-Host "Running installer: $($setup.FullName)"
    Write-Host "Follow its prompts; it installs $productName and registers it under Add/Remove Programs."
    $args = @('/S') # silent; the NSIS config installs per-user by default
    if ($Machine) { $args = @() }
    Start-Process -FilePath $setup.FullName -ArgumentList $args -Wait
    Write-Host "Installer finished."
    exit 0
}

# No installer yet: install the portable build by hand.
$portable = Join-Path $repoRoot 'target\release\rusted-shell.exe'
if (-not (Test-Path $portable)) {
    throw "Nothing to install. Run build-installer.bat first, or pass an installer path. Looked for '$portable'."
}

Write-Host "Installing the portable build to $installDir"
New-Item -ItemType Directory -Force -Path $installDir | Out-Null
Copy-Item $portable $installedExe -Force
Write-Host "  copied $exeName"

# WebView2 is a Windows component; the installer bootstrapper normally handles it.
$webview = Get-ItemProperty -Path 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}' -ErrorAction SilentlyContinue
if (-not $webview) {
    Write-Warning "WebView2 runtime was not detected. Install it from https://developer.microsoft.com/microsoft-edge/webview2/ or the app will not start."
}

# Shortcuts, so it appears in the Start Menu and on the Desktop like other apps.
New-Item -ItemType Directory -Force -Path $startMenu | Out-Null
$shell = New-Object -ComObject WScript.Shell
foreach ($target in @($shortcut, $desktopShortcut)) {
    $link = $shell.CreateShortcut($target)
    $link.TargetPath = $installedExe
    $link.WorkingDirectory = $installDir
    $link.Description = 'RUSTED desktop file manager'
    $link.IconLocation = $installedExe
    $link.Save()
    Write-Host "  created $target"
}

Write-Host ""
Write-Host "Installed $productName to $installDir" -ForegroundColor Green
Write-Host "Launch it from the Start Menu, or run: $installedExe"
Write-Host "Uninstall with: powershell -ExecutionPolicy Bypass -File install.ps1 -Uninstall"
