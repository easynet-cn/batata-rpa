<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import { Warning } from '@element-plus/icons-vue';
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
  <div class="try-catch-node" :class="{ executing: isExecuting }">
    <Handle type="target" :position="Position.Top" />
    <div class="node-header">
      <el-icon :size="14" color="#fff"><Warning /></el-icon>
      <span>异常处理</span>
    </div>
    <div class="node-body">
      <span class="node-label">{{ label || 'Try-Catch' }}</span>
      <span v-if="data.errorVariable" class="node-detail">
        错误变量: {{ data.errorVariable }}
      </span>
    </div>
    <div class="node-outputs">
      <div class="output-try">
        <span>执行</span>
        <Handle id="try" type="source" :position="Position.Bottom" :style="{ left: '25%' }" />
      </div>
      <div class="output-catch">
        <span>异常</span>
        <Handle id="catch" type="source" :position="Position.Bottom" :style="{ left: '50%' }" />
      </div>
      <div class="output-finally">
        <span>完成</span>
        <Handle id="finally" type="source" :position="Position.Bottom" :style="{ left: '75%' }" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.try-catch-node {
  min-width: 180px;
  background: #fff;
  border: 2px solid #f56c6c;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.node-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: #f56c6c;
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

.node-detail {
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.node-outputs {
  display: flex;
  justify-content: space-around;
  padding: 6px 10px;
  border-top: 1px solid var(--el-border-color-lighter);
  font-size: 11px;
  color: var(--el-text-color-secondary);
}

.output-try, .output-catch, .output-finally {
  position: relative;
  text-align: center;
}

.output-try span {
  color: #409eff;
}

.output-catch span {
  color: #f56c6c;
}

.output-finally span {
  color: #67c23a;
}

.try-catch-node.executing {
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
