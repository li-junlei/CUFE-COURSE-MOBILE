<template>
  <div class="time-table-manager">
    <!-- 时间表列表 -->
    <div class="table-list">
      <div
        v-for="table in timeTables"
        :key="table.id"
        class="table-list-item"
        :class="{ 'is-selected': modelValue === table.id }"
        @click="handleSelect(table.id)"
      >
        <div class="item-main">
          <div class="item-header">
            <span class="item-name">{{ table.name }}</span>
            <el-tag v-if="table.id === 'default'" size="small" effect="dark" round class="status-tag">默认</el-tag>
          </div>
          <div class="item-meta">
            <span><el-icon><Clock /></el-icon> {{ table.periods.length }} 节次</span>
          </div>
        </div>

        <div class="item-actions">
          <el-button
            circle
            text
            @click.stop="handleEdit(table)"
            class="action-btn"
            title="编辑"
          >
            <el-icon><Edit /></el-icon>
          </el-button>
          <el-button
            v-if="table.id !== 'default'"
            circle
            text
            type="danger"
            @click.stop="handleDelete(table.id)"
            class="action-btn danger"
            title="删除"
          >
            <el-icon><Delete /></el-icon>
          </el-button>
        </div>
      </div>

      <div v-if="timeTables.length === 0" class="empty-state">
        <el-icon :size="48" class="empty-icon"><Clock /></el-icon>
        <p>暂无时间表</p>
      </div>
    </div>

    <!-- 底部新建按钮 -->
    <div class="dialog-footer">
      <el-button type="primary" class="modern-button primary full-width" @click="handleCreate">
        <el-icon style="margin-right: 6px;"><Plus /></el-icon> 新增时间表
      </el-button>
    </div>

    <!-- 编辑对话框 -->
    <el-dialog
      v-model="showEditDialog"
      :title="isCreating ? '新建时间表' : '编辑时间表'"
      width="90%"
      align-center
      append-to-body
      class="custom-dialog"
      style="max-width: 480px;"
    >
      <div class="dialog-content" v-loading="saving">
        <div class="setting-item">
          <div class="setting-label">名称</div>
          <el-input
            v-model="editForm.name"
            placeholder="请输入时间表名称"
            class="modern-input"
          />
        </div>

        <div class="setting-item">
          <div class="label-row">
            <div class="setting-label">节次设置</div>
            <el-button link type="primary" size="small" @click="addPeriod">
              <el-icon><Plus /></el-icon> 添加节次
            </el-button>
          </div>

          <div class="periods-list">
            <div v-for="(p, index) in editForm.periods" :key="index" class="period-row">
              <div class="period-index">{{ index + 1 }}</div>
              <el-input v-model="p.start" placeholder="08:00" class="time-input modern-input" />
              <span class="sep">-</span>
              <el-input v-model="p.end" placeholder="08:45" class="time-input modern-input" />
              <el-button circle text type="danger" size="small" @click="removePeriod(index)" class="delete-period-btn">
                <el-icon><Close /></el-icon>
              </el-button>
            </div>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <el-button @click="showEditDialog = false" class="modern-button">取消</el-button>
        <el-button type="primary" @click="saveEdit" class="modern-button primary">保存</el-button>
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { ElMessage, ElMessageBox } from 'element-plus';
import { Plus, Delete, Clock, Close, Edit } from '@element-plus/icons-vue';
import { useTimeTable } from '../composables/useTimeTable';
import type { TimeTable } from '../types';

const props = defineProps<{
  modelValue?: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', id: string): void;
  (e: 'select', table: TimeTable): void;
  (e: 'saved'): void;
  (e: 'deleted'): void;
}>();

const { timeTables, listTimeTables, saveTimeTable, deleteTimeTable } = useTimeTable();

const showEditDialog = ref(false);
const isCreating = ref(false);
const saving = ref(false);

const editForm = ref<TimeTable>({
  id: '',
  name: '',
  periods: []
});

onMounted(() => {
  listTimeTables();
});

function handleSelect(id: string) {
  if (props.modelValue !== undefined) {
    emit('update:modelValue', id);
    const table = timeTables.value.find(t => t.id === id);
    if (table) emit('select', table);
  }
}

