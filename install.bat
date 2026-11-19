@echo off
setlocal EnableDelayedExpansion
title AeroVM Setup - Trinh cai dat AeroVM
cd /d "%~dp0"

echo ======================================================================
echo              AeroVM Setup - Trinh Cai Dat Ung Dung May Ao UTM
echo ======================================================================
echo.

set "TARGET_DIR=%LOCALAPPDATA%\Programs\AeroVM"
set "DESKTOP_DIR=%USERPROFILE%\Desktop"
set "START_MENU=%APPDATA%\Microsoft\Windows\Start Menu\Programs"

echo [*] Thu muc cai dat rieng: "!TARGET_DIR!"
if not exist "!TARGET_DIR!" mkdir "!TARGET_DIR!"

echo [*] Dang sao chep tap tin chuong trinh vao thu muc rieng...
if exist "aerovm.exe" (
    copy /y "aerovm.exe" "!TARGET_DIR!\aerovm.exe" >nul
) else if exist "target\release\aerovm.exe" (
    copy /y "target\release\aerovm.exe" "!TARGET_DIR!\aerovm.exe" >nul
) else (
    echo [!] Khong tim thay tap tin aerovm.exe!
    pause
    exit /b 1
)

if exist "README.md" copy /y "README.md" "!TARGET_DIR!\README.md" >nul
if exist "uninstall.bat" copy /y "uninstall.bat" "!TARGET_DIR!\uninstall.bat" >nul
if exist "run.bat" copy /y "run.bat" "!TARGET_DIR!\run.bat" >nul

echo [*] Dang tao Shortcut tren Desktop va Start Menu...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$ws = New-Object -ComObject WScript.Shell; $s1 = $ws.CreateShortcut(\"$env:USERPROFILE\Desktop\AeroVM.lnk\"); $s1.TargetPath = \"$env:LOCALAPPDATA\Programs\AeroVM\aerovm.exe\"; $s1.WorkingDirectory = \"$env:LOCALAPPDATA\Programs\AeroVM\"; $s1.Description = \"AeroVM - UTM Style Virtual Machines\"; $s1.Save(); $sm = \"$env:APPDATA\Microsoft\Windows\Start Menu\Programs\"; if (Test-Path $sm) { $s2 = $ws.CreateShortcut(\"$sm\AeroVM.lnk\"); $s2.TargetPath = \"$env:LOCALAPPDATA\Programs\AeroVM\aerovm.exe\"; $s2.WorkingDirectory = \"$env:LOCALAPPDATA\Programs\AeroVM\"; $s2.Save() }"

echo.
echo ======================================================================
echo   CAI DAT THANH CONG!
echo   - Thu muc ung dung: "!TARGET_DIR!"
echo   - Da tao bieu tuong AeroVM tren Desktop va Start Menu.
echo   - Ban co the nhap dup vao Shortcut AeroVM tren Desktop de su dung.
echo     (Chay ngam hoan toan 100%%, 0 cua so console den)
echo ======================================================================
echo.

set "START_NOW=Y"
set /p START_NOW="Ban co muon khoi dong AeroVM ngay bay gio khong? (Y/N): "
if /i "!START_NOW!"=="Y" (
    start "" "!TARGET_DIR!\aerovm.exe"
)
