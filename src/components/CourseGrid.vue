<template>
  <div 
    class="schedule-container"
    ref="containerRef"
    @mousedown="startSwipe"
    @mousemove="handleSwipe"
    @mouseup="endSwipe"
    @mouseleave="endSwipe"
    @touchstart="startSwipeTouch"
    @touchmove="handleSwipeTouch"
    @touchend="endSwipe"
  >
    <!-- 左侧时间栏 -->
    <div class="time-bar">
      <div v-for="(time, index) in courseTime" :key="index" class="time-slot">
        <span class="period-number">{{ index + 1 }}</span>
        <span class="period-time">{{ time }}</span>
      </div>
    </div>

    <!-- 右侧课表区域 -->
    <div
      class="schedule-area"
      :class="{ 'show-grid': showGridLines }"
      :style="{
        backgroundImage: bgImage ? `url(${bgImage})` : 'none',
        transform: `translateX(${swipeOffset}px)`,
        transition: isSwipingEnd ? 'transform 0.3s ease' : 'none'
      }"
    >
      <!-- 网格辅助线 -->
      <div v-if="showGridLines" class="grid-lines">
        <!-- 横线（节次分隔线，跳过第一条上框线） -->
        <div
          v-for="i in maxPeriods - 1"
          :key="'h-' + (i + 1)"
          class="grid-line horizontal"
          :style="{ top: ((i) * 100 / maxPeriods) + '%' }"
        />
        <!-- 竖线（星期分隔线，跳过第一条左框线） -->
        <div
          v-for="i in 6"
          :key="'v-' + (i + 1)"
          class="grid-line vertical"
          :style="{ left: ((i) * 100 / 7) + '%' }"
        />
      </div>

      <!-- 课程卡片 -->
      <CourseCard
        v-for="(course, index) in courses"
        :key="index"
        :course="course"
        :week="week"
        :max-periods="maxPeriods"
        :color="colors[index % colors.length]"
        :card-opacity="cardOpacity"
        :show-teacher="showTeacher"
        :show-location="showLocation"
        :simplified-location="simplifiedLocation"
        @click="onCardClick"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import CourseCard from './CourseCard.vue';
import type { Course, PeriodTime } from '../types';

interface Props {
  courses: Course[];
  week: number;
  endWeek: number;
  colors: string[];
  bgImage?: string;
  maxPeriods?: number;
  periodTimes?: PeriodTime[]; // Custom time labels with start and end
  showGridLines?: boolean;
  cardOpacity?: number;
  showTeacher?: boolean;
  showLocation?: boolean;
  simplifiedLocation?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  bgImage: '',
  maxPeriods: 13,
  periodTimes: () => [],
  showGridLines: false,
  cardOpacity: 95,
  showTeacher: true,
  showLocation: true,
  simplifiedLocation: false,
});

const emit = defineEmits<{
  (e: 'update:week', value: number): void;
  (e: 'course-click', course: Course): void;
}>();

// Dynamic Time Slots
const courseTime = computed(() => {
  // If custom times provided and length matches (or close enough), use them.
  // We prioritize maxPeriods length.
  const slots = [];
  for (let i = 0; i < props.maxPeriods; i++) {
      if (props.periodTimes && props.periodTimes[i]) {
          const time = props.periodTimes[i];
          // Format as "start-end"
          slots.push(`${time.start}-${time.end}`);
      } else {
          // Default generation (continuous periods, no breaks)
          const p = i + 1;
          // Start from 8:00, each period is 45 minutes with 10 min break
          const startHour = 8 + Math.floor((p - 1) * 55 / 60);
          const startMin = ((p - 1) * 55) % 60;

          const endMin = (startMin + 45) % 60;
          const endHour = startHour + Math.floor((startMin + 45) / 60);

          slots.push(`${startHour}:${startMin.toString().padStart(2, '0')}-${endHour}:${endMin.toString().padStart(2, '0')}`);
      }
  }
  return slots;
});

