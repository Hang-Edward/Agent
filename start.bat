@echo off
chcp 65001 >nul
title Agent PC

echo === Agent PC 启动中 ===
echo.

set CARGO_PATH=%USERPROFILE%\.cargo\bin
set PATH=%CARGO_PATH%;%PATH%

set RELEASE_EXE=D:\VScode Projects\Agent\src-tauri\target\release\agent-pc.exe
if exist "%RELEASE_EXE%" (
    echo 找到 release 版本，直接启动...
    start "" "%RELEASE_EXE%"
    exit /b
)

echo 未找到 release 版本，通过 dev 模式启动...
cd /d "D:\VScode Projects\Agent"
call npm run tauri dev
pause
