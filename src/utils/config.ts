// 配置默认值工具函数
import type { AppConfig } from '../types';

/**
 * 获取默认应用配置
 */
export function getDefaultConfig(): AppConfig {
  return {
    first_day: undefined,
    max_periods: 13,
    end_week: 20,
    period_times: undefined,
    show_grid_lines: false,
    show_teacher: true,
    show_location: true,
    simplified_location: false,
    card_opacity: 90,
    background_image: undefined,
    current_schedule_id: undefined,
    edu_systems: undefined,
    last_edu_system_id: undefined,
    edu_system_url: undefined,

    reminder_enabled: false,
    reminder_debug_logging: false,
    time_tables: undefined,
  };
}

/**
 * 合并配置（保留现有值，使用默认值填充缺失值）
 */
export function mergeConfig(existing: AppConfig, defaults: AppConfig = getDefaultConfig()): AppConfig {
  return {
    ...defaults,
    ...existing,
  };
}
