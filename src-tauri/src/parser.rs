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

        // 提取周次信息 (如 "4-5周,7-18周")
        let zcd = item.get("zcd")
            .and_then(|v| v.as_str())
            .unwrap_or("1-18周");

        let weeks = parse_week_string(zcd);

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
            week_type: 0, // 默认为全周
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

/// 解析周次字符串 (如 "4-5周,7-18周", "3-18周", "2周,6周")
fn parse_week_string(week_str: &str) -> Vec<i32> {
    let mut weeks = Vec::new();

    // 移除"周"字，然后按逗号分割
    let clean = week_str.replace("周", "");
    let parts: Vec<&str> = clean.split(',').collect();

    for part in parts {
        let part = part.trim();
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

    // 去重并排序
    weeks.sort();
    weeks.dedup();

    weeks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_course_html() {
        let html = r#"
        <!DOCTYPE html>
        <html>
        <body>
            <table class="timetable">
                <thead>
                    <tr><th colspan="9">Course Schedule</th></tr>
                    <tr><th>Section</th><th>Time</th><th>Mon</th><th>Tue</th><th>Wed</th><th>Thu</th><th>Fri</th><th>Sat</th><th>Sun</th></tr>
                </thead>
                <tbody>
                    <tr>
                        <td rowspan="4">上午</td>
                        <td>1</td>
                        <td>
                            <div class="timetable_con">
                                <span class="title">高等数学★</span>
                                <p title="教师">张三</p>
                                <p>1-16周</p>
                                <p title="上课地点">主教101</p>
                            </div>
                        </td>
                        <td>
                            <div class="timetable_con">
                                <span class="title">Test Course</span>
                                <p title="教师">Smith</p>
                                <p>2-4周</p>
                                <p title="上课地点">Room 202</p>
                            </div>
                        </td>
                        <td></td><td></td><td></td><td></td><td></td>
                    </tr>
                </tbody>
            </table>
        </body>
        </html>
        "#;

        let courses = parse_course_html(html).expect("Failed to parse");
        println!("Extracted courses: {:?}", courses);
        // Note: The parser logic for "Mon"/"Tue" depends on specific Chinese strings "周一", "周二" etc in header or assumed order.
        // My parser code: `let day_list = vec!["周一", "周二", ...];`
        // And it finds period columns based on skipping first 1 or 2 cols.
        // Wait, the parser logic iterates `cells` starting from index.
        // It assumes 3rd row is data.
        // And it maps columns to days using `col_rowspan` and `day_idx`.
        // BUT `day_list` is just a list of names to assign.
        // The parser logic does NOT look at the Table Header to determine which column is Monday.
        // It assumes standard layout: Period Col -> Mon -> Tue -> ...
        
        // In my mock HTML above, I have `<td>1</td>` (Period) then `<td>...` (Mon).
        // Let's verify parser logic:
        // `is_time_slot` check: "上午" is in first cell? Yes.
        // `start_cell_idx = 2`. Correct.
        // `cells.skip(2)` -> Mon, Tue...
        
        // So first course (Mon) should be parsed.
        // Second course (Tue) should be parsed.
        
        assert!(courses.len() >= 1);
        let c1 = &courses[0];
        assert_eq!(c1.name, "高等数学");
        assert_eq!(c1.day_of_week, 1); // Mon
        
        if courses.len() > 1 {
            let c2 = &courses[1];
            assert_eq!(c2.name, "Test Course");
            assert_eq!(c2.day_of_week, 2); // Tue
        }
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
