@echo off
echo === Inkwell GPU Server - Windows Setup ===
echo.
echo This script will:
echo   1. Check for Rust, CMake, and Visual Studio Build Tools
echo   2. Build the GPU server
echo   3. Create a desktop shortcut
echo.
echo Press any key to continue or Ctrl+C to cancel...
pause > nul

powershell -ExecutionPolicy Bypass -File "%~dp0install-windows.ps1"
pause
