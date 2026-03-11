<template>
  <el-dialog
    v-model="dialogVisible"
    title=""
    width="90%"
    align-center
    :show-close="false"
    class="custom-dialog import-dialog"
    append-to-body
    style="max-width: 480px;"
    @close="handleClose"
  >
    <div class="dialog-header">
      <div class="dialog-title">导入课表</div>
      <div class="dialog-close-btn" @click="handleClose">
        <el-icon :size="20"><Close /></el-icon>
      </div>
    </div>

    <div class="dialog-content">
      <!-- 学年选择 -->
      <div class="setting-item">
        <div class="setting-label">学年</div>
        <el-select v-model="selectedYear" placeholder="选择学年" class="modern-input" style="width: 100%">
          <el-option
            v-for="year in availableYears"
            :key="year.value"
            :label="year.label"
            :value="year.value"
          />
        </el-select>
      </div>

      <!-- 学期选择 -->
      <div class="setting-item">
        <div class="setting-label">学期</div>
        <el-radio-group v-model="selectedTerm" class="term-group" style="width: 100%; display: flex; gap: 10px;">
          <el-radio :value="1" border class="term-radio" style="flex: 1; margin-right: 0;">
            <div style="display: flex; align-items: center; gap: 4px;">
              第一学期
            </div>
          </el-radio>
          <el-radio :value="2" border class="term-radio" style="flex: 1;">
            <div style="display: flex; align-items: center; gap: 4px;">
              第二学期
            </div>
          </el-radio>
        </el-radio-group>
      </div>

      <!-- 课表名称输入 -->
      <div class="setting-item">
        <div class="setting-label">课表名称</div>
        <el-input
          v-model="scheduleName"
          placeholder="请输入课表名称"
          class="modern-input"
          clearable
          @keyup.enter="handleImport"
        >
          <template #prefix>
            <el-icon><Collection /></el-icon>
          </template>
        </el-input>
        <div class="setting-desc">为导入的课表设置一个易于识别的名称</div>
      </div>

      <!-- 第一周第一天日期选择 -->
      <div class="setting-item">
        <div class="setting-label">第一周第一天（周一）</div>
        <el-date-picker
          v-model="firstDayDate"
          type="date"
          placeholder="选择第一周周一的日期"
          format="YYYY-MM-DD"
          value-format="YYYY-MM-DD"
          class="modern-input"
          style="width: 100%"
          :clearable="false"
        />
        <div class="setting-desc">选择校历第一周的周一日期，用于计算当前周次</div>
      </div>
    </div>

    <div class="dialog-footer">
      <el-button @click="handleClose" class="modern-button">取消</el-button>
      <el-button
        type="primary"
        @click="handleImport"
        class="modern-button primary"
        :loading="loading"
        :disabled="!scheduleName.trim()"
      >
        {{ loading ? '导入中...' : '导入' }}
      </el-button>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { Close, Collection } from '@element-plus/icons-vue';

interface Props {
  modelValue: boolean;
}

interface Emits {
  (e: 'update:modelValue', value: boolean): void;
  (e: 'import-success', scheduleId: string): void;
}

const props = defineProps<Props>();
const emit = defineEmits<Emits>();

const dialogVisible = computed({
  get: () => props.modelValue,
  set: (value) => emit('update:modelValue', value)
});

const selectedYear = ref(2024);
const selectedTerm = ref(1);
const scheduleName = ref('');
const firstDayDate = ref('');
const loading = ref(false);

// 生成可选学年列表（最近5年）
const availableYears = computed(() => {
  const currentYear = new Date().getFullYear();
  const years = [];
  for (let i = 0; i < 5; i++) {
    const year = currentYear - i;
    years.push({
      value: year,
      label: `${year}-${year + 1}`
    });
  }
  return years;
});

