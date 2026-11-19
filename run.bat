@echo off
title AeroVM Runner
chcp 65001 >nul
cd /d "%~dp0"

if exist "%~dp0aerovm.exe" (
    start "" "%~dp0aerovm.exe"
) else if exist "%~dp0target\release\aerovm.exe" (
    start "" "%~dp0target\release\aerovm.exe"
) else (
    echo [!] Khong tim thay tep aerovm.exe!
    echo [*] Ban co the chay install.bat de cai dat hoac cargo build --release
    pause
)
