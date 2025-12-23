<template>
  <div class="recorder-panel">
    <div class="recorder-header">
      <span class="title">
        <Video :size="16" />
        流程录制
      </span>
      <span class="tag" :class="statusClass">
        {{ statusText }}
      </span>
    </div>

    <div class="recorder-content">
      <!-- Recording name input (when idle) -->
      <div v-if="recorder.isIdle" class="recording-setup">
        <input
          v-model="recordingName"
          class="input"
          placeholder="录制名称"
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
          <button class="btn-text" @click="showAllActions = !showAllActions">
            {{ showAllActions ? '收起' : '展开' }}
          </button>
        </div>
        <div class="action-list-body">
          <div
            v-for="action in displayedActions"
            :key="action.id"
            class="action-item"
          >
            <component :is="getActionIcon(action.action_type)" :size="14" :class="getActionIconClass(action.action_type)" />
            <span class="action-text">{{ formatAction(action) }}</span>
            <span class="action-time">{{ formatTime(action.timestamp) }}</span>
          </div>
        </div>
      </div>
    </div>

    <div class="recorder-footer">
      <template v-if="recorder.isIdle">
        <button class="btn btn-primary" @click="startRecording">
          <Play :size="14" />
          开始录制
        </button>
      </template>

      <template v-else-if="recorder.isRecording">
        <button class="btn btn-warning" @click="pauseRecording">
          <Pause :size="14" />
          暂停
        </button>
        <button class="btn btn-danger" @click="stopRecording">
          <XCircle :size="14" />
          停止
        </button>
      </template>

      <template v-else-if="recorder.isPaused">
        <button class="btn btn-primary" @click="resumeRecording">
          <Play :size="14" />
          继续
        </button>
        <button class="btn btn-danger" @click="stopRecording">
          <XCircle :size="14" />
          停止
        </button>
      </template>
    </div>

    <!-- Convert dialog -->
    <div v-if="showConvertDialog" class="dialog-overlay" @click.self="discardRecording">
      <div class="dialog">
        <div class="dialog-header">
          <span>录制完成</span>
          <button class="btn-icon" @click="discardRecording">
            <X :size="18" />
          </button>
        </div>
        <div class="dialog-body convert-dialog-content">
          <p>录制已完成，共 {{ recorder.session?.actions?.length || 0 }} 个操作。</p>
          <p>是否将录制结果转换为工作流？</p>
        </div>
        <div class="dialog-footer">
          <button class="btn" @click="discardRecording">放弃</button>
          <button class="btn btn-primary" @click="convertRecording">
            转换为工作流
          </button>
        </div>
      </div>
    </div>

    <!-- Toast notifications -->
    <div class="toast-container">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast"
        :class="[`toast-${toast.type}`]"
      >
        {{ toast.message }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRecorderStore, type RecordedAction } from '@/stores/recorder';
import { useWorkflowStore } from '@/stores/workflow';
import {
  Video,
  Play,
  Pause,
  XCircle,
  Pointer,
  Pencil,
  Mouse,
  Keyboard,
  Timer,
  X,
} from 'lucide-vue-next';

const recorder = useRecorderStore();
const workflowStore = useWorkflowStore();

const recordingName = ref('新录制');
const showAllActions = ref(false);
const showConvertDialog = ref(false);

// Toast system
const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'info' }[]>([]);
let toastId = 0;

function showToast(message: string, type: 'success' | 'error' | 'info' = 'success') {
  const id = ++toastId;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id);
  }, 3000);
}

const statusClass = computed(() => {
  if (recorder.isRecording) return 'tag-danger';
  if (recorder.isPaused) return 'tag-warning';
  return 'tag-info';
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
      return Pencil;
    case 'Scroll':
      return Mouse;
    case 'Hotkey':
      return Keyboard;
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
    showToast('录制已开始', 'success');
  } catch (e) {
    showToast(`启动录制失败: ${e}`, 'error');
  }
}

async function pauseRecording() {
  try {
    await recorder.pauseRecording();
    showToast('录制已暂停', 'info');
  } catch (e) {
    showToast(`暂停录制失败: ${e}`, 'error');
  }
}

async function resumeRecording() {
  try {
    await recorder.resumeRecording();
    showToast('录制已继续', 'success');
  } catch (e) {
    showToast(`继续录制失败: ${e}`, 'error');
  }
}

