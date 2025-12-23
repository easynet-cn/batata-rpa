<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue';
import { VueFlow, useVueFlow } from '@vue-flow/core';
import type { Node, Edge } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import { MiniMap } from '@vue-flow/minimap';
import { Play, Download, ChevronRight, Pause, StepForward, Crosshair, ChevronDown, Video, FolderOpen, FileText } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import { useWorkflowStore, useExecutionStore, type DebugMode } from '@/stores';
import { useRecorderStore } from '@/stores/recorder';
import { NODE_CONFIGS, type NodeType } from '@/types';
import NodePalette from '@/components/designer/NodePalette.vue';
import PropertyPanel from '@/components/designer/PropertyPanel.vue';
import VariablePanel from '@/components/designer/VariablePanel.vue';
import DebugPanel from '@/components/designer/DebugPanel.vue';
import RecorderPanel from '@/components/designer/RecorderPanel.vue';
import StartNode from '@/components/designer/nodes/StartNode.vue';
import EndNode from '@/components/designer/nodes/EndNode.vue';
import ActionNode from '@/components/designer/nodes/ActionNode.vue';
import ConditionNode from '@/components/designer/nodes/ConditionNode.vue';
import LoopNode from '@/components/designer/nodes/LoopNode.vue';
import TryCatchNode from '@/components/designer/nodes/TryCatchNode.vue';

// Simple toast notification system
const toasts = ref<{ id: number; message: string; type: 'success' | 'error' | 'warning' }[]>([]);
let toastId = 0;

function showToast(message: string, type: 'success' | 'error' | 'warning' = 'success') {
  const id = ++toastId;
  toasts.value.push({ id, message, type });
  setTimeout(() => {
    toasts.value = toasts.value.filter(t => t.id !== id);
  }, 3000);
}

const workflowStore = useWorkflowStore();
const executionStore = useExecutionStore();
const recorderStore = useRecorderStore();

// 本地节点和边数组，用于 Vue Flow
const nodes = ref<Node[]>([]);
const edges = ref<Edge[]>([]);

// 防止 watch 循环的标志
let isUpdatingFromStore = false;
let isUpdatingFromLocal = false;

const { onConnect, addEdges, onNodeClick, screenToFlowCoordinate } = useVueFlow();

// 初始化时同步 store 数据到本地
onMounted(() => {
  if (!workflowStore.currentWorkflow) {
    workflowStore.createWorkflow('新建流程');
  }
  // 同步初始节点
  syncFromStore();
});

// 从 store 同步到本地
function syncFromStore() {
  if (workflowStore.currentWorkflow) {
    isUpdatingFromStore = true;
    nodes.value = workflowStore.currentWorkflow.nodes.map(n => ({
      id: n.id,
      type: n.type,
      position: n.position,
      data: n.data,
      label: n.label,
    }));
    edges.value = workflowStore.currentWorkflow.edges.map(e => ({
      id: e.id,
      source: e.source,
      target: e.target,
      sourceHandle: e.sourceHandle,
      targetHandle: e.targetHandle,
    }));
    setTimeout(() => { isUpdatingFromStore = false; }, 0);
  }
}

// 监听 store 变化，同步到本地
watch(() => workflowStore.currentWorkflow?.nodes.length, () => {
  if (isUpdatingFromLocal) return;
  syncFromStore();
});

watch(() => workflowStore.currentWorkflow?.edges.length, () => {
  if (isUpdatingFromLocal) return;
  if (workflowStore.currentWorkflow) {
    isUpdatingFromStore = true;
    edges.value = workflowStore.currentWorkflow.edges.map(e => ({
      id: e.id,
      source: e.source,
      target: e.target,
      sourceHandle: e.sourceHandle,
      targetHandle: e.targetHandle,
    }));
    setTimeout(() => { isUpdatingFromStore = false; }, 0);
  }
});

// 监听本地节点变化，同步到 store（仅位置变化等）
watch(nodes, (newNodes) => {
  if (isUpdatingFromStore) return;
  if (workflowStore.currentWorkflow) {
    isUpdatingFromLocal = true;
    workflowStore.currentWorkflow.nodes = newNodes.map(n => ({
      id: n.id,
      type: n.type || 'default',
      position: n.position,
      data: n.data || {},
      label: typeof n.label === 'string' ? n.label : undefined,
    }));
    setTimeout(() => { isUpdatingFromLocal = false; }, 0);
  }
}, { deep: true });

