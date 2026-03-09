/**
 * 课程颜色数组（21 种颜色）
 */
export const COURSE_COLORS = [
  '#99CCFF',
  '#FFCC99',
  '#FFCCCC',
  '#CC6699',
  '#99CCCC',
  '#FF6666',
  '#CCCC66',
  '#66CC99',
  '#FF9966',
  '#66CCCC',
  '#6699CC',
  '#99CC99',
  '#669966',
  '#99CC99',
  '#99CCCC',
  '#66CCFF',
  '#CCCCFF',
  '#99CC66',
  '#CCCC99',
  '#FF9999',
  '#76B1E4',
];

/**
 * 随机打乱数组
 * @param array - 要打乱的数组
 * @returns 打乱后的数组
 */
function shuffle<T>(array: T[]): T[] {
  const result = [...array];
  for (let i = result.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [result[i], result[j]] = [result[j], result[i]];
  }
  return result;
}

/**
 * 获取随机排序的课程颜色数组
 * @returns 随机排序的颜色数组
 */
export function getShuffledColors(): string[] {
  return shuffle(COURSE_COLORS);
}

/**
 * 根据索引获取课程颜色
 * @param index - 课程索引
 * @param colors - 颜色数组
 * @returns 课程颜色
 */
export function getCourseColor(index: number, colors: string[]): string {
  return colors[index % colors.length];
}
