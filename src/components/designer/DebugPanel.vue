<script setup lang="ts">
import { computed } from 'vue';
import { useExecutionStore } from '@/stores';

const executionStore = useExecutionStore();

const variables = computed(() => {
  if (!executionStore.state?.variables) return [];
  return Object.entries(executionStore.state.variables).map(([name, value]) => ({
    name,
    value: formatValue(value),
    type: getType(value),
  }));
});

const logs = computed(() => {
  if (!executionStore.state?.logs) return [];
  return executionStore.state.logs.slice(-50).reverse();
});

const currentNode = computed(() => executionStore.state?.currentNodeId || '');
const status = computed(() => executionStore.state?.status || 'idle');
const isDebugging = computed(() => executionStore.isDebugging);

function formatValue(value: unknown): string {
  if (value === null || value === undefined) return 'null';
  if (typeof value === 'object') {
    try {
      return JSON.stringify(value, null, 2);
    } catch {
      return String(value);
    }
  }
  return String(value);
}

function getType(value: unknown): string {
  if (value === null) return 'null';
  if (Array.isArray(value)) return 'array';
  return typeof value;
}

function getLogColor(level: string): string {
  switch (level) {
    case 'error': return 'danger';
    case 'warn': return 'warning';
    default: return 'info';
  }
}
</script>

<template>
  <div class="debug-panel">
    <div class="debug-section">
      <div class="section-header">
        <span>执行状态</span>
        <el-tag :type="status === 'running' ? 'success' : status === 'paused' ? 'warning' : 'info'" size="small">
          {{ status === 'running' ? '运行中' : status === 'paused' ? '已暂停' : status }}
        </el-tag>
      </div>
      <div class="status-info" v-if="isDebugging">
        <div class="status-item">
          <span class="label">当前节点:</span>
          <span class="value">{{ currentNode || '无' }}</span>
        </div>
      </div>
    </div>

    <div class="debug-section variables-section">
      <div class="section-header">
        <span>变量监视</span>
        <el-tag size="small">{{ variables.length }}</el-tag>
      </div>
      <div class="variables-list" v-if="variables.length > 0">
        <div v-for="variable in variables" :key="variable.name" class="variable-item">
          <div class="variable-header">
            <span class="variable-name">{{ variable.name }}</span>
            <el-tag size="small" type="info">{{ variable.type }}</el-tag>
          </div>
          <div class="variable-value">{{ variable.value }}</div>
        </div>
      </div>
      <el-empty v-else description="暂无变量" :image-size="60" />
    </div>

    <div class="debug-section logs-section">
      <div class="section-header">
        <span>执行日志</span>
        <el-tag size="small">{{ logs.length }}</el-tag>
      </div>
      <div class="logs-list" v-if="logs.length > 0">
        <div v-for="log in logs" :key="log.id" class="log-item">
          <el-tag :type="getLogColor(log.level)" size="small" class="log-level">
            {{ log.level }}
          </el-tag>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
      <el-empty v-else description="暂无日志" :image-size="60" />
    </div>
  </div>
</template>

<style scoped>
.debug-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  padding: 12px;
  gap: 12px;
  overflow: hidden;
}

.debug-section {
  flex-shrink: 0;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-weight: 500;
  margin-bottom: 8px;
  color: var(--el-text-color-primary);
}

.status-info {
  background: var(--el-fill-color-light);
  padding: 8px;
  border-radius: 4px;
}

.status-item {
  display: flex;
  gap: 8px;
}

.status-item .label {
  color: var(--el-text-color-secondary);
}

.status-item .value {
  font-family: monospace;
  color: var(--el-color-primary);
}

.variables-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.variables-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.variable-item {
  background: var(--el-fill-color-light);
  padding: 8px;
  border-radius: 4px;
}

.variable-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 4px;
}

.variable-name {
  font-weight: 500;
  color: var(--el-color-primary);
}

.variable-value {
  font-family: monospace;
  font-size: 12px;
  color: var(--el-text-color-regular);
  word-break: break-all;
  white-space: pre-wrap;
  max-height: 80px;
  overflow-y: auto;
}

.logs-section {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}

.logs-list {
  flex: 1;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.log-item {
  display: flex;
  align-items: flex-start;
  gap: 8px;
  padding: 4px;
  font-size: 12px;
}

.log-level {
  flex-shrink: 0;
}

.log-message {
  flex: 1;
  word-break: break-all;
  font-family: monospace;
}
</style>
