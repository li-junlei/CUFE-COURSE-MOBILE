// 背景图模块 - 背景图管理相关命令

use crate::storage::StorageManager;
use chrono::Utc;
use std::fs;
use std::path::Path;

/// 保存背景图
#[tauri::command]
pub async fn save_background_image(source_path: String) -> Result<String, String> {
    let storage = StorageManager::new()?;
    let bg_dir = storage.background_dir();

    // 读取源文件
    let file_name = format!("bg_{}.jpg", Utc::now().timestamp());
    let dest_path = bg_dir.join(&file_name);

    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("复制背景图失败: {}", e))?;

    // 保存到配置
    let mut config = storage.load_config().unwrap_or_default();
    let stored_path = dest_path.to_string_lossy().to_string();
    config.background_image = Some(stored_path.clone());
    storage.save_config(&config)?;

    Ok(stored_path)
}

/// 上传背景图（接收字节数组，适用于移动端/跨平台前端直传）
#[tauri::command]
pub async fn upload_background_image(bytes: Vec<u8>) -> Result<String, String> {
    let storage = StorageManager::new()?;
    let bg_dir = storage.background_dir();

    let file_name = format!("bg_{}.jpg", Utc::now().timestamp());
    let dest_path = bg_dir.join(&file_name);

    fs::write(&dest_path, &bytes)
        .map_err(|e| format!("保存背景图失败: {}", e))?;

    let mut config = storage.load_config().unwrap_or_default();
    let stored_path = dest_path.to_string_lossy().to_string();
    config.background_image = Some(stored_path.clone());
    storage.save_config(&config)?;

    Ok(stored_path)
}

/// 删除背景图
#[tauri::command]
pub fn delete_background_image() -> Result<(), String> {
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;

    if let Some(path_or_name) = config.background_image {
        let candidate = Path::new(&path_or_name);
        if candidate.is_absolute() {
            if candidate.exists() {
                fs::remove_file(candidate)
                    .map_err(|e| format!("删除背景文件失败: {}", e))?;
            }
        } else {
            storage.delete_background(&path_or_name)?;
        }
    }

    // 更新配置
    let mut config = storage.load_config().unwrap_or_default();
    config.background_image = None;
    storage.save_config(&config)?;

    Ok(())
}
