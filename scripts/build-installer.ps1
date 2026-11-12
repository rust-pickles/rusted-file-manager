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
# Deliberately no $PSNativeCommandUseErrorActionPreference here: it is PS7-only,
# and when it fires it aborts on the native command itself with no message, so
# the readable throw below never runs and the user sees a CategoryInfo dump
# instead of the reason. Every native call is checked for exit code instead.

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

# Locates the MSVC linker that the Rust msvc target needs. Rust calls link.exe
# through an environment that rustup normally sets up, but a toolchain installed
# by rustup-init does not come with the linker at all: it belongs to Visual
# Studio Build Tools. Without this check the build dies halfway through
# compiling dependencies with a bare "linker `link.exe` not found".
function Find-MsvcLinker {
    # Already on PATH (a Developer Command Prompt, or a previous run in this shell).
    $onPath = Get-Command 'link.exe' -ErrorAction SilentlyContinue
    if ($onPath) { return $onPath.Source }

    # vswhere ships with every VS 2017+ installer, including Build Tools.
    $vswhere = Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\Installer\vswhere.exe'
    if (-not (Test-Path $vswhere)) { return $null }

    $vsPath = & $vswhere -latest -products * `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath 2>$null
    if (-not $vsPath) { return $null }

    # The linker lives under VC\Tools\MSVC\<version>\bin\Host<x64>\<arch>.
    $found = Get-ChildItem -Path (Join-Path $vsPath 'VC\Tools\MSVC') -Recurse -Filter 'link.exe' -ErrorAction SilentlyContinue |
        Where-Object { $_.FullName -match '\\Hostx64\\x64\\' } |
        Sort-Object FullName -Descending |
        Select-Object -First 1
    if ($found) { return $found.FullName }
    return $null
}

function Install-MsvcBuildTools {
    Write-Step "Installing the C++ build tools (needed by the Rust linker)"
    $bootstrapper = Join-Path $env:TEMP 'vs_BuildTools.exe'
    Invoke-WebRequest -Uri 'https://aka.ms/vs/17/release/vs_BuildTools.exe' `
        -OutFile $bootstrapper -UseBasicParsing

    # VCTools is the MSVC compiler and linker. Windows11SDK is required for the
    # Windows headers and import libraries; both are needed to link a Tauri app.
    $arguments = @(
        '--quiet', '--wait', '--norestart', '--nocache',
        '--installPath', (Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio\2022\BuildTools'),
        '--add', 'Microsoft.VisualStudio.Component.VC.Tools.x86.x64',
        '--add', 'Microsoft.VisualStudio.Component.Windows11SDK.22621'
    )
    Write-Host "This downloads roughly 2-4 GB and takes several minutes. Windows will ask for permission once."
    # Build Tools installs under Program Files, so it needs elevation. Asking for
    # it here is better than failing with a cryptic exit code.
    $isAdmin = ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole(
        [Security.Principal.WindowsBuiltInRole]::Administrator)
    $elevate = @{}
    if (-not $isAdmin) { $elevate['Verb'] = 'RunAs' }
    $process = Start-Process -FilePath $bootstrapper -ArgumentList $arguments -Wait -PassThru @elevate
    # 0 = installed, 3010 = installed but a reboot is pending. Both are fine.
    if ($process.ExitCode -ne 0 -and $process.ExitCode -ne 3010) {
        throw "The C++ build tools installer failed (exit $($process.ExitCode)). Install 'Desktop development with C++' from https://visualstudio.microsoft.com/visual-cpp-build-tools/ by hand, then re-run this script."
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

# rustup installs the compiler but never the linker. Catch it here, where the
# message makes sense, instead of ten minutes into compiling dependencies.
if (-not (Find-MsvcLinker)) {
    Install-MsvcBuildTools
    if (-not (Find-MsvcLinker)) {
        throw "The C++ build tools finished installing but link.exe still cannot be found. Reboot, then re-run this script. If it still fails, install 'Desktop development with C++' from https://visualstudio.microsoft.com/visual-cpp-build-tools/."
    }
}
$linker = Find-MsvcLinker
# rustc finds the MSVC libraries itself, but putting the linker on PATH also
# covers build scripts that shell out to link.exe directly.
$env:Path = "$(Split-Path -Parent $linker);$env:Path"
Write-Host "C++ linker: $linker"

Write-Step "Installing frontend dependencies"
npm install
if ($LASTEXITCODE -ne 0) { throw "npm install failed with exit code $LASTEXITCODE" }

if ($Portable -or $NoBundle) {
    Write-Step "Building the standalone executable"
    # tauri build runs `npm run build` first (beforeBuildCommand), so the
    # webview assets are always current.
    npx tauri build --no-bundle
    if ($LASTEXITCODE -ne 0) {
    throw "tauri build failed with exit code $LASTEXITCODE. Scroll up to the first line starting with 'error:' - that is the real cause; the last line is only the exit."
}
    $exe = Join-Path $repoRoot 'target\release\rusted-shell.exe'
    if (-not (Test-Path $exe)) { throw "build finished but $exe is missing" }
    Write-Host ""
    Write-Host "Executable: $exe" -ForegroundColor Green
    exit 0
}

Write-Step "Building the installer"
# NSIS is the Windows target configured in src-tauri/tauri.conf.json.
npx tauri build --bundles nsis
if ($LASTEXITCODE -ne 0) {
    throw "tauri build failed with exit code $LASTEXITCODE. Scroll up to the first line starting with 'error:' - that is the real cause; the last line is only the exit."
}

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
