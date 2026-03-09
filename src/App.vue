<template>
  <el-config-provider :locale="zhCn">
  <div id="app" :style="{ backgroundImage: backgroundImage ? `url(${backgroundImage})` : 'none' }">
    <!-- 自定义导航栏 -->
    <div class="navbar">
      <div class="navbar-left" @click="showPopup = !showPopup">
        <el-icon :size="22"><MoreFilled /></el-icon>
      </div>
      <div class="navbar-center" @click="showWeekSelector = !showWeekSelector">
        <span>第 {{ currentWeek }} 周</span>
        <el-icon :size="14"><ArrowDown /></el-icon>
      </div>
      <div class="navbar-right" v-if="semesterStatus">
        <span class="semester-status" :class="semesterStatus.type">
          {{ semesterStatus.text }}
        </span>
      </div>
    </div>

    <!-- 弹出菜单 -->
      <PopupMenu
      :show="showPopup"
      @close="showPopup = false"
      @schedule-manage="handleScheduleManage"
      @import-schedule="handleNewSchedule"
      @appearance="showAppearanceDialog = true"
      @settings="showSettingsDialog = true"
      @profile="showUserProfileDialog = true"
    />

    <!-- 加载状态 -->
    <div v-if="loading" class="loading">
      <el-icon class="is-loading" :size="20"><Loading /></el-icon>
      <span>正在从教务系统获取数据...</span>
    </div>

    <!-- 导入课表对话框 -->
    <ImportScheduleDialog
      v-model="showImportDialog"
      @import-success="handleImportSuccess"
    />

    <!-- 设置对话框 -->
    <el-dialog
      v-model="showSettingsDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 460px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">设置</div>
        <div class="dialog-close-btn" @click="showSettingsDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>

      <div class="dialog-content no-scrollbar">
        <!-- 提醒功能设置 -->
        <div class="settings-section">
          <div class="section-title">提醒功能</div>
          <div class="setting-item">
            <span>启用上课提醒</span>
            <el-switch v-model="reminderEnabled" />
          </div>
          <div class="setting-item" v-if="reminderEnabled">
            <span>提醒调试日志</span>
            <el-switch v-model="reminderDebugLogging" />
          </div>
          <div class="setting-hint">
            情况A：无紧邻前序课程 → 课程开始前15分钟提醒<br>
            情况B：有紧邻前序课程 → 上一节课结束前3分钟提醒
          </div>
          <div class="setting-hint" v-if="reminderEnabled && reminderDebugLogging">
            调试日志将输出到开发者控制台（F12）以便排查提醒触发时机
          </div>
          <!-- 开发阶段测试按钮 -->
          <div class="setting-item test-button" v-if="reminderEnabled">
            <el-button type="primary" size="small" @click="testNotification">
              测试通知
            </el-button>
            <span class="test-hint">点击模拟触发一次课程提醒</span>
          </div>
        </div>


        <!-- 更新设置 -->
        <div class="settings-section">
          <div class="section-title">软件更新</div>
          <div class="setting-item">
            <span>自动检查更新</span>
            <el-switch v-model="config.auto_check_update" />
          </div>
          <div class="setting-item">
            <el-button size="small" @click="handleManualCheckUpdate">
              检查更新
            </el-button>
          </div>
        </div>
      </div>

      <div class="dialog-footer">
        <el-button @click="showSettingsDialog = false" class="modern-button">取消</el-button>
        <el-button type="primary" @click="saveSettings" class="modern-button primary">
          保存更改
        </el-button>
      </div>
    </el-dialog>

    <!-- 课表外观对话框 -->
    <el-dialog
      v-model="showAppearanceDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 500px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">外观设置</div>
        <div class="dialog-close-btn" @click="showAppearanceDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>

      <div class="dialog-content">
        <div class="appearance-grid">
          <div class="appearance-item" @click="handleUploadBackground">
            <div class="item-preview bg-preview">
               <el-image
                 v-if="backgroundImage"
                 :src="backgroundImage"
                 fit="cover"
                 style="width: 100%; height: 100%;"
               />
               <div v-else class="placeholder-icon">
                 <el-icon :size="32"><Picture /></el-icon>
               </div>
               <div class="hover-overlay">
                 <el-icon><Upload /></el-icon>
                 <span>更换背景</span>
               </div>
            </div>
            <div class="item-info">
              <div class="item-title">自定义背景图</div>
              <div class="item-desc">支持 JPG/PNG/WEBP</div>
            </div>
          </div>

          <div class="appearance-item" :class="{ disabled: !backgroundImage }" @click="backgroundImage && handleDeleteBackground()">
            <div class="item-preview delete-preview">
               <el-icon :size="32"><Delete /></el-icon>
            </div>
            <div class="item-info">
              <div class="item-title">移除背景</div>
              <div class="item-desc">恢复默认纯净背景</div>
            </div>
          </div>

          <!-- 网格辅助线开关 -->
          <div class="appearance-item switch-item" style="cursor: default;">
            <div class="item-preview switch-preview">
               <el-icon :size="32"><Grid /></el-icon>
            </div>
            <div class="item-info">
              <div class="item-title">显示网格辅助线</div>
              <div class="item-desc">帮助对齐课程卡片</div>
              <div class="item-control">
                <el-switch
                  v-model="config.show_grid_lines"
                  @change="handleToggleGridLines"
                  :active-icon="Check"
                  :inactive-icon="Close"
                  style="--el-switch-on-color: var(--primary-color);"
                />
              </div>
            </div>
          </div>

          <!-- 显示授课老师开关 -->
          <div class="appearance-item switch-item" style="cursor: default;">
            <div class="item-preview switch-preview">
               <el-icon :size="32"><User /></el-icon>
            </div>
            <div class="item-info">
              <div class="item-title">显示授课老师</div>
              <div class="item-desc">在课程卡片中显示教师姓名</div>
              <div class="item-control">
                <el-switch
                  v-model="config.show_teacher"
                  @change="handleToggleTeacher"
                  :active-icon="Check"
                  :inactive-icon="Close"
                  style="--el-switch-on-color: var(--primary-color);"
                />
              </div>
            </div>
          </div>

          <!-- 显示上课地点开关 -->
          <div class="appearance-item switch-item" style="cursor: default;">
            <div class="item-preview switch-preview">
               <el-icon :size="32"><Location /></el-icon>
            </div>
            <div class="item-info">
              <div class="item-title">显示上课地点</div>
              <div class="item-desc">在课程卡片中显示教室位置</div>
              <div class="item-control">
                <el-switch
                  v-model="config.show_location"
                  @change="handleToggleLocation"
                  :active-icon="Check"
                  :inactive-icon="Close"
                  style="--el-switch-on-color: var(--primary-color);"
                />
              </div>
            </div>
          </div>

          <!-- 简化地点显示开关 -->
          <div class="appearance-item switch-item" style="cursor: default;">
            <div class="item-preview switch-preview">
               <el-icon :size="32"><Edit /></el-icon>
            </div>
            <div class="item-info">
              <div class="item-title">简化地点显示</div>
              <div class="item-desc">隐藏"沙河校区"等前缀</div>
              <div class="item-control">
                <el-switch
                  v-model="config.simplified_location"
                  @change="handleToggleSimplifiedLocation"
                  :active-icon="Check"
                  :inactive-icon="Close"
                  style="--el-switch-on-color: var(--primary-color);"
                  :disabled="!config.show_location"
                />
              </div>
            </div>
          </div>

          <!-- 课程卡片不透明度 -->
          <div class="appearance-item slider-item" style="cursor: default;">
            <div class="item-preview slider-preview">
               <el-icon :size="32"><View /></el-icon>
            </div>
            <div class="item-info">
              <div class="item-title">课程卡片不透明度</div>
              <div class="item-desc">{{ config.card_opacity }}%</div>
              <div class="item-control">
                <el-slider
                  v-model.number="config.card_opacity"
                  :min="50"
                  :max="100"
                  :step="5"
                  @change="handleCardOpacityChange"
                  :show-tooltip="false"
                  style="width: 100%;"
                />
              </div>
            </div>
          </div>
        </div>
      </div>
    </el-dialog>

    <!-- 课表管理对话框 -->
    <el-dialog
      v-model="showScheduleManageDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 520px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">课表管理</div>
        <div class="dialog-actions" style="margin-left: auto; margin-right: 12px; display: flex; gap: 8px;">
           <el-button 
             v-if="scheduleList.length > 1"
             :type="isSorting ? 'success' : 'info'" 
             link 
             size="small"
             @click="toggleSortMode"
           >
             <el-icon style="margin-right: 4px;"><component :is="isSorting ? 'Check' : 'Sort'" /></el-icon>
             {{ isSorting ? '完成' : '排序' }}
           </el-button>
        </div>
        <div class="dialog-close-btn" @click="showScheduleManageDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>

      <div class="dialog-content no-scrollbar">
        <VueDraggable 
          class="schedule-list-container"
          v-model="scheduleList"
          item-key="id"
          :disabled="!isSorting"
          animation="200"
          ghost-class="sortable-ghost"
          drag-class="sortable-drag"
          :force-fallback="true"
        >
          <template #item="{ element: schedule }">
            <div 
              class="schedule-list-item"
              :class="{ 
                'is-active': schedule.id === currentScheduleId,
                'is-sorting': isSorting
              }"
              @click="!isSorting && schedule.id !== currentScheduleId && handleSwitchSchedule(schedule.id)"
            >
              <div class="item-main">
                <div class="item-header">
                  <span class="item-name">{{ schedule.name }}</span>
                  <el-tag v-if="schedule.id === currentScheduleId" size="small" effect="dark" round class="status-tag">当前使用</el-tag>
                </div>
                <div class="item-meta">
                  <span><el-icon><Collection /></el-icon> {{ schedule.course_count }} 门课</span>
                  <span v-if="schedule.school_year && schedule.school_term">
                    <el-icon><Calendar /></el-icon>
                    {{ schedule.school_year }}-{{ schedule.school_year + 1 }}学年第{{ schedule.school_term }}学期
                  </span>
                </div>
              </div>
              
              <div class="item-actions">
                <el-button
                  circle
                  text
                  @click.stop="handleExportSchedule(schedule)"
                  class="action-btn"
                  title="导出课表文件"
                >
                  <el-icon><Download /></el-icon>
                </el-button>
                <el-button
                  circle
                  text
                  @click.stop="handleEditSchedule(schedule)"
                  class="action-btn"
                  title="编辑设置"
                >
                  <el-icon><Edit /></el-icon>
                </el-button>
                <el-button
                  v-if="canUpdateSchedule(schedule)"
                  circle
                  text
                  type="primary"
                  @click.stop="handleUpdateSchedule(schedule)"
                  class="action-btn primary"
                  title="更新课表"
                  :loading="updatingScheduleId === schedule.id"
                >
                  <el-icon><Refresh /></el-icon>
                </el-button>
                <el-button
                  v-if="canUpdateSchedule(schedule)"
                  circle
                  text
                  type="warning"
                  @click.stop="handleImportExams(schedule)"
                  class="action-btn warning"
                  title="导入考试"
                  :loading="importingExamsScheduleId === schedule.id"
                >
                  <el-icon><DocumentChecked /></el-icon>
                </el-button>
                <el-button
                  circle
                  text
                  type="danger"
                  @click.stop="handleDeleteSchedule(schedule.id)"
                  class="action-btn danger"
                  title="删除课表"
                >
                  <el-icon><Delete /></el-icon>
                </el-button>
              </div>
            </div>
          </template>
        </VueDraggable>

        <div v-if="scheduleList.length === 0" class="empty-state">
            <el-icon :size="48" class="empty-icon"><Collection /></el-icon>
            <p>暂无课表数据</p>
        </div>
      </div>
      
      <div class="dialog-footer" style="display: flex; gap: 12px;">
          <el-button @click="handleImportScheduleFromFile" class="modern-button" style="flex: 1;">
            <el-icon style="margin-right: 6px;"><FolderOpened /></el-icon> 从文件导入
          </el-button>
          <el-button type="primary" class="modern-button primary" style="flex: 1.5;" @click="handleNewSchedule">
            <el-icon style="margin-right: 6px;"><Plus /></el-icon> 新建 / 在线导入
          </el-button>
      </div>
    </el-dialog>

    <ScheduleEditDialog
        v-model="showScheduleEditDialog"
        :schedule-id="editingScheduleMeta?.id"
        :initial-data="editingScheduleMeta"
        @saved="loadScheduleList"
    />

    <!-- 设置课表日期对话框 (Deprecated, replaced by ScheduleEditDialog)
    <el-dialog
      v-model="showScheduleDateDialog"
      title=""
      width="90%"
      align-center
      :show-close="false"
      class="custom-dialog"
      append-to-body
      style="max-width: 400px;"
    >
      <div class="dialog-header">
        <div class="dialog-title">设置日期</div>
        <div class="dialog-close-btn" @click="showScheduleDateDialog = false">
          <el-icon :size="20"><Close /></el-icon>
        </div>
      </div>

      <div class="dialog-content">
        <div class="setting-item">
          <div class="setting-label">课表名称</div>
          <el-input
            v-model="editingSchedule.name"
            disabled
            class="modern-input"
          >
            <template #prefix><el-icon><Collection /></el-icon></template>
          </el-input>
        </div>

        <div class="setting-item">
          <div class="setting-label">第一周起始日</div>
          <el-date-picker
            v-model="editingSchedule.first_day_date"
            type="date"
            placeholder="选择第一周周一的日期"
            format="YYYY/MM/DD"
            value-format="YYYY-MM-DD"
            style="width: 100%"
            class="modern-input"
            :clearable="false"
          />
          <div class="setting-desc">校历第一周的周一日期，用于计算当前周次</div>
        </div>
      </div>

      <div class="dialog-footer">
        <el-button @click="showScheduleDateDialog = false" class="modern-button">取消</el-button>
        <el-button type="primary" @click="_handleSaveScheduleDate" class="modern-button primary">
          保存设置
        </el-button>
      </div>
    </el-dialog>
    -->

    <!-- 主内容 -->
    <div v-if="!showImportDialog" class="main-content">
      <!-- 周次选择器 -->
      <WeekSelector
        :show="showWeekSelector"
        :week="currentWeek"
        :courses="courses"
        :max-weeks="currentSemesterWeeks"
        @update:week="handleWeekChange"
      />

      <!-- 星期栏 -->
      <div class="week-bar">
        <div class="week-bar-month">
          <span>{{ getDate(1).split('/')[0] }}</span>
          <span>月</span>
        </div>
        <div
          v-for="(day, index) in weekDays"
          :key="index"
          class="week-bar-item"
          :class="{ 'is-today': isToday(index + 1) }"
        >
          <span>{{ day }}</span>
          <span class="date">{{ getDate(index + 1) }}</span>
        </div>
      </div>

      <!-- 课表网格 -->
      <CourseGrid
        :courses="courses"
        :week="currentWeek"
        :end-week="currentSemesterWeeks"
        :colors="courseColors"
        :bg-image="backgroundImage"
        :max-periods="currentMaxPeriods"
        :period-times="currentPeriodTimes"
        :show-grid-lines="config.show_grid_lines"
        :card-opacity="config.card_opacity"
        :show-teacher="config.show_teacher"
        :show-location="config.show_location"
        :simplified-location="config.simplified_location"
        @update:week="handleWeekChange"
        @course-click="handleCourseClick"
      />

    <!-- 课程详情底部抽屉 -->
    <Transition name="slide-up">
      <div v-if="showDetailSheet && selectedCourse" class="detail-sheet-overlay" @click="showDetailSheet = false">
        <div class="detail-sheet" @click.stop>
          <div class="detail-handle"></div>
          
          <div class="detail-header">
            <div class="detail-name">{{ selectedCourse.name }}</div>
          </div>

          <div class="detail-content">
            <!-- Row 2: Weeks -->
            <div class="detail-row">
              <div class="detail-icon-box blue">
                <el-icon><Calendar /></el-icon>
              </div>
              <div class="detail-info">
                <span class="detail-label">周次</span>
                <span class="detail-value">{{ getWeeksText(selectedCourse.weeks) }}</span>
              </div>
            </div>

            <!-- Row 3: Time -->
            <div class="detail-row">
              <div class="detail-icon-box green">
                <el-icon><Timer /></el-icon>
              </div>
              <div class="detail-info">
                <span class="detail-label">时间</span>
                <span class="detail-value">{{ getCourseTimeText(selectedCourse) }}</span>
              </div>
            </div>

            <!-- Row 4: Teacher -->
            <div class="detail-row">
              <div class="detail-icon-box orange">
                <el-icon><User /></el-icon>
              </div>
              <div class="detail-info">
                <span class="detail-label">教师</span>
                <span class="detail-value">{{ selectedCourse.teacher || '未设置' }}</span>
              </div>
            </div>

            <!-- Row 5: Location -->
            <div class="detail-row">
              <div class="detail-icon-box purple">
                <el-icon><Location /></el-icon>
              </div>
              <div class="detail-info">
                <span class="detail-label">教室</span>
                <span class="detail-value">{{ selectedCourse.location || '未设置' }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </Transition>

    </div>

    <!-- 用户个人中心对话框 -->
    <UserProfileDialog
      v-model="showUserProfileDialog"
      :user-info="globalUserInfo"
      @login-success="handleLoginSuccess"
      @logout="handleLogout"
    />

    <!-- 更新提示对话框 -->
    <el-dialog
      v-model="showUpdateDialog"
      title="发现新版本"
      width="420px"
      :close-on-click-modal="false"
      class="update-dialog"
    >
      <div class="update-content">
        <div class="version-badge">v{{ pendingUpdate?.version }}</div>
        <h3>{{ pendingUpdate?.title }}</h3>
        <div class="release-notes">{{ pendingUpdate?.releaseNotes }}</div>
      </div>
      <template #footer>
        <div class="update-actions">
          <el-button @click="handleSkipVersion">此版本不再提醒</el-button>
          <el-button @click="showUpdateDialog = false">暂不更新</el-button>
          <el-button type="primary" @click="handleGoToDownload">去下载</el-button>
        </div>
      </template>
    </el-dialog>

    <!-- 主题切换按钮 -->
    <div class="theme-toggle" @click="toggleTheme" :title="isDark ? '切换亮色模式' : '切换深色模式'">
      <el-icon :size="20">
        <Moon v-if="!isDark" />
        <Sunny v-else />
      </el-icon>
    </div>
  </div>
  </el-config-provider>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue';
import { ElMessage, ElConfigProvider } from 'element-plus';
import zhCn from 'element-plus/es/locale/lang/zh-cn';
import { MoreFilled, ArrowDown, Loading, Plus, Picture, Delete, Close, Calendar, Collection, Sunny, Moon, Upload, Timer, User, Location, Edit, Check, Grid, View, Refresh, DocumentChecked, Download, FolderOpened } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { localDataDir } from '@tauri-apps/api/path';
import { open as openDialog, save as saveDialog } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';
import { readFile } from '@tauri-apps/plugin-fs';
import VueDraggable from 'vuedraggable';
import { useCourse } from './composables/useCourse';
import { useSchedule } from './composables/useSchedule';
import { useConfig } from './composables/useConfig';
import { useAuth } from './composables/useAuth';
import { useImportExport } from './composables/useImportExport';
import { useReminder } from './composables/useReminder';
import { calculateDate } from './utils/date';
import { getShuffledColors } from './utils/color';
import PopupMenu from './components/PopupMenu.vue';
import WeekSelector from './components/WeekSelector.vue';
import CourseGrid from './components/CourseGrid.vue';
import ScheduleEditDialog from './components/ScheduleEditDialog.vue';
import ImportScheduleDialog from './components/ImportScheduleDialog.vue';
import UserProfileDialog from './components/UserProfileDialog.vue';
import type { AppConfig, ScheduleMetadata, UserInfo, ScheduleDiff, Course, UpdateInfo, TimeTable } from './types';

// 从 composables 导入状态和方法
const { scheduleList, currentScheduleId, currentWeek, activeSchedule, loadScheduleList, deleteSchedule: deleteScheduleFn, switchSchedule: switchScheduleFn, reorderSchedules: reorderSchedulesFn } = useSchedule();
const { config, backgroundImage, checkForUpdate, skipVersion } = useConfig();
const {
  globalUserInfo: globalUserInfoRef,
  restoreLoginState,
  startSessionKeepAlive,
  stopSessionKeepAlive,
} = useAuth();
const { exportSchedule, importScheduleFromFile } = useImportExport();
const { startReminderService, stopReminderService, testNotification } = useReminder();

// 使用导入的全局状态
const globalUserInfo = globalUserInfoRef;

// UI 状态
const showPopup = ref(false);
const showWeekSelector = ref(false);
const showImportDialog = ref(false);
const showSettingsDialog = ref(false);
const reminderEnabled = ref(false); // 提醒功能开关
const reminderDebugLogging = ref(false); // 提醒调试日志开关

const showUpdateDialog = ref(false); // 更新提示对话框
const pendingUpdate = ref<UpdateInfo | null>(null); // 待处理的更新信息
const showAppearanceDialog = ref(false);
const showScheduleManageDialog = ref(false);
const showUserProfileDialog = ref(false); // 用户个人中心对话框
const isSorting = ref(false); // 排序模式状态
const showScheduleEditDialog = ref(false);
const editingScheduleMeta = ref<ScheduleMetadata | undefined>(undefined);
const loading = ref(false); // 加载状态

const courseColors = getShuffledColors();

// 课表数据
const { courses, loadCachedSchedule } = useCourse();

// 其他状态
const updatingScheduleId = ref<string | null>(null);

// 注意: activeSchedule 已从 useSchedule 导入

// Computed: 当前使用的时间表
const activeTimeTable = computed(() => {
    const tableId = activeSchedule.value?.time_table_id;
    if (tableId && config.value.time_tables) {
        return config.value.time_tables.find(t => t.id === tableId);
    }
    // Fallback: use default or first or config.period_times
    if (config.value.time_tables && config.value.time_tables.length > 0) {
        const def = config.value.time_tables.find(t => t.id === 'default');
        return def || config.value.time_tables[0];
    }
    return undefined;
});

// Computed: 当前最大节数
const currentMaxPeriods = computed(() => {
    // 优先使用课表设置，其次全局设置，最后默认13
    return activeSchedule.value?.max_periods || config.value.max_periods || 13;
});

// Computed: 当前学期周数
const currentSemesterWeeks = computed(() => {
    return activeSchedule.value?.weeks_count || config.value.end_week || 20;
});

// Computed: 当前节次时间表
const currentPeriodTimes = computed(() => {
  if (activeTimeTable.value) {
    return activeTimeTable.value.periods;
  }
  return config.value.period_times || [];
});

const reminderTimeTables = computed<TimeTable[]>(() => {
  if (activeTimeTable.value) return [activeTimeTable.value];
  if (config.value.time_tables && config.value.time_tables.length > 0) {
    return [config.value.time_tables[0]];
  }
  return [];
});

const weekDays = ['周一', '周二', '周三', '周四', '周五', '周六', '周日'];

// 重新计算当前周次
function recalculateCurrentWeek() {
  const startTimestamp = activeSchedule.value?.first_day || config.value.first_day;
  if (startTimestamp) {
    const week = Math.floor((Date.now() - startTimestamp * 1000) / (7 * 24 * 60 * 60 * 1000)) + 1;
    // 限制在合理范围内 (或根据 activeSchedule.weeks_count)
    const maxWeeks = currentSemesterWeeks.value || 20;
    currentWeek.value = Math.max(Math.min(week, maxWeeks), 1);
  }
}

// 计算学期状态
const semesterStatus = computed(() => {
  const startTimestamp = activeSchedule.value?.first_day || config.value.first_day;
  if (!startTimestamp) return null;

  const startTime = startTimestamp * 1000;
  const endTime = startTime + (currentSemesterWeeks.value || 20) * 7 * 24 * 60 * 60 * 1000;
  const now = Date.now();

  if (now < startTime) {
    return { type: 'not-started', text: '未开学' };
  } else if (now > endTime) {
    return { type: 'ended', text: '学期已结束' };
  }
  return null; // 学期进行中
});

// 监听 activeSchedule 或 config 变化，自动更新当前周次
watch(
  [() => activeSchedule.value?.first_day, () => config.value.first_day],
  () => {
    recalculateCurrentWeek();
  },
  { immediate: true }
);

// 获取日期
function getDate(day: number): string {
  // 优先使用当前课表的设置
  const startTimestamp = activeSchedule.value?.first_day || config.value.first_day;

  if (!startTimestamp) {
    return '--/--';
  }
  return calculateDate(startTimestamp, currentWeek.value, day);
}

// 判断某天是否是今天
function isToday(day: number): boolean {
  const startTimestamp = activeSchedule.value?.first_day || config.value.first_day;
  if (!startTimestamp) return false;

  // 计算指定星期几的日期
  const startDate = new Date(startTimestamp * 1000);
  const targetDate = new Date(startDate);
  targetDate.setDate(startDate.getDate() + (currentWeek.value - 1) * 7 + (day - 1));

  // 获取今天的日期（只比较年月日，不比较时分秒）
  const today = new Date();
  const todayDateOnly = new Date(today.getFullYear(), today.getMonth(), today.getDate());
  const targetDateOnly = new Date(targetDate.getFullYear(), targetDate.getMonth(), targetDate.getDate());

  return targetDateOnly.getTime() === todayDateOnly.getTime();
}

// 详情抽屉状态
const showDetailSheet = ref(false);
const selectedCourse = ref<Course | null>(null);

function handleCourseClick(course: Course) {
  selectedCourse.value = course;
  showDetailSheet.value = true;
}

function getWeeksText(weeks: number[]) {
  if (weeks.length === 0) return '无';
  
  // Sort and unique
  const sorted = [...new Set(weeks)].sort((a, b) => a - b);
  
  const ranges: string[] = [];
  let start = sorted[0];
  let prev = sorted[0];
  
  for (let i = 1; i < sorted.length; i++) {
    const current = sorted[i];
    if (current !== prev + 1) {
      // End of a range
      if (start === prev) {
         ranges.push(`${start}`);
      } else {
         ranges.push(`${start}-${prev}`);
      }
      start = current;
    }
    prev = current;
  }
  
  // Handle final range
  if (start === prev) {
    ranges.push(`${start}`);
  } else {
    ranges.push(`${start}-${prev}`);
  }
  
  return ranges.join(', ') + ' 周';
}

function getCourseTimeText(course: Course) {
  const startPeriod = course.periods[0];
  const endPeriod = course.periods[course.periods.length - 1];
  
  const startTime = getPeriodTime(startPeriod, true);
  const endTime = getPeriodTime(endPeriod, false);
  
  return `第 ${startPeriod}-${endPeriod} 节  ${startTime}-${endTime}`;
}

function getPeriodTime(period: number, isStart: boolean) {
  // Use saved times if available, else calc default
  if (currentPeriodTimes.value[period - 1]) {
    return isStart ? currentPeriodTimes.value[period - 1].start : currentPeriodTimes.value[period - 1].end;
  }
  // Default fallback calculation matches CourseGrid logic (simplified)
  const p = period;
  const startHour = 8 + Math.floor((p - 1) * 55 / 60);
  const startMin = ((p - 1) * 55) % 60;
  const endMin = (startMin + 45) % 60;
  const endHour = startHour + Math.floor((startMin + 45) / 60);
  
  if (isStart) 
    return `${startHour}:${startMin.toString().padStart(2, '0')}`;
  else
    return `${endHour}:${endMin.toString().padStart(2, '0')}`;
}


// 处理周次变化
function handleWeekChange(week: number) {
  currentWeek.value = week;
  showWeekSelector.value = false;
}

// 打开内置浏览器
async function handleScheduleManage() {
  showPopup.value = false;
  showScheduleManageDialog.value = true;
  await loadScheduleList();
}

// 导出课表
async function handleExportSchedule(schedule: ScheduleMetadata) {
  try {
    const filePath = await saveDialog({
      filters: [{
        name: 'JSON Schedule',
        extensions: ['json']
      }],
      defaultPath: `${schedule.name}.json`
    });

    if (!filePath) return;

    await exportSchedule(schedule);

    ElMessage.success('课表导出成功');
  } catch (e) {
    ElMessage.error(`导出失败: ${e}`);
  }
}

// 从文件导入课表
async function handleImportScheduleFromFile() {
  try {
    const scheduleId = await importScheduleFromFile();
    if (scheduleId) {
      ElMessage.success('课表导入成功');
      await loadScheduleList();
    }
  } catch (e) {
    ElMessage.error(`导入失败: ${e}`);
  }
}

// 切换课表
async function handleSwitchSchedule(scheduleId: string) {
  if (isSorting.value) return; // 排序模式下禁止切换
  try {
    await switchScheduleFn(scheduleId);
    await loadCachedSchedule(scheduleId);
    currentScheduleId.value = scheduleId;
    showScheduleManageDialog.value = false;
    ElMessage.success('切换成功');
    await loadConfig();

  } catch (e) {
    ElMessage.error(`切换失败: ${e}`);
  }
}

// 排序相关逻辑
function toggleSortMode() {
    if (isSorting.value) {
        // 保存排序
        saveScheduleOrder();
    }
    isSorting.value = !isSorting.value;
}

// 保存排序到后端
async function saveScheduleOrder() {
    try {
        const ids = scheduleList.value.map(s => s.id);
        await reorderSchedulesFn(ids);
        ElMessage.success('顺序已保存');
    } catch (e) {
        ElMessage.error(`保存顺序失败: ${e}`);
    }
}

// 删除课表
async function handleDeleteSchedule(scheduleId: string) {
  try {
    await deleteScheduleFn(scheduleId);
    ElMessage.success('删除成功');
    await loadScheduleList();
    // 如果删除的是当前课表,清空显示
    if (currentScheduleId.value === scheduleId) {
      courses.value = [];
    }
  } catch (e) {
    ElMessage.error(`删除失败: ${e}`);
  }
}

// 新建课表
async function handleNewSchedule() {
  showScheduleManageDialog.value = false;
  showPopup.value = false;
  showImportDialog.value = true;
}

// 打开编辑课表
function handleEditSchedule(schedule: ScheduleMetadata) {
    editingScheduleMeta.value = schedule;
    showScheduleEditDialog.value = true;
}

// 判断课表是否可以更新 (有学年学期信息)
function canUpdateSchedule(schedule: ScheduleMetadata): boolean {
  return schedule.school_year !== undefined && schedule.school_term !== undefined;
}

// 处理课表更新
async function handleUpdateSchedule(schedule: ScheduleMetadata) {
  updatingScheduleId.value = schedule.id;

  try {
    // 调用后端更新命令
    const diff = await invoke<ScheduleDiff>('update_schedule_with_diff', {
      scheduleId: schedule.id,
    });

    // 构建结果消息
    let message = `更新完成！\n未变化: ${diff.unchanged_count} 门课`;
    if (diff.added_count > 0) message += `\n➕ 新增: ${diff.added_count} 门课`;
    if (diff.modified_count > 0) message += `\n🔄 修改: ${diff.modified_count} 门课`;
    if (diff.removed_count > 0) message += `\n➖ 删除: ${diff.removed_count} 门课`;

    // 显示成功消息
    ElMessage.success({
      message,
      duration: 5000,
      showClose: true,
    });

    // 重新加载课表列表
    await loadScheduleList();

    // 如果更新的课表是当前课表，重新加载课程数据
    if (currentScheduleId.value === schedule.id) {
      await loadCachedSchedule(schedule.id);
    }
  } catch (e) {
    console.error('更新课表失败:', e);

    // 根据错误类型显示不同提示
    const errorMsg = String(e);
    if (errorMsg.includes('没有学年学期信息')) {
      ElMessage.error('该课表没有学年学期信息，无法更新。请删除后重新导入。');
    } else if (errorMsg.includes('未找到登录信息') || errorMsg.includes('请先登录')) {
      ElMessage.error('未登录，请先在个人中心登录');
    } else if (errorMsg.includes('登录已失效')) {
      ElMessage.error('登录已失效，请重新登录');
    } else {
      ElMessage.error(`更新失败: ${errorMsg}`);
    }
  } finally {
    updatingScheduleId.value = null;
  }
}

// 导入考试信息
const importingExamsScheduleId = ref<string | null>(null);

async function handleImportExams(schedule: ScheduleMetadata) {
  if (!schedule.school_year || !schedule.school_term) {
    ElMessage.warning('该课表没有学年学期信息，请先编辑课表填写学年学期');
    return;
  }

  if (!schedule.first_day) {
    ElMessage.warning('该课表没有设置学期开始日期，请先在课表编辑中填写「学期第一天」');
    return;
  }

  try {
    importingExamsScheduleId.value = schedule.id;

    const exams = await invoke<Course[]>('fetch_and_import_exams', {
      scheduleId: schedule.id
    });

    ElMessage.success(`成功导入 ${exams.length} 门考试`);

    // 如果导入的课表是当前课表，重新加载课程数据
    if (currentScheduleId.value === schedule.id) {
      await loadCachedSchedule(schedule.id);
    }
  } catch (e) {
    console.error('导入考试失败:', e);

    // 根据错误类型显示不同提示
    const errorMsg = String(e);
    if (errorMsg.includes('学年信息') || errorMsg.includes('学期信息')) {
      ElMessage.warning('请先在课表编辑中填写学年学期信息');
    } else if (errorMsg.includes('学期开始日期') || errorMsg.includes('学期第一天')) {
      ElMessage.warning('请先在课表编辑中填写「学期第一天」');
    } else if (errorMsg.includes('未查询到考试')) {
      ElMessage.info('未查询到考试安排');
    } else if (errorMsg.includes('请先登录')) {
      ElMessage.error('登录已失效，请重新登录');
    } else {
      ElMessage.error(`导入考试失败: ${errorMsg}`);
    }
  } finally {
    importingExamsScheduleId.value = null;
  }
}

// 确认导入课表
async function handleUploadBackground() {
  showPopup.value = false;
  
  try {
    const selected = await openDialog({
      multiple: false,
      filters: [{
        name: '图片',
        extensions: ['png', 'jpg', 'jpeg', 'gif', 'webp']
      }]
    });

    if (selected) {
      // 读取图片文件并转换为base64
      const fileData = await readFile(selected as string);
      const base64 = btoa(
        new Uint8Array(fileData).reduce((data, byte) => data + String.fromCharCode(byte), '')
      );
      
      // 获取文件扩展名
      const ext = (selected as string).split('.').pop()?.toLowerCase() || 'png';
      const mimeType = ext === 'jpg' ? 'image/jpeg' : `image/${ext}`;
      
      backgroundImage.value = `data:${mimeType};base64,${base64}`;
      
      // 保存配置
      const savedPath = await invoke<string>('save_background_image', { sourcePath: selected });
      config.value.background_image = savedPath;
      
      ElMessage.success('背景设置成功');
    }
  } catch (e) {
    console.error('选择背景失败:', e);
    ElMessage.error(`设置背景失败: ${e}`);
  }
}

// 处理删除背景
async function handleDeleteBackground() {
  showPopup.value = false;

  try {
    await invoke('delete_background_image');
    config.value.background_image = undefined;
    backgroundImage.value = '';
    ElMessage.success('背景已删除');
  } catch (e) {
    ElMessage.error(`删除失败: ${e}`);
  }
}

// 处理网格辅助线开关
async function handleToggleGridLines() {
  try {
    await invoke('save_app_config', { config: config.value });
  } catch (e) {
    ElMessage.error(`保存设置失败: ${e}`);
  }
}

// 处理课程卡片不透明度变化
async function handleCardOpacityChange() {
  try {
    await invoke('save_app_config', { config: config.value });
  } catch (e) {
    ElMessage.error(`保存设置失败: ${e}`);
  }
}

// 处理显示教师开关
async function handleToggleTeacher() {
  try {
    await invoke('save_app_config', { config: config.value });
  } catch (e) {
    ElMessage.error(`保存设置失败: ${e}`);
  }
}

// 处理显示地点开关
async function handleToggleLocation() {
  try {
    await invoke('save_app_config', { config: config.value });
  } catch (e) {
    ElMessage.error(`保存设置失败: ${e}`);
  }
}

// 处理简化地点显示开关
async function handleToggleSimplifiedLocation() {
  try {
    await invoke('save_app_config', { config: config.value });
  } catch (e) {
    ElMessage.error(`保存设置失败: ${e}`);
  }
}

async function handleImportSuccess(_scheduleId: string) {
  // 1. 仅重新加载列表，不自动切换
  await loadScheduleList();
  ElMessage.success('课表导入成功，请在课表管理中切换查看');
}

// 打开个人中心

// 登录成功处理
function handleLoginSuccess(userInfo: UserInfo) {
  globalUserInfo.value = userInfo;
}

// 退出登录处理
function handleLogout() {
  globalUserInfo.value = null;
}

// 保存设置
async function saveSettings() {
  try {
    // 更新提醒功能设置
    config.value.reminder_enabled = reminderEnabled.value;
    config.value.reminder_debug_logging = reminderDebugLogging.value;

    // 更新窗口关闭行为设置


    await invoke('save_app_config', { config: config.value });
    showSettingsDialog.value = false;
    ElMessage.success('设置保存成功');
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`);
  }
}

// 手动检查更新
async function handleManualCheckUpdate() {
  const update = await checkForUpdate(true);
  if (update) {
    pendingUpdate.value = update;
    showUpdateDialog.value = true;
  } else {
    ElMessage.info('当前已是最新版本');
  }
}

// 跳过此版本
async function handleSkipVersion() {
  if (pendingUpdate.value) {
    await skipVersion(pendingUpdate.value.version);
  }
  showUpdateDialog.value = false;
}

// 打开下载页面
async function handleGoToDownload() {
  if (pendingUpdate.value?.releaseUrl) {
    await openUrl(pendingUpdate.value.releaseUrl);
  }
  showUpdateDialog.value = false;
}

// 加载配置
async function loadConfig() {
  try {
    const appConfig = await invoke<AppConfig>('get_app_config');

    // 设置默认值并创建新对象
    config.value = {
      ...appConfig,
      card_opacity: appConfig.card_opacity ?? 95,
      show_grid_lines: appConfig.show_grid_lines ?? false,
      show_teacher: appConfig.show_teacher ?? true,
      show_location: appConfig.show_location ?? true,
      simplified_location: appConfig.simplified_location ?? false,
      reminder_enabled: appConfig.reminder_enabled ?? false,
      reminder_debug_logging: appConfig.reminder_debug_logging ?? false,
      auto_check_update: appConfig.auto_check_update ?? true,

    };

    // 读取窗口关闭行为设置


    // 读取提醒功能设置
    reminderEnabled.value = config.value.reminder_enabled ?? false;
    reminderDebugLogging.value = config.value.reminder_debug_logging ?? false;

    // 加载背景图
    if (config.value.background_image) {
      try {
        let imagePath = config.value.background_image;
        let fileData: Uint8Array;
        try {
          fileData = await readFile(imagePath);
        } catch {
          if (!imagePath.includes('/') && !imagePath.includes('\\')) {
            const fallbackPath = `${await localDataDir()}cufe-course/backgrounds/${imagePath}`;
            fileData = await readFile(fallbackPath);
            imagePath = fallbackPath;
            config.value.background_image = fallbackPath;
          } else {
            throw new Error('读取背景文件失败');
          }
        }
        const base64 = btoa(
          new Uint8Array(fileData).reduce((data, byte) => data + String.fromCharCode(byte), '')
        );
        const ext = imagePath.split('.').pop()?.toLowerCase() || 'png';
        const mimeType = ext === 'jpg' ? 'image/jpeg' : `image/${ext}`;
        backgroundImage.value = `data:${mimeType};base64,${base64}`;
      } catch (e) {
        console.error('加载背景图失败:', e);
      }
    }
  } catch (e) {
    console.error('加载配置失败:', e);
  }
}

// 初始化应用数据（异步）
async function initializeApp() {
  // 尝试加载缓存
  const hasCache = await loadCachedSchedule();

  if (hasCache) {
    await loadConfig();
  } else {
    // 如果没有缓存,显示导入对话框
    showImportDialog.value = true;
  }

  // 初始化深色模式
  const savedTheme = localStorage.getItem('theme');
  const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;

  if (savedTheme === 'dark' || (!savedTheme && systemDark)) {
    isDark.value = true;
    document.documentElement.classList.add('dark');
    document.documentElement.classList.remove('light');
  } else {
    isDark.value = false;
    document.documentElement.classList.remove('dark');
    document.documentElement.classList.add('light');
  }
}

// 初始化
onMounted(() => {
  // 异步恢复登录状态（不阻塞 UI）
  restoreLoginState();


  // 异步初始化应用数据（不阻塞 UI）
  initializeApp();

  // 自动检查更新（不阻塞 UI）
  setTimeout(async () => {
    if (config.value.auto_check_update) {
      const update = await checkForUpdate(false);
      if (update) {
        pendingUpdate.value = update;
        showUpdateDialog.value = true;
      }
    }
  }, 2000); // 延迟2秒检查，避免阻塞启动
});

onUnmounted(() => {
  stopSessionKeepAlive();
});

watch(
  globalUserInfo,
  (userInfo) => {
    if (userInfo) {
      startSessionKeepAlive();
      return;
    }

    stopSessionKeepAlive();
  },
  { immediate: true }
);

watch(
  [reminderEnabled, reminderDebugLogging, courses, reminderTimeTables, currentWeek],
  () => {
    const canRun =
      reminderEnabled.value &&
      courses.value.length > 0 &&
      reminderTimeTables.value.length > 0;

    if (!canRun) {
      stopReminderService();
      return;
    }

    if (!config.value.reminded_courses) {
      config.value.reminded_courses = {};
    }

    stopReminderService();
    startReminderService(
      courses.value,
      reminderTimeTables.value,
      currentWeek.value,
      (key: string) => {
        if (!config.value.reminded_courses) {
          config.value.reminded_courses = {};
        }
        config.value.reminded_courses[key] = Date.now();
      },
      config.value.reminded_courses,
      reminderDebugLogging.value
    );
  },
  { immediate: true, deep: true }
);

// 深色模式
const isDark = ref(false);

function toggleTheme() {
  isDark.value = !isDark.value;
  if (isDark.value) {
    document.documentElement.classList.add('dark');
    document.documentElement.classList.remove('light');
    localStorage.setItem('theme', 'dark');
  } else {
    document.documentElement.classList.add('light');
    document.documentElement.classList.remove('dark');
    localStorage.setItem('theme', 'light');
  }
}
</script>

<style>
:root {
    /* 现代极简配色 - 浅色模式 - 实体背景 */
    --primary-color: #6366f1;
    --primary-gradient: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    --bg-color: #f5f7fa;
    --surface-color-light: #ffffff; /* Dialog Background */
    --surface-color-strong: #ffffff;
    --card-bg: #f8fafc; /* Slightly darker than white for cards */
    --input-bg: #f1f5f9; /* Inputs */
    --surface-blur: none;
    --text-main: #1e293b;
    --text-secondary: #475569;
    --text-tertiary: #94a3b8;
    --border-color: #e2e8f0;
    --border-radius-base: 16px;
    --border-radius-lg: 20px;
    --border-radius-sm: 8px;
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.08), 0 2px 4px -1px rgba(0, 0, 0, 0.04); /* Stronger shadow */
    --shadow-lg: 0 10px 15px -3px rgba(0, 0, 0, 0.08), 0 4px 6px -2px rgba(0, 0, 0, 0.04);
    --shadow-active: 0 0 0 2px rgba(99, 102, 241, 0.2);
    
    --font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  }
  
  /* 深色模式 - 支持系统偏好和手动切换 */
  html.dark :root {
    /* 现代极简配色 - 深色模式 - 实体背景 */
    --primary-color: #818cf8;
    --primary-gradient: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    --bg-color: #0f172a;
    --surface-color-light: #1e293b; /* Slate 800 */
    --surface-color-strong: #1e293b;
    --card-bg: #283446; /* Slightly lighter than 800, distinctive */
    --input-bg: #0f172a; /* Darker for proper contrast */
    --text-main: #f1f5f9;
    --text-secondary: #cbd5e1;
    --text-tertiary: #94a3b8;
    --border-color: #334155;
    --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.4);
    --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.4);
  }
  
  @media (prefers-color-scheme: dark) {
    /* 默认跟随系统，如果未设置手动偏好 */
    :root:not([class*="light"]) {
      --primary-color: #818cf8;
      --primary-gradient: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
      --bg-color: #0f172a;
      --surface-color-light: #1e293b;
      --surface-color-strong: #1e293b;
      --card-bg: #283446;
      --input-bg: #0f172a;
      --text-main: #f1f5f9;
      --text-secondary: #cbd5e1;
      --text-tertiary: #94a3b8;
      --border-color: #334155;
      --shadow-sm: 0 1px 2px 0 rgba(0, 0, 0, 0.4);
      --shadow-md: 0 4px 6px -1px rgba(0, 0, 0, 0.4);
    }
  }

* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
  -webkit-tap-highlight-color: transparent;
}

body {
  font-family: var(--font-family);
  color: var(--text-main);
  background-color: var(--bg-color);
  -webkit-font-smoothing: antialiased;
}

#app {
  width: 100vw;
  height: 100vh;
  overflow: hidden;
  background-color: var(--bg-color);
  background-size: cover;
  background-position: center;
  transition: background-color 0.3s ease;
}

/* 主题切换按钮 */
.theme-toggle {
  position: fixed;
  bottom: 24px;
  right: 24px;
  width: 44px;
  height: 44px;
  border-radius: 50%;
  background-color: var(--surface-color-strong);
  backdrop-filter: var(--surface-blur);
  border: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  box-shadow: var(--shadow-lg);
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  z-index: 100;
  color: var(--text-secondary);
}

.theme-toggle:hover {
  transform: translateY(-2px) rotate(15deg);
  background-color: var(--surface-color-strong);
  color: var(--primary-color);
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.25);
}

.theme-toggle:active {
  transform: scale(0.92);
}

html.dark .theme-toggle {
  color: #ffd700; /* Gold for sun */
}

/* 导航栏 - 浮动极简风格 */
.navbar {
  height: calc(64px + env(safe-area-inset-top));
  padding-top: env(safe-area-inset-top);
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  background-color: transparent; /* Remove solid background */
  z-index: 50;
}

.navbar-left {
  position: absolute;
  left: 24px;
  top: calc(32px + env(safe-area-inset-top));
  transform: translateY(-50%);
  cursor: pointer;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--surface-color-light);
  backdrop-filter: var(--surface-blur);
  border: 1px solid var(--border-color);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  color: var(--text-main);
}

.navbar-left:hover {
  transform: translateY(-1px);
  background-color: var(--surface-color-strong);
  box-shadow: var(--shadow-md);
}

.navbar-left:active {
  transform: scale(0.95);
}

.navbar-center {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  font-size: 16px;
  font-weight: 600;
  padding: 8px 16px;
  border-radius: 20px;
  background-color: var(--surface-color-light);
  backdrop-filter: var(--surface-blur);
  border: 1px solid var(--border-color);
  transition: all 0.2s;
  color: var(--text-main);
}

.navbar-center:hover {
  background-color: var(--surface-color-strong);
  box-shadow: var(--shadow-md);
}

.navbar-right {
  position: absolute;
  right: 24px;
  top: calc(32px + env(safe-area-inset-top));
  transform: translateY(-50%);
  display: flex;
  align-items: center;
}

.semester-status {
  font-size: 13px;
  font-weight: 500;
  padding: 6px 12px;
  border-radius: 16px;
  backdrop-filter: var(--surface-blur);
  border: 1px solid var(--border-color);
  transition: all 0.2s;
}

.semester-status.not-started {
  background-color: rgba(103, 194, 58, 0.15);
  color: #67c23a;
  border-color: rgba(103, 194, 58, 0.3);
}

.semester-status.ended {
  background-color: rgba(245, 108, 108, 0.15);
  color: #f56c6c;
  border-color: rgba(245, 108, 108, 0.3);
}

/* 加载状态 */
.loading {
  position: fixed;
  bottom: 30px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 24px;
  background-color: var(--surface-color-strong);
  backdrop-filter: var(--surface-blur);
  border-radius: 30px;
  box-shadow: var(--shadow-lg);
  border: 1px solid var(--border-color);
  z-index: 1000;
  font-size: 14px;
  font-weight: 500;
  color: var(--text-main);
}

/* 主内容 */
.main-content {
  height: calc(100vh - 64px - env(safe-area-inset-top));
  display: flex;
  flex-direction: column;
  overflow: hidden;
  padding: 0 16px calc(16px + env(safe-area-inset-bottom));
}

/* 星期栏 */
.week-bar {
  display: flex;
  height: 44px;
  margin-bottom: 8px;
  background-color: transparent;
  border: none;
  flex-shrink: 0;
  padding: 0 8px; /* Align with content */
}

.week-bar-month {
  width: 6%;
  min-width: 45px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-main);
  opacity: 0.8;
}

.week-bar-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: 13px;
  color: var(--text-secondary);
}

.week-bar-item .date {
  color: var(--text-tertiary);
  font-size: 11px;
  margin-top: 2px;
  font-weight: 500;
}

/* 今天日期强调显示 */
.week-bar-item.is-today {
  background-color: rgba(64, 158, 255, 0.12);
  border-radius: 12px;
  font-weight: 600;
  color: var(--primary-color);
  transition: all 0.3s ease;
}

.week-bar-item.is-today .date {
  color: var(--primary-color);
  font-weight: 700;
  font-size: 12px;
}

/* 统一对话框样式 */
.custom-dialog .el-dialog {
  border-radius: 24px !important;
  background-color: var(--bg-color) !important;
  box-shadow: var(--shadow-lg) !important;
  overflow: hidden;
  border: 1px solid var(--border-color) !important;
}

.custom-dialog .el-dialog__header {
  display: none;
}

.custom-dialog .el-dialog__body {
  padding: 0 !important;
  background-color: transparent !important;
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 24px 28px;
  background-color:transparent;
}

.dialog-title {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-main);
  letter-spacing: -0.5px;
}

.dialog-close {
  cursor: pointer;
  padding: 8px;
  border-radius: 50%;
  transition: all 0.2s;
  color: var(--text-tertiary);
  background-color: rgba(0,0,0,0.03);
}

.dialog-close:hover {
  background-color: rgba(0,0,0,0.08);
  color: var(--text-main);
  transform: rotate(90deg);
}

.dialog-content {
  padding: 0 28px 28px;
  max-height: 60vh;
  overflow-y: auto;
  background-color: transparent;
}

.dialog-footer {
  padding: 20px 28px;
  border-top: 1px solid var(--border-color);
  background-color: var(--surface-color-light);
}

/* 设置表单样式 */
.settings-section {
  padding: 16px;
  background: var(--surface-color-light);
  border-radius: 12px;
  margin-bottom: 16px;
}

.section-title {
  font-size: 15px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 16px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 0;
}

.setting-item span {
  font-size: 14px;
  color: var(--text-main);
}

.setting-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 8px;
  line-height: 1.6;
}

/* 更新对话框样式 */
.update-content {
  padding: 0 4px;
}

.version-badge {
  display: inline-block;
  padding: 4px 12px;
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  color: white;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
}

.update-content h3 {
  margin: 0 0 16px 0;
  font-size: 18px;
  color: var(--text-main);
}

.release-notes {
  max-height: 200px;
  overflow-y: auto;
  padding: 12px;
  background: var(--surface-color-light);
  border-radius: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.6;
  white-space: pre-wrap;
}

.update-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.close-action-radio {
  margin-top: 8px;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.test-button {
  margin-top: 12px;
  padding: 12px;
  background: var(--surface-color-strong);
  border-radius: 8px;
  justify-content: flex-start;
}

.test-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin-left: 8px;
}

.setting-group {
  margin-bottom: 24px;
}

.setting-label {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 12px;
  display: block;
}

.custom-input .el-input__wrapper {
  background-color: var(--surface-color-light) !important;
  box-shadow: none !important;
  border: 1px solid transparent !important;
  border-radius: 12px;
  padding: 10px 15px;
  transition: all 0.2s;
}

.custom-input .el-input__wrapper:hover {
  background-color: var(--surface-color-strong) !important;
}

.custom-input .el-input__wrapper.is-focus {
  background-color: #fff !important;
  box-shadow: 0 0 0 2px var(--primary-color) !important;
}

.custom-input .el-input__inner {
  color: var(--text-main) !important;
  height: auto;
}

/* 输入框深色模式适配 */
html.dark .custom-input .el-input__wrapper.is-focus {
  background-color: rgba(255,255,255,0.1) !important;
}

/* 数字输入框适配 */
.custom-number .el-input-number__decrease,
.custom-number .el-input-number__increase {
  background-color: var(--surface-color-light) !important;
  border: none !important;
  color: var(--text-secondary) !important;
}

.custom-number .el-input__wrapper {
  background-color: var(--surface-color-light) !important;
  box-shadow: none !important;
  border-radius: 12px;
}

/* 时间列表 */
.time-list {
  background-color: var(--surface-color-light);
  border-radius: 16px;
  padding: 16px;
  border: 1px solid var(--border-color);
}

.time-item {
  border-bottom: 1px solid var(--border-color);
  padding: 10px 0;
}

/* 按钮通用样式 */
.custom-button {
  border-radius: 12px;
  padding: 12px 24px;
  font-weight: 600;
  transition: all 0.2s;
  height: auto;
}

.primary-button {
  background: var(--primary-gradient);
  border: none;
  box-shadow: 0 4px 12px rgba(99, 102, 241, 0.3);
}

.primary-button:hover {
  transform: translateY(-1px);
  box-shadow: 0 6px 16px rgba(99, 102, 241, 0.4);
}

.primary-button:active {
  transform: scale(0.98);
}

/* 列表卡片样式 (Appearance & Schedule) */
.appearance-cards {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.appearance-card {
  padding: 24px;
  background-color: var(--surface-color-light);
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.2s;
  text-align: center;
  border: 1px solid var(--border-color);
}

.appearance-card:hover {
  background-color: var(--surface-color-strong);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
  border-color: var(--primary-color);
}

.schedule-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.schedule-card {
  padding: 20px;
  background-color: var(--surface-color-light);
  border-radius: 20px;
  border: 1px solid var(--border-color);
  transition: all 0.2s;
}

.schedule-card:hover {
  background-color: var(--surface-color-strong);
  transform: translateY(-1px);
  box-shadow: var(--shadow-sm);
}

.schedule-card.active {
  background-color: var(--surface-color-strong);
  border-color: var(--primary-color);
  box-shadow: 0 0 0 1px var(--primary-color);
}

.schedule-name {
  color: var(--text-main);
  font-weight: 600;
  font-size: 16px;
}

.meta-item {
  color: var(--text-secondary);
}

/* Empty State */
.empty-text { color: var(--text-secondary); }
.empty-hint { color: var(--text-tertiary); }



/* --- Unified Modern Dialog Styles (v3.1 - Fixed Selectors) --- */

/* The .custom-dialog CLASS is applied to the .el-dialog ELEMENT itself */
.custom-dialog {
  background-color: transparent !important;
  box-shadow: none !important;
  margin: 0 !important; /* Managed by align-center */
  width: 90vw !important; /* Forced visual width relative to viewport */
  max-width: 480px;
  min-width: 300px !important;
  max-height: 90vh !important;
  display: flex !important;
  flex-direction: column !important;
  position: relative !important;
  transform: none !important;
  /* Reset positioning overrides as align-center handles it, but ensure no weird offsets */
  left: auto !important;
  top: auto !important;
  --el-dialog-bg-color: transparent !important;
  border-radius: var(--border-radius-lg) !important;
}

/* Ensure no child forces width beyond 100% */
.custom-dialog .el-dialog__body {
  padding: 0 !important;
  background-color: var(--surface-color-strong) !important;
  border-radius: var(--border-radius-lg);
  box-shadow: var(--shadow-lg);
  border: 1px solid var(--border-color);
  color: var(--text-main);
  display: flex !important;
  flex-direction: column !important;
  flex: 1 !important;
  overflow: hidden !important;
  opacity: 1 !important;
  height: auto !important;
  width: 100% !important; /* Ensure body fits dialog */
  box-sizing: border-box;
}

/* Header - Fixed */
.dialog-header {
  padding: 16px 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid var(--border-color);
  background-color: var(--surface-color-light); /* Distinct header bg */
  flex-shrink: 0;
  z-index: 10;
}

.dialog-title {
  font-size: 17px;
  font-weight: 700;
  color: var(--text-main);
}

.dialog-close-btn {
  width: 32px;
  height: 32px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: var(--text-tertiary);
  transition: all 0.2s;
}

.dialog-close-btn:hover {
  background-color: rgba(0,0,0,0.05);
  color: var(--text-main);
}

/* Content - Scrollable */
.dialog-content {
  padding: 20px;
  overflow-y: auto !important; /* Enable scrolling */
  overflow-x: hidden;
  flex: 1; /* Take remaining space */
  -webkit-overflow-scrolling: touch;
}

/* Footer - Fixed */
.dialog-footer {
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background-color: var(--surface-color-light);
  flex-shrink: 0;
  z-index: 10;
}

/* Scrollbar Styling */
.dialog-content::-webkit-scrollbar {
  width: 6px;
}
.dialog-content::-webkit-scrollbar-track {
  background: transparent;
}
.dialog-content::-webkit-scrollbar-thumb {
  background-color: var(--border-color);
  border-radius: 3px;
}
.dialog-content::-webkit-scrollbar-thumb:hover {
  background-color: var(--text-tertiary);
}

.no-scrollbar::-webkit-scrollbar {
  display: none;
}
.no-scrollbar {
  scrollbar-width: none;
}

/* --- Appearance Grid --- */
.appearance-grid {
  display: grid;
  grid-template-columns: 1fr; /* Responsive: start with 1 col on very small screens */
  gap: 16px;
}

/* Tablet+ override */
@media (min-width: 400px) {
  .appearance-grid {
    grid-template-columns: repeat(2, 1fr);
  }
}


.appearance-item {
  background-color: var(--card-bg);
  border-radius: 16px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid var(--border-color);
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: var(--shadow-sm);
}

.appearance-item:hover {
  transform: translateY(-3px);
  border-color: var(--primary-color);
  box-shadow: var(--shadow-md);
}

.item-preview {
  height: 100px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: rgba(99, 102, 241, 0.05);
  position: relative;
  overflow: hidden;
  color: var(--primary-color);
}

.item-preview.delete-preview {
  background-color: rgba(239, 68, 68, 0.05);
  color: #ef4444;
}

.hover-overlay {
  position: absolute;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: rgba(0,0,0,0.4);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: white;
  opacity: 0;
  transition: opacity 0.2s;
  gap: 6px;
  font-size: 13px;
  font-weight: 500;
  backdrop-filter: blur(2px);
}

.appearance-item:hover .hover-overlay {
  opacity: 1;
}

.item-info {
  padding: 12px;
}

.item-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-main);
  margin-bottom: 4px;
}

.item-desc {
  font-size: 11px;
  color: var(--text-tertiary);
}

/* 开关控件项 */
.switch-item .item-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

/* 滑块控件项 */
.slider-item .item-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.item-control {
  display: flex;
  justify-content: flex-end;
  align-items: center;
  padding-top: 4px;
}

.slider-item .item-control {
  padding: 8px 0 4px 0;
}

/* --- Schedule List (Modern) --- */
.schedule-list-container {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-height: 200px;
}

.schedule-list-item {
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

.schedule-list-item:hover {
  background-color: var(--surface-color-strong);
  transform: translateX(4px);
  box-shadow: var(--shadow-sm);
}

.schedule-list-item.is-active {
  background: var(--surface-color-strong);
  border-color: var(--primary-color);
  box-shadow: var(--shadow-active);
}

.item-main {
  flex: 1;
  min-width: 0; /* Crucial for flex content truncation */
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
  flex-shrink: 0; /* Tag shouldn't shrink */
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

.item-actions {
  display: flex;
  gap: 4px; /* Tighter gap */
  opacity: 1; /* Always visible for mobile friendliness or handle via hover on desktop */
  flex-shrink: 0;
}

/* --- Detail Sheet Styles --- */
.detail-sheet-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.4);
  z-index: 1000;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
}

.detail-sheet {
  background-color: var(--card-bg); /* Use card bg for theme adaptation */
  border-top-left-radius: 20px;
  border-top-right-radius: 20px;
  padding: 10px 24px 40px 24px;
  box-shadow: 0 -4px 20px rgba(0, 0, 0, 0.15);
  position: relative;
  border-top: 1px solid rgba(255,255,255,0.1);
}

.detail-handle {
  width: 40px;
  height: 4px;
  background-color: rgba(100, 116, 139, 0.2);
  border-radius: 2px;
  margin: 10px auto 20px auto;
}

.detail-header {
  margin-bottom: 24px;
}

.detail-name {
  font-size: 20px;
  font-weight: 700;
  color: var(--text-main);
  line-height: 1.4;
}

.detail-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.detail-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.detail-icon-box {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 22px;
  flex-shrink: 0;
}

/* Icon Box Colors */
.detail-icon-box.blue { background-color: rgba(59, 130, 246, 0.1); color: #3b82f6; }
.detail-icon-box.green { background-color: rgba(16, 185, 129, 0.1); color: #10b981; }
.detail-icon-box.orange { background-color: rgba(249, 115, 22, 0.1); color: #f97316; }
.detail-icon-box.purple { background-color: rgba(139, 92, 246, 0.1); color: #8b5cf6; }

/* Dark mode adjustments for icon boxes handled by opacity mostly, but let's ensure visibility */
html.dark .detail-icon-box.blue { background-color: rgba(59, 130, 246, 0.2); }
html.dark .detail-icon-box.green { background-color: rgba(16, 185, 129, 0.2); }
html.dark .detail-icon-box.orange { background-color: rgba(249, 115, 22, 0.2); }
html.dark .detail-icon-box.purple { background-color: rgba(139, 92, 246, 0.2); }


.detail-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.detail-label {
  font-size: 11px;
  color: var(--text-tertiary);
  font-weight: 500;
}

.detail-value {
  font-size: 15px;
  color: var(--text-main);
  font-weight: 600;
}

/* Animations */
.slide-up-enter-active,
.slide-up-leave-active {
  transition: opacity 0.3s ease;
}

.slide-up-enter-active .detail-sheet,
.slide-up-leave-active .detail-sheet {
  transition: transform 0.3s cubic-bezier(0.16, 1, 0.3, 1);
}

.slide-up-enter-from,
.slide-up-leave-to {
  opacity: 0;
}

.slide-up-enter-from .detail-sheet,
.slide-up-leave-to .detail-sheet {
  transform: translateY(100%);
}



/* On desktop, we can hide actions until hover if preferred, but for now let's keep them accessible or check previous logic */
@media (hover: hover) {
  .item-actions {
    opacity: 0;
    transform: translateX(10px);
    transition: all 0.2s;
  }
  .schedule-list-item:hover .item-actions {
    opacity: 1;
    transform: translateX(0);
  }
}

.action-btn {
  width: 32px;
  height: 32px;
  font-size: 16px;
  color: var(--text-secondary);
  border-radius: 8px; /* Softer shape */
}

.action-btn:hover {
  background-color: rgba(99, 102, 241, 0.1);
  color: var(--primary-color);
}

.action-btn.danger:hover {
  background-color: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 40px 0;
  color: var(--text-tertiary);
  gap: 12px;
}

/* Modern Input */
.modern-input .el-input__wrapper,
.modern-number-input .el-input__wrapper {
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

/* Appearance Items - Cards */
.appearance-item {
  background-color: var(--card-bg);
  border-radius: 16px;
  overflow: hidden;
  cursor: pointer;
  border: 1px solid var(--border-color); /* Stronger border */
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  box-shadow: var(--shadow-sm); /* Add shadow by default */
}

/* Schedule List Items */
.schedule-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 14px 18px;
  background-color: var(--card-bg);
  border-radius: 14px;
  border: 1px solid var(--border-color); /* Stronger border */
  transition: all 0.2s;
  cursor: pointer;
  box-shadow: var(--shadow-sm); /* Add shadow */
  margin-bottom: 8px;
}

.schedule-list-item:hover {
  transform: translateY(-2px); /* Slight lift */
  background-color: var(--card-bg); /* Keep base color */
  border-color: var(--primary-color); /* Highlight border */
  box-shadow: var(--shadow-md);
}

.schedule-list-item.is-active {
  background: var(--card-bg);
  border-color: var(--primary-color);
  box-shadow: 0 0 0 1px var(--primary-color), var(--shadow-sm);
}

/* VueDraggable Styles */
.sortable-ghost {
  opacity: 0.4;
  background: var(--surface-color-strong);
  border: 1px dashed var(--primary-color);
}

.sortable-drag {
  opacity: 1 !important;
  background: var(--card-bg);
  box-shadow: var(--shadow-xl);
  transform: scale(1.02);
}

.schedule-list-item.is-sorting {
  cursor: grab;
}

.schedule-list-item.is-sorting:active {
  cursor: grabbing;
}

.schedule-list-item.is-sorting .item-actions {
  display: none !important;
}

/* Time Inputs */
.time-mini-input .el-input__wrapper {
  background-color: var(--input-bg) !important;
}

/* Dark Mode cleanup - most handled by vars now, just ensuring specifics */
html.dark .modern-input .el-input__wrapper {
  background-color: var(--input-bg) !important;
}

html.dark .appearance-item,
html.dark .schedule-list-item {
  background-color: var(--card-bg);
}

html.dark .schedule-list-item:hover {
  background-color: var(--card-bg); /* Avoid color shift, rely on border/shadow */
  border-color: var(--primary-color);
}
</style>
