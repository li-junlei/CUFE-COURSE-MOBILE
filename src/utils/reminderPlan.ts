import type { Course, PeriodTime } from '../types';

export interface CourseReminder {
  id: number;
  course: Course;
  at: Date;
  start: Date;
  body: string;
}

// Reserve a range so rebuilding course reminders never cancels unrelated notifications.
export const REMINDER_ID_BASE = 1_000_000;
export const REMINDER_ID_LIMIT = 2_000_000;

export function buildReminderPlan(
  courses: Course[], periods: PeriodTime[], firstDay: number | undefined,
  weeksCount: number, now = new Date()
): CourseReminder[] {
  if (!firstDay || !Number.isFinite(firstDay) || !periods.length) return [];
  const first = new Date(firstDay * 1000);
  first.setHours(0, 0, 0, 0);
  const plan: CourseReminder[] = [];
  const atTime = (date: Date, time: string): Date => {
    const result = new Date(date);
    const [hours, minutes] = time.split(':').map(Number);
    if (!Number.isInteger(hours) || hours < 0 || hours > 23 ||
        !Number.isInteger(minutes) || minutes < 0 || minutes > 59) return new Date(NaN);
    result.setHours(hours, minutes, 0, 0);
    return result;
  };

  for (const course of courses) {
    // Exams have their own dates/times; exclude them from period adjacency.
    if (course.courseType === 'exam') continue;
    const startPeriod = periods[course.periods[0] - 1];
    const endPeriod = periods[course.periods[1] - 1];
    if (!startPeriod || !endPeriod || course.dayOfWeek < 1 || course.dayOfWeek > 7) continue;
    for (const week of new Set(course.weeks)) {
      if (!Number.isInteger(week) || week < 1 || week > weeksCount) continue;
      const date = new Date(first);
      date.setDate(first.getDate() + (week - 1) * 7 + course.dayOfWeek - 1);
      const start = atTime(date, startPeriod.start);
      const hasPrevious = courses.some(c => c.courseType !== 'exam' &&
        c.dayOfWeek === course.dayOfWeek && c.weeks.includes(week) &&
        c.periods[1] === course.periods[0] - 1);
      const previousEnd = hasPrevious ? periods[course.periods[0] - 2]?.end : undefined;
      const at = previousEnd ? atTime(date, previousEnd) : new Date(start);
      at.setMinutes(at.getMinutes() - (previousEnd ? 3 : 15));
      if (!Number.isFinite(start.getTime()) || !Number.isFinite(at.getTime()) || at <= now) continue;
      plan.push({
        id: 0, course, at, start,
        body: `课程: ${course.name}\n地点: ${course.location}\n时间: ${startPeriod.start} - ${endPeriod.end}`
      });
    }
  }
  plan.sort((a, b) => a.at.getTime() - b.at.getTime());
  return plan.slice(0, REMINDER_ID_LIMIT - REMINDER_ID_BASE).map((item, index) => ({
    ...item, id: REMINDER_ID_BASE + index
  }));
}
