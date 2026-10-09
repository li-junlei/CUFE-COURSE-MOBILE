import { ref, onUnmounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { Course, TimeTable } from '../types';
import { cancel, pending, createChannel, Importance, Schedule, type Options } from '@tauri-apps/plugin-notification';
import { ElMessage } from 'element-plus';
import { buildReminderPlan, REMINDER_ID_BASE, REMINDER_ID_LIMIT } from '../utils/reminderPlan';

const CHANNEL_ID = 'course-reminders';
const isAndroid = () => /Android/i.test(navigator.userAgent);

export function useReminder() {
  const isRunning = ref(false);
  let timer: number | undefined;
  let revision = 0;
  let work = Promise.resolve();

  async function prepareNotifications(): Promise<boolean> {
    // Await the native command so delivery failures reach the caller.
    let granted = await invoke<boolean | null>('plugin:notification|is_permission_granted');
    if (!granted) {
      granted = await invoke<string>('plugin:notification|request_permission') === 'granted';
    }
    if (!granted) {
      ElMessage.warning('通知权限未授予，请到系统设置中开启通知权限。');
      return false;
    }
    if (isAndroid()) {
      await createChannel({ id: CHANNEL_ID, name: '上课提醒', importance: Importance.High, vibration: true });
    }
    return true;
  }

  async function notify(options: Options): Promise<void> {
    await invoke('plugin:notification|notify', {
      options: { ...options, ...(isAndroid() ? { channelId: CHANNEL_ID } : {}) }
    });
  }

  function clearTimer() {
    if (timer !== undefined) window.clearInterval(timer);
    timer = undefined;
    isRunning.value = false;
  }

  // Serialize mutations so a slow permission prompt cannot restore an obsolete plan.
  function queueUpdate(task: (version: number) => Promise<void>) {
    const version = ++revision;
    clearTimer();
    work = work.then(async () => {
      if (version === revision) await task(version);
    }).catch(error => {
      console.error('更新上课提醒失败:', error);
      ElMessage.error(`更新上课提醒失败: ${String(error)}`);
    });
    return work;
  }

  async function cancelCourseReminders() {
    if (!isAndroid()) return;
    const ids = (await pending()).filter(item =>
      item.id >= REMINDER_ID_BASE && item.id < REMINDER_ID_LIMIT
    ).map(item => item.id);
    if (ids.length) await cancel(ids);
  }

  function startReminderService(
    courses: Course[], timeTables: TimeTable[], firstDay: number | undefined,
    weeksCount: number, debugLogging = false
  ): Promise<void> {
    return queueUpdate(async version => {
      await cancelCourseReminders();
      if (version !== revision || !await prepareNotifications() || version !== revision) return;
      if (!firstDay) {
        ElMessage.warning('请先在课表设置中填写第一周第一天，才能安排上课提醒。');
        return;
      }
      const plan = buildReminderPlan(courses, timeTables[0]?.periods ?? [], firstDay, weeksCount);
      if (debugLogging) console.log('[上课提醒] 按实际日期安排:', plan);
      if (isAndroid()) {
        for (const item of plan) {
          if (version !== revision) return;
          if (item.at.getTime() <= Date.now()) continue;
          await notify({ id: item.id, title: '上课提醒', body: item.body,
            schedule: Schedule.at(item.at, false, true) });
        }
      } else {
        const check = async () => {
          while (plan.length && plan[0].at.getTime() <= Date.now()) {
            const item = plan.shift()!;
            if (item.start.getTime() > Date.now()) {
              await notify({ id: item.id, title: '上课提醒', body: item.body });
            }
          }
        };
        timer = window.setInterval(() => {
          void check().catch(error => console.error('发送上课提醒失败:', error));
        }, 1000);
      }
      isRunning.value = true;
    });
  }

  function stopReminderService(): Promise<void> {
    return queueUpdate(async () => { await cancelCourseReminders(); });
  }

  async function testNotification(): Promise<void> {
    try {
      if (!await prepareNotifications()) return;
      await notify({ id: 999_999, title: '上课提醒 - 测试',
        body: '课程: 测试课程\n地点: 沙河校区主教101\n时间: 08:00 - 09:35' });
      ElMessage.success('测试通知已发送！请检查系统通知。');
    } catch (error) {
      console.error('发送通知失败:', error);
      ElMessage.error(`发送通知失败: ${String(error)}`);
    }
  }

  onUnmounted(() => {
    ++revision;
    clearTimer();
    // Keep native alarms when the app/page closes.
  });

  return { isRunning, startReminderService, stopReminderService, testNotification };
}
