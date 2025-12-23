<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import { Switch } from '@element-plus/icons-vue';
import { useExecutionStore } from '@/stores';

const props = defineProps<{
  id: string;
  data: Record<string, unknown>;
  label?: string;
}>();

const executionStore = useExecutionStore();
const isExecuting = computed(() => executionStore.state?.currentNodeId === props.id);
</script>

<template>
  <div class="condition-node" :class="{ executing: isExecuting }">
    <Handle type="target" :position="Position.Top" />
    <div class="node-header">
      <el-icon :size="14" color="#fff"><Switch /></el-icon>
      <span>条件分支</span>
    </div>
    <div class="node-body">
      <span class="node-label">{{ label || '条件判断' }}</span>
      <span v-if="data.expression" class="node-expression">{{ data.expression }}</span>
    </div>
    <div class="node-outputs">
      <div class="output-true">
        <span>是</span>
        <Handle id="true" type="source" :position="Position.Bottom" :style="{ left: '25%' }" />
      </div>
      <div class="output-false">
        <span>否</span>
        <Handle id="false" type="source" :position="Position.Bottom" :style="{ left: '75%' }" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.condition-node {
  min-width: 160px;
  background: #fff;
  border: 2px solid #e6a23c;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.node-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: #e6a23c;
  color: #fff;
  font-size: 12px;
  font-weight: 500;
}

.node-body {
  padding: 8px 10px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.node-label {
  font-size: 13px;
  color: var(--el-text-color-primary);
}

.node-expression {
  font-size: 11px;
  color: var(--el-text-color-secondary);
  font-family: monospace;
  background: var(--el-fill-color-light);
  padding: 2px 6px;
  border-radius: 4px;
}

.node-outputs {
  display: flex;
  justify-content: space-around;
  padding: 6px 10px;
  border-top: 1px solid var(--el-border-color-lighter);
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.output-true, .output-false {
  position: relative;
  text-align: center;
}

.output-true span {
  color: #67c23a;
}

.output-false span {
  color: #f56c6c;
}

.condition-node.executing {
  box-shadow: 0 0 0 3px rgba(64, 158, 255, 0.5), 0 2px 8px rgba(0, 0, 0, 0.15);
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 3px rgba(64, 158, 255, 0.5), 0 2px 8px rgba(0, 0, 0, 0.15);
  }
  50% {
    box-shadow: 0 0 0 6px rgba(64, 158, 255, 0.3), 0 2px 8px rgba(0, 0, 0, 0.15);
  }
}
</style>
