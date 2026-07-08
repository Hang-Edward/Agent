$appDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Write-Host "=== Agent PC ===" -ForegroundColor Cyan
Write-Host "Building latest version..." -ForegroundColor Yellow
Set-Location $appDir
npm run tauri dev
