//! # 加密工具模块
//!
//! 提供密码和敏感信息的加密存储功能

use crate::errors::{AppError, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use ring::rand::{SecureRandom, SystemRandom};
use ring::{aead, pbkdf2};
use std::num::NonZeroU32;

/// 密码加密管理器
pub struct CryptoManager {
    /// 随机数生成器
    rng: SystemRandom,
}

/// 加密后的数据结构
#[derive(Debug, Clone)]
pub struct EncryptedData {
    /// 加密后的数据
    pub ciphertext: String,
    /// 盐值
    pub salt: String,
    /// 随机数
    pub nonce: String,
}

impl CryptoManager {
    /// 创建新的加密管理器
    pub fn new() -> Self {
        Self {
            rng: SystemRandom::new(),
        }
    }

    /// 加密密码
    ///
    /// # 参数
    /// * `password` - 要加密的密码
    /// * `master_key` - 主密钥（通常是应用标识符）
    ///
    /// # 返回值
    /// 返回加密后的数据结构
    pub fn encrypt_password(&self, password: &str, master_key: &str) -> Result<EncryptedData> {
        // 生成盐值
        let mut salt = [0u8; 16];
        self.rng
            .fill(&mut salt)
            .map_err(|e| AppError::Crypto(format!("生成盐值失败: {}", e)))?;

        // 使用PBKDF2派生密钥
        let mut key_bytes = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(100_000).unwrap(),
            &salt,
            master_key.as_bytes(),
            &mut key_bytes,
        );

        // 创建加密密钥
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
            .map_err(|e| AppError::Crypto(format!("创建加密密钥失败: {}", e)))?;
        let key = aead::LessSafeKey::new(key);

        // 生成随机数
        let mut nonce_bytes = [0u8; 12];
        self.rng
            .fill(&mut nonce_bytes)
            .map_err(|e| AppError::Crypto(format!("生成随机数失败: {}", e)))?;
        let nonce = aead::Nonce::assume_unique_for_key(nonce_bytes);

        // 加密数据
        let mut plaintext = password.as_bytes().to_vec();
        key.seal_in_place_append_tag(nonce, aead::Aad::empty(), &mut plaintext)
            .map_err(|e| AppError::Crypto(format!("加密数据失败: {}", e)))?;

        // 编码为base64
        let ciphertext = BASE64.encode(&plaintext);
        let salt_b64 = BASE64.encode(&salt);
        let nonce_b64 = BASE64.encode(&nonce_bytes);

        Ok(EncryptedData {
            ciphertext,
            salt: salt_b64,
            nonce: nonce_b64,
        })
    }

    /// 解密密码
    ///
    /// # 参数
    /// * `encrypted_data` - 加密后的数据
    /// * `master_key` - 主密钥（通常是应用标识符）
    ///
    /// # 返回值
    /// 返回解密后的密码
    pub fn decrypt_password(
        &self,
        encrypted_data: &EncryptedData,
        master_key: &str,
    ) -> Result<String> {
        // 解码base64
        let ciphertext = BASE64
            .decode(&encrypted_data.ciphertext)
            .map_err(|e| AppError::Crypto(format!("解码密文失败: {}", e)))?;
        let salt = BASE64
            .decode(&encrypted_data.salt)
            .map_err(|e| AppError::Crypto(format!("解码盐值失败: {}", e)))?;
        let nonce_bytes = BASE64
            .decode(&encrypted_data.nonce)
            .map_err(|e| AppError::Crypto(format!("解码随机数失败: {}", e)))?;

        // 重新派生密钥
        let mut key_bytes = [0u8; 32];
        pbkdf2::derive(
            pbkdf2::PBKDF2_HMAC_SHA256,
            NonZeroU32::new(100_000).unwrap(),
            &salt,
            master_key.as_bytes(),
            &mut key_bytes,
        );

        // 创建解密密钥
        let key = aead::UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
            .map_err(|e| AppError::Crypto(format!("创建解密密钥失败: {}", e)))?;
        let key = aead::LessSafeKey::new(key);

        // 构建随机数
        let mut nonce_array = [0u8; 12];
        nonce_array.copy_from_slice(&nonce_bytes);
        let nonce = aead::Nonce::assume_unique_for_key(nonce_array);

        // 解密数据
        let mut ciphertext_mut = ciphertext;
        let plaintext = key
            .open_in_place(nonce, aead::Aad::empty(), &mut ciphertext_mut)
            .map_err(|e| AppError::Crypto(format!("解密数据失败: {}", e)))?;

        // 转换为字符串
        String::from_utf8(plaintext.to_vec())
            .map_err(|e| AppError::Crypto(format!("解密后数据格式错误: {}", e)))
    }

    /// 生成安全的随机密钥
    ///
    /// # 参数
    /// * `length` - 密钥长度（字节）
    ///
    /// # 返回值
    /// 返回base64编码的随机密钥
    pub fn generate_random_key(&self, length: usize) -> Result<String> {
        let mut key = vec![0u8; length];
        self.rng
            .fill(&mut key)
            .map_err(|e| AppError::Crypto(format!("生成随机密钥失败: {}", e)))?;
        Ok(BASE64.encode(&key))
    }

    /// 计算数据的哈希值
    ///
    /// # 参数
    /// * `data` - 要计算哈希的数据
    ///
    /// # 返回值
    /// 返回十六进制格式的哈希值
    pub fn calculate_hash(&self, data: &[u8]) -> String {
        let digest = md5::compute(data);
        format!("{:x}", digest)
    }

    /// 验证数据完整性
    ///
    /// # 参数
    /// * `data` - 要验证的数据
    /// * `expected_hash` - 期望的哈希值
    ///
    /// # 返回值
    /// 返回验证结果
    pub fn verify_hash(&self, data: &[u8], expected_hash: &str) -> bool {
        let actual_hash = self.calculate_hash(data);
        actual_hash == expected_hash
    }
}

