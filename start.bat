@echo off
title Agent PC

echo === Agent PC ===
echo.

set "APP_DIR=%~dp0"
set "RELEASE_EXE=%APP_DIR%src-tauri\target\release\agent-pc.exe"
if exist "%RELEASE_EXE%" (
    echo Quick launch mode...
    start "" "%RELEASE_EXE%"
    exit /b
)

echo Dev mode (first launch may take a few minutes)...
cd /d "%APP_DIR%"
call npm run tauri dev
pause
