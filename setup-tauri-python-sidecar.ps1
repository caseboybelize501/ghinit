# setup-tauri-python-sidecar.ps1
# Purpose: Build Python FastAPI sidecar with PyInstaller, place it correctly for Tauri,
#          update tauri.conf.json, add basic sidecar spawn code to main.rs,
#          clean & test run cargo tauri dev

$ErrorActionPreference = "Stop"
$projectRoot = "D:\Users\CASE\Projects\purposeforge"
$backendDir  = Join-Path $projectRoot "backend"
$tauriDir    = Join-Path $projectRoot "src-tauri"
$binDir      = Join-Path $tauriDir "bin"

Write-Host "======================================" -ForegroundColor Cyan
Write-Host " PurposeForge Tauri + Python Sidecar Setup" -ForegroundColor Cyan
Write-Host "======================================" -ForegroundColor Cyan

# 1. Change to project root (just in case)
Set-Location $projectRoot

# 2. Check prerequisites
if (-not (Test-Path $backendDir)) {
    Write-Error "backend folder not found. Run bootstrap.ps1 first."
    exit 1
}

if (-not (Test-Path (Join-Path $backendDir "main.py"))) {
    Write-Error "main.py not found in backend/. Something went wrong with bootstrap."
    exit 1
}

# 3. Build Python sidecar with PyInstaller
Write-Host "`n[1/5] Building Python sidecar (purposeforge-backend.exe)..." -ForegroundColor Yellow
Set-Location $backendDir

# Activate venv if it exists
$venvScripts = Join-Path $backendDir ".venv\Scripts\Activate.ps1"
if (Test-Path $venvScripts) {
    Write-Host "Activating virtual environment..."
    & $venvScripts
} else {
    Write-Warning "No .venv found in backend/. Using global Python."
}

# Make sure pyinstaller is installed
pip install --quiet pyinstaller

# Build
pyinstaller --onefile --name purposeforge-backend main.py

# Check if build succeeded
$exePath = Join-Path $backendDir "dist\purposeforge-backend.exe"
if (-not (Test-Path $exePath)) {
    Write-Error "PyInstaller failed to create purposeforge-backend.exe"
    exit 1
}

# 4. Prepare bin folder and copy/rename for Tauri (Windows target)
Write-Host "`n[2/5] Copying sidecar to src-tauri\bin..." -ForegroundColor Yellow
if (-not (Test-Path $binDir)) {
    New-Item -ItemType Directory -Path $binDir | Out-Null
}

$targetExeName = "purposeforge-backend-x86_64-pc-windows-msvc.exe"
$targetExePath = Join-Path $binDir $targetExeName

Copy-Item -Path $exePath -Destination $targetExePath -Force
Write-Host "  Copied → $targetExeName"

# 5. Update tauri.conf.json (backup first)
Write-Host "`n[3/5] Updating src-tauri/tauri.conf.json..." -ForegroundColor Yellow
$confPath = Join-Path $tauriDir "tauri.conf.json"

if (-not (Test-Path $confPath)) {
    Write-Error "tauri.conf.json not found in src-tauri/"
    exit 1
}

# Backup original
Copy-Item -Path $confPath -Destination "$confPath.bak" -Force

# Read JSON
$conf = Get-Content $confPath -Raw | ConvertFrom-Json

# Ensure tauri → bundle → externalBin exists and add our sidecar
if (-not $conf.tauri) { $conf | Add-Member -NotePropertyName "tauri" -NotePropertyValue ([PSCustomObject]@{}) }
if (-not $conf.tauri.bundle) { $conf.tauri | Add-Member -NotePropertyName "bundle" -NotePropertyValue ([PSCustomObject]@{}) }
if (-not $conf.tauri.bundle.externalBin) { $conf.tauri.bundle | Add-Member -NotePropertyName "externalBin" -NotePropertyValue @() }

$existing = $conf.tauri.bundle.externalBin
$newPath = "bin/$targetExeName"
if ($existing -notcontains $newPath) {
    $conf.tauri.bundle.externalBin += $newPath
    Write-Host "  Added externalBin: $newPath"
} else {
    Write-Host "  externalBin already contains the sidecar path"
}

# Write back (pretty print)
$conf | ConvertTo-Json -Depth 100 | Set-Content $confPath -Encoding UTF8
Write-Host "  tauri.conf.json updated."

# 6. Add sidecar spawn code to main.rs (only if not already present)
Write-Host "`n[4/5] Adding sidecar spawn code to src-tauri/src/main.rs..." -ForegroundColor Yellow
$mainRsPath = Join-Path $tauriDir "src\main.rs"

if (-not (Test-Path $mainRsPath)) {
    Write-Error "src-tauri/src/main.rs not found"
    exit 1
}

$mainRsContent = Get-Content $mainRsPath -Raw

# Simple check: look for "new_sidecar" or "purposeforge-backend"
if ($mainRsContent -notmatch "purposeforge-backend" -and $mainRsContent -notmatch "new_sidecar") {
    # Basic insertion before tauri::Builder
    $spawnCode = @"
// Sidecar spawn (added by setup script)
use tauri::api::process::{Command, CommandEvent};
use tauri::Manager;

"@
    $newContent = $mainRsContent -replace '(tauri::Builder::default\(\))', "$spawnCode`$1"

    $spawnSetup = @"
        .setup(move |app| {
            let window = app.get_window("main").unwrap();

            tauri::async_runtime::spawn(async move {
                let (mut rx, _child) = Command::new_sidecar("purposeforge-backend")
                    .expect("failed to create sidecar command")
                    .spawn()
                    .expect("failed to spawn backend sidecar");

                while let Some(event) = rx.recv().await {
                    if let CommandEvent::Stdout(line) = event {
                        let _ = window.emit("backend-log", line);
                    }
                }
            });

            Ok(())
        })
"@

    $newContent = $newContent -replace '\.setup\s*\(\s*\|app\|.*?\)\s*\{.*?\}\s*', $spawnSetup

    Set-Content $mainRsPath -Value $newContent -Encoding UTF8
    Write-Host "  Added basic sidecar spawn code to main.rs"
} else {
    Write-Host "  Sidecar spawn code already appears to exist in main.rs — skipping"
}

# 7. Clean & run dev
Write-Host "`n[5/5] Cleaning build and starting Tauri dev..." -ForegroundColor Yellow
Set-Location $projectRoot
cargo clean
cargo tauri dev

Write-Host "`nDone!" -ForegroundColor Green
Write-Host "If it still crashes, check:"
Write-Host "  - Task Manager: kill any purposeforge.exe / python.exe leftovers"
Write-Host "  - Browser console in the Tauri window (right-click → Inspect)"
Write-Host "  - Terminal output for Rust panics or sidecar errors"
Write-Host "You can now access backend at http://localhost:8000 from frontend (adjust fetch URLs)"