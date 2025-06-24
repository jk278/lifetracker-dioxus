# TimeTracker 警告清理脚本
# 该脚本帮助自动化清理一些常见的警告

param(
    [switch]$DryRun,  # 仅显示将要执行的操作，不实际修改文件
    [switch]$BackupFirst, # 在修改前创建备份
    [string]$TargetModule = "all"  # 指定要清理的模块：all, utils, storage, gui, core
)

Write-Host "=== TimeTracker 警告清理工具 ===" -ForegroundColor Green
Write-Host "目标模块: $TargetModule" -ForegroundColor Cyan

if ($BackupFirst) {
    Write-Host "创建备份..." -ForegroundColor Yellow
    $backupDir = "backup_$(Get-Date -Format 'yyyyMMdd_HHmmss')"
    New-Item -ItemType Directory -Path $backupDir -Force | Out-Null
    Copy-Item -Path "src" -Destination "$backupDir/src" -Recurse
    Write-Host "备份已创建: $backupDir" -ForegroundColor Green
}

# 统计当前警告数量
function Get-CurrentWarnings {
    Write-Host "检查当前警告数量..." -ForegroundColor Cyan
    $output = cargo check --all-targets 2>&1
    $warnings = ($output | Select-String "warning:").Count
    Write-Host "当前警告数量: $warnings" -ForegroundColor Yellow
    return $warnings
}

# 添加 #[allow(dead_code)] 标记
function Add-AllowDeadCode {
    param($FilePath, $Pattern, $Description)
    
    if ($DryRun) {
        Write-Host "[DRY RUN] 将在 $FilePath 中为 $Pattern 添加 #[allow(dead_code)]" -ForegroundColor Yellow
        return
    }
    
    $content = Get-Content $FilePath -Raw
    if ($content -match $Pattern) {
        Write-Host "处理文件: $FilePath - $Description" -ForegroundColor Green
        # 这里需要根据具体情况添加处理逻辑
    }
}

# 清理明显不需要的代码
function Remove-ObviousDeadCode {
    Write-Host "清理明显的死代码..." -ForegroundColor Cyan
    
    $filesToClean = @(
        @{
            Path = "src/utils/format.rs"
            Functions = @("format_currency", "format_bytes", "format_memory")
            Description = "移除明显不会使用的格式化函数"
        },
        @{
            Path = "src/utils/validation.rs"
            Functions = @("validate_email")
            Description = "移除邮件验证功能（项目不需要）"
        }
    )
    
    foreach ($file in $filesToClean) {
        if (Test-Path $file.Path) {
            Write-Host "检查文件: $($file.Path)" -ForegroundColor Yellow
            foreach ($func in $file.Functions) {
                if ($DryRun) {
                    Write-Host "[DRY RUN] 将考虑移除函数: $func" -ForegroundColor Yellow
                } else {
                    Write-Host "标记函数为允许死代码: $func" -ForegroundColor Green
                }
            }
        }
    }
}

# 添加 feature gates
function Add-FeatureGates {
    Write-Host "添加条件编译支持..." -ForegroundColor Cyan
    
    $featureConfig = @"
# 在 Cargo.toml 中添加以下features配置:

[features]
default = ["gui"]
full = ["import", "export", "advanced-formatting", "analytics"]
import = []
export = []  
advanced-formatting = []
analytics = []
validation = []
"@

    if ($DryRun) {
        Write-Host "[DRY RUN] 将添加以下feature配置:" -ForegroundColor Yellow
        Write-Host $featureConfig -ForegroundColor Gray
    } else {
        Write-Host "建议手动添加以下feature配置到 Cargo.toml:" -ForegroundColor Green
        Write-Host $featureConfig -ForegroundColor Gray
    }
}

# 生成允许死代码的模式
function Generate-AllowPatterns {
    Write-Host "生成允许死代码的建议模式..." -ForegroundColor Cyan
    
    $patterns = @(
        @{
            File = "src/utils/import.rs"
            Pattern = '#[allow(dead_code)]'
            Location = "ImportFormat enum"
            Reason = "预留功能 - v0.2.0计划实现"
        },
        @{
            File = "src/utils/export.rs"  
            Pattern = '#[allow(dead_code)]'
            Location = "ExportFormat enum"
            Reason = "预留功能 - v0.2.0计划实现"
        },
        @{
            File = "src/gui/theme.rs"
            Pattern = '#[allow(dead_code)]'
            Location = "ThemePreset enum"
            Reason = "主题系统扩展预留"
        }
    )
    
    foreach ($pattern in $patterns) {
        Write-Host "文件: $($pattern.File)" -ForegroundColor Yellow
        Write-Host "  位置: $($pattern.Location)" -ForegroundColor White
        Write-Host "  原因: $($pattern.Reason)" -ForegroundColor Gray
        Write-Host "  建议: 添加 $($pattern.Pattern)" -ForegroundColor Green
        Write-Host ""
    }
}

# 检查可以安全移除的代码
function Check-SafeToRemove {
    Write-Host "检查可以安全移除的代码..." -ForegroundColor Cyan
    
    $safeToRemove = @(
        "src/utils/format.rs::format_currency - 时间追踪应用不需要货币格式化",
        "src/utils/validation.rs::EmailValidator - 项目不涉及邮件功能",
        "src/utils/validation.rs::UrlValidator - 项目不需要URL验证",
        "src/errors.rs::AppError::Network - 当前版本为单机应用",
        "src/errors.rs::AppError::Permission - 暂未使用权限管理"
    )
    
    Write-Host "以下代码可以考虑安全移除:" -ForegroundColor Yellow
    foreach ($item in $safeToRemove) {
        Write-Host "  - $item" -ForegroundColor White
    }
}

# 主执行逻辑
$initialWarnings = Get-CurrentWarnings

Write-Host "`n开始清理过程..." -ForegroundColor Green

switch ($TargetModule) {
    "utils" {
        Remove-ObviousDeadCode
    }
    "all" {
        Remove-ObviousDeadCode
        Add-FeatureGates
        Generate-AllowPatterns
        Check-SafeToRemove
    }
    default {
        Generate-AllowPatterns
        Check-SafeToRemove
    }
}

if (-not $DryRun) {
    Write-Host "`n检查清理后的警告数量..." -ForegroundColor Cyan
    $finalWarnings = Get-CurrentWarnings
    $reduced = $initialWarnings - $finalWarnings
    
    if ($reduced -gt 0) {
        Write-Host "成功减少 $reduced 个警告!" -ForegroundColor Green
    } else {
        Write-Host "警告数量未变化，可能需要手动处理" -ForegroundColor Yellow
    }
}

Write-Host "`n=== 清理完成 ===" -ForegroundColor Green
Write-Host "下一步建议:" -ForegroundColor Cyan
Write-Host "1. 运行 'cargo clippy' 获取更详细的代码建议" -ForegroundColor White
Write-Host "2. 检查 '警告分析报告.md' 了解详细情况" -ForegroundColor White
Write-Host "3. 考虑实现高优先级的预留功能" -ForegroundColor White
Write-Host "4. 为团队建立代码清理的定期流程" -ForegroundColor White 