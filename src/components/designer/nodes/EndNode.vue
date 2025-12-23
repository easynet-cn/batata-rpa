<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import { Square } from 'lucide-vue-next';
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
    <Square :size="20" class="text-white" />
    <span>{{ label || '结束' }}</span>
  </div>
</template>

<style scoped>
.end-node {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: #ef4444;
  color: #fff;
  border-radius: 20px;
  font-size: 14px;
  font-weight: 500;
  box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
}

.end-node.executing {
  box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.5), 0 2px 8px rgba(239, 68, 68, 0.3);
  animation: pulse 1.5s infinite;
}

@keyframes pulse {
  0%, 100% {
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.5), 0 2px 8px rgba(239, 68, 68, 0.3);
  }
  50% {
    box-shadow: 0 0 0 6px rgba(59, 130, 246, 0.3), 0 2px 8px rgba(239, 68, 68, 0.3);
  }
}
</style>
