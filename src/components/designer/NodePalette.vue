<script setup lang="ts">
import { NODE_CONFIGS, type NodeType } from '@/types';
import * as Icons from '@element-plus/icons-vue';

const categories = [
  { key: 'control', label: '流程控制' },
  { key: 'action', label: '操作指令' },
  { key: 'data', label: '数据处理' },
];

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

function getIcon(iconName: string) {
  return (Icons as Record<string, unknown>)[iconName] || Icons.Document;
}
</script>

<template>
  <div class="node-palette">
    <div class="palette-header">
      <span>节点面板</span>
    </div>

    <el-collapse :model-value="['control', 'action', 'data']">
      <el-collapse-item
        v-for="category in nodesByCategory"
        :key="category.key"
        :title="category.label"
        :name="category.key"
      >
        <div class="node-list">
          <div
            v-for="node in category.nodes"
            :key="node.type"
            class="node-item"
            draggable="true"
            @dragstart="onDragStart($event, node.type)"
          >
            <div class="node-icon" :style="{ backgroundColor: node.color }">
              <el-icon :size="16" color="#fff">
                <component :is="getIcon(node.icon)" />
              </el-icon>
            </div>
            <span class="node-label">{{ node.label }}</span>
          </div>
        </div>
      </el-collapse-item>
    </el-collapse>
  </div>
</template>

<style scoped>
.node-palette {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.palette-header {
  padding: 12px 16px;
  font-weight: 500;
  border-bottom: 1px solid var(--el-border-color-light);
}

.node-list {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.node-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  border-radius: 4px;
  cursor: grab;
  transition: background-color 0.2s;
}

.node-item:hover {
  background-color: var(--el-fill-color-light);
}

.node-item:active {
  cursor: grabbing;
}

.node-icon {
  width: 28px;
  height: 28px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.node-label {
  font-size: 13px;
}
</style>
