// 日期模块 - 日期计算相关命令

use chrono::{Utc, Duration, Datelike};

/// 获取当前周次
#[tauri::command]
pub fn get_current_week(first_day: Option<i64>) -> i32 {
    if let Some(first_day) = first_day {
        let now = Utc::now().timestamp();
        let weeks = ((now - first_day) / (7 * 24 * 60 * 60)) + 1;
        weeks.max(1) as i32
    } else {
        1
    }
}

/// 计算指定周次和星期的日期
/// 返回格式: "M/D" (如 "1/15")
#[tauri::command]
pub fn calculate_date(first_day: i64, target_week: i32, target_day: i32) -> Result<String, String> {
    if target_week < 1 {
        return Err("目标周次必须大于 0".to_string());
    }
    if !(1..=7).contains(&target_day) {
        return Err("目标星期必须在 1 到 7 之间".to_string());
    }

    let first_date = chrono::DateTime::<Utc>::from_timestamp(first_day, 0)
        .ok_or("无效的学期开始时间戳")?
        .date_naive();

    // 计算目标日期
    let target_date = first_date
        + Duration::weeks((target_week - 1) as i64)
        + Duration::days((target_day - 1) as i64);

    Ok(format!("{}/{}", target_date.month(), target_date.day()))
}
