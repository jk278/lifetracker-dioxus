# TimeTracker 构建脚本
# 支持构建不同版本：GUI版本（不显示控制台）和CLI版本（显示控制台）

param(
    [string]$Mode = "gui",      # gui 或 cli
    [string]$Target = "release", # release 或 debug
    [switch]$Clean = $false,
    [switch]$Package = $false   # 是否打包到 dist 目录
)

$Version = "1.0.0"  # 可以从 Cargo.toml 读取

# 清理构建文件
if ($Clean) {
    Write-Host "🧹 清理构建文件..." -ForegroundColor Yellow
    cargo clean
    if (Test-Path "dist") { Remove-Item "dist" -Recurse -Force }
}

# 设置构建参数
$BuildFlags = @()
$Features = @()

switch ($Mode.ToLower()) {
    "gui" {
        Write-Host "🖥️ 构建GUI版本（无控制台窗口）..." -ForegroundColor Green
        $Features += "gui"
        $BuildFlags += "--features", "gui"
    }
    "cli" {
        Write-Host "⌨️ 构建CLI版本（显示控制台）..." -ForegroundColor Green
        $Features += "cli-only"
        $BuildFlags += "--features", "cli-only"
        $BuildFlags += "--no-default-features"
    }
    default {
        Write-Host "⚠️ 未知模式: $Mode，使用默认GUI模式" -ForegroundColor Yellow
        $Features += "gui"
        $BuildFlags += "--features", "gui"
    }
}

# 设置目标配置
if ($Target.ToLower() -eq "release") {
    $BuildFlags += "--release"
    Write-Host "🚀 构建发布版本..." -ForegroundColor Blue
} else {
    Write-Host "🛠️ 构建调试版本..." -ForegroundColor Blue
}

# 执行构建
Write-Host "🔨 执行构建命令: cargo build $($BuildFlags -join ' ')" -ForegroundColor Cyan
$BuildResult = & cargo build @BuildFlags

# 检查构建结果
if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ 构建失败！" -ForegroundColor Red
    exit 1
}

$ExePath = if ($Target.ToLower() -eq "release") { "target\release\time_tracker.exe" } else { "target\debug\time_tracker.exe" }
Write-Host "✅ 构建成功！" -ForegroundColor Green
Write-Host "📄 可执行文件位置: $ExePath" -ForegroundColor Green

if ($Mode.ToLower() -eq "gui") {
    Write-Host "💡 注意: GUI版本在发布模式下不会显示控制台窗口" -ForegroundColor Yellow
} else {
    Write-Host "💡 注意: CLI版本会显示控制台窗口" -ForegroundColor Yellow
}

# 显示文件信息
if (Test-Path $ExePath) {
    $FileInfo = Get-Item $ExePath
    Write-Host "📊 文件大小: $([math]::Round($FileInfo.Length / 1MB, 2)) MB" -ForegroundColor Cyan
    Write-Host "⏰ 创建时间: $($FileInfo.CreationTime)" -ForegroundColor Cyan
}

