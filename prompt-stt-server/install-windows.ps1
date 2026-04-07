# Inkwell GPU Server - Windows 11 Installation Script
# Run in PowerShell as Administrator

Write-Host "=== Inkwell GPU Server - Windows Setup ===" -ForegroundColor Cyan

# 1. Check prerequisites
Write-Host "`n[1/5] Checking prerequisites..." -ForegroundColor Yellow

# Check Rust
if (!(Get-Command rustc -ErrorAction SilentlyContinue)) {
    Write-Host "Installing Rust..." -ForegroundColor Yellow
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y --default-toolchain stable
    $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
    Write-Host "Rust installed!" -ForegroundColor Green
} else {
    Write-Host "Rust OK: $(rustc --version)" -ForegroundColor Green
}

# Check Git
if (!(Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "ERROR: Git is required. Install from https://git-scm.com/download/win" -ForegroundColor Red
    exit 1
} else {
    Write-Host "Git OK" -ForegroundColor Green
}

# Check CMake (needed for whisper.cpp)
if (!(Get-Command cmake -ErrorAction SilentlyContinue)) {
    Write-Host "Installing CMake via winget..." -ForegroundColor Yellow
    winget install Kitware.CMake --accept-package-agreements --accept-source-agreements
    $env:PATH = "C:\Program Files\CMake\bin;$env:PATH"
} else {
    Write-Host "CMake OK" -ForegroundColor Green
}

# Check Visual Studio Build Tools (needed for C/C++ compilation)
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    $vsPath = & $vsWhere -latest -property installationPath
    Write-Host "Visual Studio OK: $vsPath" -ForegroundColor Green
} else {
    Write-Host "Installing Visual Studio Build Tools..." -ForegroundColor Yellow
    Write-Host "This is needed to compile whisper.cpp (C++ library)" -ForegroundColor Yellow
    winget install Microsoft.VisualStudio.2022.BuildTools --accept-package-agreements --accept-source-agreements --override "--wait --passive --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
    Write-Host "Build Tools installed. You may need to restart PowerShell." -ForegroundColor Green
}

# 2. Clone or update repo
Write-Host "`n[2/5] Getting source code..." -ForegroundColor Yellow
$repoDir = "$env:USERPROFILE\inkwell-gpu-server"

if (Test-Path "$repoDir\.git") {
    Write-Host "Updating existing repo..." -ForegroundColor Yellow
    Push-Location $repoDir
    git pull
    Pop-Location
} else {
    Write-Host "Cloning repository..." -ForegroundColor Yellow
    git clone https://github.com/Louisdelez/CodingPromptes.git "$env:TEMP\inkwell-repo"
    # Copy only the GPU server
    if (Test-Path $repoDir) { Remove-Item -Recurse -Force $repoDir }
    Copy-Item -Recurse "$env:TEMP\inkwell-repo\prompt-stt-server" $repoDir
    # Copy assets
    if (Test-Path "$env:TEMP\inkwell-repo\prompt-stt-server\assets") {
        Copy-Item -Recurse "$env:TEMP\inkwell-repo\prompt-stt-server\assets" "$repoDir\assets" -Force
    }
}

# 3. Build
Write-Host "`n[3/5] Building (this may take 5-10 minutes on first build)..." -ForegroundColor Yellow
Push-Location $repoDir
cargo build --release 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed! Check errors above." -ForegroundColor Red
    Pop-Location
    exit 1
}
Pop-Location
Write-Host "Build successful!" -ForegroundColor Green

# 4. Create shortcut on Desktop
Write-Host "`n[4/5] Creating desktop shortcut..." -ForegroundColor Yellow
$exePath = "$repoDir\target\release\prompt-ai-server.exe"
$shortcutPath = "$env:USERPROFILE\Desktop\Inkwell GPU Server.lnk"

$shell = New-Object -ComObject WScript.Shell
$shortcut = $shell.CreateShortcut($shortcutPath)
$shortcut.TargetPath = $exePath
$shortcut.WorkingDirectory = $repoDir
$shortcut.Description = "Inkwell GPU Server"
$shortcut.Save()
Write-Host "Shortcut created on Desktop!" -ForegroundColor Green

# 5. Done
Write-Host "`n[5/5] Installation complete!" -ForegroundColor Green
Write-Host ""
Write-Host "To launch: double-click 'Inkwell GPU Server' on your Desktop" -ForegroundColor Cyan
Write-Host "Or run: $exePath" -ForegroundColor Cyan
Write-Host ""
Write-Host "After launch:" -ForegroundColor Yellow
Write-Host "  1. In the 'Account' card, enter your server URL (e.g. http://192.168.1.x:8910)" -ForegroundColor White
Write-Host "  2. Login with your Inkwell account" -ForegroundColor White
Write-Host "  3. Give your node a name (e.g. 'PC Bureau RTX 3060')" -ForegroundColor White
Write-Host "  4. Click Connect" -ForegroundColor White
Write-Host ""
Write-Host "The GPU server will appear in the 'GPU' tab of your Inkwell web app." -ForegroundColor White

# Ask to launch
$launch = Read-Host "`nLaunch now? (Y/n)"
if ($launch -ne 'n') {
    Start-Process $exePath -WorkingDirectory $repoDir
}
