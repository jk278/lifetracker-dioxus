# LifeTracker Android 开发环境设置指南

## 0. 前置准备（Windows 用户）

### 开启 Windows 开发者模式

1. 按 `Win + I` 打开 Windows 设置
2. 导航至 "隐私和安全性" > "开发者选项"
3. 开启 "开发者模式"
4. 重启计算机使设置生效

开发者模式启用后，Windows 将支持：
- 符号链接创建（Tauri 构建必需）
- 侧载应用和调试
- 开发者工具完整访问权限

## 1. 安装 Android 开发工具

### 必需工具
1. **Java Development Kit (JDK) 21** - 手动安装
   - 下载 JDK 21: https://adoptium.net/zh-CN/temurin/releases/
   - 推荐使用 Eclipse Temurin JDK 21 (LTS)
   - 安装完成后记录安装路径，例如：`C:\Program Files\Eclipse Adoptium\jdk-21.0.7.6-hotspot`

2. **Android Studio** - 下载并安装最新版本
   - 访问 https://developer.android.com/studio
   - 安装时选择 "Standard" 安装类型
   - 确保安装了 Android SDK 和 Android Virtual Device (AVD)
   - **注意**：Tauri V2 不支持 Android SDK 35，如未找到支持的版本，开发时会自动在SDK目录下载（可能产生网络错误）

3. **Android SDK 和工具**
   - Android SDK Platform-Tools
   - Android SDK Build-Tools (最新版本)
   - Android SDK Platform API 33 或更高版本
   - Android NDK (通过 SDK Manager 安装)

### 设置环境变量

在系统环境变量中添加以下路径：

```powershell
# 设置 JAVA_HOME（根据您的 JDK 安装路径调整）
$env:JAVA_HOME = "C:\Program Files\Eclipse Adoptium\jdk-21.0.7.6-hotspot"

# 设置 Android 相关路径
$env:ANDROID_HOME = "C:\Users\$env:USERNAME\AppData\Local\Android\Sdk"
$env:NDK_HOME = "$env:ANDROID_HOME\ndk\29.0.13599879"

# 更新 PATH 环境变量
$env:PATH = "$env:JAVA_HOME\bin;$env:ANDROID_HOME\platform-tools;$env:ANDROID_HOME\cmdline-tools\latest\bin;$env:PATH"
```

**永久设置环境变量：**
1. 右键点击 "此电脑" > "属性"
2. 点击 "高级系统设置"
3. 点击 "环境变量"
4. 在 "系统变量" 中添加：
   - `JAVA_HOME`: `C:\Program Files\Eclipse Adoptium\jdk-21.0.7.6-hotspot`
   - `ANDROID_HOME`: `C:\Users\你的用户名\AppData\Local\Android\Sdk`
   - `NDK_HOME`: `%ANDROID_HOME%\ndk\29.0.13599879`
5. 在 "系统变量" 中编辑 `PATH`，添加：
   - `%JAVA_HOME%\bin`
   - `%ANDROID_HOME%\platform-tools`
   - `%ANDROID_HOME%\cmdline-tools\latest\bin`

## 2. 验证安装

**重启 PowerShell 或重新启动计算机后再进行验证**

运行以下命令验证安装：

```powershell
# 检查 Java
java -version
javac -version
echo $env:JAVA_HOME

# 检查 Android SDK
adb version

# 检查 Rust 目标
rustup target list | findstr android
```

**预期输出示例：**
```
> java -version
openjdk version "21.0.7" 2025-04-15 LTS
OpenJDK Runtime Environment Temurin-21.0.7+6 (build 21.0.7+6-LTS)
OpenJDK 64-Bit Server VM Temurin-21.0.7+6 (build 21.0.7+6-LTS, mixed mode, sharing)
```

## 3. 安装 Rust Android 目标

```powershell
# 安装 Android 目标
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android
```

## 4. 项目结构修改 (可以在安装 Rust Android 前)

为了支持移动端，需要将项目从二进制包转换为库包：

1. 修改 `Cargo.toml` 添加 `crate-type = ["cdylib", "bin"]`
2. 重命名 `src/main.rs` 为 `src/lib.rs`
3. 创建新的 `src/main.rs` 作为桌面入口点
4. 添加移动端特定的 Tauri 配置

## 5. 开发流程

### 初始化 Android 项目
```powershell
pnpm tauri android init
```

### 开发模式
```powershell
# 启动开发服务器
pnpm tauri android dev

# 指定设备
pnpm tauri android dev --device <device-id>
```

### 构建发布版本
```powershell
pnpm tauri android build

# 构建 APK
pnpm tauri android build --apk

# 构建 AAB (推荐用于 Play Store)
pnpm tauri android build --aab
```

## 6. 代码签名设置

### 生成密钥库
```powershell
keytool -genkey -v -keystore life-tracker-key.keystore -alias life-tracker -keyalg RSA -keysize 2048 -validity 10000
```

### 配置签名
在 `src-tauri/gen/android/app/build.gradle` 中配置签名信息。

## 7. 权限配置

在 `capabilities/default.json` 中配置 Android 权限：

```json
{
  "identifier": "main-capability",
  "description": "Capability for main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-open",
    "dialog:default",
    "notification:default",
    "fs:default"
  ]
}
```

## 8. 常见问题解决

### JAVA_HOME 相关问题
- **错误**: `ERROR: JAVA_HOME is not set and no 'java' command could be found in your PATH`
- **解决**: 
  1. 确保已安装 JDK 21（不是 JRE）
  2. 正确设置 JAVA_HOME 环境变量
  3. 将 `%JAVA_HOME%\bin` 添加到 PATH
  4. 重启 PowerShell 或重新启动计算机

### Android Studio JDK 配置
- 在 Android Studio 中设置 Gradle JDK：
  1. 打开 `File` > `Settings` > `Build, Execution, Deployment` > `Build Tools` > `Gradle`
  2. 设置 `Gradle JDK` 为您安装的 JDK 21 路径
  3. 或选择 `JAVA_HOME` 宏

### 环境变量问题
- 确保 `ANDROID_HOME` 和 `NDK_HOME` 正确设置
- 重启 PowerShell 或 IDE 以应用环境变量
- 使用 `echo $env:JAVA_HOME` 验证变量是否正确设置

### 构建失败
- 检查 Android SDK 版本兼容性
- 确保 NDK 版本正确
- 清理并重新构建：`pnpm tauri android build --clean`

### 设备连接问题
- 启用开发者模式和 USB 调试
- 使用 `adb devices` 检查设备连接状态

## 9. 性能优化

### 应用大小优化
- 启用 ProGuard/R8 混淆
- 使用 Android App Bundle (AAB)
- 优化资源文件

### 运行时性能
- 使用 Android 性能分析工具
- 优化数据库查询
- 合理使用异步操作

## 10. 发布准备

### Play Store 发布
1. 创建 Play Console 开发者账户
2. 准备应用图标和截图
3. 编写应用描述
4. 设置定价和分发
5. 上传 AAB 文件

### 权限说明
为应用的权限使用提供清晰的说明，特别是文件系统访问和通知权限。

## 下一步

完成环境设置后，按照以下顺序进行：
1. 运行项目结构修改脚本
2. 初始化 Android 项目
3. 配置权限和功能
4. 测试构建和运行
5. 优化和发布准备 