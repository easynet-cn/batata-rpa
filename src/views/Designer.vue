<script setup lang="ts">
import { onMounted, ref, computed, watch } from 'vue';
import { VueFlow, useVueFlow } from '@vue-flow/core';
import type { Node, Edge } from '@vue-flow/core';
import { Background } from '@vue-flow/background';
import { Controls } from '@vue-flow/controls';
import { MiniMap } from '@vue-flow/minimap';
import { VideoPlay, Download, CaretRight, VideoPause, SwitchFilled, Aim, ArrowDown, VideoCamera, FolderOpened, Document } from '@element-plus/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
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
    ElMessage.success('流程开始执行');

    // Poll execution state
    pollExecutionState(workflowStore.currentWorkflow.id);
  } catch (error) {
    ElMessage.error(`执行失败: ${error}`);
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
        ElMessage.success('流程执行完成');
        executionStore.stopExecution();
      } else if (typedState.status === 'Failed') {
        ElMessage.error('流程执行失败');
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
    ElMessage.success('示例流程加载成功');
  } catch (error) {
    ElMessage.error(`加载示例失败: ${error}`);
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
    ElMessage.success(`调试模式开始: ${mode === 'step' ? '单步执行' : '断点调试'}`);

    // Poll execution state with variable updates
    pollDebugExecutionState(workflowStore.currentWorkflow.id);
  } catch (error) {
    ElMessage.error(`调试失败: ${error}`);
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
    ElMessage.error(`单步执行失败: ${error}`);
  }
}

async function resumeDebugExecution() {
  if (!workflowStore.currentWorkflow) return;

  try {
    await invoke('resume_execution', { workflowId: workflowStore.currentWorkflow.id });
    executionStore.resumeExecution();
  } catch (error) {
    ElMessage.error(`继续执行失败: ${error}`);
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
        ElMessage.success('调试执行完成');
        executionStore.stopExecution();
      } else if (typedState.status === 'Failed') {
        ElMessage.error('调试执行失败');
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
    <div class="designer-toolbar">
      <el-button-group>
        <el-button type="primary" @click="runWorkflow" :loading="isRunning && !isDebugging" :disabled="isRunning">
          <el-icon><VideoPlay /></el-icon>
          运行
        </el-button>
        <el-dropdown @command="runDebugWorkflow" :disabled="isRunning">
          <el-button :disabled="isRunning">
            <el-icon><Aim /></el-icon>
            调试
            <el-icon class="el-icon--right"><ArrowDown /></el-icon>
          </el-button>
          <template #dropdown>
            <el-dropdown-menu>
              <el-dropdown-item command="step">
                <el-icon><SwitchFilled /></el-icon>
                单步执行
              </el-dropdown-item>
              <el-dropdown-item command="breakpoint">
                <el-icon><Aim /></el-icon>
                断点调试
              </el-dropdown-item>
            </el-dropdown-menu>
          </template>
        </el-dropdown>
      </el-button-group>

      <!-- Debug controls (visible during debugging) -->
      <el-button-group v-if="isDebugging && isRunning">
        <el-button type="success" @click="stepExecution" :disabled="!isPaused">
          <el-icon><CaretRight /></el-icon>
          单步
        </el-button>
        <el-button type="warning" @click="resumeDebugExecution" :disabled="!isPaused">
          <el-icon><VideoPlay /></el-icon>
          继续
        </el-button>
        <el-button type="danger" @click="() => { isRunning = false; executionStore.setDebugMode('none'); }">
          <el-icon><VideoPause /></el-icon>
          停止
        </el-button>
      </el-button-group>

      <el-tag v-if="isDebugging" type="warning" class="debug-tag">
        {{ debugMode === 'step' ? '单步调试' : '断点调试' }}
        {{ isPaused ? '(已暂停)' : '' }}
      </el-tag>

      <el-button @click="saveWorkflow" :disabled="isRunning">
        <el-icon><Download /></el-icon>
        保存
      </el-button>

      <el-dropdown @command="loadExample" :disabled="isRunning">
        <el-button :disabled="isRunning">
          <el-icon><FolderOpened /></el-icon>
          示例
          <el-icon class="el-icon--right"><ArrowDown /></el-icon>
        </el-button>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item
              v-for="example in exampleWorkflows"
              :key="example.file"
              :command="example.file"
            >
              <el-icon><Document /></el-icon>
              {{ example.name }}
            </el-dropdown-item>
          </el-dropdown-menu>
        </template>
      </el-dropdown>

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
        <el-tabs v-model="rightPanelTab" class="panel-tabs">
          <el-tab-pane label="属性" name="properties">
            <PropertyPanel />
          </el-tab-pane>
          <el-tab-pane label="变量" name="variables">
            <VariablePanel />
          </el-tab-pane>
          <el-tab-pane label="调试" name="debug">
            <DebugPanel />
          </el-tab-pane>
          <el-tab-pane name="recorder">
            <template #label>
              <span class="recorder-tab-label">
                <el-icon v-if="!recorderStore.isIdle" class="recording-indicator"><VideoCamera /></el-icon>
                录制
              </span>
            </template>
            <RecorderPanel />
          </el-tab-pane>
        </el-tabs>
      </div>
    </div>
  </div>
</template>

<style scoped>
.designer-container {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.designer-toolbar {
  height: 48px;
  padding: 8px 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  border-bottom: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
}

.workflow-name {
  color: var(--el-text-color-secondary);
  font-size: 14px;
}

.designer-body {
  flex: 1;
  display: flex;
  overflow: hidden;
}

.designer-palette {
  width: 200px;
  border-right: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
}

.designer-canvas {
  flex: 1;
  position: relative;
}

.designer-right-panel {
  width: 280px;
  border-left: 1px solid var(--el-border-color);
  background: var(--el-bg-color);
  display: flex;
  flex-direction: column;
}

.panel-tabs {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.panel-tabs :deep(.el-tabs__content) {
  flex: 1;
  overflow: hidden;
}

.panel-tabs :deep(.el-tab-pane) {
  height: 100%;
}

.vue-flow {
  height: 100%;
}

.debug-tag {
  margin-left: 8px;
}

.recorder-tab-label {
  display: flex;
  align-items: center;
  gap: 4px;
}

.recording-indicator {
  color: var(--el-color-danger);
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
