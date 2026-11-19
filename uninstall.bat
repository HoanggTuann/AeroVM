@echo off
title AeroVM Uninstaller - Go cai dat AeroVM
chcp 65001 >nul

echo ======================================================================
echo          🗑️ AeroVM - Trình Gỡ Cài Đặt
echo ======================================================================
echo.

set "INSTALL_DIR=%LOCALAPPDATA%\Programs\AeroVM"
set "DESKTOP_DIR=%USERPROFILE%\Desktop"
set "START_MENU=%APPDATA%\Microsoft\Windows\Start Menu\Programs"

echo [*] Dang dung cac tien trinh AeroVM dang chay...
taskkill /f /im aerovm.exe >nul 2>&1

echo [*] Dang xoa loi tat Desktop va Start Menu...
if exist "%DESKTOP_DIR%\AeroVM.lnk" del /f /q "%DESKTOP_DIR%\AeroVM.lnk"
if exist "%START_MENU%\AeroVM.lnk" del /f /q "%START_MENU%\AeroVM.lnk"

echo [*] Dang xoa thu muc ung dung...
if exist "%INSTALL_DIR%" (
    rmdir /s /q "%INSTALL_DIR%"
)

echo.
echo ======================================================================
echo   ✅ ĐÃ GỠ CÀI ĐẶT THÀNH CÔNG!
echo   (Lưu ý: Dữ liệu đĩa ảo máy ảo tại ~/.aerovm/vms vẫn được giữ an toàn).
echo ======================================================================
pause
