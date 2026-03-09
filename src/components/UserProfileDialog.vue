<template>
  <el-dialog
    v-model="dialogVisible"
    title=""
    width="90%"
    align-center
    :show-close="false"
    class="custom-dialog user-profile-dialog"
    append-to-body
    style="max-width: 480px;"
    @close="handleClose"
  >
    <div class="dialog-header">
      <div class="dialog-title">个人中心</div>
      <div class="dialog-close-btn" @click="handleClose">
        <el-icon :size="20"><Close /></el-icon>
      </div>
    </div>

    <div class="dialog-content">
      <!-- 未登录：显示登录表单 -->
      <template v-if="!userInfo">
        <div class="login-section">
          <div class="login-icon">
            <el-icon :size="64" color="var(--primary-color)"><User /></el-icon>
          </div>
          <div class="login-title">登录教务系统</div>
          <div class="login-subtitle">登录后可查看个人信息并导入课表</div>
          
          <div class="login-form">
            <div class="form-item">
              <el-input
                v-model="username"
                placeholder="请输入学号"
                class="modern-input"
                size="large"
                :disabled="loading"
              >
                <template #prefix>
                  <el-icon><User /></el-icon>
                </template>
              </el-input>
            </div>
            <div class="form-item">
              <el-input
                v-model="password"
                type="password"
                placeholder="请输入密码"
                show-password
                class="modern-input"
                size="large"
                :disabled="loading"
                @keyup.enter="handleLogin"
              >
                <template #prefix>
                  <el-icon><Key /></el-icon>
                </template>
              </el-input>
            </div>
            
            <el-button
              type="primary"
              class="login-button"
              :loading="loading"
              :disabled="!username || !password"
              @click="handleLogin"
            >
              {{ loading ? '登录中...' : '登录' }}
            </el-button>
          </div>
        </div>
      </template>

      <!-- 已登录：显示用户信息 -->
      <template v-else>
        <div class="profile-section">
          <!-- 用户头像和基本信息 -->
          <div class="profile-header">
            <div class="avatar-wrapper">
              <img
                v-if="userInfo.photo_url"
                :src="userInfo.photo_url"
                class="avatar-image"
                @error="handleAvatarError"
              />
              <el-icon v-else :size="48" color="var(--text-tertiary)"><User /></el-icon>
            </div>
            <div class="profile-basic">
              <div class="profile-name">{{ userInfo.name || '未知' }}</div>
              <div class="profile-id">{{ userInfo.student_number }}</div>
            </div>
          </div>

          <!-- 详细信息列表 -->
          <div class="profile-details">
            <div class="detail-item" v-if="userInfo.department">
              <div class="detail-label">学院</div>
              <div class="detail-value">{{ userInfo.department }}</div>
            </div>
            <div class="detail-item" v-if="userInfo.major">
              <div class="detail-label">专业</div>
              <div class="detail-value">{{ userInfo.major }}</div>
            </div>
            <div class="detail-item" v-if="userInfo.class_name">
              <div class="detail-label">班级</div>
              <div class="detail-value">{{ userInfo.class_name }}</div>
            </div>
            <div class="detail-item" v-if="userInfo.grade">
              <div class="detail-label">年级</div>
              <div class="detail-value">{{ userInfo.grade }}</div>
            </div>
            <div class="detail-item" v-if="userInfo.gender">
              <div class="detail-label">性别</div>
              <div class="detail-value">{{ userInfo.gender }}</div>
            </div>
          </div>

          <!-- 退出登录按钮 -->
          <el-button
            type="danger"
            plain
            class="logout-button"
            @click="handleLogout"
          >
            <el-icon><SwitchButton /></el-icon>
            退出登录
          </el-button>
        </div>
      </template>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { Close, User, Key, SwitchButton } from '@element-plus/icons-vue';
import type { UserInfo } from '../types';

interface Props {
  modelValue: boolean;
  userInfo?: UserInfo | null;  // 新增：从父组件接收
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
  (e: 'login-success', userInfo: UserInfo): void;
  (e: 'logout'): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
});

// 表单数据
const username = ref('');
const password = ref('');
const loading = ref(false);

// 用户信息 - 优先使用 prop，fallback 到内部状态
const internalUserInfo = ref<UserInfo | null>(null);
const userInfo = computed(() => props.userInfo ?? internalUserInfo.value);

