// CUFE 教务系统仅支持 JSON 格式，不再支持 HTML 解析
// 请使用 parse_cufe_json() 解析课表数据

use crate::models::Course;
use chrono::Datelike;

/// 解析 CUFE JSON 格式的课表数据
/// CUFE 教务系统返回的是 JSON 而不是 HTML
pub fn parse_cufe_json(json_text: &str) -> Result<Vec<Course>, String> {
    use serde_json::Value;

    println!("=== 开始解析 CUFE JSON 课表 ===");
    println!("原始响应长度: {} 字节", json_text.len());

    // 打印前200个字符用于调试
    let preview = if json_text.len() > 200 {
        &json_text[..200]
    } else {
        json_text
    };
    println!("原始响应预览:\n{}", preview);

    // 尝试移除BOM和其他可能的干扰字符
    let cleaned_text = json_text.trim().trim_start_matches('\u{feff}').trim_start_matches('\u{200b}');

    // 检查是否是HTML响应（错误页面）
    if cleaned_text.starts_with("<!DOCTYPE") || cleaned_text.starts_with("<html") || cleaned_text.starts_with("<HTML") {
        return Err("服务器返回了HTML页面而不是JSON，可能是登录已失效".to_string());
    }

    // 解析 JSON
    let json: Value = serde_json::from_str(cleaned_text)
        .map_err(|e| {
            // 提供更详细的错误信息
            let error_preview = if cleaned_text.len() > 100 {
                &cleaned_text[..100]
            } else {
                cleaned_text
            };
            format!("解析JSON失败: {}\n实际内容前100字符: {}", e, error_preview)
        })?;

    // 提取 kbList 数组
    let kb_list = json.get("kbList")
        .and_then(|v| v.as_array())
        .ok_or("JSON中未找到kbList字段，可能是API返回格式已变更")?;

    println!("找到 {} 条课程记录", kb_list.len());

    let mut courses = Vec::new();

    for item in kb_list {
        // 提取课程基本信息
        let course_name = item.get("kcmc")
            .and_then(|v| v.as_str())
            .unwrap_or("未知课程")
            .to_string();

        let teacher = item.get("xm")
            .and_then(|v| v.as_str())
            .unwrap_or("未指定")
            .to_string();

        let classroom = item.get("cdmc")
            .and_then(|v| v.as_str())
            .unwrap_or("未指定")
            .to_string();

        // 提取星期信息
        let xqjmc = item.get("xqjmc")
            .and_then(|v| v.as_str())
            .unwrap_or("星期一");

        let day_of_week = match xqjmc {
            "星期一" => 1,
            "星期二" => 2,
            "星期三" => 3,
            "星期四" => 4,
            "星期五" => 5,
            "星期六" => 6,
            "星期日" => 7,
            _ => 1,
        };

        // 提取节次信息 (如 "3-4节")
        let jc = item.get("jc")
            .and_then(|v| v.as_str())
            .unwrap_or("1-2节");

        let periods = parse_period_string(jc);

        // 提取周次信息 (如 "4-5周,7-18周", "1-15周(单)", "2-16周(双)")
        let zcd = item.get("zcd")
            .and_then(|v| v.as_str())
            .unwrap_or("1-18周");

        // 解析周次与单双周类型：(单)=单周, (双)=双周, 无标记=全周
        let (weeks, week_type) = parse_week_string(zcd);

        // 检查是否有课程类型符号 (xslxbj: "★", "○" 等)
        // 如果没有，默认为讲课类型
        let _course_type_sym = item.get("xslxbj")
            .and_then(|v| v.as_str())
            .unwrap_or("★");

        // 从课程名称中移除课程类型符号
        let course_name_clean = course_name.trim_end_matches(|c: char| {
            matches!(c, '★' | '○' | '◆' | '◇' | '●')
        }).trim().to_string();

        println!("解析课程: {} - {} - {} - {}", course_name_clean, xqjmc, jc, zcd);

        courses.push(Course {
            name: course_name_clean,
            teacher,
            weeks,
            week_type,
            day_of_week,
            periods,
            location: classroom,
            course_type: crate::models::CourseType::Regular,
            exam_info: None,
        });
    }

    println!("成功解析 {} 条课程记录", courses.len());
    Ok(courses)
}

