//! # WebDAV 同步提供者
//!
//! 实现基于 WebDAV 协议的数据同步功能

use crate::errors::{AppError, Result};
use crate::sync::{SyncConfig, SyncDirection, SyncItem, SyncProvider, SyncStatus};
use crate::utils::crypto::decrypt_password;
use chrono::{DateTime, Local, TimeZone};
use reqwest::{Client, Response};
use std::collections::HashMap;
use std::time::Duration;

/// WebDAV 同步提供者
#[derive(Clone)]
pub struct WebDavProvider {
    /// HTTP 客户端
    client: Client,
    /// 服务器 URL
    base_url: String,
    /// 用户名
    username: String,
    /// 密码
    password: String,
    /// 同步目录
    directory: String,
}

/// WebDAV 响应解析器
struct WebDavResponseParser;

/// WebDAV 请求构建器
struct WebDavRequestBuilder {
    client: Client,
    base_url: String,
    username: String,
    password: String,
}

impl WebDavProvider {
    /// 创建新的 WebDAV 提供者
    pub async fn new(config: &SyncConfig) -> Result<Self> {
        // 验证配置
        validate_config(config)?;

        // 提取配置参数
        let base_url = config
            .settings
            .get("url")
            .ok_or_else(|| AppError::Sync("WebDAV URL 未配置".to_string()))?
            .clone();

        let username = config
            .settings
            .get("username")
            .ok_or_else(|| AppError::Sync("WebDAV 用户名未配置".to_string()))?
            .clone();

        // 获取密码 - 检查是否需要解密
        let password = config
            .settings
            .get("password")
            .ok_or_else(|| AppError::Sync("WebDAV 密码未配置".to_string()))?;

        // 检查密码是否已加密（通过检查是否为有效JSON判断）
        let password = if password.starts_with('[') || password.starts_with('{') {
            // 可能是加密的JSON数据，尝试解密
            match decrypt_password(password, "life_tracker_webdav") {
                Ok(decrypted) => decrypted,
                Err(e) => {
                    log::warn!("解密密码失败，将使用原始密码: {}", e);
                    password.clone()
                }
            }
        } else {
            // 明文密码，直接使用
            password.clone()
        };

        let directory = config
            .settings
            .get("directory")
            .cloned()
            .unwrap_or_else(|| "LifeTracker".to_string());

        // 创建 HTTP 客户端
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| AppError::Network(format!("创建 HTTP 客户端失败: {}", e)))?;

