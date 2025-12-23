<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useExecutionStore, useWorkflowStore } from '@/stores';
import {
  Play,
  Pause,
  X,
  CheckCircle,
  AlertTriangle,
  XCircle,
  Trash2,
  FileText,
} from 'lucide-vue-next';

const executionStore = useExecutionStore();
const workflowStore = useWorkflowStore();

const logFilter = ref('all');
const isExecuting = ref(false);
const logListRef = ref<HTMLElement | null>(null);

// Toast system
const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'warning' }[]>([]);
let toastId = 0;

function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
  const id = ++toastId;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id);
  }, 3000);
}

const statusText = computed(() => {
  switch (executionStore.state?.status) {
    case 'running':
      return '运行中';
    case 'paused':
      return '已暂停';
    case 'completed':
      return '已完成';
    case 'failed':
      return '执行失败';
    default:
      return '空闲';
  }
});

const statusClass = computed(() => {
  switch (executionStore.state?.status) {
    case 'running':
      return 'tag-primary';
    case 'paused':
      return 'tag-warning';
    case 'completed':
      return 'tag-success';
    case 'failed':
      return 'tag-danger';
    default:
      return 'tag-info';
  }
});

const filteredLogs = computed(() => {
  const logs = executionStore.state?.logs || [];
  if (logFilter.value === 'all') {
    return logs;
  }
  return logs.filter(log => log.level === logFilter.value);
});

const executionDuration = computed(() => {
  if (!executionStore.state?.startTime) return '';
  const start = new Date(executionStore.state.startTime).getTime();
  const end = executionStore.state.endTime
    ? new Date(executionStore.state.endTime).getTime()
    : Date.now();
  const duration = Math.floor((end - start) / 1000);
  const minutes = Math.floor(duration / 60);
  const seconds = duration % 60;
  return minutes > 0 ? `${minutes}分${seconds}秒` : `${seconds}秒`;
});

function getLogIcon(level: string) {
  switch (level) {
    case 'error':
      return XCircle;
    case 'warn':
      return AlertTriangle;
    default:
      return CheckCircle;
  }
}

function getLogClass(level: string) {
  switch (level) {
    case 'error':
      return 'log-error';
    case 'warn':
      return 'log-warning';
    default:
      return 'log-info';
  }
}

async function startExecution() {
  if (!workflowStore.currentWorkflow) {
    showToast('请先选择或创建一个流程', 'warning');
    return;
  }

  isExecuting.value = true;
  executionStore.startExecution(workflowStore.currentWorkflow.id);

  try {
    const workflowData = {
      id: workflowStore.currentWorkflow.id,
      name: workflowStore.currentWorkflow.name,
      nodes: workflowStore.currentWorkflow.nodes.map(n => ({
        id: n.id,
        type: n.type,
        position: n.position,
        data: n.data,
        label: n.label,
      })),
      edges: workflowStore.currentWorkflow.edges.map(e => ({
        id: e.id,
        source: e.source,
        target: e.target,
        source_handle: e.sourceHandle,
        target_handle: e.targetHandle,
      })),
    };

    await invoke('execute_workflow', { workflow: workflowData });
    showToast('流程开始执行', 'success');
    pollExecutionState(workflowStore.currentWorkflow.id);
  } catch (error) {
    showToast(`执行失败: ${error}`, 'error');
    executionStore.failExecution(String(error));
    isExecuting.value = false;
  }
}

