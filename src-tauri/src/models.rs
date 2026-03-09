use serde::{Deserialize, Serialize};

/// 课程类型枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CourseType {
    Regular,  // 常规课程
    Exam,     // 考试
}

impl Default for CourseType {
    fn default() -> Self {
        CourseType::Regular
    }
}

/// 考试详细信息
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExamInfo {
    pub date: String,         // 考试日期 "2026-01-06"
    pub start_time: String,   // 开始时间 "10:00"
    pub end_time: String,     // 结束时间 "11:40"
    pub exam_name: String,    // 考试名称 "25-26-1期末考试"
}

/// 课程数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Course {
    /// 课程名称
    pub name: String,
    /// 教师名称
    pub teacher: String,
    /// 周次范围 [开始周, 结束周, 开始周2(可选), 结束周2(可选)]
    pub weeks: Vec<i32>,
    /// 单双周标记 (1=单周, 2=双周, 0=全周)
    pub week_type: i32,
    /// 星期 (1-7)
    pub day_of_week: i32,
    /// 节次范围 [开始节, 结束节]
    pub periods: Vec<i32>,
    /// 教室/地点
    pub location: String,

    /// 课程类型（新增，向后兼容）
    #[serde(default)]
    pub course_type: CourseType,

    /// 考试详细信息（仅考试类型有效）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exam_info: Option<ExamInfo>,
}

/// 用户凭证
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub password: String,
}

/// 登录初始化参数
/// 登录响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub success: bool,
    pub message: String,
    pub cookie: Option<String>,
}

/// 用户个人信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub student_number: String,  // 学号
    pub name: String,            // 姓名
    pub department: String,      // 学院
    pub class_name: String,      // 班级
    pub grade: String,           // 年级
    pub major: String,           // 专业
    pub gender: String,          // 性别
    pub photo_url: Option<String>, // 照片URL
}

/// 登录会话检查结果
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionStatus {
    pub logged_in: bool,
    pub relogin_performed: bool,
    pub user_info: Option<UserInfo>,
}

/// 时间表
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeTable {
    pub id: String,
    pub name: String,
    pub periods: Vec<PeriodTime>,
}

/// 教务系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EduSystem {
    pub id: String,
    pub name: String,
    pub url: String,
    pub parser_type: String,
    pub enabled: bool,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub background_image: Option<String>,
    pub first_day: Option<i64>, // 全局默认第一天 (旧配置兼容)
    pub end_week: Option<i32>,  // 全局默认结束周 (旧配置兼容)
    pub max_periods: Option<i32>, // 全局默认最大节次 (旧配置兼容)
    pub period_times: Option<Vec<PeriodTime>>, // 旧的时间表配置 (兼容)
    pub time_tables: Option<Vec<TimeTable>>, // 新的多时间表列表
    pub edu_system_url: Option<String>, // 教务系统地址 (deprecated, 使用 edu_systems)
    pub edu_systems: Option<Vec<EduSystem>>, // 支持的教务系统列表
    pub last_edu_system_id: Option<String>, // 用户上次选择的教务系统 ID
    pub current_schedule_id: Option<String>, // 当前选中的课表ID
    pub show_grid_lines: Option<bool>, // 显示网格辅助线
    pub card_opacity: Option<i32>, // 课程卡片不透明度 (0-100)
    pub show_teacher: Option<bool>, // 在卡片中显示教师
    pub show_location: Option<bool>, // 在卡片中显示上课地点
    pub simplified_location: Option<bool>, // 简化地点显示
    pub reminder_enabled: Option<bool>, // 启用上课提醒
    pub reminder_debug_logging: Option<bool>, // 启用提醒调试日志
    pub auto_check_update: Option<bool>, // 自动检查更新
    pub skipped_version: Option<String>, // 跳过的版本号
    pub close_action_minimize_to_tray: Option<bool>, // 关闭主界面时：true=最小化到托盘, false=直接退出
}

/// 时间段(包含开始和结束时间)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodTime {
    pub start: String, // 开始时间，格式: "8:00"
    pub end: String,   // 结束时间，格式: "8:45"
}

