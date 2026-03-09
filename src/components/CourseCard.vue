<template>
  <div
    v-if="isVisible"
    class="course-card"
    :style="{
        top: topPosition,
        left: leftPosition,
        height: cardHeight,
    }"
    @click.stop="handleClick"
  >
    <div
      class="card-inner"
      :class="{ 'exam-card': course.courseType === 'exam' }"
      :style="{
        backgroundColor: color,
        opacity: cardOpacity / 100
      }"
    >
      <div class="card-content-wrapper">
        <div class="course-name">{{ course.name }}</div>

        <!-- 考试时间（仅考试且有 examInfo 时显示） -->
        <div v-if="course.courseType === 'exam' && course.examInfo" class="exam-datetime">
          {{ formatExamDate(course.examInfo.date) }}
          <br>
          {{ course.examInfo.startTime }}-{{ course.examInfo.endTime }}
        </div>

        <!-- 教师和地点（常规课程显示） -->
        <div class="course-location" v-if="showLocation && course.location">@{{ formattedLocation }}</div>
        <div class="course-teacher" v-if="showTeacher && course.teacher">{{ course.teacher }}</div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { Course } from '../types';

interface Props {
  course: Course;
  week: number;
  color: string;
  maxPeriods?: number;
  cardOpacity?: number;
  showTeacher?: boolean;
  showLocation?: boolean;
  simplifiedLocation?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  maxPeriods: 13,
  cardOpacity: 95,
  showTeacher: true,
  showLocation: true,
  simplifiedLocation: false,
});

/**
 * 格式化地点显示
 */
const formattedLocation = computed(() => {
  const loc = props.course.location || '';
  if (!props.simplifiedLocation) return loc;

  // 移除常见的校区前缀，支持更多格式
  // 匹配: "沙河校区", "学院南路校区", "沙河", "南路", "学院南路" 等及其后的空格
  return loc.replace(/(沙河校区|学院南路校区|沙河|南路|学院南路)\s*/g, '');
});

/**
 * 检查课程是否在指定周次显示
 */
const isVisible = computed(() => {
  const { course, week } = props;

  // 检查是否在周次范围内 (weeks现在是完整的周次列表)
  const inRange = course.weeks.includes(week);

  // 检查单双周 (作为备用逻辑)
  const weekMatch = course.weekType === 0 || (course.weekType == 1 && week % 2 != 0) || (course.weekType == 2 && week % 2 == 0);

  return inRange && weekMatch;
});

/**
 * 此时不再需要 isShortCard 判断对齐方式，统统顶部对齐
 */

/**
 * 计算顶部位置
 */
const topPosition = computed(() => {
  const { periods } = props.course;
  const firstPeriod = periods[0];

  if (isNaN(firstPeriod)) {
    return '0%';
  }

  // 简单线性布局
  const unitHeight = 100 / props.maxPeriods;
  return `${(firstPeriod - 1) * unitHeight}%`;
});

/**
 * 计算左侧位置
 */
const leftPosition = computed(() => {
  const dayOfWeek = props.course.dayOfWeek;
  return `${(dayOfWeek - 1) * (100 / 7)}%`;
});

/**
 * 计算高度
 */
const cardHeight = computed(() => {
  const { periods } = props.course;
  const firstPeriod = periods[0];
  const lastPeriod = periods[periods.length - 1];

  if (isNaN(firstPeriod) || !lastPeriod) {
    const unitHeight = 100 / props.maxPeriods;
    return `${unitHeight}%`;
  }

  const unitHeight = 100 / props.maxPeriods;
  return `${(lastPeriod - firstPeriod + 1) * unitHeight}%`;
});

/**
 * 显示课程详情
 */
const emit = defineEmits<{
  (e: 'click', course: Course): void;
}>();

function handleClick() {
  console.log('CourseCard clicked:', props.course.name);
  emit('click', props.course);
}

/**
 * 格式化考试日期为简短格式
 */
function formatExamDate(dateStr: string): string {
  const date = new Date(dateStr);
  const month = date.getMonth() + 1;
  const day = date.getDate();
  return `${month}月${day}日`;
}
</script>

<style scoped>
.course-card {
  position: absolute;
  width: calc(100% / 7); /* Consistent with layout */
  padding: 1.5px; /* Tighter padding between cards */
  box-sizing: border-box;
  overflow: hidden;
  cursor: pointer;
  z-index: 10;
}

.card-inner {
  width: 100%;
  height: 100%;
  border-radius: 6px; /* Slightly smaller radius for tighter look */
  display: flex;
  flex-direction: column;
  padding: 4px 5px; /* Compact padding */
  box-sizing: border-box;
  transition: all 0.2s cubic-bezier(0.25, 0.8, 0.25, 1);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  opacity: 0.95;
  border: 1px solid rgba(255,255,255,0.15);
  
  /* Align top-left */
  align-items: flex-start;
  justify-content: flex-start;
  text-align: left;
}

.card-inner:hover {
  transform: translateY(-1px) scale(1.01);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  z-index: 20;
  opacity: 1;
}

.card-content-wrapper {
  width: 100%;
  height: 100%;
  overflow: hidden;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.course-name {
  font-size: 11px;
  font-weight: 700; /* Bolder */
  color: white;
  line-height: 1.25;
  /* Allow multi-line */
  display: -webkit-box;
  -webkit-line-clamp: 4; /* Max lines for name */
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-shadow: 0 1px 1px rgba(0,0,0,0.1);
}

.course-location {
  font-size: 10px;
  color: rgba(255, 255, 255, 0.9);
  line-height: 1.2;
  font-weight: 500;
  /* Allow multi-line */
  display: -webkit-box;
  -webkit-line-clamp: 2; /* Limit location to 2 lines to avoid taking too much space */
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin-top: 1px;
}

.course-teacher {
  font-size: 9px;
  color: rgba(255, 255, 255, 0.8);
  line-height: 1.2;
  font-weight: 400;
  /* Allow multi-line */
  display: -webkit-box;
  -webkit-line-clamp: 1;
  -webkit-box-orient: vertical;
  overflow: hidden;
  margin-top: 2px;
}

/* 考试卡片样式 */
.exam-card {
  background: linear-gradient(135deg, #ff6b6b 0%, #ff8787 100%) !important;
  border: 2px solid #ff5252 !important;
  box-shadow: 0 2px 8px rgba(255, 107, 107, 0.3) !important;
}

.exam-card:hover {
  box-shadow: 0 4px 16px rgba(255, 107, 107, 0.5) !important;
}

.exam-card .course-name {
  font-size: 12px;
  font-weight: 800;
}

.exam-badge {
  position: absolute;
  top: 3px;
  right: 3px;
  background: rgba(255, 255, 255, 0.95);
  color: #ff5252;
  padding: 2px 6px;
  font-size: 9px;
  border-radius: 4px;
  font-weight: bold;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
  z-index: 10;
}

.exam-datetime {
  font-size: 10px;
  margin-top: 4px;
  line-height: 1.3;
  opacity: 0.95;
  font-weight: 600;
  color: white;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.15);
}
</style>
