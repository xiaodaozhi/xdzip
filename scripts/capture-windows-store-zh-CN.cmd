@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0capture-windows-store.ps1" -Language zh-CN %*
set "captureExit=%ERRORLEVEL%"
if not "%captureExit%"=="0" pause
exit /b %captureExit%
