# Build script for net-helper -- dual-platform (Windows + Linux musl)
# Run from project root: .\build.ps1
# Optional version override:    .\build.ps1 v2026.12.31.1200

$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $MyInvocation.MyCommand.Path

# Optional version override (e.g. .\build.ps1 v2026.05.22.1500)
$Ver = $args[0]
$VersionArg = if ($Ver -and $Ver -match '^v\d{4}') { "-DNETHELPER_VERSION=$Ver" } else { "" }

Write-Host "=== Cleaning build directories ===" -ForegroundColor Cyan
if (Test-Path "$Root\build-win")   { Remove-Item "$Root\build-win"   -Recurse -Force }
if (Test-Path "$Root\build-linux") { Remove-Item "$Root\build-linux" -Recurse -Force }

Write-Host ""
Write-Host "=== Building Windows (MinGW) ===" -ForegroundColor Cyan
cmake -B "$Root\build-win" -G "MinGW Makefiles" -S "$Root" $VersionArg
cmake --build "$Root\build-win"

Write-Host ""
Write-Host "=== Building Linux (musl via Zig) ===" -ForegroundColor Cyan
cmake -B "$Root\build-linux" -G "MinGW Makefiles" -S "$Root" `
    -DCMAKE_SYSTEM_NAME=Linux `
    -DCMAKE_SYSTEM_PROCESSOR=x86_64 `
    -DCMAKE_CXX_COMPILER="zig.exe" `
    -DCMAKE_CXX_COMPILER_ARG1="c++" `
    -DCMAKE_CXX_COMPILER_TARGET="x86_64-linux-musl" `
    $VersionArg
cmake --build "$Root\build-linux"

Write-Host ""
Write-Host "=== Done ===" -ForegroundColor Green
Write-Host "Windows: build-win\net-helper.exe"
Write-Host "Linux:   build-linux\net-helper"
if ($Ver) { Write-Host "Version: $Ver" }
