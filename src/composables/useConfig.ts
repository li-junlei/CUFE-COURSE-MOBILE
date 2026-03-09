// 应用配置状态管理 Composable
import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import type { AppConfig, UpdateInfo } from '../types';

const config = ref<AppConfig>({});
const backgroundImage = ref('');
const loading = ref(false);

/**
 * 加载应用配置
 */
export async function loadAppConfig(): Promise<AppConfig> {
  try {
    const loadedConfig = await invoke<AppConfig>('get_app_config');
    config.value = loadedConfig;
    backgroundImage.value = loadedConfig.background_image || '';
    return loadedConfig;
  } catch (error) {
    console.error('加载配置失败:', error);
    return {};
  }
}

/**
 * 保存应用配置
 */
export async function saveAppConfig(newConfig: AppConfig): Promise<void> {
  await invoke('save_app_config', { config: newConfig });
  config.value = newConfig;
}

/**
 * 保存背景图
 */
export async function saveBackgroundImage(): Promise<string | null> {
  return new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/jpeg, image/png, image/webp';
    input.onchange = async (event: any) => {
      const file = event.target.files?.[0];
      if (!file) {
        resolve(null);
        return;
      }

      try {
        const buffer = await file.arrayBuffer();
        const bytes = Array.from(new Uint8Array(buffer));
        const storedPath = await invoke<string>('upload_background_image', { bytes });
        backgroundImage.value = storedPath;
        resolve(storedPath);
      } catch (error) {
        console.error('上传背景图失败:', error);
        resolve(null);
      }
    };
    input.click();
  });
}

/**
 * 删除背景图
 */
export async function deleteBackgroundImage(): Promise<void> {
  await invoke('delete_background_image');
  backgroundImage.value = '';
}

/**
 * 切换网格辅助线显示
 */
export async function toggleGridLines(show: boolean): Promise<void> {
  config.value = { ...config.value, show_grid_lines: show };
  await saveAppConfig(config.value);
}

/**
 * 切换显示授课老师
 */
export async function toggleTeacher(show: boolean): Promise<void> {
  config.value = { ...config.value, show_teacher: show };
  await saveAppConfig(config.value);
}

/**
 * 切换显示上课地点
 */
export async function toggleLocation(show: boolean): Promise<void> {
  config.value = { ...config.value, show_location: show };
  await saveAppConfig(config.value);
}

/**
 * 切换简化地点显示
 */
export async function toggleSimplifiedLocation(show: boolean): Promise<void> {
  config.value = { ...config.value, simplified_location: show };
  await saveAppConfig(config.value);
}

/**
 * 更新卡片透明度
 */
export async function updateCardOpacity(opacity: number): Promise<void> {
  config.value = { ...config.value, card_opacity: opacity };
  await saveAppConfig(config.value);
}

/** 当前应用版本号 */
const CURRENT_VERSION = '2.6.0';

/**
 * 检查更新
 * @param isManual true 表示手动检查（不受跳过版本限制）
 */
export async function checkForUpdate(isManual: boolean = false): Promise<UpdateInfo | null> {
  try {
    const appConfig = await invoke<AppConfig>('get_app_config');

    const result = await invoke<UpdateInfo | null>('check_update', {
      currentVersion: CURRENT_VERSION,
      autoCheck: !isManual,
      skippedVersion: isManual ? null : appConfig.skipped_version,
    });

    return result;
  } catch (error) {
    console.error('检查更新失败:', error);
    if (isManual) {
      ElMessage.error('网络连接失败');
    }
    return null;
  }
}

/**
 * 跳过指定版本
 */
export async function skipVersion(version: string): Promise<void> {
  try {
    const appConfig = await invoke<AppConfig>('get_app_config');
    appConfig.skipped_version = version;
    await invoke('save_app_config', { config: appConfig });
    config.value = appConfig;
  } catch (error) {
    console.error('保存跳过版本失败:', error);
  }
}

export function useConfig() {
  return {
    config,
    backgroundImage,
    loading,
    loadAppConfig,
    saveAppConfig,
    saveBackgroundImage,
    deleteBackgroundImage,
    toggleGridLines,
    toggleTeacher,
    toggleLocation,
    toggleSimplifiedLocation,
    updateCardOpacity,
    checkForUpdate,
    skipVersion,
  };
}