// 监听 dialog 打开，初始化默认值
watch(() => props.modelValue, (newVal) => {
  if (newVal) {
    // 确定当前学年和学期
    const now = new Date();
    const month = now.getMonth() + 1;
    const year = now.getFullYear();
    
    if (month >= 9) {
      selectedYear.value = year;
      selectedTerm.value = 1;
    } else if (month === 1) {
      selectedYear.value = year - 1;
      selectedTerm.value = 1;
    } else {
      selectedYear.value = year - 1;
      selectedTerm.value = 2;
    }
    
    // 自动生成课表名称
    scheduleName.value = `${selectedYear.value}-${selectedYear.value + 1}-${selectedTerm.value}`;
    
    // 自动计算第一周第一天的默认日期（学期开始日期的周一）
    // 第一学期：9月1日所在周的周一
    // 第二学期：2月20日所在周的周一
    let defaultDate: Date;
    if (selectedTerm.value === 1) {
      // 第一学期：9月1日
      defaultDate = new Date(selectedYear.value, 8, 1); // 9月1日 (月份从0开始)
    } else {
      // 第二学期：2月20日
      defaultDate = new Date(selectedYear.value + 1, 1, 20); // 2月20日
    }
    
    // 调整到该周的周一 (周一为1，周日为0)
    const dayOfWeek = defaultDate.getDay();
    const daysToMonday = dayOfWeek === 0 ? -6 : 1 - dayOfWeek;
    defaultDate.setDate(defaultDate.getDate() + daysToMonday);
    
    // 格式化为 YYYY-MM-DD
    firstDayDate.value = formatDateToString(defaultDate);
    
    loading.value = false;
  }
});

// 格式化日期为 YYYY-MM-DD
function formatDateToString(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

// 监听学年学期变化，更新课表名称
watch([selectedYear, selectedTerm], () => {
  scheduleName.value = `${selectedYear.value}-${selectedYear.value + 1}-${selectedTerm.value}`;
});

const handleClose = () => {
  dialogVisible.value = false;
};

const handleImport = async () => {
  if (!scheduleName.value.trim()) {
    ElMessage.warning('请输入课表名称');
    return;
  }

  if (!firstDayDate.value) {
    ElMessage.warning('请选择第一周第一天日期');
    return;
  }

  loading.value = true;
  try {
    const scheduleId = await invoke<string>('import_schedule_from_saved_login', {
      year: selectedYear.value,
      term: selectedTerm.value,
      scheduleName: scheduleName.value.trim(),
      firstDayDate: firstDayDate.value,
    });

    ElMessage.success('课表导入成功');
    emit('import-success', scheduleId);
    dialogVisible.value = false;
  } catch (e) {
    console.error('导入课表失败:', e);
    ElMessage.error(`${e}`);
  } finally {
    loading.value = false;
  }
};
</script>

<style scoped>
.import-dialog .dialog-content {
  padding: 20px 28px;
}

.setting-item {
  margin-bottom: 24px;
}

.setting-item:last-child {
  margin-bottom: 0;
}

.setting-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 10px;
  display: block;
}

.setting-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 8px;
  line-height: 1.4;
}

.modern-input :deep(.el-input__wrapper),
.modern-input :deep(.el-select__wrapper) {
  background-color: var(--input-bg) !important;
  box-shadow: none !important;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 4px 12px;
  transition: all 0.2s;
}

.modern-input :deep(.el-input__wrapper:hover),
.modern-input :deep(.el-input__wrapper.is-focus),
.modern-input :deep(.el-select__wrapper:hover),
.modern-input :deep(.el-select__wrapper.is-focus) {
  background-color: var(--input-bg) !important;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 2px rgba(99, 102, 241, 0.1) !important;
}

.modern-button {
  border-radius: 12px;
  padding: 12px 24px;
  font-weight: 600;
  transition: all 0.2s;
  height: auto;
}

.modern-button.primary {
  background: var(--primary-gradient);
  border: none;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
  color: white;
}

.modern-button.primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.4);
}

.modern-button.primary:active:not(:disabled) {
  transform: scale(0.98);
}

.term-radio {
  border-radius: 8px;
  margin-right: 0 !important;
}

/* 深色模式适配 */
html.dark .modern-input :deep(.el-input__wrapper.is-focus),
html.dark .modern-input :deep(.el-select__wrapper.is-focus) {
  background-color: var(--input-bg) !important;
}
</style>
