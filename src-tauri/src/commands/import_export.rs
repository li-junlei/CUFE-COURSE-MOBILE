// 导入导出模块 - 文件导入导出相关命令

use crate::models::CachedSchedule;
use crate::storage::StorageManager;
use chrono::Utc;
use std::fs;

/// 导出课表到文件
#[tauri::command]
pub async fn export_schedule(
    schedule_id: String,
    file_path: String,
) -> Result<(), String> {
    let storage = StorageManager::new()?;

    // 1. 加载完整课表数据
    let schedule = storage.load_schedule(&schedule_id)?;

    // 2. 序列化为漂亮的 JSON
    let content = serde_json::to_string_pretty(&schedule)
        .map_err(|e| format!("无法序列化课表数据: {}", e))?;

    // 3. 写入文件
    fs::write(&file_path, content)
        .map_err(|e| format!("保存文件失败: {}", e))?;

    Ok(())
}

/// 从文件导入课表
#[tauri::command]
pub async fn import_schedule(
    file_path: String,
) -> Result<String, String> {
    let storage = StorageManager::new()?;

    // 1. 读取文件
    let content = fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    // 2. 解析 JSON
    let mut schedule: CachedSchedule = serde_json::from_str(&content)
        .map_err(|e| format!("无效的课表文件格式: {}", e))?;

    // 3. 生成新的 ID 以避免冲突
    let new_id = StorageManager::generate_schedule_id();
    schedule.id = new_id.clone();

    // 4. 更新时间戳
    schedule.timestamp = Utc::now().timestamp();

    // 5. 计算排序索引 (放到最后)
    let current_list = storage.list_schedules()?;
    let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
    schedule.sort_index = Some(max_index + 1);

    // 6. 保存到本地存储
    storage.save_schedule(&schedule)?;

    Ok(new_id)
}

/// 导出课表数据为 JSON 字符串
#[tauri::command]
pub async fn export_schedule_json(schedule_id: String) -> Result<String, String> {
    let storage = StorageManager::new()?;
    let schedule = storage.load_schedule(&schedule_id)?;
    serde_json::to_string_pretty(&schedule)
        .map_err(|e| format!("无法序列化课表数据: {}", e))
}

/// 从 JSON 字符串导入课表
#[tauri::command]
pub async fn import_schedule_json(json_str: String) -> Result<String, String> {
    let storage = StorageManager::new()?;

    let mut schedule: CachedSchedule = serde_json::from_str(&json_str)
        .map_err(|e| format!("无效的课表格式: {}", e))?;

    let new_id = StorageManager::generate_schedule_id();
    schedule.id = new_id.clone();
    schedule.timestamp = Utc::now().timestamp();

    let current_list = storage.list_schedules()?;
    let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
    schedule.sort_index = Some(max_index + 1);

    storage.save_schedule(&schedule)?;

    Ok(new_id)
}
