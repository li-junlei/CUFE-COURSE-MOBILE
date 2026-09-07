<template>
  <div class="bg-cropper-overlay">
    <!-- 顶部操作栏 -->
    <div class="cropper-topbar">
      <span class="topbar-btn" @click="emit('cancel')">取消</span>
      <span class="topbar-title">裁剪背景</span>
      <span class="topbar-btn confirm" :class="{ saving }" @click="handleConfirm">
        {{ saving ? '保存中…' : '确定' }}
      </span>
    </div>

    <!-- 裁剪舞台：按屏幕比例放置固定框 -->
    <div class="cropper-stage" ref="stageRef">
      <div class="cropper-frame" ref="frameRef">
        <img ref="imgRef" :src="src" alt="" />
        <!-- 真实课表叠加预览 -->
        <div v-if="showPreview" class="preview-overlay">
          <CourseGrid
            :courses="previewCourses"
            :week="previewWeek"
            :end-week="previewEndWeek"
            :colors="previewColors"
            bg-image=""
            :max-periods="previewMaxPeriods"
            :period-times="previewPeriodTimes"
            :show-grid-lines="previewShowGridLines"
            :card-opacity="previewCardOpacity"
            :show-teacher="previewShowTeacher"
            :show-location="previewShowLocation"
            :simplified-location="previewSimplifiedLocation"
          />
        </div>
      </div>
    </div>

    <!-- 底部调节面板 -->
    <div class="cropper-controls">
      <div class="ctrl-row">
        <span>背景不透明度</span>
        <span>{{ opacity }}%</span>
      </div>
      <el-slider
        v-model.number="opacity"
        :min="0"
        :max="100"
        :step="5"
        :show-tooltip="false"
      />
      <div class="ctrl-row">
        <span>背景模糊</span>
        <span>{{ blur }}px</span>
      </div>
      <el-slider
        v-model.number="blur"
        :min="0"
        :max="30"
        :step="1"
        :show-tooltip="false"
      />
      <div class="ctrl-row switch-row">
        <span>课表预览</span>
        <el-switch v-model="showPreview" style="--el-switch-on-color: var(--primary-color);" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import Cropper from 'cropperjs';
import 'cropperjs/dist/cropper.css';
import { ElMessage } from 'element-plus';
import CourseGrid from './CourseGrid.vue';
import type { Course, PeriodTime } from '../types';

const props = defineProps<{
  /** 待裁剪图片（objectURL，由父组件负责 revoke） */
  src: string;
  /** 裁剪框宽高比（打开瞬间的视口宽/高） */
  ratio: number;
  initialOpacity: number;
  initialBlur: number;
  /** 课表预览数据（与主界面一致） */
  previewCourses: Course[];
  previewWeek: number;
  previewEndWeek: number;
  previewColors: string[];
  previewMaxPeriods: number;
  previewPeriodTimes: PeriodTime[];
  previewShowGridLines: boolean;
  previewCardOpacity: number;
  previewShowTeacher: boolean;
  previewShowLocation: boolean;
  previewSimplifiedLocation: boolean;
}>();

const emit = defineEmits<{
  (e: 'cancel'): void;
  (e: 'confirm', payload: { blobBytes: Uint8Array; opacity: number; blur: number }): void;
}>();

const stageRef = ref<HTMLElement | null>(null);
const frameRef = ref<HTMLElement | null>(null);
const imgRef = ref<HTMLImageElement | null>(null);

const opacity = ref(props.initialOpacity);
const blur = ref(props.initialBlur);
const showPreview = ref(true);
const saving = ref(false);

let cropper: Cropper | null = null;

/** 按屏幕比例在舞台内取最大矩形（JS 计算，兼容不支持 aspect-ratio 的旧 WebView） */
function fitFrame() {
  const stage = stageRef.value;
  const frame = frameRef.value;
  if (!stage || !frame) return;
  const availW = stage.clientWidth;
  const availH = stage.clientHeight;
  let w = availW;
  let h = w / props.ratio;
  if (h > availH) {
    h = availH;
    w = h * props.ratio;
  }
  frame.style.width = `${Math.floor(w)}px`;
  frame.style.height = `${Math.floor(h)}px`;
}

onMounted(() => {
  fitFrame();
  cropper = new Cropper(imgRef.value!, {
    aspectRatio: props.ratio,
    viewMode: 1,
    dragMode: 'move',
    autoCropArea: 1,
    cropBoxMovable: false,
    cropBoxResizable: false,
    toggleDragModeOnDblclick: false,
    zoomOnWheel: false,
    background: false,
  });
  window.addEventListener('resize', fitFrame);
});

onUnmounted(() => {
  window.removeEventListener('resize', fitFrame);
  cropper?.destroy();
  cropper = null;
});

async function handleConfirm() {
  if (saving.value || !cropper) return;
  saving.value = true;
  try {
    // 输出分辨率：视口物理像素，最长边上限 2160
    let outW = Math.min(Math.round(window.innerWidth * window.devicePixelRatio), 2160);
    let outH = Math.round(outW / props.ratio);
    if (outH > 2160) {
      outH = 2160;
      outW = Math.round(2160 * props.ratio);
    }

    const canvas = cropper.getCroppedCanvas({
      maxWidth: outW,
      maxHeight: outH,
      fillColor: '#ffffff',
      imageSmoothingQuality: 'high',
    });
    if (!canvas) {
      ElMessage.error('图片处理失败，请重试');
      saving.value = false;
      return;
    }
    const blob = await new Promise<Blob | null>(resolve =>
      canvas.toBlob(resolve, 'image/jpeg', 0.9)
    );
    if (!blob) {
      ElMessage.error('图片导出失败，请重试');
      saving.value = false;
      return;
    }
    emit('confirm', {
      blobBytes: new Uint8Array(await blob.arrayBuffer()),
      opacity: opacity.value,
      blur: blur.value,
    });
  } catch {
    saving.value = false;
  }
}
</script>

<style scoped>
.bg-cropper-overlay {
  position: fixed;
  inset: 0;
  z-index: 5000;
  background: var(--bg-color);
  display: flex;
  flex-direction: column;
}

.cropper-topbar {
  flex-shrink: 0;
  height: calc(48px + env(safe-area-inset-top));
  padding: env(safe-area-inset-top) 20px 0;
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.topbar-title {
  font-size: 16px;
  font-weight: 600;
  color: var(--text-main);
}

.topbar-btn {
  font-size: 15px;
  color: var(--text-tertiary);
  padding: 6px 4px;
}

.topbar-btn.confirm {
  color: #ffffff;
  background: var(--primary-gradient);
  border-radius: 14px;
  padding: 6px 16px;
}

.topbar-btn.confirm.saving {
  opacity: 0.6;
  pointer-events: none;
}

.cropper-stage {
  flex: 1;
  min-height: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  padding: 8px;
}

.cropper-frame {
  position: relative;
  overflow: hidden;
}

/* cropperjs 接管渲染后隐藏原图 */
.cropper-frame img {
  display: block;
  max-width: 100%;
  opacity: 0;
}

/* 课表叠加预览层：不拦截手势 */
.preview-overlay {
  position: absolute;
  inset: 0;
  z-index: 5;
  pointer-events: none;
}

.cropper-controls {
  flex-shrink: 0;
  padding: 12px 20px calc(12px + env(safe-area-inset-bottom));
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.ctrl-row {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  color: var(--text-main);
}

.ctrl-row.switch-row {
  align-items: center;
  margin-top: 8px;
}
</style>
