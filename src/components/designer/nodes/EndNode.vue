<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import { Close } from '@element-plus/icons-vue';
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
  <div class="end-node" :class="{ executing: isExecuting }">
    <Handle type="target" :position="Position.Top" />
    <el-icon :size="20"><Close /></el-icon>
    <span>{{ label || '结束' }}</span>
  </div>
</template>

<style scoped>
.end-node {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: #f56c6c;
  color: #fff;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 2px 8px rgba(245, 108, 108, 0.3);
}

.end-node.executing {
  box-shadow: 0 0 0 3px rgba(64, 158, 255, 0.5), 0 2px 8px rgba(245, 108, 108, 0.3);
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 3px rgba(64, 158, 255, 0.5), 0 2px 8px rgba(245, 108, 108, 0.3);
  }
  50% {
    box-shadow: 0 0 0 6px rgba(64, 158, 255, 0.3), 0 2px 8px rgba(245, 108, 108, 0.3);
  }
}
</style>
