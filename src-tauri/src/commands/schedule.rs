// 课表模块 - 课表 CRUD、导入、更新相关命令

use crate::client::EduSystemState;
use crate::crypto::decrypt_password_dpapi;
use crate::models::{Course, CachedSchedule, ScheduleMetadata, ScheduleDiff, UserCredentials, CourseType};
use crate::storage::StorageManager;
use crate::services::schedule_service::calculate_schedule_diff;
use chrono::{Utc, Datelike};
use std::sync::Arc;

/// 使用已保存的登录状态导入课表
#[tauri::command]
pub async fn import_schedule_from_saved_login(
    state: tauri::State<'_, Arc<EduSystemState>>,
    year: i32,
    term: i32,
    schedule_name: String,
) -> Result<String, String> {
    // 检查是否已登录
    if !state.is_logged_in() {
        return Err("未找到登录信息，请先在个人中心登录".to_string());
    }

    // 获取全局 client（复用登录时的会话）
    let client = state.get_client()?;

    // 获取课表
    let courses = client.get_schedule(year, term).await?;

    if courses.is_empty() {
        return Err("该学期暂无课程".to_string());
    }

    let storage = StorageManager::new()?;

    // 生成课表 ID
    let schedule_id = StorageManager::generate_schedule_id();

    // 保存课表
    let now = Utc::now().timestamp();
    let expire_time = now + (30 * 24 * 60 * 60); // 30 天后过期

    // 计算 sort_index (放在最后)
    let current_list = storage.list_schedules()?;
    let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
    let sort_index = max_index + 1;

    let cached = CachedSchedule {
        id: schedule_id.clone(),
        name: schedule_name,
        courses,
        timestamp: now,
        expire_time,
        first_day: None,
        max_periods: None,
        weeks_count: None,
        time_table_id: None,
        sort_index: Some(sort_index),
        school_year: Some(year),
        school_term: Some(term),
    };

    storage.save_schedule(&cached)?;

    Ok(schedule_id)
}

/// 导入课表（带自动重新登录）
#[tauri::command]
pub async fn import_schedule_with_auto_relogin(
    state: tauri::State<'_, Arc<EduSystemState>>,
    year: i32,
    term: i32,
    schedule_name: String,
) -> Result<String, String> {
    // 尝试使用当前会话导入
    let import_result = import_schedule_from_saved_login(
        state.clone(),
        year,
        term,
        schedule_name.clone(),
    ).await;

    // 如果成功，直接返回
    if import_result.is_ok() {
        return import_result;
    }

    // 如果失败，尝试自动重新登录
    let storage = StorageManager::new()?;

    // 检查是否有保存的凭证
    let creds = match storage.load_credentials() {
        Ok(c) => c,
        Err(_) => return Err(import_result.unwrap_err()),
    };

    // 解密密码并重新登录
    let password = decrypt_password_dpapi(&creds.password_encrypted)
        .map_err(|e| format!("密码解密失败: {}", e))?;

    let credentials = UserCredentials {
        username: creds.username,
        password,
    };

    state.auto_relogin(&credentials).await?;

    println!("已自动重新登录，重试导入课表");

    // 重试导入
    import_schedule_from_saved_login(
        state,
        year,
        term,
        schedule_name,
    ).await
}

/// 登录并获取课表
#[tauri::command]
pub async fn login_and_get_schedule(
    state: tauri::State<'_, Arc<EduSystemState>>,
    username: String,
    password: String,
) -> Result<Vec<Course>, String> {
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

    // 尝试多个学期：当前学期，如果失败则尝试上一学期，再失败尝试下一学期
    let now = Utc::now();
    let month = now.month();
    let year = now.year();

    // 确定当前推测的学年和学期
    let (current_year, current_term) = if month >= 9 {
        (year, 1) // 9月-12月: 第一学期
    } else if month == 1 {
        (year - 1, 1) // 1月: 仍视为第一学期
    } else {
        (year - 1, 2) // 2月-8月: 第二学期
    };

    // 生成待尝试的 (year, term) 列表
    let mut tasks = vec![
        (current_year, current_term),
    ];

    // 如果是第二学期，失败后尝试第一学期
    if current_term == 2 {
        tasks.push((current_year, 1));
    } else {
        tasks.push((current_year - 1, 2));
    }

    if current_term == 1 {
        tasks.push((current_year, 2));
    } else {
        tasks.push((current_year + 1, 1));
    }

    let mut final_courses = Vec::new();
    let mut last_error = "未找到有效的课表数据".to_string();

    for (y, t) in tasks {
        println!("尝试获取课表: {}-学期{}", y, t);
        match client.get_schedule(y, t).await {
            Ok(courses) => {
                if !courses.is_empty() {
                    println!("成功获取课表: {} 门课程", courses.len());
                    final_courses = courses;
                    break;
                } else {
                    println!("课表为空，尝试下一个学期...");
                    last_error = "当前学期暂无课程".to_string();
                }
            },
            Err(e) => {
                println!("获取失败: {}, 尝试下一个学期...", e);
                last_error = e;
            }
        }
    }

    if final_courses.is_empty() {
        // 清除登录状态
        state.logout();
        return Err(format!("无法获取课表: {}", last_error));
    }

    Ok(final_courses)
}

