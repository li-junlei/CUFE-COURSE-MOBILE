<template>
  <el-dialog
    :model-value="modelValue"
    @update:model-value="emit('update:modelValue', $event)"
    title="编辑课表设置"
    width="90%"
    align-center
    class="custom-dialog"
    append-to-body
    style="max-width: 480px;"
  >
    <div class="dialog-content no-scrollbar" v-loading="loading">
      
      <!-- 1. 上课时间 (Time Table) -->
      <div class="setting-section">
        <div class="section-title">上课时间</div>
        <div class="setting-item clickable" @click="showTimeTableManager = true">
            <div class="setting-row">
                <div class="setting-label">时间表方案</div>
                <div class="setting-value">
                    <span>{{ currentTimeTableName }}</span>
                    <el-icon><ArrowRight /></el-icon>
                </div>
            </div>
            <div class="setting-desc">点击选择或编辑时间表</div>
        </div>

        <!-- Preview of current time table -->
        <div class="time-preview" v-if="currentTimeTable">
            <div class="preview-title">时间表预览</div>
            <div class="preview-grid">
                <div v-for="(p, i) in previewPeriods" :key="i" class="preview-item">
                    <span class="idx">{{ i + 1 }}</span>
                    <span class="time">{{ p.start }} - {{ p.end }}</span>
                </div>
                <div v-if="currentTimeTable.periods.length > 8" class="preview-more">...</div>
            </div>
        </div>
      </div>

      <!-- 2. 课表名称 -->
      <div class="setting-section">
        <div class="section-title">课表信息</div>
        <div class="setting-item">
          <div class="setting-label">课表名称</div>
          <el-input
            v-model="form.scheduleName"
            placeholder="输入课表名称"
            class="modern-input"
            style="width: 100%"
            clearable
          />
        </div>
      </div>

      <!-- 3. 第一周第一天 -->
      <div class="setting-section">
        <div class="section-title">学期设置</div>
        <div class="setting-item">
            <div class="setting-label">第一周第一天</div>
            <el-date-picker
                v-model="form.firstDayDate"
                type="date"
                placeholder="选择日期"
                format="YYYY-MM-DD"
                value-format="YYYY-MM-DD"
                class="modern-input"
                style="width: 100%"
            />
        </div>

        <!-- 3. 一天课程节数 -->
        <div class="setting-item">
            <div class="setting-label">每天节数</div>
            <el-input-number 
                v-model="form.maxPeriods" 
                :min="4" 
                :max="20" 
                class="modern-number-input" 
                controls-position="right"
                style="width: 100%"
            />
        </div>

        <!-- 4. 学期周数 -->
        <div class="setting-item">
            <div class="setting-label">学期周数</div>
            <el-input-number 
                v-model="form.weeksCount" 
                :min="10" 
                :max="30" 
                class="modern-number-input" 
                controls-position="right"
                style="width: 100%"
            />
        </div>
      </div>

      <!-- 5. 应用到全部 -->
      <div class="apply-all-section">
        <el-popconfirm
            title="确定将此设置应用到所有课表吗？这将覆盖其他课表的设置。"
            @confirm="handleApplyAll"
        >
            <template #reference>
                <el-button type="warning" link>
                    <el-icon><CopyDocument /></el-icon> 将此设置应用到全部课表
                </el-button>
            </template>
        </el-popconfirm>
      </div>

    </div>

    <div class="dialog-footer">
      <el-button @click="emit('update:modelValue', false)" class="modern-button">取消</el-button>
      <el-button type="primary" @click="handleSave" class="modern-button primary">保存</el-button>
    </div>

    <!-- Nested TimeTable Manager Dialog - 使用 Teleport 避免嵌套问题 -->
    <Teleport to="body">
      <el-dialog
          v-model="showTimeTableManager"
          title="管理时间表"
          width="90%"
          align-center
          append-to-body
          class="custom-dialog"
          style="max-width: 480px; height: 600px;"
      >
          <TimeTableManager
              v-model="form.timeTableId"
              @select="handleTimeTableSelect"
              @saved="handleTimeTableSaved"
              @deleted="handleTimeTableSaved"
          />
      </el-dialog>
    </Teleport>

  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { ElMessage } from 'element-plus';
import { ArrowRight, CopyDocument } from '@element-plus/icons-vue';
import TimeTableManager from './TimeTableManager.vue';
import { useCourse } from '../composables/useCourse';
import { useTimeTable } from '../composables/useTimeTable';
import { formatDateString } from '../utils/date';
import type { ScheduleMetadata, TimeTable } from '../types';

const props = defineProps<{
  modelValue: boolean;
  scheduleId?: string; // ID of the schedule being edited
  initialData?: ScheduleMetadata; // Optional initial metadata
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', val: boolean): void;
  (e: 'saved'): void;
}>();

const { updateScheduleInfo, applySettingsToAll, renameSchedule } = useCourse();
const { timeTables, listTimeTables } = useTimeTable();

const loading = ref(false);
const showTimeTableManager = ref(false);

