// 考试模块 - 考试信息获取相关命令

use crate::client::EduSystemState;
use crate::models::{Course, CachedSchedule, CourseType};
use crate::storage::StorageManager;
use chrono::{Utc, DateTime};
use std::sync::Arc;

/// 获取考试安排并导入到课表
#[tauri::command]
pub async fn fetch_and_import_exams(
    state: tauri::State<'_, Arc<EduSystemState>>,
    schedule_id: String,
) -> Result<Vec<Course>, String> {
    // 加载课表元数据
    let storage = StorageManager::new()?;
    let schedule = storage.load_schedule(&schedule_id)?;

    // 检查是否有学年学期信息
    let school_year = schedule.school_year
        .ok_or("课表缺少学年信息，请先编辑课表填写学年学期")?;
    let school_term = schedule.school_term
        .ok_or("课表缺少学期信息，请先编辑课表填写学年学期")?;

    // 检查是否有第一天信息
    let first_day = schedule.first_day
        .ok_or("课表缺少学期开始日期，请先在课表编辑中填写「学期第一天」")?;

    // 转换学年格式（只需年份，如 "2024"）
    let year_str = school_year.to_string();

    // 获取考试数据
    let client = state.get_client()?;
    let exam_json = client.get_exam_schedule(&year_str, school_term).await?;

    // 转换时间戳为日期字符串
    let first_day_date = DateTime::from_timestamp(first_day, 0)
        .ok_or("无效的学期开始日期")?;
    let semester_start = first_day_date.format("%Y-%m-%d").to_string();

    // 解析考试数据
    use crate::parser::parse_exam_json;
    let exams = parse_exam_json(&exam_json, &semester_start)?;

    if exams.is_empty() {
        return Err("未查询到考试安排".to_string());
    }

    // 合并到现有课表（移除旧考试）
    let regular_courses: Vec<Course> = schedule.courses.into_iter()
        .filter(|c| c.course_type != CourseType::Exam)
        .collect();

    let mut merged_courses = regular_courses;
    merged_courses.extend(exams.clone());

    // 保存更新后的课表
    let now = Utc::now().timestamp();
    let updated_schedule = CachedSchedule {
        id: schedule_id.clone(),
        name: schedule.name,
        courses: merged_courses.clone(),
        timestamp: now,
        expire_time: schedule.expire_time,
        first_day: schedule.first_day,
        max_periods: schedule.max_periods,
        weeks_count: schedule.weeks_count,
        time_table_id: schedule.time_table_id,
        sort_index: schedule.sort_index,
        school_year: schedule.school_year,
        school_term: schedule.school_term,
    };

    storage.save_schedule(&updated_schedule)?;

    println!("成功导入 {} 门考试到课表", exams.len());
    Ok(exams)
}
