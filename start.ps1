# Agent PC — 快速启动脚本
# 用法：在终端运行 .\start.ps1
# 也可直接双击根目录的 start.bat

$appDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$releaseExe = Join-Path $appDir "src-tauri\target\release\agent-pc.exe"

if (Test-Path $releaseExe) {
    Write-Host "Starting Agent PC..." -ForegroundColor Green
    Start-Process -FilePath $releaseExe -WorkingDirectory $appDir
    exit 0
}

Write-Host "Release build not found, starting dev mode..." -ForegroundColor Yellow
Set-Location $appDir
npm run tauri dev