watch(edges, (newEdges) => {
  if (isUpdatingFromStore) return;
  if (workflowStore.currentWorkflow) {
    isUpdatingFromLocal = true;
    workflowStore.currentWorkflow.edges = newEdges.map(e => ({
      id: e.id,
      source: e.source,
      target: e.target,
      sourceHandle: e.sourceHandle ?? undefined,
      targetHandle: e.targetHandle ?? undefined,
    }));
    setTimeout(() => { isUpdatingFromLocal = false; }, 0);
  }
}, { deep: true });

onConnect((params) => {
  addEdges([params]);
  workflowStore.addEdge({
    id: `e-${params.source}-${params.target}`,
    source: params.source,
    target: params.target,
    sourceHandle: params.sourceHandle ?? undefined,
    targetHandle: params.targetHandle ?? undefined,
  });
});

onNodeClick(({ node }) => {
  workflowStore.selectNode(node.id);
});

function onDrop(event: DragEvent) {
  event.preventDefault();

  const type = event.dataTransfer?.getData('application/rpa-node') as NodeType;
  if (!type) {
    return;
  }

  const config = NODE_CONFIGS[type];
  if (!config) {
    return;
  }

  // Use screenToFlowCoordinate for proper coordinate transformation
  const position = screenToFlowCoordinate({
    x: event.clientX,
    y: event.clientY,
  });

  const newNode: Node = {
    id: `${type}-${Date.now()}`,
    type,
    position,
    data: {},
    label: config.label,
  };

  // 直接添加到本地 nodes 数组
  nodes.value = [...nodes.value, newNode];
}

function onDragOver(event: DragEvent) {
  event.preventDefault();
  if (event.dataTransfer) {
    event.dataTransfer.dropEffect = 'move';
  }
}

const isRunning = ref(false);
const rightPanelTab = ref('properties');

// Debug state
const debugMode = computed(() => executionStore.debugMode);
const isDebugging = computed(() => executionStore.isDebugging);
const isPaused = computed(() => executionStore.isPaused);

async function runWorkflow() {
  if (!workflowStore.currentWorkflow) return;

  isRunning.value = true;
  executionStore.startExecution(workflowStore.currentWorkflow.id);

  try {
    const workflowData = {
      id: workflowStore.currentWorkflow.id,
      name: workflowStore.currentWorkflow.name,
      nodes: workflowStore.currentWorkflow.nodes.map(n => ({
        id: n.id,
        type: n.type,
        position: n.position,
        data: n.data,
        label: n.label,
      })),
      edges: workflowStore.currentWorkflow.edges.map(e => ({
        id: e.id,
        source: e.source,
        target: e.target,
        source_handle: e.sourceHandle,
        target_handle: e.targetHandle,
      })),
    };

    await invoke('execute_workflow', { workflow: workflowData });
    showToast('流程开始执行', 'success');

    // Poll execution state
    pollExecutionState(workflowStore.currentWorkflow.id);
  } catch (error) {
    showToast(`执行失败: ${error}`, 'error');
    executionStore.failExecution(String(error));
    isRunning.value = false;
  }
}

interface ExecutionStateResult {
  status: string;
  current_node_id?: string;
  logs: Array<{ level: string; message: string; node_id?: string }>;
}

async function pollExecutionState(workflowId: string) {
  try {
    const state = await invoke('get_execution_state', { workflowId });
    const typedState = state as ExecutionStateResult;

    // Update current node for highlighting
    if (typedState.current_node_id) {
      executionStore.setCurrentNode(typedState.current_node_id);
    }

    // Update logs - only add new logs
    const currentLogCount = executionStore.state?.logs.length || 0;
    if (typedState.logs.length > currentLogCount) {
      const newLogs = typedState.logs.slice(currentLogCount);
      newLogs.forEach(log => {
        executionStore.addLog(log.level as 'info' | 'warn' | 'error', log.message, log.node_id);
      });
    }

    if (typedState.status === 'Running') {
      setTimeout(() => pollExecutionState(workflowId), 200);
    } else {
      isRunning.value = false;
      if (typedState.current_node_id) {
        executionStore.setCurrentNode(''); // Clear highlighting
      }
      if (typedState.status === 'Completed') {
        showToast('流程执行完成', 'success');
        executionStore.stopExecution();
      } else if (typedState.status === 'Failed') {
        showToast('流程执行失败', 'error');
      }
    }
  } catch {
    isRunning.value = false;
  }
}

