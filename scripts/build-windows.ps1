Param(
    [switch]$SkipNpmInstall = $false
)

$ErrorActionPreference = "Stop"

function Step($msg) {
    Write-Host ""
    Write-Host "==> $msg" -ForegroundColor Cyan
}

function Ensure-Command($name, $hint) {
    if (-not (Get-Command $name -ErrorAction SilentlyContinue)) {
        Write-Host "ERROR: '$name' not found. $hint" -ForegroundColor Red
        exit 1
    }
}

Step "Checking required tools"
Ensure-Command "node" "Install Node.js LTS: https://nodejs.org/"
Ensure-Command "npm.cmd" "npm.cmd comes with Node.js on Windows"
Ensure-Command "npx.cmd" "npx.cmd comes with Node.js on Windows"
Ensure-Command "rustc" "Install Rust: https://rustup.rs/"
Ensure-Command "cargo" "Install Rust: https://rustup.rs/"

Write-Host "node: $(node -v)"
Write-Host "npm:  $(npm.cmd -v)"
Write-Host "rust: $(rustc -V)"

Step "Fixing duplicate capability file (_android.json)"
$capDir = Join-Path (Get-Location) "src-tauri\capabilities"
$androidCompatCap = Join-Path $capDir "_android.json"
if (Test-Path $androidCompatCap) {
    Remove-Item -LiteralPath $androidCompatCap -Force
    Write-Host "Removed duplicate capability file: _android.json"
}

if (-not $SkipNpmInstall) {
    Step "Installing JS dependencies (npm ci)"
    npm.cmd ci
}

Step "Ensuring Windows icon file exists (icon.ico)"
$iconPng = Join-Path (Get-Location) "src-tauri\icons\icon.png"
$iconIco = Join-Path (Get-Location) "src-tauri\icons\icon.ico"
if (-not (Test-Path $iconIco)) {
    npx.cmd tauri icon $iconPng
}
if (-not (Test-Path $iconIco)) {
    Write-Host "ERROR: icon.ico was not generated at $iconIco" -ForegroundColor Red
    exit 1
}

Step "Building Windows bundle via Tauri"
npx.cmd tauri build

Step "Done. Looking for generated installers"
$bundleRoot = Join-Path (Get-Location) "src-tauri\target\release\bundle"
if (-not (Test-Path $bundleRoot)) {
    Write-Host "Bundle directory not found: $bundleRoot" -ForegroundColor Yellow
    exit 0
}

$artifacts = @()
$artifacts += Get-ChildItem -Path $bundleRoot -Recurse -File -Include *.msi,*.exe -ErrorAction SilentlyContinue

if ($artifacts.Count -eq 0) {
    Write-Host "No .msi/.exe found under $bundleRoot" -ForegroundColor Yellow
    Write-Host "Open folder and inspect output manually."
    exit 0
}

Write-Host "Artifacts:" -ForegroundColor Green
$artifacts | Sort-Object FullName | ForEach-Object {
    Write-Host " - $($_.FullName)"
}

