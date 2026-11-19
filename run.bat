@echo off
title AeroVM - Trình quản lý máy ảo phong cách UTM
cd /d "%~dp0"

echo ============================================================
echo      AeroVM - Trình quản lý máy ảo phong cách UTM
echo      100%% Rust Backend - Tang toc WHPX Hypervisor
echo ============================================================
echo [*] Dang khoi dong AeroVM...

if exist "%~dp0aerovm.exe" (
    "%~dp0aerovm.exe"
) else if exist "%~dp0target\release\aerovm.exe" (
    "%~dp0target\release\aerovm.exe"
) else (
    echo [!] Khong tim thay tep aerovm.exe!
    echo [*] Vui long kiem tra lai thu muc giai nen hoac chay: cargo build --release
    pause
)