function saveWorkflow() {
  const json = workflowStore.saveToJson();
  const blob = new Blob([json], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = `${workflowStore.currentWorkflow?.name || 'workflow'}.json`;
  a.click();
  URL.revokeObjectURL(url);
}

// 示例流程列表
const exampleWorkflows = [
  { name: 'Google搜索写入Excel', file: 'google-search-to-excel.json' },
  { name: '列出用户目录写入Excel', file: 'list-home-directory-to-excel.json' },
];

// 加载示例流程
async function loadExample(filename: string) {
  try {
    const response = await fetch(`/examples/${filename}`);
    if (!response.ok) {
      throw new Error('加载失败');
    }
    const json = await response.text();
    workflowStore.loadFromJson(json);
    syncFromStore();
    showToast('示例流程加载成功', 'success');
  } catch (error) {
    showToast(`加载示例失败: ${error}`, 'error');
  }
}

// Debug functions
async function runDebugWorkflow(mode: DebugMode) {
  if (!workflowStore.currentWorkflow) return;

  isRunning.value = true;
  executionStore.setDebugMode(mode);
  executionStore.startExecution(workflowStore.currentWorkflow.id);

  try {
    const workflowData = {
      id: workflowStore.currentWorkflow.id,
      name: workflowStore.currentWorkflow.name,
      nodes: workflowStore.currentWorkflow.nodes.map(n => ({
        id: n.id,
        type: n.type,
        position: n.position,
        data: n.data,
        label: n.label,
      })),
      edges: workflowStore.currentWorkflow.edges.map(e => ({
        id: e.id,
        source: e.source,
        target: e.target,
        source_handle: e.sourceHandle,
        target_handle: e.targetHandle,
      })),
    };

    await invoke('execute_workflow_debug', { workflow: workflowData, debugMode: mode });
    showToast(`调试模式开始: ${mode === 'step' ? '单步执行' : '断点调试'}`, 'success');

    // Poll execution state with variable updates
    pollDebugExecutionState(workflowStore.currentWorkflow.id);
  } catch (error) {
    showToast(`调试失败: ${error}`, 'error');
    executionStore.failExecution(String(error));
    executionStore.setDebugMode('none');
    isRunning.value = false;
  }
}

async function stepExecution() {
  if (!workflowStore.currentWorkflow) return;

  try {
    await invoke('step_execution', { workflowId: workflowStore.currentWorkflow.id });
  } catch (error) {
    showToast(`单步执行失败: ${error}`, 'error');
  }
}

async function resumeDebugExecution() {
  if (!workflowStore.currentWorkflow) return;

  try {
    await invoke('resume_execution', { workflowId: workflowStore.currentWorkflow.id });
    executionStore.resumeExecution();
  } catch (error) {
    showToast(`继续执行失败: ${error}`, 'error');
  }
}

async function pollDebugExecutionState(workflowId: string) {
  try {
    const state = await invoke('get_execution_state', { workflowId });
    const typedState = state as ExecutionStateResult;

    // Update current node for highlighting
    if (typedState.current_node_id) {
      executionStore.setCurrentNode(typedState.current_node_id);
    }

    // Update logs
    const currentLogCount = executionStore.state?.logs.length || 0;
    if (typedState.logs.length > currentLogCount) {
      const newLogs = typedState.logs.slice(currentLogCount);
      newLogs.forEach(log => {
        executionStore.addLog(log.level as 'info' | 'warn' | 'error', log.message, log.node_id);
      });
    }

    // Fetch variables for debug panel
    try {
      const variables = await invoke('get_variables', { workflowId });
      if (variables) {
        executionStore.updateVariables(variables as Record<string, unknown>);
      }
    } catch {
      // Ignore variable fetch errors
    }

    // Update paused state
    if (typedState.status === 'Paused') {
      executionStore.pauseExecution();
    }

    if (typedState.status === 'Running' || typedState.status === 'Paused') {
      setTimeout(() => pollDebugExecutionState(workflowId), 200);
    } else {
      isRunning.value = false;
      executionStore.setDebugMode('none');
      if (typedState.current_node_id) {
        executionStore.setCurrentNode('');
      }
      if (typedState.status === 'Completed') {
        showToast('调试执行完成', 'success');
        executionStore.stopExecution();
      } else if (typedState.status === 'Failed') {
        showToast('调试执行失败', 'error');
      }
    }
  } catch {
    isRunning.value = false;
    executionStore.setDebugMode('none');
  }
}
</script>

<template>
  <div class="designer-container">
    <!-- Toast notifications -->
    <div class="toast-container">
      <div
        v-for="toast in toasts"
        :key="toast.id"
        class="toast"
        :class="[`toast-${toast.type}`]"
      >
        {{ toast.message }}
      </div>
    </div>

    <div class="designer-toolbar">
      <div class="btn-group">
        <button class="btn btn-primary" @click="runWorkflow" :disabled="isRunning">
          <Play :size="14" />
          {{ isRunning && !isDebugging ? '执行中...' : '运行' }}
        </button>
        <div class="dropdown">
          <button class="btn" :disabled="isRunning">
            <Crosshair :size="14" />
            调试
            <ChevronDown :size="14" />
          </button>
          <div class="dropdown-menu">
            <button class="dropdown-item" @click="runDebugWorkflow('step')" :disabled="isRunning">
              <StepForward :size="14" />
              单步执行
            </button>
            <button class="dropdown-item" @click="runDebugWorkflow('breakpoint')" :disabled="isRunning">
              <Crosshair :size="14" />
              断点调试
            </button>
          </div>
        </div>
      </div>

      <!-- Debug controls (visible during debugging) -->
      <div v-if="isDebugging && isRunning" class="btn-group">
        <button class="btn btn-success" @click="stepExecution" :disabled="!isPaused">
          <ChevronRight :size="14" />
          单步
        </button>
        <button class="btn btn-warning" @click="resumeDebugExecution" :disabled="!isPaused">
          <Play :size="14" />
          继续
        </button>
        <button class="btn btn-danger" @click="() => { isRunning = false; executionStore.setDebugMode('none'); }">
          <Pause :size="14" />
          停止
        </button>
      </div>

      <span v-if="isDebugging" class="tag tag-warning">
        {{ debugMode === 'step' ? '单步调试' : '断点调试' }}
        {{ isPaused ? '(已暂停)' : '' }}
      </span>

      <button class="btn" @click="saveWorkflow" :disabled="isRunning">
        <Download :size="14" />
        保存
      </button>

      <div class="dropdown">
        <button class="btn" :disabled="isRunning">
          <FolderOpen :size="14" />
          示例
          <ChevronDown :size="14" />
        </button>
        <div class="dropdown-menu">
          <button
            v-for="example in exampleWorkflows"
            :key="example.file"
            class="dropdown-item"
            @click="loadExample(example.file)"
            :disabled="isRunning"
          >
            <FileText :size="14" />
            {{ example.name }}
          </button>
        </div>
      </div>

      <span class="workflow-name">{{ workflowStore.currentWorkflow?.name }}</span>
    </div>

    <div class="designer-body">
      <NodePalette class="designer-palette" />

      <div class="designer-canvas">
        <VueFlow
          v-model:nodes="nodes"
          v-model:edges="edges"
          fit-view-on-init
          :default-viewport="{ zoom: 1 }"
          @drop="onDrop"
          @dragover="onDragOver"
        >
          <template #node-start="nodeProps">
            <StartNode :id="nodeProps.id" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-end="nodeProps">
            <EndNode :id="nodeProps.id" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-click="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-input="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-getText="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-delay="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-log="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-setVariable="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-readFile="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-writeFile="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-condition="nodeProps">
            <ConditionNode :id="nodeProps.id" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-loop="nodeProps">
            <LoopNode :id="nodeProps.id" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-forEach="nodeProps">
            <LoopNode :id="nodeProps.id" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-tryCatch="nodeProps">
            <TryCatchNode :id="nodeProps.id" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-subflow="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-waitElement="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-openBrowser="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-navigate="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-screenshot="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-hotkey="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-webClick="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-webInput="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-webGetText="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-closeBrowser="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-executeJs="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-readExcel="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>
          <template #node-writeExcel="nodeProps">
            <ActionNode :id="nodeProps.id" :type="nodeProps.type" :data="nodeProps.data" :label="String(nodeProps.label || '')" />
          </template>

          <Background />
          <Controls />
          <MiniMap />
        </VueFlow>
      </div>

      <div class="designer-right-panel">
        <div class="panel-tabs">
          <div class="tabs-header">
            <button
              class="tab-btn"
              :class="{ active: rightPanelTab === 'properties' }"
              @click="rightPanelTab = 'properties'"
            >
              属性
            </button>
            <button
              class="tab-btn"
              :class="{ active: rightPanelTab === 'variables' }"
              @click="rightPanelTab = 'variables'"
            >
              变量
            </button>
            <button
              class="tab-btn"
              :class="{ active: rightPanelTab === 'debug' }"
              @click="rightPanelTab = 'debug'"
            >
              调试
            </button>
            <button
              class="tab-btn"
              :class="{ active: rightPanelTab === 'recorder' }"
              @click="rightPanelTab = 'recorder'"
            >
              <Video v-if="!recorderStore.isIdle" :size="14" class="recording-indicator" />
              录制
            </button>
          </div>
          <div class="tab-content">
            <PropertyPanel v-show="rightPanelTab === 'properties'" />
            <VariablePanel v-show="rightPanelTab === 'variables'" />
            <DebugPanel v-show="rightPanelTab === 'debug'" />
            <RecorderPanel v-show="rightPanelTab === 'recorder'" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.designer-container {
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
}

.toast-container {
  position: fixed;
  top: 80px;
  right: 20px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.toast {
  padding: 12px 20px;
  border-radius: 6px;
  font-size: 14px;
  color: #fff;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  animation: slideIn 0.3s ease;
}

.toast-success {
  background: #22c55e;
}

.toast-error {
  background: #ef4444;
}

.toast-warning {
  background: #f59e0b;
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.designer-toolbar {
  height: 48px;
  padding: 8px 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--bg-color);
}

.btn-group {
  display: flex;
  gap: 0;
}

.btn-group .btn {
  border-radius: 0;
}

.btn-group .btn:first-child {
  border-radius: 6px 0 0 6px;
}

.btn-group .btn:last-child,
.btn-group .dropdown:last-child .btn {
  border-radius: 0 6px 6px 0;
}

.btn-group .dropdown .btn {
  border-radius: 0;
}

.dropdown {
  position: relative;
}

.dropdown-menu {
  position: absolute;
  top: 100%;
  left: 0;
  margin-top: 4px;
  min-width: 150px;
  background: #fff;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  z-index: 100;
  display: none;
}

.dropdown:hover .dropdown-menu,
.dropdown:focus-within .dropdown-menu {
  display: block;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  background: transparent;
  font-size: 14px;
  color: #374151;
  cursor: pointer;
  text-align: left;
}

.dropdown-item:hover:not(:disabled) {
  background: #f3f4f6;
}

.dropdown-item:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-success {
  background: #22c55e;
  color: #fff;
}

.btn-success:hover:not(:disabled) {
  background: #16a34a;
}

.btn-warning {
  background: #f59e0b;
  color: #fff;
}

.btn-warning:hover:not(:disabled) {
  background: #d97706;
}

.btn-danger {
  background: #ef4444;
  color: #fff;
}

.btn-danger:hover:not(:disabled) {
  background: #dc2626;
}

.tag-warning {
  background: #fef3c7;
  color: #92400e;
  border-color: #fcd34d;
}

.workflow-name {
  color: var(--text-secondary);
  font-size: 14px;
  margin-left: auto;
}

.designer-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.designer-palette {
  width: 200px;
  border-right: 1px solid var(--border-color);
  background: var(--bg-color);
}

.designer-canvas {
  flex: 1;
  position: relative;
}

.designer-right-panel {
  width: 280px;
  border-left: 1px solid var(--border-color);
  background: var(--bg-color);
  display: flex;
  flex-direction: column;
}

.panel-tabs {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.tabs-header {
  display: flex;
  border-bottom: 1px solid var(--border-color);
  background: #f9fafb;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 10px 16px;
  border: none;
  background: transparent;
  font-size: 14px;
  color: #6b7280;
  cursor: pointer;
  border-bottom: 2px solid transparent;
  margin-bottom: -1px;
}

.tab-btn:hover {
  color: #374151;
}

.tab-btn.active {
  color: #3b82f6;
  border-bottom-color: #3b82f6;
  background: #fff;
}

.tab-content {
  flex: 1;
  overflow: hidden;
}

.vue-flow {
  height: 100%;
}

.recording-indicator {
  color: #ef4444;
  animation: pulse 1s infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.4;
  }
}
</style>

<style>
@import '@vue-flow/core/dist/style.css';
@import '@vue-flow/core/dist/theme-default.css';
@import '@vue-flow/controls/dist/style.css';
@import '@vue-flow/minimap/dist/style.css';
</style>
