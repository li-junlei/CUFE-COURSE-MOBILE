<template>
  <Transition name="popup">
    <div v-if="show" class="popup-overlay" @click.self="$emit('close')">
      <div class="popup-content">
        <div class="popup-title">课程表</div>
        <div class="popup-menu">
          <div class="menu-item" @click="$emit('schedule-manage')">
            <el-icon :size="28"><Collection /></el-icon>
            <span>课表管理</span>
          </div>
          <div class="menu-item" @click="$emit('settings')">
            <el-icon :size="28"><Setting /></el-icon>
            <span>设置</span>
          </div>
          <div class="menu-item" @click="$emit('appearance')">
            <el-icon :size="28"><Picture /></el-icon>
            <span>课表外观</span>
          </div>
          <div class="menu-item" @click="$emit('profile')">
            <el-icon :size="28"><User /></el-icon>
            <span>个人</span>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { Collection, Picture, User, Setting } from '@element-plus/icons-vue';

defineProps<{
  show: boolean;
}>();

defineEmits<{
  (e: 'close'): void;
  (e: 'schedule-manage'): void;
  (e: 'import-schedule'): void;
  (e: 'appearance'): void;
  (e: 'settings'): void;
  (e: 'profile'): void;
}>();
</script>

<style scoped>
.popup-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.2); /* Lighter overlay */
  z-index: 1000;
  backdrop-filter: blur(2px);
}

.popup-content {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  background-color: var(--surface-color-strong);
  backdrop-filter: var(--surface-blur);
  border-radius: 0 0 24px 24px;
  padding: 24px;
  box-shadow: var(--shadow-lg);
  border-bottom: 1px solid var(--border-color);
}

.popup-title {
  text-align: center;
  font-size: 16px;
  font-weight: 700;
  margin-bottom: 24px;
  padding-top: 12px;
  letter-spacing: 0.5px;
  color: var(--text-main);
  opacity: 0.9;
}

.popup-menu {
  display: flex;
  flex-wrap: wrap;
  gap: 12px;
  padding: 0 10px 16px;
  justify-content: center;
}

.menu-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 10px;
  padding: 16px;
  cursor: pointer;
  border-radius: 20px;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  font-size: 13px;
  font-weight: 500;
  color: var(--text-main);
  background-color: var(--surface-color-light);
  border: 1px solid var(--border-color);
  width: 80px; /* Fixed width for alignment */
}

.menu-item:hover {
  background-color: var(--surface-color-strong);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
  border-color: var(--primary-color);
}

.menu-item .el-icon {
  color: var(--text-secondary);
  transition: color 0.2s;
}

.menu-item:hover .el-icon {
  color: var(--primary-color);
}

/* 动画 */
.popup-enter-active,
.popup-leave-active {
  transition: opacity 0.3s ease;
}

.popup-enter-active .popup-content,
.popup-leave-active .popup-content {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.popup-enter-from,
.popup-leave-to {
  opacity: 0;
}

.popup-enter-from .popup-content,
.popup-leave-to .popup-content {
  transform: translateY(-100%);
}

/* 深色模式适配 */
@media (prefers-color-scheme: dark) {
  .menu-item {
    background-color: rgba(255, 255, 255, 0.05);
  }
}
</style>