function handleCreate() {
  isCreating.value = true;
  showEditDialog.value = true;
  editForm.value = {
    id: crypto.randomUUID(),
    name: '新时间表',
    periods: [
      { start: "08:00", end: "08:45" },
      { start: "08:55", end: "09:40" },
      { start: "10:00", end: "10:45" },
      { start: "10:55", end: "11:40" }
    ]
  };
}

function handleEdit(table: TimeTable) {
  isCreating.value = false;
  showEditDialog.value = true;
  editForm.value = JSON.parse(JSON.stringify(table));
}

function addPeriod() {
  const last = editForm.value.periods[editForm.value.periods.length - 1];
  let start = "08:00";
  if (last) {
    const [h, _m] = last.end.split(':').map(Number);
    const nextH = h + 1;
    start = `${nextH < 10 ? '0'+nextH : nextH}:${last.end.split(':')[1]}`;
  }
  editForm.value.periods.push({ start, end: start });
}

function removePeriod(index: number) {
  editForm.value.periods.splice(index, 1);
}

async function saveEdit() {
  if (!editForm.value.name) {
    ElMessage.warning('请输入名称');
    return;
  }
  if (editForm.value.periods.length === 0) {
    ElMessage.warning('至少需要一个节次');
    return;
  }

  saving.value = true;
  try {
    await saveTimeTable(editForm.value);
    ElMessage.success('保存成功');
    showEditDialog.value = false;
    emit('saved');
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  } finally {
    saving.value = false;
  }
}

async function handleDelete(id: string) {
  try {
    await ElMessageBox.confirm('确定要删除这个时间表吗？', '提示', {
      type: 'warning',
      confirmButtonText: '删除',
      cancelButtonText: '取消'
    });
    await deleteTimeTable(id);
    ElMessage.success('删除成功');
    emit('deleted');
  } catch (e) {
    if (e !== 'cancel') ElMessage.error(`删除失败: ${e}`);
  }
}
</script>

<style scoped>
.time-table-manager {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.table-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
  padding: 4px;
}

/* 列表项样式 - 完全复制课表管理 */
.table-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  background-color: var(--card-bg);
  border-radius: 14px;
  border: 1px solid var(--border-color);
  transition: all 0.2s;
  cursor: pointer;
  box-shadow: var(--shadow-sm);
}

.table-list-item:hover {
  background-color: var(--surface-color-strong);
  transform: translateX(4px);
  box-shadow: var(--shadow-sm);
}

.table-list-item.is-selected {
  background: var(--surface-color-strong);
  border-color: var(--primary-color);
  box-shadow: var(--shadow-active);
}

.item-main {
  flex: 1;
  min-width: 0;
  margin-right: 12px;
}

.item-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 4px;
}

.item-name {
  font-weight: 700;
  color: var(--text-main);
  font-size: 15px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.status-tag {
  flex-shrink: 0;
}

.item-meta {
  display: flex;
  gap: 12px;
  font-size: 12px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* 按钮样式 - 完全复制课表管理 */
.item-actions {
  display: flex;
  gap: 4px;
  opacity: 1;
  flex-shrink: 0;
}

.action-btn {
  width: 32px;
  height: 32px;
  font-size: 16px;
  color: var(--text-secondary);
  border-radius: 8px;
}

.action-btn:hover {
  background-color: rgba(99, 102, 241, 0.1);
  color: var(--primary-color);
}

.action-btn.danger:hover {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

/* 空状态 */
.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
  color: var(--text-tertiary);
  gap: 12px;
}

/* 对话框样式 */
.dialog-content {
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

.label-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.periods-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.period-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.period-index {
  width: 24px;
  font-size: 12px;
  color: var(--text-tertiary);
  text-align: center;
}

.time-input {
  flex: 1;
}

.sep {
  color: var(--text-tertiary);
}

.delete-period-btn {
  opacity: 0.5;
}

.delete-period-btn:hover {
  opacity: 1;
}

.dialog-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background-color: var(--surface-color-light);
}

.modern-input .el-input__wrapper {
  background-color: var(--input-bg) !important;
  box-shadow: none !important;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 4px 12px;
  transition: all 0.2s;
}

.modern-input .el-input__wrapper:hover,
.modern-input .el-input__wrapper.is-focus {
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

.modern-button.primary:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.4);
}

.modern-button.primary:active {
  transform: scale(0.98);
}

.full-width {
  width: 100%;
}
</style>
