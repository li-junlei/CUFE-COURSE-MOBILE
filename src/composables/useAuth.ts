// 用户认证状态管理 Composable
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { UserInfo, SessionStatus } from '../types';

const globalUserInfo = ref<UserInfo | null>(null);
const SESSION_CHECK_INTERVAL_MS = 60 * 60 * 1000;

let sessionKeepAliveTimer: ReturnType<typeof setInterval> | null = null;
let sessionCheckInFlight: Promise<UserInfo | null> | null = null;

/**
 * 恢复登录状态
 * 应用启动时调用，尝试使用保存的凭证自动登录
 */
export async function restoreLoginState(): Promise<UserInfo | null> {
  try {
    const userInfo = await invoke<UserInfo>('restore_login_session');
    globalUserInfo.value = userInfo;
    return userInfo;
  } catch (error) {
    console.log('恢复登录状态失败:', error);
    globalUserInfo.value = null;
    return null;
  }
}

/**
 * 登录并保存凭证
 */
export async function loginWithCredentials(username: string, password: string): Promise<UserInfo> {
  const userInfo = await invoke<UserInfo>('login_and_save_credentials', { username, password });
  globalUserInfo.value = userInfo;
  return userInfo;
}

/**
 * 登录（不保存凭证）
 */
export async function login(username: string, password: string): Promise<UserInfo> {
  const userInfo = await invoke<UserInfo>('login_and_get_user_info', { username, password });
  globalUserInfo.value = userInfo;
  return userInfo;
}

/**
 * 退出登录并清除凭证
 */
export async function logout(): Promise<void> {
  await invoke('logout_and_clear');
  globalUserInfo.value = null;
}

/**
 * 检查是否已登录
 */
export async function checkLoginStatus(): Promise<boolean> {
  const loggedIn = await invoke<boolean>('is_logged_in');
  return loggedIn;
}

/**
 * 获取当前用户信息
 */
export async function getCurrentUserInfo(): Promise<UserInfo | null> {
  try {
    const userInfo = await invoke<UserInfo | null>('get_current_user_info');
    if (userInfo) {
      globalUserInfo.value = userInfo;
    }
    return userInfo;
  } catch (error) {
    console.log('获取用户信息失败:', error);
    return null;
  }
}

/**
 * 检查当前登录会话，如已失效则静默自动重登录
 */
export async function ensureLoginSession(): Promise<UserInfo | null> {
  if (sessionCheckInFlight) {
    return sessionCheckInFlight;
  }

  sessionCheckInFlight = (async () => {
    try {
      const status = await invoke<SessionStatus>('ensure_login_session');
      globalUserInfo.value = status.userInfo;
      return status.userInfo;
    } catch (error) {
      console.log('定时检查登录状态失败:', error);
      return globalUserInfo.value;
    } finally {
      sessionCheckInFlight = null;
    }
  })();

  return sessionCheckInFlight;
}

/**
 * 启动登录会话保活
 */
export function startSessionKeepAlive(options: { immediate?: boolean } = {}): void {
  if (sessionKeepAliveTimer) {
    return;
  }

  if (options.immediate) {
    void ensureLoginSession();
  }

  sessionKeepAliveTimer = setInterval(() => {
    void ensureLoginSession();
  }, SESSION_CHECK_INTERVAL_MS);
}

/**
 * 停止登录会话保活
 */
export function stopSessionKeepAlive(): void {
  if (!sessionKeepAliveTimer) {
    return;
  }

  clearInterval(sessionKeepAliveTimer);
  sessionKeepAliveTimer = null;
}

export function useAuth() {
  return {
    globalUserInfo,
    restoreLoginState,
    loginWithCredentials,
    login,
    logout,
    checkLoginStatus,
    getCurrentUserInfo,
    ensureLoginSession,
    startSessionKeepAlive,
    stopSessionKeepAlive,
  };
}
