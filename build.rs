// build.rs - 构建脚本

fn main() {
    // Windows 平台的资源文件配置
    #[cfg(target_os = "windows")]
    {
        if std::env::var("CARGO_CFG_TARGET_OS").is_ok() {
            let mut res = winres::WindowsResource::new();

            // 只设置存在的文件
            if std::path::Path::new("assets/icon.ico").exists() {
                res.set_icon("assets/icon.ico");
            }

            // 简化语言设置
            res.set_language(0x0804); // 中文简体

            if let Err(e) = res.compile() {
                println!("cargo:warning=无法编译Windows资源: {}", e);
            }
        }
    }

    println!("cargo:rerun-if-changed=assets/");
}
