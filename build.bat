@echo off
chcp 65001 > nul
title TimeTracker 构建工具

echo.
echo ╔══════════════════════════════════════════════════════════════╗
echo ║                    TimeTracker 构建工具                      ║
echo ╚══════════════════════════════════════════════════════════════╝
echo.

if "%1"=="" goto :show_help
if "%1"=="help" goto :show_help
if "%1"=="--help" goto :show_help
if "%1"=="-h" goto :show_help

if "%1"=="build" goto :build
if "%1"=="dist" goto :dist
if "%1"=="clean" goto :clean
if "%1"=="dev" goto :dev

:show_help
echo 📋 可用命令：
echo.
echo   build          构建发布版本
echo   dist           构建并打包到 dist 目录
echo   dev            构建开发版本（调试模式）
echo   clean          清理构建文件
echo   help           显示此帮助信息
echo.
echo 💡 使用示例：
echo   build.bat build    # 构建发布版本
echo   build.bat dist     # 打包分发版本
echo   build.bat dev      # 开发调试版本
echo   build.bat clean    # 清理文件
echo.
goto :end

:build
echo 🔨 构建发布版本...
powershell -ExecutionPolicy Bypass -File .\build.ps1 -Mode gui -Target release
goto :end

:dist
echo 📦 构建并打包到 dist 目录...
powershell -ExecutionPolicy Bypass -File .\build.ps1 -Mode gui -Target release -Package
goto :end

:dev
echo 🛠️ 构建开发版本...
powershell -ExecutionPolicy Bypass -File .\build.ps1 -Mode gui -Target debug
goto :end

:clean
echo 🧹 清理构建文件...
if exist target rmdir /s /q target
if exist dist rmdir /s /q dist
cargo clean
echo ✅ 清理完成！
goto :end

:end
echo.
pause 