/// 解析节次字符串 (如 "3-4节", "9-11节")
fn parse_period_string(period_str: &str) -> Vec<i32> {
    // 移除"节"字
    let clean = period_str.replace("节", "").trim().to_string();

    if clean.contains('-') {
        // 格式如 "3-4"
        let parts: Vec<&str> = clean.split('-').collect();
        if parts.len() == 2 {
            if let (Ok(start), Ok(end)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
                if start <= end {
                    return (start..=end).collect();
                }
            }
        }
    }

    // 单个节次或解析失败，返回默认值
    if let Ok(p) = clean.parse::<i32>() {
        vec![p]
    } else {
        vec![1, 2] // 默认第1-2节
    }
}

/// 解析周次字符串 (如 "4-5周,7-18周", "1-15周(单)", "2-16周(双)", "2周,6周")
/// 返回 (具体周次列表, 单双周类型)
/// week_type: 0=全周, 1=单周(奇数周), 2=双周(偶数周)
/// 正方教务系统会在 zcd 末尾追加 (单)/(双)/(单双) 标记表示单双周安排
fn parse_week_string(week_str: &str) -> (Vec<i32>, i32) {
    let mut weeks = Vec::new();

    // 识别单双周标记（兼容全角括号）
    let has_odd = week_str.contains("(单)") || week_str.contains("（单）");
    let has_even = week_str.contains("(双)") || week_str.contains("（双）");
    let week_type = if has_odd && !has_even {
        1 // 仅单周
    } else if has_even && !has_odd {
        2 // 仅双周
    } else {
        0 // 无标记，或同时含单双（如"(单双)"）视为全周
    };

    // 移除"周"字与各种单双周括号标记，再按逗号分割
    let clean = week_str
        .replace("周", "")
        .replace("(单双)", "")
        .replace("（单双）", "")
        .replace("(单)", "")
        .replace("（单）", "")
        .replace("(双)", "")
        .replace("（双）", "");
    let parts: Vec<&str> = clean.split(',').collect();

    for part in parts {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.contains('-') {
            // 范围，如 "4-5"
            let range: Vec<&str> = part.split('-').collect();
            if range.len() == 2 {
                if let (Ok(start), Ok(end)) = (range[0].parse::<i32>(), range[1].parse::<i32>()) {
                    if start <= end {
                        for w in start..=end {
                            weeks.push(w);
                        }
                    }
                }
            }
        } else {
            // 单个周次，如 "2"
            if let Ok(w) = part.parse::<i32>() {
                weeks.push(w);
            }
        }
    }

    // 根据单双周类型过滤：单周保留奇数，双周保留偶数
    if week_type == 1 {
        weeks.retain(|w| w % 2 == 1);
    } else if week_type == 2 {
        weeks.retain(|w| w % 2 == 0);
    }

    // 去重并排序
    weeks.sort();
    weeks.dedup();

    (weeks, week_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_week_string_normal() {
        // 普通全周
        let (weeks, wt) = parse_week_string("1-16周");
        assert_eq!(wt, 0);
        assert_eq!(weeks, (1..=16).collect::<Vec<_>>());
    }

    #[test]
    fn test_parse_week_string_multi_range() {
        // 多段范围
        let (weeks, wt) = parse_week_string("4-5周,7-18周");
        assert_eq!(wt, 0);
        let mut expected: Vec<i32> = (4..=5).chain(7..=18).collect();
        expected.sort();
        expected.dedup();
        assert_eq!(weeks, expected);
    }

    #[test]
    fn test_parse_week_string_discrete() {
        // 离散周次
        let (weeks, wt) = parse_week_string("2周,6周");
        assert_eq!(wt, 0);
        assert_eq!(weeks, vec![2, 6]);
    }

    #[test]
    fn test_parse_week_string_odd_marker() {
        // 单周标记：1-15周(单) -> 仅奇数周
        let (weeks, wt) = parse_week_string("1-15周(单)");
        assert_eq!(wt, 1);
        assert_eq!(weeks, vec![1, 3, 5, 7, 9, 11, 13, 15]);
    }

    #[test]
    fn test_parse_week_string_even_marker() {
        // 双周标记：2-16周(双) -> 仅偶数周
        let (weeks, wt) = parse_week_string("2-16周(双)");
        assert_eq!(wt, 2);
        assert_eq!(weeks, vec![2, 4, 6, 8, 10, 12, 14, 16]);
    }

    #[test]
    fn test_parse_week_string_fullwidth_marker() {
        // 全角括号兼容
        let (weeks, wt) = parse_week_string("1-15周（单）");
        assert_eq!(wt, 1);
        assert_eq!(weeks, vec![1, 3, 5, 7, 9, 11, 13, 15]);
    }

    #[test]
    fn test_parse_cufe_json_with_odd_even_weeks() {
        // 模拟正方教务系统返回的单双周课程（参考真实抓包）
        let json = r#"{"kbList":[
            {"kcmc":"财务报表分析","xm":"卢钧","cdmc":"主教503","xqjmc":"星期一","jc":"3-5节","zcd":"1-16周","xslxbj":"★"},
            {"kcmc":"应用计量经济学","xm":"王凯","cdmc":"主教319","xqjmc":"星期五","jc":"3-5节","zcd":"1-15周(单)","xslxbj":"★"},
            {"kcmc":"应用计量经济学","xm":"王凯","cdmc":"实验楼404","xqjmc":"星期五","jc":"3-5节","zcd":"2-16周(双)","xslxbj":"○"}
        ]}"#;
        let courses = parse_cufe_json(json).expect("parse failed");
        assert_eq!(courses.len(), 3);

        // 全周课程
        assert_eq!(courses[0].name, "财务报表分析");
        assert_eq!(courses[0].week_type, 0);
        assert_eq!(courses[0].weeks.len(), 16);

        // 单周课程（讲课）
        assert_eq!(courses[1].name, "应用计量经济学");
        assert_eq!(courses[1].week_type, 1);
        assert_eq!(courses[1].weeks, vec![1, 3, 5, 7, 9, 11, 13, 15]);
        assert_eq!(courses[1].location, "主教319");

        // 双周课程（实验）
        assert_eq!(courses[2].week_type, 2);
        assert_eq!(courses[2].weeks, vec![2, 4, 6, 8, 10, 12, 14, 16]);
        assert_eq!(courses[2].location, "实验楼404");
    }

}

