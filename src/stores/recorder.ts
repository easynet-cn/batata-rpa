import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export type RecordingState = 'Idle' | 'Recording' | 'Paused';

export interface RecordedAction {
  id: string;
  action_type: string;
  timestamp: number;
  element?: {
    id: string;
    name: string;
    control_type: string;
    bounds: { x: number; y: number; width: number; height: number };
  };
  position?: [number, number];
  data: Record<string, any>;
}

export interface RecordingSession {
  id: string;
  name: string;
  state: RecordingState;
  actions: RecordedAction[];
  started_at?: string;
  ended_at?: string;
}

export interface RecordingStatus {
  state: RecordingState;
  action_count: number;
  duration_ms: number;
}

export const useRecorderStore = defineStore('recorder', () => {
  const state = ref<RecordingState>('Idle');
  const session = ref<RecordingSession | null>(null);
  const actionCount = ref(0);
  const durationMs = ref(0);
  const error = ref<string | null>(null);

  let statusInterval: ReturnType<typeof setInterval> | null = null;

  const isRecording = computed(() => state.value === 'Recording');
  const isPaused = computed(() => state.value === 'Paused');
  const isIdle = computed(() => state.value === 'Idle');

  const formattedDuration = computed(() => {
    const totalSeconds = Math.floor(durationMs.value / 1000);
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`;
  });

  async function startRecording(name?: string) {
    try {
      error.value = null;
      await invoke('start_recording', { name });
      state.value = 'Recording';
      startStatusPolling();
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function pauseRecording() {
    try {
      error.value = null;
      await invoke('pause_recording');
      state.value = 'Paused';
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function resumeRecording() {
    try {
      error.value = null;
      await invoke('resume_recording');
      state.value = 'Recording';
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function stopRecording(): Promise<RecordingSession> {
    try {
      error.value = null;
      stopStatusPolling();
      const result = await invoke<RecordingSession>('stop_recording');
      session.value = result;
      state.value = 'Idle';
      return result;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function fetchStatus() {
    try {
      const status = await invoke<RecordingStatus>('get_recording_state');
      state.value = status.state;
      actionCount.value = status.action_count;
      durationMs.value = status.duration_ms;
    } catch (e) {
      console.error('Failed to fetch recording status:', e);
    }
  }

  async function fetchSession() {
    try {
      const result = await invoke<RecordingSession>('get_recording_session');
      session.value = result;
      return result;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function convertToWorkflow() {
    try {
      error.value = null;
      const workflow = await invoke('convert_recording_to_workflow');
      return workflow;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  async function clearRecording() {
    try {
      error.value = null;
      await invoke('clear_recording');
      session.value = null;
      actionCount.value = 0;
      durationMs.value = 0;
    } catch (e) {
      error.value = String(e);
      throw e;
    }
  }

  function startStatusPolling() {
    if (statusInterval) {
      clearInterval(statusInterval);
    }
    statusInterval = setInterval(fetchStatus, 500);
  }

  function stopStatusPolling() {
    if (statusInterval) {
      clearInterval(statusInterval);
      statusInterval = null;
    }
  }

  return {
    state,
    session,
    actionCount,
    durationMs,
    error,
    isRecording,
    isPaused,
    isIdle,
    formattedDuration,
    startRecording,
    pauseRecording,
    resumeRecording,
    stopRecording,
    fetchStatus,
    fetchSession,
    convertToWorkflow,
    clearRecording,
  };
});
