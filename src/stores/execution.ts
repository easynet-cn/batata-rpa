import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { ExecutionState, ExecutionLog } from '@/types';

export type DebugMode = 'none' | 'step' | 'breakpoint';

export const useExecutionStore = defineStore('execution', () => {
  const state = ref<ExecutionState | null>(null);
  const breakpoints = ref<Set<string>>(new Set());
  const debugMode = ref<DebugMode>('none');

  const isRunning = computed(() => state.value?.status === 'running');
  const isPaused = computed(() => state.value?.status === 'paused');
  const isDebugging = computed(() => debugMode.value !== 'none');

  function startExecution(workflowId: string) {
    state.value = {
      workflowId,
      status: 'running',
      startTime: new Date().toISOString(),
      logs: [],
      variables: {},
    };
    addLog('info', '流程开始执行');
  }

  function pauseExecution() {
    if (state.value) {
      state.value.status = 'paused';
      addLog('info', '流程已暂停');
    }
  }

  function resumeExecution() {
    if (state.value) {
      state.value.status = 'running';
      addLog('info', '流程继续执行');
    }
  }

  function stopExecution() {
    if (state.value) {
      state.value.status = 'completed';
      state.value.endTime = new Date().toISOString();
      addLog('info', '流程执行完成');
    }
  }

  function failExecution(error: string) {
    if (state.value) {
      state.value.status = 'failed';
      state.value.endTime = new Date().toISOString();
      state.value.error = error;
      addLog('error', `流程执行失败: ${error}`);
    }
  }

  function setCurrentNode(nodeId: string) {
    if (state.value) {
      state.value.currentNodeId = nodeId;
    }
  }

  function setVariable(name: string, value: unknown) {
    if (state.value) {
      state.value.variables[name] = value;
    }
  }

  function addLog(
    level: ExecutionLog['level'],
    message: string,
    nodeId?: string,
    details?: unknown
  ) {
    if (state.value) {
      state.value.logs.push({
        id: crypto.randomUUID(),
        timestamp: new Date().toISOString(),
        level,
        nodeId,
        message,
        details,
      });
    }
  }

  function clearState() {
    state.value = null;
  }

  // Debug methods
  function setDebugMode(mode: DebugMode) {
    debugMode.value = mode;
  }

  function toggleBreakpoint(nodeId: string) {
    if (breakpoints.value.has(nodeId)) {
      breakpoints.value.delete(nodeId);
    } else {
      breakpoints.value.add(nodeId);
    }
  }

  function hasBreakpoint(nodeId: string): boolean {
    return breakpoints.value.has(nodeId);
  }

  function clearBreakpoints() {
    breakpoints.value.clear();
  }

  function updateVariables(variables: Record<string, unknown>) {
    if (state.value) {
      state.value.variables = variables;
    }
  }

  return {
    state,
    breakpoints,
    debugMode,
    isRunning,
    isPaused,
    isDebugging,
    startExecution,
    pauseExecution,
    resumeExecution,
    stopExecution,
    failExecution,
    setCurrentNode,
    setVariable,
    addLog,
    clearState,
    setDebugMode,
    toggleBreakpoint,
    hasBreakpoint,
    clearBreakpoints,
    updateVariables,
  };
});