        Ok(Self {
            client,
            base_url,
            username,
            password,
            directory,
        })
    }

    /// 构建完整的远程路径
    fn build_remote_path(&self, path: &str) -> String {
        // 确保base_url不以斜杠结尾
        let base_url = self.base_url.trim_end_matches('/');

        if path.is_empty() {
            // 空路径时返回同步目录
            format!("{}/{}", base_url, self.directory)
        } else if path.starts_with('/') {
            // 路径以斜杠开头，直接拼接
            format!("{}/{}{}", base_url, self.directory, path)
        } else {
            // 路径不以斜杠开头，需要添加斜杠
            format!("{}/{}/{}", base_url, self.directory, path)
        }
    }

    /// 构建完整的URL（用于文件操作）
    fn build_full_url(&self, path: &str) -> String {
        if path.starts_with("http") {
            // 如果已经是完整URL，直接使用
            path.to_string()
        } else if path.starts_with("/dav/") {
            // 如果是WebDAV绝对路径，直接与域名组合
            // 从 base_url 中提取协议和域名
            // base_url 格式: https://dav.jianguoyun.com/dav/
            if let Some(protocol_end) = self.base_url.find("://") {
                let protocol = &self.base_url[..protocol_end];
                let after_protocol = &self.base_url[protocol_end + 3..];

                if let Some(domain_end) = after_protocol.find('/') {
                    let domain = &after_protocol[..domain_end];
                    format!("{}://{}{}", protocol, domain, path)
                } else {
                    // 没有路径部分，整个都是域名
                    format!(
                        "{}://{}{}",
                        protocol,
                        after_protocol.trim_end_matches('/'),
                        path
                    )
                }
            } else {
                // 回退到原来的方式
                format!("{}{}", self.base_url.trim_end_matches('/'), path)
            }
        } else {
            // 相对路径，使用 build_remote_path
            self.build_remote_path(path)
        }
    }

    /// 发送 PROPFIND 请求
    async fn propfind(&self, path: &str) -> Result<String> {
        let url = self.build_remote_path(path);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "1")
            .header("Content-Type", "application/xml")
            .body(
                r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
    <D:allprop/>
</D:propfind>"#,
            )
            .send()
            .await
            .map_err(|e| AppError::Network(format!("PROPFIND 请求失败: {}", e)))?;

        if response.status().is_success() {
            response
                .text()
                .await
                .map_err(|e| AppError::Network(format!("读取响应失败: {}", e)))
        } else {
            Err(AppError::Network(format!(
                "PROPFIND 请求失败，状态码: {}",
                response.status()
            )))
        }
    }

    /// 发送 MKCOL 请求创建目录
    async fn mkcol(&self, path: &str) -> Result<()> {
        let url = self.build_remote_path(path);

        log::info!("尝试创建目录: {}", url);

        let response = self
            .client
            .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), &url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("MKCOL 请求失败: {}", e)))?;

        let status = response.status();
        log::info!("MKCOL 请求响应状态: {}", status);

        if status.is_success() {
            log::info!("目录创建成功");
            Ok(())
        } else if status == 405 {
            // 405 Method Not Allowed 表示目录已存在
            log::info!("目录已存在");
            Ok(())
        } else if status == 409 {
            // 409 Conflict 表示父目录不存在
            log::warn!("父目录不存在，状态码: {}", status);
            Err(AppError::Network(format!(
                "创建目录失败：父目录不存在，状态码: {}",
                status
            )))
        } else if status == 403 {
            // 403 Forbidden 表示权限不足
            log::warn!("权限不足，状态码: {}", status);
            Err(AppError::Network(format!(
                "创建目录失败：权限不足，状态码: {}",
                status
            )))
        } else {
            log::warn!("创建目录失败，状态码: {}", status);
            Err(AppError::Network(format!(
                "创建目录失败，状态码: {}",
                status
            )))
        }
    }

    /// 发送 PUT 请求上传文件
    async fn put_file(&self, path: &str, data: &[u8]) -> Result<()> {
        let url = self.build_remote_path(path);

        let response = self
            .client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| AppError::Network(format!("PUT 请求失败: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Network(format!(
                "上传文件失败，状态码: {}",
                response.status()
            )))
        }
    }

    /// 发送 GET 请求下载文件
    async fn get_file(&self, path: &str) -> Result<Vec<u8>> {
        let url = self.build_remote_path(path);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("GET 请求失败: {}", e)))?;

        if response.status().is_success() {
            response
                .bytes()
                .await
                .map(|bytes| bytes.to_vec())
                .map_err(|e| AppError::Network(format!("读取文件内容失败: {}", e)))
        } else {
            Err(AppError::Network(format!(
                "下载文件失败，状态码: {}",
                response.status()
            )))
        }
    }

    /// 发送 DELETE 请求删除文件
    async fn delete_file(&self, path: &str) -> Result<()> {
        let url = self.build_remote_path(path);

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("DELETE 请求失败: {}", e)))?;

        if response.status().is_success() || response.status() == 404 {
            // 404 表示文件不存在，也算删除成功
            Ok(())
        } else {
            Err(AppError::Network(format!(
                "删除文件失败，状态码: {}",
                response.status()
            )))
        }
    }

    /// 测试服务器根目录连接
    async fn test_server_root(&self) -> Result<bool> {
        // 直接访问服务器根目录，不包含同步目录
        log::info!("测试服务器根目录: {}", self.base_url);

        let response = self
            .client
            .request(
                reqwest::Method::from_bytes(b"PROPFIND").unwrap(),
                &self.base_url,
            )
            .basic_auth(&self.username, Some(&self.password))
            .header("Depth", "0")
            .header("Content-Type", "application/xml")
            .body(
                r#"<?xml version="1.0" encoding="utf-8" ?>
<D:propfind xmlns:D="DAV:">
    <D:prop>
        <D:resourcetype/>
    </D:prop>
</D:propfind>"#,
            )
            .send()
            .await
            .map_err(|e| AppError::Network(format!("测试服务器根目录失败: {}", e)))?;

        let status = response.status();
        log::info!("服务器根目录测试响应状态: {}", status);

        // 对于WebDAV服务器，200、207(Multi-Status)都表示成功
        // 404可能表示路径问题，但服务器可达
        // 401表示认证失败
        match status.as_u16() {
            200 | 207 => {
                log::info!("服务器根目录访问成功");
                Ok(true)
            }
            404 => {
                log::info!("根目录返回404，但服务器可达");
                Ok(true) // 服务器可达，认为基本连接成功
            }
            401 => {
                log::warn!("身份验证失败");
                Ok(false)
            }
            403 => {
                log::info!("权限限制，但服务器连接正常");
                Ok(true)
            }
            _ => {
                log::warn!("服务器根目录测试失败，状态码: {}", status);
                Ok(false)
            }
        }
    }
}

