
/// ============================================================
/// 持久化登录密码加密模块
/// ============================================================
/// 
/// Android/非Windows平台使用 Base64 编码（降级方案）
/// 后续可接入 Android Keystore 提升安全性

use base64::{Engine as _, engine::general_purpose};

/// 加密密码（Base64 编码）
pub fn encrypt_password_dpapi(password: &str) -> Result<String, String> {
    Ok(general_purpose::STANDARD.encode(password.as_bytes()))
}

/// 解密密码（Base64 解码）
pub fn decrypt_password_dpapi(encrypted_base64: &str) -> Result<String, String> {
    let bytes = general_purpose::STANDARD.decode(encrypted_base64)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    
    String::from_utf8(bytes)
        .map_err(|e| format!("UTF-8 解码失败: {}", e))
}
