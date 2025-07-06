# LifeTracker Android 构建配置指南

## Theme

- 亮色模式 `res/values/themes.xml` 和
```xml
<!-- 状态栏透明 -->
<item name="android:statusBarColor">@android:color/transparent</item>
<!-- 导航栏透明 -->
<item name="android:navigationBarColor">@android:color/transparent</item>
<!-- 状态栏图标颜色适配 -->
<item name="android:windowLightStatusBar">true</item>
```

- 暗色模式 `res/values-night/themes.xml`
```xml
<!-- 状态栏透明 -->
<item name="android:statusBarColor">@android:color/transparent</item>
<!-- 导航栏透明 -->
<item name="android:navigationBarColor">@android:color/transparent</item>
<!-- 状态栏图标颜色适配 -->
<item name="android:windowLightStatusBar">false</item>
```

---

## Build signed apk

- 生成密钥库 (密钥密码默认与密钥库密码相同)
```powershell
keytool -genkey -v -keystore life-tracker-key.keystore -alias life-tracker -keyalg RSA -keysize 2048 -validity 10000
```

- 修改 `src-tauri/gen/android/app/build.gradle.kts`
```Kotlin Script
android {
    signingConfigs {
        create("release") {
            storeFile = file("../../../life-tracker-key.keystore")
            storePassword = "your-keystore-password"
            keyAlias = "life-tracker"
            keyPassword = "your-key-password"
        }
    }
    buildTypes {
        release {
            signingConfig = signingConfigs.getByName("release")
        }
    }
}
```

---

## Icon

```powershell
pnpm tauri icon app-icon.svg
```

### Adaptive icon

- 配置结构
```
gen/android/app/src/main/res/
├── mipmap-anydpi-v26/
│   └── ic_launcher.xml                    # 主适应性图标配置
├── drawable/
│   └── ic_launcher_background.xml         # 背景层（渐变蓝色）
└── drawable-v24/
    └── ic_launcher_foreground.xml         # 前景层（时钟图标）
```

- 图标配置 `res\mipmap-anydpi-v26\ic_launcher.xml`
```xml
<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
    <background android:drawable="@drawable/ic_launcher_background"/>
    <foreground android:drawable="@drawable/ic_launcher_foreground"/>
</adaptive-icon> 
```

- 背景层 `res\drawable\ic_launcher_background.xml`
```xml
<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp"
    android:height="108dp"
    android:viewportWidth="108"
    android:viewportHeight="108">
    <!-- LifeTracker 背景色 - 使用渐变蓝色 -->
    <path
        android:fillColor="#4F46E5"
        android:pathData="M0,0h108v108h-108z" />
    <path
        android:fillColor="#6366F1"
        android:pathData="M0,0L108,108 L0,108 Z" />
</vector>
```

- 前景层 `res\drawable-v24\ic_launcher_foreground.xml`
```xml
<?xml version="1.0" encoding="utf-8"?>
<vector xmlns:android="http://schemas.android.com/apk/res/android"
    android:width="108dp"
    android:height="108dp"
    android:viewportWidth="108"
    android:viewportHeight="108">
    <!-- LifeTracker 前景图标 - 简化的时钟图标 -->
    <group android:scaleX="0.7"
           android:scaleY="0.7"
           android:pivotX="54"
           android:pivotY="54">
        <!-- 表盘外环 -->
        <path
            android:fillColor="#FFFFFF"
            android:pathData="M54,25A28.5,28.5 0 1,1 54,82A28.5,28.5 0 1,1 54,25M54,30A23.5,23.5 0 1,0 54,77A23.5,23.5 0 1,0 54,30"/>
        
        <!-- 时钟指针 -->
        <path
            android:fillColor="#FFFFFF"
            android:pathData="M52,30L56,30L56,53.5L70,53.5L70,57.5L52,57.5Z"/>
        
        <!-- 中心点 -->
        <path
            android:fillColor="#FFFFFF"
            android:pathData="M54,50.5A3,3 0 1,1 54,56.5A3,3 0 1,1 54,50.5"/>
        
        <!-- 12点和6点刻度 -->
        <path
            android:fillColor="#FFFFFF"
            android:pathData="M52,25L56,25L56,35L52,35Z"/>
        <path
            android:fillColor="#FFFFFF"
            android:pathData="M52,72L56,72L56,82L52,82Z"/>
        
        <!-- 3点和9点刻度 -->
        <path
            android:fillColor="#FFFFFF"
            android:pathData="M25,52L35,52L35,56L25,56Z"/>
        <path
            android:fillColor="#FFFFFF"
            android:pathData="M72,52L82,52L82,56L72,56Z"/>
    </group>
</vector>
```
