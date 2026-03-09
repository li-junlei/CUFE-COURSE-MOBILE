// 导入导出逻辑 Composable
import { invoke } from '@tauri-apps/api/core';
import type { ScheduleMetadata } from '../types';

/**
 * 导出课表到文件
 */
export async function exportSchedule(schedule: ScheduleMetadata): Promise<boolean> {
  try {
    const jsonStr = await invoke<string>('export_schedule_json', { scheduleId: schedule.id });
    const blob = new Blob([jsonStr], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    
    const a = document.createElement('a');
    a.href = url;
    a.download = `${schedule.name}.json`;
    a.click();
    
    setTimeout(() => URL.revokeObjectURL(url), 100);
    return true;
  } catch (error) {
    console.error('导出课表失败:', error);
    return false;
  }
}

/**
 * 从文件导入课表
 */
export async function importScheduleFromFile(): Promise<string | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json,application/json';
    input.onchange = async (event: any) => {
      const file = event.target.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      try {
        const text = await file.text();
        const scheduleId = await invoke<string>('import_schedule_json', { jsonStr: text });
        resolve(scheduleId);
      } catch (error) {
        console.error('导入课表失败:', error);
        resolve(null);
      }
    };
    input.click();
  });
}

/**
 * 从教务系统导入课表（带自动重登录）
 */
export async function importScheduleWithAutoRelogin(
  year: number,
  term: number,
  scheduleName: string
): Promise<string> {
  return await invoke<string>('import_schedule_with_auto_relogin', {
    year,
    term,
    scheduleName,
  });
}

/**
 * 更新课表（在线更新）
 */
export async function updateSchedule(scheduleId: string) {
  return await invoke<{ added_count: number; removed_count: number; modified_count: number; unchanged_count: number }>('update_schedule_with_diff', {
    scheduleId,
  });
}

/**
 * 刷新课表数据
 */
export async function refreshSchedule(): Promise<any[]> {
  return await invoke('refresh_schedule');
}

/**
 * 导入考试信息
 */
export async function importExams(scheduleId: string): Promise<any[]> {
  return await invoke('fetch_and_import_exams', { scheduleId });
}

/**
 * 上传背景图
 */
export async function uploadBackground(): Promise<string | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/jpeg, image/png, image/webp';
    input.onchange = async (event: any) => {
      const file = event.target.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }
      try {
        const buffer = await file.arrayBuffer();
        const bytes = Array.from(new Uint8Array(buffer));
        const storedPath = await invoke<string>('upload_background_image', { bytes });
        resolve(storedPath);
      } catch (error) {
        console.error('上传背景图失败:', error);
        resolve(null);
      }
    };
    input.click();
  });
}

/**
 * 删除背景图
 */
export async function deleteBackground(): Promise<void> {
  await invoke('delete_background_image');
}

export function useImportExport() {
  return {
    exportSchedule,
    importScheduleFromFile,
    importScheduleWithAutoRelogin,
    updateSchedule,
    refreshSchedule,
    importExams,
    uploadBackground,
    deleteBackground,
  };
}