async function stopRecording() {
  try {
    await recorder.stopRecording();
    await recorder.fetchSession();
    showConvertDialog.value = true;
  } catch (e) {
    showToast(`停止录制失败: ${e}`, 'error');
  }
}

async function convertRecording() {
  try {
    const workflow = await recorder.convertToWorkflow();
    if (workflow) {
      // Load the workflow into the designer
      workflowStore.loadFromJson(JSON.stringify(workflow));
      showToast('录制已转换为工作流', 'success');
    }
    showConvertDialog.value = false;
    await recorder.clearRecording();
  } catch (e) {
    showToast(`转换失败: ${e}`, 'error');
  }
}

async function discardRecording() {
  showConvertDialog.value = false;
  await recorder.clearRecording();
  showToast('录制已放弃', 'info');
}
</script>

<style scoped>
.recorder-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-color);
  border-radius: 4px;
  position: relative;
}

.recorder-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.recorder-header .title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: #1f2937;
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  font-size: 12px;
  border-radius: 4px;
  font-weight: 500;
}

.tag-info {
  background: #e0f2fe;
  color: #0369a1;
}

.tag-warning {
  background: #fef3c7;
  color: #92400e;
}

.tag-danger {
  background: #fef2f2;
  color: #dc2626;
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
  color: #6b7280;
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
  color: #6b7280;
}

.status-item .value {
  font-size: 14px;
  font-weight: 500;
  color: #1f2937;
}

.status-item .value.timer {
  font-family: monospace;
  font-size: 18px;
  color: #ef4444;
}

.action-list {
  margin-top: 16px;
  border: 1px solid var(--border-color);
  border-radius: 4px;
}

.action-list-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 12px;
  background: #f9fafb;
  border-bottom: 1px solid var(--border-color);
  font-size: 13px;
  color: #6b7280;
}

.btn-text {
  border: none;
  background: transparent;
  color: #3b82f6;
  font-size: 12px;
  cursor: pointer;
  padding: 0;
}

.btn-text:hover {
  color: #2563eb;
}

.action-list-body {
  max-height: 200px;
  overflow-y: auto;
}

.action-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-bottom: 1px solid #f3f4f6;
}

.action-item:last-child {
  border-bottom: none;
}

.action-item .icon-click {
  color: #3b82f6;
}

.action-item .icon-double-click {
  color: #f59e0b;
}

.action-item .icon-right-click {
  color: #22c55e;
}

.action-item .icon-input {
  color: #6b7280;
}

.action-item .icon-hotkey {
  color: #ef4444;
}

.action-text {
  flex: 1;
  font-size: 12px;
  color: #1f2937;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.action-time {
  font-size: 11px;
  color: #6b7280;
  font-family: monospace;
}

.recorder-footer {
  display: flex;
  gap: 8px;
  padding: 12px 16px;
  border-top: 1px solid var(--border-color);
}

.recorder-footer .btn {
  flex: 1;
}

.btn-warning {
  background: #f59e0b;
  color: #fff;
}

.btn-warning:hover {
  background: #d97706;
}

.btn-danger {
  background: #ef4444;
  color: #fff;
}

.btn-danger:hover {
  background: #dc2626;
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  border-radius: 4px;
  cursor: pointer;
  color: #6b7280;
}

.btn-icon:hover {
  background: #f3f4f6;
  color: #374151;
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.dialog {
  width: 400px;
  background: #fff;
  border-radius: 8px;
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.15);
}

.dialog-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  font-weight: 500;
  font-size: 16px;
}

.dialog-body {
  padding: 20px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  padding: 16px 20px;
  border-top: 1px solid var(--border-color);
}

.convert-dialog-content {
  text-align: center;
}

.convert-dialog-content p {
  margin: 8px 0;
  color: #374151;
}

.toast-container {
  position: fixed;
  top: 80px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toast {
  padding: 12px 20px;
  border-radius: 6px;
  font-size: 14px;
  color: #fff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  animation: slideIn 0.3s ease;
}

.toast-success {
  background: #22c55e;
}

.toast-error {
  background: #ef4444;
}

.toast-info {
  background: #3b82f6;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}
</style>
