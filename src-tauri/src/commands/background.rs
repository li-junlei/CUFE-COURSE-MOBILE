// 背景图模块 - 背景图管理相关命令

use crate::storage::StorageManager;
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::Path;

/// 背景图上传结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundUploadResult {
    pub image_path: String,
    pub original_path: Option<String>,
}

/// 校验并规范化原图扩展名，非法值回退 jpg
fn sanitize_ext(ext: Option<String>) -> String {
    match ext {
        Some(e) if (2..=5).contains(&e.len()) && e.chars().all(|c| c.is_ascii_alphanumeric()) => {
            e.to_lowercase()
        }
        _ => "jpg".to_string(),
    }
}

/// 删除背景文件（兼容绝对路径 / backgrounds 目录下纯文件名两种记录方式）
fn remove_background_file(storage: &StorageManager, path_or_name: &str) {
    let candidate = Path::new(path_or_name);
    if candidate.is_absolute() {
        if candidate.exists() {
            let _ = fs::remove_file(candidate);
        }
    } else {
        let _ = storage.delete_background(path_or_name);
    }
}

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
/// bytes: 显示用图片（裁剪结果或动图原图）；original_bytes: 原始未裁剪图（重新裁剪用）
#[tauri::command]
pub async fn upload_background_image(
    bytes: Vec<u8>,
    original_bytes: Option<Vec<u8>>,
    original_ext: Option<String>,
) -> Result<BackgroundUploadResult, String> {
    let storage = StorageManager::new()?;
    let bg_dir = storage.background_dir();

    // 旧文件快照，全部成功后清理
    let old_config = storage.load_config().unwrap_or_default();

    // 毫秒时间戳，避免连续上传同秒互相覆盖
    let ts = Utc::now().timestamp_millis();
    let dest_path = bg_dir.join(format!("bg_{}.jpg", ts));

    fs::write(&dest_path, &bytes).map_err(|e| format!("保存背景图失败: {}", e))?;

    let original_path = match original_bytes {
        Some(ob) => {
            let path = bg_dir.join(format!("bg_{}_o.{}", ts, sanitize_ext(original_ext)));
            fs::write(&path, &ob).map_err(|e| format!("保存背景原图失败: {}", e))?;
            Some(path.to_string_lossy().to_string())
        }
        None => None,
    };

    let mut config = storage.load_config().unwrap_or_default();
    config.background_image = Some(dest_path.to_string_lossy().to_string());
    config.background_original = original_path.clone();
    storage.save_config(&config)?;

    // 写新文件与配置均成功后，清理旧背景文件（失败不影响结果）
    if let Some(old) = &old_config.background_image {
        remove_background_file(&storage, old);
    }
    if let Some(old) = &old_config.background_original {
        remove_background_file(&storage, old);
    }

    Ok(BackgroundUploadResult {
        image_path: dest_path.to_string_lossy().to_string(),
        original_path,
    })
}

/// 删除背景图
#[tauri::command]
pub fn delete_background_image() -> Result<(), String> {
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;

    if let Some(path_or_name) = config.background_image.as_ref() {
        remove_background_file(&storage, path_or_name);
    }
    if let Some(path_or_name) = config.background_original.as_ref() {
        remove_background_file(&storage, path_or_name);
    }

    // 更新配置（不透明度/模糊度保留，换新背景时延续上次调节）
    let mut config = storage.load_config().unwrap_or_default();
    config.background_image = None;
    config.background_original = None;
    storage.save_config(&config)?;

    Ok(())
}
