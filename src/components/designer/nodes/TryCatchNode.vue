<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import { AlertTriangle } from 'lucide-vue-next';
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
      <AlertTriangle :size="14" class="text-white" />
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
  border: 2px solid #ef4444;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.node-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: #ef4444;
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
  color: #1f2937;
}

.node-detail {
  font-size: 11px;
  color: #6b7280;
}

.node-outputs {
  display: flex;
  justify-content: space-around;
  padding: 6px 10px;
  border-top: 1px solid #e5e7eb;
  font-size: 11px;
  color: #6b7280;
}

.output-try, .output-catch, .output-finally {
  position: relative;
  text-align: center;
}

.output-try span {
  color: #3b82f6;
}

.output-catch span {
  color: #ef4444;
}

.output-finally span {
  color: #22c55e;
}

.try-catch-node.executing {
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.5), 0 2px 8px rgba(0, 0, 0, 0.15);
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.5), 0 2px 8px rgba(0, 0, 0, 0.15);
  }
  50% {
    box-shadow: 0 0 0 6px rgba(59, 130, 246, 0.3), 0 2px 8px rgba(0, 0, 0, 0.15);
  }
}
</style>
