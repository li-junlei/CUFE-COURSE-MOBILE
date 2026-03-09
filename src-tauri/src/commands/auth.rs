// 认证模块 - 登录、登出、会话管理相关命令

use crate::client::EduSystemState;
use crate::crypto::{decrypt_password_dpapi, encrypt_password_dpapi};
use crate::models::{UserCredentials, UserInfo, PersistentCredentials, SessionStatus};
use crate::storage::StorageManager;
use std::sync::Arc;

fn is_session_expired_error(error: &str) -> bool {
    error.contains("登录已失效")
        || error.contains("请重新登录")
        || error.contains("未找到登录信息")
        || error.contains("需要学号信息")
        || error.contains("无法获取用户信息")
        || error.contains("HTML页面")
}

/// 登录并获取用户信息（不获取课表）
#[tauri::command]
pub async fn login_and_get_user_info(
    state: tauri::State<'_, Arc<EduSystemState>>,
    username: String,
    password: String,
) -> Result<UserInfo, String> {
    let credentials = UserCredentials {
        username: username.clone(),
        password,
    };

    // 初始化全局 client
    state.initialize_client(username.clone());
    let mut client = state.get_client()?;

    // 执行登录
    let login_response = client.login(&credentials).await?;
    if !login_response.success {
        // 登录失败，清除状态
        state.logout();
        return Err(format!("登录失败: {}", login_response.message));
    }

    // 获取用户信息
    let user_info = client.get_user_info().await?;

    Ok(user_info)
}

/// 退出登录（旧版本兼容）
#[tauri::command]
pub fn logout_user(state: tauri::State<'_, Arc<EduSystemState>>) -> Result<(), String> {
    // 清除全局客户端状态（自动清除 cookies）
    state.logout();
    println!("已退出登录");
    Ok(())
}

/// 登录并保存凭证
#[tauri::command]
pub async fn login_and_save_credentials(
    state: tauri::State<'_, Arc<EduSystemState>>,
    username: String,
    password: String,
) -> Result<UserInfo, String> {
    let credentials = UserCredentials {
        username: username.clone(),
        password,
    };

    // 初始化客户端并登录
    state.initialize_client(username.clone());
    let mut client = state.get_client()?;

    let login_response = client.login(&credentials).await?;
    if !login_response.success {
        state.logout();
        return Err(format!("登录失败: {}", login_response.message));
    }

    // 获取用户信息
    let user_info = client.get_user_info().await?;

    // 加密密码
    let encrypted_password = encrypt_password_dpapi(&credentials.password)
        .map_err(|e| format!("密码加密失败: {}", e))?;

    // 保存凭证
    let storage = StorageManager::new()?;
    let persistent_creds = PersistentCredentials {
        username,
        password_encrypted: encrypted_password,
        edu_system_url: "https://xuanke.cufe.edu.cn/jwglxt/".to_string(),
        saved_at: chrono::Utc::now().timestamp(),
    };

    storage.save_credentials(&persistent_creds)
        .map_err(|e| format!("保存凭证失败: {}", e))?;

    println!("凭证已保存到本地");
    Ok(user_info)
}

/// 恢复登录会话（应用启动时调用）
#[tauri::command]
pub async fn restore_login_session(
    state: tauri::State<'_, Arc<EduSystemState>>,
) -> Result<UserInfo, String> {
    let storage = StorageManager::new()?;

    // 加载保存的凭证
    let creds = storage.load_credentials()?;

    // 解密密码
    let password = decrypt_password_dpapi(&creds.password_encrypted)
        .map_err(|e| format!("密码解密失败: {}", e))?;

    // 重新登录
    state.initialize_client(creds.username.clone());
    let mut client = state.get_client()?;

    let credentials = UserCredentials {
        username: creds.username,
        password,
    };

    let login_response = client.login(&credentials).await?;
    if !login_response.success {
        state.logout();
        storage.clear_credentials()?; // 清除无效凭证
        return Err("保存的凭证已失效，请重新登录".to_string());
    }

    // 获取用户信息
    let user_info = client.get_user_info().await?;

    println!("已自动恢复登录状态");
    Ok(user_info)
}

/// 获取当前登录用户信息（不重新登录）
#[tauri::command]
pub async fn get_current_user_info(
    state: tauri::State<'_, Arc<EduSystemState>>,
) -> Result<Option<UserInfo>, String> {
    // 检查是否已登录
    if !state.is_logged_in() {
        return Ok(None);
    }

    // 获取全局 client（复用登录时的会话）
    let client = state.get_client()?;

    // 获取用户信息（复用现有会话，不需要重新登录）
    let user_info = client.get_user_info().await?;
    Ok(Some(user_info))
}

/// 退出登录并清除所有凭证
#[tauri::command]
pub async fn logout_and_clear(
    state: tauri::State<'_, Arc<EduSystemState>>,
) -> Result<(), String> {
    // 清除内存状态
    state.logout();

    // 清除保存的凭证
    let storage = StorageManager::new()?;
    storage.clear_credentials()?;

    println!("已退出登录并清除所有凭证");
    Ok(())
}

/// 检查是否已登录
#[tauri::command]
pub fn is_logged_in(state: tauri::State<'_, Arc<EduSystemState>>) -> bool {
    state.is_logged_in()
}

/// 检查登录会话，如已失效则尝试后台自动重登录
#[tauri::command]
pub async fn ensure_login_session(
    state: tauri::State<'_, Arc<EduSystemState>>,
) -> Result<SessionStatus, String> {
    let storage = StorageManager::new()?;

    if state.is_logged_in() {
        let client = state.get_client()?;
        match client.get_user_info().await {
            Ok(user_info) => {
                return Ok(SessionStatus {
                    logged_in: true,
                    relogin_performed: false,
                    user_info: Some(user_info),
                });
            }
            Err(error) if !is_session_expired_error(&error) => {
                return Err(error);
            }
            Err(error) => {
                eprintln!("检测到登录会话失效，准备自动重登录: {}", error);
                state.logout();
            }
        }
    }

    let creds = match storage.load_credentials() {
        Ok(creds) => creds,
        Err(_) => {
            return Ok(SessionStatus {
                logged_in: false,
                relogin_performed: false,
                user_info: None,
            });
        }
    };

    let password = decrypt_password_dpapi(&creds.password_encrypted)
        .map_err(|e| format!("密码解密失败: {}", e))?;

    let credentials = UserCredentials {
        username: creds.username.clone(),
        password,
    };

    state.initialize_client(creds.username);
    let mut client = state.get_client()?;

    let login_response = match client.login(&credentials).await {
        Ok(response) => response,
        Err(error) if is_session_expired_error(&error) => {
            state.logout();
            storage.clear_credentials()?;
            return Ok(SessionStatus {
                logged_in: false,
                relogin_performed: true,
                user_info: None,
            });
        }
        Err(error) => {
            return Err(error);
        }
    };

    if !login_response.success {
        state.logout();
        storage.clear_credentials()?;
        return Ok(SessionStatus {
            logged_in: false,
            relogin_performed: true,
            user_info: None,
        });
    }

    let user_info = client.get_user_info().await?;

    Ok(SessionStatus {
        logged_in: true,
        relogin_performed: true,
        user_info: Some(user_info),
    })
}
