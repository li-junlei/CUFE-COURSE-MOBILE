// 配置模块 - 应用配置、时间表管理相关命令

use crate::models::{AppConfig, EduSystem, TimeTable};
use crate::storage::StorageManager;
use std::collections::HashSet;

/// 获取应用配置 (含自动迁移逻辑)
#[tauri::command]
pub fn get_app_config() -> Result<AppConfig, String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config()?;

    // 自动迁移：如果 time_tables 为空但 period_times 存在，创建默认时间表
    let has_time_tables = config.time_tables.as_ref().map_or(false, |v| !v.is_empty());
    if !has_time_tables {
        if let Some(ref periods) = config.period_times {
            let default_table = TimeTable {
                id: "default".to_string(),
                name: "默认时间表".to_string(),
                periods: periods.clone(),
            };
            config.time_tables = Some(vec![default_table]);
            storage.save_config(&config)?;
        }
    }

    // 自动迁移：智能更新 edu_systems
    // 定义所有默认的教务系统
    let default_systems = vec![
        EduSystem {
            id: "cufe".to_string(),
            name: "中央财经大学".to_string(),
            url: "https://xuanke.cufe.edu.cn/jwglxt/".to_string(),
            parser_type: "cufe_default".to_string(),
            enabled: true,
        },
        EduSystem {
            id: "zju".to_string(),
            name: "浙江大学".to_string(),
            url: "https://zdbk.zju.edu.cn/jwglxt/".to_string(),
            parser_type: "zju_default".to_string(),
            enabled: true,
        },
    ];

    let mut needs_save = false;
    let mut current_systems = config.edu_systems.take().unwrap_or_default();

    // 构建现有系统的 ID 集合
    let existing_ids: HashSet<String> =
        current_systems.iter().map(|s| s.id.clone()).collect();

    // 添加缺失的默认系统
    for default_system in default_systems {
        if !existing_ids.contains(&default_system.id) {
            println!("添加新的教务系统: {}", default_system.name);
            current_systems.push(default_system);
            needs_save = true;
        }
    }

    // 如果有旧配置且包含 cufe URL，保留用户的 URL
    if let Some(ref old_url) = config.edu_system_url {
        if old_url.contains("cufe.edu.cn") {
            if let Some(cufe_system) = current_systems.iter_mut().find(|s| s.id == "cufe") {
                if cufe_system.url != *old_url {
                    cufe_system.url = old_url.clone();
                    needs_save = true;
                }
            }
        }
    }

    // 确保 last_edu_system_id 有值
    if config.last_edu_system_id.is_none() && !current_systems.is_empty() {
        config.last_edu_system_id = Some(current_systems[0].id.clone());
        needs_save = true;
    }

    config.edu_systems = Some(current_systems);

    if needs_save {
        storage.save_config(&config)?;
    }

    Ok(config)
}

/// 保存应用配置
#[tauri::command]
pub fn save_app_config(config: AppConfig) -> Result<(), String> {
    let storage = StorageManager::new()?;
    storage.save_config(&config)?;
    Ok(())
}

/// 保存时间表
#[tauri::command]
pub fn save_time_table(time_table: TimeTable) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();

    let mut tables = config.time_tables.unwrap_or_default();

    // 如果ID已存在则更新，否则添加
    if let Some(index) = tables.iter().position(|t| t.id == time_table.id) {
        tables[index] = time_table;
    } else {
        tables.push(time_table);
    }

    config.time_tables = Some(tables);
    storage.save_config(&config)?;
    Ok(())
}

/// 删除时间表
#[tauri::command]
pub fn delete_time_table(id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();

    if let Some(tables) = config.time_tables {
        let new_tables: Vec<TimeTable> = tables.into_iter().filter(|t| t.id != id).collect();
        config.time_tables = Some(new_tables);
        storage.save_config(&config)?;
    }
    Ok(())
}

/// 获取时间表列表
#[tauri::command]
pub fn list_time_tables() -> Result<Vec<TimeTable>, String> {
    let config = get_app_config()?;
    Ok(config.time_tables.unwrap_or_default())
}

/// 将指定课表的设置应用到所有课表
#[tauri::command]
pub fn apply_settings_to_all(source_schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;

    // 1. 获取源课表设置
    let source = storage.load_schedule(&source_schedule_id)?;
    // 注意：不应用 first_day，因为不同学期或不同用户的课表起始日可能不同
    let max_periods = source.max_periods;
    let weeks_count = source.weeks_count;
    let time_table_id = source.time_table_id;

    // 2. 获取所有课表ID
    let metadata_list = storage.list_schedules()?;

    // 3. 遍历更新
    for meta in metadata_list {
        if meta.id == source_schedule_id {
            continue;
        }

        let mut schedule = storage.load_schedule(&meta.id)?;
        schedule.max_periods = max_periods;
        schedule.weeks_count = weeks_count;
        schedule.time_table_id = time_table_id.clone();

        storage.save_schedule(&schedule)?;
    }

    // 4. 更新全局配置 (如果存在相关项)
    let mut config = storage.load_config()?;
    config.max_periods = max_periods;
    storage.save_config(&config)?;

    Ok(())
}

/// 清空所有数据
#[tauri::command]
pub fn clear_all_data() -> Result<(), String> {
    let storage = StorageManager::new()?;
    storage.clear_all()?;
    Ok(())
}