/// ============================================================
/// 考试数据解析
/// ============================================================

/// 解析 CUFE JSON 格式的考试数据并转换为 Course 列表
/// 参数：
/// - exam_json: 考试 JSON 数据
/// - semester_start_date: 学期开始日期（第一周周一），格式 "2025-09-01"
pub fn parse_exam_json(exam_json: &serde_json::Value, semester_start_date: &str) -> Result<Vec<Course>, String> {
    use chrono::NaiveDate;
    use crate::models::{CourseType, ExamInfo};

    println!("=== 开始解析考试数据 ===");

    // 解析学期开始日期
    let semester_start = NaiveDate::parse_from_str(semester_start_date, "%Y-%m-%d")
        .map_err(|e| format!("解析学期开始日期失败: {}", e))?;

    // 提取 items 数组
    let items = exam_json.get("items")
        .and_then(|v| v.as_array())
        .ok_or("JSON 中缺少 items 数组")?;

    if items.is_empty() {
        println!("未找到考试数据");
        return Ok(Vec::new());
    }

    println!("找到 {} 门考试", items.len());

    let mut exams: Vec<Course> = Vec::new();

    for (idx, item) in items.iter().enumerate() {
        // 提取字段
        let course_name = item.get("kcmc")
            .and_then(|v| v.as_str())
            .unwrap_or("未知课程");

        let exam_time_str = item.get("kssj")
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("考试 {} 缺少考试时间字段", idx + 1))?;

        let location = item.get("cdmc")
            .and_then(|v| v.as_str())
            .unwrap_or("未知地点");

        let exam_name = item.get("ksmc")
            .and_then(|v| v.as_str())
            .unwrap_or("考试");

        // 解析考试时间 "2026-01-06(10:00-11:40)"
        let (exam_date_str, start_time, end_time) = parse_exam_time(exam_time_str)?;

        // 解析考试日期
        let exam_date = NaiveDate::parse_from_str(&exam_date_str, "%Y-%m-%d")
            .map_err(|e| format!("解析考试日期失败: {}", e))?;

        // 计算星期几 (1=周一, 7=周日)
        let day_of_week = exam_date.weekday().num_days_from_monday() as i32 + 1;

        // 计算周次
        let days_diff = exam_date.signed_duration_since(semester_start).num_days();
        let week_number = (days_diff / 7) + 1;

        if week_number < 1 || week_number > 25 {
            println!("警告：考试 {} 的日期 {} 不在合理的学期范围内（第{}周）", course_name, exam_date_str, week_number);
        }

        // 映射时间到节次
        let periods = map_time_to_periods(&start_time, &end_time)?;

        println!("解析考试 {}: {} - {} 第{}周 周{} 第{:?}节",
            idx + 1, course_name, exam_date_str, week_number, day_of_week, periods);

        exams.push(Course {
            name: format!("【考试】{}", course_name),
            teacher: String::new(), // 考试没有教师信息
            weeks: vec![week_number as i32],
            week_type: 0,
            day_of_week,
            periods,
            location: location.to_string(),
            course_type: CourseType::Exam,
            exam_info: Some(ExamInfo {
                date: exam_date_str,
                start_time,
                end_time,
                exam_name: exam_name.to_string(),
            }),
        });
    }

    println!("成功解析 {} 门考试", exams.len());
    Ok(exams)
}

