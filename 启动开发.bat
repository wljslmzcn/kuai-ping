@echo off
chcp 65001 >nul
echo ================================
echo    快Ping - 开发环境启动
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
echo [2/2] 启动Tauri开发模式...
echo.
echo 提示: 首次运行需要编译Rust，可能需要几分钟
echo.

call npm run tauri dev
pause
