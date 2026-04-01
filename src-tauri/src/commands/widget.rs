// Widget 模块 - 为 Android App Widget 提供数据

use crate::storage::StorageManager;
use chrono::{Utc, Datelike, FixedOffset, TimeZone};
#[cfg(target_os = "android")]
use jni::objects::{JObject, JString, JValue};
#[cfg(target_os = "android")]
use jni::JavaVM;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Widget 课程信息（简化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetCourseInfo {
    pub name: String,
    pub location: String,
    pub start_time: String,
    pub end_time: String,
    pub day_of_week: i32,
    pub weeks: Vec<i32>,
    pub week_type: i32,
    pub period_start: i32,
    pub period_end: i32,
}

/// Widget 响应数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetData {
    pub schema_version: i32,
    pub generated_at: i64,
    pub first_day: Option<i64>,
    pub weeks_count: Option<i32>,
    pub current_week: i32,
    pub current_date: String,
    pub current_day_of_week: i32,
    pub day_name: String,
    pub courses: Vec<WidgetCourseInfo>,
    pub is_empty: bool,
    pub message: Option<String>,
}

fn compute_current_week(now_ts: i64, first_day: Option<i64>, weeks_count: Option<i32>) -> i32 {
    let week = if let Some(first_day) = first_day {
        (((now_ts - first_day) / (7 * 24 * 60 * 60)) + 1).max(1) as i32
    } else {
        1
    };

    if let Some(max_weeks) = weeks_count.filter(|value| *value > 0) {
        week.clamp(1, max_weeks)
    } else {
        week.max(1)
    }
}

fn is_week_in_ranges(weeks: &[i32], week_type: i32, week: i32) -> bool {
    let in_weeks = weeks.contains(&week);
    if !in_weeks {
        return false;
    }

    match week_type {
        1 => week % 2 == 1,
        2 => week % 2 == 0,
        _ => true,
    }
}

/// 获取星期名称
fn get_day_name(day: i32) -> String {
    match day {
        1 => "星期一".to_string(),
        2 => "星期二".to_string(),
        3 => "星期三".to_string(),
        4 => "星期四".to_string(),
        5 => "星期五".to_string(),
        6 => "星期六".to_string(),
        7 => "星期日".to_string(),
        _ => "".to_string(),
    }
}

/// 获取课程时间字符串
fn get_course_time_string(course: &crate::models::Course, time_table: &[crate::models::PeriodTime]) -> (String, String) {
    if let Some(start_period) = course.periods.first() {
        let start_idx = (*start_period - 1) as usize;
        if let Some(end_period) = course.periods.last() {
            let end_idx = (*end_period - 1) as usize;
            
            let start_time = time_table.get(start_idx).map(|t| t.start.clone()).unwrap_or_default();
            let end_time = time_table.get(end_idx).map(|t| t.end.clone()).unwrap_or_default();
            
            return (start_time, end_time);
        }
    }
    ("".to_string(), "".to_string())
}

/// 简化地点显示（与课表卡片规则保持一致）
fn format_location(location: &str, simplified: bool) -> String {
    if !simplified {
        return location.to_string();
    }

    let mut formatted = location.to_string();
    for prefix in ["沙河校区", "学院南路校区", "沙河", "南路", "学院南路"] {
        formatted = formatted.replace(prefix, "");
    }
    formatted.trim().to_string()
}

