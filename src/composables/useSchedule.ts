// 课表视图状态管理 Composable
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { ScheduleMetadata, Course } from '../types';

const scheduleList = ref<ScheduleMetadata[]>([]);
const currentScheduleId = ref<string>();
const currentWeek = ref(1);
const courses = ref<Course[]>([]);
const loading = ref(false);

/**
 * 当前激活的课表元数据
 */
export const activeSchedule = computed(() => {
  return scheduleList.value.find(s => s.id === currentScheduleId.value);
});

/**
 * 加载课表列表
 */
export async function loadScheduleList(): Promise<ScheduleMetadata[]> {
  try {
    const list = await invoke<ScheduleMetadata[]>('list_schedules');
    scheduleList.value = list;
    return list;
  } catch (error) {
    console.error('加载课表列表失败:', error);
    return [];
  }
}

/**
 * 加载课表数据
 */
export async function loadSchedule(scheduleId?: string): Promise<Course[]> {
  loading.value = true;
  try {
    const loadedCourses = await invoke<Course[]>('load_cached_schedule', { scheduleId: scheduleId || null });
    courses.value = loadedCourses;
    if (scheduleId) {
      currentScheduleId.value = scheduleId;
    }
    return loadedCourses;
  } catch (error) {
    console.error('加载课表失败:', error);
    return [];
  } finally {
    loading.value = false;
  }
}

/**
 * 删除课表
 */
export async function deleteSchedule(scheduleId: string): Promise<void> {
  await invoke('delete_schedule', { scheduleId });
  // 如果删除的是当前选中的课表，清空选中状态
  if (currentScheduleId.value === scheduleId) {
    currentScheduleId.value = undefined;
    courses.value = [];
  }
  // 刷新列表
  await loadScheduleList();
}

/**
 * 切换当前课表
 */
export async function switchSchedule(scheduleId: string): Promise<void> {
  await invoke('switch_schedule', { scheduleId });
  currentScheduleId.value = scheduleId;
  // 重新加载课表数据
  await loadSchedule(scheduleId);
}

/**
 * 重新排序课表
 */
export async function reorderSchedules(sortedIds: string[]): Promise<void> {
  await invoke('reorder_schedules', { sortedIds });
  await loadScheduleList();
}

/**
 * 计算当前周次
 */
export function recalculateCurrentWeek(firstDay: number | undefined, weeksCount: number | undefined): void {
  if (firstDay) {
    const week = Math.floor((Date.now() - firstDay * 1000) / (7 * 24 * 60 * 60 * 1000)) + 1;
    const maxWeeks = weeksCount || 20;
    currentWeek.value = Math.max(Math.min(week, maxWeeks), 1);
  } else {
    currentWeek.value = 1;
  }
}

/**
 * 计算学期状态
 */
export function getSemesterStatus(
  firstDay: number | undefined,
  weeksCount: number | undefined
): { type: 'not-started' | 'ended' | null; text: string } | null {
  if (!firstDay) return null;

  const startTime = firstDay * 1000;
  const endTime = startTime + (weeksCount || 20) * 7 * 24 * 60 * 60 * 1000;
  const now = Date.now();

  if (now < startTime) {
    return { type: 'not-started', text: '未开学' };
  } else if (now > endTime) {
    return { type: 'ended', text: '学期已结束' };
  }
  return null; // 学期进行中
}

export function useSchedule() {
  return {
    scheduleList,
    currentScheduleId,
    currentWeek,
    courses,
    loading,
    activeSchedule,
    loadScheduleList,
    loadSchedule,
    deleteSchedule,
    switchSchedule,
    reorderSchedules,
    recalculateCurrentWeek,
    getSemesterStatus,
  };
}