/// 课表元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleMetadata {
    pub id: String,              // 课表唯一ID
    pub name: String,            // 课表名称
    pub created_at: i64,         // 创建时间
    pub updated_at: i64,         // 更新时间
    pub course_count: usize,     // 课程数量
    pub first_day: Option<i64>,  // 第一周第一天的时间戳
    pub max_periods: Option<i32>, // 最大节次
    pub weeks_count: Option<i32>, // 学期周数 (默认20)
    pub time_table_id: Option<String>, // 关联的时间表ID
    pub sort_index: Option<i32>, // 排序索引
    pub school_year: Option<i32>,   // 学年 (如 2024)
    pub school_term: Option<i32>,   // 学期 (1, 2, 3)
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            background_image: None,
            first_day: None,
            end_week: None,
            max_periods: Some(13),
            period_times: Some(vec![
                PeriodTime { start: "8:00".to_string(), end: "8:45".to_string() },
                PeriodTime { start: "8:55".to_string(), end: "9:40".to_string() },
                PeriodTime { start: "10:00".to_string(), end: "10:45".to_string() },
                PeriodTime { start: "10:55".to_string(), end: "11:40".to_string() },
                PeriodTime { start: "11:50".to_string(), end: "12:35".to_string() },
                PeriodTime { start: "12:45".to_string(), end: "13:30".to_string() },
                PeriodTime { start: "14:00".to_string(), end: "14:45".to_string() },
                PeriodTime { start: "14:55".to_string(), end: "15:40".to_string() },
                PeriodTime { start: "16:00".to_string(), end: "16:45".to_string() },
                PeriodTime { start: "16:55".to_string(), end: "17:40".to_string() },
                PeriodTime { start: "17:50".to_string(), end: "18:35".to_string() },
                PeriodTime { start: "19:20".to_string(), end: "20:05".to_string() },
                PeriodTime { start: "20:15".to_string(), end: "21:00".to_string() },
            ]),
            time_tables: Some(vec![]),
            edu_system_url: Some("https://xuanke.cufe.edu.cn/jwglxt/".to_string()),
            edu_systems: Some(vec![
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
                }
            ]),
            last_edu_system_id: Some("cufe".to_string()),
            current_schedule_id: None,
            show_grid_lines: Some(false),
            card_opacity: Some(95), // 默认 95% 不透明度
            show_teacher: Some(true), // 默认显示教师
            show_location: Some(true), // 默认显示地点
            simplified_location: Some(false), // 默认不简化地点
            reminder_enabled: Some(false), // 默认关闭提醒
            reminder_debug_logging: Some(false), // 默认关闭提醒调试日志
            auto_check_update: Some(true), // 默认开启自动检查更新
            skipped_version: None, // 默认不跳过任何版本
            close_action_minimize_to_tray: Some(true), // 默认最小化到托盘
        }
    }
}

/// 缓存的课表数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedSchedule {
    pub id: String,              // 课表ID
    pub name: String,            // 课表名称
    pub courses: Vec<Course>,
    pub timestamp: i64,
    pub expire_time: i64,
    pub first_day: Option<i64>,  // 第一周第一天的时间戳
    pub max_periods: Option<i32>, // 最大节次
    pub weeks_count: Option<i32>, // 学期周数
    pub time_table_id: Option<String>, // 关联的时间表ID
    pub sort_index: Option<i32>, // 排序索引
    pub school_year: Option<i32>,   // 学年
    pub school_term: Option<i32>,   // 学期
}

impl CachedSchedule {
    /// 检查缓存是否过期（30天）
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp();
        now > self.expire_time
    }
}

/// ============================================================
/// 持久化登录相关数据结构
/// ============================================================

/// 持久化凭证 (加密存储)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentCredentials {
    pub username: String,              // 学号（明文）
    pub password_encrypted: String,    // 密码（DPAPI 加密后的 Base64）
    pub edu_system_url: String,        // 教务系统 URL
    pub saved_at: i64,                 // 保存时间戳
}

/// ============================================================
/// 课表更新相关数据结构
/// ============================================================

/// 课表更新差异统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDiff {
    pub added_count: usize,        // 新增课程数
    pub removed_count: usize,      // 删除课程数
    pub modified_count: usize,     // 修改课程数
    pub unchanged_count: usize,    // 未变课程数
}
