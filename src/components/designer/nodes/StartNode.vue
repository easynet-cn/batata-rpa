<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import { CaretRight } from '@element-plus/icons-vue';
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
  <div class="start-node" :class="{ executing: isExecuting }">
    <el-icon :size="20"><CaretRight /></el-icon>
    <span>{{ label || '开始' }}</span>
    <Handle type="source" :position="Position.Bottom" />
  </div>
</template>

<style scoped>
.start-node {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: #67c23a;
  color: #fff;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 2px 8px rgba(103, 194, 58, 0.3);
}

.start-node.executing {
  box-shadow: 0 0 0 3px rgba(64, 158, 255, 0.5), 0 2px 8px rgba(103, 194, 58, 0.3);
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 3px rgba(64, 158, 255, 0.5), 0 2px 8px rgba(103, 194, 58, 0.3);
  }
  50% {
    box-shadow: 0 0 0 6px rgba(64, 158, 255, 0.3), 0 2px 8px rgba(103, 194, 58, 0.3);
  }
}
</style>
