<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { useExecutionStore, useWorkflowStore } from '@/stores';
import {
  VideoPlay,
  VideoPause,
  Close,
  SuccessFilled,
  WarningFilled,
  CircleCloseFilled,
  Delete,
} from '@element-plus/icons-vue';

const executionStore = useExecutionStore();
const workflowStore = useWorkflowStore();

const logFilter = ref('all');
const isExecuting = ref(false);
const logListRef = ref<HTMLElement | null>(null);

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

const statusType = computed(() => {
  switch (executionStore.state?.status) {
    case 'running':
      return 'primary';
    case 'paused':
      return 'warning';
    case 'completed':
      return 'success';
    case 'failed':
      return 'danger';
    default:
      return 'info';
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
      return CircleCloseFilled;
    case 'warn':
      return WarningFilled;
    default:
      return SuccessFilled;
  }
}

function getLogColor(level: string) {
  switch (level) {
    case 'error':
      return 'var(--el-color-danger)';
    case 'warn':
      return 'var(--el-color-warning)';
    default:
      return 'var(--el-color-success)';
  }
}

async function startExecution() {
  if (!workflowStore.currentWorkflow) {
    ElMessage.warning('请先选择或创建一个流程');
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
        node_type: n.type,
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
    ElMessage.success('流程开始执行');
    pollExecutionState(workflowStore.currentWorkflow.id);
  } catch (error) {
    ElMessage.error(`执行失败: ${error}`);
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
        ElMessage.success('流程执行完成');
        executionStore.stopExecution();
      } else if (state.status === 'Failed') {
        ElMessage.error('流程执行失败');
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
    <div class="runner-header">
      <div class="header-left">
        <h3>执行监控</h3>
        <span v-if="workflowStore.currentWorkflow" class="workflow-name">
          {{ workflowStore.currentWorkflow.name }}
        </span>
      </div>
      <div class="runner-controls">
        <el-tag :type="statusType" size="large">{{ statusText }}</el-tag>
        <span v-if="executionDuration" class="duration">
          耗时: {{ executionDuration }}
        </span>
        <el-button-group>
          <el-button
            type="primary"
            :disabled="isExecuting"
            :loading="isExecuting"
            @click="startExecution"
          >
            <el-icon v-if="!isExecuting"><VideoPlay /></el-icon>
            {{ isExecuting ? '执行中...' : '开始' }}
          </el-button>
          <el-button
            v-if="executionStore.isRunning && !executionStore.isPaused"
            @click="executionStore.pauseExecution"
          >
            <el-icon><VideoPause /></el-icon>
            暂停
          </el-button>
          <el-button
            v-if="executionStore.isPaused"
            type="success"
            @click="executionStore.resumeExecution"
          >
            <el-icon><VideoPlay /></el-icon>
            继续
          </el-button>
          <el-button
            type="danger"
            :disabled="!executionStore.isRunning && !executionStore.isPaused"
            @click="executionStore.stopExecution"
          >
            <el-icon><Close /></el-icon>
            停止
          </el-button>
        </el-button-group>
      </div>
    </div>

    <div class="runner-body">
      <div class="log-panel">
        <div class="panel-header">
          <h4>执行日志</h4>
          <div class="log-controls">
            <el-select v-model="logFilter" size="small" style="width: 100px">
              <el-option label="全部" value="all" />
              <el-option label="信息" value="info" />
              <el-option label="警告" value="warn" />
              <el-option label="错误" value="error" />
            </el-select>
            <el-button size="small" :icon="Delete" @click="clearLogs">清空</el-button>
          </div>
        </div>
        <div ref="logListRef" class="log-list">
          <el-empty v-if="!filteredLogs.length" description="暂无日志" />
          <div
            v-for="log in filteredLogs"
            :key="log.id"
            class="log-item"
            :class="log.level"
          >
            <el-icon :style="{ color: getLogColor(log.level) }">
              <component :is="getLogIcon(log.level)" />
            </el-icon>
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
          <el-tag size="small">
            {{ Object.keys(executionStore.state?.variables || {}).length }} 个变量
          </el-tag>
        </div>
        <div class="variable-list">
          <el-empty v-if="!Object.keys(executionStore.state?.variables || {}).length" description="暂无变量" />
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
}

.workflow-name {
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.runner-controls {
  display: flex;
  align-items: center;
  gap: 16px;
}

.duration {
  color: var(--el-text-color-secondary);
  font-size: 14px;
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
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  background: var(--el-bg-color);
}

.panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.panel-header h4 {
  margin: 0;
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

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 8px 0;
  border-bottom: 1px solid var(--el-border-color-lighter);
  font-size: 13px;
}

.log-item.error {
  background: rgba(245, 108, 108, 0.05);
}

.log-item.warn {
  background: rgba(230, 162, 60, 0.05);
}

.log-time {
  color: var(--el-text-color-secondary);
  font-size: 12px;
  min-width: 80px;
  font-family: monospace;
}

.log-node {
  color: var(--el-color-primary);
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
  color: var(--el-text-color-secondary);
  border-top: 1px solid var(--el-border-color-lighter);
}

.variable-panel {
  flex: 1;
  display: flex;
  flex-direction: column;
  border: 1px solid var(--el-border-color);
  border-radius: 8px;
  background: var(--el-bg-color);
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
  border-bottom: 1px solid var(--el-border-color-lighter);
}

.variable-name {
  font-weight: 500;
  font-size: 13px;
  color: var(--el-color-primary);
}

.variable-value {
  font-size: 12px;
  font-family: monospace;
  color: var(--el-text-color-secondary);
  word-break: break-all;
}
</style>