impl Default for CryptoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 加密密码的便捷函数
pub fn encrypt_password(password: &str, master_key: &str) -> Result<String> {
    let crypto = CryptoManager::new();
    let encrypted = crypto.encrypt_password(password, master_key)?;

    // 将加密数据序列化为JSON字符串
    let json_str =
        serde_json::to_string(&(&encrypted.ciphertext, &encrypted.salt, &encrypted.nonce))
            .map_err(|e| AppError::Crypto(format!("序列化加密数据失败: {}", e)))?;

    Ok(json_str)
}

/// 解密密码的便捷函数
pub fn decrypt_password(encrypted_json: &str, master_key: &str) -> Result<String> {
    let crypto = CryptoManager::new();

    // 检查输入是否为空或仅包含空白字符
    if encrypted_json.trim().is_empty() {
        return Err(AppError::Crypto("加密数据为空".to_string()));
    }

    // 尝试反序列化加密数据，添加更详细的错误处理
    let (ciphertext, salt, nonce): (String, String, String) =
        match serde_json::from_str(encrypted_json) {
            Ok(data) => data,
            Err(e) => {
                log::warn!("反序列化加密数据失败: {}, 数据: {}", e, encrypted_json);
                return Err(AppError::Crypto(format!(
                    "加密数据格式无效，可能需要重新设置密码。错误详情: {}",
                    e
                )));
            }
        };

    // 验证反序列化后的数据完整性
    if ciphertext.trim().is_empty() || salt.trim().is_empty() || nonce.trim().is_empty() {
        return Err(AppError::Crypto(
            "加密数据不完整，请重新设置密码".to_string(),
        ));
    }

    let encrypted_data = EncryptedData {
        ciphertext,
        salt,
        nonce,
    };

    // 尝试解密，如果失败则给出更友好的错误信息
    crypto
        .decrypt_password(&encrypted_data, master_key)
        .map_err(|e| {
            log::warn!("解密密码失败: {}", e);
            AppError::Crypto("密码解密失败，请检查密码是否正确或重新设置".to_string())
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_password() {
        let crypto = CryptoManager::new();
        let password = "test_password_123";
        let master_key = "life_tracker_app";

        // 加密
        let encrypted = crypto.encrypt_password(password, master_key).unwrap();

        // 解密
        let decrypted = crypto.decrypt_password(&encrypted, master_key).unwrap();

        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_convenience_functions() {
        let password = "test_password_123";
        let master_key = "life_tracker_app";

        // 加密
        let encrypted = encrypt_password(password, master_key).unwrap();

        // 解密
        let decrypted = decrypt_password(&encrypted, master_key).unwrap();

        assert_eq!(password, decrypted);
    }

    #[test]
    fn test_hash_verification() {
        let crypto = CryptoManager::new();
        let data = b"test data";

        let hash = crypto.calculate_hash(data);
        assert!(crypto.verify_hash(data, &hash));
        assert!(!crypto.verify_hash(b"different data", &hash));
    }

    #[test]
    fn test_generate_random_key() {
        let crypto = CryptoManager::new();

        let key1 = crypto.generate_random_key(32).unwrap();
        let key2 = crypto.generate_random_key(32).unwrap();

        // 确保生成的密钥不同
        assert_ne!(key1, key2);

        // 确保密钥长度正确（base64编码后的长度）
        assert!(!key1.is_empty());
        assert!(!key2.is_empty());
    }
}
