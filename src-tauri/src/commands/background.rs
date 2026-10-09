// 背景图模块 - 背景图管理相关命令

use crate::storage::StorageManager;
use base64::{Engine as _, engine::general_purpose};
use chrono::Utc;
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

/// 背景图上传结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundUploadResult {
    pub file_name: String,
    pub original_file_name: Option<String>,
}

/// 背景文件读取结果
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackgroundFileData {
    pub data_base64: String, // 文件字节 base64 编码
    pub ext: String,         // 实际扩展名（小写、无点），前端用 extToMime 拼 data URL
}

/// 校验并规范化扩展名，非法值回退 jpg
fn sanitize_ext(ext: Option<String>) -> String {
    match ext {
        Some(e) if (2..=5).contains(&e.len()) && e.chars().all(|c| c.is_ascii_alphanumeric()) => {
            e.to_lowercase()
        }
        _ => "jpg".to_string(),
    }
}

/// 将 config 记录解析为绝对路径
/// （兼容历史 config 中的绝对路径 / backgrounds 目录下纯文件名两种记录方式）
fn resolve_background_path(storage: &StorageManager, path_or_name: &str) -> PathBuf {
    let candidate = Path::new(path_or_name);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        storage.background_dir().join(path_or_name)
    }
}

/// 删除背景文件
fn remove_background_file(storage: &StorageManager, path_or_name: &str) {
    let path = resolve_background_path(storage, path_or_name);
    if path.exists() {
        let _ = fs::remove_file(path);
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
/// display_ext: 显示图真实扩展名（动图直传时前端传入），缺省 jpg
#[tauri::command]
pub async fn upload_background_image(
    bytes: Vec<u8>,
    original_bytes: Option<Vec<u8>>,
    original_ext: Option<String>,
    display_ext: Option<String>,
) -> Result<BackgroundUploadResult, String> {
    let storage = StorageManager::new()?;
    let bg_dir = storage.background_dir();

    // 旧文件快照，全部成功后清理
    let old_config = storage.load_config().unwrap_or_default();

    // 毫秒时间戳，避免连续上传同秒互相覆盖
    let ts = Utc::now().timestamp_millis();
    let file_name = format!("bg_{}.{}", ts, sanitize_ext(display_ext));
    let dest_path = bg_dir.join(&file_name);

    fs::write(&dest_path, &bytes).map_err(|e| format!("保存背景图失败: {}", e))?;

    let original_file_name = match original_bytes {
        Some(ob) => {
            let name = format!("bg_{}_o.{}", ts, sanitize_ext(original_ext));
            let path = bg_dir.join(&name);
            fs::write(&path, &ob).map_err(|e| format!("保存背景原图失败: {}", e))?;
            Some(name)
        }
        None => None,
    };

    let mut config = storage.load_config().unwrap_or_default();
    config.background_image = Some(file_name.clone());
    config.background_original = original_file_name.clone();
    storage.save_config(&config)?;

    // 写新文件与配置均成功后，清理旧背景文件（失败不影响结果）
    if let Some(old) = &old_config.background_image {
        remove_background_file(&storage, old);
    }
    if let Some(old) = &old_config.background_original {
        remove_background_file(&storage, old);
    }

    Ok(BackgroundUploadResult {
        file_name,
        original_file_name,
    })
}

/// 读取背景文件字节（写读同源走 Rust，绕开 fs 插件在 Android 上的路径/scope 解析问题）
/// kind: "display"=显示图（默认）；"original"=裁剪原图，无原图记录时回退显示图
#[tauri::command]
pub fn read_background_file(kind: Option<String>) -> Result<BackgroundFileData, String> {
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;

    let recorded = match kind.as_deref() {
        Some("original") => config.background_original.or(config.background_image),
        _ => config.background_image,
    }
    .ok_or("未设置背景图")?;

    let path = resolve_background_path(&storage, &recorded);
    let bytes = fs::read(&path).map_err(|e| format!("读取背景文件失败: {}", e))?;
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .unwrap_or_else(|| "jpg".to_string());

    Ok(BackgroundFileData {
        data_base64: general_purpose::STANDARD.encode(bytes),
        ext,
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
