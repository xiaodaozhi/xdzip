@echo off
powershell.exe -NoProfile -ExecutionPolicy Bypass -File "%~dp0capture-windows-store.ps1" -Language en-US %*
set "captureExit=%ERRORLEVEL%"
pause
exit /b %captureExit%
