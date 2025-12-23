<script setup lang="ts">
import { ref } from 'vue';
import { NODE_CONFIGS, type NodeType } from '@/types';
import { useWorkflowStore } from '@/stores';
import { ChevronDown, ChevronRight } from 'lucide-vue-next';
import * as LucideIcons from 'lucide-vue-next';

const workflowStore = useWorkflowStore();

const categories = [
  { key: 'control', label: '流程控制' },
  { key: 'action', label: '操作指令' },
  { key: 'data', label: '数据处理' },
];

const expandedCategories = ref<Record<string, boolean>>({
  control: true,
  action: true,
  data: true,
});

const nodesByCategory = categories.map((cat) => ({
  ...cat,
  nodes: Object.values(NODE_CONFIGS).filter((n) => n.category === cat.key),
}));

function onDragStart(event: DragEvent, type: NodeType) {
  if (event.dataTransfer) {
    event.dataTransfer.setData('application/rpa-node', type);
    event.dataTransfer.effectAllowed = 'move';
  }
}

function onNodeClick(type: NodeType) {
  const config = NODE_CONFIGS[type];
  if (!config || !workflowStore.currentWorkflow) {
    return;
  }

  const existingNodes = workflowStore.currentNodes;
  const offsetX = (existingNodes.length % 5) * 50;
  const offsetY = Math.floor(existingNodes.length / 5) * 80;

  const newNode = {
    id: `${type}-${Date.now()}`,
    type,
    position: { x: 300 + offsetX, y: 150 + offsetY },
    data: {},
    label: config.label,
  };

  workflowStore.addNode(newNode);
}

function getIcon(iconName: string) {
  return (LucideIcons as Record<string, unknown>)[iconName] || LucideIcons.FileText;
}

function toggleCategory(key: string) {
  expandedCategories.value[key] = !expandedCategories.value[key];
}
</script>

<template>
  <div class="node-palette">
    <div class="palette-header">节点面板</div>

    <div class="flex-1 overflow-y-auto">
      <div v-for="category in nodesByCategory" :key="category.key" class="node-category">
        <div class="node-category-header" @click="toggleCategory(category.key)">
          <span>{{ category.label }}</span>
          <component
            :is="expandedCategories[category.key] ? ChevronDown : ChevronRight"
            :size="16"
            class="text-gray-400"
          />
        </div>

        <div v-show="expandedCategories[category.key]" class="node-list">
          <div
            v-for="node in category.nodes"
            :key="node.type"
            class="node-item"
            draggable="true"
            title="点击添加到画布，或拖拽到指定位置"
            @dragstart="onDragStart($event, node.type)"
            @click="onNodeClick(node.type)"
          >
            <div class="node-icon" :style="{ backgroundColor: node.color }">
              <component :is="getIcon(node.icon)" :size="16" />
            </div>
            <span class="node-label">{{ node.label }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