/// 解析考试时间字符串
/// 格式: "2026-01-06(10:00-11:40)"
/// 返回: (日期, 开始时间, 结束时间)
fn parse_exam_time(time_str: &str) -> Result<(String, String, String), String> {
    // 查找括号位置
    let open_paren = time_str.find('(')
        .ok_or_else(|| format!("考试时间格式错误，缺少括号: {}", time_str))?;
    let close_paren = time_str.find(')')
        .ok_or_else(|| format!("考试时间格式错误，缺少右括号: {}", time_str))?;

    // 提取日期部分
    let date = time_str[..open_paren].to_string();

    // 提取时间部分 "10:00-11:40"
    let time_range = &time_str[open_paren + 1..close_paren];

    // 分割开始和结束时间
    let time_parts: Vec<&str> = time_range.split('-').collect();
    if time_parts.len() != 2 {
        return Err(format!("考试时间范围格式错误: {}", time_range));
    }

    Ok((date, time_parts[0].to_string(), time_parts[1].to_string()))
}

/// 将考试时间映射到节次
/// 基于 CUFE 默认时间表
fn map_time_to_periods(start_time: &str, end_time: &str) -> Result<Vec<i32>, String> {
    // CUFE 默认时间表（参考 models.rs 中的 AppConfig::default）
    let time_slots = vec![
        ("08:00", "08:45", vec![1]),
        ("08:55", "09:40", vec![2]),
        ("10:00", "10:45", vec![3]),
        ("10:55", "11:40", vec![4]),
        ("11:50", "12:35", vec![5]),
        ("12:45", "13:30", vec![6]),
        ("14:00", "14:45", vec![7]),
        ("14:55", "15:40", vec![8]),
        ("16:00", "16:45", vec![9]),
        ("16:55", "17:40", vec![10]),
        ("17:50", "18:35", vec![11]),
        ("19:20", "20:05", vec![12]),
        ("20:15", "21:00", vec![13]),
    ];

    // 找到开始时间对应的节次
    let start_period = time_slots.iter()
        .find(|(slot_start, _, _)| start_time <= *slot_start)
        .or_else(|| time_slots.iter().find(|(slot_start, slot_end, _)| start_time >= *slot_start && start_time <= *slot_end))
        .map(|(_, _, periods)| periods[0])
        .unwrap_or(1);

    // 找到结束时间对应的节次
    let end_period = time_slots.iter()
        .rev()
        .find(|(_, slot_end, _)| end_time >= *slot_end)
        .or_else(|| time_slots.iter().rev().find(|(slot_start, slot_end, _)| end_time >= *slot_start && end_time <= *slot_end))
        .map(|(_, _, periods)| periods[0])
        .unwrap_or(13);

    // 如果找不到精确匹配，尝试估算
    let final_start = if start_period == 1 && start_time > "09:00" {
        // 如果开始时间在上午但不是第1节，估算节次
        if start_time >= "10:00" { 3 } else { 1 }
    } else {
        start_period
    };

    let final_end = if end_period == 13 && end_time < "20:00" {
        // 如果结束时间不在晚上，估算节次
        if end_time <= "12:00" { 4 } else if end_time <= "16:00" { 8 } else { 10 }
    } else {
        end_period
    };

    Ok(vec![final_start, final_end])
}