async function pollExecutionState(workflowId: string) {
  try {
    const state = await invoke('get_execution_state', { workflowId }) as {
      status: string;
      current_node_id?: string;
      logs: Array<{ level: string; message: string; node_id?: string }>;
    };

    if (state.current_node_id) {
      executionStore.setCurrentNode(state.current_node_id);
    }

    const currentLogCount = executionStore.state?.logs.length || 0;
    if (state.logs.length > currentLogCount) {
      const newLogs = state.logs.slice(currentLogCount);
      newLogs.forEach(log => {
        executionStore.addLog(log.level as 'info' | 'warn' | 'error', log.message, log.node_id);
      });
      scrollToBottom();
    }

    if (state.status === 'Running') {
      setTimeout(() => pollExecutionState(workflowId), 200);
    } else {
      isExecuting.value = false;
      executionStore.setCurrentNode('');
      if (state.status === 'Completed') {
        showToast('流程执行完成', 'success');
        executionStore.stopExecution();
      } else if (state.status === 'Failed') {
        showToast('流程执行失败', 'error');
      }
    }
  } catch {
    isExecuting.value = false;
  }
}

function clearLogs() {
  executionStore.clearState();
}

function scrollToBottom() {
  nextTick(() => {
    if (logListRef.value) {
      logListRef.value.scrollTop = logListRef.value.scrollHeight;
    }
  });
}

// Auto scroll when new logs added
watch(() => executionStore.state?.logs.length, () => {
  if (isExecuting.value) {
    scrollToBottom();
  }
});
</script>

<template>
  <div class="runner-container">
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

    <div class="runner-header">
      <div class="header-left">
        <h3>执行监控</h3>
        <span v-if="workflowStore.currentWorkflow" class="workflow-name">
          {{ workflowStore.currentWorkflow.name }}
        </span>
      </div>
      <div class="runner-controls">
        <span class="tag" :class="statusClass">{{ statusText }}</span>
        <span v-if="executionDuration" class="duration">
          耗时: {{ executionDuration }}
        </span>
        <div class="btn-group">
          <button
            class="btn btn-primary"
            :disabled="isExecuting"
            @click="startExecution"
          >
            <Play v-if="!isExecuting" :size="14" />
            {{ isExecuting ? '执行中...' : '开始' }}
          </button>
          <button
            v-if="executionStore.isRunning && !executionStore.isPaused"
            class="btn"
            @click="executionStore.pauseExecution"
          >
            <Pause :size="14" />
            暂停
          </button>
          <button
            v-if="executionStore.isPaused"
            class="btn btn-success"
            @click="executionStore.resumeExecution"
          >
            <Play :size="14" />
            继续
          </button>
          <button
            class="btn btn-danger"
            :disabled="!executionStore.isRunning && !executionStore.isPaused"
            @click="executionStore.stopExecution"
          >
            <X :size="14" />
            停止
          </button>
        </div>
      </div>
    </div>

    <div class="runner-body">
      <div class="log-panel">
        <div class="panel-header">
          <h4>执行日志</h4>
          <div class="log-controls">
            <select v-model="logFilter" class="select select-small">
              <option value="all">全部</option>
              <option value="info">信息</option>
              <option value="warn">警告</option>
              <option value="error">错误</option>
            </select>
            <button class="btn btn-small" @click="clearLogs">
              <Trash2 :size="14" />
              清空
            </button>
          </div>
        </div>
        <div ref="logListRef" class="log-list">
          <div v-if="!filteredLogs.length" class="empty-state">
            <FileText :size="48" class="empty-icon" />
            <span>暂无日志</span>
          </div>
          <div
            v-for="log in filteredLogs"
            :key="log.id"
            class="log-item"
            :class="log.level"
          >
            <component :is="getLogIcon(log.level)" :size="14" :class="getLogClass(log.level)" />
            <span class="log-time">{{ new Date(log.timestamp).toLocaleTimeString() }}</span>
            <span v-if="log.nodeId" class="log-node">[{{ log.nodeId.split('-')[0] }}]</span>
            <span class="log-message">{{ log.message }}</span>
          </div>
        </div>
        <div class="log-stats">
          共 {{ executionStore.state?.logs.length || 0 }} 条日志
          <span v-if="logFilter !== 'all'">
            (已过滤显示 {{ filteredLogs.length }} 条)
          </span>
        </div>
      </div>

      <div class="variable-panel">
        <div class="panel-header">
          <h4>变量状态</h4>
          <span class="tag tag-info">
            {{ Object.keys(executionStore.state?.variables || {}).length }} 个变量
          </span>
        </div>
        <div class="variable-list">
          <div v-if="!Object.keys(executionStore.state?.variables || {}).length" class="empty-state">
            <FileText :size="40" class="empty-icon" />
            <span>暂无变量</span>
          </div>
          <div
            v-for="(value, key) in executionStore.state?.variables"
            :key="key"
            class="variable-item"
          >
            <span class="variable-name">{{ key }}</span>
            <span class="variable-value">{{ JSON.stringify(value) }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.runner-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 16px;
  position: relative;
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

.toast-warning {
  background: #f59e0b;
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

.runner-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.header-left h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
}

.workflow-name {
  color: #6b7280;
  font-size: 14px;
}

.runner-controls {
  display: flex;
  align-items: center;
  gap: 16px;
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: 4px 12px;
  font-size: 13px;
  border-radius: 4px;
  font-weight: 500;
}

.tag-info {
  background: #e0f2fe;
  color: #0369a1;
}

.tag-primary {
  background: #dbeafe;
  color: #1d4ed8;
}

.tag-success {
  background: #dcfce7;
  color: #15803d;
}

.tag-warning {
  background: #fef3c7;
  color: #92400e;
}

.tag-danger {
  background: #fef2f2;
  color: #dc2626;
}

.duration {
  color: #6b7280;
  font-size: 14px;
}

.btn-group {
  display: flex;
  gap: 0;
}

.btn-group .btn {
  border-radius: 0;
}

.btn-group .btn:first-child {
  border-radius: 6px 0 0 6px;
}

.btn-group .btn:last-child {
  border-radius: 0 6px 6px 0;
}

.btn-success {
  background: #22c55e;
  color: #fff;
}

.btn-success:hover:not(:disabled) {
  background: #16a34a;
}

.btn-danger {
  background: #ef4444;
  color: #fff;
}

.btn-danger:hover:not(:disabled) {
  background: #dc2626;
}

.btn-small {
  padding: 4px 10px;
  font-size: 13px;
}

.select-small {
  padding: 4px 8px;
  font-size: 13px;
  min-width: 80px;
}

.runner-body {
  flex: 1;
  display: flex;
  gap: 16px;
  overflow: hidden;
}

.log-panel {
  flex: 2;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: #fff;
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--border-color);
}

