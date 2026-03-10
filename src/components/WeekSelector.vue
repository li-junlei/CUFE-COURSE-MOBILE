<template>
  <Transition name="week-selector">
    <div v-show="show" class="week-selector">
    <div 
      class="week-list" 
      ref="weekListRef"
      @mousedown="startDrag"
      @mouseleave="stopDrag"
      @mouseup="stopDrag"
      @mousemove="onDrag"
    >
        <div
          v-for="w in maxWeeks"
          :key="w"
          :id="`week-${w}`"
          class="week-item"
          :class="{ active: w === week }"
          @click="selectWeek(w)"
        >
          <div class="week-title">第{{ w }}周</div>
          <div class="week-dots">
            <div
              v-for="(_, index) in 35"
              :key="index"
              class="dot"
              :style="{ backgroundColor: getDotColor(w, index) }"
            ></div>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import type { Course } from '../types';

interface Props {
  show: boolean;
  week: number;
  courses: Course[];
  maxWeeks?: number; // 最大周数，默认20
}

const props = defineProps<Props>();

const emit = defineEmits<{
  (e: 'update:week', value: number): void;
}>();

// 最大周数，默认20
const maxWeeks = computed(() => props.maxWeeks || 20);

const weekListRef = ref<HTMLElement | null>(null);

/**
 * 计算每个周次的点阵状态
 */
const weekDotStatus = computed(() => {
  const dots: (DotStatus | null)[] = new Array(35).fill(null);

  for (const course of props.courses) {
    if (isNaN(course.periods[0]) || course.periods[0] === 5) {
      continue;
    }

    // 计算点阵索引
    const periodIndex = Math.floor((course.periods[0] - 1) / 2); // 0-4
    const dayIndex = course.dayOfWeek - 1; // 0-6
    const dotIndex = periodIndex * 7 + dayIndex; // 0-34

    if (dotIndex >= 0 && dotIndex < 35) {
      if (dots[dotIndex] === null || course.periods.length > dots[dotIndex]!.periodCount) {
        dots[dotIndex] = {
          weeks: course.weeks,
          periodCount: course.periods.length,
          weekType: course.weekType,
        };
      }
    }
  }

  return dots;
});

interface DotStatus {
  weeks: number[];
  periodCount: number;
  weekType: number;
}

/**
 * 获取点的颜色
 */
function getDotColor(week: number, dotIndex: number): string {
  const dotStatus = weekDotStatus.value[dotIndex];
  if (!dotStatus) {
    return '#d4d4d4'; // 灰色 - 无课
  }

  const { weeks, periodCount } = dotStatus;

  // 检查是否在周次范围内 (weeks现在是完整的周次列表)
  const inRange = weeks.includes(week);

  // 检查单双周 (虽然weeks列表已经是准确的，但保留weekType逻辑作为显示区分或其他用途)
  // const weekMatch = weekType === 0 || weekType % 2 === (week % 2);

  if (!inRange) {
    return '#d4d4d4'; // 灰色 - 无课
  }

  // 绿色 - 有课 2 节，蓝色 - 有课 1 节
  return periodCount === 2 ? 'rgb(56,201,153)' : '#76B1E4';
}

/**
 * 选择周次
 */
function selectWeek(w: number) {
  if (isDragging.value) return;
  emit('update:week', w);
}

/**
 * 滚动到当前周次
 */
watch(
  () => props.show,
  async (newVal) => {
    if (newVal) {
      await nextTick();
      const el = document.getElementById(`week-${props.week}`);
      if (el && weekListRef.value) {
        el.scrollIntoView({ behavior: 'smooth', inline: 'center' });
      }
    }
  }
);

/* --- Drag Handling --- */
const isDown = ref(false);
const startX = ref(0);
const scrollLeftStart = ref(0);
const isDragging = ref(false);

function startDrag(e: MouseEvent) {
  isDown.value = true;
  startX.value = e.pageX;
  if (weekListRef.value) {
    scrollLeftStart.value = weekListRef.value.scrollLeft;
  }
  // isDragging remains false until we move
  isDragging.value = false;
}

function stopDrag() {
  isDown.value = false;
  // Delay resetting isDragging to block the subsequent click
  setTimeout(() => {
    isDragging.value = false;
  }, 0);
}

function onDrag(e: MouseEvent) {
  if (!isDown.value || !weekListRef.value) return;
  
  e.preventDefault();
  const x = e.pageX;
  const walk = (x - startX.value) * 1.5; // Scroll-fast multiplier
  
  if (Math.abs(x - startX.value) > 5) {
    isDragging.value = true;
  }
  
  weekListRef.value.scrollLeft = scrollLeftStart.value - walk;
}
</script>

<style scoped>
.week-selector {
  width: 100%;
  background-color: transparent;
  overflow: visible;
  margin-bottom: 4px;
}

.week-list {
  display: flex;
  overflow-x: auto;
  gap: 8px;
  padding: 8px 16px;
  scrollbar-width: none; /* Hide scrollbar for cleaner look */
  cursor: grab;
}

.week-list:active {
  cursor: grabbing;
}

.week-list::-webkit-scrollbar {
  display: none;
}

.week-item {
  flex-shrink: 0;
  width: 56px; /* Narrower, capsule style */
  height: auto;
  padding: 8px 4px;
  border-radius: 20px; /* Capsule shape */
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  background-color: var(--surface-color-light);
  border: 1px solid var(--border-color);
  backdrop-filter: var(--surface-blur);
}

.week-item:hover {
  background-color: var(--surface-color-strong);
  transform: translateY(-2px);
}

.week-item.active {
  background: var(--primary-gradient);
  color: white;
  border-color: transparent;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.4);
  transform: scale(1.05);
}

.week-title {
  font-size: 11px;
  margin-bottom: 6px;
  white-space: nowrap;
  font-weight: 600;
  color: var(--text-secondary);
}

.week-item.active .week-title {
  color: rgba(255, 255, 255, 0.95);
}

.week-dots {
  display: grid;
  grid-template-columns: repeat(7, 1fr);
  grid-template-rows: repeat(5, 1fr);
  gap: 2px;
  width: 42px; /* Smaller dots area */
  height: 30px;
}

.dot {
  width: 4px;
  height: 4px; /* Simpler dots */
  border-radius: 50%;
  transition: background-color 0.2s;
}

/* 动画 */
.week-selector-enter-active,
.week-selector-leave-active {
  transition: max-height 0.3s ease, opacity 0.3s ease;
}

.week-selector-enter-from,
.week-selector-leave-to {
  max-height: 0;
  opacity: 0;
}

.week-selector-enter-to,
.week-selector-leave-from {
  max-height: 220px;
  opacity: 1;
}
</style>
