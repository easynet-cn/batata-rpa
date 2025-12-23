<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import { RefreshCw } from 'lucide-vue-next';
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
  <div class="loop-node" :class="{ executing: isExecuting }">
    <Handle type="target" :position="Position.Top" />
    <div class="node-header">
      <RefreshCw :size="14" class="text-white" />
      <span>循环</span>
    </div>
    <div class="node-body">
      <span class="node-label">{{ label || '循环执行' }}</span>
      <div class="loop-info">
        <span v-if="data.loopType === 'count'">
          循环 {{ data.count || 1 }} 次
        </span>
        <span v-else-if="data.loopType === 'while'">
          条件: {{ data.condition || '...' }}
        </span>
        <span v-else>
          遍历: {{ data.listVariable || '...' }}
        </span>
      </div>
    </div>
    <div class="node-outputs">
      <div class="output-body">
        <span>循环体</span>
        <Handle id="body" type="source" :position="Position.Bottom" :style="{ left: '30%' }" />
      </div>
      <div class="output-done">
        <span>完成</span>
        <Handle id="done" type="source" :position="Position.Bottom" :style="{ left: '70%' }" />
      </div>
    </div>
  </div>
</template>

<style scoped>
.loop-node {
  min-width: 160px;
  background: #fff;
  border: 2px solid #6b7280;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.node-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: #6b7280;
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

.loop-info {
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

.output-body, .output-done {
  position: relative;
  text-align: center;
}

.output-body span {
  color: #3b82f6;
}

.output-done span {
  color: #22c55e;
}

.loop-node.executing {
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
