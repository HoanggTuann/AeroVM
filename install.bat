@echo off
title AeroVM Setup - Trinh cai dat AeroVM
chcp 65001 >nul
cd /d "%~dp0"

echo ======================================================================
echo          🚀 AeroVM Setup - Trình Cài Đặt Ứng Dụng Máy Ảo UTM
echo ======================================================================
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\Programs\AeroVM"
set "DESKTOP_DIR=%USERPROFILE%\Desktop"
set "START_MENU=%APPDATA%\Microsoft\Windows\Start Menu\Programs"

echo [*] Thu muc cai dat: "%INSTALL_DIR%"
echo [*] Dang tao thu muc cai dat rieng...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

echo [*] Dang sao chep tap tin chuong trinh...
if exist "aerovm.exe" (
    copy /y "aerovm.exe" "%INSTALL_DIR%\" >nul
) else if exist "target\release\aerovm.exe" (
    copy /y "target\release\aerovm.exe" "%INSTALL_DIR%\" >nul
) else (
    echo [!] Khong tim thay tap tin aerovm.exe!
    pause
    exit /b 1
)

copy /y "README.md" "%INSTALL_DIR%\" >nul 2>&1
copy /y "uninstall.bat" "%INSTALL_DIR%\" >nul 2>&1

echo [*] Dang tao loi tat (Shortcut) tren Desktop va Start Menu...
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$WshShell = New-Object -ComObject WScript.Shell; ^
    $Shortcut = $WshShell.CreateShortcut('%DESKTOP_DIR%\AeroVM.lnk'); ^
    $Shortcut.TargetPath = '%INSTALL_DIR%\aerovm.exe'; ^
    $Shortcut.WorkingDirectory = '%INSTALL_DIR%'; ^
    $Shortcut.Description = 'AeroVM - Trình Quản Lý Máy Ảo Phong Cách UTM'; ^
    $Shortcut.Save(); ^
    $StartShortcut = $WshShell.CreateShortcut('%START_MENU%\AeroVM.lnk'); ^
    $StartShortcut.TargetPath = '%INSTALL_DIR%\aerovm.exe'; ^
    $StartShortcut.WorkingDirectory = '%INSTALL_DIR%'; ^
    $StartShortcut.Save();"

echo.
echo ======================================================================
echo   🎉 CÀI ĐẶT THÀNH CÔNG!
echo   - Ứng dụng đã được cài đặt vào thư mục riêng biệt:
echo     "%INSTALL_DIR%"
echo   - Đã tạo biểu tượng AeroVM trên Desktop (Màn hình chính).
echo   - Từ nay bạn chỉ cần nhấp đúp vào icon "AeroVM" trên Desktop!
echo     (Chạy ngầm hoàn toàn, 0 cửa sổ console đen, giao diện UTM mượt mà)
echo ======================================================================
echo.

set /p START_NOW="Bạn có muốn khởi động AeroVM ngay bây giờ không? (Y/N): "
if /i "%START_NOW%"=="Y" (
    start "" "%INSTALL_DIR%\aerovm.exe"
)
