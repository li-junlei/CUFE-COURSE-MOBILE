// 课表业务逻辑服务

use crate::models::{Course, ScheduleDiff, CourseType};
use std::collections::HashMap;

/// 计算两个课表之间的差异
pub fn calculate_schedule_diff(
    old_courses: &[Course],
    new_courses: &[Course],
) -> ScheduleDiff {
    // 为课程生成唯一键 (name + day_of_week + periods)
    fn course_key(course: &Course) -> String {
        format!("{}|{}|{:?}", course.name, course.day_of_week, course.periods)
    }

    // 构建课程映射
    let old_map: HashMap<String, &Course> = old_courses
        .iter()
        .map(|c| (course_key(c), c))
        .collect();
    let new_map: HashMap<String, &Course> = new_courses
        .iter()
        .map(|c| (course_key(c), c))
        .collect();

    let mut added_count = 0;
    let mut removed_count = 0;
    let mut modified_count = 0;

    // 查找新增和修改的课程
    for (key, new_course) in &new_map {
        match old_map.get(key) {
            Some(old_course) => {
                // 检查是否有变化（排除时间的字段）
                if old_course.teacher != new_course.teacher
                    || old_course.location != new_course.location
                    || old_course.weeks != new_course.weeks
                    || old_course.week_type != new_course.week_type
                {
                    modified_count += 1;
                }
            }
            None => added_count += 1,
        }
    }

    // 查找删除的课程
    for (key, course) in &old_map {
        if !new_map.contains_key(key) {
            // 如果是考试，不计入删除（因为教务系统数据本就不包含考试）
            if course.course_type == CourseType::Exam {
                continue;
            }
            removed_count += 1;
        }
    }

    let unchanged_count = new_courses.len() - added_count - modified_count;

    ScheduleDiff {
        added_count,
        removed_count,
        modified_count,
        unchanged_count,
    }
}
