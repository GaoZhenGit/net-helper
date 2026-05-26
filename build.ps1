# Build script for net-helper -- dual-platform (Windows + Linux musl)
# Run from project root: .\build.ps1 [-Clean] [version]

$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $MyInvocation.MyCommand.Path)

$Clean = $args -contains '-Clean'
$Ver  = ($args | Where-Object { $_ -match '^(v?\d+\.\d+\.\d+|v\d{4}\.\d{2}\.\d{2}\.\d{4})$' } | Select-Object -First 1)
if ($Ver) { $env:NETHELPER_VERSION = $Ver } else { $env:NETHELPER_VERSION = $null }

$winTarget  = "x86_64-pc-windows-gnu"
$linuxTarget = "x86_64-unknown-linux-musl"

if ($Clean) {
    Write-Host "=== Cleaning project artifacts (keeping dependencies) ===" -ForegroundColor Cyan
    cargo clean -p net-helper --release --target $winTarget
    Write-Host "  Removed target\$winTarget\release\net-helper*"
    cargo clean -p net-helper --release --target $linuxTarget
    Write-Host "  Removed target\$linuxTarget\release\net-helper*"
}

Write-Host "`n=== Windows ===" -ForegroundColor Cyan
cargo build --release --target $winTarget
Copy-Item "target\$winTarget\release\net-helper.exe" "target\net-helper.exe" -Force

Write-Host "`n=== Linux (musl) ===" -ForegroundColor Cyan
cargo zigbuild --release --target $linuxTarget
Copy-Item "target\$linuxTarget\release\net-helper" "target\net-helper" -Force

Write-Host "`n=== Done ===" -ForegroundColor Green
Write-Host "target\net-helper.exe"
Write-Host "target\net-helper"
if ($Ver) { Write-Host "Version: $Ver" }