/// 刷新课表数据（使用全局 client）
#[tauri::command]
pub async fn refresh_schedule(state: tauri::State<'_, Arc<EduSystemState>>) -> Result<Vec<Course>, String> {
    // 检查是否已登录
    if !state.is_logged_in() {
        return Err("未找到登录信息，请先在个人中心登录".to_string());
    }

    // 获取全局 client（复用登录时的会话）
    let client = state.get_client()?;

    // 计算当前学期
    let now = Utc::now();
    let month = now.month();
    let year = now.year();

    let (school_year, term) = if month >= 9 {
        (year, 1)
    } else if month == 1 {
        (year - 1, 1)
    } else {
        (year - 1, 2)
    };

    // 获取课表
    let courses = client.get_schedule(school_year, term).await?;

    Ok(courses)
}

/// 加载缓存的课表数据
#[tauri::command]
pub fn load_cached_schedule(schedule_id: Option<String>) -> Result<Vec<Course>, String> {
    let storage = StorageManager::new()?;

    // 如果没有指定 ID,尝试从配置中获取
    let id = if let Some(sid) = schedule_id {
        sid
    } else {
        let config = storage.load_config()?;
        if let Some(current_id) = config.current_schedule_id {
            current_id
        } else {
            return Err("没有选中的课表".to_string());
        }
    };

    let cached = storage.load_schedule(&id)?;

    if cached.is_expired() {
        return Err("课表数据已过期，请刷新".to_string());
    }

    Ok(cached.courses)
}

/// 保存课表数据到缓存
#[tauri::command]
pub fn save_schedule_cache(
    courses: Vec<Course>,
    name: String,
    schedule_id: Option<String>,
    first_day: Option<i64>,
    max_periods: Option<i32>,
    weeks_count: Option<i32>,
    time_table_id: Option<String>
) -> Result<String, String> {
    let storage = StorageManager::new()?;

    let now = Utc::now().timestamp();
    let expire_time = now + (30 * 24 * 60 * 60); // 30 天后过期

    // 生成或使用指定的 ID
    let id = if let Some(sid) = schedule_id {
        sid
    } else {
        StorageManager::generate_schedule_id()
    };

    // 如果是新课表，需要计算 sort_index
    let mut sort_index = None;
    if let Ok(existing) = storage.load_schedule(&id) {
        sort_index = existing.sort_index;
    }
    if sort_index.is_none() {
        let current_list = storage.list_schedules()?;
        let max_index = current_list.iter().filter_map(|s| s.sort_index).max().unwrap_or(-1);
        sort_index = Some(max_index + 1);
    }

    let cached = CachedSchedule {
        id: id.clone(),
        name,
        courses,
        timestamp: now,
        expire_time,
        first_day,
        max_periods,
        weeks_count,
        time_table_id,
        sort_index,
        school_year: None,
        school_term: None,
    };

    storage.save_schedule(&cached)?;

    Ok(id)
}

/// 获取所有课表列表
#[tauri::command]
pub fn list_schedules() -> Result<Vec<ScheduleMetadata>, String> {
    let storage = StorageManager::new()?;
    storage.list_schedules()
}

/// 删除指定课表
#[tauri::command]
pub fn delete_schedule(schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;

    // 获取当前配置
    let mut config = storage.load_config().unwrap_or_default();

    // 如果删除的是当前选中的课表,清空选中状态
    if let Some(current_id) = &config.current_schedule_id {
        if current_id == &schedule_id {
            config.current_schedule_id = None;
            storage.save_config(&config)?;
        }
    }

    storage.delete_schedule(&schedule_id)
}

