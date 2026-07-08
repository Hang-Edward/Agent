@echo off
title Agent PC

echo === Agent PC ===
echo Building latest version (may take a moment)...
echo.

cd /d "%~dp0"
call npm run tauri dev
pause