/// 获取 Widget 数据 - 当天课程信息
#[tauri::command]
pub fn get_widget_data() -> Result<WidgetData, String> {
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;
    
    // 获取当前时间 - 使用东八区时间，与 first_day 配置保持一致
    let now = FixedOffset::east_opt(8 * 3600)
        .unwrap()
        .from_utc_datetime(&Utc::now().naive_utc());
    let day_of_week = now.weekday().num_days_from_monday() as i32 + 1; // 1-7
    
    // 格式化日期
    let current_date = format!("{}月{}日", now.month(), now.day());
    let fallback_week = compute_current_week(now.timestamp(), config.first_day, config.end_week);
    
    let simplified_location = config.simplified_location.unwrap_or(false);

    // 获取课表数据
    let schedule_id = if let Some(id) = config.current_schedule_id {
        id
    } else {
        return Ok(WidgetData {
            schema_version: 2,
            generated_at: now.timestamp(),
            first_day: config.first_day,
            weeks_count: config.end_week,
            current_week: fallback_week,
            current_date,
            current_day_of_week: day_of_week,
            day_name: get_day_name(day_of_week),
            courses: vec![],
            is_empty: true,
            message: Some("课程结束啦 🎉".to_string()),
        });
    };
    
    let cached = match storage.load_schedule(&schedule_id) {
        Ok(c) => c,
        Err(_) => {
            return Ok(WidgetData {
                schema_version: 2,
                generated_at: now.timestamp(),
                first_day: config.first_day,
                weeks_count: config.end_week,
                current_week: fallback_week,
                current_date,
                current_day_of_week: day_of_week,
                day_name: get_day_name(day_of_week),
                courses: vec![],
                is_empty: true,
                message: Some("课程结束啦 🎉".to_string()),
            });
        }
    };

    let first_day = cached.first_day.or(config.first_day);
    let weeks_count = cached.weeks_count.or(config.end_week);
    let current_week = compute_current_week(now.timestamp(), first_day, weeks_count);
    
    // 获取时间表
    let time_table = if let Some(tables) = &config.time_tables {
        if let Some(current_id) = &cached.time_table_id {
            tables.iter().find(|t| &t.id == current_id).map(|t| t.periods.clone())
        } else {
            tables.first().map(|t| t.periods.clone())
        }
    } else {
        config.period_times.clone()
    };
    
    let time_table = time_table.unwrap_or_else(|| {
        vec![
            crate::models::PeriodTime { start: "8:00".to_string(), end: "8:45".to_string() },
            crate::models::PeriodTime { start: "8:55".to_string(), end: "9:40".to_string() },
            crate::models::PeriodTime { start: "10:00".to_string(), end: "10:45".to_string() },
            crate::models::PeriodTime { start: "10:55".to_string(), end: "11:40".to_string() },
            crate::models::PeriodTime { start: "11:50".to_string(), end: "12:35".to_string() },
            crate::models::PeriodTime { start: "12:45".to_string(), end: "13:30".to_string() },
            crate::models::PeriodTime { start: "14:00".to_string(), end: "14:45".to_string() },
            crate::models::PeriodTime { start: "14:55".to_string(), end: "15:40".to_string() },
            crate::models::PeriodTime { start: "16:00".to_string(), end: "16:45".to_string() },
            crate::models::PeriodTime { start: "16:55".to_string(), end: "17:40".to_string() },
            crate::models::PeriodTime { start: "17:50".to_string(), end: "18:35".to_string() },
            crate::models::PeriodTime { start: "19:20".to_string(), end: "20:05".to_string() },
            crate::models::PeriodTime { start: "20:15".to_string(), end: "21:00".to_string() },
        ]
    });
    
    let mut widget_courses: Vec<WidgetCourseInfo> = cached.courses.iter()
        .filter(|course| {
            // 排除考试类型
            if course.course_type == crate::models::CourseType::Exam {
                return false;
            }
            true
        })
        .map(|course| {
            let (start_time, end_time) = get_course_time_string(course, &time_table);
            WidgetCourseInfo {
                name: course.name.clone(),
                location: format_location(&course.location, simplified_location),
                start_time,
                end_time,
                day_of_week: course.day_of_week,
                weeks: course.weeks.clone(),
                week_type: course.week_type,
                period_start: *course.periods.first().unwrap_or(&1),
                period_end: *course.periods.last().unwrap_or(&1),
            }
        })
        .collect();
    
    // 按节次排序
    widget_courses.sort_by(|a, b| {
        a.day_of_week
            .cmp(&b.day_of_week)
            .then(a.period_start.cmp(&b.period_start))
    });

    let has_today_courses = widget_courses.iter().any(|course| {
        course.day_of_week == day_of_week
            && is_week_in_ranges(&course.weeks, course.week_type, current_week)
    });
    let is_empty = !has_today_courses;
    let message = if is_empty {
        Some("课程结束啦 🎉".to_string())
    } else {
        None
    };
    
    Ok(WidgetData {
        schema_version: 2,
        generated_at: now.timestamp(),
        first_day,
        weeks_count,
        current_week,
        current_date,
        current_day_of_week: day_of_week,
        day_name: get_day_name(day_of_week),
        courses: widget_courses,
        is_empty,
        message,
    })
}

