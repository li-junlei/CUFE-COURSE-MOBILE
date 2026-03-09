import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Course, ScheduleMetadata } from '../types';

export function useCourse() {
  const courses = ref<Course[]>([]);
  const loading = ref(false);
  const error = ref<string | null>(null);

  async function loadCachedSchedule(scheduleId?: string): Promise<boolean> {
    try {
      const result = await invoke<Course[]>('load_cached_schedule', {
        scheduleId,
      });
      courses.value = result;
      return true;
    } catch {
      return false;
    }
  }

  async function saveScheduleCache(
    coursesData: Course[],
    name: string,
    scheduleId?: string,
    firstDay?: number,
    maxPeriods?: number,
    weeksCount?: number,
    timeTableId?: string,
  ): Promise<string> {
    try {
      return await invoke<string>('save_schedule_cache', {
        courses: coursesData,
        name,
        scheduleId,
        firstDay,
        maxPeriods,
        weeksCount,
        timeTableId,
      });
    } catch (e) {
      console.error('保存缓存失败:', e);
      throw e;
    }
  }

  async function updateScheduleInfo(
    scheduleId: string,
    firstDay?: number,
    maxPeriods?: number,
    weeksCount?: number,
    timeTableId?: string,
  ): Promise<void> {
    try {
      await invoke('update_schedule_info', {
        scheduleId,
        firstDay,
        maxPeriods,
        weeksCount,
        timeTableId,
      });
    } catch (e) {
      console.error('更新课表信息失败:', e);
      throw e;
    }
  }

  async function applySettingsToAll(sourceScheduleId: string): Promise<void> {
    try {
      await invoke('apply_settings_to_all', { sourceScheduleId });
    } catch (e) {
      console.error('应用设置到全部失败:', e);
      throw e;
    }
  }

  async function renameSchedule(scheduleId: string, newName: string): Promise<void> {
    try {
      await invoke('rename_schedule', { scheduleId, newName });
    } catch (e) {
      console.error('重命名课表失败:', e);
      throw e;
    }
  }

  async function reorderSchedules(sortedIds: string[]): Promise<void> {
    try {
      await invoke('reorder_schedules', { sortedIds });
    } catch (e) {
      console.error('重新排序失败:', e);
      throw e;
    }
  }

  async function listSchedules(): Promise<ScheduleMetadata[]> {
    try {
      return await invoke<ScheduleMetadata[]>('list_schedules');
    } catch (e) {
      console.error('获取课表列表失败:', e);
      throw e;
    }
  }

  async function deleteSchedule(scheduleId: string): Promise<void> {
    try {
      await invoke('delete_schedule', { scheduleId });
    } catch (e) {
      console.error('删除课表失败:', e);
      throw e;
    }
  }

  async function switchSchedule(scheduleId: string): Promise<void> {
    try {
      await invoke('switch_schedule', { scheduleId });
    } catch (e) {
      console.error('切换课表失败:', e);
      throw e;
    }
  }

  const maxWeek = computed(() => {
    let max = 0;
    for (const course of courses.value) {
      if (course.weeks?.length) {
        max = Math.max(max, Math.max(...course.weeks));
      }
    }
    return max || 20;
  });

  return {
    courses,
    loading,
    error,
    loadCachedSchedule,
    saveScheduleCache,
    listSchedules,
    deleteSchedule,
    switchSchedule,
    updateScheduleInfo,
    applySettingsToAll,
    renameSchedule,
    reorderSchedules,
    maxWeek,
  };
}