.panel-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 500;
}

.log-controls {
  display: flex;
  gap: 8px;
}

.log-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 16px;
}

.empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 40px;
  color: #9ca3af;
}

.empty-icon {
  color: #d1d5db;
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid #f3f4f6;
  font-size: 13px;
}

.log-item.error {
  background: rgba(239, 68, 68, 0.05);
}

.log-item.warn {
  background: rgba(245, 158, 11, 0.05);
}

.log-info {
  color: #22c55e;
}

.log-warning {
  color: #f59e0b;
}

.log-error {
  color: #ef4444;
}

.log-time {
  color: #6b7280;
  font-size: 12px;
  min-width: 80px;
  font-family: monospace;
}

.log-node {
  color: #3b82f6;
  font-size: 11px;
  font-family: monospace;
}

.log-message {
  flex: 1;
  word-break: break-word;
}

.log-stats {
  padding: 8px 16px;
  font-size: 12px;
  color: #6b7280;
  border-top: 1px solid var(--border-color);
}

.variable-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: #fff;
}

.variable-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px 16px;
}

.variable-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 8px 0;
  border-bottom: 1px solid #f3f4f6;
}

.variable-name {
  font-weight: 500;
  font-size: 13px;
  color: #3b82f6;
}

.variable-value {
  font-size: 12px;
  font-family: monospace;
  color: #6b7280;
  word-break: break-all;
}
</style>
