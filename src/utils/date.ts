/**
 * 格式化日期为 M/D 格式
 * @param date - 日期对象
 * @returns 格式化后的日期字符串
 */
export function formatDate(date: Date): string {
  const month = date.getMonth() + 1;
  const day = date.getDate();
  return `${month}/${day}`;
}

/**
 * 格式化日期为 YYYY-MM-DD 格式（本地时间）
 * @param date - 日期对象
 * @returns 格式化后的日期字符串
 */
export function formatDateString(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/**
 * 计算指定周次和星期的日期
 * @param firstDay - 学期第一天的时间戳
 * @param targetWeek - 目标周次
 * @param targetDay - 目标星期 (1-7)
 * @returns 格式化后的日期字符串
 */
export function calculateDate(firstDay: number, targetWeek: number, targetDay: number): string {
  // firstDay 是秒级时间戳，需要转换为毫秒级
  const firstDate = new Date(firstDay * 1000);
  const targetDate = new Date(firstDate);

  // 计算目标日期
  targetDate.setDate(firstDate.getDate() + (targetWeek - 1) * 7 + (targetDay - 1));

  return formatDate(targetDate);
}

/**
 * 获取当前周次
 * @param firstDay - 学期第一天的时间戳
 * @returns 当前周次
 */
export function getCurrentWeek(firstDay: number): number {
  const now = Date.now();
  // firstDay 是秒级时间戳，需要转换为毫秒级
  const week = Math.floor((now - firstDay * 1000) / (7 * 24 * 60 * 60 * 1000)) + 1;
  return Math.max(week, 1);
}
