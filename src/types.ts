/**
 * 课程类型
 */
export type CourseType = 'regular' | 'exam';

/**
 * 考试详细信息
 */
export interface ExamInfo {
  /** 考试日期 "2026-01-06" */
  date: string;
  /** 开始时间 "10:00" */
  startTime: string;
  /** 结束时间 "11:40" */
  endTime: string;
  /** 考试名称 "25-26-1期末考试" */
  examName: string;
}

/**
 * 课程数据结构
 * 注意：字段名使用 camelCase，与后端 Rust serde 序列化一致
 */
export interface Course {
  /** 课程名称 */
  name: string;
  /** 教师名称 */
  teacher: string;
  /** 周次范围 [开始周, 结束周, 开始周2(可选), 结束周2(可选)] */
  weeks: number[];
  /** 单双周标记 (1=单周, 2=双周, 0=全周) */
  weekType: number;
  /** 星期 (1-7) */
  dayOfWeek: number;
  /** 节次范围 [开始节, 结束节] */
  periods: number[];
  /** 教室/地点 */
  location: string;

  /** 课程类型（新增，向后兼容） */
  courseType?: CourseType;
  /** 考试详细信息（仅考试类型有效） */
  examInfo?: ExamInfo;
}

/**
 * 时间段(包含开始和结束时间)
 */
export interface PeriodTime {
  /** 开始时间，格式: "8:00" */
  start: string;
  /** 结束时间，格式: "8:45" */
  end: string;
}

/**
 * 时间表
 */
export interface TimeTable {
  id: string;
  name: string;
  periods: PeriodTime[];
}

/**
 * 教务系统配置
 */
export interface EduSystem {
  /** 系统唯一标识 */
  id: string;
  /** 系统显示名称 */
  name: string;
  /** 教务系统 URL */
  url: string;
  /** 解析器类型 */
  parser_type: string;
  /** 是否启用 */
  enabled: boolean;
}

/**
 * 应用配置
 */
export interface AppConfig {
  /** 背景图片文件名 */
  background_image?: string;
  /** 全局默认第一天 (旧配置兼容) */
  first_day?: number;
  /** 全局默认结束周 (旧配置兼容) */
  end_week?: number;
  /** 全局默认最大节次 (旧配置兼容) */
  max_periods?: number;
  /** 旧的时间表配置 (兼容) */
  period_times?: PeriodTime[];
  /** 新的多时间表列表 */
  time_tables?: TimeTable[];
  /** 教务系统地址 (deprecated, 使用 edu_systems) */
  edu_system_url?: string;
  /** 支持的教务系统列表 */
  edu_systems?: EduSystem[];
  /** 用户上次选择的教务系统 ID */
  last_edu_system_id?: string;
  /** 当前选中的课表ID */
  current_schedule_id?: string;
  /** 显示网格辅助线 */
  show_grid_lines?: boolean;
  /** 课程卡片不透明度 (0-100) */
  card_opacity?: number;
  /** 在卡片中显示教师 */
  show_teacher?: boolean;
  /** 在卡片中显示上课地点 */
  show_location?: boolean;
  /** 简化地点显示 (隐藏"沙河校区"等前缀) */
  simplified_location?: boolean;
  /** 启用上课提醒 */
  reminder_enabled?: boolean;
  /** 启用提醒调试日志 */
  reminder_debug_logging?: boolean;
  /** 已提醒的课程记录 (防止重复提醒) key: "dayOfWeek_periods", value: timestamp */
  reminded_courses?: Record<string, number>;
  /** 自动检查更新 */
  auto_check_update?: boolean;
  /** 跳过的版本号 */
  skipped_version?: string;
  /** 关闭主界面时：true=最小化到托盘, false=直接退出 */

}

/**
 * 课表元数据
 */
export interface ScheduleMetadata {
  /** 课表唯一ID */
  id: string;
  /** 课表名称 */
  name: string;
  /** 创建时间 */
  created_at: number;
  /** 更新时间 */
  updated_at: number;
  /** 课程数量 */
  course_count: number;
  /** 第一周第一天的时间戳 */
  first_day?: number;
  /** 最大节次 */
  max_periods?: number;
  /** 学期周数 (默认20) */
  weeks_count?: number;
  /** 关联的时间表ID */
  time_table_id?: string;
  /** 排序索引 */
  sort_index?: number;
  /** 学年 (如 2024) */
  school_year?: number;
  /** 学期 (1, 2, 3) */
  school_term?: number;
}

/**
 * 用户个人信息
 */
export interface UserInfo {
  /** 学号 */
  student_number: string;
  /** 姓名 */
  name: string;
  /** 学院 */
  department: string;
  /** 班级 */
  class_name: string;
  /** 年级 */
  grade: string;
  /** 专业 */
  major: string;
  /** 性别 */
  gender: string;
  /** 照片URL */
  photo_url?: string;
}

export interface SessionStatus {
  loggedIn: boolean;
  reloginPerformed: boolean;
  userInfo: UserInfo | null;
}

/**
 * 课表更新差异统计
 */
export interface ScheduleDiff {
  /** 新增课程数 */
  added_count: number;
  /** 删除课程数 */
  removed_count: number;
  /** 修改课程数 */
  modified_count: number;
  /** 未变课程数 */
  unchanged_count: number;
}

/** 软件更新信息 */
export interface UpdateInfo {
  /** 最新版本号 */
  version: string;
  /** 发布标题 */
  title: string;
  /** 发布说明 */
  releaseNotes: string;
  /** Release 页面链接 */
  releaseUrl: string;
}