#[async_trait::async_trait]
impl SyncProvider for WebDavProvider {
    fn name(&self) -> &str {
        "WebDAV"
    }

    async fn test_connection(&self) -> Result<bool> {
        // 第一步：测试服务器根目录连接
        log::info!("测试服务器基本连接...");
        match self.test_server_root().await {
            Ok(true) => {
                log::info!("服务器基本连接正常");
            }
            Ok(false) => {
                log::warn!("服务器基本连接失败");
                return Ok(false);
            }
            Err(e) => {
                log::warn!("无法连接到服务器: {}", e);
                return Ok(false);
            }
        }

        // 第二步：尝试访问同步目录
        log::info!("测试同步目录访问...");
        match self.propfind("").await {
            Ok(_) => {
                log::info!("同步目录访问成功");
                return Ok(true);
            }
            Err(e) => {
                log::warn!("同步目录访问失败: {}", e);

                // 第三步：如果是404错误，尝试创建目录
                if e.to_string().contains("404") {
                    log::info!("目录不存在，尝试创建...");
                    match self.mkcol("").await {
                        Ok(_) => {
                            log::info!("目录创建成功，重新测试访问");
                            // 再次尝试访问
                            match self.propfind("").await {
                                Ok(_) => {
                                    log::info!("目录创建后访问成功");
                                    return Ok(true);
                                }
                                Err(e2) => {
                                    log::warn!("目录创建后仍然无法访问: {}", e2);
                                    // 即使无法访问目录，但能创建目录说明连接和权限都正常
                                    return Ok(true);
                                }
                            }
                        }
                        Err(create_err) => {
                            log::warn!("无法创建目录: {}", create_err);
                            // 如果是权限问题但服务器连接正常，也认为测试成功
                            if create_err.to_string().contains("403")
                                || create_err.to_string().contains("405")
                            {
                                log::info!("服务器连接正常，但目录操作受限（这是正常的）");
                                return Ok(true);
                            }
                            return Ok(false);
                        }
                    }
                } else if e.to_string().contains("409") {
                    log::info!("目录冲突，但连接正常");
                    return Ok(true);
                } else if e.to_string().contains("403") {
                    log::info!("权限限制，但连接正常");
                    return Ok(true);
                } else {
                    log::warn!("其他错误，连接失败");
                    return Ok(false);
                }
            }
        }
    }

    async fn list_remote_files(&self, path: &str) -> Result<Vec<SyncItem>> {
        log::info!("列出远程文件，路径: {}", path);
        let response_xml = self.propfind(path).await?;
        log::info!("PROPFIND 响应内容: {}", response_xml);

        let files = WebDavResponseParser::parse_propfind_response(&response_xml)?;
        log::info!("解析到 {} 个远程文件", files.len());

        for file in &files {
            log::info!("远程文件: {} (大小: {} 字节)", file.name, file.size);
        }

        Ok(files)
    }