const form = ref({
    scheduleName: '',
    firstDayDate: '',
    maxPeriods: 13,
    weeksCount: 20,
    timeTableId: ''
});

// Computed properties for display
const currentTimeTable = computed(() => {
    return timeTables.value.find(t => t.id === form.value.timeTableId);
});

const currentTimeTableName = computed(() => {
    return currentTimeTable.value?.name || '未选择 (默认)';
});

const previewPeriods = computed(() => {
    // Show first 8 periods
    return currentTimeTable.value?.periods.slice(0, 8) || [];
});

/**
 * Load initial data when dialog opens
 */
watch(() => props.modelValue, async (val) => {
    console.log('ScheduleEditDialog modelValue changed:', val);
    if (val && props.initialData) {
        console.log('Initializing dialog with data:', props.initialData);
        // Initialize form from props
        const meta = props.initialData;

        // Schedule name
        form.value.scheduleName = meta.name || '';

        // Date - 使用本地时间格式化，而不是 UTC
        form.value.firstDayDate = meta.first_day
            ? formatDateString(new Date(meta.first_day * 1000))
            : '';

        // Configs (with defaults)
        form.value.maxPeriods = meta.max_periods || 13;
        form.value.weeksCount = meta.weeks_count || 20;
        form.value.timeTableId = meta.time_table_id || '';

        // Ensure time tables are loaded
        if (timeTables.value.length === 0) {
            await listTimeTables();
        }

        // If no time table selected, try to select default
        if (!form.value.timeTableId) {
            const def = timeTables.value.find(t => t.id === 'default');
            if (def) form.value.timeTableId = def.id;
        }
    }
});

function handleTimeTableSelect(table: TimeTable) {
    if (table) {
        form.value.timeTableId = table.id;
        showTimeTableManager.value = false;
    }
}

// 处理时间表保存事件，刷新列表
async function handleTimeTableSaved() {
    await listTimeTables();
}

async function handleSave() {
    if (!props.scheduleId) return;

    loading.value = true;
    try {
        // Rename schedule if name changed
        if (form.value.scheduleName &&
            form.value.scheduleName !== props.initialData?.name) {
            await renameSchedule(props.scheduleId, form.value.scheduleName);
        }

        // Convert date to timestamp
        let firstDayTimestamp: number | undefined = undefined;
        if (form.value.firstDayDate) {
            const parts = form.value.firstDayDate.split('-');
            const date = new Date(parseInt(parts[0]), parseInt(parts[1]) - 1, parseInt(parts[2]));
            firstDayTimestamp = Math.floor(date.getTime() / 1000);
        }

        await updateScheduleInfo(
            props.scheduleId,
            firstDayTimestamp,
            form.value.maxPeriods,
            form.value.weeksCount,
            form.value.timeTableId
        );
        
        ElMessage.success('设置保存成功');
        emit('saved');
        emit('update:modelValue', false);
    } catch (e) {
        ElMessage.error(`保存失败: ${e}`);
    } finally {
        loading.value = false;
    }
}

async function handleApplyAll() {
    if (!props.scheduleId) return;
    
    // First save current
    await handleSave();
    
    // Then apply to all
    loading.value = true;
    try {
        await applySettingsToAll(props.scheduleId);
        ElMessage.success('已应用到所有课表');
        emit('saved'); // Refresh parent
    } catch (e) {
        ElMessage.error(`应用失败: ${e}`);
    } finally {
        loading.value = false;
    }
}
</script>

<style scoped>
.setting-section {
    margin-bottom: 24px;
}

.section-title {
    font-size: 14px;
    font-weight: 600;
    color: var(--text-tertiary);
    margin-bottom: 12px;
}

.setting-item {
    margin-bottom: 16px;
    padding: 12px;
    background-color: var(--surface-color-light);
    border-radius: 12px;
    border: 1px solid var(--border-color);
}

.setting-item.clickable {
    cursor: pointer;
    transition: all 0.2s;
}
.setting-item.clickable:hover {
    background-color: var(--surface-color-strong);
}

.setting-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 4px;
}

.setting-label {
    font-size: 14px;
    color: var(--text-main);
    font-weight: 500;
}

.setting-value {
    display: flex;
    align-items: center;
    gap: 4px;
    color: var(--primary-color);
    font-size: 14px;
}

.setting-desc {
    font-size: 12px;
    color: var(--text-tertiary);
}

.time-preview {
    margin-top: 12px;
    padding: 12px;
    background-color: rgba(0,0,0,0.03);
    border-radius: 8px;
}

.preview-title {
    font-size: 12px;
    color: var(--text-tertiary);
    margin-bottom: 8px;
}

.preview-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
}

.preview-item {
    font-size: 12px;
    color: var(--text-secondary);
    display: flex;
    gap: 8px;
}

.preview-item .idx {
    color: var(--text-tertiary);
    width: 14px;
}

.preview-more {
    grid-column: span 2;
    text-align: center;
    font-size: 12px;
    color: var(--text-tertiary);
}

.apply-all-section {
    margin-top: 24px;
    text-align: center;
}
</style>
