@echo off
:: TimeTracker Makefile (Windows)
:: 标准化构建命令，类似于 make

if "%1"=="" (
    echo 使用方法: make [命令]
    echo.
    echo 可用命令:
    echo   all       构建所有版本
    echo   build     构建发布版本  
    echo   dev       构建开发版本
    echo   dist      打包分发版本
    echo   clean     清理构建文件
    echo   test      运行测试
    echo   check     检查代码
    echo   help      显示帮助
    goto :eof
)

if "%1"=="all" goto :all
if "%1"=="build" goto :build  
if "%1"=="dev" goto :dev
if "%1"=="dist" goto :dist
if "%1"=="clean" goto :clean
if "%1"=="test" goto :test
if "%1"=="check" goto :check
if "%1"=="help" goto :help

echo 未知命令: %1
goto :eof

:all
echo 构建所有版本...
call :build
call :dist
goto :eof

:build
cargo build --release --features gui
goto :eof

:dev  
cargo build --features gui
goto :eof

:dist
powershell -ExecutionPolicy Bypass -File .\build.ps1 -Mode gui -Target release -Package
goto :eof

:clean
cargo clean
if exist dist rmdir /s /q dist
goto :eof

:test
cargo test
goto :eof

:check
cargo check --features gui
goto :eof

:help
echo TimeTracker 构建系统
echo.
echo 可用命令:
echo   make all      构建所有版本
echo   make build    构建发布版本
echo   make dev      构建开发版本  
echo   make dist     打包分发版本
echo   make clean    清理构建文件
echo   make test     运行测试
echo   make check    检查代码
echo   make help     显示帮助
goto :eof 