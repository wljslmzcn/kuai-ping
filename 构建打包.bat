@echo off
chcp 65001 >nul
echo ================================
echo    快Ping - 构建打包
echo ================================
echo.

echo [1/2] 安装前端依赖...
call npm install
if errorlevel 1 (
    echo 依赖安装失败！
    pause
    exit /b 1
)

echo.
echo [2/2] 构建生产版本...
echo.
echo 提示: 构建过程可能需要几分钟，请耐心等待
echo.

call npm run tauri build
if errorlevel 1 (
    echo 构建失败！
    pause
    exit /b 1
)

echo.
echo ================================
echo 构建完成！
echo.
echo 产物位置:
echo   src-tauri\target\release\bundle\
echo ================================
pause