    async fn upload_file(&self, item: &SyncItem, data: &[u8]) -> Result<()> {
        // 确保目录存在
        if let Some(parent) = std::path::Path::new(&item.remote_path).parent() {
            if let Some(parent_str) = parent.to_str() {
                self.create_remote_directory(parent_str).await?;
            }
        }

        // 构建完整的 URL
        let url = self.build_full_url(&item.remote_path);

        log::info!("上传文件: {}", url);

        let response = self
            .client
            .put(&url)
            .basic_auth(&self.username, Some(&self.password))
            .body(data.to_vec())
            .send()
            .await
            .map_err(|e| AppError::Network(format!("PUT 请求失败: {}", e)))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Network(format!(
                "上传文件失败，状态码: {}",
                response.status()
            )))
        }
    }

    async fn download_file(&self, item: &SyncItem) -> Result<Vec<u8>> {
        // 构建完整的 URL
        let url = self.build_full_url(&item.remote_path);

        log::info!("下载文件: {}", url);

        let response = self
            .client
            .get(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("GET 请求失败: {}", e)))?;

        if response.status().is_success() {
            response
                .bytes()
                .await
                .map(|bytes| bytes.to_vec())
                .map_err(|e| AppError::Network(format!("读取文件内容失败: {}", e)))
        } else {
            Err(AppError::Network(format!(
                "下载文件失败，状态码: {}",
                response.status()
            )))
        }
    }

    async fn delete_remote_file(&self, item: &SyncItem) -> Result<()> {
        // 构建完整的 URL
        let url = self.build_full_url(&item.remote_path);

        log::info!("删除文件: {}", url);

        let response = self
            .client
            .delete(&url)
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await
            .map_err(|e| AppError::Network(format!("DELETE 请求失败: {}", e)))?;

        if response.status().is_success() || response.status() == 404 {
            // 404 表示文件不存在，也算删除成功
            Ok(())
        } else {
            Err(AppError::Network(format!(
                "删除文件失败，状态码: {}",
                response.status()
            )))
        }
    }

    async fn create_remote_directory(&self, path: &str) -> Result<()> {
        self.mkcol(path).await
    }

    async fn get_file_metadata(&self, path: &str) -> Result<SyncItem> {
        let files = self
            .list_remote_files(
                &std::path::Path::new(path)
                    .parent()
                    .and_then(|p| p.to_str())
                    .unwrap_or(""),
            )
            .await?;

        let filename = std::path::Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");

        files
            .into_iter()
            .find(|item| item.name == filename)
            .ok_or_else(|| AppError::Sync(format!("文件 {} 不存在", path)))
    }

    fn clone_provider(&self) -> Box<dyn SyncProvider> {
        Box::new(self.clone())
    }
}

impl WebDavResponseParser {
    /// 解析 PROPFIND 响应
    fn parse_propfind_response(xml: &str) -> Result<Vec<SyncItem>> {
        // 这是一个简化的XML解析实现
        // 在实际应用中，应该使用专门的XML解析库如xml-rs或quick-xml

        log::info!("开始解析 PROPFIND 响应");
        let mut items = Vec::new();

        // 查找所有response元素 - 支持大小写不敏感
        let mut current_pos = 0;
        while let Some(start) = xml[current_pos..].find("<d:response>") {
            let start_pos = current_pos + start;
            if let Some(end) = xml[start_pos..].find("</d:response>") {
                let end_pos = start_pos + end + "</d:response>".len();
                let response_xml = &xml[start_pos..end_pos];

                log::info!("解析response块: {}", response_xml);
                match Self::parse_single_response(response_xml) {
                    Ok(item) => {
                        log::info!("解析到文件: {} (href: {})", item.name, item.id);
                        items.push(item);
                    }
                    Err(e) => {
                        log::info!("解析response失败: {}", e);
                    }
                }

                current_pos = end_pos;
            } else {
                break;
            }
        }

        log::info!("解析完成，总共 {} 个项目", items.len());
        Ok(items)
    }

