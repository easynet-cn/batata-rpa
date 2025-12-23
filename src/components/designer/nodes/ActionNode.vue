<script setup lang="ts">
import { computed } from 'vue';
import { Handle, Position } from '@vue-flow/core';
import { NODE_CONFIGS } from '@/types';
import { useExecutionStore } from '@/stores';
import * as LucideIcons from 'lucide-vue-next';

const props = defineProps<{
  id: string;
  type: string;
  data: Record<string, unknown>;
  label?: string;
}>();

const executionStore = useExecutionStore();
const config = computed(() => NODE_CONFIGS[props.type as keyof typeof NODE_CONFIGS]);
const icon = computed(() => (LucideIcons as Record<string, unknown>)[config.value?.icon] || LucideIcons.FileText);
const isExecuting = computed(() => executionStore.state?.currentNodeId === props.id);
</script>

<template>
  <div class="action-node" :class="{ executing: isExecuting }" :style="{ borderColor: config?.color }">
    <Handle type="target" :position="Position.Top" />
    <div class="node-header" :style="{ backgroundColor: config?.color }">
      <component :is="icon" :size="14" class="text-white" />
      <span>{{ config?.label }}</span>
    </div>
    <div class="node-body">
      <span class="node-label">{{ label || config?.label }}</span>
      <span v-if="data.elementName" class="node-detail">{{ data.elementName }}</span>
      <span v-if="data.text" class="node-detail">{{ data.text }}</span>
      <span v-if="data.delay" class="node-detail">{{ data.delay }}ms</span>
    </div>
    <Handle type="source" :position="Position.Bottom" />
  </div>
</template>

<style scoped>
.action-node {
  min-width: 150px;
  background: #fff;
  border: 2px solid;
  border-radius: 8px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.node-header {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
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
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 130px;
}

.action-node.executing {
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
