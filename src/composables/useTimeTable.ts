import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import type { TimeTable } from '../types';

export function useTimeTable() {
    const timeTables = ref<TimeTable[]>([]);
    const loading = ref(false);
    const error = ref<string | null>(null);

    /**
     * 获取所有时间表
     */
    async function listTimeTables(): Promise<TimeTable[]> {
        loading.value = true;
        try {
            const result = await invoke<TimeTable[]>('list_time_tables');
            timeTables.value = result;
            return result;
        } catch (e) {
            error.value = String(e);
            throw e;
        } finally {
            loading.value = false;
        }
    }

    /**
     * 保存时间表
     */
    async function saveTimeTable(timeTable: TimeTable): Promise<void> {
        try {
            await invoke('save_time_table', { timeTable });
            await listTimeTables(); // 刷新列表
        } catch (e) {
            console.error('保存时间表失败:', e);
            throw e;
        }
    }

    /**
     * 删除时间表
     */
    async function deleteTimeTable(id: string): Promise<void> {
        try {
            await invoke('delete_time_table', { id });
            await listTimeTables(); // 刷新列表
        } catch (e) {
            console.error('删除时间表失败:', e);
            throw e;
        }
    }

    return {
        timeTables,
        loading,
        error,
        listTimeTables,
        saveTimeTable,
        deleteTimeTable,
    };
}
