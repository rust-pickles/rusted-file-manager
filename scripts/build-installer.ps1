<#
.SYNOPSIS
    Builds the RUSTED installer (and optionally the raw executable) locally.

.DESCRIPTION
    Produces:
      dist-installer/RUSTED_<version>_x64-setup.exe   the NSIS installer
      target/release/rusted-shell.exe                 the portable executable

    Everything it needs is fetched on first run (Rust toolchain, Node packages,
    the NSIS bundler toolchain), so the first build takes a while and later ones
    are fast.

.PARAMETER Portable
    Skip the installer and build only the standalone executable.

.PARAMETER NoBundle
    Alias for -Portable, kept for familiarity with `cargo tauri build`.

.EXAMPLE
    powershell -ExecutionPolicy Bypass -File scripts\build-installer.ps1
#>
[CmdletBinding()]
param(
    [switch]$Portable,
    [switch]$NoBundle
)

$ErrorActionPreference = 'Stop'
# Every native command must fail the script, not silently continue.
$PSNativeCommandUseErrorActionPreference = $true

$repoRoot = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $repoRoot

function Write-Step([string]$Message) {
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Assert-Command([string]$Name, [string]$Hint) {
    if (-not (Get-Command $Name -ErrorAction SilentlyContinue)) {
        throw "'$Name' was not found. $Hint"
    }
}

Write-Step "Checking prerequisites"
Assert-Command 'node' 'Install Node.js 20 or newer from https://nodejs.org and re-run this script.'
Assert-Command 'npm'  'npm ships with Node.js; reinstall Node.js if it is missing.'

$nodeMajor = [int]((node --version).TrimStart('v').Split('.')[0])
if ($nodeMajor -lt 20) {
    throw "Node.js 20 or newer is required (found $(node --version))."
}
Write-Host "node $(node --version) / npm $(npm --version)"

if (-not (Get-Command 'cargo' -ErrorAction SilentlyContinue)) {
    Write-Step "Installing the Rust toolchain"
    # rustup-init is the only supported installer; it is fetched over TLS.
    $rustup = Join-Path $env:TEMP 'rustup-init.exe'
    Invoke-WebRequest -Uri 'https://win.rustup.rs/x86_64' -OutFile $rustup -UseBasicParsing
    & $rustup -y --default-toolchain stable --profile minimal
    if ($LASTEXITCODE -ne 0) { throw "rustup-init failed with exit code $LASTEXITCODE" }
    # Make cargo visible to the rest of this process.
    $env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
}
Write-Host "cargo $(cargo --version)"

Write-Step "Installing frontend dependencies"
npm install
if ($LASTEXITCODE -ne 0) { throw "npm install failed with exit code $LASTEXITCODE" }

if ($Portable -or $NoBundle) {
    Write-Step "Building the standalone executable"
    # tauri build runs `npm run build` first (beforeBuildCommand), so the
    # webview assets are always current.
    npx tauri build --no-bundle
    if ($LASTEXITCODE -ne 0) { throw "tauri build failed with exit code $LASTEXITCODE" }
    $exe = Join-Path $repoRoot 'target\release\rusted-shell.exe'
    if (-not (Test-Path $exe)) { throw "build finished but $exe is missing" }
    Write-Host ""
    Write-Host "Executable: $exe" -ForegroundColor Green
    exit 0
}

Write-Step "Building the installer"
# NSIS is the Windows target configured in src-tauri/tauri.conf.json.
npx tauri build --bundles nsis
if ($LASTEXITCODE -ne 0) { throw "tauri build failed with exit code $LASTEXITCODE" }

$version = (Get-Content (Join-Path $repoRoot 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json).version
$setup = Get-ChildItem -Path (Join-Path $repoRoot 'target\release\bundle\nsis') -Filter '*setup.exe' -ErrorAction SilentlyContinue |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if (-not $setup) {
    throw "tauri build reported success but no *setup.exe was produced under target\release\bundle\nsis"
}

# Copy to a predictable place so the file is easy to find and hand to someone.
$outDir = Join-Path $repoRoot 'dist-installer'
New-Item -ItemType Directory -Force -Path $outDir | Out-Null
$out = Join-Path $outDir ("RUSTED_{0}_x64-setup.exe" -f $version)
Copy-Item $setup.FullName $out -Force

$sizeMb = [math]::Round((Get-Item $out).Length / 1MB, 1)
Write-Host ""
Write-Host "Installer: $out ($sizeMb MB)" -ForegroundColor Green
Write-Host "Portable exe: $(Join-Path $repoRoot 'target\release\rusted-shell.exe')" -ForegroundColor Green
