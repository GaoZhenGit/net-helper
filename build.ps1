# Build script for net-helper -- dual-platform (Windows + Linux musl)
# Run from project root: .\build.ps1
# Optional version override: .\build.ps1 v2026.12.31.1200

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path

$Ver = $args[0]
$env:NETHELPER_VERSION = if ($Ver -and $Ver -match '^v\d{4}') { $Ver } else { "" }

Write-Host "=== Cleaning build directories ===" -ForegroundColor Cyan
if (Test-Path "$Root\build-win")   { Remove-Item "$Root\build-win"   -Recurse -Force }
if (Test-Path "$Root\build-linux") { Remove-Item "$Root\build-linux" -Recurse -Force }

Write-Host ""
Write-Host "=== Building Windows (MinGW) ===" -ForegroundColor Cyan
cargo build --release --target x86_64-pc-windows-gnu --manifest-path "$Root\Cargo.toml"
New-Item -ItemType Directory -Force -Path "$Root\build-win" | Out-Null
Copy-Item "$Root\target\x86_64-pc-windows-gnu\release\net-helper.exe" "$Root\build-win\net-helper.exe"

Write-Host ""
Write-Host "=== Building Linux (musl) ===" -ForegroundColor Cyan
cargo build --release --target x86_64-unknown-linux-musl --manifest-path "$Root\Cargo.toml"
New-Item -ItemType Directory -Force -Path "$Root\build-linux" | Out-Null
Copy-Item "$Root\target\x86_64-unknown-linux-musl\release\net-helper" "$Root\build-linux\net-helper"

Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Green
Write-Host "Windows: build-win\net-helper.exe"
Write-Host "Linux:   build-linux\net-helper"
if ($Ver) { Write-Host "Version: $Ver" }
