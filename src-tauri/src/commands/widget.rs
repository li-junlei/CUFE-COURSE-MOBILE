// Widget 模块 - 为 Android App Widget 提供数据

use crate::models::Course;
use crate::storage::StorageManager;
use chrono::{Datelike, Local};
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
    pub period_start: i32,
    pub period_end: i32,
}

/// Widget 响应数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WidgetData {
    pub current_week: i32,
    pub current_date: String,
    pub current_day_of_week: i32,
    pub day_name: String,
    pub courses: Vec<WidgetCourseInfo>,
    pub is_empty: bool,
    pub message: Option<String>,
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

/// 检查课程是否在指定周上课
fn is_course_in_week(course: &Course, week: i32) -> bool {
    // 检查周次范围
    let in_week_range = course.weeks.iter().any(|_| {
        let (start, end) = if course.weeks.len() >= 4 {
            // 有两个范围的情况 [start1, end1, start2, end2]
            (course.weeks[0], course.weeks[1])
        } else if course.weeks.len() >= 2 {
            (course.weeks[0], course.weeks[1])
        } else {
            return false;
        };
        
        let in_first_range = week >= start && week <= end;
        
        // 检查单双周
        if course.week_type == 0 {
            in_first_range
        } else if course.week_type == 1 {
            // 单周
            in_first_range && (week % 2 == 1)
        } else if course.week_type == 2 {
            // 双周
            in_first_range && (week % 2 == 0)
        } else {
            in_first_range
        }
    });
    
    // 如果有第二个范围
    if course.weeks.len() >= 4 {
        let start2 = course.weeks[2];
        let end2 = course.weeks[3];
        let in_second_range = week >= start2 && week <= end2;
        
        let final_second = if course.week_type == 0 {
            in_second_range
        } else if course.week_type == 1 {
            in_second_range && (week % 2 == 1)
        } else if course.week_type == 2 {
            in_second_range && (week % 2 == 0)
        } else {
            in_second_range
        };
        
        in_week_range || final_second
    } else {
        in_week_range
    }
}

/// 获取课程时间字符串
fn get_course_time_string(course: &Course, time_table: &[crate::models::PeriodTime]) -> (String, String) {
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

/// 获取 Widget 数据 - 当天课程信息
#[tauri::command]
pub fn get_widget_data() -> Result<WidgetData, String> {
    let storage = StorageManager::new()?;
    let config = storage.load_config()?;
    
    // 获取当前时间
    let now = Local::now();
    let day_of_week = now.weekday().num_days_from_monday() as i32 + 1; // 1-7
    
    // 格式化日期
    let current_date = format!("{}月{}日", now.month(), now.day());
    
    // 计算当前周次
    let current_week = if let Some(first_day) = config.first_day {
        let weeks = ((now.timestamp() - first_day) / (7 * 24 * 60 * 60)) + 1;
        weeks.max(1) as i32
    } else {
        1
    };
    
    // 获取课表数据
    let schedule_id = if let Some(id) = config.current_schedule_id {
        id
    } else {
        return Ok(WidgetData {
            current_week,
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
                current_week,
                current_date,
                current_day_of_week: day_of_week,
                day_name: get_day_name(day_of_week),
                courses: vec![],
                is_empty: true,
                message: Some("课程结束啦 🎉".to_string()),
            });
        }
    };
    
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
    
    // 过滤当天课程
    let mut today_courses: Vec<WidgetCourseInfo> = cached.courses.iter()
        .filter(|course| {
            // 排除考试类型
            if course.course_type == crate::models::CourseType::Exam {
                return false;
            }
            // 检查星期
            if course.day_of_week != day_of_week {
                return false;
            }
            // 检查周次
            if !is_course_in_week(course, current_week) {
                return false;
            }
            true
        })
        .map(|course| {
            let (start_time, end_time) = get_course_time_string(course, &time_table);
            WidgetCourseInfo {
                name: course.name.clone(),
                location: course.location.clone(),
                start_time,
                end_time,
                period_start: *course.periods.first().unwrap_or(&1),
                period_end: *course.periods.last().unwrap_or(&1),
            }
        })
        .collect();
    
    // 按节次排序
    today_courses.sort_by(|a, b| a.period_start.cmp(&b.period_start));
    
    let is_empty = today_courses.is_empty();
    let message = if is_empty {
        Some("课程结束啦 🎉".to_string())
    } else {
        None
    };
    
    Ok(WidgetData {
        current_week,
        current_date,
        current_day_of_week: day_of_week,
        day_name: get_day_name(day_of_week),
        courses: today_courses,
        is_empty,
        message,
    })
}

/// 获取 Widget 数据文件路径
fn get_widget_data_path() -> PathBuf {
    // Android 应用 files 目录 (同一个包内 Widget 可以访问)
    let widget_dir = PathBuf::from("/data/data/com.lijunlei.cufecourse/files/widget");
    fs::create_dir_all(&widget_dir).ok();
    widget_dir.join("widget_data.json")
}

/// 保存 Widget 数据到文件（供 Android Widget 读取）
#[tauri::command]
pub fn save_widget_data() -> Result<WidgetData, String> {
    let data = get_widget_data()?;
    
    // 保存到文件
    let path = get_widget_data_path();
    let json = serde_json::to_string_pretty(&data)
        .map_err(|e| format!("序列化 Widget 数据失败: {}", e))?;
    
    fs::write(&path, json)
        .map_err(|e| format!("写入 Widget 数据文件失败: {}", e))?;
    
    Ok(data)
}