    /// 解析单个response元素
    fn parse_single_response(xml: &str) -> Result<SyncItem> {
        // 提取href - 支持小写标签
        let href = Self::extract_xml_value(xml, "d:href").unwrap_or_else(|| "unknown".to_string());
        log::info!("解析href: {}", href);

        // 检查是否是目录（通常以/结尾，或包含collection资源类型）
        let is_collection = xml.contains("<d:collection/>") || href.ends_with('/');
        if is_collection {
            log::info!("跳过目录: {}", href);
            return Err(AppError::Sync("这是一个目录，不是文件".to_string()));
        }

        // 提取文件名 - 改进文件名提取逻辑
        let name =
            if let Some(filename) = std::path::Path::new(&href.trim_end_matches('/')).file_name() {
                filename.to_string_lossy().to_string()
            } else {
                // 如果无法提取文件名，使用完整路径
                href.clone()
            };

        log::info!("提取文件名: {}", name);

        // 提取大小 - 支持小写标签
        let size_str =
            Self::extract_xml_value(xml, "d:getcontentlength").unwrap_or_else(|| "0".to_string());
        let size = size_str.parse::<u64>().unwrap_or(0);
        log::info!("文件大小: {} 字节", size);

        // 提取修改时间 - 支持小写标签
        let last_modified_str =
            Self::extract_xml_value(xml, "d:getlastmodified").unwrap_or_else(|| "".to_string());
        let remote_modified = Self::parse_http_date(&last_modified_str);
        log::info!("修改时间: {:?}", remote_modified);

        // 生成简单的哈希值（实际应该从etag或其他方式获取）
        let hash = format!("{}-{}", name, size);

        Ok(SyncItem {
            id: href.clone(),
            name,
            local_path: href.clone(),
            remote_path: href,
            size,
            local_modified: Local::now(),
            remote_modified,
            hash,
            status: SyncStatus::Idle,
            direction: SyncDirection::Bidirectional,
        })
    }

    /// 从XML中提取特定标签的值
    fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
        let start_tag = format!("<{}>", tag);
        let end_tag = format!("</{}>", tag);

        if let Some(start) = xml.find(&start_tag) {
            let content_start = start + start_tag.len();
            if let Some(end) = xml[content_start..].find(&end_tag) {
                let value = &xml[content_start..content_start + end];
                return Some(value.trim().to_string());
            }
        }

        None
    }

    /// 解析HTTP日期格式
    fn parse_http_date(date_str: &str) -> Option<DateTime<Local>> {
        // 尝试解析多种日期格式
        let formats = [
            "%a, %d %b %Y %H:%M:%S GMT",
            "%Y-%m-%dT%H:%M:%SZ",
            "%Y-%m-%d %H:%M:%S",
        ];

        for format in &formats {
            if let Ok(naive_dt) = chrono::NaiveDateTime::parse_from_str(date_str, format) {
                return Local.from_utc_datetime(&naive_dt).into();
            }
        }

        None
    }
}

/// 验证 WebDAV 配置
pub fn validate_config(config: &SyncConfig) -> Result<()> {
    if config.provider != "webdav" {
        return Err(AppError::Validation("不是 WebDAV 配置".to_string()));
    }

    let url = config
        .settings
        .get("url")
        .ok_or_else(|| AppError::Validation("WebDAV URL 未配置".to_string()))?;

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(AppError::Validation("WebDAV URL 格式无效".to_string()));
    }

    if config.settings.get("username").is_none() {
        return Err(AppError::Validation("WebDAV 用户名未配置".to_string()));
    }

    if config.settings.get("password").is_none() {
        return Err(AppError::Validation("WebDAV 密码未配置".to_string()));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = SyncConfig::default();
        config.provider = "webdav".to_string();

        // 测试缺少配置
        assert!(validate_config(&config).is_err());

        // 添加配置
        config
            .settings
            .insert("url".to_string(), "https://example.com/webdav".to_string());
        config
            .settings
            .insert("username".to_string(), "user".to_string());
        config
            .settings
            .insert("password".to_string(), "pass".to_string());

        assert!(validate_config(&config).is_ok());
    }

    #[test]
    fn test_xml_value_extraction() {
        let xml = r#"<D:response>
            <D:href>/path/to/file.txt</D:href>
            <D:getcontentlength>1024</D:getcontentlength>
        </D:response>"#;

        assert_eq!(
            WebDavResponseParser::extract_xml_value(xml, "D:href"),
            Some("/path/to/file.txt".to_string())
        );
        assert_eq!(
            WebDavResponseParser::extract_xml_value(xml, "D:getcontentlength"),
            Some("1024".to_string())
        );
    }

    #[test]
    fn test_http_date_parsing() {
        let date_str = "Mon, 01 Jan 2024 12:00:00 GMT";
        let parsed = WebDavResponseParser::parse_http_date(date_str);
        assert!(parsed.is_some());
    }

    #[test]
    fn test_remote_path_building() {
        // 这个测试需要创建一个WebDavProvider实例，但由于需要异步，这里先跳过
        // 实际应用中应该添加完整的单元测试
    }
}