/// 切换当前课表
#[tauri::command]
pub fn switch_schedule(schedule_id: String) -> Result<(), String> {
    let storage = StorageManager::new()?;
    let mut config = storage.load_config().unwrap_or_default();
    config.current_schedule_id = Some(schedule_id.clone());

    // 同步课表的 first_day 到全局配置
    let schedule = storage.load_schedule(&schedule_id)?;
    config.first_day = schedule.first_day;

    storage.save_config(&config)?;
    Ok(())
}

/// 重新排序课表
#[tauri::command]
pub fn reorder_schedules(sorted_ids: Vec<String>) -> Result<(), String> {
    let storage = StorageManager::new()?;

    for (index, id) in sorted_ids.iter().enumerate() {
        if let Ok(mut schedule) = storage.load_schedule(id) {
            schedule.sort_index = Some(index as i32);
            storage.save_schedule(&schedule)?;
        }
    }

    Ok(())
}

/// 更新课表信息
#[tauri::command]
pub fn update_schedule_info(
    schedule_id: String,
    first_day: Option<i64>,
    max_periods: Option<i32>,
    weeks_count: Option<i32>,
    time_table_id: Option<String>
) -> Result<(), String> {
    println!("更新课表信息 - schedule_id: {}", schedule_id);

    let storage = StorageManager::new()?;

    // 加载课表
    let mut cached = storage.load_schedule(&schedule_id)?;

    // 更新字段
    cached.first_day = first_day;
    cached.max_periods = max_periods;
    cached.weeks_count = weeks_count;
    cached.time_table_id = time_table_id;

    // 保存
    storage.save_schedule(&cached)?;

    // 如果这是当前选中的课表，也更新全局配置 (兼容性)
    let config = storage.load_config()?;
    if let Some(ref current_id) = config.current_schedule_id {
        if current_id == &schedule_id {
            let mut new_config = config;
            new_config.first_day = first_day;
            if let Some(mp) = max_periods { new_config.max_periods = Some(mp); }
            storage.save_config(&new_config)?;
        }
    }

    Ok(())
}

/// 重命名课表
#[tauri::command]
pub fn rename_schedule(schedule_id: String, new_name: String) -> Result<(), String> {
    println!("重命名课表 - schedule_id: {}, new_name: {}", schedule_id, new_name);

    let storage = StorageManager::new()?;

    // 加载课表
    let mut cached = storage.load_schedule(&schedule_id)?;

    // 更新名称
    cached.name = new_name.clone();

    // 保存
    storage.save_schedule(&cached)?;

    Ok(())
}

/// 更新课表并返回差异统计
#[tauri::command]
pub async fn update_schedule_with_diff(
    state: tauri::State<'_, Arc<EduSystemState>>,
    schedule_id: String,
) -> Result<ScheduleDiff, String> {
    let storage = StorageManager::new()?;

    // 1. 加载现有课表
    let old_schedule = storage.load_schedule(&schedule_id)?;

    // 2. 获取学年学期信息
    let (year, term) = match (old_schedule.school_year, old_schedule.school_term) {
        (Some(y), Some(t)) => (y, t),
        _ => return Err("该课表没有学年学期信息，无法更新。请删除后重新导入。".to_string()),
    };

    // 3. 检查登录状态，如未登录则尝试自动重登录
    if !state.is_logged_in() {
        // 尝试加载保存的凭证并自动登录
        match storage.load_credentials() {
            Ok(creds) => {
                let password = decrypt_password_dpapi(&creds.password_encrypted)?;
                let credentials = UserCredentials {
                    username: creds.username,
                    password,
                };
                state.auto_relogin(&credentials).await?;
            }
            Err(_) => return Err("未找到登录信息，请先在个人中心登录".to_string()),
        }
    }

    // 4. 获取最新课表数据
    let client = state.get_client()?;
    let new_courses = client.get_schedule(year, term).await?;

    if new_courses.is_empty() {
        return Err("该学期暂无课程".to_string());
    }

    // 5. 计算差异
    let diff = calculate_schedule_diff(&old_schedule.courses, &new_courses);

    // 6. 合并课程：保留原有的考试信息，更新常规课程
    let mut merged_courses: Vec<Course> = old_schedule.courses.iter()
        .filter(|c| c.course_type == CourseType::Exam)
        .cloned()
        .collect();
    merged_courses.extend(new_courses);

    // 7. 保存新课表数据
    let mut updated_schedule = old_schedule.clone();
    updated_schedule.courses = merged_courses;
    updated_schedule.timestamp = Utc::now().timestamp();
    storage.save_schedule(&updated_schedule)?;

    Ok(diff)
}
