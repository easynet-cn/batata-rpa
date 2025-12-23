<template>
  <div class="recorder-panel">
    <div class="recorder-header">
      <span class="title">
        <el-icon><VideoCamera /></el-icon>
        流程录制
      </span>
      <el-tag
        :type="statusType"
        size="small"
        effect="dark"
      >
        {{ statusText }}
      </el-tag>
    </div>

    <div class="recorder-content">
      <!-- Recording name input (when idle) -->
      <div v-if="recorder.isIdle" class="recording-setup">
        <el-input
          v-model="recordingName"
          placeholder="录制名称"
          size="small"
          clearable
        />
        <p class="hint">
          点击开始录制后，您的鼠标点击、键盘输入等操作将被自动记录并转换为工作流节点。
        </p>
      </div>

      <!-- Recording status (when recording or paused) -->
      <div v-else class="recording-status">
        <div class="status-item">
          <span class="label">录制时间</span>
          <span class="value timer">{{ recorder.formattedDuration }}</span>
        </div>
        <div class="status-item">
          <span class="label">已录制操作</span>
          <span class="value">{{ recorder.actionCount }}</span>
        </div>
      </div>

      <!-- Action list preview -->
      <div v-if="!recorder.isIdle && recorder.session?.actions?.length" class="action-list">
        <div class="action-list-header">
          <span>最近操作</span>
          <el-button text size="small" @click="showAllActions = !showAllActions">
            {{ showAllActions ? '收起' : '展开' }}
          </el-button>
        </div>
        <el-scrollbar max-height="200px">
          <div
            v-for="action in displayedActions"
            :key="action.id"
            class="action-item"
          >
            <el-icon :class="getActionIconClass(action.action_type)">
              <component :is="getActionIcon(action.action_type)" />
            </el-icon>
            <span class="action-text">{{ formatAction(action) }}</span>
            <span class="action-time">{{ formatTime(action.timestamp) }}</span>
          </div>
        </el-scrollbar>
      </div>
    </div>

    <div class="recorder-footer">
      <template v-if="recorder.isIdle">
        <el-button
          type="primary"
          size="small"
          @click="startRecording"
        >
          <el-icon><VideoPlay /></el-icon>
          开始录制
        </el-button>
      </template>

      <template v-else-if="recorder.isRecording">
        <el-button
          type="warning"
          size="small"
          @click="pauseRecording"
        >
          <el-icon><VideoPause /></el-icon>
          暂停
        </el-button>
        <el-button
          type="danger"
          size="small"
          @click="stopRecording"
        >
          <el-icon><CircleClose /></el-icon>
          停止
        </el-button>
      </template>

      <template v-else-if="recorder.isPaused">
        <el-button
          type="primary"
          size="small"
          @click="resumeRecording"
        >
          <el-icon><VideoPlay /></el-icon>
          继续
        </el-button>
        <el-button
          type="danger"
          size="small"
          @click="stopRecording"
        >
          <el-icon><CircleClose /></el-icon>
          停止
        </el-button>
      </template>
    </div>

    <!-- Convert dialog -->
    <el-dialog
      v-model="showConvertDialog"
      title="录制完成"
      width="400px"
      :close-on-click-modal="false"
    >
      <div class="convert-dialog-content">
        <p>录制已完成，共 {{ recorder.session?.actions?.length || 0 }} 个操作。</p>
        <p>是否将录制结果转换为工作流？</p>
      </div>
      <template #footer>
        <el-button @click="discardRecording">放弃</el-button>
        <el-button type="primary" @click="convertRecording">
          转换为工作流
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRecorderStore, type RecordedAction } from '@/stores/recorder';
import { useWorkflowStore } from '@/stores/workflow';
import { ElMessage } from 'element-plus';
import {
  VideoCamera,
  VideoPlay,
  VideoPause,
  CircleClose,
  Pointer,
  Edit,
  Mouse,
  Key,
  Timer,
} from '@element-plus/icons-vue';

const recorder = useRecorderStore();
const workflowStore = useWorkflowStore();

const recordingName = ref('新录制');
const showAllActions = ref(false);
const showConvertDialog = ref(false);

const statusType = computed(() => {
  if (recorder.isRecording) return 'danger';
  if (recorder.isPaused) return 'warning';
  return 'info';
});

const statusText = computed(() => {
  if (recorder.isRecording) return '录制中';
  if (recorder.isPaused) return '已暂停';
  return '未开始';
});

const displayedActions = computed(() => {
  const actions = recorder.session?.actions || [];
  if (showAllActions.value) {
    return actions;
  }
  return actions.slice(-5);
});

function getActionIcon(actionType: string) {
  switch (actionType) {
    case 'Click':
    case 'DoubleClick':
    case 'RightClick':
      return Pointer;
    case 'Input':
      return Edit;
    case 'Scroll':
      return Mouse;
    case 'Hotkey':
      return Key;
    case 'Wait':
      return Timer;
    default:
      return Pointer;
  }
}