// CSS Variable for Row Height
const rowHeightStr = computed(() => `${100 / props.maxPeriods}%`);

// 滑动状态
const containerRef = ref<HTMLElement | null>(null);
const isSwiping = ref(false);
const isSwipingEnd = ref(false);
const swipeStartX = ref(0);
const swipeOffset = ref(0);
const swipeThreshold = 80; // 滑动阈值

function startSwipe(e: MouseEvent) {
  isSwiping.value = true;
  isSwipingEnd.value = false;
  swipeStartX.value = e.clientX;
  swipeOffset.value = 0;
}

function startSwipeTouch(e: TouchEvent) {
  isSwiping.value = true;
  isSwipingEnd.value = false;
  swipeStartX.value = e.touches[0].clientX;
  swipeOffset.value = 0;
}

function handleSwipe(e: MouseEvent) {
  if (!isSwiping.value) return;
  const diff = e.clientX - swipeStartX.value;
  // 限制滑动范围
  swipeOffset.value = Math.max(-150, Math.min(150, diff));
}

function handleSwipeTouch(e: TouchEvent) {
  if (!isSwiping.value) return;
  const diff = e.touches[0].clientX - swipeStartX.value;
  swipeOffset.value = Math.max(-150, Math.min(150, diff));
}

function onCardClick(course: Course) {
  console.log('CourseGrid receiving click for:', course.name);
  emit('course-click', course);
}

function endSwipe() {
  if (!isSwiping.value) return;

  isSwiping.value = false;
  isSwipingEnd.value = true;

  // 判断滑动方向和距离
  if (swipeOffset.value > swipeThreshold) {
    // 向右滑动 - 上一周
    const newWeek = props.week > 1 ? props.week - 1 : props.endWeek;
    emit('update:week', newWeek);
  } else if (swipeOffset.value < -swipeThreshold) {
    // 向左滑动 - 下一周
    const newWeek = props.week < props.endWeek ? props.week + 1 : 1;
    emit('update:week', newWeek);
  }

  // 复位
  swipeOffset.value = 0;

  setTimeout(() => {
    isSwipingEnd.value = false;
  }, 300);
}
</script>

<style scoped>
.schedule-container {
  display: flex;
  width: 100%;
  height: 100%;
  user-select: none;
  cursor: grab;
  position: relative;
}

.schedule-container:active {
  cursor: grabbing;
}

/* 左侧时间栏 - 悬浮透明风格 */
.time-bar {
  width: 48px; /* Fixed width */
  min-width: 48px;
  height: 100%;
  display: flex;
  flex-direction: column;
  background-color: transparent; /* Remove solid background */
  z-index: 10;
  padding-top: 4px; /* Slight offset */
}

.time-slot {
  height: v-bind(rowHeightStr);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  position: relative;
}

.period-number {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  opacity: 0.9;
  line-height: 1.2;
}

.period-time {
  color: var(--text-tertiary);
  font-size: 10px;
  transform: scale(0.9);
  white-space: nowrap;
  font-weight: 500;
}

/* 右侧课表区域 */
.schedule-area {
  position: relative;
  flex: 1;
  width: calc(100% - 48px);
  height: 100%;
  background-size: cover;
  background-position: center;
  background-repeat: no-repeat;
}

/* 显示网格时的样式 */
.schedule-area.show-grid {
  /* 添加水平辅助线 */
  background-image: linear-gradient(to bottom, transparent calc(100% - 1px), var(--border-color) 100%);
  background-size: 100% v-bind(rowHeightStr);
}

/* 网格辅助线容器 */
.grid-lines {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
  z-index: 1;
}

/* 网格线 */
.grid-line {
  position: absolute;
  opacity: 0.8;
}

.grid-line.horizontal {
  width: 100%;
  height: 0;
  left: 0;
  border-top: 2px dashed var(--border-color);
}

.grid-line.vertical {
  width: 0;
  height: 100%;
  top: 0;
  border-left: 2px dashed var(--border-color);
}

</style>
