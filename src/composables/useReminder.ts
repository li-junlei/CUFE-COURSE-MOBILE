import { ref, onUnmounted } from 'vue';
import type { Course, TimeTable, PeriodTime } from '../types';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
import { ElMessage } from 'element-plus';

/**
 * 提醒服务 Composable
 * 情况A（无紧邻前序课程）：如果当前课程的上一节次没有课，则在当前课程开始前15分钟触发提醒
 * 情况B（有紧邻前序课程）：如果当前课程的上一节次有课（即连续上课），则在上一节课结束前3分钟触发提醒
 */
export function useReminder() {
  const reminderTimer = ref<number | null>(null);
  const isRunning = ref(false);

  // 解析时间字符串为 Date 对象（使用今天作为基准）
  function parseTimeToDate(timeStr: string): Date {
    const [hours, minutes] = timeStr.split(':').map(Number);
    const date = new Date();
    date.setHours(hours, minutes, 0, 0);
    return date;
  }

  // 查找今天的课程
  function getTodayCourses(courses: Course[], currentWeek: number): Course[] {
    const now = new Date();
    const dayOfWeek = now.getDay() === 0 ? 7 : now.getDay(); // 1-7

    return courses.filter(course => {
      // 检查星期几
      if (course.dayOfWeek !== dayOfWeek) return false;

      // 检查当前周是否在课程周次范围内
      if (course.weekType === 1 && currentWeek % 2 !== 1) return false; // 单周
      if (course.weekType === 2 && currentWeek % 2 !== 0) return false; // 双周
      if (course.weekType === 0 && !course.weeks.includes(currentWeek)) return false; // 全周

      return true;
    });
  }

  // 生成课程唯一键（用于防止重复提醒）
  function getCourseKey(course: Course, _dayOffset: number = 0): string {
    const date = new Date();
    const dayOfWeek = date.getDay() === 0 ? 7 : date.getDay();
    return `${dayOfWeek}_${course.periods[0]}_${course.periods[1]}`;
  }

  // 发送通知
  async function sendCourseNotification(course: Course, periodTime: PeriodTime): Promise<boolean> {
    try {
      // 检查通知权限
      let permissionGranted = await isPermissionGranted();
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
      }

      if (!permissionGranted) {
        ElMessage.warning('通知权限未授予，请到系统设置中开启通知权限。');
        return false;
      }

      // 发送通知
      await sendNotification({
        title: '上课提醒',
        body: `课程: ${course.name}\n地点: ${course.location}\n时间: ${periodTime.start} - ${periodTime.end}`
      });

      return true;
    } catch (error) {
      console.error('发送通知失败:', error);
      return false;
    }
  }

  // 检查并发送提醒
  async function checkAndNotify(
    courses: Course[],
    timeTables: TimeTable[],
    currentWeek: number,
    _remindedCourses: Record<string, number> = {},
    onReminded?: (_key: string) => void,
    debugLogging: boolean = false
  ): Promise<void> {
    if (!timeTables.length || !timeTables[0]?.periods?.length) return;

    const now = new Date();
    const currentTime = now.getHours() * 60 + now.getMinutes();
    if (debugLogging) {
      console.log('[提醒调试] 检查触发窗口:', {
        currentWeek,
        currentTime,
        courseCount: courses.length
      });
    }

    // 获取今天的课程
    const todayCourses = getTodayCourses(courses, currentWeek);
    if (todayCourses.length === 0) return;

    // 情况A: 检查即将开始的课程（无前序课程）- 开始前15分钟
    for (const course of todayCourses) {
      const periodIndex = course.periods[0] - 1;
      if (periodIndex < 0 || periodIndex >= timeTables[0].periods.length) continue;

      const periodTime = timeTables[0].periods[periodIndex];
      const startTime = parseTimeToDate(periodTime.start);
      const startMinutes = startTime.getHours() * 60 + startTime.getMinutes();

      // 计算提醒时间（开始前15分钟）
      const reminderMinutes = startMinutes - 15;
      const key = getCourseKey(course);

      // 检查是否已提醒过（5分钟内不重复提醒）
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const lastReminded = _remindedCourses[key] || 0;
      if (now.getTime() - lastReminded < 5 * 60 * 1000) continue;

      // 检查是否在提醒时间窗口内（前后1分钟误差）
      if (Math.abs(currentTime - reminderMinutes) <= 1) {
        // 检查是否有紧邻的前序课程
        const hasPrevious = todayCourses.some(c =>
          c.periods[1] === course.periods[0] - 1
        );

        if (!hasPrevious) {
          // 情况A：无紧邻前序课程
          console.log('[提醒服务] 发送情况A提醒:', course.name);
          const sent = await sendCourseNotification(course, periodTime);
          if (sent) {
            _remindedCourses[key] = now.getTime();
            onReminded?.(key);
            if (debugLogging) {
              console.log('[提醒调试] 情况A已触发:', { key, course: course.name });
            }
          }
        }
      }
    }

    // 情况B: 检查即将结束的课程（有后续紧邻课程）- 结束前3分钟
    for (const course of todayCourses) {
      // 查找是否有后续紧邻课程
      const hasNext = todayCourses.some(c =>
        c.periods[0] === course.periods[1] + 1
      );

      if (!hasNext) continue;

      const periodIndex = course.periods[1] - 1;
      if (periodIndex < 0 || periodIndex >= timeTables[0].periods.length) continue;

      const periodTime = timeTables[0].periods[periodIndex];
      const endTime = parseTimeToDate(periodTime.end);
      const endMinutes = endTime.getHours() * 60 + endTime.getMinutes();

      // 计算提醒时间（结束前3分钟）
      const reminderMinutes = endMinutes - 3;
      const key = getCourseKey(course);

      // 检查是否已提醒过
      // eslint-disable-next-line @typescript-eslint/no-unused-vars
      const lastReminded = _remindedCourses[key] || 0;
      if (now.getTime() - lastReminded < 5 * 60 * 1000) continue;

      // 检查是否在提醒时间窗口内
      if (Math.abs(currentTime - reminderMinutes) <= 1) {
        console.log('[提醒服务] 发送情况B提醒:', course.name);
        const sent = await sendCourseNotification(course, periodTime);
        if (sent) {
          _remindedCourses[key] = now.getTime();
          onReminded?.(key);
          if (debugLogging) {
            console.log('[提醒调试] 情况B已触发:', { key, course: course.name });
          }
        }
      }
    }
  }

  // 启动提醒服务
  function startReminderService(
    courses: Course[],
    timeTables: TimeTable[],
    currentWeek: number,
    _onReminded?: (_key: string) => void,
    remindedCourses: Record<string, number> = {},
    debugLogging: boolean = false
  ): void {
    if (isRunning.value) {
      return;
    }

    // 立即执行一次检查
    void checkAndNotify(courses, timeTables, currentWeek, remindedCourses, _onReminded, debugLogging);

    // 设置定时器，每30秒检查一次
    reminderTimer.value = window.setInterval(() => {
      void checkAndNotify(courses, timeTables, currentWeek, remindedCourses, _onReminded, debugLogging);
    }, 30000);

    isRunning.value = true;
    console.log('[提醒服务] 已启动');
  }

  // 停止提醒服务
  function stopReminderService(): void {
    if (reminderTimer.value !== null) {
      clearInterval(reminderTimer.value);
      reminderTimer.value = null;
    }
    isRunning.value = false;
    console.log('[提醒服务] 已停止');
  }

  // 测试通知
  async function testNotification(): Promise<void> {
    try {
      const permissionGranted = await isPermissionGranted();
      if (!permissionGranted) {
        const permission = await requestPermission();
        if (permission !== 'granted') {
          ElMessage.warning('通知权限未授予，请到系统设置中开启通知权限。');
          return;
        }
      }

      await sendNotification({
        title: '上课提醒 - 测试',
        body: '课程: 测试课程\n地点: 沙河校区主教101\n时间: 08:00 - 09:35'
      });
      ElMessage.success('测试通知已发送！请检查系统通知。');
    } catch (error) {
      console.error('发送通知失败:', error);
      ElMessage.error('发送通知失败: ' + (error as Error).message);
    }
  }

  // 组件卸载时自动停止服务
  onUnmounted(() => {
    stopReminderService();
  });

  return {
    isRunning,
    startReminderService,
    stopReminderService,
    testNotification,
    checkAndNotify
  };
}
