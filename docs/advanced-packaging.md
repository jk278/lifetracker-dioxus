# 高级打包选项

## 🎯 **专业安装包**

### Windows MSI 安装包
```powershell
# 安装工具
cargo install cargo-wix

# 生成配置
cargo wix init

# 构建 MSI
cargo wix --no-build --nocapture
```

### 跨平台打包
```bash
# 安装 cargo-bundle
cargo install cargo-bundle

# Windows
cargo bundle --release
# 输出: target/release/bundle/msi/TimeTracker.msi

# macOS (在 Mac 上运行)
cargo bundle --release  
# 输出: target/release/bundle/osx/TimeTracker.app

# Linux
cargo bundle --release
# 输出: target/release/bundle/deb/timetracker_1.0.0_amd64.deb
```

### Linux 包管理器
```bash
# DEB 包 (Ubuntu/Debian)
cargo install cargo-deb
cargo deb

# RPM 包 (CentOS/RHEL/Fedora)
cargo install cargo-rpm  
cargo rpm build

# AppImage (通用 Linux)
# 需要额外配置 AppImage 工具
```

## 🌐 **跨平台编译**

### 添加编译目标
```bash
# Windows (从其他平台编译)
rustup target add x86_64-pc-windows-gnu
cargo build --release --target x86_64-pc-windows-gnu

# macOS (从其他平台编译)  
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Linux (从其他平台编译)
rustup target add x86_64-unknown-linux-gnu  
cargo build --release --target x86_64-unknown-linux-gnu
```

## 📋 **分发清单**

### 当前已完成 ✅
- [x] Windows 便携版 (.zip)
- [x] 完整中文支持
- [x] GUI + CLI 双模式
- [x] 用户友好的启动脚本
- [x] 详细使用文档

### 可选扩展 📦
- [ ] Windows MSI 安装包
- [ ] macOS .dmg 安装包  
- [ ] Linux .deb/.rpm 包
- [ ] 代码签名证书
- [ ] 自动更新机制

## 🎯 **推荐分发策略**

### 个人用户
- **便携版** - 最简单，解压即用

### 企业用户  
- **MSI 安装包** - 支持组策略部署
- **静默安装** - 批量部署友好

### 开源发布
- **GitHub Releases** - 多平台自动构建
- **包管理器** - 用户安装便利 