// 登录（带凭证保存）
async function handleLogin() {
  if (!username.value || !password.value) {
    ElMessage.warning('请输入学号和密码');
    return;
  }

  loading.value = true;
  try {
    // 使用新的登录命令（自动保存凭证）
    const info = await invoke<UserInfo>('login_and_save_credentials', {
      username: username.value,
      password: password.value,
    });

    // 更新内部状态并通知父组件
    internalUserInfo.value = info;
    emit('login-success', info);
    ElMessage.success(`登录成功，欢迎 ${info.name || info.student_number}`);

    // 清空密码（安全考虑）
    password.value = '';
  } catch (e) {
    console.error('登录失败:', e);
    ElMessage.error(`${e}`);
  } finally {
    loading.value = false;
  }
}

// 退出登录（清除凭证）
async function handleLogout() {
  try {
    // 使用新的退出登录命令（清除所有凭证）
    await invoke('logout_and_clear');
    internalUserInfo.value = null;
    username.value = '';
    password.value = '';
    emit('logout');
    ElMessage.success('已退出登录');
  } catch (e) {
    console.error('退出登录失败:', e);
    ElMessage.error(`退出失败: ${e}`);
  }
}

// 头像加载失败处理
function handleAvatarError(e: Event) {
  const target = e.target as HTMLImageElement;
  target.style.display = 'none';
}

// 关闭对话框
function handleClose() {
  dialogVisible.value = false;
}

// 对话框打开时检查登录状态
watch(() => props.modelValue, async (newVal) => {
  if (newVal && !userInfo.value) {
    // 没有用户信息，尝试从后端获取
    try {
      const info = await invoke<UserInfo | null>('get_current_user_info');
      if (info) {
        internalUserInfo.value = info;
      }
    } catch (e) {
      console.log('获取登录状态失败:', e);
    }
  }
});
</script>

<style scoped>
.user-profile-dialog .dialog-content {
  padding: 24px;
}

/* 登录部分 */
.login-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px 0;
}

.login-icon {
  width: 100px;
  height: 100px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(99, 102, 241, 0.1) 0%, rgba(139, 92, 246, 0.1) 100%);
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 20px;
}

.login-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-main);
  margin-bottom: 8px;
}

.login-subtitle {
  font-size: 13px;
  color: var(--text-tertiary);
  margin-bottom: 28px;
}

.login-form {
  width: 100%;
  max-width: 320px;
}

.form-item {
  margin-bottom: 16px;
}

.login-button {
  width: 100%;
  height: 48px;
  font-size: 16px;
  font-weight: 600;
  border-radius: 12px;
  margin-top: 8px;
  background: var(--primary-gradient);
  border: none;
}

.login-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.4);
}

/* 用户信息部分 */
.profile-section {
  display: flex;
  flex-direction: column;
  align-items: center;
}

.profile-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 24px;
}

.avatar-wrapper {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: var(--surface-color-light);
  border: 3px solid var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: center;
  overflow: hidden;
  margin-bottom: 12px;
}

.avatar-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.profile-basic {
  text-align: center;
}

.profile-name {
  font-size: 22px;
  font-weight: 700;
  color: var(--text-main);
  margin-bottom: 4px;
}

.profile-id {
  font-size: 14px;
  color: var(--text-secondary);
}

.profile-details {
  width: 100%;
  background: var(--surface-color-light);
  border-radius: 16px;
  padding: 16px;
  margin-bottom: 20px;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  padding: 12px 0;
  border-bottom: 1px solid var(--border-color);
}

.detail-item:last-child {
  border-bottom: none;
}

.detail-label {
  font-size: 14px;
  color: var(--text-tertiary);
}

.detail-value {
  font-size: 14px;
  font-weight: 500;
  color: var(--text-main);
}

.logout-button {
  width: 100%;
  height: 44px;
  font-size: 15px;
  border-radius: 12px;
}

/* 输入框样式 */
.modern-input :deep(.el-input__wrapper) {
  background-color: var(--input-bg) !important;
  box-shadow: none !important;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 4px 12px;
  transition: all 0.2s;
}

.modern-input :deep(.el-input__wrapper:hover),
.modern-input :deep(.el-input__wrapper.is-focus) {
  background-color: var(--input-bg) !important;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1) !important;
}
</style>