/// 获取 Widget 数据文件路径
fn get_widget_data_path(storage: &StorageManager) -> PathBuf {
    let widget_dir = storage
        .cookie_path()
        .parent()
        .and_then(|path| path.parent())
        .map(|path| path.join("widget"))
        .unwrap_or_else(|| PathBuf::from("/data/data/com.lijunlei.cufecourse/files/widget"));
    fs::create_dir_all(&widget_dir).ok();
    widget_dir.join("widget_data.json")
}

/// 保存 Widget 数据到文件（供 Android Widget 读取）
#[tauri::command]
pub fn save_widget_data() -> Result<WidgetData, String> {
    let data = get_widget_data()?;
    let storage = StorageManager::new()?;
    
    // 保存到文件
    let path = get_widget_data_path(&storage);
    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化 Widget 数据失败: {}", e))?;
    
    fs::write(&path, json)
        .map_err(|e| format!("写入 Widget 数据文件失败: {}", e))?;
    
    Ok(data)
}

#[cfg(target_os = "android")]
fn notify_android_widget_update() -> Result<(), String> {
    let android_context = ndk_context::android_context();
    let vm = unsafe { JavaVM::from_raw(android_context.vm().cast()) }
        .map_err(|e| format!("获取 Android VM 失败: {e}"))?;
    let mut env = vm
        .attach_current_thread()
        .map_err(|e| format!("附加 Android 线程失败: {e}"))?;

    let context_raw = android_context.context().cast();
    let context = unsafe { JObject::from_raw(context_raw) };
    let local_context = env
        .new_local_ref(&context)
        .map_err(|e| format!("获取 Android Context 失败: {e}"))?;
    std::mem::forget(context);

    let package_name_obj = env
        .call_method(&local_context, "getPackageName", "()Ljava/lang/String;", &[])
        .and_then(|value| value.l())
        .map_err(|e| format!("读取包名失败: {e}"))?;
    let package_name = JString::from(package_name_obj);
    let package_name_text = env
        .get_string(&package_name)
        .map_err(|e| format!("解析包名失败: {e}"))?
        .to_string_lossy()
        .into_owned();
    let class_name = env
        .new_string(format!("{package_name_text}.CourseWidgetProvider"))
        .map_err(|e| format!("创建类名失败: {e}"))?;
    let action = env
        .new_string("com.lijunlei.cufecourse.ACTION_WIDGET_UPDATE")
        .map_err(|e| format!("创建 action 失败: {e}"))?;
    let intent = env
        .new_object(
            "android/content/Intent",
            "(Ljava/lang/String;)V",
            &[JValue::Object(&JObject::from(action))],
        )
        .map_err(|e| format!("创建 Intent 失败: {e}"))?;

    env.call_method(
        &intent,
        "setClassName",
        "(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
        &[
            JValue::Object(&JObject::from(package_name)),
            JValue::Object(&JObject::from(class_name)),
        ],
    )
    .map_err(|e| format!("设置广播目标失败: {e}"))?;

    env.call_method(
        &local_context,
        "sendBroadcast",
        "(Landroid/content/Intent;)V",
        &[JValue::Object(&intent)],
    )
    .map_err(|e| format!("发送 Widget 刷新广播失败: {e}"))?;

    Ok(())
}

#[cfg(not(target_os = "android"))]
fn notify_android_widget_update() -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub fn sync_widget_data() -> Result<WidgetData, String> {
    let data = save_widget_data()?;
    notify_android_widget_update()?;
    Ok(data)
}
