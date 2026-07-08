# Agent PC — 快速启动脚本
# 用法：在终端运行 .\start.ps1，或右键 → 使用 PowerShell 运行

Write-Host "=== Agent PC 启动中 ===" -ForegroundColor Cyan
Write-Host ""

# 检查前置条件
$cargoPath = "$env:USERPROFILE\.cargo\bin"
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    $env:PATH = "$cargoPath;$env:PATH"
    if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
        Write-Host "[错误] 未找到 Rust/Cargo，请先安装 Rust。" -ForegroundColor Red
        exit 1
    }
}

# 检查 release 版本是否存在（直接运行更快）
$releaseExe = "D:\VScode Projects\Agent\src-tauri\target\release\agent-pc.exe"
if (Test-Path $releaseExe) {
    Write-Host "找到 release 版本，直接启动..." -ForegroundColor Green
    Write-Host "提示：关闭窗口即可退出。" -ForegroundColor Yellow
    Start-Process -FilePath $releaseExe -WorkingDirectory "D:\VScode Projects\Agent"
    exit 0
}

# 不存在则通过 dev 模式构建并运行
Write-Host "未找到 release 版本，通过 dev 模式启动（首次编译可能需要几分钟）..." -ForegroundColor Yellow
Set-Location "D:\VScode Projects\Agent"
$env:PATH = "$cargoPath;$env:PATH"
npm run tauri dev
