<script setup lang="ts">
import { computed } from 'vue';
import { useExecutionStore } from '@/stores';
import { FileText } from 'lucide-vue-next';

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

function getLogClass(level: string): string {
  switch (level) {
    case 'error': return 'log-error';
    case 'warn': return 'log-warning';
    default: return 'log-info';
  }
}

function getStatusClass(s: string): string {
  if (s === 'running') return 'tag-success';
  if (s === 'paused') return 'tag-warning';
  return 'tag-info';
}

function getStatusText(s: string): string {
  if (s === 'running') return '运行中';
  if (s === 'paused') return '已暂停';
  return s;
}
</script>

<template>
  <div class="debug-panel">
    <div class="debug-section">
      <div class="section-header">
        <span>执行状态</span>
        <span class="tag" :class="getStatusClass(status)">
          {{ getStatusText(status) }}
        </span>
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
        <span class="tag tag-info">{{ variables.length }}</span>
      </div>
      <div class="variables-list" v-if="variables.length > 0">
        <div v-for="variable in variables" :key="variable.name" class="variable-item">
          <div class="variable-header">
            <span class="variable-name">{{ variable.name }}</span>
            <span class="tag tag-info">{{ variable.type }}</span>
          </div>
          <div class="variable-value">{{ variable.value }}</div>
        </div>
      </div>
      <div v-else class="empty-state">
        <FileText :size="40" class="empty-icon" />
        <span class="empty-text">暂无变量</span>
      </div>
    </div>

    <div class="debug-section logs-section">
      <div class="section-header">
        <span>执行日志</span>
        <span class="tag tag-info">{{ logs.length }}</span>
      </div>
      <div class="logs-list" v-if="logs.length > 0">
        <div v-for="log in logs" :key="log.id" class="log-item">
          <span class="log-level" :class="getLogClass(log.level)">
            {{ log.level }}
          </span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
      <div v-else class="empty-state">
        <FileText :size="40" class="empty-icon" />
        <span class="empty-text">暂无日志</span>
      </div>
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
  color: #1f2937;
}

.tag {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  font-size: 12px;
  border-radius: 4px;
}

.tag-info {
  background: #e0f2fe;
  color: #0369a1;
}

.tag-success {
  background: #dcfce7;
  color: #15803d;
}

.tag-warning {
  background: #fef3c7;
  color: #92400e;
}

.status-info {
  background: #f9fafb;
  padding: 8px;
  border-radius: 4px;
}

.status-item {
  display: flex;
  gap: 8px;
}

.status-item .label {
  color: #6b7280;
}

.status-item .value {
  font-family: monospace;
  color: #3b82f6;
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
  background: #f9fafb;
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
  color: #3b82f6;
}

.variable-value {
  font-family: monospace;
  font-size: 12px;
  color: #374151;
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
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 500;
  text-transform: uppercase;
}

.log-info {
  background: #e0f2fe;
  color: #0369a1;
}

.log-warning {
  background: #fef3c7;
  color: #92400e;
}

.log-error {
  background: #fef2f2;
  color: #dc2626;
}

.log-message {
  flex: 1;
  word-break: break-all;
  font-family: monospace;
}

.empty-state {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 20px;
}

.empty-icon {
  color: #d1d5db;
}

.empty-text {
  color: #9ca3af;
  font-size: 14px;
}
</style>
