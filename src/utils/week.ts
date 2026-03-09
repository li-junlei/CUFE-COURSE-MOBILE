// 周次计算工具函数

/**
 * 计算当前周次
 * @param firstDay 学期开始时间戳（秒）
 * @param weeksCount 学期周数
 * @returns 当前周次
 */
export function calculateCurrentWeek(firstDay: number | undefined, weeksCount: number | undefined): number {
  if (!firstDay) return 1;

  const week = Math.floor((Date.now() - firstDay * 1000) / (7 * 24 * 60 * 60 * 1000)) + 1;
  const maxWeeks = weeksCount || 20;
  return Math.max(Math.min(week, maxWeeks), 1);
}

/**
 * 获取学期状态
 * @param firstDay 学期开始时间戳（秒）
 * @param weeksCount 学期周数
 * @returns 学期状态
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
  return null;
}