# 打包到 dist 目录
if ($Package) {
    Write-Host ""
    Write-Host "📦 开始打包到 dist 目录..." -ForegroundColor Green
    
    # 创建 dist 目录
    $DistDir = "dist"
    if (!(Test-Path $DistDir)) {
        New-Item -ItemType Directory -Path $DistDir | Out-Null
    }
    
    # 确定包名
    $PackageName = "TimeTracker-v$Version-$($Mode.ToUpper())"
    $PackageDir = Join-Path $DistDir $PackageName
    
    # 清理旧的包目录
    if (Test-Path $PackageDir) {
        Remove-Item $PackageDir -Recurse -Force
    }
    
    # 创建包目录
    New-Item -ItemType Directory -Path $PackageDir | Out-Null
    
    # 复制可执行文件
    $TargetExe = Join-Path $PackageDir "TimeTracker.exe"
    Copy-Item $ExePath $TargetExe
    Write-Host "📄 复制可执行文件: TimeTracker.exe" -ForegroundColor Gray
    
    # 复制文档文件
    if (Test-Path "README.md") {
        Copy-Item "README.md" $PackageDir
        Write-Host "📄 复制文档: README.md" -ForegroundColor Gray
    }
    
    # 复制图标
    if (Test-Path "assets\icon.png") {
        Copy-Item "assets\icon.png" $PackageDir
        Write-Host "📄 复制图标: icon.png" -ForegroundColor Gray
    }
    
    # 创建数据目录
    $DataDir = Join-Path $PackageDir "data"
    New-Item -ItemType Directory -Path $DataDir | Out-Null
    Write-Host "📁 创建数据目录: data\" -ForegroundColor Gray
    
    # 创建启动脚本
    $StartScript = Join-Path $PackageDir "启动.bat"
    @"
@echo off
chcp 65001 > nul
title TimeTracker v$Version
echo 🚀 启动 TimeTracker v$Version...
echo.
TimeTracker.exe
if errorlevel 1 (
    echo.
    echo ❌ 程序异常退出，按任意键查看帮助...
    pause > nul
    TimeTracker.exe --help
    pause
)
"@ | Out-File $StartScript -Encoding ASCII
    Write-Host "📄 创建启动脚本: 启动.bat" -ForegroundColor Gray
    
    # 创建使用说明
    $ReadmeFile = Join-Path $PackageDir "使用说明.txt"
    @"
TimeTracker v$Version $($Mode.ToUpper())版本
===============================

🚀 快速开始
-----------
双击 "启动.bat" 启动应用程序
或者直接双击 "TimeTracker.exe"

📋 命令行使用
-----------
TimeTracker.exe --help      查看帮助
TimeTracker.exe --version   查看版本
TimeTracker.exe --gui       启动图形界面

💾 数据存储
-----------
所有数据保存在 data\ 目录下，可随程序一起移动。

📁 文件说明
-----------
- TimeTracker.exe    主程序
- 启动.bat          启动器脚本  
- data\             数据目录
- README.md         详细文档
- 使用说明.txt      本文件

版本信息
--------
版本: v$Version
模式: $($Mode.ToUpper())
打包时间: $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')
"@ | Out-File $ReadmeFile -Encoding UTF8
    Write-Host "📄 创建使用说明: 使用说明.txt" -ForegroundColor Gray
    
    # 创建压缩包
    $ZipFile = Join-Path $DistDir "$PackageName.zip"
    if (Test-Path $ZipFile) {
        Remove-Item $ZipFile -Force
    }
    
    Write-Host "🗜️ 创建压缩包..." -ForegroundColor Yellow
    Compress-Archive -Path "$PackageDir\*" -DestinationPath $ZipFile -CompressionLevel Optimal
    
    # 显示结果
    Write-Host ""
    Write-Host "🎉 打包完成！" -ForegroundColor Green
    Write-Host "📁 包目录: $PackageDir" -ForegroundColor Cyan
    
    if (Test-Path $ZipFile) {
        $ZipSize = [math]::Round((Get-Item $ZipFile).Length / 1MB, 2)
        Write-Host "🗜️ 压缩包: $ZipFile ($ZipSize MB)" -ForegroundColor Cyan
    }
    
    # 显示dist目录内容
    Write-Host ""
    Write-Host "📦 dist 目录内容:" -ForegroundColor Yellow
    Get-ChildItem $DistDir | ForEach-Object {
        $size = if ($_.PSIsContainer) { 
            "[目录]" 
        } else { 
            "$([math]::Round($_.Length / 1MB, 2)) MB" 
        }
        Write-Host "  📄 $($_.Name) $size" -ForegroundColor Gray
    }
}

# 使用示例提示
Write-Host ""
Write-Host "💡 使用示例:" -ForegroundColor Yellow
Write-Host "  .\build.ps1 -Mode gui -Target release         # 构建GUI版本" -ForegroundColor Gray
Write-Host "  .\build.ps1 -Mode gui -Target release -Package # 构建并打包" -ForegroundColor Gray
Write-Host "  .\build.ps1 -Mode cli -Target release -Package # 构建CLI版本并打包" -ForegroundColor Gray
Write-Host ""
Write-Host "📋 或使用简化命令:" -ForegroundColor Yellow
Write-Host "  build.bat build    # 构建发布版本" -ForegroundColor Gray
Write-Host "  build.bat dist     # 构建并打包" -ForegroundColor Gray
Write-Host "  build.bat dev      # 开发版本" -ForegroundColor Gray
Write-Host "  build.bat clean    # 清理文件" -ForegroundColor Gray 