function getActionIconClass(actionType: string) {
  switch (actionType) {
    case 'Click':
      return 'icon-click';
    case 'DoubleClick':
      return 'icon-double-click';
    case 'RightClick':
      return 'icon-right-click';
    case 'Input':
      return 'icon-input';
    case 'Scroll':
      return 'icon-scroll';
    case 'Hotkey':
      return 'icon-hotkey';
    default:
      return '';
  }
}

function formatAction(action: RecordedAction): string {
  const type = action.action_type;
  const element = action.element;
  const pos = action.position;

  switch (type) {
    case 'Click':
      if (element) {
        return `点击 "${element.name}"`;
      }
      if (pos) {
        return `点击 (${pos[0]}, ${pos[1]})`;
      }
      return '点击';
    case 'DoubleClick':
      if (element) {
        return `双击 "${element.name}"`;
      }
      return '双击';
    case 'RightClick':
      if (element) {
        return `右键点击 "${element.name}"`;
      }
      return '右键点击';
    case 'Input':
      const text = action.data?.text || '';
      return `输入 "${text.slice(0, 20)}${text.length > 20 ? '...' : ''}"`;
    case 'Scroll':
      return `滚动 (${action.data?.deltaX || 0}, ${action.data?.deltaY || 0})`;
    case 'Hotkey':
      const modifiers = action.data?.modifiers || [];
      const key = action.data?.key || '';
      return `按键 ${modifiers.join('+')}${modifiers.length ? '+' : ''}${key}`;
    case 'Wait':
      return `等待 ${action.data?.delay || 0}ms`;
    default:
      return type;
  }
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  });
}

async function startRecording() {
  try {
    await recorder.startRecording(recordingName.value);
    ElMessage.success('录制已开始');
  } catch (e) {
    ElMessage.error(`启动录制失败: ${e}`);
  }
}

async function pauseRecording() {
  try {
    await recorder.pauseRecording();
    ElMessage.info('录制已暂停');
  } catch (e) {
    ElMessage.error(`暂停录制失败: ${e}`);
  }
}

async function resumeRecording() {
  try {
    await recorder.resumeRecording();
    ElMessage.success('录制已继续');
  } catch (e) {
    ElMessage.error(`继续录制失败: ${e}`);
  }
}

async function stopRecording() {
  try {
    await recorder.stopRecording();
    await recorder.fetchSession();
    showConvertDialog.value = true;
  } catch (e) {
    ElMessage.error(`停止录制失败: ${e}`);
  }
}

async function convertRecording() {
  try {
    const workflow = await recorder.convertToWorkflow();
    if (workflow) {
      // Load the workflow into the designer
      workflowStore.loadFromJson(JSON.stringify(workflow));
      ElMessage.success('录制已转换为工作流');
    }
    showConvertDialog.value = false;
    await recorder.clearRecording();
  } catch (e) {
    ElMessage.error(`转换失败: ${e}`);
  }
}

async function discardRecording() {
  showConvertDialog.value = false;
  await recorder.clearRecording();
  ElMessage.info('录制已放弃');
}
</script>

<style scoped>
.recorder-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--el-bg-color);
  border-radius: 4px;
}

.recorder-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-light);
}

.recorder-header .title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.recorder-content {
  flex: 1;
  padding: 16px;
  overflow-y: auto;
}

.recording-setup {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.recording-setup .hint {
  font-size: 12px;
  color: var(--el-text-color-secondary);
  line-height: 1.6;
  margin: 0;
}

.recording-status {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.status-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.status-item .label {
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.status-item .value {
  font-size: 14px;
  font-weight: 500;
  color: var(--el-text-color-primary);
}

.status-item .value.timer {
  font-family: monospace;
  font-size: 18px;
  color: var(--el-color-danger);
}

.action-list {
  margin-top: 16px;
  border: 1px solid var(--el-border-color-light);
  border-radius: 4px;
}

.action-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: var(--el-fill-color-light);
  border-bottom: 1px solid var(--el-border-color-light);
  font-size: 13px;
  color: var(--el-text-color-secondary);
}

.action-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.action-item:last-child {
  border-bottom: none;
}

.action-item .el-icon {
  font-size: 14px;
  color: var(--el-color-primary);
}

.action-item .icon-click {
  color: var(--el-color-primary);
}

.action-item .icon-double-click {
  color: var(--el-color-warning);
}

.action-item .icon-right-click {
  color: var(--el-color-success);
}

.action-item .icon-input {
  color: var(--el-color-info);
}

.action-item .icon-hotkey {
  color: var(--el-color-danger);
}

.action-text {
  flex: 1;
  font-size: 12px;
  color: var(--el-text-color-primary);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.action-time {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  font-family: monospace;
}

.recorder-footer {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--el-border-color-light);
}

.recorder-footer .el-button {
  flex: 1;
}

.convert-dialog-content {
  text-align: center;
}

.convert-dialog-content p {
  margin: 8px 0;
  color: var(--el-text-color-regular);
}
</style>
